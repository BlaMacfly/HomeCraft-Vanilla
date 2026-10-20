//! MariaDB portable : configuration, initialisation du datadir, démarrage, santé, arrêt propre, requêtes.

use crate::credentials::DbCredentials;
use crate::paths::{Layout, MARIADB_PORT};
use mysql::prelude::*;
use mysql::{Conn, OptsBuilder, Row, Value};
use std::net::{SocketAddr, TcpStream};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone)]
pub struct MariaDb {
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

/// Tables de `characters` reliées à un personnage par une autre colonne que `guid`.
const CHARACTER_LINKS: &[(&str, &str)] = &[
    ("character_pet", "owner_guid"),
    ("item_instance", "owner_guid"),
    ("item_loot", "owner_guid"),
    ("corpse", "player_guid"),
    ("character_social", "friend"),
    ("character_deleted_items", "player_guid"),
    ("auction", "seller_guid"),
    ("group_member", "member_guid"),
    ("petition_sign", "player_guid"),
    ("petition", "owner_guid"),
    ("guild_member", "guid"),
    ("gm_tickets", "guid"),
    ("gm_surveys", "guid"),
    ("playerbot", "char_guid"),
];

/// Supprime des personnages et ce qui leur appartient dans la base `characters`. Chaque table est désignée par la
/// colonne qui porte vraiment le numéro du personnage : dans `item_instance`, `corpse`, `creature_respawn` ou `pet_*`,
/// la colonne `guid` est le numéro d'un objet, d'un cadavre, d'une créature ou d'un familier (les effacer par `guid`
/// détruirait les données d'autres personnages). Les tables `character_*` sont, elles, indexées par le personnage.
pub fn delete_characters(c: &mut Conn, guids: &[u32]) -> Result<(), String> {
    if guids.is_empty() {
        return Ok(());
    }
    let err = |e: mysql::Error| e.to_string();
    let list = guids.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(",");
    let columns: Vec<(String, String)> = c.query("SELECT table_name, column_name FROM information_schema.columns WHERE table_schema='characters'").map_err(err)?;
    let has = |t: &str, col: &str| columns.iter().any(|(a, b)| a == t && b == col);
    let mut sql: Vec<String> = Vec::new();
    if has("character_pet", "owner_guid") {
        let pets: Vec<u32> = c.query(format!("SELECT id FROM characters.character_pet WHERE owner_guid IN ({list})")).map_err(err)?;
        if !pets.is_empty() {
            let pet_list = pets.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(",");
            for t in ["pet_aura", "pet_spell", "pet_spell_cooldown"] {
                if has(t, "guid") {
                    sql.push(format!("DELETE FROM characters.`{t}` WHERE guid IN ({pet_list})"));
                }
            }
        }
    }
    if has("mail", "receiver_guid") {
        if has("mail_items", "mail_id") {
            sql.push(format!("DELETE FROM characters.mail_items WHERE mail_id IN (SELECT id FROM characters.mail WHERE receiver_guid IN ({list}))"));
        }
        sql.push(format!("DELETE FROM characters.mail WHERE receiver_guid IN ({list})"));
    }
    let mut own_tables: Vec<&str> = columns.iter().filter(|(t, col)| col == "guid" && t.starts_with("character_")).map(|(t, _)| t.as_str()).collect();
    own_tables.sort_unstable();
    own_tables.dedup();
    for t in own_tables {
        sql.push(format!("DELETE FROM characters.`{t}` WHERE guid IN ({list})"));
    }
    for (t, col) in CHARACTER_LINKS {
        if has(t, col) {
            sql.push(format!("DELETE FROM characters.`{t}` WHERE `{col}` IN ({list})"));
        }
    }
    sql.push(format!("DELETE FROM characters.characters WHERE guid IN ({list})"));
    for s in sql {
        c.query_drop(&s).map_err(|e| format!("{e} ({s})"))?;
    }
    Ok(())
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

impl MariaDb {
    pub fn new(layout: Layout, creds: DbCredentials) -> MariaDb {
        MariaDb { layout, creds }
    }

    pub fn creds(&self) -> &DbCredentials {
        &self.creds
    }

    fn ini_path(&self) -> std::path::PathBuf {
        self.layout.server_config().join("my.ini")
    }

    /// my.ini géré : régénéré à chaque démarrage, aucune donnée personnelle dedans.
    pub fn write_my_ini(&self) -> Result<(), String> {
        let l = &self.layout;
        std::fs::create_dir_all(l.db_data()).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(l.logs()).map_err(|e| e.to_string())?;
        let ini = format!(
            "[mysqld]\nbasedir={base}\ndatadir={data}\nport={port}\nbind-address=127.0.0.1\nskip-name-resolve\n\
max_allowed_packet=128M\ncharacter-set-server=utf8mb3\ncollation-server=utf8mb3_general_ci\n\
innodb_buffer_pool_size=256M\ninnodb_flush_log_at_trx_commit=2\nlog_error={log}\npid-file={pid}\n\n[client]\nport={port}\ndefault-character-set=utf8mb3\n",
            base = l.mariadb_dir().display(),
            data = l.db_data().display(),
            port = MARIADB_PORT,
            log = l.logs().join("mariadb-error.log").display(),
            pid = l.db_data().join("mariadb.pid").display(),
        );
        std::fs::write(self.ini_path(), ini).map_err(|e| e.to_string())
    }

    pub fn is_initialized(&self) -> bool {
        self.layout.db_data().join("mysql").is_dir()
    }

    /// Premier lancement : création du datadir avec le mot de passe root généré.
    pub fn initialize(&self) -> Result<(), String> {
        let exe = self.layout.mariadb_bin().join("mariadb-install-db.exe");
        let status = Command::new(&exe)
            .arg(format!("--datadir={}", self.layout.db_data().display()))
            .arg(format!("--password={}", self.creds.root_password))
            .arg(format!("--port={MARIADB_PORT}"))
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("mariadb-install-db : {e}"))?;
        if !status.success() {
            return Err(format!("mariadb-install-db a échoué (code {:?})", status.code()));
        }
        Ok(())
    }

    fn opts(&self, root: bool, db: Option<&str>) -> OptsBuilder {
        let (u, p) = if root { ("root", self.creds.root_password.as_str()) } else { (self.creds.user.as_str(), self.creds.password.as_str()) };
        OptsBuilder::new()
            .ip_or_hostname(Some("127.0.0.1"))
            .tcp_port(MARIADB_PORT)
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
        if !port_open(MARIADB_PORT) {
            return false;
        }
        match self.root_conn() {
            Ok(mut c) => c.query_first::<u64, _>("SELECT 1").map(|v| v == Some(1)).unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Démarre mariadbd (détaché, sans fenêtre) et attend la santé réelle.
    /// Renvoie `true` si le processus a été lancé ici, `false` s'il était déjà en service (il ne nous appartient pas).
    pub fn start(&self, timeout: Duration) -> Result<bool, String> {
        if self.ping() {
            return Ok(false);
        }
        let exe = self.layout.mariadb_bin().join("mariadbd.exe");
        let mut child = Command::new(&exe)
            .arg(format!("--defaults-file={}", self.ini_path().display()))
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| crate::util::spawn_error("mariadbd", &e))?;
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self.ping() {
                return Ok(true);
            }
            if let Ok(Some(st)) = child.try_wait() {
                return Err(format!("mariadbd s'est arrêté (code {:?}), voir Logs/mariadb-error.log", st.code()));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        Err("la base de données ne répond pas dans le délai imparti".into())
    }

    /// Arrêt propre par la commande SQL SHUTDOWN (jamais de kill).
    pub fn shutdown(&self, timeout: Duration) -> Result<(), String> {
        if !port_open(MARIADB_PORT) {
            return Ok(());
        }
        let mut c = self.root_conn()?;
        let _ = c.query_drop("SHUTDOWN");
        drop(c);
        let start = Instant::now();
        while start.elapsed() < timeout {
            if !port_open(MARIADB_PORT) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        Err("la base de données répond toujours après le délai d'arrêt".into())
    }

    pub fn databases_present(&self) -> Result<bool, String> {
        let mut c = self.root_conn()?;
        let n: Option<u64> = c
            .query_first("SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name IN ('realmd','characters','mangos','logs')")
            .map_err(|e| e.to_string())?;
        Ok(n == Some(4))
    }

    pub fn characters_count(&self) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        c.query_first("SELECT COUNT(*) FROM characters.characters").map(|v| v.unwrap_or(0)).map_err(|e| e.to_string())
    }

    pub fn accounts_count(&self) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        c.query_first("SELECT COUNT(*) FROM realmd.account").map(|v| v.unwrap_or(0)).map_err(|e| e.to_string())
    }

    pub const ALL_DBS: [(&'static str, &'static str); 4] = [("realmd", "logon.sql"), ("characters", "characters.sql"), ("mangos", "mangos.sql"), ("logs", "logs.sql")];

    /// Bases absentes parmi les quatre attendues.
    pub fn databases_missing(&self) -> Result<Vec<&'static str>, String> {
        let mut c = self.root_conn()?;
        let present: Vec<String> = c
            .query("SELECT schema_name FROM information_schema.schemata WHERE schema_name IN ('realmd','characters','mangos','logs')")
            .map_err(|e| e.to_string())?;
        Ok(Self::ALL_DBS.iter().map(|(d, _)| *d).filter(|d| !present.iter().any(|p| p == d)).collect())
    }

    /// Bases dont la table `migrations` manque (schéma incomplet).
    pub fn databases_without_migrations(&self) -> Result<Vec<String>, String> {
        let mut c = self.root_conn()?;
        let mut out = Vec::new();
        for (d, _) in Self::ALL_DBS {
            let n: Option<u64> = c.query_first(format!("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='{d}' AND table_name='migrations'")).map_err(|e| e.to_string())?;
            if n != Some(1) {
                out.push(d.to_string());
            }
        }
        Ok(out)
    }

    fn client_args(&self) -> Vec<String> {
        vec![
            "--host=127.0.0.1".into(),
            format!("--port={MARIADB_PORT}"),
            "--protocol=TCP".into(),
            "--user=root".into(),
            format!("--password={}", self.creds.root_password),
            "--default-character-set=utf8mb3".into(),
        ]
    }

    /// Import d'un fichier SQL dans une base existante (client mariadb.exe, entrée standard = fichier).
    pub fn import_file(&self, db: &str, file: &std::path::Path) -> Result<(), String> {
        let f = std::fs::File::open(file).map_err(|e| format!("{} : {e}", file.display()))?;
        let status = Command::new(self.layout.mariadb_bin().join("mariadb.exe"))
            .args(self.client_args())
            .args(["--max_allowed_packet=134217728", db])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::from(f))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("mariadb.exe : {e}"))?;
        if !status.success() {
            return Err(format!("import de {} échoué (code {:?})", file.display(), status.code()));
        }
        Ok(())
    }

    /// Dump cohérent : tables verrouillées pendant l'export (les bases mêlent MyISAM et InnoDB,
    /// --single-transaction ne suffirait pas).
    pub fn dump_database(&self, db: &str, out: &std::path::Path) -> Result<(), String> {
        let status = Command::new(self.layout.mariadb_bin().join("mariadb-dump.exe"))
            .args(self.client_args())
            .args(["--lock-tables", "--add-drop-table", "--routines", "--triggers", "--skip-dump-date"])
            .arg(format!("--result-file={}", out.display()))
            .arg(db)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("mariadb-dump : {e}"))?;
        if !status.success() || !out.is_file() {
            return Err(format!("export de {db} échoué (code {:?})", status.code()));
        }
        Ok(())
    }

    pub fn online_count(&self) -> u64 {
        self.root_conn()
            .ok()
            .and_then(|mut c| c.query_first::<u64, _>("SELECT COUNT(*) FROM characters.characters WHERE online=1").ok().flatten())
            .unwrap_or(0)
    }

    /// Import des bases listées depuis les dumps du package (mêmes étapes que Tools/Init-Database.ps1).
    /// Refuse de recréer `characters` si elle contient des personnages : ils ne sont jamais écrasés ici.
    pub fn import_dumps(&self, dbs: &[&str], progress: &dyn Fn(&str)) -> Result<(), String> {
        if dbs.contains(&"characters") && self.characters_count().unwrap_or(0) > 0 {
            return Err("la base characters contient des personnages : recréation refusée (restaurer une sauvegarde à la place)".into());
        }
        let user = &self.creds.user;
        let pass = &self.creds.password;
        let mut c = self.root_conn()?;
        for host in ["127.0.0.1", "localhost"] {
            c.query_drop(format!("CREATE USER IF NOT EXISTS '{user}'@'{host}' IDENTIFIED BY '{pass}'")).map_err(|e| e.to_string())?;
            c.query_drop(format!("ALTER USER '{user}'@'{host}' IDENTIFIED BY '{pass}'")).map_err(|e| e.to_string())?;
        }
        for (db, file) in Self::ALL_DBS {
            if !dbs.contains(&db) {
                continue;
            }
            progress(&format!("import de {db} ({file})"));
            c.query_drop(format!("DROP DATABASE IF EXISTS `{db}`")).map_err(|e| e.to_string())?;
            c.query_drop(format!("CREATE DATABASE `{db}` DEFAULT CHARSET utf8mb3 COLLATE utf8mb3_general_ci")).map_err(|e| e.to_string())?;
            for host in ["127.0.0.1", "localhost"] {
                c.query_drop(format!("GRANT ALL PRIVILEGES ON `{db}`.* TO '{user}'@'{host}'")).map_err(|e| e.to_string())?;
            }
            self.import_file(db, &self.layout.sql_dumps().join(file))?;
            // Compléments du projet (ex. auctionhousebot_solo.sql) appliqués à la base monde.
            if db == "mangos" {
                let custom = self.layout.root.join("Server").join("Database").join("custom");
                if let Ok(rd) = std::fs::read_dir(&custom) {
                    let mut files: Vec<_> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().map(|x| x == "sql").unwrap_or(false)).collect();
                    files.sort();
                    for f in files {
                        progress(&format!("complément {}", f.file_name().unwrap_or_default().to_string_lossy()));
                        self.import_file("mangos", &f)?;
                    }
                }
            }
        }
        c.query_drop("FLUSH PRIVILEGES").map_err(|e| e.to_string())?;
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

    /// Exécute une instruction d'écriture (UPDATE/INSERT/DELETE/… ) et renvoie le nombre de lignes affectées.
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
        let mut c = self.conn(None)?;
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
        let mut c = self.conn(Some(db))?;
        let total: Option<u64> = c.query_first(format!("SELECT COUNT(*) FROM `{table}`{where_clause}")).map_err(|e| e.to_string())?;
        let res = self.query(Some(db), &format!("SELECT * FROM `{table}`{where_clause}{order_clause} LIMIT {limit} OFFSET {offset}"), limit as usize)?;
        Ok((res, total.unwrap_or(0)))
    }

    pub fn tables(&self, db: &str) -> Result<Vec<String>, String> {
        let mut c = self.conn(None)?;
        c.query(format!("SELECT table_name FROM information_schema.tables WHERE table_schema='{db}' ORDER BY table_name"))
            .map_err(|e| e.to_string())
    }

    /// Faction (55 Alliance / 29 Horde) du dernier personnage joué, pour l'AHBot.
    pub fn last_played_faction(&self) -> Option<u32> {
        let mut c = self.root_conn().ok()?;
        // Personnages du joueur seulement : les bots générés ont une date de déconnexion récente et une faction au hasard.
        let race: Option<u8> = c
            .query_first(format!(
                "SELECT c.race FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE a.username<>'{}' ORDER BY c.logout_time DESC LIMIT 1",
                crate::botgen::BOT_ACCOUNT
            ))
            .ok()?;
        race.map(|r| if matches!(r, 2 | 5 | 6 | 8) { 29 } else { 55 })
    }

    /// Suppression définitive d'un compte et de ses personnages, serveur arrêté : toutes les tables de la base
    /// `characters` possédant une colonne `guid` (données du personnage), `item_instance` (owner_guid), `playerbot`,
    /// puis `realmd.account`, `account_access`, `account_banned`, `realmcharacters`. Renvoie le nombre de personnages supprimés.
    pub fn delete_account_offline(&self, username: &str) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        let err = |e: mysql::Error| e.to_string();
        let upper = username.to_ascii_uppercase().replace('\'', "");
        let Some(id) = c.query_first::<u32, _>(format!("SELECT id FROM realmd.account WHERE UPPER(username)='{upper}'")).map_err(err)? else {
            return Err(format!("compte {username} introuvable"));
        };
        let guids: Vec<u32> = c.query(format!("SELECT guid FROM characters.characters WHERE account={id}")).map_err(err)?;
        delete_characters(&mut c, &guids)?;
        for sql in [
            format!("DELETE FROM realmd.account_access WHERE id={id}"),
            format!("DELETE FROM realmd.account_banned WHERE id={id}"),
            format!("DELETE FROM realmd.realmcharacters WHERE acctid={id}"),
            format!("DELETE FROM realmd.account WHERE id={id}"),
        ] {
            let _ = c.query_drop(sql); // tables optionnelles selon le schéma
        }
        Ok(guids.len() as u64)
    }

    /// Personnages bots enregistrés pour le peuplement (RandomBot).
    pub fn bot_pool_count(&self) -> Option<u64> {
        self.root_conn().ok().and_then(|mut c| c.query_first::<u64, _>("SELECT COUNT(*) FROM characters.playerbot").ok().flatten())
    }

    pub fn ahbot_rows(&self) -> u64 {
        self.root_conn()
            .ok()
            .and_then(|mut c| c.query_first::<u64, _>("SELECT COUNT(*) FROM mangos.auctionhousebot").ok().flatten())
            .unwrap_or(0)
    }
}
