//! realmd et mangosd : génération des configurations, royaume, démarrage, console, arrêt propre.

use crate::db::{port_open, MariaDb, CREATE_NO_WINDOW};
use crate::paths::{Layout, MARIADB_PORT, REALMD_PORT, REALM_NAME, WORLD_PORT};
use crate::settings::{AhFaction, Settings};
use mysql::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn set_conf_value(lines: &mut [String], key: &str, value: &str) -> Result<(), String> {
    for l in lines.iter_mut() {
        let trimmed = l.trim_start();
        if trimmed.starts_with(key) && trimmed[key.len()..].trim_start().starts_with('=') {
            *l = format!("{key} = {value}");
            return Ok(());
        }
    }
    Err(format!("clé absente du modèle VMaNGOS : {key}"))
}

fn db_info(creds: &crate::credentials::DbCredentials, db: &str) -> String {
    format!("\"127.0.0.1;{MARIADB_PORT};{};{};{db}\"", creds.user, creds.password)
}

/// Port de Tools/New-ServerConfig.ps1 : les .conf sont régénérés à chaque démarrage.
pub fn generate_configs(layout: &Layout, db: &MariaDb, settings: &Settings) -> Result<(), String> {
    let cfg = layout.server_config();
    let log_dir = layout.server_logs();
    std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(layout.server_data()).map_err(|e| e.to_string())?;
    let creds = db.creds();
    let q = |p: std::path::PathBuf| format!("\"{}\"", Layout::conf_path(&p));

    // realmd.conf
    let tpl = std::fs::read_to_string(cfg.join("realmd.conf.dist.in")).map_err(|e| format!("realmd.conf.dist.in : {e}"))?;
    let mut r: Vec<String> = tpl.lines().map(str::to_string).collect();
    set_conf_value(&mut r, "LoginDatabaseInfo", &db_info(creds, "realmd"))?;
    set_conf_value(&mut r, "LogsDir", &q(log_dir.clone()))?;
    set_conf_value(&mut r, "BindIP", "\"127.0.0.1\"")?;
    set_conf_value(&mut r, "RealmServerPort", &REALMD_PORT.to_string())?;
    set_conf_value(&mut r, "PidFile", &q(log_dir.join("realmd.pid")))?;
    set_conf_value(&mut r, "WaitAtStartupError", "0")?;
    // Serveur local solo : pas de contrôle d'intégrité du client (le build 5880 n'a pas d'empreinte de référence).
    set_conf_value(&mut r, "StrictVersionCheck", "0")?;
    std::fs::write(cfg.join("realmd.conf"), r.join("\n") + "\n").map_err(|e| e.to_string())?;

    // mangosd.conf
    let tpl = std::fs::read_to_string(cfg.join("mangosd.conf.dist.in")).map_err(|e| format!("mangosd.conf.dist.in : {e}"))?;
    let mut m: Vec<String> = tpl.lines().map(str::to_string).collect();
    set_conf_value(&mut m, "DataDir", &q(layout.server_data()))?;
    set_conf_value(&mut m, "LogsDir", &q(log_dir.clone()))?;
    set_conf_value(&mut m, "LoginDatabase.Info", &db_info(creds, "realmd"))?;
    set_conf_value(&mut m, "WorldDatabase.Info", &db_info(creds, "mangos"))?;
    set_conf_value(&mut m, "CharacterDatabase.Info", &db_info(creds, "characters"))?;
    set_conf_value(&mut m, "LogsDatabase.Info", &db_info(creds, "logs"))?;
    set_conf_value(&mut m, "BindIP", "\"127.0.0.1\"")?;
    set_conf_value(&mut m, "WorldServerPort", &WORLD_PORT.to_string())?;
    set_conf_value(&mut m, "PidFile", &q(log_dir.join("mangosd.pid")))?;
    set_conf_value(&mut m, "WaitAtStartupError", "0")?;
    set_conf_value(&mut m, "Motd", "\"Bienvenue dans HomeCraft Edition Vanilla !\"")?;
    set_conf_value(&mut m, "DBC.Locale", "255")?;
    set_conf_value(&mut m, "Warden.WinEnabled", "0")?;
    set_conf_value(&mut m, "Anticheat.Enable", "0")?;
    set_conf_value(&mut m, "Console.Enable", "1")?;
    // Réglages des onglets PlayerBots et Monde : valeurs du catalogue (défaut = modèle, surcharges de l'utilisateur).
    // Seules les clés du catalogue sont surchargeables ; les clés gérées ci-dessus ne le sont jamais.
    for opt in crate::conf_catalog::BOT_OPTIONS.iter().chain(crate::conf_catalog::WORLD_OPTIONS.iter()) {
        let v = crate::conf_catalog::value(&settings.conf, opt);
        set_conf_value(&mut m, opt.key, &crate::conf_catalog::conf_value(opt, v))?;
    }
    // Compagnons désactivés dans l'onglet PlayerBots : invocation bloquée (MaxBots = 1 est la limite minimale
    // acceptée par VMaNGOS, 0 signifiant illimité ; SkipChecks = 0 rétablit toutes les restrictions).
    if !settings.partybots_enabled {
        set_conf_value(&mut m, "PartyBot.SkipChecks", "0")?;
        set_conf_value(&mut m, "PartyBot.MaxBots", "1")?;
        set_conf_value(&mut m, "BattleBot.AutoJoin", "0")?;
    }

    let fid = match settings.ah_faction {
        AhFaction::Alliance => 55,
        AhFaction::Horde => 29,
        AhFaction::Auto => db.last_played_faction().unwrap_or(55),
    };
    let rows = db.ahbot_rows();
    let count = if rows > 0 { rows * 3 } else { 500 };
    set_conf_value(&mut m, "AHBot.Enable", if settings.ahbot_enabled && rows > 0 { "1" } else { "0" })?;
    set_conf_value(&mut m, "AHBot.ah.fid", &fid.to_string())?;
    set_conf_value(&mut m, "AHBot.itemcount", &count.to_string())?;
    std::fs::write(cfg.join("mangosd.conf"), m.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(())
}

/// Builds 1.12.1 acceptés : 5875 (version de référence) et 5880 (variante livrée par certains clients, dont
/// le client frFR fourni : l'écran de connexion affiche « 1.12.1 (5880) » bien que la ressource FileVersion dise 5875).
pub const CLIENT_BUILDS: [u16; 2] = [5875, 5880];

/// Royaume local + autorisation des builds : `realmlist.realmbuilds` (sinon le royaume apparaît hors ligne au client)
/// et `allowed_clients` (sinon realmd répond « version invalide »).
pub fn ensure_realm(db: &MariaDb) -> Result<(), String> {
    let mut c = db.root_conn()?;
    let builds = CLIENT_BUILDS.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(" ");
    c.query_drop(format!(
        "INSERT INTO realmd.realmlist (id,name,address,localAddress,localSubnetMask,port,icon,realmflags,timezone,allowedSecurityLevel,population,gamebuild_min,gamebuild_max,flag,realmbuilds) \
         VALUES (1,'{REALM_NAME}','127.0.0.1','127.0.0.1','255.255.255.0',{WORLD_PORT},1,0,0,0,0,{min},{max},0,'{builds}') \
         ON DUPLICATE KEY UPDATE name='{REALM_NAME}', address='127.0.0.1', port={WORLD_PORT}, gamebuild_min={min}, gamebuild_max={max}, realmbuilds='{builds}'",
        min = CLIENT_BUILDS[0],
        max = CLIENT_BUILDS[CLIENT_BUILDS.len() - 1],
    ))
    .map_err(|e| e.to_string())?;
    for b in CLIENT_BUILDS {
        let n: Option<u64> = c.query_first(format!("SELECT COUNT(*) FROM realmd.allowed_clients WHERE build={b} AND os='Win' AND platform='x86'")).map_err(|e| e.to_string())?;
        if n == Some(0) {
            c.query_drop(format!("INSERT INTO realmd.allowed_clients (major_version,minor_version,bugfix_version,hotfix_version,build,os,platform,integrity_hash) VALUES (1,12,1,'',{b},'Win','x86','')")).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub struct Realmd {
    child: Child,
}

impl Realmd {
    pub fn start(layout: &Layout, timeout: Duration) -> Result<Realmd, String> {
        if port_open(REALMD_PORT) {
            return Err(format!("le port {REALMD_PORT} est déjà utilisé par un autre programme"));
        }
        let bin = layout.server_bin();
        let logs = layout.server_logs();
        let out = std::fs::File::create(logs.join("realmd.stdout.log")).map_err(|e| e.to_string())?;
        let err = std::fs::File::create(logs.join("realmd.stderr.log")).map_err(|e| e.to_string())?;
        let mut child = Command::new(bin.join("realmd.exe"))
            .arg("-c")
            .arg(layout.server_config().join("realmd.conf"))
            .current_dir(&bin)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .spawn()
            .map_err(|e| crate::util::spawn_error("realmd", &e))?;
        let start = Instant::now();
        while start.elapsed() < timeout {
            if port_open(REALMD_PORT) {
                return Ok(Realmd { child });
            }
            if let Ok(Some(st)) = child.try_wait() {
                return Err(format!("realmd s'est arrêté (code {:?}), voir Logs/server/realmd.stdout.log", st.code()));
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        let _ = child.kill();
        Err("realmd ne répond pas dans le délai imparti".into())
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// realmd n'a pas de console : arrêt par terminaison (il est sans état).
    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Mangosd {
    child: Child,
    stdin: Option<ChildStdin>,
    pub world_ready: Arc<AtomicBool>,
}

impl Mangosd {
    /// Démarre mangosd avec sa console sur l'entrée standard ; chaque ligne de sortie est envoyée dans `sink`.
    pub fn start(layout: &Layout, sink: Sender<String>) -> Result<Mangosd, String> {
        if port_open(WORLD_PORT) {
            return Err(format!("le port {WORLD_PORT} est déjà utilisé par un autre programme"));
        }
        let bin = layout.server_bin();
        let mut child = Command::new(bin.join("mangosd.exe"))
            .arg("-c")
            .arg(layout.server_config().join("mangosd.conf"))
            .current_dir(&bin)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::util::spawn_error("mangosd", &e))?;
        let world_ready = Arc::new(AtomicBool::new(false));
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().ok_or("sortie standard de mangosd indisponible")?;
        let stderr = child.stderr.take().ok_or("sortie d'erreur de mangosd indisponible")?;
        let console_log = layout.server_logs().join("mangosd.console.log");
        let file = std::fs::File::create(&console_log).ok();
        for (reader, is_err) in [(Box::new(stdout) as Box<dyn std::io::Read + Send>, false), (Box::new(stderr), true)] {
            let sink = sink.clone();
            let ready = world_ready.clone();
            let mut file = file.as_ref().and_then(|f| f.try_clone().ok());
            std::thread::spawn(move || {
                let mut reader = BufReader::new(reader);
                let mut buf = Vec::new();
                loop {
                    buf.clear();
                    match reader.read_until(b'\n', &mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            let line = String::from_utf8_lossy(&buf).trim_end().to_string();
                            if line.is_empty() {
                                continue;
                            }
                            if line.contains("World initialized") {
                                ready.store(true, Ordering::SeqCst);
                            }
                            if let Some(f) = file.as_mut() {
                                let _ = writeln!(f, "{line}");
                            }
                            let _ = sink.send(if is_err { format!("[mangosd!] {line}") } else { format!("[mangosd] {line}") });
                        }
                    }
                }
            });
        }
        Ok(Mangosd { child, stdin, world_ready })
    }

    /// Prêt = port du monde ouvert ET « World initialized » lu sur la console.
    pub fn wait_ready(&mut self, timeout: Duration) -> Result<(), String> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self.world_ready.load(Ordering::SeqCst) && port_open(WORLD_PORT) {
                return Ok(());
            }
            if let Ok(Some(st)) = self.child.try_wait() {
                return Err(format!("mangosd s'est arrêté pendant le chargement (code {:?}), voir Logs/server/mangosd.console.log", st.code()));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        Err("le serveur de jeu ne répond pas dans le délai imparti".into())
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn send(&mut self, command: &str) -> Result<(), String> {
        match self.stdin.as_mut() {
            Some(s) => writeln!(s, "{command}").and_then(|_| s.flush()).map_err(|e| format!("console : {e}")),
            None => Err("console fermée".into()),
        }
    }

    /// « server exit » puis fermeture de l'entrée standard (indispensable : le thread console de mangosd
    /// ne se termine qu'à la fin de stdin). Terminaison forcée seulement après le délai.
    pub fn stop(&mut self, timeout: Duration) -> bool {
        let _ = self.send("server exit");
        drop(self.stdin.take());
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Ok(Some(_)) = self.child.try_wait() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        false
    }
}
