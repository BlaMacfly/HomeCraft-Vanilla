//! realmd et mangosd : génération des configurations, royaume, démarrage, console, arrêt propre.

use crate::db::{port_open, MariaDb, CREATE_NO_WINDOW};
use crate::paths::{Layout, MARIADB_PORT, REALMD_PORT, REALM_NAME, WORLD_PORT};
use crate::settings::Settings;
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
    Err(format!("clé absente du modèle CMaNGOS : {key}"))
}

/// Comme `set_conf_value`, mais ajoute la clé en fin de fichier si le modèle ne l'a qu'en commentaire (beaucoup
/// d'options de aiplayerbot.conf.dist ne sont présentes que sous la forme « # AiPlayerbot.X = défaut »).
fn set_or_append(lines: &mut Vec<String>, key: &str, value: &str) {
    if set_conf_value(lines, key, value).is_err() {
        lines.push(format!("{key} = {value}"));
    }
}

fn db_info(creds: &crate::credentials::DbCredentials, db: &str) -> String {
    format!("\"127.0.0.1;{MARIADB_PORT};{};{};{db}\"", creds.user, creds.password)
}

fn read_template(layout: &Layout, name: &str) -> Result<Vec<String>, String> {
    let p = layout.server_bin().join(name);
    let t = std::fs::read_to_string(&p).map_err(|e| format!("{name} : {e}"))?;
    Ok(t.lines().map(str::to_string).collect())
}

/// Comptes de bots aléatoires à créer : playerbots crée 10 personnages par compte et connecte ses bots parmi eux.
/// Une marge de 20 % laisse des personnages disponibles quand certains sont déjà pris (groupes, guildes).
pub fn random_bot_accounts(max_bots: u32) -> u32 {
    (max_bots * 12).div_ceil(100).max(1)
}

/// Les .conf sont régénérés à chaque démarrage depuis les modèles .conf.dist de Server\cmangos ; ils sont écrits
/// dans Server\Config (mangosd reçoit leurs chemins en arguments).
pub fn generate_configs(layout: &Layout, db: &MariaDb, settings: &Settings) -> Result<(), String> {
    let cfg = layout.server_config();
    let log_dir = layout.server_logs();
    std::fs::create_dir_all(&cfg).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(layout.server_data()).map_err(|e| e.to_string())?;
    let creds = db.creds();
    let q = |p: std::path::PathBuf| format!("\"{}\"", Layout::conf_path(&p));

    // realmd.conf
    let mut r = read_template(layout, "realmd.conf.dist")?;
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
    let mut m = read_template(layout, "mangosd.conf.dist")?;
    set_conf_value(&mut m, "DataDir", &q(layout.server_data()))?;
    set_conf_value(&mut m, "LogsDir", &q(log_dir.clone()))?;
    set_conf_value(&mut m, "LoginDatabaseInfo", &db_info(creds, "realmd"))?;
    set_conf_value(&mut m, "WorldDatabaseInfo", &db_info(creds, "mangos"))?;
    set_conf_value(&mut m, "CharacterDatabaseInfo", &db_info(creds, "characters"))?;
    set_conf_value(&mut m, "LogsDatabaseInfo", &db_info(creds, "logs"))?;
    set_conf_value(&mut m, "BindIP", "\"127.0.0.1\"")?;
    set_conf_value(&mut m, "WorldServerPort", &WORLD_PORT.to_string())?;
    set_conf_value(&mut m, "PidFile", &q(log_dir.join("mangosd.pid")))?;
    set_conf_value(&mut m, "WaitAtStartupError", "0")?;
    set_conf_value(&mut m, "DBC.Locale", "255")?;
    set_conf_value(&mut m, "Console.Enable", "1")?;
    set_conf_value(&mut m, "Ra.Enable", "0")?;
    set_conf_value(&mut m, "SOAP.Enabled", "0")?;
    set_conf_value(&mut m, "mmap.enabled", "1")?;
    // Chaque bot connecté ouvre une session : avec la limite du modèle (100), au-delà de 100 bots le joueur était
    // mis en file d'attente (le client restait sur « Connecté »). 0 = pas de limite.
    set_conf_value(&mut m, "PlayerLimit", "0")?;
    // Réglages de l'onglet Monde : valeurs du catalogue (défaut = modèle, surcharges de l'utilisateur). Seules les
    // clés du catalogue sont surchargeables ; les clés gérées ci-dessus ne le sont jamais.
    for opt in crate::conf_catalog::WORLD_OPTIONS {
        let v = crate::conf_catalog::value(&settings.conf, opt);
        set_conf_value(&mut m, opt.key, &crate::conf_catalog::conf_value(opt, v))?;
    }
    std::fs::write(cfg.join("mangosd.conf"), m.join("\n") + "\n").map_err(|e| e.to_string())?;

    // aiplayerbot.conf : réglages de l'onglet PlayerBots, plus les clés gérées.
    let mut p = read_template(layout, "aiplayerbot.conf.dist")?;
    for opt in crate::conf_catalog::BOT_OPTIONS {
        let v = crate::conf_catalog::value(&settings.conf, opt);
        set_or_append(&mut p, opt.key, &crate::conf_catalog::conf_value(opt, v));
    }
    let max_bots: u32 = crate::conf_catalog::value(&settings.conf, crate::conf_catalog::bot_opt("AiPlayerbot.MaxRandomBots")).parse().unwrap_or(0);
    set_or_append(&mut p, "AiPlayerbot.Enabled", if settings.partybots_enabled { "1" } else { "0" });
    set_or_append(&mut p, "AiPlayerbot.RandomBotAccountCount", &random_bot_accounts(max_bots).to_string());
    set_or_append(&mut p, "AiPlayerbot.DeleteRandomBotAccounts", "0");
    set_or_append(&mut p, "AiPlayerbot.RandomBotAccountPrefix", crate::botgen::BOT_ACCOUNT_PREFIX);
    // Sinon le mot de passe d'un compte de bot est son nom : n'importe qui pourrait s'y connecter.
    set_or_append(&mut p, "AiPlayerbot.RandomBotRandomPassword", "1");
    // Cartes du jeu Vanilla seulement (le modèle cite aussi l'Outreterre et le Norfendre).
    set_or_append(&mut p, "AiPlayerbot.RandomBotMaps", "0,1");
    // Serveur hors ligne : aucune discussion par modèle de langage (activée par défaut, vers un service local).
    set_or_append(&mut p, "AiPlayerbot.LLMEnabled", "0");
    std::fs::write(cfg.join("aiplayerbot.conf"), p.join("\n") + "\n").map_err(|e| e.to_string())?;

    // ahbot.conf : AHBot du cœur (achète et vend aux trois hôtels des ventes). L'AHBot du module playerbots ne lit que
    // le ahbot.conf du dossier de travail (Server\cmangos, où il n'y en a pas) ; AhBot.Enabled = 0 le garantit si
    // quelqu'un y copiait ce fichier.
    let mut a = read_template(layout, "ahbot.conf.dist")?;
    let chance = if settings.ahbot_enabled { "10" } else { "0" };
    set_conf_value(&mut a, "AuctionHouseBot.Chance.Sell", chance)?;
    set_conf_value(&mut a, "AuctionHouseBot.Chance.Buy", chance)?;
    set_or_append(&mut a, "AhBot.Enabled", "0");
    std::fs::write(cfg.join("ahbot.conf"), a.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(())
}

/// Builds 1.12.1 acceptés : 5875 (version de référence) et 5880 (variante livrée par certains clients, dont
/// le client frFR fourni : l'écran de connexion affiche « 1.12.1 (5880) » bien que la ressource FileVersion dise 5875).
/// realmd et mangosd les connaissent grâce à Build\cmangos-patches\homecraft.patch.
pub const CLIENT_BUILDS: [u16; 2] = [5875, 5880];

/// Royaume local : `realmlist.realmbuilds` doit citer le build du client, sinon le royaume apparaît hors ligne.
pub fn ensure_realm(db: &MariaDb) -> Result<(), String> {
    let mut c = db.root_conn()?;
    let builds = CLIENT_BUILDS.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(" ");
    c.query_drop(format!(
        "INSERT INTO realmd.realmlist (id,name,address,port,icon,realmflags,timezone,allowedSecurityLevel,population,realmbuilds) \
         VALUES (1,'{REALM_NAME}','127.0.0.1',{WORLD_PORT},1,0,1,0,0,'{builds}') \
         ON DUPLICATE KEY UPDATE name='{REALM_NAME}', address='127.0.0.1', port={WORLD_PORT}, realmflags=0, realmbuilds='{builds}'"
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
        let cfg = layout.server_config();
        // Mesures mémoire du module playerbots (compilées d'office, une ligne par mise à jour, ~10 Mo par jour de
        // jeu) : repartent de zéro à chaque démarrage.
        for f in ["memory.csv", "heap.csv"] {
            let _ = std::fs::remove_file(layout.server_logs().join(f));
        }
        // Dossier de travail = Server\cmangos : mangosd y cherche le module OpenSSL legacy.dll. L'AHBot du module
        // playerbots y cherche ahbot.conf sans chemin : il n'y en a pas, il reste désactivé (seul l'AHBot du cœur,
        // configuré par -a, approvisionne les hôtels des ventes).
        let mut child = Command::new(bin.join("mangosd.exe"))
            .arg("-c")
            .arg(cfg.join("mangosd.conf"))
            .arg("-a")
            .arg(cfg.join("ahbot.conf"))
            .arg("-p")
            .arg(cfg.join("aiplayerbot.conf"))
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
                            let line = crate::util::decode_console(&buf).trim_end().to_string();
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
