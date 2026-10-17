//! Client fourni par l'utilisateur : détection (3.3.5a ou WotLK Classic 3.4.3), vérification, configuration
//! (realmlist / Config.wtf / portal), lancement. Le client n'est jamais fourni ni téléchargé.

use crate::db::CREATE_NO_WINDOW;
use crate::paths::{CLASSIC_CLIENT_BUILD, CLIENT_BUILD};
use crate::settings::ClientKind;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Archives communes attendues dans `Data/` d'un client 3.3.5a (les archives de locale sont dans `Data/<locale>/`).
pub const REQUIRED_MPQ: &[&str] = &["common.MPQ", "common-2.MPQ", "expansion.MPQ", "lichking.MPQ", "patch.MPQ", "patch-2.MPQ", "patch-3.MPQ"];
pub const LOCALE_MPQ: &[&str] = &["locale-{L}.MPQ", "patch-{L}.MPQ", "patch-{L}-2.MPQ", "patch-{L}-3.MPQ"];

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Detected {
    #[default]
    None,
    /// Wow.exe 3.3.5 + Data/<locale>
    Wotlk335,
    /// _classic_/WowClassic.exe (CASC) + Arctium WoW Launcher.exe
    Classic343,
}

/// Exécutables d'un client 3.3.5a, par ordre de préférence. Les quatre premiers viennent de wowdecomp
/// (décompilation C++ open source du client 12340, github.com/SatyPardus/wowdecomp) : x64 d'abord ; `WowClient*`
/// est la même chose que `Wow*` avec un journal de plantage (`WowClient_crash.log`). `Wow.exe` = client d'origine 32 bits.
pub const WOTLK_EXES: &[&str] = &["Wow64.exe", "WowClient64.exe", "Wow32.exe", "WowClient32.exe", "Wow.exe", "WoW.exe"];

#[derive(Clone, Debug, Default)]
pub struct ClientInfo {
    pub dir: PathBuf,
    pub kind: Detected,
    /// Exécutable retenu (3.3.5a) : voir `WOTLK_EXES`
    pub exe: PathBuf,
    /// Client décompilé wowdecomp (pas de ressource de version : build lu dans l'exécutable)
    pub decomp: bool,
    pub exe_present: bool,
    pub version: Option<(u16, u16, u16, u16)>,
    /// Locale détectée (dossier `Data/xxXX`), 3.3.5a seulement
    pub locale: String,
    pub missing_mpq: Vec<String>,
    /// 3.4.3 : Arctium WoW Launcher.exe présent à la racine du client
    pub arctium_present: bool,
    /// 3.4.3 : version lue dans .build.info
    pub build_info: String,
}

impl ClientInfo {
    pub fn is_ok(&self) -> bool {
        match self.kind {
            Detected::None => false,
            Detected::Wotlk335 => self.exe_present && matches!(self.version, Some((3, 3, 5, b)) if b == CLIENT_BUILD) && self.missing_mpq.is_empty() && !self.locale.is_empty(),
            Detected::Classic343 => self.exe_present && self.arctium_present && self.build_info.contains(&CLASSIC_CLIENT_BUILD.to_string()),
        }
    }
    pub fn version_string(&self) -> String {
        match (&self.kind, self.version) {
            (Detected::Classic343, _) => format!("WotLK Classic {}", if self.build_info.is_empty() { "?" } else { self.build_info.as_str() }),
            (_, Some((a, b, c, d))) if self.decomp => {
                let bits = if self.exe.file_name().map(|n| n.to_string_lossy().contains("64")).unwrap_or(false) { "x64" } else { "x86" };
                format!("{a}.{b}.{c}.{d} (wowdecomp {bits})")
            }
            (_, Some((a, b, c, d))) => format!("{a}.{b}.{c}.{d}"),
            _ => "?".into(),
        }
    }
    pub fn is_classic(&self) -> bool {
        self.kind == Detected::Classic343
    }
    /// Dossier des AddOns et du WTF selon le client.
    pub fn interface_root(&self) -> PathBuf {
        if self.is_classic() { self.dir.join("_classic_") } else { self.dir.clone() }
    }
}

/// Version du fichier via l'API Windows, sans dépendance C (chaîne « FileVersion » d'abord, bloc binaire en repli).
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
        let (ptr, _) = query("\\")?;
        let info = &*(ptr as *const VS_FIXEDFILEINFO);
        Some(((info.dwFileVersionMS >> 16) as u16, (info.dwFileVersionMS & 0xffff) as u16, (info.dwFileVersionLS >> 16) as u16, (info.dwFileVersionLS & 0xffff) as u16))
    }
}

/// Build d'un client wowdecomp : l'exécutable n'a pas de ressource de version, mais embarque la chaîne
/// « 3.3.5a (12340) » (version affichée à l'écran de connexion). Renvoie (3, 3, 5, build) si elle est trouvée.
pub fn embedded_version(path: &Path) -> Option<(u16, u16, u16, u16)> {
    let bytes = std::fs::read(path).ok()?;
    let needle = b"3.3.5a (";
    let pos = bytes.windows(needle.len()).position(|w| w == needle)?;
    let digits: String = bytes[pos + needle.len()..].iter().take(8).take_while(|b| b.is_ascii_digit()).map(|b| *b as char).collect();
    Some((3, 3, 5, digits.parse().ok()?))
}

/// Locale d'un client 3.3.5a : premier dossier `Data/xxXX` contenant `locale-xxXX.MPQ` (frFR privilégié).
fn detect_locale(dir: &Path) -> String {
    let data = dir.join("Data");
    let mut found: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&data) {
        for e in rd.filter_map(Result::ok) {
            let n = e.file_name().to_string_lossy().into_owned();
            if n.len() == 4 && e.path().is_dir() && e.path().join(format!("locale-{n}.MPQ")).is_file() {
                found.push(n);
            }
        }
    }
    found.sort_by_key(|l| if l == "frFR" { 0 } else { 1 });
    found.into_iter().next().unwrap_or_default()
}

/// Version du client Classic lue dans `.build.info` (colonne « Version »).
fn classic_build_info(dir: &Path) -> String {
    let txt = std::fs::read_to_string(dir.join(".build.info")).unwrap_or_default();
    let mut lines = txt.lines();
    let Some(header) = lines.next() else { return String::new() };
    let cols: Vec<&str> = header.split('|').collect();
    let Some(vi) = cols.iter().position(|c| c.starts_with("Version!")) else { return String::new() };
    for l in lines {
        let f: Vec<&str> = l.split('|').collect();
        if f.len() > vi && !f[vi].trim().is_empty() {
            return f[vi].trim().to_string();
        }
    }
    String::new()
}

pub fn inspect(dir: &Path, preferred: ClientKind) -> ClientInfo {
    let classic_exe = dir.join("_classic_").join("WowClassic.exe");
    let wotlk_exe = WOTLK_EXES.iter().map(|n| dir.join(n)).find(|p| p.is_file());
    let has_classic = classic_exe.is_file();
    let has_wotlk = wotlk_exe.is_some();
    let kind = match preferred {
        ClientKind::Classic343 => Detected::Classic343,
        ClientKind::Wotlk335 => Detected::Wotlk335,
        ClientKind::Auto if has_classic => Detected::Classic343,
        ClientKind::Auto if has_wotlk => Detected::Wotlk335,
        ClientKind::Auto => Detected::None,
    };
    match kind {
        Detected::Classic343 => ClientInfo {
            dir: dir.to_path_buf(),
            kind,
            exe: classic_exe.clone(),
            decomp: false,
            exe_present: has_classic,
            version: if has_classic { file_version(&classic_exe) } else { None },
            locale: String::new(),
            missing_mpq: Vec::new(),
            arctium_present: dir.join("Arctium WoW Launcher.exe").is_file(),
            build_info: classic_build_info(dir),
        },
        Detected::Wotlk335 | Detected::None => {
            let exe = wotlk_exe.unwrap_or_else(|| dir.join("Wow.exe"));
            let exe_present = exe.is_file();
            let decomp = exe_present && exe.file_name().map(|n| n.to_string_lossy().to_ascii_lowercase()).map(|n| n.starts_with("wow64") || n.starts_with("wow32") || n.starts_with("wowclient")).unwrap_or(false);
            let version = if !exe_present { None } else if decomp { embedded_version(&exe) } else { file_version(&exe) };
            let locale = detect_locale(dir);
            let mut missing: Vec<String> = REQUIRED_MPQ.iter().filter(|m| !dir.join("Data").join(m).is_file()).map(|m| m.to_string()).collect();
            if locale.is_empty() {
                missing.push("Data/<locale>/locale-xxXX.MPQ".into());
            } else {
                for m in LOCALE_MPQ {
                    let name = m.replace("{L}", &locale);
                    if !dir.join("Data").join(&locale).join(&name).is_file() {
                        missing.push(format!("{locale}/{name}"));
                    }
                }
            }
            ClientInfo { dir: dir.to_path_buf(), kind: if exe_present { Detected::Wotlk335 } else { Detected::None }, exe, decomp, exe_present, version, locale, missing_mpq: missing, arctium_present: false, build_info: String::new() }
        }
    }
}

/// Taille de l'écran principal (pixels).
pub fn primary_screen() -> (u32, u32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe { (GetSystemMetrics(SM_CXSCREEN).max(640) as u32, GetSystemMetrics(SM_CYSCREEN).max(480) as u32) }
}

/// Résolution sûre pour le client : en fenêtré, la fenêtre (bordures comprises) doit tenir dans l'écran.
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

/// Fusionne `WTF/Config.wtf` clé par clé (les autres réglages du joueur sont conservés).
fn merge_config_wtf(cfg: &Path, managed: &[(&str, String)]) -> Result<(), String> {
    if let Some(p) = cfg.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let existing = std::fs::read_to_string(cfg).unwrap_or_default();
    let mut out: Vec<String> = Vec::new();
    let mut done = vec![false; managed.len()];
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
    std::fs::write(cfg, out.join("\r\n") + "\r\n").map_err(|e| format!("Config.wtf : {e}"))
}

fn current_resolution(cfg: &Path, key: &str) -> Option<(u32, u32)> {
    let existing = std::fs::read_to_string(cfg).ok()?;
    existing.lines().find_map(|l| {
        let l = l.trim();
        let v = l.strip_prefix(&format!("SET {key} "))?.trim().trim_matches('"');
        let (w, h) = v.split_once('x')?;
        Some((w.parse().ok()?, h.parse().ok()?))
    })
}

/// 3.3.5a : `Data/<locale>/realmlist.wtf` → 127.0.0.1 (original conservé une fois), `WTF/Config.wtf` fusionné.
/// 3.4.3 : `_classic_/WTF/Config.wtf` → `portal 127.0.0.1` (HermesProxy écoute sur 1119).
pub fn configure(info: &ClientInfo, windowed: bool, realm_name: &str) -> Result<(), String> {
    let dir = &info.dir;
    if info.is_classic() {
        let cfg = dir.join("_classic_").join("WTF").join("Config.wtf");
        let managed = [("portal", "127.0.0.1".to_string()), ("gxMaximize", if windowed { "0" } else { "1" }.to_string())];
        return merge_config_wtf(&cfg, &managed);
    }
    let rl = dir.join("Data").join(&info.locale).join("realmlist.wtf");
    let orig = dir.join("Data").join(&info.locale).join("realmlist.wtf.orig");
    if rl.is_file() && !orig.exists() {
        std::fs::copy(&rl, &orig).map_err(|e| e.to_string())?;
    }
    std::fs::write(&rl, "set realmlist 127.0.0.1\r\n").map_err(|e| format!("realmlist.wtf : {e}"))?;
    let cfg = dir.join("WTF").join("Config.wtf");
    let (rw, rh) = safe_resolution(current_resolution(&cfg, "gxResolution"), windowed);
    let managed = [
        ("locale", info.locale.clone()),
        ("realmList", "127.0.0.1".to_string()),
        ("realmName", realm_name.to_string()),
        ("gxWindow", if windowed { "1" } else { "0" }.to_string()),
        ("gxMaximize", "0".to_string()),
        ("gxResolution", format!("{rw}x{rh}")),
    ];
    merge_config_wtf(&cfg, &managed)
}

/// Lance le jeu : exécutable 3.3.5a retenu par `inspect` (wowdecomp x64 en priorité, sinon `Wow.exe`) ou `Arctium WoW Launcher.exe --version=Classic --staticseed` (3.4.3, via HermesProxy).
pub fn launch(info: &ClientInfo) -> Result<Child, String> {
    let dir = &info.dir;
    let (exe, args): (PathBuf, Vec<&str>) = if info.is_classic() {
        (dir.join("Arctium WoW Launcher.exe"), vec!["--version=Classic", "--staticseed"])
    } else {
        (info.exe.clone(), vec![])
    };
    Command::new(&exe)
        .args(args)
        .current_dir(dir)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{} : {e}", exe.display()))
}

/// Noms des processus du jeu à fermer de force (FERMER LE JEU).
pub fn game_process_names(info: &ClientInfo) -> &'static [&'static str] {
    if info.is_classic() { &["WowClassic.exe", "Arctium WoW Launcher.exe"] } else { WOTLK_EXES }
}
