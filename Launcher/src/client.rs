//! Client WoW 1.12.1 fourni par l'utilisateur : vérification, configuration, lancement.
//! Deux sortes de client sont reconnues :
//! - le client **d'origine** (`WoW.exe` 1.12.1, 32 bits) : `realmlist.wtf` et `WTF\Config.wtf` ;
//! - **Benilla** (client libre réécrit en Rust, 64 bits, MIT/Apache-2.0) : un exécutable à la racine
//!   (`benilla.exe`, ou renommé, par ex. `HomeCraft.exe`), les MPQ d'origine dans `Data\` et tout l'état
//!   local dans `benilla-config\` (`config.toml`, section `[cvars]` ; `AddOns\`…). Sa langue vient du
//!   `Wow.ini` intégré à `interface.MPQ` : des MPQ issus d'un client frFR donnent un jeu en français.

use crate::db::CREATE_NO_WINDOW;
use crate::paths::REALM_NAME;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Archives requises par le client d'origine.
pub const REQUIRED_MPQ: &[&str] = &[
    "base.MPQ", "dbc.MPQ", "fonts.MPQ", "interface.MPQ", "misc.MPQ", "model.MPQ", "patch.MPQ", "patch-2.MPQ",
    "sound.MPQ", "speech.MPQ", "terrain.MPQ", "texture.MPQ", "wmo.MPQ",
];
/// Archives lues par Benilla (les douze MPQ 1.12.1 ; `base.MPQ` ne lui sert pas).
pub const BENILLA_MPQ: &[&str] = &[
    "dbc.MPQ", "fonts.MPQ", "interface.MPQ", "misc.MPQ", "model.MPQ", "patch.MPQ", "patch-2.MPQ",
    "sound.MPQ", "speech.MPQ", "terrain.MPQ", "texture.MPQ", "wmo.MPQ",
];
/// Archives ouvertes par les extracteurs VMaNGOS (MapExtractor, VMapExtractor). `base.MPQ` en fait partie :
/// sans elle VMapExtractor plante (0xC0000005) au milieu des modèles WMO. Benilla, lui, ne la monte jamais.
pub const EXTRACTION_MPQ: &[&str] = &["base.MPQ", "dbc.MPQ", "misc.MPQ", "model.MPQ", "terrain.MPQ", "texture.MPQ", "wmo.MPQ", "patch.MPQ", "patch-2.MPQ"];
/// Dossier d'état de Benilla, à côté de son exécutable.
pub const BENILLA_HOME: &str = "benilla-config";
/// Exécutables d'un client d'origine ou déposés par le launcher : jamais pris pour Benilla.
const NOT_A_CLIENT: &[&str] = &["wow.exe", "vanillafixes.exe", "repair.exe", "wowerror.exe", "launcher.exe", "background downloader.exe"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClientKind {
    /// Aucun client reconnu dans le dossier.
    #[default]
    None,
    /// WoW.exe 1.12.1 d'origine (32 bits).
    Original,
    /// Benilla, client libre 64 bits.
    Benilla,
}

#[derive(Clone, Debug, Default)]
pub struct ClientInfo {
    pub dir: PathBuf,
    pub kind: ClientKind,
    /// Exécutable à lancer (`WoW.exe`, `HomeCraft.exe`, `benilla.exe`…).
    pub exe_name: String,
    pub exe_present: bool,
    /// Version de WoW.exe (client d'origine).
    pub version: Option<(u16, u16, u16, u16)>,
    /// Version de l'exécutable Benilla (« 0.2.0 »).
    pub benilla_version: Option<String>,
    pub missing_mpq: Vec<String>,
}

impl ClientInfo {
    pub fn is_ok(&self) -> bool {
        match self.kind {
            ClientKind::Original => {
                let build_ok = matches!(self.version, Some((1, 12, 1, b)) if crate::server::CLIENT_BUILDS.contains(&b));
                self.exe_present && build_ok && self.missing_mpq.is_empty()
            }
            ClientKind::Benilla => self.exe_present && self.missing_mpq.is_empty(),
            ClientKind::None => false,
        }
    }
    pub fn is_benilla(&self) -> bool {
        self.kind == ClientKind::Benilla
    }
    pub fn version_string(&self) -> String {
        match self.kind {
            ClientKind::Benilla => format!("{} (64 bits, {})", self.benilla_version.as_deref().unwrap_or("?"), self.exe_name),
            _ => match self.version {
                Some((a, b, c, d)) => format!("{a}.{b}.{c}.{d}"),
                None => "?".into(),
            },
        }
    }
    /// Libellé du type de client, pour l'interface.
    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            ClientKind::Original => crate::i18n::fr::CLIENT_KIND_ORIGINAL,
            ClientKind::Benilla => crate::i18n::fr::CLIENT_KIND_BENILLA,
            ClientKind::None => crate::i18n::fr::CLIENT_KIND_NONE,
        }
    }
}

/// Ressource de version d'un exécutable via l'API Windows, sans dépendance C : chaîne « FileVersion »
/// (StringFileInfo, première traduction) et bloc binaire VS_FIXEDFILEINFO.
fn version_resource(path: &Path) -> (Option<String>, Option<(u16, u16, u16, u16)>) {
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO};
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
    let file: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let mut handle = 0u32;
        let size = GetFileVersionInfoSizeW(file.as_ptr(), &mut handle);
        if size == 0 {
            return (None, None);
        }
        let mut data = vec![0u8; size as usize];
        if GetFileVersionInfoW(file.as_ptr(), 0, size, data.as_mut_ptr() as *mut _) == 0 {
            return (None, None);
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
        let mut text = None;
        if let Some((ptr, len)) = query("\\VarFileInfo\\Translation") {
            if len >= 4 {
                let lang = *(ptr as *const u16);
                let cp = *(ptr as *const u16).add(1);
                if let Some((sptr, slen)) = query(&format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\FileVersion")) {
                    let s = std::slice::from_raw_parts(sptr as *const u16, slen as usize);
                    text = Some(String::from_utf16_lossy(s).trim_end_matches('\0').trim().to_string());
                }
            }
        }
        let fixed = query("\\").map(|(ptr, _)| {
            let info = &*(ptr as *const VS_FIXEDFILEINFO);
            ((info.dwFileVersionMS >> 16) as u16, (info.dwFileVersionMS & 0xffff) as u16, (info.dwFileVersionLS >> 16) as u16, (info.dwFileVersionLS & 0xffff) as u16)
        });
        (text, fixed)
    }
}

/// Version d'un WoW.exe. On lit d'abord la chaîne « FileVersion » (« 1, 12, 1, 5875 ») car le bloc binaire
/// du client frFR annonce 1.12.1.5880 ; repli sur le bloc binaire si la chaîne est absente.
pub fn file_version(path: &Path) -> Option<(u16, u16, u16, u16)> {
    let (text, fixed) = version_resource(path);
    if let Some(t) = text {
        let nums: Vec<u16> = t.split(|c: char| !c.is_ascii_digit()).filter(|p| !p.is_empty()).filter_map(|p| p.parse().ok()).collect();
        if nums.len() >= 4 {
            return Some((nums[0], nums[1], nums[2], nums[3]));
        }
    }
    fixed
}

/// Exécutable Benilla d'un dossier : `benilla.exe`, sinon `HomeCraft.exe`, sinon le seul autre `.exe` de la racine,
/// à condition que le dossier ait l'allure d'une installation Benilla (`benilla-config\`, ou les licences
/// MIT + Apache livrées avec Benilla, ou un exécutable nommé benilla).
fn find_benilla_exe(dir: &Path) -> Option<String> {
    let exes: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.to_ascii_lowercase().ends_with(".exe") && !NOT_A_CLIENT.contains(&n.to_ascii_lowercase().as_str()))
        .collect();
    let named = |want: &str| exes.iter().find(|n| n.eq_ignore_ascii_case(want)).cloned();
    let looks_benilla = dir.join(BENILLA_HOME).is_dir() || (dir.join("LICENSE-MIT").is_file() && dir.join("LICENSE-APACHE").is_file());
    if let Some(n) = named("benilla.exe") {
        return Some(n);
    }
    if !looks_benilla {
        return None;
    }
    named("HomeCraft.exe").or_else(|| if exes.len() == 1 { exes.first().cloned() } else { None })
}

pub fn inspect(dir: &Path) -> ClientInfo {
    let data = dir.join("Data");
    let missing = |list: &[&str]| list.iter().filter(|m| !data.join(m).is_file()).map(|m| m.to_string()).collect::<Vec<_>>();
    let wow = dir.join("WoW.exe");
    if wow.is_file() {
        return ClientInfo { dir: dir.to_path_buf(), kind: ClientKind::Original, exe_name: "WoW.exe".into(), exe_present: true, version: file_version(&wow), benilla_version: None, missing_mpq: missing(REQUIRED_MPQ) };
    }
    if let Some(exe) = find_benilla_exe(dir) {
        let (text, fixed) = version_resource(&dir.join(&exe));
        let benilla_version = text.or_else(|| fixed.map(|(a, b, c, _)| format!("{a}.{b}.{c}")));
        return ClientInfo { dir: dir.to_path_buf(), kind: ClientKind::Benilla, exe_name: exe, exe_present: true, version: None, benilla_version, missing_mpq: missing(BENILLA_MPQ) };
    }
    ClientInfo { dir: dir.to_path_buf(), kind: ClientKind::None, exe_name: "WoW.exe".into(), exe_present: false, version: None, benilla_version: None, missing_mpq: missing(REQUIRED_MPQ) }
}

/// Clients reconnus dans les dossiers voisins de l'installation (et dans `Client\`), pour les proposer dans Paramètres.
pub fn discover(root: &Path) -> Vec<ClientInfo> {
    let mut candidates: Vec<PathBuf> = vec![root.join("Client")];
    if let Some(parent) = root.parent() {
        if let Ok(rd) = std::fs::read_dir(parent) {
            for e in rd.filter_map(Result::ok) {
                let p = e.path();
                if p.is_dir() && p != root {
                    candidates.push(p.clone());
                    // Un niveau de plus (ex. « Jeux\WoW 1.12 ») : seulement les dossiers contenant Data\.
                    if let Ok(sub) = std::fs::read_dir(&p) {
                        candidates.extend(sub.filter_map(Result::ok).map(|s| s.path()).filter(|s| s.join("Data").is_dir()));
                    }
                }
            }
        }
    }
    let mut out: Vec<ClientInfo> = candidates.iter().filter(|p| p.join("Data").is_dir()).map(|p| inspect(p)).filter(|c| c.is_ok()).collect();
    out.sort_by(|a, b| a.dir.cmp(&b.dir));
    out.dedup_by(|a, b| a.dir == b.dir);
    out
}

/// Archives d'extraction absentes du `Data\` d'un client.
pub fn extraction_missing(dir: &Path) -> Vec<String> {
    EXTRACTION_MPQ.iter().filter(|m| !dir.join("Data").join(m).is_file()).map(|m| m.to_string()).collect()
}

/// Client Benilla sans `base.MPQ` : dépose une archive de remplacement HomeCraft dans son `Data\` (seule archive
/// d'extraction qu'un dossier Benilla n'a pas ; son contenu n'est pas lu par l'extracteur). Renvoie le message à
/// journaliser si une archive a été créée. Sans effet pour le client d'origine ou s'il manque d'autres archives.
pub fn ensure_benilla_base_mpq(info: &ClientInfo) -> Result<Option<String>, String> {
    if !info.is_benilla() || extraction_missing(&info.dir) != ["base.MPQ"] {
        return Ok(None);
    }
    let out = info.dir.join("Data").join("base.MPQ");
    crate::mpqbuild::build_placeholder(&out)?;
    Ok(Some(format!("base.MPQ absente du client Benilla : archive de remplacement HomeCraft créée ({}), sans contenu du jeu, ignorée par Benilla", out.display())))
}

/// Dossier client à utiliser pour extraire les données du serveur : le client choisi s'il a toutes les archives
/// d'extraction, sinon un autre client complet trouvé près de l'installation (lecture seule, rien n'est copié).
/// Erreur explicite sinon.
pub fn extraction_source(selected: &ClientInfo, root: &Path) -> Result<PathBuf, String> {
    let missing = extraction_missing(&selected.dir);
    if missing.is_empty() {
        return Ok(selected.dir.clone());
    }
    if let Some(other) = discover(root).into_iter().find(|c| c.dir != selected.dir && extraction_missing(&c.dir).is_empty()) {
        return Ok(other.dir);
    }
    Err(format!("{} {} ({}). {}", crate::i18n::fr::EXTRACT_MISSING, missing.join(", "), selected.dir.join("Data").display(), crate::i18n::fr::EXTRACT_MISSING_HINT))
}

/// Taille de l'écran principal (pixels).
pub fn primary_screen() -> (u32, u32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe { (GetSystemMetrics(SM_CXSCREEN).max(640) as u32, GetSystemMetrics(SM_CYSCREEN).max(480) as u32) }
}

/// Résolution sûre pour le client : en fenêtré, la fenêtre (bordures comprises) doit tenir dans l'écran ;
/// en plein écran, ne jamais dépasser l'écran. Choisie parmi les modes 4:3 / 16:9 / 16:10 connus du client 1.12.1.
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

fn parse_res(v: &str) -> Option<(u32, u32)> {
    let (w, h) = v.trim().trim_matches('"').split_once('x')?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

/// Configure le client avant le lancement, selon son type.
pub fn configure(info: &ClientInfo, windowed: bool) -> Result<(), String> {
    match info.kind {
        ClientKind::Benilla => configure_benilla(&info.dir, windowed),
        _ => configure_original(&info.dir, windowed),
    }
}

/// Client d'origine : realmlist.wtf -> 127.0.0.1 (original conservé une fois) ; Config.wtf fusionné clé par clé.
/// La résolution est bornée à l'écran (un client 1.12.1 en fenêtré 1920x1080 sur un écran 1920x1080 donne un écran noir).
fn configure_original(dir: &Path, windowed: bool) -> Result<(), String> {
    let rl = dir.join("realmlist.wtf");
    let orig = dir.join("realmlist.wtf.orig");
    if rl.is_file() && !orig.exists() {
        std::fs::copy(&rl, &orig).map_err(|e| e.to_string())?;
    }
    std::fs::write(&rl, "set realmlist 127.0.0.1\r\nset patchlist 127.0.0.1").map_err(|e| format!("realmlist.wtf : {e}"))?;

    let wtf = dir.join("WTF");
    std::fs::create_dir_all(&wtf).map_err(|e| e.to_string())?;
    let cfg = wtf.join("Config.wtf");
    let existing = std::fs::read_to_string(&cfg).unwrap_or_default();
    let current_res = existing.lines().find_map(|l| parse_res(l.trim().strip_prefix("SET gxResolution ")?));
    let (rw, rh) = safe_resolution(current_res, windowed);
    let managed: [(&str, String); 6] = [
        ("locale", "frFR".into()),
        ("realmList", "127.0.0.1".into()),
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

/// Benilla : `benilla-config\config.toml`, section `[cvars]`, fusionnée clé par clé (le reste du fichier,
/// commentaires et autres sections compris, est conservé ; Benilla relit ces valeurs au démarrage).
/// `gxWindow` "1" = fenêtré, "0" = plein écran sans bordure ; `gxResolution` n'est que la taille de la fenêtre.
fn configure_benilla(dir: &Path, windowed: bool) -> Result<(), String> {
    let home = dir.join(BENILLA_HOME);
    std::fs::create_dir_all(&home).map_err(|e| format!("{BENILLA_HOME} : {e}"))?;
    let cfg = home.join("config.toml");
    let existing = std::fs::read_to_string(&cfg).unwrap_or_default();
    let current_res = existing.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        if k.trim() == "gxResolution" { parse_res(v) } else { None }
    });
    let (rw, rh) = safe_resolution(current_res, true);
    let managed: Vec<(&str, String)> = vec![
        ("realmList", format!("127.0.0.1:{}", crate::paths::REALMD_PORT)),
        ("realmName", REALM_NAME.into()),
        ("gxWindow", if windowed { "1" } else { "0" }.into()),
        ("gxResolution", format!("{rw}x{rh}")),
    ];
    std::fs::write(&cfg, merge_cvars_toml(&existing, &managed)).map_err(|e| format!("config.toml : {e}"))
}

/// Remplace ou ajoute des clés dans la section `[cvars]` d'un `config.toml`, sans toucher au reste.
pub fn merge_cvars_toml(existing: &str, managed: &[(&str, String)]) -> String {
    let quote = |v: &str| format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""));
    let mut out: Vec<String> = Vec::new();
    let mut done = vec![false; managed.len()];
    let mut in_cvars = false;
    let mut cvars_seen = false;
    let flush = |out: &mut Vec<String>, done: &mut Vec<bool>| {
        for (i, (k, v)) in managed.iter().enumerate() {
            if !done[i] {
                out.push(format!("{k} = {}", quote(v)));
                done[i] = true;
            }
        }
    };
    for line in existing.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            if in_cvars {
                // Fin de [cvars] : les clés absentes y sont ajoutées avant la section suivante.
                while out.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
                    out.pop();
                }
                flush(&mut out, &mut done);
                out.push(String::new());
            }
            in_cvars = t == "[cvars]";
            cvars_seen |= in_cvars;
            out.push(line.to_string());
            continue;
        }
        if in_cvars {
            if let Some((k, _)) = t.split_once('=') {
                if let Some(i) = managed.iter().position(|(m, _)| *m == k.trim()) {
                    out.push(format!("{} = {}", managed[i].0, quote(&managed[i].1)));
                    done[i] = true;
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }
    if in_cvars {
        flush(&mut out, &mut done);
    } else if !cvars_seen {
        if out.last().map(|l| !l.trim().is_empty()).unwrap_or(false) {
            out.push(String::new());
        }
        out.push("[cvars]".into());
        flush(&mut out, &mut done);
    }
    out.join("\n") + "\n"
}

const VF_FILES: &[&str] = &["VanillaFixes.exe", "VfPatcher.dll"];
const VF_OPTIONAL: &[&str] = &["dlls.txt"]; // copié seulement s'il n'existe pas : l'utilisateur peut y ajouter ses DLL
const DXVK_DLL: &str = "d3d9.dll";
const DXVK_CONF: &str = "dxvk.conf";

fn same_file(a: &Path, b: &Path) -> bool {
    match (crate::util::sha256_file(a), crate::util::sha256_file(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

/// Dépose (ou retire) VanillaFixes et DXVK dans le dossier du client d'origine selon les options.
/// Renvoie `true` si le jeu doit être lancé via VanillaFixes.exe. Ne touche jamais aux fichiers Blizzard ;
/// ne retire `d3d9.dll` que s'il s'agit de la copie DXVK fournie (même empreinte). Sans objet pour Benilla.
pub fn apply_vanillafixes(client_dir: &Path, tools_dir: &Path, vanillafixes: bool, dxvk: bool) -> Result<bool, String> {
    let have_tools = VF_FILES.iter().all(|f| tools_dir.join(f).is_file());
    if vanillafixes {
        if !have_tools {
            return Err(crate::i18n::fr::SET_VF_MISSING.to_string());
        }
        for f in VF_FILES {
            let (src, dst) = (tools_dir.join(f), client_dir.join(f));
            if !dst.is_file() || !same_file(&src, &dst) {
                std::fs::copy(&src, &dst).map_err(|e| format!("{f} : {e}"))?;
            }
        }
        for f in VF_OPTIONAL {
            let dst = client_dir.join(f);
            if !dst.exists() {
                let _ = std::fs::copy(tools_dir.join(f), dst);
            }
        }
    }
    let (src_dll, dst_dll) = (tools_dir.join(DXVK_DLL), client_dir.join(DXVK_DLL));
    if dxvk && src_dll.is_file() {
        if !dst_dll.is_file() || !same_file(&src_dll, &dst_dll) {
            std::fs::copy(&src_dll, &dst_dll).map_err(|e| format!("{DXVK_DLL} : {e}"))?;
        }
        if !client_dir.join(DXVK_CONF).exists() {
            let _ = std::fs::copy(tools_dir.join(DXVK_CONF), client_dir.join(DXVK_CONF));
        }
    } else if dst_dll.is_file() && src_dll.is_file() && same_file(&src_dll, &dst_dll) {
        std::fs::remove_file(&dst_dll).map_err(|e| format!("{DXVK_DLL} : {e}"))?;
    }
    Ok(vanillafixes)
}

/// Lance le client : VanillaFixes.exe si demandé (client d'origine), sinon l'exécutable du client.
pub fn launch(info: &ClientInfo, via_vanillafixes: bool) -> Result<Child, String> {
    let exe = if via_vanillafixes { "VanillaFixes.exe".to_string() } else { info.exe_name.clone() };
    Command::new(info.dir.join(&exe))
        .current_dir(&info.dir)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| crate::util::spawn_error(&exe, &e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cvars_merge_replaces_adds_and_keeps_the_rest() {
        let src = "# entête\n[cvars]\nrealmList = \"1.2.3.4\"\nfarclip = \"777\"\n\n[autre]\nx = 1\n";
        let out = merge_cvars_toml(src, &[("realmList", "127.0.0.1:3724".into()), ("gxWindow", "1".into())]);
        assert!(out.contains("# entête"));
        assert!(out.contains("realmList = \"127.0.0.1:3724\""));
        assert!(out.contains("farclip = \"777\""));
        assert!(out.contains("gxWindow = \"1\""));
        assert!(out.contains("[autre]\nx = 1"));
        assert!(out.find("gxWindow").unwrap() < out.find("[autre]").unwrap(), "clé ajoutée dans [cvars]");
        assert!(!out.contains("1.2.3.4"));
    }

    #[test]
    fn cvars_merge_creates_the_section() {
        let out = merge_cvars_toml("", &[("realmList", "127.0.0.1".into())]);
        assert_eq!(out, "[cvars]\nrealmList = \"127.0.0.1\"\n");
    }

    #[test]
    fn benilla_detected_by_its_config_folder() {
        let tmp = std::env::temp_dir().join(format!("hc-benilla-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("Data")).unwrap();
        std::fs::create_dir_all(tmp.join(BENILLA_HOME)).unwrap();
        std::fs::write(tmp.join("HomeCraft.exe"), b"MZ").unwrap();
        for m in BENILLA_MPQ {
            std::fs::write(tmp.join("Data").join(m), b"").unwrap();
        }
        let info = inspect(&tmp);
        assert_eq!(info.kind, ClientKind::Benilla);
        assert_eq!(info.exe_name, "HomeCraft.exe");
        assert!(info.is_ok(), "base.MPQ n'est pas exigé pour Benilla");
        configure(&info, true).unwrap();
        let cfg = std::fs::read_to_string(tmp.join(BENILLA_HOME).join("config.toml")).unwrap();
        assert!(cfg.contains("realmList = \"127.0.0.1:3724\""));
        assert!(cfg.contains("gxWindow = \"1\""));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn a_plain_folder_with_an_exe_is_not_benilla() {
        let tmp = std::env::temp_dir().join(format!("hc-notbenilla-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("Data")).unwrap();
        std::fs::write(tmp.join("Setup.exe"), b"MZ").unwrap();
        assert_eq!(inspect(&tmp).kind, ClientKind::None);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn benilla_without_base_mpq_needs_another_source_for_extraction() {
        let tmp = std::env::temp_dir().join(format!("hc-extract-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let root = tmp.join("HomeCraft");
        let benilla = tmp.join("Benilla");
        std::fs::create_dir_all(root.join("Server")).unwrap();
        std::fs::create_dir_all(benilla.join("Data")).unwrap();
        std::fs::create_dir_all(benilla.join(BENILLA_HOME)).unwrap();
        std::fs::write(benilla.join("HomeCraft.exe"), b"MZ").unwrap();
        for m in BENILLA_MPQ {
            std::fs::write(benilla.join("Data").join(m), b"").unwrap();
        }
        let info = inspect(&benilla);
        assert!(info.is_ok());
        assert_eq!(extraction_missing(&benilla), vec!["base.MPQ".to_string()]);
        let err = extraction_source(&info, &root).unwrap_err();
        assert!(err.contains("base.MPQ"), "{err}");
        // Archive de remplacement créée pour Benilla, puis le client devient une source d'extraction valide.
        assert!(ensure_benilla_base_mpq(&info).unwrap().is_some());
        let base = benilla.join("Data").join("base.MPQ");
        let bytes = std::fs::read(&base).unwrap();
        assert_eq!(&bytes[..4], b"MPQ\x1a", "en-tête MPQ");
        assert!(ensure_benilla_base_mpq(&info).unwrap().is_none(), "rien à faire la seconde fois");
        assert_eq!(extraction_source(&info, &root).unwrap(), benilla);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Le dossier de l'utilisateur, s'il existe sur ce poste.
    #[test]
    fn real_benilla_folder_if_present() {
        let dir = Path::new(r"C:\Users\Marty\Desktop\WoW Client FR");
        if !dir.is_dir() {
            return;
        }
        let info = inspect(dir);
        assert_eq!(info.kind, ClientKind::Benilla, "{info:?}");
        assert!(info.is_ok(), "{info:?}");
        println!("{} — {}", info.dir.display(), info.version_string());
    }
}
