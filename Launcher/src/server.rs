//! realmd et mangosd (CMaNGOS-tbc) : génération des configurations, royaume, démarrage, console, arrêt propre.
//!
//! Pièges CMaNGOS sous Windows (voir Documentation/02-Construction.md) :
//! - le dossier courant DOIT être Server/mangos-tbc : le core y cherche legacy.dll (fournisseur OpenSSL 3),
//!   sinon realmd et mangosd s'arrêtent aussitôt (« Failed to load Legacy provider ») ;
//! - anticheat.conf est lu dans le dossier courant : il n'est pas écrit (dossier du programme vérifié par manifeste),
//!   le module est désactivé par variables d'environnement (Anticheat_Enable=0, …), ses défauts valant déjà 0.

use crate::db::{port_open, MariaDb, CREATE_NO_WINDOW};
use crate::paths::{Layout, CLIENT_BUILD, DB_CHARACTERS, DB_LOGS, DB_REALMD, DB_WORLD, MARIADB_PORT, REALMD_PORT, REALM_NAME, WORLD_PORT};
use crate::settings::Settings;
use mysql::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Modèles copiés depuis le build (Server/mangos-tbc/*.conf.dist) vers Server/Config, puis transformés en .conf.
pub const CONF_FILES: [&str; 4] = ["realmd", "mangosd", "aiplayerbot", "ahbot"];

/// Variables d'environnement passées à mangosd : module anticheat (et son Warden) désactivé, serveur local.
pub const ANTICHEAT_ENV: [(&str, &str); 3] = [("Anticheat_Enable", "0"), ("Anticheat_Warden_Enable", "0"), ("Anticheat_Antispam_Enable", "0")];

fn set_conf_value(lines: &mut [String], key: &str, value: &str) -> Result<(), String> {
    for l in lines.iter_mut() {
        let trimmed = l.trim_start();
        if trimmed.starts_with(key) && trimmed[key.len()..].trim_start().starts_with('=') {
            *l = format!("{key} = {value}");
            return Ok(());
        }
    }
    Err(format!("clé absente du modèle CMaNGOS : {key}"))
}

fn db_info(creds: &crate::credentials::DbCredentials, db: &str) -> String {
    format!("\"127.0.0.1;{MARIADB_PORT};{};{};{db}\"", creds.user, creds.password)
}

/// Copie les modèles du build dans Server/Config s'ils manquent ou ont changé (mise à jour du core).
fn refresh_templates(layout: &Layout) -> Result<(), String> {
    let cfg = layout.server_config();
    std::fs::create_dir_all(&cfg).map_err(|e| e.to_string())?;
    for n in CONF_FILES {
        let src = layout.server_bin().join(format!("{n}.conf.dist"));
        let dst = cfg.join(format!("{n}.conf.dist"));
        let new = std::fs::read(&src).map_err(|e| format!("{} : {e}", src.display()))?;
        if std::fs::read(&dst).ok().as_deref() != Some(new.as_slice()) {
            std::fs::write(&dst, &new).map_err(|e| format!("{} : {e}", dst.display()))?;
        }
    }
    Ok(())
}

fn read_template(layout: &Layout, n: &str) -> Result<Vec<String>, String> {
    let p = layout.server_config().join(format!("{n}.conf.dist"));
    let tpl = std::fs::read_to_string(&p).map_err(|e| format!("{} : {e}", p.display()))?;
    Ok(tpl.lines().map(str::to_string).collect())
}

fn write_conf(layout: &Layout, n: &str, lines: &[String]) -> Result<(), String> {
    std::fs::write(layout.server_config().join(format!("{n}.conf")), lines.join("\n") + "\n").map_err(|e| e.to_string())
}

/// Port de Tools/New-ServerConfig.ps1 : les .conf sont régénérés à chaque démarrage.
pub fn generate_configs(layout: &Layout, db: &MariaDb, settings: &Settings) -> Result<(), String> {
    refresh_templates(layout)?;
    let log_dir = layout.server_logs();
    std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(layout.server_data()).map_err(|e| e.to_string())?;
    let creds = db.creds();
    let q = |p: std::path::PathBuf| format!("\"{}\"", Layout::conf_path(&p));

    // realmd.conf
    let mut r = read_template(layout, "realmd")?;
    set_conf_value(&mut r, "LoginDatabaseInfo", &db_info(creds, DB_REALMD))?;
    set_conf_value(&mut r, "LogsDir", &q(log_dir.clone()))?;
    set_conf_value(&mut r, "BindIP", "\"127.0.0.1\"")?;
    set_conf_value(&mut r, "RealmServerPort", &REALMD_PORT.to_string())?;
    set_conf_value(&mut r, "PidFile", &q(log_dir.join("realmd.pid")))?;
    set_conf_value(&mut r, "WaitAtStartupError", "0")?;
    set_conf_value(&mut r, "StrictVersionCheck", "0")?;
    write_conf(layout, "realmd", &r)?;

    // mangosd.conf (clés de connexion sans point en CMaNGOS : LoginDatabaseInfo…)
    let mut m = read_template(layout, "mangosd")?;
    set_conf_value(&mut m, "DataDir", &q(layout.server_data()))?;
    set_conf_value(&mut m, "LogsDir", &q(log_dir.clone()))?;
    set_conf_value(&mut m, "LoginDatabaseInfo", &db_info(creds, DB_REALMD))?;
    set_conf_value(&mut m, "WorldDatabaseInfo", &db_info(creds, DB_WORLD))?;
    set_conf_value(&mut m, "CharacterDatabaseInfo", &db_info(creds, DB_CHARACTERS))?;
    set_conf_value(&mut m, "LogsDatabaseInfo", &db_info(creds, DB_LOGS))?;
    set_conf_value(&mut m, "BindIP", "\"127.0.0.1\"")?;
    set_conf_value(&mut m, "WorldServerPort", &WORLD_PORT.to_string())?;
    set_conf_value(&mut m, "PidFile", &q(log_dir.join("mangosd.pid")))?;
    set_conf_value(&mut m, "WaitAtStartupError", "0")?;
    set_conf_value(&mut m, "DBC.Locale", "255")?; // auto-détection : frFR (DBC extraits du client français)
    set_conf_value(&mut m, "Expansion", "1")?; // The Burning Crusade
    set_conf_value(&mut m, "Console.Enable", "1")?; // console sur l'entrée standard : commandes du launcher
    set_conf_value(&mut m, "Ra.Enable", "0")?;
    set_conf_value(&mut m, "SOAP.Enabled", "0")?;
    // Réglages de l'onglet Monde : valeurs du catalogue (défaut = modèle, surcharges de l'utilisateur).
    for opt in crate::conf_catalog::WORLD_OPTIONS.iter() {
        let v = crate::conf_catalog::value(&settings.conf, opt);
        set_conf_value(&mut m, opt.key, &crate::conf_catalog::conf_value(opt, v))?;
    }
    write_conf(layout, "mangosd", &m)?;

    // aiplayerbot.conf (module cmangos/playerbots compilé dans le build « with-all »)
    let mut b = read_template(layout, "aiplayerbot")?;
    for opt in crate::conf_catalog::BOT_OPTIONS.iter() {
        let v = crate::conf_catalog::value(&settings.conf, opt);
        set_conf_value(&mut b, opt.key, &crate::conf_catalog::conf_value(opt, v))?;
    }
    if !settings.partybots_enabled {
        set_conf_value(&mut b, "AiPlayerbot.Enabled", "0")?;
    }
    // Comptes de bots aléatoires : 10 personnages au plus par compte ; un compte pour 9 bots (marge).
    let max_bots: u32 = crate::conf_catalog::bot_value(&settings.conf, "AiPlayerbot.MaxRandomBots").parse().unwrap_or(50);
    set_conf_value(&mut b, "AiPlayerbot.RandomBotAccountCount", &max_bots.div_ceil(9).max(1).to_string())?;
    write_conf(layout, "aiplayerbot", &b)?;
    // Le module ne lit PAS l'option -p de mangosd (Main.cpp l'affecte à _PLAYERBOT_CONFIG, mais PlayerbotAIConfig lit
    // la constante _D_AIPLAYERBOT_CONFIG = "aiplayerbot.conf", relative au dossier courant) : sans cette copie dans
    // Server/mangos-tbc, « AI Playerbot is Disabled. Unable to open configuration file » (constaté le 2026-10-17).
    // Fichier généré, exclu du manifeste des fichiers gérés.
    std::fs::write(layout.server_bin().join("aiplayerbot.conf"), b.join("\n") + "\n").map_err(|e| format!("aiplayerbot.conf : {e}"))?;

    // ahbot.conf : AHBot intégré (tous les hôtels des ventes). Désactivé = probabilités de vente/achat à 0.
    let mut a = read_template(layout, "ahbot")?;
    if !settings.ahbot_enabled {
        set_conf_value(&mut a, "AuctionHouseBot.Chance.Sell", "0")?;
        set_conf_value(&mut a, "AuctionHouseBot.Chance.Buy", "0")?;
    }
    write_conf(layout, "ahbot", &a)?;
    Ok(())
}

/// Royaume local : realmlist id 1 « HomeCraft BC », 127.0.0.1:port du monde, type Normal, build 8606 seul accepté
/// (`realmbuilds`). CMaNGOS-tbc n'a pas de table allowed_clients.
pub fn ensure_realm(db: &MariaDb) -> Result<(), String> {
    let mut c = db.root_conn()?;
    let build = CLIENT_BUILD.3;
    c.query_drop(format!(
        "INSERT INTO {DB_REALMD}.realmlist (id,name,address,port,icon,realmflags,timezone,allowedSecurityLevel,population,realmbuilds) \
         VALUES (1,'{REALM_NAME}','127.0.0.1',{WORLD_PORT},0,0,1,0,0,'{build}') \
         ON DUPLICATE KEY UPDATE name='{REALM_NAME}', address='127.0.0.1', port={WORLD_PORT}, icon=0, realmbuilds='{build}'"
    ))
    .map_err(|e| e.to_string())?;
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
            .map_err(|e| format!("realmd : {e}"))?;
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
        let cfg = layout.server_config();
        let mut child = Command::new(bin.join("mangosd.exe"))
            .arg("-c")
            .arg(cfg.join("mangosd.conf"))
            .arg("-a")
            .arg(cfg.join("ahbot.conf"))
            .arg("-p")
            .arg(cfg.join("aiplayerbot.conf"))
            .envs(ANTICHEAT_ENV)
            .current_dir(&bin)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("mangosd : {e}"))?;
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

    /// Prêt = port du monde ouvert ET « CMANGOS: World initialized » lu sur la console.
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

    /// « server shutdown 0 » (« server exit » exige le niveau console) puis fermeture de l'entrée standard,
    /// indispensable : le thread console de mangosd ne se termine qu'à la fin de stdin. Terminaison forcée après le délai.
    pub fn stop(&mut self, timeout: Duration) -> bool {
        let _ = self.send("server shutdown 0");
        std::thread::sleep(Duration::from_millis(1500));
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
