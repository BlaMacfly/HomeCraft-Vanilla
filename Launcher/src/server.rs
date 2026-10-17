//! authserver et worldserver (AzerothCore) : génération des configurations, royaume, démarrage, console, arrêt propre.

use crate::db::{port_open, MySql, CREATE_NO_WINDOW};
use crate::paths::{Layout, AUTH_PORT, CLIENT_BUILD, DB_AUTH, DB_CHARACTERS, DB_PLAYERBOTS, DB_PORT, DB_WORLD, WORLD_PORT};
use crate::settings::Settings;
use mysql::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Remplace « Clé = valeur » (clé exacte, hors commentaires). Erreur si la clé n'existe pas dans le modèle.
fn set_conf_value(lines: &mut [String], key: &str, value: &str) -> Result<(), String> {
    for l in lines.iter_mut() {
        let trimmed = l.trim_start();
        if let Some(rest) = trimmed.strip_prefix(key) {
            if rest.trim_start().starts_with('=') {
                *l = format!("{key} = {value}");
                return Ok(());
            }
        }
    }
    Err(format!("clé absente du modèle : {key}"))
}

fn db_info(creds: &crate::credentials::DbCredentials, db: &str) -> String {
    format!("\"127.0.0.1;{DB_PORT};{};{};{db}\"", creds.user, creds.password)
}

fn write_conf(out: &Path, lines: &[String], extra: &[(String, String)]) -> Result<(), String> {
    let mut text = lines.join("\n");
    if !extra.is_empty() {
        text.push_str("\n\n# --- HomeCraft : clés hors modèle (lues par le cœur patché Playerbot) ---\n");
        for (k, v) in extra {
            text.push_str(&format!("{k} = {v}\n"));
        }
    }
    text.push('\n');
    std::fs::write(out, text).map_err(|e| format!("{} : {e}", out.display()))
}

fn read_dist(cfg: &Path, name: &str) -> Result<Vec<String>, String> {
    let p = cfg.join(name);
    let tpl = std::fs::read_to_string(&p).map_err(|e| format!("{} : {e}", p.display()))?;
    Ok(tpl.lines().map(str::to_string).collect())
}

/// Port de Tools/New-ServerConfig.ps1 : les .conf sont régénérés à chaque démarrage.
/// `authserver.conf`, `worldserver.conf` et `modules/*.conf` (copies des modèles avec surcharges).
pub fn generate_configs(layout: &Layout, db: &MySql, settings: &Settings) -> Result<(), String> {
    let cfg = layout.server_config();
    let log_dir = layout.server_logs();
    std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(layout.server_data()).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(cfg.join("modules")).map_err(|e| e.to_string())?;
    let creds = db.creds();
    let q = |p: PathBuf| format!("\"{}\"", Layout::conf_path(&p));
    // Dépôt source du cœur : amorçage/updates par le serveur seulement s'il est présent (poste de développement).
    let source = layout.core_source_dir();
    let updates_mask = if source.is_some() { 15 } else { 0 };
    let source_conf = source.as_ref().map(|p| q(p.clone())).unwrap_or_else(|| "\"\"".into());
    let mysql_exe = q(layout.mysql_bin().join("mysql.exe"));

    // authserver.conf
    let mut a = read_dist(&cfg, "authserver.conf.dist")?;
    set_conf_value(&mut a, "LogsDir", &q(log_dir.clone()))?;
    set_conf_value(&mut a, "RealmServerPort", &AUTH_PORT.to_string())?;
    set_conf_value(&mut a, "BindIP", "\"127.0.0.1\"")?;
    set_conf_value(&mut a, "PidFile", &q(log_dir.join("authserver.pid")))?;
    set_conf_value(&mut a, "LoginDatabaseInfo", &db_info(creds, DB_AUTH))?;
    set_conf_value(&mut a, "Updates.EnableDatabases", if updates_mask & 1 != 0 { "1" } else { "0" })?;
    set_conf_value(&mut a, "Updates.AutoSetup", "1")?;
    set_conf_value(&mut a, "SourceDirectory", &source_conf)?;
    set_conf_value(&mut a, "MySQLExecutable", &mysql_exe)?;
    write_conf(&cfg.join("authserver.conf"), &a, &[])?;

    // worldserver.conf
    let mut w = read_dist(&cfg, "worldserver.conf.dist")?;
    set_conf_value(&mut w, "RealmID", "1")?;
    set_conf_value(&mut w, "WorldServerPort", &WORLD_PORT.to_string())?;
    set_conf_value(&mut w, "BindIP", "\"127.0.0.1\"")?;
    set_conf_value(&mut w, "LoginDatabaseInfo", &db_info(creds, DB_AUTH))?;
    set_conf_value(&mut w, "WorldDatabaseInfo", &db_info(creds, DB_WORLD))?;
    set_conf_value(&mut w, "CharacterDatabaseInfo", &db_info(creds, DB_CHARACTERS))?;
    set_conf_value(&mut w, "DataDir", &q(layout.server_data()))?;
    set_conf_value(&mut w, "LogsDir", &q(log_dir.clone()))?;
    set_conf_value(&mut w, "PidFile", &q(log_dir.join("worldserver.pid")))?;
    set_conf_value(&mut w, "Console.Enable", "1")?;
    set_conf_value(&mut w, "Ra.Enable", "0")?;
    set_conf_value(&mut w, "SOAP.Enabled", "0")?;
    set_conf_value(&mut w, "Updates.EnableDatabases", &updates_mask.to_string())?;
    set_conf_value(&mut w, "Updates.AutoSetup", "1")?;
    set_conf_value(&mut w, "SourceDirectory", &source_conf)?;
    set_conf_value(&mut w, "MySQLExecutable", &mysql_exe)?;
    // Obligatoire : Warden rejette le client 3.4.3 et HermesProxy ne relaie pas ses paquets.
    set_conf_value(&mut w, "Warden.Enabled", "0")?;
    set_conf_value(&mut w, "FlashAtStart", "0")?;
    for opt in crate::conf_catalog::options_for(crate::conf_catalog::ConfFile::World) {
        let v = crate::conf_catalog::value(&settings.conf, opt);
        set_conf_value(&mut w, opt.key, &crate::conf_catalog::conf_value(opt, v))?;
    }
    let extra = vec![
        ("PlayerbotsDatabaseInfo".to_string(), db_info(creds, DB_PLAYERBOTS)),
        ("Playerbots.Updates.EnableDatabases".to_string(), if updates_mask & 8 != 0 { "1".to_string() } else { "0".to_string() }),
    ];
    write_conf(&cfg.join("worldserver.conf"), &w, &extra)?;

    // Modules : chaque modèle copié en .conf, avec surcharges pour playerbots et l'AHBot.
    let modules = cfg.join("modules");
    let mut dists: Vec<PathBuf> = std::fs::read_dir(&modules)
        .map_err(|e| format!("{} : {e}", modules.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().ends_with(".conf.dist"))
        .collect();
    dists.sort();
    for dist in dists {
        let name = dist.file_name().unwrap().to_string_lossy().replace(".conf.dist", ".conf");
        // Base : le .conf existant (il conserve l'activation des modules et les modifications faites à la main via
        // l'onglet Modules) ; le modèle .conf.dist si le .conf manque ou s'il lui manque une clé gérée.
        let conf_path = modules.join(&name);
        let from_dist: Vec<String> = std::fs::read_to_string(&dist).map_err(|e| e.to_string())?.lines().map(str::to_string).collect();
        let from_conf: Option<Vec<String>> = std::fs::read_to_string(&conf_path).ok().map(|t| t.lines().map(str::to_string).collect());
        let mut lines = match from_conf {
            Some(mut existing) => {
                if apply_module_overrides(&name, &mut existing, creds, settings).is_ok() { existing } else { from_dist }
            }
            None => from_dist,
        };
        apply_module_overrides(&name, &mut lines, creds, settings)?;
        write_conf(&conf_path, &lines, &[])?;
    }
    Ok(())
}

/// Surcharges du launcher sur un `.conf` de module (clés gérées : base, bots, AHBot).
fn apply_module_overrides(name: &str, lines: &mut [String], creds: &crate::credentials::DbCredentials, settings: &Settings) -> Result<(), String> {
    {
        if name == "playerbots.conf" {
            set_conf_value(lines, "PlayerbotsDatabaseInfo", &db_info(creds, DB_PLAYERBOTS))?;
            set_conf_value(lines, "AiPlayerbot.MinRandomBots", &settings.random_bots.to_string())?;
            set_conf_value(lines, "AiPlayerbot.MaxRandomBots", &settings.random_bots.to_string())?;
            set_conf_value(lines, "AiPlayerbot.RandomBotAutologin", if settings.random_bots > 0 { "1" } else { "0" })?;
            // Serveur de commandes TCP du module (port 8888 par défaut, à l'écoute sur TOUTES les interfaces, sans
            // authentification) : inutile en solo et contraire à la règle « tout sur 127.0.0.1 ». 0 = désactivé.
            // Constaté au premier lancement réel (Tests/Test-Launcher.ps1, 2026-10-16).
            set_conf_value(lines, "AiPlayerbot.CommandServerPort", "0")?;
            for opt in crate::conf_catalog::options_for(crate::conf_catalog::ConfFile::Playerbots) {
                let v = crate::conf_catalog::value(&settings.conf, opt);
                set_conf_value(lines, opt.key, &crate::conf_catalog::conf_value(opt, v))?;
            }
        } else if name == "mod_ahbot.conf" {
            let on = settings.ahbot_enabled && settings.ahbot_account > 0 && settings.ahbot_guid > 0;
            set_conf_value(lines, "AuctionHouseBot.EnableSeller", if on { "1" } else { "0" })?;
            set_conf_value(lines, "AuctionHouseBot.Account", &settings.ahbot_account.to_string())?;
            set_conf_value(lines, "AuctionHouseBot.GUID", &settings.ahbot_guid.to_string())?;
        }
    }
    Ok(())
}

/// Active ou désactive un module en écrivant sa clé dans `configs/modules/<conf>` (créé depuis le modèle si absent).
pub fn set_module_enabled(layout: &Layout, module_id: &str, enabled: bool) -> Result<(), String> {
    let def = crate::modules_catalog::find(module_id).ok_or_else(|| format!("module inconnu : {module_id}"))?;
    let conf = def.conf.ok_or_else(|| format!("{} : pas de fichier de configuration", def.label))?;
    let (key, on, off) = def.enable.ok_or_else(|| format!("{} : pas de clé d'activation", def.label))?;
    let modules = layout.server_config().join("modules");
    let path = modules.join(conf);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => std::fs::read_to_string(modules.join(format!("{conf}.dist"))).map_err(|e| format!("{conf}.dist : {e}"))?,
    };
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    set_conf_value(&mut lines, key, if enabled { on } else { off })?;
    write_conf(&path, &lines, &[])
}

/// Rétablit le modèle d'un module : supprime son `.conf`, régénéré depuis le `.conf.dist` au prochain démarrage
/// (ou immédiatement si la base est joignable, par `generate_configs`).
pub fn reset_module_conf(layout: &Layout, module_id: &str) -> Result<(), String> {
    let def = crate::modules_catalog::find(module_id).ok_or_else(|| format!("module inconnu : {module_id}"))?;
    let conf = def.conf.ok_or_else(|| format!("{} : pas de fichier de configuration", def.label))?;
    let path = layout.server_config().join("modules").join(conf);
    if path.is_file() {
        std::fs::remove_file(&path).map_err(|e| format!("{} : {e}", path.display()))?;
    }
    // Copie du modèle pour que le bouton « Ouvrir » et l'état restent cohérents sans attendre un démarrage.
    let dist = layout.server_config().join("modules").join(format!("{conf}.dist"));
    std::fs::copy(&dist, &path).map_err(|e| format!("{} : {e}", dist.display()))?;
    Ok(())
}

/// Royaume local : ligne `realmlist` id 1 (INSERT … ON DUPLICATE KEY UPDATE : authserver refuse de démarrer sans
/// royaume, et le dump du royaume livre la table vide).
pub fn ensure_realm(db: &MySql, realm_name: &str) -> Result<(), String> {
    let mut c = db.root_conn()?;
    let rn = realm_name.replace('\'', "''");
    c.query_drop(format!(
        "INSERT INTO {DB_AUTH}.realmlist (id, name, address, localAddress, localSubnetMask, port, icon, flag, timezone, allowedSecurityLevel, population, gamebuild) \
         VALUES (1, '{rn}', '127.0.0.1', '127.0.0.1', '255.255.255.0', {WORLD_PORT}, 0, 0, 0, 0, 0, {CLIENT_BUILD}) \
         ON DUPLICATE KEY UPDATE name='{rn}', address='127.0.0.1', localAddress='127.0.0.1', localSubnetMask='255.255.255.0', port={WORLD_PORT}, gamebuild={CLIENT_BUILD}"
    ))
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Arguments communs : exécutable et `-c <conf>` depuis la racine courte (chemin sans espace).
fn server_paths(layout: &Layout, exe: &str, conf: &str) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let short = layout.short_root()?;
    let bin = short.join("Server").join("acore");
    Ok((bin.join(exe), bin.join("configs").join(conf), bin))
}

pub struct AuthServer {
    child: Child,
}

impl AuthServer {
    pub fn start(layout: &Layout, timeout: Duration) -> Result<AuthServer, String> {
        if port_open(AUTH_PORT) {
            return Err(format!("le port {AUTH_PORT} est déjà utilisé par un autre programme"));
        }
        let (exe, conf, bin) = server_paths(layout, "authserver.exe", "authserver.conf")?;
        let logs = layout.server_logs();
        std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
        let out = std::fs::File::create(logs.join("authserver.stdout.log")).map_err(|e| e.to_string())?;
        let err = std::fs::File::create(logs.join("authserver.stderr.log")).map_err(|e| e.to_string())?;
        let mut child = Command::new(&exe)
            .arg("-c")
            .arg(&conf)
            .current_dir(&bin)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .spawn()
            .map_err(|e| format!("authserver : {e}"))?;
        let start = Instant::now();
        while start.elapsed() < timeout {
            if port_open(AUTH_PORT) {
                return Ok(AuthServer { child });
            }
            if let Ok(Some(st)) = child.try_wait() {
                let hint = std::fs::read_to_string(logs.join("authserver.stderr.log")).unwrap_or_default();
                let hint = hint.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").chars().take(200).collect::<String>();
                return Err(format!("authserver s'est arrêté (code {:?}) {hint} — voir Logs/server/Auth.log", st.code()));
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        let _ = child.kill();
        Err("authserver ne répond pas dans le délai imparti".into())
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// authserver n'a pas de console : arrêt par terminaison (il est sans état).
    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct WorldServer {
    child: Child,
    stdin: Option<ChildStdin>,
    pub world_ready: Arc<AtomicBool>,
}

impl WorldServer {
    /// Démarre worldserver avec sa console sur l'entrée standard ; chaque ligne de sortie est envoyée dans `sink`.
    pub fn start(layout: &Layout, sink: Sender<String>) -> Result<WorldServer, String> {
        if port_open(WORLD_PORT) {
            return Err(format!("le port {WORLD_PORT} est déjà utilisé par un autre programme"));
        }
        let (exe, conf, bin) = server_paths(layout, "worldserver.exe", "worldserver.conf")?;
        let mut child = Command::new(&exe)
            .arg("-c")
            .arg(&conf)
            .current_dir(&bin)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("worldserver : {e}"))?;
        let world_ready = Arc::new(AtomicBool::new(false));
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().ok_or("sortie standard de worldserver indisponible")?;
        let stderr = child.stderr.take().ok_or("sortie d'erreur de worldserver indisponible")?;
        let console_log = layout.server_logs().join("worldserver.console.log");
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
                            if line.contains("World Initialized") || line.contains("World initialized") {
                                ready.store(true, Ordering::SeqCst);
                            }
                            if let Some(f) = file.as_mut() {
                                let _ = writeln!(f, "{line}");
                            }
                            // Les barres de progression des updates sont très verbeuses : on filtre les lignes « >> Applying ».
                            if line.starts_with(">> Applying") {
                                continue;
                            }
                            let _ = sink.send(if is_err { format!("[worldserver!] {line}") } else { format!("[worldserver] {line}") });
                        }
                    }
                }
            });
        }
        Ok(WorldServer { child, stdin, world_ready })
    }

    /// Prêt = port du monde ouvert ET « World Initialized » lu sur la console.
    pub fn wait_ready(&mut self, timeout: Duration) -> Result<(), String> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self.world_ready.load(Ordering::SeqCst) && port_open(WORLD_PORT) {
                return Ok(());
            }
            if let Ok(Some(st)) = self.child.try_wait() {
                return Err(format!("worldserver s'est arrêté pendant le chargement (code {:?}), voir Logs/server/worldserver.console.log", st.code()));
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

    /// « server shutdown 1 » puis fermeture de l'entrée standard (le thread console ne se termine qu'à la fin de stdin).
    /// Terminaison forcée seulement après le délai.
    pub fn stop(&mut self, timeout: Duration) -> bool {
        let _ = self.send("server shutdown 1");
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

/// HermesProxy (client 3.4.3) : processus séparé, journal dans Logs/server/hermesproxy.log.
pub struct HermesProxy {
    child: Child,
}

impl HermesProxy {
    pub fn exe(layout: &Layout) -> PathBuf {
        layout.hermes_dir().join("HermesProxy.exe")
    }

    pub fn is_installed(layout: &Layout) -> bool {
        Self::exe(layout).is_file()
    }

    pub fn start(layout: &Layout, extra_args: &str, timeout: Duration) -> Result<HermesProxy, String> {
        let exe = Self::exe(layout);
        if !exe.is_file() {
            return Err(format!("HermesProxy absent : {}", exe.display()));
        }
        if port_open(crate::paths::HERMES_BNET_PORT) {
            return Err(format!("le port {} (HermesProxy) est déjà utilisé", crate::paths::HERMES_BNET_PORT));
        }
        let logs = layout.server_logs();
        std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
        let out = std::fs::File::create(logs.join("hermesproxy.log")).map_err(|e| e.to_string())?;
        let err = out.try_clone().map_err(|e| e.to_string())?;
        let mut cmd = Command::new(&exe);
        // Serveur 3.3.5a local : hôte et port d'authentification, build serveur WotLK.
        cmd.args(["--set", "ServerAddress=127.0.0.1"]).args(["--set", &format!("ServerPort={AUTH_PORT}")]).args(["--set", "ServerBuild=12340"]);
        for a in extra_args.split_whitespace() {
            cmd.arg(a);
        }
        let mut child = cmd
            .current_dir(layout.hermes_dir())
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .spawn()
            .map_err(|e| format!("HermesProxy : {e}"))?;
        let start = Instant::now();
        while start.elapsed() < timeout {
            if port_open(crate::paths::HERMES_BNET_PORT) {
                return Ok(HermesProxy { child });
            }
            if let Ok(Some(st)) = child.try_wait() {
                return Err(format!("HermesProxy s'est arrêté (code {:?}), voir Logs/server/hermesproxy.log", st.code()));
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        let _ = child.kill();
        Err("HermesProxy ne répond pas dans le délai imparti".into())
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
