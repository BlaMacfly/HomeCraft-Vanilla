//! Gestionnaire de patchs client : archives MPQ supplémentaires du client 2.4.3.
//! Le client charge, après ses archives officielles, les fichiers `Data\patch-X.MPQ` et
//! `Data\frFR\patch-frFR-X.MPQ` (X : chiffre 3 à 9 ou lettre ; les suivants ayant priorité).
//! Désactiver = renommer en `.MPQ.off`. Le launcher ne fournit ni ne télécharge aucun patch.
//! Côté serveur, l'extraction travaille sur une vue des seules archives officielles (repair.rs) : les patchs
//! ajoutés n'ont aucun effet sur les données du serveur (constat 2026-10-17 : sans cette vue, ad.exe lirait
//! patch-frFR-3.MPQ, qui modifie Spell.dbc). Les lettres A-Z sont reprises du Vanilla : leur prise en charge par le
//! client 2.4.3 reste à vérifier en jeu.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct PatchEntry {
    /// Nom logique relatif à Data (sans suffixe .off) : « patch-4.MPQ » ou « frFR/patch-frFR-3.MPQ ».
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub active: bool,
    pub official: bool,
    /// Le client ne charge que `patch-X.MPQ` / `patch-frFR-X.MPQ` (X : 3-9 ou A-Z) ; sinon l'archive est ignorée.
    pub loaded_by_client: bool,
}

pub fn is_official(name: &str) -> bool {
    crate::client::is_official_mpq(name)
}

/// Nom de fichier seul (sans « frFR/ »).
fn file_part(name: &str) -> &str {
    name.rsplit(['/', '\\']).next().unwrap_or(name)
}

/// Suffixe X valide : un chiffre de 3 à 9 ou une lettre.
fn valid_suffix(stem: &str) -> bool {
    let mut chars = stem.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => c.is_ascii_alphabetic() || ('3'..='9').contains(&c),
        _ => false,
    }
}

/// `patch-X.MPQ` (dans Data) ou `patch-frFR-X.MPQ` (dans Data\frFR).
pub fn is_loadable_patch_name(name: &str) -> bool {
    let lower = name.replace('\\', "/").to_ascii_lowercase();
    if let Some(f) = lower.strip_prefix("frfr/") {
        return f.strip_prefix("patch-frfr-").and_then(|r| r.strip_suffix(".mpq")).map(valid_suffix).unwrap_or(false);
    }
    lower.strip_prefix("patch-").and_then(|r| r.strip_suffix(".mpq")).map(valid_suffix).unwrap_or(false)
}

pub fn list(client_dir: &Path) -> Vec<PatchEntry> {
    let mut out = Vec::new();
    for (dir, prefix) in [(client_dir.join("Data"), ""), (client_dir.join("Data").join("frFR"), "frFR/")] {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for e in rd.filter_map(Result::ok) {
            let path = e.path();
            if !path.is_file() {
                continue;
            }
            let file = e.file_name().to_string_lossy().into_owned();
            let lower = file.to_ascii_lowercase();
            let (base, active) = if let Some(n) = lower.strip_suffix(".off") {
                if !n.ends_with(".mpq") {
                    continue;
                }
                (file[..file.len() - 4].to_string(), false)
            } else if lower.ends_with(".mpq") {
                (file.clone(), true)
            } else {
                continue;
            };
            let name = format!("{prefix}{base}");
            let official = is_official(&name);
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            out.push(PatchEntry { loaded_by_client: official || is_loadable_patch_name(&name), name, path, size, active, official });
        }
    }
    // Patchs ajoutés d'abord (ce que l'utilisateur gère), archives officielles ensuite.
    out.sort_by(|a, b| (a.official, a.name.to_ascii_lowercase()).cmp(&(b.official, b.name.to_ascii_lowercase())));
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
    let base = file_part(&entry.name);
    let target = if active { parent.join(base) } else { parent.join(format!("{base}.off")) };
    if target.exists() {
        return Err(format!("{} existe déjà", target.display()));
    }
    std::fs::rename(&entry.path, &target).map_err(|e| format!("{} : {e}", entry.name))?;
    Ok(target)
}

/// Prochain nom `patch-X.MPQ` libre dans Data : chiffres 3-9 puis lettres A-Z.
pub fn next_free_name(client_dir: &Path) -> Option<String> {
    let existing: Vec<String> = list(client_dir).iter().map(|p| p.name.to_ascii_lowercase()).collect();
    ('3'..='9').chain('A'..='Z').map(|c| format!("patch-{c}.MPQ")).find(|n| !existing.contains(&n.to_ascii_lowercase()))
}

/// Copie une archive dans le client : `patch-frFR-X.MPQ` va dans Data\frFR, toute autre archive dans Data
/// (renommée vers le prochain `patch-X.MPQ` libre si son nom n'est pas chargeable). Renvoie le nom final.
pub fn add(src: &Path, client_dir: &Path, progress: &dyn Fn(&str)) -> Result<String, String> {
    let file = src.file_name().ok_or("fichier sans nom")?.to_string_lossy().into_owned();
    if !file.to_ascii_lowercase().ends_with(".mpq") {
        return Err("seuls les fichiers .MPQ sont acceptés".into());
    }
    let locale_name = format!("frFR/{file}");
    if is_official(&file) || is_official(&locale_name) {
        return Err("ce nom est celui d'une archive officielle du client".into());
    }
    let name = if is_loadable_patch_name(&locale_name) {
        locale_name
    } else if is_loadable_patch_name(&file) {
        file.clone()
    } else {
        next_free_name(client_dir).ok_or("plus aucun nom patch-X disponible")?
    };
    let dest = client_dir.join("Data").join(&name);
    let dest_off = PathBuf::from(format!("{}.off", dest.display()));
    if dest.exists() || dest_off.exists() {
        return Err(format!("{name} existe déjà dans Data"));
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

#[cfg(test)]
mod tests {
    #[test]
    fn noms_de_patchs() {
        assert!(super::is_loadable_patch_name("patch-4.MPQ"));
        assert!(super::is_loadable_patch_name("frFR/patch-frFR-3.MPQ"));
        assert!(!super::is_loadable_patch_name("frFR/patch-4.MPQ"));
        assert!(!super::is_loadable_patch_name("patch-2.MPQ"));
        assert!(super::is_official("frFR/patch-frFR-2.MPQ"));
        assert!(!super::is_official("frFR/patch-frFR-3.MPQ"));
    }
}
