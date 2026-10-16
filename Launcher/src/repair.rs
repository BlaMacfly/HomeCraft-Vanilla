//! Vérification et réparation : manifeste de fichiers gérés (Build/files-manifest.json, produit par le
//! développeur), données extraites du client, extracteurs VMaNGOS, export de diagnostic.

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

/// Présence des données extraites du client (seuils minimaux : 1.12.1 compte 158 DBC, 2 429 cartes).
pub fn data_status(layout: &Layout) -> Vec<(String, bool)> {
    let d = layout.server_data();
    vec![
        ("dbc".into(), util::dir_file_count(&d.join("5875").join("dbc")) >= 150),
        ("maps".into(), util::dir_file_count(&d.join("maps")) >= 2000),
        ("vmaps".into(), util::dir_file_count(&d.join("vmaps")) >= 1000),
        // Une génération interrompue laisse le marqueur en place : les mmaps sont alors considérées absentes et régénérées.
        ("mmaps".into(), util::dir_file_count(&d.join("mmaps")) >= 1000 && !mmaps_marker(layout).exists()),
    ]
}

/// Marqueur « génération des mmaps en cours » (créé au lancement de MoveMapGenerator, retiré à la fin).
pub fn mmaps_marker(layout: &Layout) -> PathBuf {
    layout.server_data().join("mmaps").join(".generation-en-cours")
}

/// Lance MoveMapGenerator sans attendre (génération en arrière-plan pendant que le serveur tourne).
/// L'appelant surveille le processus, puis appelle `mmaps_finished`.
pub fn spawn_mmaps(layout: &Layout, threads: usize, sink: &Sender<String>) -> Result<std::process::Child, String> {
    let bin = layout.server_bin();
    let data = layout.server_data();
    std::fs::create_dir_all(data.join("mmaps")).map_err(|e| e.to_string())?;
    std::fs::write(mmaps_marker(layout), "generation en cours").map_err(|e| e.to_string())?;
    spawn_tool("MoveMapGenerator", &bin.join("MoveMapGenerator.exe"), &["--threads".into(), threads.to_string(), "--silent".into()], &data, sink)
}

/// À appeler quand MoveMapGenerator s'est terminé : retire le marqueur si la génération a réussi.
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

/// Extraction des données du client avec les outils VMaNGOS (mêmes étapes que Tools/Extract-ClientData.ps1).
pub fn extract(layout: &Layout, client_dir: &Path, steps: &[ExtractStep], sink: &Sender<String>, progress: &dyn Fn(&str)) -> Result<(), String> {
    let bin = layout.server_bin();
    let data = layout.server_data();
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let q = |p: &Path| p.to_string_lossy().into_owned();
    if steps.contains(&ExtractStep::DbcMaps) {
        progress("extraction des DBC et des cartes");
        run_tool("MapExtractor", &bin.join("MapExtractor.exe"), &["-i".into(), q(client_dir), "-o".into(), q(&data), "--silent".into()], &data, sink)?;
        // mangosd attend DataDir/5875/dbc ; MapExtractor écrit <sortie>/dbc.
        let src = data.join("dbc");
        if src.is_dir() {
            let target = data.join("5875").join("dbc");
            std::fs::create_dir_all(data.join("5875")).map_err(|e| e.to_string())?;
            if target.exists() {
                std::fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
            }
            std::fs::rename(&src, &target).map_err(|e| e.to_string())?;
        }
    }
    if steps.contains(&ExtractStep::Vmaps) {
        progress("extraction des modèles de collision (vmaps)");
        run_tool("VMapExtractor", &bin.join("VMapExtractor.exe"), &["-l".into(), "-d".into(), q(&client_dir.join("Data")), "--silent".into()], &data, sink)?;
        std::fs::create_dir_all(data.join("vmaps")).map_err(|e| e.to_string())?;
        progress("assemblage des vmaps");
        run_tool("VMapAssembler", &bin.join("VMapAssembler.exe"), &["Buildings".into(), "vmaps".into()], &data, sink)?;
    }
    if steps.contains(&ExtractStep::Mmaps) {
        progress("génération des cartes de déplacement (mmaps, long)");
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).to_string();
        run_tool("MoveMapGenerator", &bin.join("MoveMapGenerator.exe"), &["--threads".into(), threads, "--silent".into()], &data, sink)?;
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
