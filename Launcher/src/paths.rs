//! Disposition des dossiers et ports du projet. Tout est relatif à la racine d'installation,
//! détectée depuis l'emplacement de l'exécutable (ou forcée par la variable HOMECRAFT_WOTLK_ROOT).
//!
//! Particularité AzerothCore sous Windows : `authserver -c <chemin>` refuse un chemin contenant un espace
//! (boost::program_options + fs::path), et les configurations de modules sont cherchées dans
//! `<dossier courant>/configs/modules/`. Les serveurs sont donc lancés depuis `Server\acore` avec un chemin
//! « court » de la racine (nom 8.3, ou jonction NTFS en repli) : voir `short_root`.

use std::path::{Path, PathBuf};

pub const DB_PORT: u16 = 3306; // port par défaut pré-rempli par Keira3 (décision du 2026-10-17 ; avant : 3315, partagé avec l'édition Vanilla)
pub const AUTH_PORT: u16 = 3724;
pub const WORLD_PORT: u16 = 8085;
/// HermesProxy (client 3.4.3) : Battle.net, REST, royaume, instance.
pub const HERMES_BNET_PORT: u16 = 1119;
pub const HERMES_REST_PORT: u16 = 8081;
pub const HERMES_REALM_PORT: u16 = 8084;
pub const HERMES_INSTANCE_PORT: u16 = 8086;
pub const CLIENT_BUILD: u16 = 12340;
pub const CLASSIC_CLIENT_BUILD: u32 = 54261;
pub const REALM_NAME: &str = "HomeCraft";

pub const DB_AUTH: &str = "acore_auth";
pub const DB_CHARACTERS: &str = "acore_characters";
pub const DB_WORLD: &str = "acore_world";
pub const DB_PLAYERBOTS: &str = "acore_playerbots";
pub const ALL_DATABASES: [&str; 4] = [DB_AUTH, DB_CHARACTERS, DB_WORLD, DB_PLAYERBOTS];

#[derive(Clone, Debug)]
pub struct Layout {
    pub root: PathBuf,
}

impl Layout {
    pub fn detect() -> Layout {
        if let Ok(r) = std::env::var("HOMECRAFT_WOTLK_ROOT") {
            return Layout { root: PathBuf::from(r) };
        }
        let exe = std::env::current_exe().ok();
        let mut dir = exe.as_deref().and_then(Path::parent).map(Path::to_path_buf);
        while let Some(d) = dir {
            if d.join("Server").is_dir() && d.join("Tools").is_dir() {
                return Layout { root: d };
            }
            dir = d.parent().map(Path::to_path_buf);
        }
        Layout { root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")) }
    }

    // Programme (remplaçable à la mise à jour)
    pub fn server_bin(&self) -> PathBuf { self.root.join("Server").join("acore") }
    /// Modèles `.conf.dist` et `.conf` générés ; `modules/` pour les modules (emplacement imposé par AzerothCore).
    pub fn server_config(&self) -> PathBuf { self.server_bin().join("configs") }
    pub fn server_data(&self) -> PathBuf { self.root.join("Server").join("Data") }
    pub fn mysql_dir(&self) -> PathBuf { self.root.join("Server").join("Database").join("mysql") }
    pub fn mysql_bin(&self) -> PathBuf { self.mysql_dir().join("bin") }
    pub fn my_ini(&self) -> PathBuf { self.root.join("Server").join("Config").join("my.ini") }
    pub fn sql_dumps(&self) -> PathBuf { self.root.join("Server").join("Database").join("sql") }
    /// Compléments SQL appliqués après l'import d'une base : `custom/<base>/*.sql` (ordre alphabétique).
    pub fn sql_custom(&self) -> PathBuf { self.root.join("Server").join("Database").join("custom") }
    pub fn heidisql_exe(&self) -> PathBuf { self.root.join("Tools").join("HeidiSQL").join("heidisql.exe") }
    /// Keira3 (éditeur de la base monde AzerothCore), exécutable portable compilé depuis la balise v3.10.5
    pub fn keira_exe(&self) -> PathBuf { self.root.join("Tools").join("Keira3").join("Keira3.exe") }
    pub fn hermes_dir(&self) -> PathBuf { self.root.join("Tools").join("HermesProxy") }
    pub fn default_client_dir(&self) -> PathBuf { self.root.join("Client") }

    // Données personnelles (jamais écrasées par une mise à jour)
    pub fn user_data(&self) -> PathBuf { self.root.join("UserData") }
    pub fn db_data(&self) -> PathBuf { self.user_data().join("Database") }
    pub fn credentials_file(&self) -> PathBuf { self.user_data().join("db-credentials.json") }
    pub fn settings_file(&self) -> PathBuf { self.user_data().join("settings.json") }
    pub fn backups(&self) -> PathBuf { self.user_data().join("Sauvegardes") }

    // Journaux
    pub fn logs(&self) -> PathBuf { self.root.join("Logs") }
    pub fn server_logs(&self) -> PathBuf { self.logs().join("server") }
    pub fn launcher_log(&self) -> PathBuf { self.logs().join("launcher.log") }

    /// Dépôt source du cœur (data/sql/base, updates, modules/*/data/sql) pour l'amorçage automatique des bases
    /// par l'updater d'AzerothCore. Présent sur le poste de développement seulement (clone `Build/src/…`) ;
    /// absent du package : les dumps font alors foi et l'updater est désactivé.
    pub fn core_source_dir(&self) -> Option<PathBuf> {
        let candidates = [self.root.join("Build").join("src").join("sylvaniacore-wotlk"), PathBuf::from(r"C:\hc")];
        candidates.into_iter().find(|p| p.join("data").join("sql").join("base").is_dir())
    }

    /// Les composants gérés sont-ils présents ? (état NON INSTALLÉ sinon)
    pub fn components_present(&self) -> Vec<(String, bool)> {
        let checks = [
            ("worldserver.exe", self.server_bin().join("worldserver.exe")),
            ("authserver.exe", self.server_bin().join("authserver.exe")),
            ("mysqld.exe", self.mysql_bin().join("mysqld.exe")),
            ("worldserver.conf.dist", self.server_config().join("worldserver.conf.dist")),
            ("authserver.conf.dist", self.server_config().join("authserver.conf.dist")),
        ];
        let mut out: Vec<(String, bool)> = checks.iter().map(|(n, p)| (n.to_string(), p.is_file())).collect();
        out.push(("dump acore_world".into(), crate::db::find_dump(&self.sql_dumps(), "world").is_some()));
        out
    }

    /// Le support est-il inscriptible ? (faux sur un CD/DVD ou un dossier protégé : il faut copier sur le disque)
    pub fn is_writable(&self) -> bool {
        let probe_dir = self.user_data();
        if std::fs::create_dir_all(&probe_dir).is_err() {
            return false;
        }
        let probe = probe_dir.join(".ecriture-test");
        let ok = std::fs::write(&probe, b"ok").is_ok();
        let _ = std::fs::remove_file(&probe);
        ok
    }

    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Chemin au format attendu par les .conf d'AzerothCore (séparateurs « / »).
    pub fn conf_path(p: &Path) -> String {
        p.to_string_lossy().replace('\\', "/")
    }

    /// Racine sans espace pour les arguments de ligne de commande des serveurs : la racine elle-même si elle
    /// n'en contient pas, sinon son nom court 8.3 (GetShortPathNameW), sinon une jonction NTFS
    /// `<lecteur>:\HomeCraftW` créée vers la racine (sans droits administrateur).
    pub fn short_root(&self) -> Result<PathBuf, String> {
        let s = self.root.to_string_lossy();
        if !s.contains(' ') {
            return Ok(self.root.clone());
        }
        if let Some(short) = short_path_name(&self.root) {
            if !short.to_string_lossy().contains(' ') && short.join("Server").is_dir() {
                return Ok(short);
            }
        }
        let drive = s.chars().next().unwrap_or('C');
        let junction = PathBuf::from(format!("{drive}:\\HomeCraftW"));
        if junction.join("Server").join("acore").is_dir() {
            return Ok(junction);
        }
        if junction.exists() {
            return Err(format!("{} existe déjà et ne pointe pas sur cette installation", junction.display()));
        }
        use std::os::windows::process::CommandExt;
        let st = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(&junction)
            .arg(&self.root)
            .creation_flags(crate::db::CREATE_NO_WINDOW)
            .status()
            .map_err(|e| format!("mklink : {e}"))?;
        if st.success() && junction.join("Server").join("acore").is_dir() {
            Ok(junction)
        } else {
            Err(format!("impossible de créer la jonction {} (le chemin d'installation contient un espace)", junction.display()))
        }
    }
}

/// Nom court 8.3 d'un chemin (None si indisponible sur ce volume).
pub fn short_path_name(p: &Path) -> Option<PathBuf> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows_sys::Win32::Storage::FileSystem::GetShortPathNameW;
    let wide: Vec<u16> = p.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let n = GetShortPathNameW(wide.as_ptr(), std::ptr::null_mut(), 0);
        if n == 0 {
            return None;
        }
        let mut buf = vec![0u16; n as usize + 1];
        let m = GetShortPathNameW(wide.as_ptr(), buf.as_mut_ptr(), buf.len() as u32);
        if m == 0 || m as usize > buf.len() {
            return None;
        }
        buf.truncate(m as usize);
        Some(PathBuf::from(std::ffi::OsString::from_wide(&buf)))
    }
}
