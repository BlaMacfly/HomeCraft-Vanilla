//! Gestionnaire de patchs client 3.3.5a : archives MPQ supplémentaires dans `Data/` (`patch-X.MPQ`, X : chiffre 4-9
//! ou lettre A-Z) et dans `Data/<locale>/` (`patch-<locale>-X.MPQ`), les suivantes ayant priorité.
//! Désactiver = renommer en `.MPQ.off`. Le launcher ne fournit ni ne télécharge aucun patch.
//! Côté serveur, les extracteurs AzerothCore ne lisent que les archives officielles : un patch n'a aucun effet
//! sur les données du serveur. Sans objet pour le client 3.4.3 (données CASC).

use std::path::{Path, PathBuf};

pub const OFFICIAL: &[&str] = &["common.MPQ", "common-2.MPQ", "expansion.MPQ", "lichking.MPQ", "patch.MPQ", "patch-2.MPQ", "patch-3.MPQ"];

#[derive(Clone, Debug)]
pub struct PatchEntry {
    /// Nom logique (sans suffixe .off)
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub active: bool,
    pub official: bool,
    /// Le client ne charge que `patch-X.MPQ` / `patch-<locale>-X.MPQ` (X : 4-9 ou A-Z) ; sinon l'archive est ignorée même active.
    pub loaded_by_client: bool,
    /// Archive du dossier de locale (`Data/frFR`)
    pub locale: bool,
}

pub fn is_official(name: &str, locale: &str) -> bool {
    if OFFICIAL.iter().any(|o| o.eq_ignore_ascii_case(name)) {
        return true;
    }
    if locale.is_empty() {
        return false;
    }
    let l = locale.to_ascii_lowercase();
    let n = name.to_ascii_lowercase();
    ["locale-{L}.mpq", "patch-{L}.mpq", "patch-{L}-2.mpq", "patch-{L}-3.mpq", "base-{L}.mpq", "backup-{L}.mpq", "speech-{L}.mpq", "expansion-locale-{L}.mpq", "expansion-speech-{L}.mpq", "lichking-locale-{L}.mpq", "lichking-speech-{L}.mpq"]
        .iter()
        .any(|p| p.replace("{L}", &l) == n)
}

/// `patch-X.MPQ` ou `patch-<locale>-X.MPQ` avec X chiffre 4-9 ou lettre.
pub fn is_loadable_patch_name(name: &str, locale: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("patch-") else { return false };
    let Some(stem) = rest.strip_suffix(".mpq") else { return false };
    let stem = match stem.strip_prefix(&format!("{}-", locale.to_ascii_lowercase())) {
        Some(s) if !locale.is_empty() => s,
        _ => stem,
    };
    let mut chars = stem.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => c.is_ascii_alphabetic() || ('4'..='9').contains(&c),
        _ => false,
    }
}

fn scan(dir: &Path, locale: &str, is_locale_dir: bool, out: &mut Vec<PatchEntry>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.filter_map(Result::ok) {
        let path = e.path();
        if !path.is_file() {
            continue;
        }
        let file = e.file_name().to_string_lossy().into_owned();
        let lower = file.to_ascii_lowercase();
        let (name, active) = if let Some(n) = lower.strip_suffix(".off") {
            if !n.ends_with(".mpq") {
                continue;
            }
            (file[..file.len() - 4].to_string(), false)
        } else if lower.ends_with(".mpq") {
            (file.clone(), true)
        } else {
            continue;
        };
        let official = is_official(&name, locale);
        let size = e.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(PatchEntry { loaded_by_client: official || is_loadable_patch_name(&name, locale), name, path, size, active, official, locale: is_locale_dir });
    }
}

pub fn list(client_dir: &Path, locale: &str) -> Vec<PatchEntry> {
    let mut out = Vec::new();
    scan(&client_dir.join("Data"), locale, false, &mut out);
    if !locale.is_empty() {
        scan(&client_dir.join("Data").join(locale), locale, true, &mut out);
    }
    // Patchs ajoutés d'abord (ce que l'utilisateur gère), archives officielles ensuite.
    out.sort_by(|a, b| (a.official, a.locale, a.name.to_ascii_lowercase()).cmp(&(b.official, b.locale, b.name.to_ascii_lowercase())));
    out
}

/// Active (retire `.off`) ou désactive (ajoute `.off`) un patch non officiel.
pub fn set_active(entry: &PatchEntry, active: bool) -> Result<PathBuf, String> {
    if entry.official {
        return Err("archive officielle : non modifiable".into());
    }
    if entry.active == active {
        return Ok(entry.path.clone());
    }
    let parent = entry.path.parent().ok_or("dossier inconnu")?;
    let target = if active { parent.join(&entry.name) } else { parent.join(format!("{}.off", entry.name)) };
    if target.exists() {
        return Err(format!("{} existe déjà", target.display()));
    }
    std::fs::rename(&entry.path, &target).map_err(|e| format!("{} : {e}", entry.name))?;
    Ok(target)
}

/// Prochain nom `patch-X.MPQ` libre dans `Data/` : chiffres 4-9 puis lettres A-Z.
pub fn next_free_name(client_dir: &Path, locale: &str) -> Option<String> {
    let existing: Vec<String> = list(client_dir, locale).iter().filter(|p| !p.locale).map(|p| p.name.to_ascii_lowercase()).collect();
    ('4'..='9').chain('A'..='Z').map(|c| format!("patch-{c}.MPQ")).find(|n| !existing.contains(&n.to_ascii_lowercase()))
}

/// Copie une archive dans `Data/` (ou `Data/<locale>/` si son nom est `patch-<locale>-X.MPQ`). Si son nom n'est pas
/// chargeable par le client, elle est renommée vers le prochain `patch-X.MPQ` libre. Renvoie le nom final.
pub fn add(src: &Path, client_dir: &Path, locale: &str, progress: &dyn Fn(&str)) -> Result<String, String> {
    let file = src.file_name().ok_or("fichier sans nom")?.to_string_lossy().into_owned();
    if !file.to_ascii_lowercase().ends_with(".mpq") {
        return Err("seuls les fichiers .MPQ sont acceptés".into());
    }
    if is_official(&file, locale) {
        return Err("ce nom est celui d'une archive officielle du client".into());
    }
    let is_locale_patch = !locale.is_empty() && file.to_ascii_lowercase().starts_with(&format!("patch-{}-", locale.to_ascii_lowercase()));
    let name = if is_loadable_patch_name(&file, locale) { file.clone() } else { next_free_name(client_dir, locale).ok_or("plus aucun nom patch-X disponible")? };
    let dir = if is_locale_patch { client_dir.join("Data").join(locale) } else { client_dir.join("Data") };
    let dest = dir.join(&name);
    if dest.exists() || dir.join(format!("{name}.off")).exists() {
        return Err(format!("{name} existe déjà dans {}", dir.display()));
    }
    progress(&format!("copie de {file} vers {name}"));
    std::fs::copy(src, &dest).map_err(|e| format!("{file} : {e}"))?;
    if let Ok(meta) = std::fs::metadata(&dest) {
        let mut perm = meta.permissions();
        if perm.readonly() {
            perm.set_readonly(false);
            let _ = std::fs::set_permissions(&dest, perm);
        }
    }
    Ok(name)
}

pub fn remove(entry: &PatchEntry) -> Result<(), String> {
    if entry.official {
        return Err("archive officielle : suppression refusée".into());
    }
    std::fs::remove_file(&entry.path).map_err(|e| format!("{} : {e}", entry.name))
}
