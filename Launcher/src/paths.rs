//! Disposition des dossiers et ports du projet. Tout est relatif à la racine d'installation,
//! détectée depuis l'emplacement de l'exécutable (ou forcée par la variable HOMECRAFT_BC_ROOT).

use std::path::{Path, PathBuf};

pub const MARIADB_PORT: u16 = 3317;
pub const REALMD_PORT: u16 = 3726;
pub const WORLD_PORT: u16 = 8087;
pub const CLIENT_BUILD: (u16, u16, u16, u16) = (2, 4, 3, 8606);
pub const REALM_NAME: &str = "HomeCraft BC";
// Bases CMaNGOS-tbc (convention <era><base> de TBC-DB), une par rôle. Le dump de chacune porte son nom (<base>.sql).
pub const DB_REALMD: &str = "tbcrealmd";
pub const DB_CHARACTERS: &str = "tbccharacters";
pub const DB_WORLD: &str = "tbcmangos";
pub const DB_LOGS: &str = "tbclogs";
/// Table de version du schéma de chaque base (remplace la table `migrations` de VMaNGOS).
pub const DB_VERSION_TABLES: [(&str, &str); 4] = [(DB_REALMD, "realmd_db_version"), (DB_CHARACTERS, "character_db_version"), (DB_WORLD, "db_version"), (DB_LOGS, "logs_db_version")];

#[derive(Clone, Debug)]
pub struct Layout {
    pub root: PathBuf,
}

impl Layout {
    pub fn detect() -> Layout {
        if let Ok(r) = std::env::var("HOMECRAFT_BC_ROOT") {
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
    pub fn server_bin(&self) -> PathBuf { self.root.join("Server").join("mangos-tbc") }
    pub fn server_config(&self) -> PathBuf { self.root.join("Server").join("Config") }
    pub fn server_data(&self) -> PathBuf { self.root.join("Server").join("Data") }
    pub fn extractors(&self) -> PathBuf { self.server_bin().join("Extractors") }
    /// Vue « archives officielles seules » du client pour les extracteurs (liens physiques, voir repair.rs).
    pub fn extract_view(&self) -> PathBuf { self.user_data().join("extract-view") }
    pub fn mariadb_dir(&self) -> PathBuf { self.root.join("Server").join("Database").join("mariadb") }
    pub fn mariadb_bin(&self) -> PathBuf { self.mariadb_dir().join("bin") }
    pub fn sql_dumps(&self) -> PathBuf { self.root.join("Server").join("Database").join("sql") }
    pub fn heidisql_exe(&self) -> PathBuf { self.root.join("Tools").join("HeidiSQL").join("heidisql.exe") }
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

    /// Les composants gérés sont-ils présents ? (état NON INSTALLÉ sinon)
    pub fn components_present(&self) -> Vec<(String, bool)> {
        let checks = [
            ("mangosd.exe", self.server_bin().join("mangosd.exe")),
            ("realmd.exe", self.server_bin().join("realmd.exe")),
            ("mariadbd.exe", self.mariadb_bin().join("mariadbd.exe")),
            ("mangosd.conf.dist", self.server_bin().join("mangosd.conf.dist")),
            ("realmd.conf.dist", self.server_bin().join("realmd.conf.dist")),
            ("aiplayerbot.conf.dist", self.server_bin().join("aiplayerbot.conf.dist")),
            ("ahbot.conf.dist", self.server_bin().join("ahbot.conf.dist")),
            ("ad.exe", self.extractors().join("ad.exe")),
            ("tbcmangos.sql", self.sql_dumps().join("tbcmangos.sql")),
        ];
        checks.iter().map(|(n, p)| (n.to_string(), p.is_file())).collect()
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

    /// Chemin au format attendu par les .conf de CMaNGOS (séparateurs « / »).
    pub fn conf_path(p: &Path) -> String {
        p.to_string_lossy().replace('\\', "/")
    }
}
