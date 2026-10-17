//! Identifiants du compte SQL local dédié. Même format que Tools/Common.ps1 (UserData/db-credentials.json).

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbCredentials {
    pub user: String,
    pub password: String,
    #[serde(rename = "rootPassword")]
    pub root_password: String,
    /// Ancien mot de passe root aléatoire, conservé jusqu'à la migration vers `ROOT_PASSWORD`
    #[serde(rename = "legacyRootPassword", default, skip_serializing_if = "Option::is_none")]
    pub legacy_root_password: Option<String>,
}

fn token(len: usize) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut out = String::with_capacity(len);
    while out.len() < len {
        let mut v: u64 = rand::random();
        for _ in 0..10 {
            if out.len() == len {
                break;
            }
            out.push(ALPHABET[(v % ALPHABET.len() as u64) as usize] as char);
            v /= ALPHABET.len() as u64;
        }
    }
    out
}

/// Mot de passe root de la base locale : celui que Keira3 pré-remplit (avec le port 3306), pour s'y connecter sans
/// rien saisir. La base n'écoute que sur 127.0.0.1 (décision utilisateur du 2026-10-17).
pub const ROOT_PASSWORD: &str = "root";

pub fn load_or_create(path: &Path) -> Result<DbCredentials, String> {
    if path.is_file() {
        let txt = std::fs::read_to_string(path).map_err(|e| format!("lecture {} : {e}", path.display()))?;
        // Les scripts PowerShell écrivent ce fichier en UTF-8 avec BOM : l'ignorer.
        let mut cred: DbCredentials = serde_json::from_str(txt.trim_start_matches('\u{feff}')).map_err(|e| format!("format {} : {e}", path.display()))?;
        // Installation antérieure au 2026-10-17 (mot de passe root aléatoire) : l'ancien est gardé pour la migration
        // (`MySql::start` applique le nouveau), le nouveau devient la référence.
        if cred.root_password != ROOT_PASSWORD {
            cred.legacy_root_password = Some(cred.root_password.clone());
            cred.root_password = ROOT_PASSWORD.into();
            save(path, &cred)?;
        }
        return Ok(cred);
    }
    let cred = DbCredentials { user: "acore".into(), password: token(32), root_password: ROOT_PASSWORD.into(), legacy_root_password: None };
    save(path, &cred)?;
    Ok(cred)
}

pub fn save(path: &Path, cred: &DbCredentials) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("création {} : {e}", dir.display()))?;
    }
    std::fs::write(path, serde_json::to_string_pretty(cred).unwrap()).map_err(|e| format!("écriture {} : {e}", path.display()))
}
