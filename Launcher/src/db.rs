//! MySQL 8.4 portable : configuration (my.ini), initialisation du datadir, démarrage, santé, arrêt propre,
//! import des dumps du royaume, requêtes de l'explorateur. AzerothCore exige MySQL 8 (MariaDB refusée).

use crate::credentials::DbCredentials;
use crate::paths::{Layout, ALL_DATABASES, DB_AUTH, DB_CHARACTERS, DB_PLAYERBOTS, DB_PORT, DB_WORLD};
use mysql::prelude::*;
use mysql::{Conn, OptsBuilder, Row, Value};
use std::net::{SocketAddr, TcpStream};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone)]
pub struct MySql {
    layout: Layout,
    creds: DbCredentials,
}

#[derive(Clone, Debug, Default)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub truncated: bool,
    /// Pour une instruction d'écriture : lignes affectées
    pub affected: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub primary: bool,
    pub default: Option<String>,
    pub extra: String,
}

/// Littéral SQL à partir d'une saisie : `NULL` (insensible à la casse) → NULL, sinon chaîne échappée.
pub fn sql_literal(v: &str) -> String {
    if v.eq_ignore_ascii_case("null") {
        "NULL".into()
    } else {
        format!("'{}'", v.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

pub fn is_read_statement(sql: &str) -> bool {
    let head = sql.trim_start().to_ascii_uppercase();
    head.starts_with("SELECT") || head.starts_with("SHOW") || head.starts_with("DESCRIBE") || head.starts_with("EXPLAIN") || head.starts_with("DESC ")
}

pub fn port_open(port: u16) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(400)).is_ok()
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::NULL => "NULL".to_string(),
        Value::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
        Value::Int(i) => i.to_string(),
        Value::UInt(u) => u.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Double(d) => d.to_string(),
        other => other.as_sql(false),
    }
}

/// Mot-clé du nom de fichier de dump pour chaque base (`acore_world.sql.gz`, `acore_auth_schema.sql`, …).
fn dump_keyword(db: &str) -> &'static str {
    match db {
        DB_AUTH => "auth",
        DB_CHARACTERS => "characters",
        DB_WORLD => "world",
        _ => "playerbots",
    }
}

/// Dump `.sql` ou `.sql.gz` dont le nom contient le mot-clé (le plus gros si plusieurs).
pub fn find_dump(dir: &Path, keyword: &str) -> Option<PathBuf> {
    let rd = std::fs::read_dir(dir).ok()?;
    let mut best: Option<(u64, PathBuf)> = None;
    for e in rd.filter_map(Result::ok) {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_ascii_lowercase();
        let is_sql = name.ends_with(".sql") || name.ends_with(".sql.gz");
        let stem = name.trim_end_matches(".gz").trim_end_matches(".sql");
        let parts: Vec<&str> = stem.split(|c: char| c == '_' || c == '-' || c == '.').collect();
        if !is_sql || !parts.iter().any(|p| *p == keyword) {
            continue;
        }
        let size = e.metadata().map(|m| m.len()).unwrap_or(0);
        if best.as_ref().map(|(s, _)| size > *s).unwrap_or(true) {
            best = Some((size, p));
        }
    }
    best.map(|(_, p)| p)
}

impl MySql {
    pub fn new(layout: Layout, creds: DbCredentials) -> MySql {
        MySql { layout, creds }
    }

    pub fn creds(&self) -> &DbCredentials {
        &self.creds
    }

    fn ini_path(&self) -> PathBuf {
        self.layout.my_ini()
    }

    /// my.ini géré : régénéré à chaque démarrage, aucune donnée personnelle dedans. Mêmes réglages que Tools/Start-Database.ps1.
    pub fn write_my_ini(&self) -> Result<(), String> {
        let l = &self.layout;
        std::fs::create_dir_all(l.db_data()).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(l.logs()).map_err(|e| e.to_string())?;
        if let Some(p) = self.ini_path().parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let ini = format!(
            "[mysqld]\nbasedir={base}\ndatadir={data}\nport={port}\nbind-address=127.0.0.1\nmysqlx=OFF\nskip-name-resolve\n\
max_allowed_packet=1G\ncharacter-set-server=utf8mb4\ncollation-server=utf8mb4_unicode_ci\n\
innodb_buffer_pool_size=1G\ninnodb_redo_log_capacity=512M\ninnodb_flush_log_at_trx_commit=2\nsql_mode=NO_ENGINE_SUBSTITUTION\n\
log_error={log}\npid-file={pid}\nlog_bin=OFF\n\n[client]\nport={port}\ndefault-character-set=utf8mb4\n",
            base = Layout::conf_path(&l.mysql_dir()),
            data = Layout::conf_path(&l.db_data()),
            port = DB_PORT,
            log = Layout::conf_path(&l.logs().join("mysql-error.log")),
            pid = Layout::conf_path(&l.db_data().join("mysqld.pid")),
        );
        std::fs::write(self.ini_path(), ini).map_err(|e| e.to_string())
    }

    pub fn is_initialized(&self) -> bool {
        self.layout.db_data().join("mysql").is_dir()
    }

    /// Premier lancement : création du datadir (`--initialize-insecure` : root sans mot de passe ; le mot de passe
    /// est posé au premier démarrage par un fichier --init-file, voir `start`).
    pub fn initialize(&self) -> Result<(), String> {
        let exe = self.layout.mysql_bin().join("mysqld.exe");
        let status = Command::new(&exe)
            .arg(format!("--defaults-file={}", self.ini_path().display()))
            .arg("--initialize-insecure")
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("mysqld --initialize-insecure : {e}"))?;
        if !status.success() {
            return Err(format!("mysqld --initialize-insecure a échoué (code {:?}), voir Logs/mysql-error.log", status.code()));
        }
        // Marqueur : le prochain démarrage doit poser le mot de passe root (datadir neuf).
        std::fs::write(self.layout.db_data().join(".root-a-initialiser"), b"1").map_err(|e| e.to_string())
    }

    fn opts(&self, root: bool, db: Option<&str>) -> OptsBuilder {
        let (u, p) = if root { ("root", self.creds.root_password.as_str()) } else { (self.creds.user.as_str(), self.creds.password.as_str()) };
        OptsBuilder::new()
            .ip_or_hostname(Some("127.0.0.1"))
            .tcp_port(DB_PORT)
            .prefer_socket(false)
            .user(Some(u))
            .pass(Some(p))
            .db_name(db)
            .tcp_connect_timeout(Some(Duration::from_secs(3)))
    }

    pub fn conn(&self, db: Option<&str>) -> Result<Conn, String> {
        Conn::new(self.opts(false, db)).map_err(|e| format!("connexion SQL : {e}"))
    }

    pub fn root_conn(&self) -> Result<Conn, String> {
        Conn::new(self.opts(true, None)).map_err(|e| format!("connexion SQL root : {e}"))
    }

    /// Santé réelle : port ouvert ET requête aboutie.
    pub fn ping(&self) -> bool {
        if !port_open(DB_PORT) {
            return false;
        }
        match self.root_conn() {
            Ok(mut c) => c.query_first::<u64, _>("SELECT 1").map(|v| v == Some(1)).unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Démarre mysqld (détaché, sans fenêtre) et attend la santé réelle.
    /// Renvoie `true` si le processus a été lancé ici, `false` s'il était déjà en service (il ne nous appartient pas).
    /// Premier démarrage après `initialize` : un fichier --init-file pose le mot de passe root et crée
    /// root@127.0.0.1 (avec skip-name-resolve, root@localhost n'accepte pas les connexions TCP) ; il est supprimé aussitôt.
    pub fn start(&self, timeout: Duration) -> Result<bool, String> {
        if self.ping() {
            self.forget_legacy_root();
            return Ok(false);
        }
        // Base déjà en service avec l'ancien mot de passe root (migration du 2026-10-17) : changement à chaud.
        if port_open(DB_PORT) {
            if let Some(old) = self.creds.legacy_root_password.as_deref() {
                let opts = self.opts(true, None).pass(Some(old));
                if let Ok(mut c) = Conn::new(opts) {
                    let pw = self.creds.root_password.replace('\'', "");
                    for user in ["'root'@'localhost'", "'root'@'127.0.0.1'"] {
                        let _ = c.query_drop(format!("ALTER USER {user} IDENTIFIED BY '{pw}'"));
                    }
                    let _ = c.query_drop("FLUSH PRIVILEGES");
                }
                if self.ping() {
                    self.forget_legacy_root();
                    return Ok(false);
                }
            }
        }
        let exe = self.layout.mysql_bin().join("mysqld.exe");
        let marker = self.layout.db_data().join(".root-a-initialiser");
        let init_file = self.layout.db_data().join("init-root.sql");
        // Datadir neuf, ou migration du mot de passe root : un --init-file (exécuté par le serveur, sans connexion,
        // donc sans connaître l'ancien mot de passe) pose le mot de passe root.
        let fresh = marker.is_file() || self.creds.legacy_root_password.is_some();
        let mut cmd = Command::new(&exe);
        cmd.arg(format!("--defaults-file={}", self.ini_path().display()));
        if fresh {
            let pw = self.creds.root_password.replace('\'', "");
            let sql = format!(
                "ALTER USER 'root'@'localhost' IDENTIFIED BY '{pw}';\nCREATE USER IF NOT EXISTS 'root'@'127.0.0.1' IDENTIFIED BY '{pw}';\n\
ALTER USER 'root'@'127.0.0.1' IDENTIFIED BY '{pw}';\nGRANT ALL PRIVILEGES ON *.* TO 'root'@'127.0.0.1' WITH GRANT OPTION;\nFLUSH PRIVILEGES;\n"
            );
            std::fs::write(&init_file, sql).map_err(|e| e.to_string())?;
            cmd.arg(format!("--init-file={}", init_file.display()));
        }
        let mut child = cmd.creation_flags(CREATE_NO_WINDOW).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e| format!("mysqld : {e}"))?;
        let start = Instant::now();
        let result = loop {
            if self.ping() {
                break Ok(true);
            }
            if let Ok(Some(st)) = child.try_wait() {
                break Err(format!("mysqld s'est arrêté (code {:?}), voir Logs/mysql-error.log", st.code()));
            }
            if start.elapsed() >= timeout {
                break Err("la base de données ne répond pas dans le délai imparti".into());
            }
            std::thread::sleep(Duration::from_millis(500));
        };
        if fresh {
            let _ = std::fs::remove_file(&init_file);
            if result.is_ok() {
                let _ = std::fs::remove_file(&marker);
                self.forget_legacy_root();
            }
        }
        result
    }

    /// Migration terminée : l'ancien mot de passe root est retiré du fichier d'identifiants.
    fn forget_legacy_root(&self) {
        if self.creds.legacy_root_password.is_none() {
            return;
        }
        let path = self.layout.credentials_file();
        if let Ok(mut c) = crate::credentials::load_or_create(&path) {
            c.legacy_root_password = None;
            let _ = crate::credentials::save(&path, &c);
        }
    }

    /// Arrêt propre par la commande SQL SHUTDOWN (jamais de kill).
    pub fn shutdown(&self, timeout: Duration) -> Result<(), String> {
        if !port_open(DB_PORT) {
            return Ok(());
        }
        let mut c = self.root_conn()?;
        let _ = c.query_drop("SHUTDOWN");
        drop(c);
        let start = Instant::now();
        while start.elapsed() < timeout {
            if !port_open(DB_PORT) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        Err("la base de données répond toujours après le délai d'arrêt".into())
    }

    pub fn characters_count(&self) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        c.query_first(format!("SELECT COUNT(*) FROM {DB_CHARACTERS}.characters")).map(|v| v.unwrap_or(0)).map_err(|e| e.to_string())
    }

    /// Personnages réels (hors comptes de bots aléatoires `rndbot*`).
    pub fn player_characters_count(&self) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        c.query_first(format!(
            "SELECT COUNT(*) FROM {DB_CHARACTERS}.characters ch JOIN {DB_AUTH}.account a ON a.id=ch.account WHERE a.username NOT LIKE 'RNDBOT%'"
        ))
        .map(|v| v.unwrap_or(0))
        .map_err(|e| e.to_string())
    }

    pub fn accounts_count(&self) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        c.query_first(format!("SELECT COUNT(*) FROM {DB_AUTH}.account WHERE username NOT LIKE 'RNDBOT%'")).map(|v| v.unwrap_or(0)).map_err(|e| e.to_string())
    }

    /// Bases absentes parmi les quatre attendues.
    pub fn databases_missing(&self) -> Result<Vec<&'static str>, String> {
        let mut c = self.root_conn()?;
        let list = ALL_DATABASES.iter().map(|d| format!("'{d}'")).collect::<Vec<_>>().join(",");
        let present: Vec<String> = c.query(format!("SELECT schema_name FROM information_schema.schemata WHERE schema_name IN ({list})")).map_err(|e| e.to_string())?;
        Ok(ALL_DATABASES.iter().copied().filter(|d| !present.iter().any(|p| p == d)).collect())
    }

    /// Bases présentes mais sans table `updates` (schéma incomplet). `acore_playerbots` est remplie par le serveur.
    pub fn databases_without_schema(&self) -> Result<Vec<String>, String> {
        let mut c = self.root_conn()?;
        let mut out = Vec::new();
        for d in [DB_AUTH, DB_CHARACTERS, DB_WORLD] {
            let n: Option<u64> = c.query_first(format!("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='{d}' AND table_name='updates'")).map_err(|e| e.to_string())?;
            if n != Some(1) {
                out.push(d.to_string());
            }
        }
        Ok(out)
    }

    fn client_args(&self) -> Vec<String> {
        vec![
            "--host=127.0.0.1".into(),
            format!("--port={DB_PORT}"),
            "--protocol=TCP".into(),
            "--user=root".into(),
            format!("--password={}", self.creds.root_password),
            "--default-character-set=utf8mb4".into(),
        ]
    }

    /// Import d'un fichier SQL (`.sql` ou `.sql.gz`) dans une base existante (client mysql.exe, entrée standard).
    pub fn import_file(&self, db: &str, file: &Path) -> Result<(), String> {
        let is_gz = file.extension().map(|x| x.eq_ignore_ascii_case("gz")).unwrap_or(false);
        let mut child = Command::new(self.layout.mysql_bin().join("mysql.exe"))
            .args(self.client_args())
            .args(["--max_allowed_packet=1073741824", "--force", db])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("mysql.exe : {e}"))?;
        {
            let mut stdin = child.stdin.take().ok_or("entrée standard de mysql.exe indisponible")?;
            let f = std::fs::File::open(file).map_err(|e| format!("{} : {e}", file.display()))?;
            let mut reader: Box<dyn std::io::Read> = if is_gz { Box::new(flate2::read::GzDecoder::new(std::io::BufReader::new(f))) } else { Box::new(std::io::BufReader::new(f)) };
            std::io::copy(&mut reader, &mut stdin).map_err(|e| format!("import {} : {e}", file.display()))?;
        }
        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            let first = err.lines().find(|l| l.contains("ERROR")).unwrap_or("").chars().take(300).collect::<String>();
            return Err(format!("import de {} échoué (code {:?}) {first}", file.display(), out.status.code()));
        }
        Ok(())
    }

    /// Dump cohérent (tables verrouillées pendant l'export).
    pub fn dump_database(&self, db: &str, out: &Path) -> Result<(), String> {
        let status = Command::new(self.layout.mysql_bin().join("mysqldump.exe"))
            .args(self.client_args())
            .args(["--lock-tables", "--add-drop-table", "--routines", "--triggers", "--skip-dump-date", "--column-statistics=0"])
            .arg(format!("--result-file={}", out.display()))
            .arg(db)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("mysqldump : {e}"))?;
        if !status.success() || !out.is_file() {
            return Err(format!("export de {db} échoué (code {:?})", status.code()));
        }
        Ok(())
    }

    pub fn online_count(&self) -> u64 {
        self.root_conn()
            .ok()
            .and_then(|mut c| c.query_first::<u64, _>(format!("SELECT COUNT(*) FROM {DB_CHARACTERS}.characters WHERE online=1")).ok().flatten())
            .unwrap_or(0)
    }

    /// Import des bases listées depuis les dumps du package (mêmes étapes que Tools/Init-Database.ps1), puis
    /// compléments `Server/Database/custom/<base>/*.sql` (ordre alphabétique) et nettoyage des objets propres au VPS.
    /// Refuse de recréer `acore_characters` si elle contient des personnages de joueurs : ils ne sont jamais écrasés ici.
    pub fn import_dumps(&self, dbs: &[&str], progress: &dyn Fn(&str)) -> Result<(), String> {
        if dbs.contains(&DB_CHARACTERS) && self.player_characters_count().unwrap_or(0) > 0 {
            return Err("la base des personnages contient des personnages : recréation refusée (restaurer une sauvegarde à la place)".into());
        }
        let user = &self.creds.user;
        let pass = &self.creds.password;
        let mut c = self.root_conn()?;
        for host in ["127.0.0.1", "localhost"] {
            c.query_drop(format!("CREATE USER IF NOT EXISTS '{user}'@'{host}' IDENTIFIED BY '{pass}'")).map_err(|e| e.to_string())?;
            c.query_drop(format!("ALTER USER '{user}'@'{host}' IDENTIFIED BY '{pass}'")).map_err(|e| e.to_string())?;
        }
        for db in ALL_DATABASES {
            if !dbs.contains(&db) {
                continue;
            }
            let dump = find_dump(&self.layout.sql_dumps(), dump_keyword(db));
            progress(&format!("création de {db}"));
            c.query_drop(format!("DROP DATABASE IF EXISTS `{db}`")).map_err(|e| e.to_string())?;
            c.query_drop(format!("CREATE DATABASE `{db}` DEFAULT CHARSET utf8mb4 COLLATE utf8mb4_unicode_ci")).map_err(|e| e.to_string())?;
            for host in ["127.0.0.1", "localhost"] {
                c.query_drop(format!("GRANT ALL PRIVILEGES ON `{db}`.* TO '{user}'@'{host}'")).map_err(|e| e.to_string())?;
            }
            match dump {
                Some(f) => {
                    progress(&format!("import de {db} ({})", f.file_name().unwrap_or_default().to_string_lossy()));
                    self.import_file(db, &f)?;
                }
                None if db == DB_PLAYERBOTS => progress("acore_playerbots : aucun dump, la base sera remplie par le serveur"),
                None => return Err(format!("dump de {db} introuvable dans {}", self.layout.sql_dumps().display())),
            }
            // Compléments du projet : Server/Database/custom/<base>/*.sql
            let custom = self.layout.sql_custom().join(db);
            if let Ok(rd) = std::fs::read_dir(&custom) {
                let mut files: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().map(|x| x == "sql").unwrap_or(false)).collect();
                files.sort();
                for f in files {
                    progress(&format!("complément {}/{}", db, f.file_name().unwrap_or_default().to_string_lossy()));
                    self.import_file(db, &f)?;
                }
            }
        }
        c.query_drop("FLUSH PRIVILEGES").map_err(|e| e.to_string())?;
        drop(c);
        self.post_import_cleanup(progress)?;
        Ok(())
    }

    /// Objets du dump inutilisables ou bloquants en local (voir Documentation/03-Phase2-Serveur-local.md §4 ter) :
    /// 1. triggers/routines/vues/événements dont le DEFINER n'existe pas ici → MySQL refuse toute écriture (erreur 1449) ;
    /// 2. `active_arena_season` vide (dump « schéma seul ») → assertion au démarrage du worldserver.
    pub fn post_import_cleanup(&self, progress: &dyn Fn(&str)) -> Result<(), String> {
        let mut c = self.root_conn()?;
        let err = |e: mysql::Error| e.to_string();
        let users: Vec<String> = c.query("SELECT CONCAT(user,'@',host) FROM mysql.user").map_err(err)?;
        let objs: Vec<(String, String, String, String)> = c
            .query(
                "SELECT 'TRIGGER', trigger_schema, trigger_name, definer FROM information_schema.triggers WHERE trigger_schema LIKE 'acore_%' \
                 UNION ALL SELECT routine_type, routine_schema, routine_name, definer FROM information_schema.routines WHERE routine_schema LIKE 'acore_%' \
                 UNION ALL SELECT 'VIEW', table_schema, table_name, definer FROM information_schema.views WHERE table_schema LIKE 'acore_%' \
                 UNION ALL SELECT 'EVENT', event_schema, event_name, definer FROM information_schema.events WHERE event_schema LIKE 'acore_%'",
            )
            .map_err(err)?;
        for (kind, sch, name, definer) in objs {
            if users.iter().any(|u| *u == definer) {
                continue;
            }
            progress(&format!("suppression {kind} {sch}.{name} (définisseur {definer} inexistant)"));
            c.query_drop(format!("DROP {kind} IF EXISTS `{sch}`.`{name}`")).map_err(err)?;
        }
        let has: Option<u64> = c.query_first(format!("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='{DB_CHARACTERS}' AND table_name='active_arena_season'")).map_err(err)?;
        if has == Some(1) {
            let n: Option<u64> = c.query_first(format!("SELECT COUNT(*) FROM {DB_CHARACTERS}.active_arena_season")).map_err(err)?;
            if n == Some(0) {
                progress("saison d'arène 8 active (valeur des royaumes Sylvania)");
                c.query_drop(format!("INSERT INTO {DB_CHARACTERS}.active_arena_season (season_id, season_state) VALUES (8, 1)")).map_err(err)?;
            }
        }
        Ok(())
    }

    /// Requête de lecture (explorateur intégré). Les écritures passent par `exec`.
    pub fn query(&self, db: Option<&str>, sql: &str, max_rows: usize) -> Result<QueryResult, String> {
        if !is_read_statement(sql) {
            return Err(crate::i18n::fr::DB_READONLY_ONLY.to_string());
        }
        let mut c = self.root_conn_db(db)?;
        let mut result = c.query_iter(sql).map_err(|e| e.to_string())?;
        let mut out = QueryResult::default();
        if let Some(set) = result.iter() {
            out.columns = set.columns().as_ref().iter().map(|c| c.name_str().into_owned()).collect();
            for row in set {
                let row: Row = row.map_err(|e| e.to_string())?;
                if out.rows.len() >= max_rows {
                    out.truncated = true;
                    break;
                }
                out.rows.push((0..row.len()).map(|i| row.as_ref(i).map(value_to_string).unwrap_or_default()).collect());
            }
        }
        Ok(out)
    }

    /// Exécute une instruction d'écriture (UPDATE/INSERT/DELETE/…) et renvoie le nombre de lignes affectées.
    pub fn exec(&self, db: Option<&str>, sql: &str) -> Result<u64, String> {
        let mut c = self.root_conn_db(db)?;
        c.query_drop(sql).map_err(|e| e.to_string())?;
        Ok(c.affected_rows())
    }

    fn root_conn_db(&self, db: Option<&str>) -> Result<Conn, String> {
        Conn::new(self.opts(true, db)).map_err(|e| format!("connexion SQL : {e}"))
    }

    /// Colonnes d'une table : (nom, type, nullable, clé, défaut).
    pub fn table_columns(&self, db: &str, table: &str) -> Result<Vec<ColumnInfo>, String> {
        let mut c = self.root_conn()?;
        let rows: Vec<(String, String, String, String, Option<String>, String)> = c
            .query(format!(
                "SELECT column_name, column_type, is_nullable, column_key, column_default, extra FROM information_schema.columns \
                 WHERE table_schema='{db}' AND table_name='{table}' ORDER BY ordinal_position"
            ))
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(|(name, col_type, nullable, key, default, extra)| ColumnInfo { name, col_type, nullable: nullable == "YES", primary: key == "PRI", default, extra }).collect())
    }

    /// Lignes d'une table avec filtre et pagination ; renvoie aussi le total filtré.
    pub fn table_rows(&self, db: &str, table: &str, filter: &str, order: Option<(&str, bool)>, offset: u64, limit: u64) -> Result<(QueryResult, u64), String> {
        let where_clause = if filter.trim().is_empty() { String::new() } else { format!(" WHERE {}", filter.trim()) };
        let order_clause = match order {
            Some((col, asc)) => format!(" ORDER BY `{col}` {}", if asc { "ASC" } else { "DESC" }),
            None => String::new(),
        };
        let mut c = self.root_conn_db(Some(db))?;
        let total: Option<u64> = c.query_first(format!("SELECT COUNT(*) FROM `{table}`{where_clause}")).map_err(|e| e.to_string())?;
        let res = self.query(Some(db), &format!("SELECT * FROM `{table}`{where_clause}{order_clause} LIMIT {limit} OFFSET {offset}"), limit as usize)?;
        Ok((res, total.unwrap_or(0)))
    }

    pub fn tables(&self, db: &str) -> Result<Vec<String>, String> {
        let mut c = self.root_conn()?;
        c.query(format!("SELECT table_name FROM information_schema.tables WHERE table_schema='{db}' ORDER BY table_name"))
            .map_err(|e| e.to_string())
    }

    /// Suppression définitive d'un compte et de ses personnages, serveur arrêté : toutes les tables de
    /// `acore_characters` possédant une colonne `guid`, `item_instance` (owner_guid), puis les tables de compte
    /// de `acore_auth`. Renvoie le nombre de personnages supprimés.
    pub fn delete_account_offline(&self, username: &str) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        let err = |e: mysql::Error| e.to_string();
        let upper = username.to_ascii_uppercase().replace('\'', "");
        let Some(id) = c.query_first::<u32, _>(format!("SELECT id FROM {DB_AUTH}.account WHERE UPPER(username)='{upper}'")).map_err(err)? else {
            return Err(format!("compte {username} introuvable"));
        };
        let guids: Vec<u32> = c.query(format!("SELECT guid FROM {DB_CHARACTERS}.characters WHERE account={id}")).map_err(err)?;
        if !guids.is_empty() {
            let list = guids.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(",");
            let tables: Vec<String> = c
                .query(format!("SELECT table_name FROM information_schema.columns WHERE table_schema='{DB_CHARACTERS}' AND column_name='guid' AND table_name<>'characters'"))
                .map_err(err)?;
            for t in tables {
                let _ = c.query_drop(format!("DELETE FROM {DB_CHARACTERS}.`{t}` WHERE guid IN ({list})"));
            }
            let _ = c.query_drop(format!("DELETE FROM {DB_CHARACTERS}.item_instance WHERE owner_guid IN ({list})"));
            c.query_drop(format!("DELETE FROM {DB_CHARACTERS}.characters WHERE guid IN ({list})")).map_err(err)?;
        }
        for sql in [
            format!("DELETE FROM {DB_AUTH}.account_access WHERE id={id}"),
            format!("DELETE FROM {DB_AUTH}.account_banned WHERE id={id}"),
            format!("DELETE FROM {DB_AUTH}.account_muted WHERE guid={id}"),
            format!("DELETE FROM {DB_AUTH}.realmcharacters WHERE acctid={id}"),
            format!("DELETE FROM {DB_AUTH}.account WHERE id={id}"),
        ] {
            let _ = c.query_drop(sql); // tables optionnelles selon le schéma
        }
        Ok(guids.len() as u64)
    }

    /// Comptes de bots aléatoires créés par mod-playerbots (`rndbot*`) et leurs personnages.
    pub fn bot_pool(&self) -> Option<(u64, u64)> {
        let mut c = self.root_conn().ok()?;
        let accounts: u64 = c.query_first(format!("SELECT COUNT(*) FROM {DB_AUTH}.account WHERE username LIKE 'RNDBOT%'")).ok()??;
        let chars: u64 = c.query_first(format!("SELECT COUNT(*) FROM {DB_CHARACTERS}.characters ch JOIN {DB_AUTH}.account a ON a.id=ch.account WHERE a.username LIKE 'RNDBOT%'")).ok()??;
        Some((accounts, chars))
    }

    /// Supprime tous les comptes de bots aléatoires et leurs personnages (le serveur les recrée au démarrage suivant).
    pub fn delete_bot_pool(&self) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        let names: Vec<String> = c.query(format!("SELECT username FROM {DB_AUTH}.account WHERE username LIKE 'RNDBOT%'")).map_err(|e| e.to_string())?;
        drop(c);
        let mut total = 0;
        for n in names {
            total += self.delete_account_offline(&n)?;
        }
        // Tables de suivi du module : liens compte -> bot et clés.
        if let Ok(mut c) = self.root_conn() {
            for t in ["playerbots_random_bots", "playerbots_account_keys", "playerbots_account_links", "playerbots_account_type"] {
                let _ = c.query_drop(format!("DELETE FROM {DB_PLAYERBOTS}.`{t}`"));
            }
        }
        Ok(total)
    }
}
