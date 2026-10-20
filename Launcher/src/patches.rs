//! Gestionnaire de patchs client : archives MPQ supplémentaires dans Client/Data.
//! Le client 1.12.1 charge, après les archives officielles, les fichiers `patch-X.MPQ` où X est un chiffre (3 à 9)
//! ou une lettre (A à Z), les suivants ayant priorité. Désactiver = renommer en `.MPQ.off`.
//! Le launcher ne fournit ni ne télécharge aucun patch : l'utilisateur apporte les siens.
//! Côté serveur, les extracteurs VMaNGOS ne lisent que model/dbc/terrain/patch/patch-2 : les patchs ajoutés
//! n'ont aucun effet sur les données du serveur (modifications client uniquement).

use std::path::{Path, PathBuf};

pub const OFFICIAL: &[&str] = &[
    "base.MPQ", "dbc.MPQ", "fonts.MPQ", "interface.MPQ", "misc.MPQ", "model.MPQ", "patch.MPQ", "patch-2.MPQ",
    "sound.MPQ", "speech.MPQ", "terrain.MPQ", "texture.MPQ", "wmo.MPQ", "backup.MPQ",
];

#[derive(Clone, Debug)]
pub struct PatchEntry {
    /// Nom logique (sans suffixe .off)
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub active: bool,
    pub official: bool,
    /// Le client ne charge que `patch-X.MPQ` (X : 3-9 ou A-Z) ; sinon l'archive est ignorée même active.
    pub loaded_by_client: bool,
}

pub fn is_official(name: &str) -> bool {
    OFFICIAL.iter().any(|o| o.eq_ignore_ascii_case(name))
}

/// `patch-X.MPQ` avec X chiffre 3-9 ou lettre.
pub fn is_loadable_patch_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("patch-") else { return false };
    let Some(stem) = rest.strip_suffix(".mpq") else { return false };
    let mut chars = stem.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => c.is_ascii_alphabetic() || ('3'..='9').contains(&c),
        _ => false,
    }
}

pub fn list(client_dir: &Path) -> Vec<PatchEntry> {
    let data = client_dir.join("Data");
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(&data) else { return out };
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
        let official = is_official(&name);
        let size = e.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(PatchEntry { loaded_by_client: official || is_loadable_patch_name(&name), name, path, size, active, official });
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
    let target = if active { parent.join(&entry.name) } else { parent.join(format!("{}.off", entry.name)) };
    if target.exists() {
        return Err(format!("{} existe déjà", target.display()));
    }
    std::fs::rename(&entry.path, &target).map_err(|e| format!("{} : {e}", entry.name))?;
    Ok(target)
}

/// Prochain nom `patch-X.MPQ` libre : chiffres 3-9 puis lettres A-Z.
pub fn next_free_name(client_dir: &Path) -> Option<String> {
    let existing: Vec<String> = list(client_dir).iter().map(|p| p.name.to_ascii_lowercase()).collect();
    ('3'..='9').chain('A'..='Z').map(|c| format!("patch-{c}.MPQ")).find(|n| !existing.contains(&n.to_ascii_lowercase()))
}

/// Copie une archive dans Client/Data. Si son nom n'est pas chargeable par le client, elle est renommée
/// vers le prochain `patch-X.MPQ` libre. Renvoie le nom final.
pub fn add(src: &Path, client_dir: &Path, progress: &dyn Fn(&str)) -> Result<String, String> {
    let file = src.file_name().ok_or("fichier sans nom")?.to_string_lossy().into_owned();
    if !file.to_ascii_lowercase().ends_with(".mpq") {
        return Err("seuls les fichiers .MPQ sont acceptés".into());
    }
    if is_official(&file) {
        return Err("ce nom est celui d'une archive officielle du client".into());
    }
    let name = if is_loadable_patch_name(&file) { file.clone() } else { next_free_name(client_dir).ok_or("plus aucun nom patch-X disponible")? };
    let dest = client_dir.join("Data").join(&name);
    if dest.exists() || client_dir.join("Data").join(format!("{name}.off")).exists() {
        return Err(format!("{name} existe déjà dans Data"));
    }
    progress(&format!("copie de {file} vers {name}"));
    // Copie sous un nom que le client ne charge pas, puis renommage : une copie interrompue (disque plein…) ne
    // laisse jamais une archive tronquée que le jeu tenterait d'ouvrir.
    let partial = client_dir.join("Data").join(format!("{name}.copie-en-cours"));
    if let Err(e) = std::fs::copy(src, &partial) {
        let _ = std::fs::remove_file(&partial);
        return Err(format!("{file} : {e}"));
    }
    std::fs::rename(&partial, &dest).map_err(|e| {
        let _ = std::fs::remove_file(&partial);
        format!("{file} : {e}")
    })?;
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
