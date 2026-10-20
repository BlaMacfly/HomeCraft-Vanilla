//! Utilitaires : horodatage local sans dépendance externe, empreintes SHA-256, taille lisible.

use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

fn local_offset_seconds() -> i64 {
    use windows_sys::Win32::System::Time::GetTimeZoneInformation;
    unsafe {
        let mut tz = std::mem::zeroed();
        let r = GetTimeZoneInformation(&mut tz);
        let bias = tz.Bias + if r == 2 { tz.DaylightBias } else { tz.StandardBias };
        -(bias as i64) * 60
    }
}

/// (année, mois, jour) depuis un nombre de jours depuis 1970-01-01 (algorithme de H. Hinnant).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub struct LocalTime {
    pub y: i64,
    pub mo: u32,
    pub d: u32,
    pub h: u32,
    pub mi: u32,
    pub s: u32,
}

pub fn now_local() -> LocalTime {
    let epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) + local_offset_seconds();
    let days = epoch.div_euclid(86_400);
    let t = epoch.rem_euclid(86_400) as u32;
    let (y, mo, d) = civil_from_days(days);
    LocalTime { y, mo, d, h: t / 3600, mi: (t % 3600) / 60, s: t % 60 }
}

/// « 23:05:28 »
pub fn time_hms() -> String {
    let t = now_local();
    format!("{:02}:{:02}:{:02}", t.h, t.mi, t.s)
}

/// « 2026-10-14 23:05:28 »
pub fn datetime_readable() -> String {
    let t = now_local();
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", t.y, t.mo, t.d, t.h, t.mi, t.s)
}

/// « 20261014-230528 » (noms de fichiers)
pub fn datetime_compact() -> String {
    let t = now_local();
    format!("{:04}{:02}{:02}-{:02}{:02}{:02}", t.y, t.mo, t.d, t.h, t.mi, t.s)
}

/// Ligne écrite par un programme du serveur : UTF-8 si elle en est, sinon page de code OEM 850 (mangosd écrit ses
/// messages français pour la console Windows : « créé » arrive en 0x82 0x82 ; un décodage UTF-8 seul affichait
/// « Compte cr�� », un décodage Windows-1252 « Compte cr‚‚ »).
pub fn decode_console(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    const CP850_HIGH: &str = "ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜø£Ø×ƒáíóúñÑªº¿®¬½¼¡«»░▒▓│┤ÁÂÀ©╣║╗╝¢¥┐└┴┬├─┼ãÃ╚╔╩╦╠═╬¤ðÐÊËÈıÍÎÏ┘┌█▄¦Ì▀ÓßÔÒõÕµþÞÚÛÙýÝ¯´\u{ad}±‗¾¶§÷¸°¨·¹³²■\u{a0}";
    let high: Vec<char> = CP850_HIGH.chars().collect();
    bytes.iter().map(|&b| if b < 0x80 { b as char } else { high.get((b - 0x80) as usize).copied().unwrap_or('?') }).collect()
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("{} : {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

pub fn human_size(bytes: u64) -> String {
    if bytes >= 1 << 30 {
        format!("{:.2} Go", bytes as f64 / (1u64 << 30) as f64)
    } else if bytes >= 1 << 20 {
        format!("{:.1} Mo", bytes as f64 / (1u64 << 20) as f64)
    } else {
        format!("{} Ko", bytes / 1024)
    }
}

/// Copie récursive (installation depuis un support en lecture seule). `skip` : noms de dossiers de premier niveau ignorés.
pub fn copy_tree(src: &Path, dst: &Path, skip: &[&str], progress: &dyn Fn(usize, usize)) -> Result<(), String> {
    fn collect(dir: &Path, base: &Path, skip: &[&str], out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
        for e in std::fs::read_dir(dir).map_err(|e| format!("{} : {e}", dir.display()))? {
            let e = e.map_err(|e| e.to_string())?;
            let p = e.path();
            let rel = p.strip_prefix(base).unwrap_or(&p);
            if p.is_dir() {
                if rel.components().count() == 1 && skip.iter().any(|s| rel.as_os_str() == *s) {
                    continue;
                }
                collect(&p, base, skip, out)?;
            } else {
                out.push(rel.to_path_buf());
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    collect(src, src, skip, &mut files)?;
    let total = files.len();
    for (i, rel) in files.iter().enumerate() {
        let to = dst.join(rel);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{} : {e}", parent.display()))?;
        }
        std::fs::copy(src.join(rel), &to).map_err(|e| format!("{} : {e}", rel.display()))?;
        // Les fichiers venant d'un CD sont marqués lecture seule : on retire l'attribut.
        if let Ok(meta) = std::fs::metadata(&to) {
            let mut perm = meta.permissions();
            if perm.readonly() {
                perm.set_readonly(false);
                let _ = std::fs::set_permissions(&to, perm);
            }
        }
        progress(i + 1, total);
    }
    Ok(())
}

pub fn dir_file_count(dir: &Path) -> usize {
    std::fs::read_dir(dir).map(|r| r.filter_map(Result::ok).filter(|e| e.path().is_file()).count()).unwrap_or(0)
}

/// Message d'erreur de lancement d'un programme, avec une explication quand Windows a bloqué l'exécutable
/// (Contrôle intelligent des applications, stratégie WDAC, antivirus) : le code seul n'aide pas le joueur.
pub fn spawn_error(name: &str, e: &std::io::Error) -> String {
    match e.raw_os_error() {
        // 4551 : stratégie de contrôle d'application ; 4553 : intégrité du code ; 1260 : stratégie de groupe.
        Some(4551) | Some(4553) | Some(1260) => format!("{name} : {e}. {}", crate::i18n::fr::SPAWN_BLOCKED_APP_CONTROL),
        // 225 : fichier détecté comme virus ou logiciel indésirable.
        Some(225) => format!("{name} : {e}. {}", crate::i18n::fr::SPAWN_BLOCKED_ANTIVIRUS),
        _ => format!("{name} : {e}"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn console_lines_in_ansi_are_decoded() {
        assert_eq!(super::decode_console(&[b'c', b'r', 0x82, 0x82, b' ', 0x85]), "créé à");
        assert_eq!(super::decode_console("déjà".as_bytes()), "déjà");
        assert_eq!(super::decode_console(&[0x80, 0x87, 0x8A, 0xFF]), "Ççè\u{a0}");
    }

    #[test]
    fn blocked_executable_gets_an_explanation() {
        let e = std::io::Error::from_raw_os_error(4551);
        let m = super::spawn_error("VMapExtractor", &e);
        assert!(m.starts_with("VMapExtractor : "), "{m}");
        assert!(m.contains("Contrôle intelligent des applications"), "{m}");
        let other = super::spawn_error("x", &std::io::Error::from_raw_os_error(2));
        assert!(!other.contains("Contrôle intelligent"), "{other}");
    }
}
