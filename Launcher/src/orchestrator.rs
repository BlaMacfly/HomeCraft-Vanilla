//! Machine à états et pilotage des services, sur un thread dédié. L'interface n'y accède que par
//! des commandes (canal) et par un instantané partagé (`Snapshot`).

use crate::backup::{self, BackupInfo};
use crate::client::{self, ClientInfo};
use crate::credentials;
use crate::db::{self, MySql, QueryResult};
use crate::i18n::fr;
use crate::paths::{Layout, ALL_DATABASES, AUTH_PORT, DB_PORT, DB_WORLD, HERMES_BNET_PORT, WORLD_PORT};
use crate::repair::{self, ExtractStep, VerifyReport};
use crate::server::{self, AuthServer, HermesProxy, WorldServer};
use crate::settings::Settings;
use crate::util;
use std::collections::VecDeque;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Child;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    NotInstalled,
    Installed,
    Verifying,
    Initializing,
    StartingDb,
    StartingAuth,
    StartingWorld,
    Ready,
    Playing,
    Stopping,
    Error,
    Repairing,
}

impl State {
    pub fn label(self) -> &'static str {
        match self {
            State::NotInstalled => fr::STATE_NOT_INSTALLED,
            State::Installed => fr::STATE_INSTALLED,
            State::Verifying => fr::STATE_VERIFYING,
            State::Initializing => fr::STATE_INITIALIZING,
            State::StartingDb => fr::STATE_STARTING_DB,
            State::StartingAuth => fr::STATE_STARTING_AUTH,
            State::StartingWorld => fr::STATE_STARTING_WORLD,
            State::Ready => fr::STATE_READY,
            State::Playing => fr::STATE_PLAYING,
            State::Stopping => fr::STATE_STOPPING,
            State::Error => fr::STATE_ERROR,
            State::Repairing => fr::STATE_REPAIRING,
        }
    }
    pub fn is_busy(self) -> bool {
        matches!(self, State::Verifying | State::Initializing | State::StartingDb | State::StartingAuth | State::StartingWorld | State::Stopping | State::Repairing)
    }
    pub fn services_up(self) -> bool {
        matches!(self, State::Ready | State::Playing)
    }
}

#[derive(Clone, Debug, Default)]
pub struct AccountRow {
    pub username: String,
    pub gmlevel: u8,
    pub last_login: String,
    pub characters: u64,
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub state: State,
    pub message: String,
    pub error: Option<String>,
    pub progress: f32,
    pub db_up: bool,
    pub auth_up: bool,
    pub world_up: bool,
    pub hermes_up: bool,
    pub hermes_installed: bool,
    pub bots_enabled: bool,
    pub client: ClientInfo,
    pub online: u64,
    pub logs: VecDeque<String>,
    pub accounts: Vec<AccountRow>,
    pub databases: Vec<String>,
    pub tables: Vec<String>,
    pub query: Option<Result<QueryResult, String>>,
    pub notice: Option<String>,
    pub components: Vec<(String, bool)>,
    pub backups: Vec<BackupInfo>,
    pub report: Option<VerifyReport>,
    pub report_fresh: bool,
    pub readonly: bool,
    pub exit_requested: bool,
    /// (comptes rndbot, personnages de bots)
    pub bot_pool: Option<(u64, u64)>,
    pub patches: Vec<crate::patches::PatchEntry>,
    pub addons: Vec<crate::addons::AddonEntry>,
    pub table_view: Option<TableView>,
    /// Une écriture a eu lieu : la page affichée doit être rechargée
    pub table_dirty: bool,
    pub mmaps: MmapsStatus,
    /// Modules du cœur et leur activation (lue dans configs/modules/*.conf)
    pub modules: Vec<crate::modules_catalog::ModuleState>,
}

/// État des cartes de déplacement (mmaps), générées en arrière-plan.
#[derive(Clone, Debug, PartialEq)]
pub enum MmapsStatus {
    Unknown,
    Present,
    Absent,
    /// Génération lancée à cet instant
    Running(std::time::Instant),
    /// Générées pendant cette session : prises en compte au prochain démarrage du serveur
    DoneRestartNeeded,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct TableView {
    pub db: String,
    pub table: String,
    pub columns: Vec<crate::db::ColumnInfo>,
    pub page: QueryResult,
    pub total: u64,
    pub offset: u64,
}

pub enum Command {
    StartAll,
    Play,
    StopAll,
    Verify,
    Repair,
    GenerateMmaps,
    Backup(String),
    RefreshBackups,
    Restore(PathBuf),
    DeleteBackup(PathBuf),
    ExportDiagnostic,
    Console(String),
    RefreshAccounts,
    CreateAccount { user: String, pass: String, gm: u8 },
    SetPassword { user: String, pass: String },
    SetGm { user: String, gm: u8 },
    DeleteAccount(String),
    ListTables(String),
    Query { db: String, sql: String },
    /// Instruction d'écriture confirmée par l'utilisateur
    Exec { db: String, sql: String },
    /// Charge structure + page de données d'une table
    LoadTable { db: String, table: String, filter: String, order: Option<(String, bool)>, offset: u64 },
    OpenHeidi,
    /// Lance Keira3 (éditeur de la base monde)
    OpenKeira,
    SettingsChanged(Settings),
    InstallTo(PathBuf),
    KillGame,
    /// Supprime tous les comptes/personnages de bots aléatoires (serveur arrêté)
    DeleteBots,
    RefreshPatches,
    /// nom logique du patch (ex. patch-4.MPQ), actif ou non
    PatchSetActive(String, bool),
    PatchAdd(PathBuf),
    PatchRemove(String),
    RefreshAddons,
    AddonAdd(PathBuf),
    AddonSetActive(String, bool),
    AddonRemove(String),
    /// Recrée la base monde depuis le package ; personnages et comptes conservés
    ReimportWorld,
    RefreshModules,
    /// (identifiant du module, activé)
    ModuleSetEnabled(String, bool),
    /// Rétablit le modèle .conf.dist du module
    ModuleReset(String),
    Shutdown,
}

pub struct Orchestrator {
    layout: Layout,
    settings: Settings,
    shared: Arc<Mutex<Snapshot>>,
    rx: Receiver<Command>,
    log_tx: Sender<String>,
    log_rx: Receiver<String>,
    db: Option<MySql>,
    /// Le processus mysqld a été lancé par ce launcher (sinon : base d'une autre instance ou d'un script, jamais arrêtée ici)
    db_owned: bool,
    /// Base démarrée pour une opération annexe (sauvegarde, réparation…) : arrêtée dès la fin de l'opération si possédée
    db_annex: bool,
    auth: Option<AuthServer>,
    world: Option<WorldServer>,
    hermes: Option<HermesProxy>,
    game: Option<Child>,
    log_file: Option<std::fs::File>,
    /// mmaps_generator en arrière-plan
    mmaps_job: Option<Child>,
}

pub fn spawn(layout: Layout, settings: Settings) -> (Arc<Mutex<Snapshot>>, Sender<Command>) {
    let shared = Arc::new(Mutex::new(Snapshot {
        state: State::Installed,
        message: fr::MSG_IDLE.into(),
        error: None,
        progress: 0.0,
        db_up: false,
        auth_up: false,
        world_up: false,
        hermes_up: false,
        hermes_installed: HermesProxy::is_installed(&layout),
        bots_enabled: settings.random_bots > 0,
        client: ClientInfo::default(),
        online: 0,
        logs: VecDeque::new(),
        accounts: Vec::new(),
        databases: ALL_DATABASES.iter().map(|d| d.to_string()).collect(),
        tables: Vec::new(),
        query: None,
        notice: None,
        components: Vec::new(),
        backups: Vec::new(),
        report: None,
        report_fresh: false,
        readonly: false,
        exit_requested: false,
        bot_pool: None,
        patches: Vec::new(),
        addons: Vec::new(),
        table_view: None,
        table_dirty: false,
        mmaps: MmapsStatus::Unknown,
        modules: crate::modules_catalog::states(&layout.server_config().join("modules")),
    }));
    let (tx, rx) = mpsc::channel();
    let (log_tx, log_rx) = mpsc::channel();
    let shared2 = shared.clone();
    std::thread::Builder::new()
        .name("orchestrateur".into())
        .spawn(move || {
            let log_file = std::fs::create_dir_all(layout.logs()).ok().and_then(|_| std::fs::OpenOptions::new().create(true).append(true).open(layout.launcher_log()).ok());
            let mut o = Orchestrator { layout, settings, shared: shared2, rx, log_tx, log_rx, db: None, db_owned: false, db_annex: false, auth: None, world: None, hermes: None, game: None, log_file, mmaps_job: None };
            o.run();
        })
        .expect("thread orchestrateur");
    (shared, tx)
}

impl Orchestrator {
    fn log(&mut self, line: impl Into<String>) {
        let line = line.into();
        let stamped = format!("{} {line}", util::time_hms());
        if let Some(f) = self.log_file.as_mut() {
            let _ = writeln!(f, "{stamped}");
        }
        let mut s = self.shared.lock().unwrap();
        s.logs.push_back(stamped);
        while s.logs.len() > 3000 {
            s.logs.pop_front();
        }
    }

    fn set(&self, state: State, message: &str, progress: f32) {
        let mut s = self.shared.lock().unwrap();
        s.state = state;
        s.message = message.to_string();
        s.progress = progress;
        if state != State::Error {
            s.error = None;
        }
    }

    fn set_message(&self, message: String) {
        self.shared.lock().unwrap().message = message;
    }

    fn notice(&self, text: String) {
        self.shared.lock().unwrap().notice = Some(text);
    }

    fn state(&self) -> State {
        self.shared.lock().unwrap().state
    }

    fn fail(&mut self, context: &str, err: String) {
        let msg = format!("{context} : {err}");
        self.log(format!("ERREUR {msg}"));
        let mut s = self.shared.lock().unwrap();
        s.state = State::Error;
        s.error = Some(msg.clone());
        s.message = msg;
        s.progress = 0.0;
    }

    fn client_dir(&self) -> PathBuf {
        if self.settings.client_dir.trim().is_empty() { self.layout.default_client_dir() } else { PathBuf::from(self.settings.client_dir.trim()) }
    }

    fn client_info(&self) -> ClientInfo {
        client::inspect(&self.client_dir(), self.settings.client_kind)
    }

    fn drain_server_logs(&mut self) {
        while let Ok(l) = self.log_rx.try_recv() {
            self.log(l);
        }
    }

    fn run(&mut self) {
        if !self.layout.is_writable() {
            self.log(format!("support en lecture seule : {}", self.layout.root.display()));
            let mut s = self.shared.lock().unwrap();
            s.readonly = true;
            s.state = State::NotInstalled;
            s.message = fr::RO_TITLE.into();
        } else {
            self.verify(false);
            self.refresh_backups();
        }
        loop {
            self.drain_server_logs();
            match self.rx.recv_timeout(Duration::from_millis(1000)) {
                Ok(Command::Shutdown) => {
                    self.stop_all();
                    self.abort_mmaps();
                    break;
                }
                Ok(cmd) => self.handle(cmd),
                Err(mpsc::RecvTimeoutError::Timeout) => self.tick(),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.stop_all();
                    self.abort_mmaps();
                    break;
                }
            }
        }
    }

    fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::StartAll => {
                if self.start_all() {
                    self.set(State::Ready, fr::MSG_READY, 1.0);
                }
            }
            Command::Play => {
                if !self.state().services_up() && !self.start_all() {
                    return;
                }
                self.play();
            }
            Command::StopAll => self.stop_all(),
            Command::Verify => self.verify(true),
            Command::Repair => self.repair(),
            Command::GenerateMmaps => self.start_mmaps(true),
            Command::ReimportWorld => self.reimport_world(),
            Command::Backup(label) => self.backup(&label),
            Command::RefreshBackups => self.refresh_backups(),
            Command::Restore(p) => self.restore(&p),
            Command::DeleteBackup(p) => {
                match std::fs::remove_file(&p) {
                    Ok(()) => self.log(format!("sauvegarde supprimée : {}", p.display())),
                    Err(e) => self.log(format!("suppression impossible : {e}")),
                }
                self.refresh_backups();
            }
            Command::ExportDiagnostic => {
                let report = self.shared.lock().unwrap().report.clone();
                match repair::export_diagnostic(&self.layout, report.as_ref()) {
                    Ok(p) => {
                        self.log(format!("diagnostic exporté : {}", p.display()));
                        self.notice(format!("{}{}", fr::MSG_DIAG_DONE, p.display()));
                    }
                    Err(e) => self.log(format!("diagnostic : {e}")),
                }
            }
            Command::Console(c) => self.console(&c),
            Command::RefreshAccounts => self.refresh_accounts(),
            Command::CreateAccount { user, pass, gm } => {
                // La première commande envoyée juste après le démarrage peut être ignorée : on précède d'une commande neutre.
                self.console("server info");
                std::thread::sleep(Duration::from_millis(300));
                self.console(&format!("account create {user} {pass}"));
                std::thread::sleep(Duration::from_millis(600));
                if gm > 0 {
                    self.console(&format!("account set gmlevel {user} {gm} -1"));
                    std::thread::sleep(Duration::from_millis(400));
                }
                self.refresh_accounts();
            }
            Command::SetPassword { user, pass } => self.console(&format!("account set password {user} {pass} {pass}")),
            Command::DeleteAccount(user) => {
                if self.state() == State::Playing {
                    self.notice(fr::ACC_DELETE_PLAYING.into());
                    return;
                }
                if self.world.is_some() {
                    // Le serveur supprime lui-même le compte et ses personnages.
                    self.console(&format!("account delete {user}"));
                    std::thread::sleep(Duration::from_millis(800));
                    self.log(format!("compte {user} : suppression demandée au serveur"));
                } else {
                    let was_temp = self.db.is_none();
                    let r = self.ensure_db().and_then(|db| db.delete_account_offline(&user));
                    match r {
                        Ok(n) => {
                            self.log(format!("compte {user} supprimé ({n} personnage(s))"));
                            self.notice(format!("{} {user} ({n} {})", fr::ACC_DELETED, fr::BK_CHARS.to_lowercase()));
                        }
                        Err(e) => self.notice(format!("{} : {e}", fr::ACC_DELETE_FAIL)),
                    }
                    self.refresh_accounts();
                    if was_temp {
                        self.release_temp_db();
                    }
                    return;
                }
                self.refresh_accounts();
            }
            Command::SetGm { user, gm } => {
                self.console(&format!("account set gmlevel {user} {gm} -1"));
                std::thread::sleep(Duration::from_millis(400));
                self.refresh_accounts();
            }
            Command::ListTables(dbname) => {
                let r = self.db.as_ref().ok_or(fr::DB_NEED_DB.to_string()).and_then(|d| d.tables(&dbname));
                let mut s = self.shared.lock().unwrap();
                match r {
                    Ok(t) => s.tables = t,
                    Err(e) => s.query = Some(Err(e)),
                }
            }
            Command::Query { db: dbname, sql } => {
                let r = self.db.as_ref().ok_or(fr::DB_NEED_DB.to_string()).and_then(|d| d.query(Some(&dbname), &sql, 500));
                self.shared.lock().unwrap().query = Some(r);
            }
            Command::Exec { db: dbname, sql } => {
                let r = self.db.as_ref().ok_or(fr::DB_NEED_DB.to_string()).and_then(|d| d.exec(Some(&dbname), &sql));
                match &r {
                    Ok(n) => self.log(format!("SQL ({dbname}) : {n} ligne(s) affectée(s) — {}", sql.chars().take(160).collect::<String>())),
                    Err(e) => self.log(format!("SQL ({dbname}) ERREUR : {e}")),
                }
                let mut s = self.shared.lock().unwrap();
                s.query = Some(r.map(|n| QueryResult { affected: Some(n), ..Default::default() }));
                s.table_dirty = true;
            }
            Command::LoadTable { db: dbname, table, filter, order, offset } => {
                let r = self.db.as_ref().ok_or(fr::DB_NEED_DB.to_string()).and_then(|d| {
                    let cols = d.table_columns(&dbname, &table)?;
                    let (page, total) = d.table_rows(&dbname, &table, &filter, order.as_ref().map(|(c, a)| (c.as_str(), *a)), offset, 200)?;
                    Ok(TableView { db: dbname.clone(), table: table.clone(), columns: cols, page, total, offset })
                });
                let mut s = self.shared.lock().unwrap();
                match r {
                    Ok(v) => {
                        s.table_view = Some(v);
                        s.query = None;
                    }
                    Err(e) => s.query = Some(Err(e)),
                }
                s.table_dirty = false;
            }
            Command::OpenHeidi => self.open_heidi(),
            Command::OpenKeira => self.open_keira(),
            Command::SettingsChanged(s) => {
                self.settings = s;
                self.shared.lock().unwrap().bots_enabled = self.settings.random_bots > 0;
                self.verify(false);
            }
            Command::InstallTo(dest) => self.install_to(&dest),
            Command::RefreshPatches => self.refresh_patches(),
            Command::RefreshAddons => self.refresh_addons(),
            Command::AddonAdd(src) => {
                if self.state() == State::Playing {
                    self.notice(fr::ADDON_PLAYING.into());
                    return;
                }
                let prev = self.shared.lock().unwrap().message.clone();
                let shared = self.shared.clone();
                let root = self.client_info().interface_root();
                match crate::addons::add_from_zip(&src, &root, &|m| shared.lock().unwrap().message = m.to_string()) {
                    Ok(names) => {
                        self.log(format!("AddOns installés : {} (depuis {})", names.join(", "), src.display()));
                        self.notice(format!("{} {}", fr::ADDON_ADDED, names.join(", ")));
                    }
                    Err(e) => self.notice(format!("{} : {e}", fr::ADDON_ADD_FAIL)),
                }
                self.shared.lock().unwrap().message = prev;
                self.refresh_addons();
            }
            Command::AddonSetActive(name, active) => {
                if self.state() == State::Playing {
                    self.notice(fr::ADDON_PLAYING.into());
                    return;
                }
                let entries = crate::addons::list(&self.client_info().interface_root());
                if let Some(e) = entries.iter().find(|a| a.name.eq_ignore_ascii_case(&name)) {
                    match crate::addons::set_active(e, active) {
                        Ok(()) => self.log(format!("AddOn {} {}", e.name, if active { "activé" } else { "désactivé" })),
                        Err(err) => self.notice(err),
                    }
                }
                self.refresh_addons();
            }
            Command::AddonRemove(name) => {
                if self.state() == State::Playing {
                    self.notice(fr::ADDON_PLAYING.into());
                    return;
                }
                let entries = crate::addons::list(&self.client_info().interface_root());
                if let Some(e) = entries.iter().find(|a| a.name.eq_ignore_ascii_case(&name)) {
                    match crate::addons::remove(e) {
                        Ok(()) => self.log(format!("AddOn supprimé : {}", e.name)),
                        Err(err) => self.notice(err),
                    }
                }
                self.refresh_addons();
            }
            Command::PatchSetActive(name, active) => {
                if self.state() == State::Playing {
                    self.notice(fr::PATCH_PLAYING.into());
                    return;
                }
                let info = self.client_info();
                let entries = crate::patches::list(&info.dir, &info.locale);
                match entries.iter().find(|p| p.name.eq_ignore_ascii_case(&name)) {
                    Some(e) => match crate::patches::set_active(e, active) {
                        Ok(_) => self.log(format!("patch {} {}", e.name, if active { "activé" } else { "désactivé" })),
                        Err(err) => self.notice(format!("{} : {err}", e.name)),
                    },
                    None => self.notice(format!("{name} : introuvable")),
                }
                self.refresh_patches();
            }
            Command::PatchAdd(src) => {
                if self.state() == State::Playing {
                    self.notice(fr::PATCH_PLAYING.into());
                    return;
                }
                let prev = self.shared.lock().unwrap().message.clone();
                let shared = self.shared.clone();
                let info = self.client_info();
                match crate::patches::add(&src, &info.dir, &info.locale, &|m| shared.lock().unwrap().message = m.to_string()) {
                    Ok(name) => {
                        self.log(format!("patch ajouté : {name} (depuis {})", src.display()));
                        self.notice(format!("{} {name}", fr::PATCH_ADDED));
                    }
                    Err(e) => self.notice(format!("{} : {e}", fr::PATCH_ADD_FAIL)),
                }
                self.shared.lock().unwrap().message = prev;
                self.refresh_patches();
            }
            Command::PatchRemove(name) => {
                if self.state() == State::Playing {
                    self.notice(fr::PATCH_PLAYING.into());
                    return;
                }
                let info = self.client_info();
                let entries = crate::patches::list(&info.dir, &info.locale);
                if let Some(e) = entries.iter().find(|p| p.name.eq_ignore_ascii_case(&name)) {
                    match crate::patches::remove(e) {
                        Ok(()) => self.log(format!("patch supprimé : {}", e.name)),
                        Err(err) => self.notice(err),
                    }
                }
                self.refresh_patches();
            }
            Command::DeleteBots => {
                if self.world.is_some() || self.state().is_busy() {
                    self.notice(fr::BOTPOOL_DELETE_RUNNING.into());
                    return;
                }
                let was_temp = self.db.is_none();
                let r = self.ensure_db().and_then(|db| db.delete_bot_pool());
                match r {
                    Ok(k) => {
                        self.log(format!("{k} personnage(s) de bots supprimé(s), comptes rndbot supprimés"));
                        self.notice(format!("{k} personnages de bots supprimés."));
                    }
                    Err(e) => self.log(format!("ERREUR suppression des bots : {e}")),
                }
                self.refresh_accounts();
                if was_temp {
                    self.release_temp_db();
                }
            }
            Command::RefreshModules => self.refresh_modules(),
            Command::ModuleSetEnabled(id, on) => {
                match server::set_module_enabled(&self.layout, &id, on) {
                    Ok(()) => {
                        self.log(format!("module {id} {}", if on { "activé" } else { "désactivé" }));
                        if self.state().services_up() {
                            self.notice(fr::MOD_RESTART_HINT.into());
                        }
                    }
                    Err(e) => self.notice(format!("{} : {e}", fr::MOD_FAIL)),
                }
                self.refresh_modules();
            }
            Command::ModuleReset(id) => {
                match server::reset_module_conf(&self.layout, &id) {
                    Ok(()) => {
                        self.log(format!("module {id} : configuration rétablie depuis le modèle"));
                        self.notice(fr::MOD_RESET_DONE.into());
                    }
                    Err(e) => self.notice(format!("{} : {e}", fr::MOD_FAIL)),
                }
                self.refresh_modules();
            }
            Command::KillGame => {
                if let Some(mut g) = self.game.take() {
                    let _ = g.kill();
                    let _ = g.wait();
                    self.log("jeu fermé de force à la demande de l'utilisateur");
                }
                let info = self.client_info();
                for name in client::game_process_names(&info) {
                    let _ = std::process::Command::new("taskkill").args(["/F", "/IM", name]).creation_flags(crate::db::CREATE_NO_WINDOW).output();
                }
                self.log(format!("processus du jeu terminés (client : {})", info.dir.display()));
                if self.state() == State::Playing {
                    self.set(State::Ready, fr::MSG_READY, 1.0);
                }
            }
            Command::Shutdown => {}
        }
    }

    fn refresh_patches(&mut self) {
        let info = self.client_info();
        let list = if info.is_classic() { Vec::new() } else { crate::patches::list(&info.dir, &info.locale) };
        self.shared.lock().unwrap().patches = list;
        self.refresh_addons();
    }

    fn refresh_modules(&mut self) {
        let list = crate::modules_catalog::states(&self.layout.server_config().join("modules"));
        let mut s = self.shared.lock().unwrap();
        s.bots_enabled = self.settings.random_bots > 0 && list.iter().find(|m| m.id == "mod-playerbots").and_then(|m| m.enabled).unwrap_or(true);
        s.modules = list;
    }

    fn refresh_addons(&mut self) {
        let list = crate::addons::list(&self.client_info().interface_root());
        self.shared.lock().unwrap().addons = list;
    }

    /// Installation depuis un support en lecture seule : copie intégrale vers `dest`, puis lancement du launcher copié.
    fn install_to(&mut self, dest: &std::path::Path) {
        self.set(State::Repairing, fr::MSG_INSTALLING, 0.0);
        let shared = self.shared.clone();
        let r = std::fs::create_dir_all(dest).map_err(|e| e.to_string()).and_then(|_| {
            util::copy_tree(&self.layout.root, dest, &["UserData", "Logs", "target", "Build"], &|i, n| {
                let mut s = shared.lock().unwrap();
                s.progress = i as f32 / n.max(1) as f32;
                s.message = format!("{} {i}/{n}", fr::MSG_INSTALLING);
            })
        });
        match r {
            Ok(()) => {
                self.log(format!("copié vers {}", dest.display()));
                let exe_name = std::env::current_exe().ok().and_then(|p| p.file_name().map(|n| n.to_os_string())).unwrap_or_else(|| "HomeCraft Edition WoTLK.exe".into());
                let new_exe = dest.join(&exe_name);
                let launched = std::process::Command::new(&new_exe).current_dir(dest).spawn().is_ok();
                self.log(if launched { "launcher installé démarré" } else { "lancement du launcher installé impossible : ouvrez-le manuellement" });
                let mut s = self.shared.lock().unwrap();
                s.state = State::NotInstalled;
                s.message = fr::MSG_INSTALLED.into();
                s.progress = 0.0;
                s.exit_requested = launched;
            }
            Err(e) => self.fail("installation", e),
        }
    }

    /// Contrôle périodique : santé réelle des services, fin du jeu, plantages.
    fn tick(&mut self) {
        let db_up = self.db.as_ref().map(|d| d.ping()).unwrap_or(false);
        let auth_up = self.auth.as_mut().map(|r| r.is_running()).unwrap_or(false) && db::port_open(AUTH_PORT);
        let world_up = self.world.as_mut().map(|m| m.is_running()).unwrap_or(false) && db::port_open(WORLD_PORT);
        let hermes_up = self.hermes.as_mut().map(|h| h.is_running()).unwrap_or(false) && db::port_open(HERMES_BNET_PORT);
        let online = if db_up && world_up { self.db.as_ref().map(|d| d.online_count()).unwrap_or(0) } else { 0 };
        let state = self.state();
        {
            let mut s = self.shared.lock().unwrap();
            s.db_up = db_up;
            s.auth_up = auth_up;
            s.world_up = world_up;
            s.hermes_up = hermes_up;
            s.online = online;
        }
        if state.services_up() && !(db_up && auth_up && world_up) {
            let which = if !db_up { "base de données" } else if !auth_up { "serveur d'authentification" } else { "serveur de jeu" };
            self.fail("service interrompu", format!("{which} ne répond plus"));
            self.stop_services_quiet();
            return;
        }
        if state == State::Playing {
            if let Some(g) = self.game.as_mut() {
                if let Ok(Some(st)) = g.try_wait() {
                    self.log(format!("jeu terminé (code {:?})", st.code()));
                    self.game = None;
                    // Client 3.4.3 : le processus lancé est Arctium, qui se termine dès que le jeu est ouvert ;
                    // on surveille alors WowClassic.exe lui-même.
                    if self.client_info().is_classic() && process_running("WowClassic.exe") {
                        return;
                    }
                    self.set(State::Ready, fr::MSG_READY, 1.0);
                    if self.settings.auto_backup {
                        self.backup(fr::BK_AUTO_LABEL);
                        self.rotate_auto_backups();
                        self.set_message(fr::MSG_READY.into());
                    }
                }
            } else if self.client_info().is_classic() && !process_running("WowClassic.exe") {
                self.log("jeu terminé (WowClassic.exe)");
                self.set(State::Ready, fr::MSG_READY, 1.0);
                if self.settings.auto_backup {
                    self.backup(fr::BK_AUTO_LABEL);
                    self.rotate_auto_backups();
                    self.set_message(fr::MSG_READY.into());
                }
            }
        }
        self.poll_mmaps();
    }

    // ------------------------------------------------------------------ mmaps en arrière-plan

    fn start_mmaps(&mut self, force: bool) {
        if self.mmaps_job.is_some() {
            if force {
                self.notice(fr::MMAPS_ALREADY_RUNNING.into());
            }
            return;
        }
        let present = repair::data_status(&self.layout).iter().any(|(n, ok)| n == "mmaps" && *ok);
        if present && !force {
            self.shared.lock().unwrap().mmaps = MmapsStatus::Present;
            return;
        }
        let data = repair::data_status(&self.layout);
        let has = |n: &str| data.iter().any(|(k, ok)| k == n && *ok);
        if !has("maps") || !has("vmaps") {
            self.log("mmaps : cartes ou vmaps absentes, génération impossible");
            self.shared.lock().unwrap().mmaps = MmapsStatus::Absent;
            return;
        }
        let threads = (std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4) / 2).max(1);
        match repair::spawn_mmaps(&self.layout, threads, &self.log_tx) {
            Ok(child) => {
                self.log(format!("mmaps {} : génération en arrière-plan ({threads} threads)", if present { "régénérées" } else { "absentes" }));
                self.mmaps_job = Some(child);
                self.shared.lock().unwrap().mmaps = MmapsStatus::Running(std::time::Instant::now());
            }
            Err(e) => {
                self.log(format!("mmaps : {e}"));
                self.shared.lock().unwrap().mmaps = MmapsStatus::Failed(e);
            }
        }
    }

    fn poll_mmaps(&mut self) {
        let Some(job) = self.mmaps_job.as_mut() else { return };
        match job.try_wait() {
            Ok(Some(status)) => {
                self.mmaps_job = None;
                let ok = status.success();
                repair::mmaps_finished(&self.layout, ok);
                let present = repair::data_status(&self.layout).iter().any(|(n, k)| n == "mmaps" && *k);
                let mut s = self.shared.lock().unwrap();
                if ok && present {
                    s.mmaps = MmapsStatus::DoneRestartNeeded;
                    drop(s);
                    self.log("mmaps générées en arrière-plan : prises en compte au prochain démarrage du serveur");
                } else {
                    let e = format!("mmaps_generator terminé avec le code {:?}", status.code());
                    s.mmaps = MmapsStatus::Failed(e.clone());
                    drop(s);
                    self.log(format!("ERREUR mmaps : {e}"));
                }
            }
            Ok(None) => {}
            Err(e) => {
                self.mmaps_job = None;
                self.log(format!("ERREUR mmaps : {e}"));
                self.shared.lock().unwrap().mmaps = MmapsStatus::Failed(e.to_string());
            }
        }
    }

    fn abort_mmaps(&mut self) {
        if let Some(mut job) = self.mmaps_job.take() {
            let _ = job.kill();
            let _ = job.wait();
            self.log("génération des mmaps interrompue par la fermeture du launcher : reprendra au prochain démarrage du serveur");
        }
    }

    fn refresh_mmaps_status(&mut self) {
        if self.mmaps_job.is_some() {
            return;
        }
        let present = repair::data_status(&self.layout).iter().any(|(n, ok)| n == "mmaps" && *ok);
        let mut s = self.shared.lock().unwrap();
        if !matches!(s.mmaps, MmapsStatus::DoneRestartNeeded | MmapsStatus::Failed(_)) {
            s.mmaps = if present { MmapsStatus::Present } else { MmapsStatus::Absent };
        }
    }

    fn rotate_auto_backups(&mut self) {
        let keep = self.settings.auto_backup_keep.max(1) as usize;
        let auto: Vec<_> = backup::list(&self.layout).into_iter().filter(|b| b.manifest.label == fr::BK_AUTO_LABEL).collect();
        let mut removed = 0;
        for b in auto.iter().skip(keep) {
            if std::fs::remove_file(&b.path).is_ok() {
                removed += 1;
            }
        }
        if removed > 0 {
            self.log(format!("rotation des sauvegardes automatiques : {removed} supprimée(s), {keep} conservée(s)"));
            self.refresh_backups();
        }
    }

    /// Recrée la base monde depuis les dumps du package (compléments compris). Personnages et comptes intacts.
    fn reimport_world(&mut self) {
        let st = self.state();
        if st.is_busy() {
            return;
        }
        if st == State::Playing {
            self.notice(fr::WORLD_REIMPORT_PLAYING.into());
            return;
        }
        if st.services_up() {
            self.set(State::Stopping, fr::MSG_STOPPING, 0.3);
            self.stop_world_only();
        }
        self.set(State::Repairing, fr::MSG_WORLD_REIMPORT, 0.2);
        let r = self.ensure_db().and_then(|db| {
            let shared = self.shared.clone();
            let tx = self.log_tx.clone();
            db.import_dumps(&[DB_WORLD], &|step| {
                shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_WORLD_REIMPORT);
                let _ = tx.send(format!("import : {step}"));
            })
        });
        self.drain_server_logs();
        self.release_temp_db();
        match r {
            Ok(()) => {
                self.log("base monde réimportée (personnages et comptes conservés)");
                self.set(State::Installed, fr::MSG_WORLD_REIMPORT_DONE, 0.0);
            }
            Err(e) => self.fail("réimport de la base monde", e),
        }
    }

    // ------------------------------------------------------------------ vérification / réparation

    fn verify(&mut self, full: bool) {
        let prev = self.state();
        let components = self.layout.components_present();
        let info = self.client_info();
        let all = components.iter().all(|(_, ok)| *ok);
        self.log(format!(
            "vérification : composants {}, client {} ({}), manques : {}",
            if all { "OK" } else { "INCOMPLETS" },
            if info.exe_present { info.version_string() } else { "absent".into() },
            info.dir.display(),
            info.missing_mpq.len()
        ));
        {
            let mut s = self.shared.lock().unwrap();
            s.components = components;
            s.client = info;
            s.hermes_installed = HermesProxy::is_installed(&self.layout);
        }
        self.refresh_patches();
        self.refresh_modules();
        self.refresh_mmaps_status();
        // Base déjà en service (lancée par un script ou une autre instance) : on s'y attache sans en prendre la propriété.
        if self.db.is_none() {
            if let Ok(creds) = credentials::load_or_create(&self.layout.credentials_file()) {
                let d = MySql::new(self.layout.clone(), creds);
                if d.ping() {
                    self.db = Some(d);
                    self.db_owned = false;
                    self.db_annex = false;
                    self.shared.lock().unwrap().db_up = true;
                    self.log("MySQL déjà en service : utilisée telle quelle (non gérée par ce launcher)");
                    self.refresh_accounts();
                }
            }
        }
        if full && !prev.is_busy() {
            let restore_state = prev;
            self.set(State::Verifying, fr::MSG_VERIFYING_FILES, 0.1);
            let report = self.build_report();
            self.log(format!("fichiers gérés : {} vérifiés, {} manquants, {} altérés", report.checked, report.missing.len(), report.altered.len()));
            let mut s = self.shared.lock().unwrap();
            s.report = Some(report);
            s.report_fresh = true;
            s.state = restore_state;
            s.message = if restore_state.services_up() { fr::MSG_READY.into() } else { fr::MSG_IDLE.into() };
            s.progress = 0.0;
            return;
        }
        if !prev.services_up() && !prev.is_busy() {
            let mut s = self.shared.lock().unwrap();
            s.state = if all { State::Installed } else { State::NotInstalled };
            s.message = if all { fr::MSG_IDLE.into() } else { fr::MSG_NOT_INSTALLED.into() };
        }
    }

    fn build_report(&mut self) -> VerifyReport {
        let mut report = VerifyReport::default();
        match repair::load_manifest(&self.layout) {
            Some(m) => {
                report.manifest_present = true;
                let shared = self.shared.clone();
                let (missing, altered) = repair::verify_files(&self.layout, &m, &|i, n| {
                    let mut s = shared.lock().unwrap();
                    s.progress = i as f32 / n.max(1) as f32;
                    s.message = format!("{} {i}/{n}", fr::MSG_VERIFYING_FILES);
                });
                report.checked = m.files.len();
                report.missing = missing;
                report.altered = altered;
            }
            None => report.notes.push(fr::RPT_MANIFEST_MISSING.into()),
        }
        report.data = repair::data_status(&self.layout);
        report.db_ok = self.db.as_ref().filter(|d| d.ping()).map(|d| d.databases_missing().map(|m| m.is_empty()).unwrap_or(false) && d.databases_without_schema().map(|m| m.is_empty()).unwrap_or(false));
        if !report.files_ok() {
            report.notes.push(fr::RPT_UNREPAIRABLE.into());
        }
        report
    }

    /// Base disponible pour une opération hors service (sauvegarde, réparation) : démarrée temporairement si besoin.
    fn ensure_db(&mut self) -> Result<MySql, String> {
        if let Some(d) = self.db.as_ref() {
            if d.ping() {
                return Ok(d.clone());
            }
        }
        let creds = credentials::load_or_create(&self.layout.credentials_file())?;
        let d = MySql::new(self.layout.clone(), creds);
        if d.ping() {
            self.log("MySQL déjà en service : utilisée telle quelle");
            self.db = Some(d.clone());
            self.db_owned = false;
            self.db_annex = false;
            return Ok(d);
        }
        d.write_my_ini()?;
        if !d.is_initialized() {
            self.log("première initialisation du datadir MySQL");
            d.initialize()?;
        }
        let started = d.start(Duration::from_secs(120))?;
        self.log(if started { "MySQL démarrée temporairement" } else { "MySQL déjà en service : utilisée telle quelle" });
        self.db = Some(d.clone());
        self.db_owned = started;
        self.db_annex = started;
        Ok(d)
    }

    fn release_temp_db(&mut self) {
        if self.db_annex {
            if let Some(d) = self.db.take() {
                if self.db_owned {
                    match d.shutdown(Duration::from_secs(90)) {
                        Ok(()) => self.log("MySQL temporaire arrêtée"),
                        Err(e) => self.log(format!("MySQL : {e}")),
                    }
                }
            }
            self.db_annex = false;
            self.db_owned = false;
            self.shared.lock().unwrap().db_up = false;
        }
    }

    fn repair(&mut self) {
        let st = self.state();
        if st.is_busy() || st == State::Playing {
            return;
        }
        let services_were_up = st.services_up();
        self.set(State::Repairing, fr::MSG_REPAIR, 0.05);
        let mut notes: Vec<String> = Vec::new();
        let result: Result<(), String> = (|| {
            self.set_message(format!("{} (base de données)", fr::MSG_REPAIR));
            let db = self.ensure_db()?;
            let mut missing: Vec<&str> = db.databases_missing()?;
            for d in db.databases_without_schema()? {
                if d != crate::paths::DB_CHARACTERS {
                    if let Some(k) = ALL_DATABASES.iter().find(|k| **k == d.as_str()) {
                        if !missing.contains(k) {
                            missing.push(k);
                        }
                    }
                }
            }
            if !missing.is_empty() {
                self.log(format!("bases à réimporter : {}", missing.join(", ")));
                let shared = self.shared.clone();
                let tx = self.log_tx.clone();
                db.import_dumps(&missing, &|step| {
                    shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_REPAIR);
                    let _ = tx.send(format!("import : {step}"));
                })?;
                notes.push(format!("bases réimportées : {}", missing.join(", ")));
            }
            self.shared.lock().unwrap().progress = 0.4;
            server::generate_configs(&self.layout, &db, &self.settings)?;
            server::ensure_realm(&db, self.settings.realm_name())?;
            db.post_import_cleanup(&|_| {})?;
            notes.push("configurations régénérées".into());
            let data = repair::data_status(&self.layout);
            let mut steps = Vec::new();
            if !data.iter().any(|(n, ok)| n == "dbc" && *ok) || !data.iter().any(|(n, ok)| n == "maps" && *ok) {
                steps.push(ExtractStep::DbcMaps);
            }
            if !data.iter().any(|(n, ok)| n == "vmaps" && *ok) {
                steps.push(ExtractStep::Vmaps);
            }
            if !steps.is_empty() {
                let info = self.client_info();
                if !info.is_ok() || info.is_classic() {
                    return Err(fr::MSG_CLIENT_335_NEEDED_FOR_DATA.into());
                }
                self.shared.lock().unwrap().progress = 0.5;
                let shared = self.shared.clone();
                repair::extract(&self.layout, &info.dir, &steps, &self.log_tx, &|step| shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_EXTRACT))?;
                notes.push("données du client ré-extraites".into());
            }
            Ok(())
        })();
        self.drain_server_logs();
        if let Err(e) = result {
            self.fail("réparation", e);
            self.release_temp_db();
            return;
        }
        self.shared.lock().unwrap().progress = 0.9;
        let mut report = self.build_report();
        report.notes.extend(notes);
        self.release_temp_db();
        self.log(format!("réparation terminée : {}", report.notes.join(" ; ")));
        let mut s = self.shared.lock().unwrap();
        s.report = Some(report);
        s.report_fresh = true;
        s.state = if services_were_up { State::Ready } else { State::Installed };
        s.message = fr::MSG_REPAIR_DONE.into();
        s.progress = 0.0;
    }

    // ------------------------------------------------------------------ sauvegardes

    fn refresh_backups(&mut self) {
        let list = backup::list(&self.layout);
        self.shared.lock().unwrap().backups = list;
    }

    fn backup(&mut self, label: &str) {
        let st = self.state();
        if st.is_busy() {
            return;
        }
        if st == State::Playing {
            self.log(fr::BK_PLAYING_WARN);
            self.console("saveall");
            std::thread::sleep(Duration::from_secs(3));
        }
        let prev_msg = self.shared.lock().unwrap().message.clone();
        self.set_message(fr::MSG_BACKUP.into());
        let r = self.ensure_db().and_then(|db| {
            let shared = self.shared.clone();
            backup::create(&self.layout, &db, label, &|step| shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_BACKUP))
        });
        self.release_temp_db();
        match r {
            Ok(info) => {
                self.log(format!("sauvegarde créée : {} ({} personnages, {} comptes, {})", info.path.display(), info.manifest.characters, info.manifest.accounts, util::human_size(info.size)));
                self.set_message(format!("{} {}", fr::MSG_BACKUP_DONE, info.manifest.created));
            }
            Err(e) => {
                self.log(format!("ERREUR sauvegarde : {e}"));
                self.notice(format!("Sauvegarde impossible : {e}"));
                self.set_message(prev_msg);
            }
        }
        self.refresh_backups();
    }

    fn restore(&mut self, path: &std::path::Path) {
        let st = self.state();
        if st.is_busy() {
            return;
        }
        if st == State::Playing {
            self.notice(fr::BK_RESTORE_PLAYING.into());
            return;
        }
        if st.services_up() {
            self.set(State::Stopping, fr::MSG_STOPPING, 0.3);
            self.stop_world_only();
        }
        self.set(State::Repairing, fr::MSG_RESTORE, 0.2);
        let r = self.ensure_db().and_then(|db| {
            let shared = self.shared.clone();
            backup::restore(&self.layout, &db, path, &|step| shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_RESTORE))
        });
        self.release_temp_db();
        self.refresh_backups();
        match r {
            Ok(m) => {
                self.log(format!("restauration terminée depuis {} ({} personnages, {} comptes)", path.display(), m.characters, m.accounts));
                self.settings = Settings::load(&self.layout.settings_file());
                self.set(State::Installed, fr::MSG_RESTORE_DONE, 0.0);
            }
            Err(e) => self.fail("restauration", e),
        }
    }

    // ------------------------------------------------------------------ démarrage / arrêt

    /// Enchaînement complet : vérification -> initialisation -> base -> authentification -> monde.
    fn start_all(&mut self) -> bool {
        let st = self.state();
        if st.is_busy() || st.services_up() {
            return st.services_up();
        }
        if self.shared.lock().unwrap().readonly {
            self.fail("démarrage", fr::RO_TITLE.into());
            return false;
        }
        self.set(State::Verifying, fr::MSG_VERIFY, 0.05);
        self.verify(false);
        if self.state() == State::NotInstalled {
            self.fail("vérification", "composants manquants".into());
            return false;
        }

        self.set(State::Initializing, fr::MSG_INIT, 0.15);
        let creds = match credentials::load_or_create(&self.layout.credentials_file()) {
            Ok(c) => c,
            Err(e) => return self.fail_ret("identifiants", e),
        };
        let db = MySql::new(self.layout.clone(), creds);
        if let Err(e) = db.write_my_ini() {
            return self.fail_ret("configuration MySQL", e);
        }
        if !db.is_initialized() {
            self.log("première initialisation du datadir MySQL");
            if let Err(e) = db.initialize() {
                return self.fail_ret("initialisation MySQL", e);
            }
        }

        self.set(State::StartingDb, fr::MSG_DB, 0.3);
        let started = match db.start(Duration::from_secs(120)) {
            Ok(s) => s,
            Err(e) => return self.fail_ret("démarrage MySQL", e),
        };
        self.db_owned = started;
        self.db_annex = false;
        self.log(format!("MySQL opérationnelle sur 127.0.0.1:{DB_PORT}{}", if started { "" } else { " (déjà en service, non gérée par ce launcher)" }));
        match db.databases_missing() {
            Ok(missing) if !missing.is_empty() => {
                self.set(State::StartingDb, fr::MSG_DB_IMPORT, 0.4);
                let shared = self.shared.clone();
                let tx = self.log_tx.clone();
                if let Err(e) = db.import_dumps(&missing, &|step| {
                    shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_DB_IMPORT);
                    let _ = tx.send(format!("import : {step}"));
                }) {
                    self.drain_server_logs();
                    return self.fail_ret("import des bases", e);
                }
                self.drain_server_logs();
                self.log(format!("bases importées : {}", missing.join(", ")));
            }
            Ok(_) => {}
            Err(e) => return self.fail_ret("contrôle des bases", e),
        }
        if let Err(e) = server::generate_configs(&self.layout, &db, &self.settings).and_then(|_| server::ensure_realm(&db, self.settings.realm_name())) {
            return self.fail_ret("configuration serveur", e);
        }
        self.db = Some(db);

        // Première installation : données du client (dbc, maps, vmaps) extraites si absentes. Les mmaps restent optionnelles.
        let data = repair::data_status(&self.layout);
        let need = |n: &str| !data.iter().any(|(k, ok)| k == n && *ok);
        let mut steps = Vec::new();
        if need("dbc") || need("maps") {
            steps.push(ExtractStep::DbcMaps);
        }
        if need("vmaps") {
            steps.push(ExtractStep::Vmaps);
        }
        if !steps.is_empty() {
            let info = self.client_info();
            if !info.is_ok() {
                return self.fail_ret("données du client", fr::MSG_CLIENT_NEEDED.into());
            }
            if info.is_classic() {
                return self.fail_ret("données du client", fr::MSG_CLIENT_335_NEEDED_FOR_DATA.into());
            }
            self.set(State::Initializing, fr::MSG_FIRST_EXTRACT, 0.45);
            let shared = self.shared.clone();
            if let Err(e) = repair::extract(&self.layout, &info.dir, &steps, &self.log_tx, &|step| shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_FIRST_EXTRACT)) {
                self.drain_server_logs();
                return self.fail_ret("extraction", e);
            }
            self.drain_server_logs();
            self.log("données du client extraites");
        }

        self.set(State::StartingAuth, fr::MSG_AUTH, 0.55);
        match AuthServer::start(&self.layout, Duration::from_secs(60)) {
            Ok(r) => self.auth = Some(r),
            Err(e) => return self.fail_ret("démarrage authserver", e),
        }
        self.log(format!("authserver opérationnel sur 127.0.0.1:{AUTH_PORT}"));

        self.set(State::StartingWorld, fr::MSG_WORLD, 0.7);
        let mut m = match WorldServer::start(&self.layout, self.log_tx.clone()) {
            Ok(m) => m,
            Err(e) => return self.fail_ret("démarrage worldserver", e),
        };
        let deadline = std::time::Instant::now() + Duration::from_secs(1200);
        loop {
            self.drain_server_logs();
            match m.wait_ready(Duration::from_millis(500)) {
                Ok(()) => break,
                Err(e) if std::time::Instant::now() > deadline || !m.is_running() => return self.fail_ret("chargement du monde", e),
                Err(_) => {}
            }
        }
        self.world = Some(m);
        self.log(format!("monde opérationnel sur 127.0.0.1:{WORLD_PORT}"));
        {
            let mut s = self.shared.lock().unwrap();
            s.db_up = true;
            s.auth_up = true;
            s.world_up = true;
        }
        self.refresh_accounts();
        if self.mmaps_job.is_none() {
            self.shared.lock().unwrap().mmaps = MmapsStatus::Unknown;
        }
        self.refresh_mmaps_status();
        if self.settings.auto_mmaps {
            self.start_mmaps(false);
        }
        true
    }

    fn fail_ret(&mut self, context: &str, err: String) -> bool {
        self.fail(context, err);
        self.stop_services_quiet();
        false
    }

    fn play(&mut self) {
        let info = self.client_info();
        if !info.is_ok() {
            self.fail("client", format!("client non valide dans {} ({})", info.dir.display(), if info.is_classic() { "WotLK Classic 3.4.3 + Arctium attendus" } else { "3.3.5.12340 attendu" }));
            return;
        }
        if info.is_classic() {
            if !HermesProxy::is_installed(&self.layout) {
                self.fail("HermesProxy", fr::MSG_HERMES_NEEDED.into());
                return;
            }
            let up = self.hermes.as_mut().map(|h| h.is_running()).unwrap_or(false);
            if !up {
                self.set_message(fr::MSG_PROXY.into());
                match HermesProxy::start(&self.layout, &self.settings.hermes_args, Duration::from_secs(30)) {
                    Ok(h) => {
                        self.hermes = Some(h);
                        self.shared.lock().unwrap().hermes_up = true;
                        self.log(format!("HermesProxy opérationnel sur 127.0.0.1:{HERMES_BNET_PORT}"));
                    }
                    Err(e) => {
                        self.fail("HermesProxy", e);
                        return;
                    }
                }
            }
        }
        if let Err(e) = client::configure(&info, self.settings.windowed, self.settings.realm_name()) {
            self.fail("configuration du client", e);
            return;
        }
        match client::launch(&info) {
            Ok(child) => {
                self.log(if info.is_classic() { "jeu lancé via Arctium WoW Launcher (HermesProxy)" } else { "jeu lancé" });
                self.game = Some(child);
                self.set(State::Playing, fr::MSG_PLAYING, 1.0);
            }
            Err(e) => self.fail("lancement du jeu", e),
        }
    }

    /// Arrête worldserver, authserver et HermesProxy, conserve MySQL (restauration, etc.).
    fn stop_world_only(&mut self) {
        if let Some(mut m) = self.world.take() {
            let clean = m.stop(Duration::from_secs(120));
            self.log(if clean { "worldserver arrêté proprement" } else { "worldserver arrêté de force après délai" });
        }
        if let Some(mut r) = self.auth.take() {
            r.stop();
            self.log("authserver arrêté");
        }
        if let Some(mut h) = self.hermes.take() {
            h.stop();
            self.log("HermesProxy arrêté");
        }
        let mut s = self.shared.lock().unwrap();
        s.auth_up = false;
        s.world_up = false;
        s.hermes_up = false;
        s.online = 0;
    }

    fn stop_services_quiet(&mut self) {
        self.stop_world_only();
        if let Some(d) = self.db.take() {
            if self.db_owned {
                match d.shutdown(Duration::from_secs(90)) {
                    Ok(()) => self.log("MySQL arrêtée"),
                    Err(e) => self.log(format!("MySQL : {e}")),
                }
            } else {
                self.log("MySQL laissée en service (non démarrée par ce launcher)");
            }
        }
        self.db_owned = false;
        self.db_annex = false;
        self.shared.lock().unwrap().db_up = false;
    }

    fn stop_all(&mut self) {
        if self.state() == State::Stopping {
            return;
        }
        self.set(State::Stopping, fr::MSG_STOPPING, 0.5);
        if let Some(g) = self.game.as_mut() {
            if matches!(g.try_wait(), Ok(None)) {
                self.log("le jeu est encore ouvert : les services s'arrêtent, le client sera déconnecté");
            }
        }
        self.game = None;
        self.stop_services_quiet();
        self.set(State::Installed, fr::MSG_STOPPED, 0.0);
        self.verify(false);
    }

    // ------------------------------------------------------------------ console, comptes, outils

    fn console(&mut self, command: &str) {
        self.log(format!("console> {command}"));
        match self.world.as_mut() {
            Some(m) => {
                if let Err(e) = m.send(command) {
                    self.log(format!("console : {e}"));
                }
            }
            None => self.log(fr::ACC_NEED_SERVER),
        }
    }

    fn refresh_accounts(&mut self) {
        let Some(db) = self.db.as_ref() else { return };
        let r = db.root_conn().and_then(|mut c| {
            use mysql::prelude::Queryable;
            c.query_map(
                "SELECT a.username, COALESCE(MAX(x.gmlevel), 0) AS gm, COALESCE(DATE_FORMAT(a.last_login,'%Y-%m-%d %H:%i'),'') AS ll, \
                 (SELECT COUNT(*) FROM acore_characters.characters ch WHERE ch.account=a.id) AS nb \
                 FROM acore_auth.account a LEFT JOIN acore_auth.account_access x ON x.id=a.id \
                 WHERE a.username NOT LIKE 'RNDBOT%' GROUP BY a.id ORDER BY a.username",
                |(username, gm, ll, nb): (String, u8, String, u64)| AccountRow { username, gmlevel: gm, last_login: ll, characters: nb },
            )
            .map_err(|e| e.to_string())
        });
        let pool = db.bot_pool();
        match r {
            Ok(list) => {
                let mut s = self.shared.lock().unwrap();
                s.accounts = list;
                s.bot_pool = pool;
            }
            Err(e) => self.log(format!("comptes : {e}")),
        }
    }

    /// Lance Keira3. Il n'accepte pas de paramètres de connexion en ligne de commande : il mémorise la connexion
    /// saisie une fois (« Remember me »). L'interface copie le mot de passe dans le presse-papiers.
    fn open_keira(&mut self) {
        let exe = self.layout.keira_exe();
        if !exe.is_file() {
            self.notice(fr::DB_KEIRA_MISSING.into());
            return;
        }
        let Some(db) = self.db.as_ref() else {
            self.notice(fr::DB_NEED_DB.into());
            return;
        };
        let _ = db;
        match std::process::Command::new(&exe).current_dir(exe.parent().unwrap()).spawn() {
            Ok(_) => {
                self.log("Keira3 ouvert");
                self.notice(fr::DB_KEIRA_OPENED.into());
            }
            Err(e) => self.notice(format!("Keira3 : {e}")),
        }
    }

    fn open_heidi(&mut self) {
        let exe = self.layout.heidisql_exe();
        if !exe.is_file() {
            self.notice(fr::DB_HEIDI_MISSING.into());
            return;
        }
        let Some(db) = self.db.as_ref() else {
            self.notice(fr::DB_NEED_DB.into());
            return;
        };
        let c = db.creds();
        let r = std::process::Command::new(&exe)
            .args([
                "-n=0".to_string(),
                "-h=127.0.0.1".to_string(),
                format!("-P={DB_PORT}"),
                "-u=root".to_string(),
                format!("-p={}", c.root_password),
                format!("-db={}", ALL_DATABASES.join(";")),
            ])
            .current_dir(exe.parent().unwrap())
            .spawn();
        match r {
            Ok(_) => self.log("HeidiSQL ouvert"),
            Err(e) => self.log(format!("HeidiSQL : {e}")),
        }
    }
}

/// Un processus de ce nom tourne-t-il ? (tasklist, sans fenêtre)
fn process_running(image: &str) -> bool {
    std::process::Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {image}"), "/NH"])
        .creation_flags(crate::db::CREATE_NO_WINDOW)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase().contains(&image.to_ascii_lowercase()))
        .unwrap_or(false)
}
