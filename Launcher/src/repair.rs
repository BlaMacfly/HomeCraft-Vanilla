//! Vérification et réparation : manifeste de fichiers gérés (Build/files-manifest.json, produit par le
//! développeur), données extraites du client, extracteurs CMaNGOS (vue des archives officielles), export de diagnostic.

use crate::db::CREATE_NO_WINDOW;
use crate::paths::Layout;
use crate::util;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilesManifest {
    pub generated: String,
    pub files: Vec<FileEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct VerifyReport {
    pub manifest_present: bool,
    pub checked: usize,
    pub missing: Vec<String>,
    pub altered: Vec<String>,
    pub data: Vec<(String, bool)>, // dbc / maps / vmaps / mmaps
    pub db_ok: Option<bool>,
    pub notes: Vec<String>,
}

impl VerifyReport {
    pub fn files_ok(&self) -> bool {
        self.missing.is_empty() && self.altered.is_empty()
    }
    pub fn data_ok(&self) -> bool {
        self.data.iter().filter(|(n, _)| n != "mmaps").all(|(_, ok)| *ok)
    }
}

pub fn manifest_path(layout: &Layout) -> PathBuf {
    layout.root.join("Build").join("files-manifest.json")
}

pub fn load_manifest(layout: &Layout) -> Option<FilesManifest> {
    let txt = std::fs::read_to_string(manifest_path(layout)).ok()?;
    serde_json::from_str(txt.trim_start_matches('\u{feff}')).ok()
}

/// Compare les fichiers gérés au manifeste (taille puis SHA-256).
pub fn verify_files(layout: &Layout, manifest: &FilesManifest, progress: &dyn Fn(usize, usize)) -> (Vec<String>, Vec<String>) {
    let mut missing = Vec::new();
    let mut altered = Vec::new();
    let total = manifest.files.len();
    for (i, f) in manifest.files.iter().enumerate() {
        progress(i + 1, total);
        let p = layout.root.join(&f.path);
        match std::fs::metadata(&p) {
            Err(_) => missing.push(f.path.clone()),
            Ok(m) => {
                if m.len() != f.size || util::sha256_file(&p).map(|h| h != f.sha256).unwrap_or(true) {
                    altered.push(f.path.clone());
                }
            }
        }
    }
    (missing, altered)
}

/// Présence des données extraites du client. Seuils minimaux, relevés sur le client 2.4.3 frFR de l'utilisateur
/// (2026-10-17) : 185 DBC, 3 586 cartes, 8 607 fichiers vmaps.
pub fn data_status(layout: &Layout) -> Vec<(String, bool)> {
    let d = layout.server_data();
    vec![
        ("dbc".into(), util::dir_file_count(&d.join("dbc")) >= 180),
        ("maps".into(), util::dir_file_count(&d.join("maps")) >= 3000),
        ("vmaps".into(), util::dir_file_count(&d.join("vmaps")) >= 5000),
        // Une génération interrompue laisse le marqueur en place : les mmaps sont alors considérées absentes et régénérées.
        ("mmaps".into(), util::dir_file_count(&d.join("mmaps")) >= 1000 && !mmaps_marker(layout).exists()),
    ]
}

/// Marqueur « génération des mmaps en cours » (créé au lancement de MoveMapGen, retiré à la fin).
pub fn mmaps_marker(layout: &Layout) -> PathBuf {
    layout.server_data().join("mmaps").join(".generation-en-cours")
}

/// Arguments de MoveMapGen, transposés de MoveMapGen.sh (cas « maps » : toutes les cartes + objets du monde).
fn mmaps_args(layout: &Layout, threads: usize) -> Vec<String> {
    let ext = layout.extractors();
    let q = |p: PathBuf| p.to_string_lossy().into_owned();
    vec![
        "--silent".into(),
        "--configInputPath".into(),
        q(ext.join("config.json")),
        "--offMeshInput".into(),
        q(ext.join("offmesh.txt")),
        "--workdir".into(),
        // Barres obliques et « / » final : un antislash final échapperait le guillemet fermant (ligne de commande Windows).
        format!("{}/", layout.server_data().display().to_string().replace('\\', "/")),
        "--threads".into(),
        threads.to_string(),
        "--buildGameObjects".into(),
    ]
}

/// Lance MoveMapGen sans attendre (génération en arrière-plan pendant que le serveur tourne).
/// L'appelant surveille le processus, puis appelle `mmaps_finished`.
pub fn spawn_mmaps(layout: &Layout, threads: usize, sink: &Sender<String>) -> Result<std::process::Child, String> {
    let data = layout.server_data();
    std::fs::create_dir_all(data.join("mmaps")).map_err(|e| e.to_string())?;
    std::fs::write(mmaps_marker(layout), "generation en cours").map_err(|e| e.to_string())?;
    spawn_tool("MoveMapGen", &layout.extractors().join("MoveMapGen.exe"), &mmaps_args(layout, threads), &data, sink)
}

/// À appeler quand MoveMapGen s'est terminé : retire le marqueur si la génération a réussi.
pub fn mmaps_finished(layout: &Layout, success: bool) {
    if success {
        let _ = std::fs::remove_file(mmaps_marker(layout));
    }
}

/// Longueur de chemin au-delà de laquelle ad.exe ne trouve plus la locale (« No locales detected », constaté avec
/// ~150 caractères ; 43 caractères fonctionnent). Marge prise à 100.
const MAX_VIEW_PATH: usize = 100;

/// Vue « archives officielles seules » du client : dossier contenant des liens physiques vers WoW.exe et les seules
/// archives officielles (client::is_official_mpq). Les extracteurs CMaNGOS chargent TOUS les patch-*.MPQ présents :
/// un patch tiers (patch-frFR-3.MPQ d'un serveur privé, qui modifie Spell.dbc et 4 autres DBC) passerait sinon
/// dans les données du serveur et ne correspondrait plus à TBC-DB. Le client n'est jamais modifié.
/// Emplacement : UserData\extract-view (même volume que le client et chemin court), sinon <lecteur du client>:\HCBC-extract.
fn build_official_view(layout: &Layout, client_dir: &Path, progress: &dyn Fn(&str)) -> Result<PathBuf, String> {
    let fallback = client_dir.components().next().map(|c| PathBuf::from(format!("{}\\", c.as_os_str().to_string_lossy()))).unwrap_or_default().join("HCBC-extract");
    let mut candidates = vec![layout.extract_view(), fallback];
    candidates.retain(|p| p.to_string_lossy().len() <= MAX_VIEW_PATH);
    let mut last_err = String::from("aucun emplacement court disponible pour la vue d'extraction");
    for view in candidates {
        let _ = std::fs::remove_dir_all(&view);
        let attempt = (|| -> Result<(), String> {
            std::fs::create_dir_all(view.join("Data").join("frFR")).map_err(|e| e.to_string())?;
            std::fs::hard_link(client_dir.join("WoW.exe"), view.join("WoW.exe")).map_err(|e| format!("WoW.exe : {e}"))?;
            for rel in crate::client::REQUIRED_MPQ.iter().chain(crate::client::OPTIONAL_OFFICIAL_MPQ.iter()) {
                let src = client_dir.join("Data").join(rel);
                if src.is_file() {
                    std::fs::hard_link(&src, view.join("Data").join(rel)).map_err(|e| format!("{rel} : {e}"))?;
                }
            }
            Ok(())
        })();
        match attempt {
            Ok(()) => return Ok(view),
            Err(e) => {
                let _ = std::fs::remove_dir_all(&view);
                last_err = format!("{} : {e}", view.display());
            }
        }
    }
    progress("vue d'extraction impossible");
    Err(last_err)
}

/// Archives du client ignorées par l'extraction (non officielles), pour le journal.
pub fn unofficial_archives(client_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for (sub, prefix) in [(client_dir.join("Data"), ""), (client_dir.join("Data").join("frFR"), "frFR/")] {
        if let Ok(rd) = std::fs::read_dir(sub) {
            for e in rd.filter_map(Result::ok) {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.to_ascii_lowercase().ends_with(".mpq") && !crate::client::is_official_mpq(&format!("{prefix}{name}")) {
                    out.push(format!("{prefix}{name}"));
                }
            }
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExtractStep {
    DbcMaps,
    Vmaps,
    Mmaps,
}

fn run_tool(name: &str, exe: &Path, args: &[String], cwd: &Path, sink: &Sender<String>) -> Result<(), String> {
    let mut child = spawn_tool(name, exe, args, cwd, sink)?;
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("{name} a échoué (code {:?})", status.code())) }
}

fn spawn_tool(name: &str, exe: &Path, args: &[String], cwd: &Path, sink: &Sender<String>) -> Result<std::process::Child, String> {
    let mut child = Command::new(exe)
        .args(args)
        .current_dir(cwd)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{name} : {e}"))?;
    if let Some(out) = child.stdout.take() {
        let sink = sink.clone();
        let name = name.to_string();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(out);
            let mut buf = Vec::new();
            let mut last = std::time::Instant::now() - std::time::Duration::from_secs(10);
            loop {
                buf.clear();
                match reader.read_until(b'\n', &mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        // Les extracteurs écrivent des barres de progression très verbeuses : une ligne toutes les 5 s suffit.
                        if last.elapsed() >= std::time::Duration::from_secs(5) {
                            let line = String::from_utf8_lossy(&buf).trim().to_string();
                            if !line.is_empty() {
                                let _ = sink.send(format!("[{name}] {line}"));
                                last = std::time::Instant::now();
                            }
                        }
                    }
                }
            }
        });
    }
    Ok(child)
}

/// Extraction des données du client avec les outils CMaNGOS (Server/mangos-tbc/Extractors), mêmes étapes que
/// Tools/Extract-ClientData.ps1 (transposition de ExtractResources.sh, basse résolution) sur la vue officielle.
pub fn extract(layout: &Layout, client_dir: &Path, steps: &[ExtractStep], sink: &Sender<String>, progress: &dyn Fn(&str)) -> Result<(), String> {
    let ext = layout.extractors();
    let data = layout.server_data();
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let q = |p: &Path| p.to_string_lossy().into_owned();
    let needs_client = steps.contains(&ExtractStep::DbcMaps) || steps.contains(&ExtractStep::Vmaps);
    let view = if needs_client {
        let ignored = unofficial_archives(client_dir);
        if !ignored.is_empty() {
            let _ = sink.send(format!("[extraction] archives non officielles ignorées : {}", ignored.join(", ")));
        }
        Some(build_official_view(layout, client_dir, progress)?)
    } else {
        None
    };
    let result = (|| -> Result<(), String> {
        if let Some(view) = view.as_ref() {
            if steps.contains(&ExtractStep::DbcMaps) {
                progress("extraction des DBC et des cartes");
                run_tool("ad", &ext.join("ad.exe"), &["-i".into(), q(view), "-o".into(), q(&data)], &data, sink)?;
                // ad.exe sort avec le code 0 même quand il n'a rien extrait (« No locales detected ») : contrôle explicite.
                if util::dir_file_count(&data.join("dbc")) < 180 {
                    return Err("ad.exe n'a pas extrait les DBC (locale du client non détectée ?)".into());
                }
            }
            if steps.contains(&ExtractStep::Vmaps) {
                progress("extraction des modèles de collision (vmaps)");
                run_tool("vmap_extractor", &ext.join("vmap_extractor.exe"), &["-d".into(), q(&view.join("Data")), "-o".into(), q(&data)], &data, sink)?;
                std::fs::create_dir_all(data.join("vmaps")).map_err(|e| e.to_string())?;
                progress("assemblage des vmaps");
                run_tool("vmap_assembler", &ext.join("vmap_assembler.exe"), &[q(&data.join("Buildings")), q(&data.join("vmaps"))], &data, sink)?;
            }
        }
        if steps.contains(&ExtractStep::Mmaps) {
            progress("génération des cartes de déplacement (mmaps, long)");
            // Génération bloquante (l'utilisateur attend) : tous les cœurs sauf deux. Marqueur posé pendant la génération,
            // retiré seulement en cas de succès : une génération interrompue est refaite au démarrage suivant.
            let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).saturating_sub(2).max(1);
            std::fs::create_dir_all(data.join("mmaps")).map_err(|e| e.to_string())?;
            std::fs::write(mmaps_marker(layout), "generation en cours").map_err(|e| e.to_string())?;
            let r = run_tool("MoveMapGen", &ext.join("MoveMapGen.exe"), &mmaps_args(layout, threads), &data, sink);
            mmaps_finished(layout, r.is_ok());
            r?;
        }
        Ok(())
    })();
    if let Some(view) = view {
        let _ = std::fs::remove_dir_all(view); // liens seulement : le client n'est pas touché
    }
    result
}

/// Archive de diagnostic : journaux + configurations sans mots de passe + état des fichiers.
pub fn export_diagnostic(layout: &Layout, report: Option<&VerifyReport>) -> Result<PathBuf, String> {
    std::fs::create_dir_all(layout.user_data()).map_err(|e| e.to_string())?;
    let out = layout.user_data().join(format!("diagnostic-{}.zip", util::datetime_compact()));
    let f = std::fs::File::create(&out).map_err(|e| e.to_string())?;
    let mut w = zip::ZipWriter::new(f);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut add_file = |name: &str, p: &Path, redact: bool| -> Result<(), String> {
        if !p.is_file() {
            return Ok(());
        }
        w.start_file(name, opts).map_err(|e| e.to_string())?;
        if redact {
            let txt = std::fs::read_to_string(p).unwrap_or_default();
            let redacted: Vec<String> = txt.lines().map(|l| if l.contains("Database") && l.contains(';') { let k = l.split('=').next().unwrap_or("").trim(); format!("{k} = \"<masqué>\"") } else { l.to_string() }).collect();
            w.write_all(redacted.join("\n").as_bytes()).map_err(|e| e.to_string())?;
        } else {
            let mut src = std::fs::File::open(p).map_err(|e| e.to_string())?;
            std::io::copy(&mut src, &mut w).map_err(|e| e.to_string())?;
        }
        Ok(())
    };
    add_file("launcher.log", &layout.launcher_log(), false)?;
    add_file("mariadb-error.log", &layout.logs().join("mariadb-error.log"), false)?;
    for n in ["mangosd.console.log", "Server.log", "DBErrors.log", "realmd.stdout.log", "Realmd.log"] {
        add_file(n, &layout.server_logs().join(n), false)?;
    }
    add_file("mangosd.conf", &layout.server_config().join("mangosd.conf"), true)?;
    add_file("realmd.conf", &layout.server_config().join("realmd.conf"), true)?;
    add_file("aiplayerbot.conf", &layout.server_config().join("aiplayerbot.conf"), true)?;
    // Rapports de plantage de mangosd (écrits dans son dossier courant) : les 3 plus récents (.txt et .dmp).
    if let Ok(rd) = std::fs::read_dir(layout.server_bin().join("Crashes")) {
        let mut crashes: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_file()).collect();
        crashes.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
        for p in crashes.iter().rev().take(6) {
            let name = format!("Crashes/{}", p.file_name().unwrap_or_default().to_string_lossy());
            add_file(&name, p, false)?;
        }
    }
    add_file("ahbot.conf", &layout.server_config().join("ahbot.conf"), true)?;
    w.start_file("etat.txt", opts).map_err(|e| e.to_string())?;
    let mut etat = String::new();
    etat.push_str(&format!("date : {}\nracine : {}\n", util::datetime_readable(), layout.root.display()));
    for (n, ok) in layout.components_present() {
        etat.push_str(&format!("{n} : {}\n", if ok { "présent" } else { "ABSENT" }));
    }
    for (n, ok) in data_status(layout) {
        etat.push_str(&format!("données {n} : {}\n", if ok { "OK" } else { "manquantes" }));
    }
    if let Some(r) = report {
        etat.push_str(&format!("manifeste : {}\nfichiers vérifiés : {}\nmanquants : {:?}\naltérés : {:?}\n", r.manifest_present, r.checked, r.missing, r.altered));
    }
    w.write_all(etat.as_bytes()).map_err(|e| e.to_string())?;
    w.finish().map_err(|e| e.to_string())?;
    Ok(out)
}
