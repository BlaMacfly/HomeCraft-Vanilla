//! Gestionnaire d'AddOns du client (Client/Interface/AddOns) : liste (lecture des .toc), ajout depuis une
//! archive zip (détection automatique du ou des dossiers d'AddOn), désactivation par renommage `.off`,
//! suppression. Le launcher ne fournit ni ne télécharge aucun AddOn.

use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct AddonEntry {
    pub name: String,
    pub path: PathBuf,
    pub title: String,
    pub version: String,
    pub interface: String,
    pub active: bool,
    pub has_toc: bool,
}

pub fn addons_dir(client_dir: &Path) -> PathBuf {
    client_dir.join("Interface").join("AddOns")
}

fn parse_toc(path: &Path) -> (String, String, String) {
    let txt = std::fs::read(path).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
    let mut title = String::new();
    let mut version = String::new();
    let mut interface = String::new();
    for l in txt.lines() {
        let l = l.trim_start_matches('\u{feff}').trim();
        if let Some(v) = l.strip_prefix("## Title:") {
            title = strip_color_codes(v.trim());
        } else if let Some(v) = l.strip_prefix("## Version:") {
            version = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("## Interface:") {
            interface = v.trim().to_string();
        }
    }
    (title, version, interface)
}

/// Retire les codes couleur |cffRRGGBB … |r des titres.
fn strip_color_codes(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '|' {
            match chars.peek() {
                Some('c') => {
                    chars.next();
                    for _ in 0..8 {
                        chars.next();
                    }
                }
                Some('r') => {
                    chars.next();
                }
                _ => out.push(c),
            }
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

pub fn list(client_dir: &Path) -> Vec<AddonEntry> {
    let dir = addons_dir(client_dir);
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(&dir) else { return out };
    for e in rd.filter_map(Result::ok) {
        let path = e.path();
        if !path.is_dir() {
            continue;
        }
        let folder = e.file_name().to_string_lossy().into_owned();
        let (name, active) = match folder.strip_suffix(".off") {
            Some(n) => (n.to_string(), false),
            None => (folder.clone(), true),
        };
        let toc = path.join(format!("{name}.toc"));
        let has_toc = toc.is_file();
        let (title, version, interface) = if has_toc { parse_toc(&toc) } else { (String::new(), String::new(), String::new()) };
        out.push(AddonEntry { name, path, title, version, interface, active, has_toc });
    }
    out.sort_by_key(|a| a.name.to_ascii_lowercase());
    out
}

pub fn set_active(entry: &AddonEntry, active: bool) -> Result<(), String> {
    if entry.active == active {
        return Ok(());
    }
    let parent = entry.path.parent().ok_or("dossier inconnu")?;
    let target = if active { parent.join(&entry.name) } else { parent.join(format!("{}.off", entry.name)) };
    if target.exists() {
        return Err(format!("{} existe déjà", target.display()));
    }
    std::fs::rename(&entry.path, &target).map_err(|e| format!("{} : {e}", entry.name))
}

pub fn remove(entry: &AddonEntry) -> Result<(), String> {
    std::fs::remove_dir_all(&entry.path).map_err(|e| format!("{} : {e}", entry.name))
}

/// Installe un ou plusieurs AddOns depuis une archive zip. Un AddOn est reconnu par un fichier `Nom.toc`
/// dans un dossier `Nom` (ou à la racine de l'archive, ou dans un dossier au nom différent, ex. `Nom-master`) ;
/// les .toc imbriqués dans un AddOn détecté (bibliothèques) sont ignorés. Renvoie les noms installés.
pub fn add_from_zip(zip_path: &Path, client_dir: &Path, progress: &dyn Fn(&str)) -> Result<Vec<String>, String> {
    let f = std::fs::File::open(zip_path).map_err(|e| format!("{} : {e}", zip_path.display()))?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| format!("archive illisible : {e}"))?;
    let names: Vec<String> = (0..z.len()).filter_map(|i| z.by_index(i).ok().map(|e| e.name().replace('\\', "/"))).collect();

    // Candidats : (préfixe dans l'archive, nom de l'AddOn)
    let mut candidates: Vec<(String, String)> = Vec::new();
    for n in &names {
        if !n.to_ascii_lowercase().ends_with(".toc") || n.ends_with('/') {
            continue;
        }
        let (dir, file) = match n.rfind('/') {
            Some(i) => (&n[..=i], &n[i + 1..]),
            None => ("", n.as_str()),
        };
        let stem = &file[..file.len() - 4];
        candidates.push((dir.to_string(), stem.to_string()));
    }
    // Les .toc dont le dossier porte le nom de l'AddOn priment ; on écarte ceux imbriqués dans un autre candidat.
    candidates.sort_by_key(|(d, _)| d.matches('/').count());
    let mut roots: Vec<(String, String)> = Vec::new();
    for (dir, stem) in candidates {
        if roots.iter().any(|(rd, _)| !rd.is_empty() && dir.starts_with(rd) && dir != *rd) {
            continue; // bibliothèque imbriquée dans un AddOn déjà retenu
        }
        let dir_name = dir.trim_end_matches('/').rsplit('/').next().unwrap_or("").to_string();
        let same_level_other = roots.iter().any(|(rd, _)| rd == &dir);
        if same_level_other && !dir_name.eq_ignore_ascii_case(&stem) {
            continue; // plusieurs .toc au même niveau : on garde celui qui porte le nom du dossier
        }
        roots.push((dir, stem));
    }
    if roots.is_empty() {
        return Err("aucun fichier .toc trouvé : ce n'est pas une archive d'AddOn".into());
    }

    let base = addons_dir(client_dir);
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    let mut installed = Vec::new();
    for (prefix, name) in roots {
        let dest = base.join(&name);
        progress(&format!("installation de {name}"));
        if dest.exists() {
            std::fs::remove_dir_all(&dest).map_err(|e| format!("{name} : {e}"))?;
        }
        let _ = std::fs::remove_dir_all(base.join(format!("{name}.off")));
        for i in 0..z.len() {
            let mut entry = z.by_index(i).map_err(|e| e.to_string())?;
            let ename = entry.name().replace('\\', "/");
            if !ename.starts_with(&prefix) {
                continue;
            }
            let rel = &ename[prefix.len()..];
            if rel.is_empty() || rel.contains("..") {
                continue;
            }
            let target = dest.join(rel);
            if ename.ends_with('/') {
                std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
                continue;
            }
            if let Some(p) = target.parent() {
                std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
            }
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            std::fs::write(&target, buf).map_err(|e| format!("{} : {e}", target.display()))?;
        }
        installed.push(name);
    }
    Ok(installed)
}
