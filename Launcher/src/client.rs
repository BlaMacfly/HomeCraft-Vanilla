//! Client WoW 2.4.3 (build 8606, frFR) fourni par l'utilisateur : vérification, configuration
//! (realmlist.wtf à la racine du client, Config.wtf), lancement.
//!
//! Disposition 2.4.3 vérifiée sur le client de l'utilisateur (2026-10-17) : archives générales dans Data\,
//! archives de langue dans Data\frFR\, realmlist.wtf À LA RACINE du client (le déplacement dans Data\<locale>\
//! date de WotLK), WTF\Config.wtf.

use crate::db::CREATE_NO_WINDOW;
use crate::paths::{CLIENT_BUILD, REALMD_PORT, REALM_NAME};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Archives officielles 2.4.3 obligatoires (chemins relatifs à Data\).
pub const REQUIRED_MPQ: &[&str] = &[
    "common.MPQ", "expansion.MPQ", "patch.MPQ", "patch-2.MPQ",
    "frFR/locale-frFR.MPQ", "frFR/expansion-locale-frFR.MPQ", "frFR/patch-frFR.MPQ", "frFR/patch-frFR-2.MPQ",
    "frFR/speech-frFR.MPQ", "frFR/expansion-speech-frFR.MPQ",
];
/// Archives officielles facultatives (présentes selon l'installation).
pub const OPTIONAL_OFFICIAL_MPQ: &[&str] = &["frFR/base-frFR.MPQ", "frFR/backup-frFR.MPQ"];

/// Une archive est-elle officielle ? Toute autre (patch-frFR-3.MPQ d'un serveur tiers, patch-X.MPQ ajouté…) est
/// ignorée par l'extraction des données serveur (repair.rs) : les extracteurs chargent sinon tous les patchs.
pub fn is_official_mpq(rel: &str) -> bool {
    let rel = rel.replace('\\', "/");
    REQUIRED_MPQ.iter().chain(OPTIONAL_OFFICIAL_MPQ.iter()).any(|m| m.eq_ignore_ascii_case(&rel))
}

#[derive(Clone, Debug, Default)]
pub struct ClientInfo {
    pub dir: PathBuf,
    pub exe_present: bool,
    pub version: Option<(u16, u16, u16, u16)>,
    pub missing_mpq: Vec<String>,
}

impl ClientInfo {
    pub fn is_ok(&self) -> bool {
        self.exe_present && self.version == Some(CLIENT_BUILD) && self.missing_mpq.is_empty()
    }
    pub fn version_string(&self) -> String {
        match self.version {
            Some((a, b, c, d)) => format!("{a}.{b}.{c}.{d}"),
            None => "?".into(),
        }
    }
}

/// Version du fichier via l'API Windows, sans dépendance C. On lit d'abord la chaîne « FileVersion »
/// (StringFileInfo : « 2, 4, 3, 8606 » sur le client fourni). Le bloc binaire VS_FIXEDFILEINFO peut mentir (constaté sur un
/// client 1.12.1 : 5880 au lieu de 5875) ; il sert de repli si la chaîne est absente.
pub fn file_version(path: &Path) -> Option<(u16, u16, u16, u16)> {
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO};
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
    let file: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let mut handle = 0u32;
        let size = GetFileVersionInfoSizeW(file.as_ptr(), &mut handle);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        if GetFileVersionInfoW(file.as_ptr(), 0, size, data.as_mut_ptr() as *mut _) == 0 {
            return None;
        }
        let query = |sub: &str| -> Option<(*mut core::ffi::c_void, u32)> {
            let sub = wide(sub);
            let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
            let mut len = 0u32;
            if VerQueryValueW(data.as_ptr() as *const _, sub.as_ptr(), &mut ptr, &mut len) == 0 || ptr.is_null() {
                None
            } else {
                Some((ptr, len))
            }
        };
        // 1. Chaîne FileVersion dans la première traduction disponible.
        if let Some((ptr, len)) = query("\\VarFileInfo\\Translation") {
            if len >= 4 {
                let lang = *(ptr as *const u16);
                let cp = *(ptr as *const u16).add(1);
                if let Some((sptr, slen)) = query(&format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\FileVersion")) {
                    let s = std::slice::from_raw_parts(sptr as *const u16, slen as usize);
                    let text = String::from_utf16_lossy(s);
                    let nums: Vec<u16> = text.split(|c: char| !c.is_ascii_digit()).filter(|p| !p.is_empty()).filter_map(|p| p.parse().ok()).collect();
                    if nums.len() >= 4 {
                        return Some((nums[0], nums[1], nums[2], nums[3]));
                    }
                }
            }
        }
        // 2. Repli : bloc binaire.
        let (ptr, _) = query("\\")?;
        let info = &*(ptr as *const VS_FIXEDFILEINFO);
        Some(((info.dwFileVersionMS >> 16) as u16, (info.dwFileVersionMS & 0xffff) as u16, (info.dwFileVersionLS >> 16) as u16, (info.dwFileVersionLS & 0xffff) as u16))
    }
}

/// Taille de l'écran principal (pixels).
pub fn primary_screen() -> (u32, u32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe { (GetSystemMetrics(SM_CXSCREEN).max(640) as u32, GetSystemMetrics(SM_CYSCREEN).max(480) as u32) }
}

/// Résolution sûre pour le client : en fenêtré, la fenêtre (bordures comprises) doit tenir dans l'écran ;
/// en plein écran, ne jamais dépasser l'écran. Choisie parmi les modes 4:3 / 16:9 / 16:10 connus du client 2.4.3.
pub fn safe_resolution(current: Option<(u32, u32)>, windowed: bool) -> (u32, u32) {
    let (sw, sh) = primary_screen();
    let (max_w, max_h) = if windowed { (sw.saturating_sub(16), sh.saturating_sub(120)) } else { (sw, sh) };
    const MODES: [(u32, u32); 12] = [(800, 600), (1024, 768), (1152, 864), (1280, 720), (1280, 800), (1280, 1024), (1366, 768), (1440, 900), (1600, 900), (1680, 1050), (1920, 1080), (2560, 1440)];
    if let Some((w, h)) = current {
        if w <= max_w && h <= max_h {
            return (w, h);
        }
    }
    MODES.iter().rev().copied().find(|(w, h)| *w <= max_w && *h <= max_h).unwrap_or((800, 600))
}

/// realmlist.wtf (racine du client) -> 127.0.0.1:<port d'authentification> (original conservé une fois en .orig) ;
/// Config.wtf fusionné clé par clé. La résolution est bornée à l'écran (piège Vanilla : fenêtré à la taille exacte de
/// l'écran = écran noir).
pub fn configure(dir: &Path, windowed: bool) -> Result<(), String> {
    let rl = dir.join("realmlist.wtf");
    let orig = dir.join("realmlist.wtf.orig");
    if rl.is_file() && !orig.exists() {
        std::fs::copy(&rl, &orig).map_err(|e| e.to_string())?;
    }
    std::fs::write(&rl, format!("set realmlist 127.0.0.1:{REALMD_PORT}\r\nset patchlist 127.0.0.1\r\n")).map_err(|e| format!("realmlist.wtf : {e}"))?;

    let wtf = dir.join("WTF");
    std::fs::create_dir_all(&wtf).map_err(|e| e.to_string())?;
    let cfg = wtf.join("Config.wtf");
    let existing = std::fs::read_to_string(&cfg).unwrap_or_default();
    let current_res = existing.lines().find_map(|l| {
        let l = l.trim();
        let v = l.strip_prefix("SET gxResolution ")?.trim().trim_matches('"');
        let (w, h) = v.split_once('x')?;
        Some((w.parse().ok()?, h.parse().ok()?))
    });
    let (rw, rh) = safe_resolution(current_res, windowed);
    let managed: [(&str, String); 6] = [
        ("locale", "frFR".into()),
        ("realmList", format!("127.0.0.1:{REALMD_PORT}")),
        ("realmName", REALM_NAME.into()),
        ("patchlist", "127.0.0.1".into()),
        ("gxWindow", if windowed { "1" } else { "0" }.into()),
        ("gxResolution", format!("{rw}x{rh}")),
    ];
    let mut out: Vec<String> = Vec::new();
    let mut done = [false; 6];
    for line in existing.lines() {
        let mut replaced = false;
        if let Some(rest) = line.trim().strip_prefix("SET ") {
            if let Some(key) = rest.split_whitespace().next() {
                if let Some(i) = managed.iter().position(|(k, _)| *k == key) {
                    out.push(format!("SET {} \"{}\"", key, managed[i].1));
                    done[i] = true;
                    replaced = true;
                }
            }
        }
        if !replaced {
            out.push(line.to_string());
        }
    }
    for (i, (k, v)) in managed.iter().enumerate() {
        if !done[i] {
            out.push(format!("SET {k} \"{v}\""));
        }
    }
    std::fs::write(&cfg, out.join("\r\n") + "\r\n").map_err(|e| format!("Config.wtf : {e}"))
}


pub fn inspect(dir: &Path) -> ClientInfo {
    let exe = dir.join("WoW.exe"); // « Wow.exe » sur le client fourni : Windows ignore la casse
    let exe_present = exe.is_file();
    let version = if exe_present { file_version(&exe) } else { None };
    let missing_mpq = REQUIRED_MPQ.iter().filter(|m| !dir.join("Data").join(m).is_file()).map(|m| m.to_string()).collect();
    ClientInfo { dir: dir.to_path_buf(), exe_present, version, missing_mpq }
}

pub fn launch(dir: &Path) -> Result<Child, String> {
    Command::new(dir.join("WoW.exe"))
        .current_dir(dir)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("WoW.exe : {e}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn official_archives() {
        assert!(super::is_official_mpq("frFR\\patch-frFR-2.MPQ"));
        assert!(super::is_official_mpq("common.mpq"));
        assert!(!super::is_official_mpq("frFR/patch-frFR-3.MPQ"));
        assert!(!super::is_official_mpq("patch-3.MPQ"));
    }
}
