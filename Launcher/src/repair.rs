//! Vérification et réparation : manifeste de fichiers gérés (Build/files-manifest.json, produit par le
//! développeur), données extraites du client, extracteurs AzerothCore, export de diagnostic.

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

/// Identifiant maximal d'un fichier DBC (première colonne de chaque enregistrement).
pub fn dbc_max_id(path: &Path) -> Option<u32> {
    let b = std::fs::read(path).ok()?;
    if b.len() < 20 || &b[0..4] != b"WDBC" {
        return None;
    }
    let rc = u32::from_le_bytes(b[4..8].try_into().ok()?) as usize;
    let rs = u32::from_le_bytes(b[12..16].try_into().ok()?) as usize;
    let mut max = 0u32;
    for i in 0..rc {
        let o = 20 + i * rs;
        if o + 4 > b.len() {
            break;
        }
        max = max.max(u32::from_le_bytes(b[o..o + 4].try_into().ok()?));
    }
    Some(max)
}

/// Les DBC sont-ils bien au niveau 3.3.5a ? (Map.dbc doit contenir 724 Ruby Sanctum, AreaTable.dbc 4987 : certains
/// clients frFR « 3.3.5a » recomposés embarquent des DBC 3.3.3a et le serveur refuse de démarrer.)
pub fn dbc_up_to_date(layout: &Layout) -> Option<bool> {
    let d = layout.server_data().join("dbc");
    let map = dbc_max_id(&d.join("Map.dbc"))?;
    let area = dbc_max_id(&d.join("AreaTable.dbc"))?;
    Some(map >= 724 && area >= 4987)
}

/// Présence des données extraites du client (seuils : 3.3.5a compte 246 DBC, 5 703 tuiles de cartes, 12 465 vmaps, 3 738 mmaps).
pub fn data_status(layout: &Layout) -> Vec<(String, bool)> {
    let d = layout.server_data();
    vec![
        ("dbc".into(), util::dir_file_count(&d.join("dbc")) >= 240 && dbc_up_to_date(layout).unwrap_or(false)),
        ("maps".into(), util::dir_file_count(&d.join("maps")) >= 5000),
        ("vmaps".into(), util::dir_file_count(&d.join("vmaps")) >= 10_000),
        // Une génération interrompue laisse le marqueur en place : les mmaps sont alors considérées absentes et régénérées.
        ("mmaps".into(), util::dir_file_count(&d.join("mmaps")) >= 3000 && !mmaps_marker(layout).exists()),
    ]
}

/// Marqueur « génération des mmaps en cours » (créé au lancement de mmaps_generator, retiré à la fin).
pub fn mmaps_marker(layout: &Layout) -> PathBuf {
    layout.server_data().join("mmaps").join(".generation-en-cours")
}

/// Lance mmaps_generator sans attendre (génération en arrière-plan pendant que le serveur tourne).
pub fn spawn_mmaps(layout: &Layout, threads: usize, sink: &Sender<String>) -> Result<std::process::Child, String> {
    let bin = layout.server_bin();
    let data = layout.server_data();
    std::fs::create_dir_all(data.join("mmaps")).map_err(|e| e.to_string())?;
    std::fs::write(mmaps_marker(layout), "generation en cours").map_err(|e| e.to_string())?;
    spawn_tool("mmaps_generator", &bin.join("mmaps_generator.exe"), &["--threads".into(), threads.to_string(), "--silent".into()], &data, sink)
}

/// À appeler quand mmaps_generator s'est terminé : retire le marqueur si la génération a réussi.
pub fn mmaps_finished(layout: &Layout, success: bool) {
    if success {
        let _ = std::fs::remove_file(mmaps_marker(layout));
    }
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
    if !exe.is_file() {
        return Err(format!("{name} : {} introuvable", exe.display()));
    }
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

/// Extraction des données du client 3.3.5a avec les outils AzerothCore (mêmes étapes que Tools/Extract-ClientData.ps1) :
/// `map_extractor -i <client> -o <data> -e 7` (dbc, maps, Cameras), `vmap4_extractor -l -d <client>\Data` puis
/// `vmap4_assembler Buildings vmaps`, `mmaps_generator`. Les DBC fournis séparément (`dbc/` déjà au niveau 3.3.5a) sont conservés.
pub fn extract(layout: &Layout, client_dir: &Path, steps: &[ExtractStep], sink: &Sender<String>, progress: &dyn Fn(&str)) -> Result<(), String> {
    let bin = layout.server_bin();
    let data = layout.server_data();
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let q = |p: &Path| p.to_string_lossy().into_owned();
    if steps.contains(&ExtractStep::DbcMaps) {
        progress("extraction des DBC, des cartes et des caméras");
        let keep_dbc = dbc_up_to_date(layout) == Some(true);
        let backup = data.join("dbc.ok");
        if keep_dbc {
            let _ = std::fs::remove_dir_all(&backup);
            std::fs::rename(data.join("dbc"), &backup).map_err(|e| e.to_string())?;
        }
        run_tool("map_extractor", &bin.join("map_extractor.exe"), &["-i".into(), q(client_dir), "-o".into(), q(&data), "-e".into(), "7".into()], &data, sink)?;
        if keep_dbc {
            let _ = std::fs::remove_dir_all(data.join("dbc"));
            std::fs::rename(&backup, data.join("dbc")).map_err(|e| e.to_string())?;
        } else if dbc_up_to_date(layout) == Some(false) {
            return Err(crate::i18n::fr::MSG_DBC_OUTDATED.into());
        }
    }
    if steps.contains(&ExtractStep::Vmaps) {
        progress("extraction des modèles de collision (vmaps)");
        run_tool("vmap4_extractor", &bin.join("vmap4_extractor.exe"), &["-l".into(), "-d".into(), q(&client_dir.join("Data"))], &data, sink)?;
        std::fs::create_dir_all(data.join("vmaps")).map_err(|e| e.to_string())?;
        progress("assemblage des vmaps");
        run_tool("vmap4_assembler", &bin.join("vmap4_assembler.exe"), &["Buildings".into(), "vmaps".into()], &data, sink)?;
        let _ = std::fs::remove_dir_all(data.join("Buildings"));
    }
    if steps.contains(&ExtractStep::Mmaps) {
        progress("génération des cartes de déplacement (mmaps, ~20 min)");
        std::fs::create_dir_all(data.join("mmaps")).map_err(|e| e.to_string())?;
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).to_string();
        run_tool("mmaps_generator", &bin.join("mmaps_generator.exe"), &["--threads".into(), threads, "--silent".into()], &data, sink)?;
    }
    Ok(())
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
            let redacted: Vec<String> = txt.lines().map(|l| if l.contains("DatabaseInfo") && l.contains(';') { let k = l.split('=').next().unwrap_or("").trim(); format!("{k} = \"<masqué>\"") } else { l.to_string() }).collect();
            w.write_all(redacted.join("\n").as_bytes()).map_err(|e| e.to_string())?;
        } else {
            let mut src = std::fs::File::open(p).map_err(|e| e.to_string())?;
            std::io::copy(&mut src, &mut w).map_err(|e| e.to_string())?;
        }
        Ok(())
    };
    add_file("launcher.log", &layout.launcher_log(), false)?;
    add_file("mysql-error.log", &layout.logs().join("mysql-error.log"), false)?;
    for n in ["worldserver.console.log", "Server.log", "DBErrors.log", "Auth.log", "authserver.stderr.log", "Playerbots.log", "hermesproxy.log"] {
        add_file(n, &layout.server_logs().join(n), false)?;
    }
    add_file("worldserver.conf", &layout.server_config().join("worldserver.conf"), true)?;
    add_file("authserver.conf", &layout.server_config().join("authserver.conf"), true)?;
    add_file("playerbots.conf", &layout.server_config().join("modules").join("playerbots.conf"), true)?;
    w.start_file("etat.txt", opts).map_err(|e| e.to_string())?;
    let mut etat = String::new();
    etat.push_str(&format!("date : {}\nracine : {}\nversion : {}\n", util::datetime_readable(), layout.root.display(), Layout::version()));
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
