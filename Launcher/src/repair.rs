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
        // Les versions ≤ 0.9.9 généraient directement dans mmaps : avec leur marqueur, la génération est inachevée.
        ("mmaps".into(), util::dir_file_count(&d.join("mmaps")) >= 1000 && !legacy_mmaps_marker(layout).exists()),
    ]
}

// Les mmaps sont générées dans un dossier de travail et n'entrent dans Server\Data\mmaps qu'une fois complètes,
// serveur arrêté. mangosd charge toute tuile présente sur le disque : avec des mmaps partielles (génération en cours,
// ou installées pendant que des zones sont déjà chargées), le carnet de navigation d'un continent existe sans la
// tuile d'une capitale, et PopulateAreaBotAI::BeforeAddToMap boucle sans fin sur Map::GetWalkRandomPosition :
// le monde se fige (joueur bloqué sur « Connecté »).

/// Dossier de travail de MoveMapGenerator (il lit maps et vmaps et écrit mmaps relativement à son dossier courant).
pub fn mmaps_staging(layout: &Layout) -> PathBuf {
    layout.server_data().join("mmaps-generation")
}

/// Marqueur « génération des mmaps en cours » (créé au lancement de MoveMapGenerator, retiré s'il réussit).
pub fn mmaps_marker(layout: &Layout) -> PathBuf {
    mmaps_staging(layout).join(".generation-en-cours")
}

fn legacy_mmaps_marker(layout: &Layout) -> PathBuf {
    layout.server_data().join("mmaps").join(".generation-en-cours")
}

/// Génération terminée, en attente d'installation (au prochain démarrage du serveur).
pub fn mmaps_pending(layout: &Layout) -> bool {
    util::dir_file_count(&mmaps_staging(layout).join("mmaps")) >= 1000 && !mmaps_marker(layout).exists()
}

/// Remplit `dst` de liens physiques vers les fichiers de `src` (copie si le volume ne les accepte pas).
fn link_dir_files(src: &Path, dst: &Path) -> Result<(), String> {
    if dst.exists() {
        std::fs::remove_dir_all(dst).map_err(|e| format!("{} : {e}", dst.display()))?;
    }
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for e in std::fs::read_dir(src).map_err(|e| format!("{} : {e}", src.display()))?.flatten() {
        let p = e.path();
        if p.is_file() {
            let to = dst.join(e.file_name());
            if std::fs::hard_link(&p, &to).is_err() {
                std::fs::copy(&p, &to).map_err(|e| format!("{} : {e}", p.display()))?;
            }
        }
    }
    Ok(())
}

/// À appeler serveur arrêté, juste avant de lancer mangosd. Installe des mmaps complètes en attente
/// (renvoie true), et range dans le dossier de travail des mmaps partielles laissées par une ancienne version
/// (MoveMapGenerator reprend les tuiles déjà faites).
pub fn install_pending_mmaps(layout: &Layout) -> Result<bool, String> {
    let live = layout.server_data().join("mmaps");
    let staging = mmaps_staging(layout);
    if legacy_mmaps_marker(layout).exists() {
        std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
        if staging.join("mmaps").exists() {
            std::fs::remove_dir_all(&live).map_err(|e| format!("{} : {e}", live.display()))?;
        } else {
            std::fs::rename(&live, staging.join("mmaps")).map_err(|e| format!("{} : {e}", live.display()))?;
            let _ = std::fs::remove_file(staging.join("mmaps").join(".generation-en-cours"));
        }
        std::fs::write(mmaps_marker(layout), "generation en cours").map_err(|e| e.to_string())?;
    }
    if !mmaps_pending(layout) {
        return Ok(false);
    }
    if live.exists() {
        std::fs::remove_dir_all(&live).map_err(|e| format!("{} : {e}", live.display()))?;
    }
    std::fs::rename(staging.join("mmaps"), &live).map_err(|e| format!("installation des mmaps : {e}"))?;
    std::fs::remove_dir_all(&staging).map_err(|e| format!("{} : {e}", staging.display()))?;
    Ok(true)
}

/// Lance MoveMapGenerator sans attendre (génération en arrière-plan pendant que le serveur tourne), dans le
/// dossier de travail. L'appelant surveille le processus, puis appelle `mmaps_finished`.
pub fn spawn_mmaps(layout: &Layout, threads: usize, sink: &Sender<String>) -> Result<std::process::Child, String> {
    let bin = layout.server_bin();
    let data = layout.server_data();
    let staging = mmaps_staging(layout);
    std::fs::create_dir_all(staging.join("mmaps")).map_err(|e| e.to_string())?;
    link_dir_files(&data.join("maps"), &staging.join("maps"))?;
    link_dir_files(&data.join("vmaps"), &staging.join("vmaps"))?;
    std::fs::write(mmaps_marker(layout), "generation en cours").map_err(|e| e.to_string())?;
    spawn_tool("MoveMapGenerator", &bin.join("MoveMapGenerator.exe"), &["--threads".into(), threads.to_string(), "--silent".into()], &staging, sink)
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
    let tail = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let (mut child, reader) = spawn_tool_with_tail(name, exe, args, cwd, sink, Some(tail.clone()))?;
    let status = child.wait().map_err(|e| e.to_string())?;
    if let Some(r) = reader {
        let _ = r.join();
    }
    if status.success() {
        return Ok(());
    }
    // Derniers messages significatifs de l'outil : le code de sortie seul (souvent 1) n'explique rien.
    let last = tail.lock().map(|t| t.join(" / ")).unwrap_or_default();
    Err(if last.is_empty() { format!("{name} a échoué (code {:?})", status.code()) } else { format!("{name} a échoué (code {:?}) : {last}", status.code()) })
}

/// Ligne d'erreur d'un extracteur : transmise au journal immédiatement (pas d'échantillonnage) et retenue pour le message d'échec.
fn is_error_line(line: &str) -> bool {
    let l = line.to_ascii_lowercase();
    l.contains("error") || l.contains("fatal") || l.contains("polluted") || l.contains("could not") || l.contains("couldn't") || l.contains("not found") || l.contains("unable")
}

fn spawn_tool(name: &str, exe: &Path, args: &[String], cwd: &Path, sink: &Sender<String>) -> Result<std::process::Child, String> {
    spawn_tool_with_tail(name, exe, args, cwd, sink, None).map(|(c, _)| c)
}

fn spawn_tool_with_tail(name: &str, exe: &Path, args: &[String], cwd: &Path, sink: &Sender<String>, tail: Option<std::sync::Arc<std::sync::Mutex<Vec<String>>>>) -> Result<(std::process::Child, Option<std::thread::JoinHandle<()>>), String> {
    let mut child = Command::new(exe)
        .args(args)
        .current_dir(cwd)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| crate::util::spawn_error(name, &e))?;
    let mut handle = None;
    if let Some(out) = child.stdout.take() {
        let sink = sink.clone();
        let name = name.to_string();
        handle = Some(std::thread::spawn(move || {
            let mut reader = BufReader::new(out);
            let mut buf = Vec::new();
            let mut last = std::time::Instant::now() - std::time::Duration::from_secs(10);
            let mut errors_sent = 0u32;
            loop {
                buf.clear();
                match reader.read_until(b'\n', &mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let line = String::from_utf8_lossy(&buf).trim().to_string();
                        if line.is_empty() {
                            continue;
                        }
                        let error = is_error_line(&line);
                        if error {
                            if let Some(t) = &tail {
                                if let Ok(mut t) = t.lock() {
                                    t.push(line.chars().take(200).collect());
                                    let n = t.len();
                                    if n > 3 {
                                        t.drain(..n - 3);
                                    }
                                }
                            }
                        }
                        // Les extracteurs écrivent des barres de progression très verbeuses : une ligne toutes les 5 s
                        // suffit, sauf les erreurs, transmises toutes.
                        // Plafond : un extracteur peut signaler des centaines de modèles manquants sans échouer.
                        if (error && errors_sent < 20) || last.elapsed() >= std::time::Duration::from_secs(5) {
                            let _ = sink.send(format!("[{name}] {line}"));
                            last = std::time::Instant::now();
                            if error {
                                errors_sent += 1;
                            }
                        }
                    }
                }
            }
        }));
    }
    Ok((child, handle))
}

/// Extraction des données du client avec les outils VMaNGOS (mêmes étapes que Tools/Extract-ClientData.ps1).
/// Archives officielles du client 1.12.1. Les extracteurs du serveur ouvrent aussi `patch-3.MPQ` à `patch-99.MPQ` :
/// un patch ajouté (graphismes HD…) y injecterait ses modèles (« Couldn't open RootWmo », collisions fausses, voire
/// plantage 0xC0000374 de VMapExtractor). L'extraction ne voit donc que ces archives.
const OFFICIAL_MPQ: &[&str] = &[
    "backup.MPQ", "base.MPQ", "dbc.MPQ", "fonts.MPQ", "interface.MPQ", "misc.MPQ", "model.MPQ", "patch.MPQ", "patch-2.MPQ",
    "sound.MPQ", "speech.MPQ", "terrain.MPQ", "texture.MPQ", "wmo.MPQ",
];
/// Suffixe temporaire des patchs ajoutés, quand ils doivent être masqués pendant l'extraction (repli sans lien physique).
const HIDDEN_SUFFIX: &str = ".homecraft-extraction";

/// `patch-N.MPQ` avec N ≥ 3 : jamais une archive officielle de la 1.12.1, mais lue par les extracteurs.
pub fn is_added_patch(name: &str) -> bool {
    let l = name.to_ascii_lowercase();
    l.strip_prefix("patch-").and_then(|r| r.strip_suffix(".mpq")).and_then(|n| n.parse::<u32>().ok()).map(|n| n >= 3).unwrap_or(false)
}

/// Source d'extraction ne contenant que les archives officielles. Restaure tout à la fin (`Drop`), même en cas d'erreur.
struct ExtractionSource {
    root: PathBuf,
    linked: Option<PathBuf>,
    hidden: Vec<(PathBuf, PathBuf)>,
}

impl Drop for ExtractionSource {
    fn drop(&mut self) {
        for (hidden, original) in self.hidden.drain(..) {
            let _ = std::fs::rename(&hidden, &original);
        }
        if let Some(dir) = self.linked.take() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

/// Prépare la source d'extraction : le client tel quel s'il n'a aucun patch ajouté ; sinon un dossier de liens physiques
/// vers ses seules archives officielles (`Server\Data\extraction-client`) ; à défaut (client sur un autre volume),
/// les patchs ajoutés sont renommés le temps de l'extraction puis rétablis.
fn prepare_source(layout: &Layout, client_dir: &Path, sink: &Sender<String>) -> Result<ExtractionSource, String> {
    let data = client_dir.join("Data");
    let entries: Vec<(String, PathBuf)> = std::fs::read_dir(&data)
        .map_err(|e| format!("{} : {e}", data.display()))?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok().map(|n| (n, e.path())))
        .collect();
    // Patchs masqués par une extraction précédente interrompue : rétablis d'abord.
    for (name, path) in &entries {
        if let Some(orig) = name.strip_suffix(HIDDEN_SUFFIX) {
            let target = data.join(orig);
            if !target.exists() {
                let _ = std::fs::rename(path, &target);
            }
        }
    }
    let added: Vec<String> = entries.iter().map(|(n, _)| n.clone()).filter(|n| is_added_patch(n)).collect();
    if added.is_empty() {
        return Ok(ExtractionSource { root: client_dir.to_path_buf(), linked: None, hidden: Vec::new() });
    }
    let _ = sink.send(format!("[extraction] patchs ajoutés ignorés par l'extraction du serveur : {}", added.join(", ")));
    let link_root = layout.server_data().join("extraction-client");
    if link_root.exists() {
        std::fs::remove_dir_all(&link_root).map_err(|e| format!("{} : {e}", link_root.display()))?;
    }
    std::fs::create_dir_all(link_root.join("Data")).map_err(|e| e.to_string())?;
    let mut linked_all = true;
    for (name, path) in &entries {
        if OFFICIAL_MPQ.iter().any(|o| o.eq_ignore_ascii_case(name)) && std::fs::hard_link(path, link_root.join("Data").join(name)).is_err() {
            linked_all = false;
            break;
        }
    }
    if linked_all {
        return Ok(ExtractionSource { root: link_root.clone(), linked: Some(link_root), hidden: Vec::new() });
    }
    // Client sur un autre volume : pas de lien physique possible, on masque les patchs ajoutés.
    let _ = std::fs::remove_dir_all(&link_root);
    let mut src = ExtractionSource { root: client_dir.to_path_buf(), linked: None, hidden: Vec::new() };
    for name in &added {
        let original = data.join(name);
        let hidden = data.join(format!("{name}{HIDDEN_SUFFIX}"));
        std::fs::rename(&original, &hidden).map_err(|e| format!("{name} (masquage temporaire pour l'extraction) : {e}"))?;
        src.hidden.push((hidden, original));
    }
    Ok(src)
}

pub fn extract(layout: &Layout, client_dir: &Path, steps: &[ExtractStep], sink: &Sender<String>, progress: &dyn Fn(&str)) -> Result<(), String> {
    let bin = layout.server_bin();
    let data = layout.server_data();
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let source = prepare_source(layout, client_dir, sink)?;
    let client_dir = source.root.as_path();
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
        // Dossier de travail de VMapExtractor : s'il reste d'une extraction interrompue (programme bloqué par Windows,
        // launcher fermé), l'extracteur refuse de repartir (« Your output directory seems to be polluted », code 1).
        // Il est entièrement régénéré : on le vide toujours avant de commencer.
        let work = data.join("Buildings");
        // VMapExtractor plante de façon intermittente (0xC0000374 corruption du tas, 0xC0000005) : la même commande
        // réussit ensuite (vérifié le 2026-10-19). Jusqu'à trois essais, chacun sur un dossier de travail propre ;
        // un échec « normal » (code positif) n'est pas retenté.
        let mut attempt = 1;
        loop {
            if work.exists() {
                std::fs::remove_dir_all(&work).map_err(|e| format!("nettoyage de {} : {e}", work.display()))?;
            }
            match run_tool("VMapExtractor", &bin.join("VMapExtractor.exe"), &["-l".into(), "-d".into(), q(&client_dir.join("Data")), "--silent".into()], &data, sink) {
                Ok(()) => break,
                Err(e) if attempt < 3 && e.contains("(code Some(-") => {
                    let _ = sink.send(format!("[extraction] VMapExtractor s'est arrêté brutalement (essai {attempt}/3), nouvelle tentative : {e}"));
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_patches_are_patch_3_and_above() {
        for n in ["patch-3.MPQ", "patch-4.mpq", "PATCH-5.MPQ", "patch-12.MPQ"] {
            assert!(is_added_patch(n), "{n}");
        }
        for n in ["patch.MPQ", "patch-2.MPQ", "patch-A.MPQ", "patch-3.MPQ.off", "base.MPQ", "patch-3.MPQ.homecraft-extraction"] {
            assert!(!is_added_patch(n), "{n}");
        }
    }

    #[test]
    fn extraction_source_links_official_archives_only_and_cleans_up() {
        let tmp = std::env::temp_dir().join(format!("hc-src-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let root = tmp.join("HomeCraft");
        std::fs::create_dir_all(root.join("Server").join("Data")).unwrap();
        std::fs::create_dir_all(root.join("Tools")).unwrap();
        let client = tmp.join("Client");
        std::fs::create_dir_all(client.join("Data")).unwrap();
        for n in ["base.MPQ", "dbc.MPQ", "patch.MPQ", "patch-2.MPQ", "patch-4.MPQ", "patch-5.MPQ"] {
            std::fs::write(client.join("Data").join(n), n.as_bytes()).unwrap();
        }
        let layout = Layout { root: root.clone() };
        let (tx, _rx) = std::sync::mpsc::channel();
        {
            let src = prepare_source(&layout, &client, &tx).unwrap();
            let names: Vec<String> = std::fs::read_dir(src.root.join("Data")).unwrap().filter_map(|e| e.ok()?.file_name().into_string().ok()).collect();
            assert!(names.contains(&"patch-2.MPQ".to_string()));
            assert!(!names.iter().any(|n| is_added_patch(n)), "{names:?}");
        }
        assert!(!root.join("Server").join("Data").join("extraction-client").exists(), "dossier de liens supprimé");
        assert!(client.join("Data").join("patch-5.MPQ").is_file(), "patchs du joueur intacts");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn partial_mmaps_never_reach_the_server() {
        let tmp = std::env::temp_dir().join(format!("hc-mmaps-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let root = tmp.join("HomeCraft");
        let data = root.join("Server").join("Data");
        std::fs::create_dir_all(data.join("mmaps")).unwrap();
        let layout = Layout { root: root.clone() };
        let fill = |dir: &Path, n: usize| (0..n).for_each(|i| std::fs::write(dir.join(format!("000{i:04}.mmtile")), b"t").unwrap());
        // Génération inachevée d'une ancienne version, directement dans mmaps : rangée dans le dossier de travail.
        fill(&data.join("mmaps"), 400);
        std::fs::write(data.join("mmaps").join(".generation-en-cours"), "x").unwrap();
        assert!(!install_pending_mmaps(&layout).unwrap());
        assert!(!data.join("mmaps").exists(), "mmaps partielles retirées du serveur");
        assert_eq!(util::dir_file_count(&mmaps_staging(&layout).join("mmaps")), 400, "tuiles conservées pour la reprise");
        assert!(mmaps_marker(&layout).exists() && !mmaps_pending(&layout));
        // Génération terminée : installée en bloc, dossier de travail supprimé.
        fill(&mmaps_staging(&layout).join("mmaps"), 1200);
        mmaps_finished(&layout, true);
        assert!(mmaps_pending(&layout));
        assert!(install_pending_mmaps(&layout).unwrap());
        assert_eq!(util::dir_file_count(&data.join("mmaps")), 1200);
        assert!(!mmaps_staging(&layout).exists());
        assert!(data_status(&layout).iter().any(|(n, ok)| n == "mmaps" && *ok));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
