//! Identifiants du compte SQL local dédié. Même format que Tools/Common.ps1 (UserData/db-credentials.json).

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbCredentials {
    pub user: String,
    pub password: String,
    #[serde(rename = "rootPassword")]
    pub root_password: String,
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

pub fn load_or_create(path: &Path) -> Result<DbCredentials, String> {
    if path.is_file() {
        let txt = std::fs::read_to_string(path).map_err(|e| format!("lecture {} : {e}", path.display()))?;
        // Les scripts PowerShell écrivent ce fichier en UTF-8 avec BOM : l'ignorer.
        return serde_json::from_str(txt.trim_start_matches('\u{feff}')).map_err(|e| format!("format {} : {e}", path.display()));
    }
    let cred = DbCredentials { user: "mangos".into(), password: token(32), root_password: token(22) };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("création {} : {e}", dir.display()))?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&cred).unwrap()).map_err(|e| format!("écriture {} : {e}", path.display()))?;
    Ok(cred)
}
