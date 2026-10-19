//! Machine à états et pilotage des services, sur un thread dédié. L'interface n'y accède que par
//! des commandes (canal) et par un instantané partagé (`Snapshot`).

use crate::backup::{self, BackupInfo};
use crate::client::{self, ClientInfo};
use crate::credentials;
use crate::db::{self, MariaDb, QueryResult};
use crate::i18n::fr;
use crate::paths::{Layout, MARIADB_PORT, REALMD_PORT, WORLD_PORT};
use crate::repair::{self, ExtractStep, VerifyReport};
use crate::server::{self, Mangosd, Realmd};
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
    pub realmd_up: bool,
    pub world_up: bool,
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
    pub bot_pool: Option<u64>,
    pub patches: Vec<crate::patches::PatchEntry>,
    pub addons: Vec<crate::addons::AddonEntry>,
    pub table_view: Option<TableView>,
    /// Une écriture a eu lieu : la page affichée doit être rechargée
    pub table_dirty: bool,
    pub mmaps: MmapsStatus,
}

/// État des cartes de déplacement (mmaps), générées en arrière-plan.
#[derive(Clone, Debug, PartialEq)]
pub enum MmapsStatus {
    Unknown,
    Present,
    Absent,
    /// Génération lancée à `since` ; tuiles présentes dans le dossier de travail au lancement et maintenant.
    Running { since: std::time::Instant, start_tiles: usize, tiles: usize },
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
    SettingsChanged(Settings),
    InstallTo(PathBuf),
    KillGame,
    GenerateBots(u32),
    DeleteBots,
    RefreshPatches,
    /// nom logique du patch (ex. patch-3.MPQ), actif ou non
    PatchSetActive(String, bool),
    PatchAdd(PathBuf),
    PatchRemove(String),
    /// Construit un patch-X.MPQ depuis un dossier (arborescence de fichiers)
    PatchBuild(PathBuf),
    RefreshAddons,
    AddonAdd(PathBuf),
    AddonSetActive(String, bool),
    AddonRemove(String),
    /// Recrée la base monde (mangos) depuis le package ; personnages et comptes conservés
    ReimportWorld,
    Shutdown,
}

pub struct Orchestrator {
    layout: Layout,
    settings: Settings,
    shared: Arc<Mutex<Snapshot>>,
    rx: Receiver<Command>,
    log_tx: Sender<String>,
    log_rx: Receiver<String>,
    db: Option<MariaDb>,
    /// Le processus mariadbd a été lancé par ce launcher (sinon : base d'une autre instance ou d'un script, jamais arrêtée ici)
    db_owned: bool,
    /// Base démarrée pour une opération annexe (sauvegarde, réparation…) : arrêtée dès la fin de l'opération si possédée
    db_annex: bool,
    realmd: Option<Realmd>,
    mangosd: Option<Mangosd>,
    game: Option<Child>,
    log_file: Option<std::fs::File>,
    /// MoveMapGenerator en arrière-plan
    mmaps_job: Option<Child>,
    /// Compteur de `tick` : les ports ne sont sondés qu'une fois toutes les 10 secondes (chaque sondage ouvre une
    /// connexion au serveur, journalisée par mangosd) ; entre deux, seul l'état des processus est contrôlé.
    ticks: u64,
    ports_up: (bool, bool),
}

pub fn spawn(layout: Layout, settings: Settings) -> (Arc<Mutex<Snapshot>>, Sender<Command>) {
    let shared = Arc::new(Mutex::new(Snapshot {
        state: State::Installed,
        message: fr::MSG_IDLE.into(),
        error: None,
        progress: 0.0,
        db_up: false,
        realmd_up: false,
        world_up: false,
        bots_enabled: settings.partybots_enabled,
        client: ClientInfo::default(),
        online: 0,
        logs: VecDeque::new(),
        accounts: Vec::new(),
        databases: vec!["characters".into(), "realmd".into(), "mangos".into(), "logs".into()],
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
    }));
    let (tx, rx) = mpsc::channel();
    let (log_tx, log_rx) = mpsc::channel();
    let shared2 = shared.clone();
    std::thread::Builder::new()
        .name("orchestrateur".into())
        .spawn(move || {
            let log_file = std::fs::create_dir_all(layout.logs()).ok().and_then(|_| std::fs::OpenOptions::new().create(true).append(true).open(layout.launcher_log()).ok());
            let mut o = Orchestrator { layout, settings, shared: shared2, rx, log_tx, log_rx, db: None, db_owned: false, db_annex: false, realmd: None, mangosd: None, game: None, log_file, mmaps_job: None, ticks: 0, ports_up: (false, false) };
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
                        self.shared.lock().unwrap().notice = Some(format!("{}{}", fr::MSG_DIAG_DONE, p.display()));
                    }
                    Err(e) => self.log(format!("diagnostic : {e}")),
                }
            }
            Command::Console(c) => self.console(&c),
            Command::RefreshAccounts => self.refresh_accounts(),
            Command::CreateAccount { user, pass, gm } => {
                self.console(&format!("account create {user} {pass}"));
                std::thread::sleep(Duration::from_millis(400));
                self.console(&format!("account set gmlevel {user} {gm}"));
                std::thread::sleep(Duration::from_millis(400));
                self.refresh_accounts();
            }
            Command::SetPassword { user, pass } => self.console(&format!("account set password {user} {pass} {pass}")),
            Command::DeleteAccount(user) => {
                if self.state() == State::Playing {
                    self.shared.lock().unwrap().notice = Some(fr::ACC_DELETE_PLAYING.into());
                    return;
                }
                if self.mangosd.is_some() {
                    // Le serveur supprime lui-même le compte et ses personnages (commande console SEC_CONSOLE).
                    self.console(&format!("account delete {user}"));
                    std::thread::sleep(Duration::from_millis(800));
                    self.log(format!("compte {user} : suppression demandée au serveur"));
                } else {
                    let was_temp = self.db.is_none();
                    let r = self.ensure_db().and_then(|db| db.delete_account_offline(&user));
                    match r {
                        Ok(n) => {
                            self.log(format!("compte {user} supprimé ({n} personnage(s))"));
                            self.shared.lock().unwrap().notice = Some(format!("{} {user} ({n} {})", fr::ACC_DELETED, fr::BK_CHARS.to_lowercase()));
                        }
                        Err(e) => self.shared.lock().unwrap().notice = Some(format!("{} : {e}", fr::ACC_DELETE_FAIL)),
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
                self.console(&format!("account set gmlevel {user} {gm}"));
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
            Command::SettingsChanged(s) => {
                self.settings = s;
                self.shared.lock().unwrap().bots_enabled = self.settings.partybots_enabled;
                self.verify(false);
                self.apply_settings_live();
            }
            Command::InstallTo(dest) => self.install_to(&dest),
            Command::GenerateBots(n) => self.generate_bots(n),
            Command::RefreshPatches => self.refresh_patches(),
            Command::PatchBuild(dir) => {
                let st = self.state();
                if st == State::Playing || st.is_busy() {
                    self.shared.lock().unwrap().notice = Some(fr::PATCH_PLAYING.into());
                    return;
                }
                let client = self.client_dir();
                let Some(name) = crate::patches::next_free_name(&client) else {
                    self.shared.lock().unwrap().notice = Some(fr::PATCH_ADD_FAIL.into());
                    return;
                };
                let out = client.join("Data").join(&name);
                self.set(State::Repairing, fr::PATCH_BUILDING, 0.0);
                let shared = self.shared.clone();
                let r = crate::mpqbuild::build_from_dir(&dir, &out, &|i, n| {
                    let mut s = shared.lock().unwrap();
                    s.progress = i as f32 / n.max(1) as f32;
                    s.message = format!("{} {i}/{n}", fr::PATCH_BUILDING);
                });
                let check = r.as_ref().ok().and_then(|_| crate::mpqbuild::inspect(&out).ok());
                let mut s = self.shared.lock().unwrap();
                s.state = if st.services_up() { State::Ready } else { State::Installed };
                s.progress = 0.0;
                s.message = if st.services_up() { fr::MSG_READY.into() } else { fr::MSG_IDLE.into() };
                drop(s);
                match (r, check) {
                    (Ok(n), Some((listed, size))) => {
                        self.log(format!("patch construit : {name} ({n} fichiers, {listed} listés, {})", util::human_size(size)));
                        self.shared.lock().unwrap().notice = Some(format!("{} {name} ({n} fichiers, {})", fr::PATCH_BUILT, util::human_size(size)));
                    }
                    (Ok(n), None) => {
                        self.log(format!("patch construit : {name} ({n} fichiers) mais relecture impossible"));
                        self.shared.lock().unwrap().notice = Some(format!("{} {name}", fr::PATCH_BUILT));
                    }
                    (Err(e), _) => {
                        self.log(format!("ERREUR construction du patch : {e}"));
                        self.shared.lock().unwrap().notice = Some(format!("{} : {e}", fr::PATCH_ADD_FAIL));
                    }
                }
                self.refresh_patches();
            }
            Command::RefreshAddons => self.refresh_addons(),
            Command::AddonAdd(src) => {
                if self.state() == State::Playing {
                    self.shared.lock().unwrap().notice = Some(fr::ADDON_PLAYING.into());
                    return;
                }
                let prev = self.shared.lock().unwrap().message.clone();
                let shared = self.shared.clone();
                match crate::addons::add_from_zip(&src, &self.client_dir(), &|m| shared.lock().unwrap().message = m.to_string()) {
                    Ok(names) => {
                        self.log(format!("AddOns installés : {} (depuis {})", names.join(", "), src.display()));
                        self.shared.lock().unwrap().notice = Some(format!("{} {}", fr::ADDON_ADDED, names.join(", ")));
                    }
                    Err(e) => self.shared.lock().unwrap().notice = Some(format!("{} : {e}", fr::ADDON_ADD_FAIL)),
                }
                self.shared.lock().unwrap().message = prev;
                self.refresh_addons();
            }
            Command::AddonSetActive(name, active) => {
                if self.state() == State::Playing {
                    self.shared.lock().unwrap().notice = Some(fr::ADDON_PLAYING.into());
                    return;
                }
                let entries = crate::addons::list(&self.client_dir());
                if let Some(e) = entries.iter().find(|a| a.name.eq_ignore_ascii_case(&name)) {
                    match crate::addons::set_active(e, active) {
                        Ok(()) => self.log(format!("AddOn {} {}", e.name, if active { "activé" } else { "désactivé" })),
                        Err(err) => self.shared.lock().unwrap().notice = Some(err),
                    }
                }
                self.refresh_addons();
            }
            Command::AddonRemove(name) => {
                if self.state() == State::Playing {
                    self.shared.lock().unwrap().notice = Some(fr::ADDON_PLAYING.into());
                    return;
                }
                let entries = crate::addons::list(&self.client_dir());
                if let Some(e) = entries.iter().find(|a| a.name.eq_ignore_ascii_case(&name)) {
                    match crate::addons::remove(e) {
                        Ok(()) => self.log(format!("AddOn supprimé : {}", e.name)),
                        Err(err) => self.shared.lock().unwrap().notice = Some(err),
                    }
                }
                self.refresh_addons();
            }
            Command::PatchSetActive(name, active) => {
                if self.state() == State::Playing {
                    self.shared.lock().unwrap().notice = Some(fr::PATCH_PLAYING.into());
                    return;
                }
                let entries = crate::patches::list(&self.client_dir());
                match entries.iter().find(|p| p.name.eq_ignore_ascii_case(&name)) {
                    Some(e) => match crate::patches::set_active(e, active) {
                        Ok(_) => self.log(format!("patch {} {}", e.name, if active { "activé" } else { "désactivé" })),
                        Err(err) => self.shared.lock().unwrap().notice = Some(format!("{} : {err}", e.name)),
                    },
                    None => self.shared.lock().unwrap().notice = Some(format!("{name} : introuvable")),
                }
                self.refresh_patches();
            }
            Command::PatchAdd(src) => {
                if self.state() == State::Playing {
                    self.shared.lock().unwrap().notice = Some(fr::PATCH_PLAYING.into());
                    return;
                }
                let prev = self.shared.lock().unwrap().message.clone();
                let shared = self.shared.clone();
                match crate::patches::add(&src, &self.client_dir(), &|m| shared.lock().unwrap().message = m.to_string()) {
                    Ok(name) => {
                        self.log(format!("patch ajouté : {name} (depuis {})", src.display()));
                        self.shared.lock().unwrap().notice = Some(format!("{} {name}", fr::PATCH_ADDED));
                    }
                    Err(e) => self.shared.lock().unwrap().notice = Some(format!("{} : {e}", fr::PATCH_ADD_FAIL)),
                }
                self.shared.lock().unwrap().message = prev;
                self.refresh_patches();
            }
            Command::PatchRemove(name) => {
                if self.state() == State::Playing {
                    self.shared.lock().unwrap().notice = Some(fr::PATCH_PLAYING.into());
                    return;
                }
                let entries = crate::patches::list(&self.client_dir());
                if let Some(e) = entries.iter().find(|p| p.name.eq_ignore_ascii_case(&name)) {
                    match crate::patches::remove(e) {
                        Ok(()) => self.log(format!("patch supprimé : {}", e.name)),
                        Err(err) => self.shared.lock().unwrap().notice = Some(err),
                    }
                }
                self.refresh_patches();
            }
            Command::DeleteBots => {
                let was_temp = self.db.is_none();
                let r = self.ensure_db().and_then(|db| crate::botgen::delete_all(&db));
                match r {
                    Ok(k) => {
                        self.log(format!("{k} personnage(s) bot(s) supprimé(s)"));
                        self.shared.lock().unwrap().notice = Some(format!("{k} personnages bots supprimés."));
                        if self.mangosd.is_some() {
                            self.console("bot reload");
                        }
                    }
                    Err(e) => self.log(format!("ERREUR suppression des bots : {e}")),
                }
                self.refresh_accounts();
                if was_temp {
                    self.release_temp_db();
                }
            }
            Command::KillGame => {
                if let Some(mut g) = self.game.take() {
                    let _ = g.kill();
                    let _ = g.wait();
                    self.log("jeu fermé de force à la demande de l'utilisateur");
                }
                // Instances de WoW.exe lancées hors launcher (ou via VanillaFixes) : fermées aussi si elles viennent du dossier client.
                let dir = self.client_dir();
                let _ = std::process::Command::new("taskkill").args(["/F", "/IM", "WoW.exe"]).creation_flags(crate::db::CREATE_NO_WINDOW).output();
                self.log(format!("WoW.exe terminé (client : {})", dir.display()));
                if self.state() == State::Playing {
                    self.set(State::Ready, fr::MSG_READY, 1.0);
                }
            }
            Command::Shutdown => {}
        }
    }

    fn refresh_patches(&mut self) {
        let list = crate::patches::list(&self.client_dir());
        self.shared.lock().unwrap().patches = list;
        self.refresh_addons();
    }

    fn refresh_addons(&mut self) {
        let list = crate::addons::list(&self.client_dir());
        self.shared.lock().unwrap().addons = list;
    }

    /// Génère des personnages bots de peuplement ; recharge la liste du serveur s'il tourne.
    fn generate_bots(&mut self, count: u32) {
        if self.state().is_busy() {
            return;
        }
        let was_temp = self.db.is_none();
        let prev_msg = self.shared.lock().unwrap().message.clone();
        let shared = self.shared.clone();
        let r = self.ensure_db().and_then(|db| {
            // Installation neuve : les bases ne sont importées qu'au premier démarrage du serveur.
            if !db.databases_missing()?.is_empty() {
                return Err(fr::BOTGEN_NEED_FIRST_START.to_string());
            }
            crate::botgen::generate(&db, count, &|i, n| shared.lock().unwrap().message = format!("{} {i}/{n}", fr::BOTGEN_PROGRESS))
        });
        match r {
            Ok(rep) => {
                self.log(format!("{} personnages bots créés ({} Alliance, {} Horde)", rep.created, rep.alliance, rep.horde));
                let mut s = self.shared.lock().unwrap();
                s.notice = Some(format!("{} {} ({} Alliance, {} Horde).", rep.created, fr::BOTGEN_DONE, rep.alliance, rep.horde));
                s.message = prev_msg;
                drop(s);
                if self.mangosd.is_some() {
                    self.console("bot reload");
                }
            }
            Err(e) => {
                self.log(format!("ERREUR génération des bots : {e}"));
                let mut s = self.shared.lock().unwrap();
                s.notice = Some(format!("{} : {e}", fr::BOTGEN_FAIL));
                s.message = prev_msg;
            }
        }
        self.refresh_accounts();
        if was_temp {
            self.release_temp_db();
        }
    }

    /// Installation depuis un support en lecture seule : copie intégrale vers `dest`, puis lancement du launcher copié.
    fn install_to(&mut self, dest: &std::path::Path) {
        self.set(State::Repairing, fr::MSG_INSTALLING, 0.0);
        let shared = self.shared.clone();
        let r = std::fs::create_dir_all(dest).map_err(|e| e.to_string()).and_then(|_| {
            util::copy_tree(&self.layout.root, dest, &["UserData", "Logs", "target"], &|i, n| {
                let mut s = shared.lock().unwrap();
                s.progress = i as f32 / n.max(1) as f32;
                s.message = format!("{} {i}/{n}", fr::MSG_INSTALLING);
            })
        });
        match r {
            Ok(()) => {
                self.log(format!("copié vers {}", dest.display()));
                let exe_name = std::env::current_exe().ok().and_then(|p| p.file_name().map(|n| n.to_os_string())).unwrap_or_else(|| "vanilla-solo-launcher.exe".into());
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
        let realmd_alive = self.realmd.as_mut().map(|r| r.is_running()).unwrap_or(false);
        let world_alive = self.mangosd.as_mut().map(|m| m.is_running()).unwrap_or(false);
        // Sondage des ports toutes les 10 s (ou dès qu'un processus redémarre) ; un processus arrêté est vu à la seconde.
        if self.ticks % 10 == 0 || (realmd_alive && !self.ports_up.0) || (world_alive && !self.ports_up.1) {
            self.ports_up = (realmd_alive && db::port_open(REALMD_PORT), world_alive && db::port_open(WORLD_PORT));
        }
        if self.ticks % 10 == 0 && self.mmaps_job.is_some() {
            let n = repair::mmaps_staged_tiles(&self.layout);
            if let MmapsStatus::Running { tiles, .. } = &mut self.shared.lock().unwrap().mmaps {
                *tiles = n;
            }
        }
        self.ticks += 1;
        let realmd_up = realmd_alive && self.ports_up.0;
        let world_up = world_alive && self.ports_up.1;
        let online = if db_up && world_up { self.db.as_ref().map(|d| d.online_count()).unwrap_or(0) } else { 0 };
        let state = self.state();
        {
            let mut s = self.shared.lock().unwrap();
            s.db_up = db_up;
            s.realmd_up = realmd_up;
            s.world_up = world_up;
            s.online = online;
        }
        if state.services_up() && !(db_up && realmd_up && world_up) {
            let which = if !db_up { "base de données" } else if !realmd_up { "serveur d'authentification" } else { "serveur de jeu" };
            self.fail("service interrompu", format!("{which} ne répond plus"));
            self.stop_services_quiet();
            return;
        }
        if state == State::Playing {
            if let Some(g) = self.game.as_mut() {
                if let Ok(Some(st)) = g.try_wait() {
                    self.log(format!("jeu terminé (code {:?})", st.code()));
                    self.game = None;
                    self.set(State::Ready, fr::MSG_READY, 1.0);
                    if self.settings.auto_backup {
                        self.backup(fr::BK_AUTO_LABEL);
                        self.rotate_auto_backups();
                        self.set_message(fr::MSG_READY.into());
                    }
                }
            }
        }
        self.poll_mmaps();
    }

    /// Serveur en marche : réglages appliqués sans redémarrage. `mangosd.conf` est réécrit, puis `reload config`
    /// (VMaNGOS relit le fichier) et `bot reload` (le gestionnaire de bots relit ses réglages et connecte aussitôt le
    /// nombre minimal de bots de peuplement). Sans cela, `bot reload` relisait l'ancienne configuration en mémoire et
    /// rien ne changeait avant le redémarrage suivant.
    fn apply_settings_live(&mut self) {
        if self.mangosd.is_none() || !self.state().services_up() {
            return;
        }
        let Some(db) = self.db.clone() else { return };
        match server::generate_configs(&self.layout, &db, &self.settings) {
            Ok(()) => {
                self.console("reload config");
                std::thread::sleep(Duration::from_millis(800));
                self.console("bot reload");
                self.log("réglages appliqués au serveur en marche (reload config, bot reload)");
            }
            Err(e) => self.log(format!("réglages : application à chaud impossible ({e}) ; ils seront pris en compte au prochain démarrage")),
        }
    }

    // ------------------------------------------------------------------ mmaps en arrière-plan

    /// Lance MoveMapGenerator en arrière-plan si les mmaps manquent (ou toujours si `force`).
    fn start_mmaps(&mut self, force: bool) {
        if self.mmaps_job.is_some() {
            if force {
                self.shared.lock().unwrap().notice = Some(fr::MMAPS_ALREADY_RUNNING.into());
            }
            return;
        }
        if repair::mmaps_pending(&self.layout) {
            // Déjà générées, installées au prochain démarrage du serveur.
            self.shared.lock().unwrap().mmaps = MmapsStatus::DoneRestartNeeded;
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
        // La moitié des cœurs : le serveur et le jeu doivent rester fluides pendant la génération.
        let threads = (std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4) / 2).max(1);
        match repair::spawn_mmaps(&self.layout, threads, &self.log_tx) {
            Ok(child) => {
                self.log(format!("mmaps {} : génération en arrière-plan ({threads} threads)", if present { "régénérées" } else { "absentes" }));
                self.mmaps_job = Some(child);
                self.shared.lock().unwrap().mmaps = { let t = repair::mmaps_staged_tiles(&self.layout); MmapsStatus::Running { since: std::time::Instant::now(), start_tiles: t, tiles: t } };
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
                let pending = repair::mmaps_pending(&self.layout);
                let mut s = self.shared.lock().unwrap();
                if ok && pending {
                    s.mmaps = MmapsStatus::DoneRestartNeeded;
                    drop(s);
                    if self.mangosd.is_none() {
                        self.install_mmaps();
                        self.refresh_mmaps_status();
                    } else {
                        self.log("mmaps générées en arrière-plan : installées au prochain démarrage du serveur");
                    }
                } else {
                    let e = format!("MoveMapGenerator terminé avec le code {:?}", status.code());
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

    /// Fermeture du launcher : la génération est interrompue, le marqueur reste et elle reprendra au prochain démarrage.
    fn abort_mmaps(&mut self) {
        if let Some(mut job) = self.mmaps_job.take() {
            let _ = job.kill();
            let _ = job.wait();
            self.log("génération des mmaps interrompue par la fermeture du launcher : reprendra au prochain démarrage du serveur");
        }
    }

    /// Serveur arrêté uniquement : installe les mmaps générées (voir `repair::install_pending_mmaps`).
    fn install_mmaps(&mut self) {
        match repair::install_pending_mmaps(&self.layout) {
            Ok(true) => {
                self.log("mmaps générées installées");
                self.shared.lock().unwrap().mmaps = MmapsStatus::Unknown;
            }
            Ok(false) => {}
            Err(e) => self.log(format!("ERREUR mmaps : {e}")),
        }
    }

    fn refresh_mmaps_status(&mut self) {
        if self.mmaps_job.is_some() {
            return;
        }
        if repair::mmaps_pending(&self.layout) {
            self.shared.lock().unwrap().mmaps = MmapsStatus::DoneRestartNeeded;
            return;
        }
        let present = repair::data_status(&self.layout).iter().any(|(n, ok)| n == "mmaps" && *ok);
        let mut s = self.shared.lock().unwrap();
        if !matches!(s.mmaps, MmapsStatus::DoneRestartNeeded | MmapsStatus::Failed(_)) {
            s.mmaps = if present { MmapsStatus::Present } else { MmapsStatus::Absent };
        }
    }

    /// Ne conserve que les `auto_backup_keep` sauvegardes automatiques les plus récentes.
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
            self.shared.lock().unwrap().notice = Some(fr::WORLD_REIMPORT_PLAYING.into());
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
            db.import_dumps(&["mangos"], &|step| {
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

    /// Vérification : composants, client, et (si `full`) fichiers gérés par empreinte + données + bases.
    fn verify(&mut self, full: bool) {
        let prev = self.state();
        let components = self.layout.components_present();
        let info = client::inspect(&self.client_dir());
        let all = components.iter().all(|(_, ok)| *ok);
        self.log(format!(
            "vérification : composants {}, client {} ({}), MPQ manquants : {}",
            if all { "OK" } else { "INCOMPLETS" },
            if info.exe_present { format!("{} {}", info.kind_label(), info.version_string()) } else { "absent".into() },
            info.dir.display(),
            info.missing_mpq.len()
        ));
        {
            let mut s = self.shared.lock().unwrap();
            s.components = components;
            s.client = info;
        }
        self.refresh_patches();
        self.refresh_mmaps_status();
        // Base déjà en service (lancée par un script ou une autre instance) : on s'y attache pour l'explorateur
        // et les comptes, sans en prendre la propriété.
        if self.db.is_none() {
            if let Ok(creds) = credentials::load_or_create(&self.layout.credentials_file()) {
                let d = MariaDb::new(self.layout.clone(), creds);
                if d.ping() {
                    self.db = Some(d);
                    self.db_owned = false;
                    self.db_annex = false;
                    self.shared.lock().unwrap().db_up = true;
                    self.log("MariaDB déjà en service : utilisée telle quelle (non gérée par ce launcher)");
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
        report.db_ok = self.db.as_ref().filter(|d| d.ping()).map(|d| d.databases_missing().map(|m| m.is_empty()).unwrap_or(false) && d.databases_without_migrations().map(|m| m.is_empty()).unwrap_or(false));
        if !report.files_ok() {
            report.notes.push(fr::RPT_UNREPAIRABLE.into());
        }
        report
    }

    /// Base disponible pour une opération hors service (sauvegarde, réparation) : démarrée temporairement si besoin.
    fn ensure_db(&mut self) -> Result<MariaDb, String> {
        if let Some(d) = self.db.as_ref() {
            if d.ping() {
                return Ok(d.clone());
            }
        }
        let creds = credentials::load_or_create(&self.layout.credentials_file())?;
        let d = MariaDb::new(self.layout.clone(), creds);
        // Base déjà en service (autre instance du launcher, script Tools) : on l'utilise sans jamais l'arrêter ensuite.
        if d.ping() {
            self.log("MariaDB déjà en service : utilisée telle quelle");
            self.db = Some(d.clone());
            self.db_owned = false;
            self.db_annex = false;
            return Ok(d);
        }
        d.write_my_ini()?;
        if !d.is_initialized() {
            self.log("première initialisation du datadir MariaDB");
            d.initialize()?;
        }
        let started = d.start(Duration::from_secs(90))?;
        self.log(if started { "MariaDB démarrée temporairement" } else { "MariaDB déjà en service : utilisée telle quelle" });
        self.db = Some(d.clone());
        self.db_owned = started;
        self.db_annex = started;
        Ok(d)
    }

    fn release_temp_db(&mut self) {
        if self.db_annex {
            if let Some(d) = self.db.take() {
                if self.db_owned {
                    match d.shutdown(Duration::from_secs(60)) {
                        Ok(()) => self.log("MariaDB temporaire arrêtée"),
                        Err(e) => self.log(format!("MariaDB : {e}")),
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
            // 1. Base : bases manquantes réimportées (jamais characters si elle contient des personnages).
            self.set_message(format!("{} (base de données)", fr::MSG_REPAIR));
            let db = self.ensure_db()?;
            let mut missing = db.databases_missing()?;
            for d in db.databases_without_migrations()? {
                if d != "characters" && !missing.iter().any(|m| *m == d.as_str()) {
                    missing.push(match d.as_str() { "realmd" => "realmd", "mangos" => "mangos", "logs" => "logs", _ => continue });
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
            // 2. Configurations gérées régénérées.
            self.shared.lock().unwrap().progress = 0.4;
            server::generate_configs(&self.layout, &db, &self.settings)?;
            server::ensure_realm(&db)?;
            notes.push("configurations régénérées".into());
            // 3. Données du client.
            let data = repair::data_status(&self.layout);
            let mut steps = Vec::new();
            if !data.iter().any(|(n, ok)| n == "dbc" && *ok) || !data.iter().any(|(n, ok)| n == "maps" && *ok) {
                steps.push(ExtractStep::DbcMaps);
            }
            if !data.iter().any(|(n, ok)| n == "vmaps" && *ok) {
                steps.push(ExtractStep::Vmaps);
            }
            if !steps.is_empty() {
                let info = client::inspect(&self.client_dir());
                if !info.is_ok() {
                    return Err(format!("données du client absentes et client non valide dans {}", info.dir.display()));
                }
                if let Some(msg) = client::ensure_benilla_base_mpq(&info)? {
                    self.log(msg);
                }
                let source = client::extraction_source(&info, &self.layout.root)?;
                if source != info.dir {
                    self.log(format!("extraction : archives lues dans {} (il manque {} au client choisi)", source.display(), client::extraction_missing(&info.dir).join(", ")));
                }
                self.shared.lock().unwrap().progress = 0.5;
                let shared = self.shared.clone();
                repair::extract(&self.layout, &source, &steps, &self.log_tx, &|step| shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_EXTRACT))?;
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
                self.shared.lock().unwrap().notice = Some(format!("Sauvegarde impossible : {e}"));
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
            self.shared.lock().unwrap().notice = Some(fr::BK_RESTORE_PLAYING.into());
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
        let db = MariaDb::new(self.layout.clone(), creds);
        if let Err(e) = db.write_my_ini() {
            return self.fail_ret("configuration MariaDB", e);
        }
        if !db.is_initialized() {
            self.log("première initialisation du datadir MariaDB");
            if let Err(e) = db.initialize() {
                return self.fail_ret("initialisation MariaDB", e);
            }
        }

        self.set(State::StartingDb, fr::MSG_DB, 0.3);
        let started = match db.start(Duration::from_secs(90)) {
            Ok(s) => s,
            Err(e) => return self.fail_ret("démarrage MariaDB", e),
        };
        self.db_owned = started;
        self.db_annex = false;
        self.log(format!("MariaDB opérationnelle sur 127.0.0.1:{MARIADB_PORT}{}", if started { "" } else { " (déjà en service, non gérée par ce launcher)" }));
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
        if let Err(e) = server::generate_configs(&self.layout, &db, &self.settings).and_then(|_| server::ensure_realm(&db)) {
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
            let info = client::inspect(&self.client_dir());
            if !info.is_ok() {
                return self.fail_ret("données du client", fr::MSG_CLIENT_NEEDED.into());
            }
            match client::ensure_benilla_base_mpq(&info) {
                Ok(Some(msg)) => self.log(msg),
                Ok(None) => {}
                Err(e) => return self.fail_ret("données du client", e),
            }
            let source = match client::extraction_source(&info, &self.layout.root) {
                Ok(s) => s,
                Err(e) => return self.fail_ret("données du client", e),
            };
            if source != info.dir {
                self.log(format!("extraction : archives lues dans {} (il manque {} au client choisi)", source.display(), client::extraction_missing(&info.dir).join(", ")));
            }
            self.set(State::Initializing, fr::MSG_FIRST_EXTRACT, 0.45);
            let shared = self.shared.clone();
            if let Err(e) = repair::extract(&self.layout, &source, &steps, &self.log_tx, &|step| shared.lock().unwrap().message = format!("{} ({step})", fr::MSG_FIRST_EXTRACT)) {
                self.drain_server_logs();
                return self.fail_ret("extraction", e);
            }
            self.drain_server_logs();
            self.log("données du client extraites");
        }

        self.set(State::StartingAuth, fr::MSG_AUTH, 0.55);
        match Realmd::start(&self.layout, Duration::from_secs(30)) {
            Ok(r) => self.realmd = Some(r),
            Err(e) => return self.fail_ret("démarrage realmd", e),
        }
        self.log(format!("realmd opérationnel sur 127.0.0.1:{REALMD_PORT}"));

        self.set(State::StartingWorld, fr::MSG_WORLD, 0.7);
        // Serveur arrêté : seul moment où des mmaps neuves peuvent entrer sans être chargées à moitié.
        if self.mmaps_job.is_none() {
            self.install_mmaps();
        }
        let mut m = match Mangosd::start(&self.layout, self.log_tx.clone()) {
            Ok(m) => m,
            Err(e) => return self.fail_ret("démarrage mangosd", e),
        };
        let deadline = std::time::Instant::now() + Duration::from_secs(600);
        loop {
            self.drain_server_logs();
            match m.wait_ready(Duration::from_millis(500)) {
                Ok(()) => break,
                Err(e) if std::time::Instant::now() > deadline || !m.is_running() => return self.fail_ret("chargement du monde", e),
                Err(_) => {}
            }
        }
        self.mangosd = Some(m);
        self.log(format!("monde opérationnel sur 127.0.0.1:{WORLD_PORT}"));
        {
            let mut s = self.shared.lock().unwrap();
            s.db_up = true;
            s.realmd_up = true;
            s.world_up = true;
        }
        self.refresh_accounts();
        // Le serveur vient de charger les mmaps présentes : l'indicateur « au prochain démarrage » n'a plus lieu d'être.
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
        let dir = self.client_dir();
        let info = client::inspect(&dir);
        if !info.is_ok() {
            self.fail("client", format!("client non valide dans {} (attendu : WoW.exe 1.12.1.5875 ou client Benilla, avec les archives de Data)", dir.display()));
            return;
        }
        if let Err(e) = client::configure(&info, self.settings.windowed) {
            self.fail("configuration du client", e);
            return;
        }
        // VanillaFixes et DXVK ne concernent que WoW.exe (32 bits, Direct3D 9) : Benilla a son propre rendu (wgpu).
        let via_vf = if info.is_benilla() {
            false
        } else {
            match client::apply_vanillafixes(&dir, &self.layout.vanillafixes_dir(), self.settings.vanillafixes, self.settings.dxvk) {
                Ok(v) => v,
                Err(e) => {
                    self.log(format!("VanillaFixes/DXVK : {e} — lancement direct de WoW.exe"));
                    false
                }
            }
        };
        match client::launch(&info, via_vf) {
            Ok(child) => {
                self.log(if via_vf { "jeu lancé via VanillaFixes".to_string() } else { format!("jeu lancé : {} ({})", info.exe_name, info.kind_label()) });
                self.game = Some(child);
                self.set(State::Playing, fr::MSG_PLAYING, 1.0);
            }
            Err(e) => self.fail("lancement du jeu", e),
        }
    }

    /// Arrête mangosd et realmd, conserve MariaDB (restauration, etc.).
    fn stop_world_only(&mut self) {
        if let Some(mut m) = self.mangosd.take() {
            let clean = m.stop(Duration::from_secs(60));
            self.log(if clean { "mangosd arrêté proprement" } else { "mangosd arrêté de force après délai" });
        }
        if let Some(mut r) = self.realmd.take() {
            r.stop();
            self.log("realmd arrêté");
        }
        let mut s = self.shared.lock().unwrap();
        s.realmd_up = false;
        s.world_up = false;
        s.online = 0;
    }

    fn stop_services_quiet(&mut self) {
        self.stop_world_only();
        if let Some(d) = self.db.take() {
            if self.db_owned {
                match d.shutdown(Duration::from_secs(60)) {
                    Ok(()) => self.log("MariaDB arrêtée"),
                    Err(e) => self.log(format!("MariaDB : {e}")),
                }
            } else {
                self.log("MariaDB laissée en service (non démarrée par ce launcher)");
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
        match self.mangosd.as_mut() {
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
                "SELECT a.username, COALESCE(MAX(aa.gmlevel), a.gmlevel) AS gm, COALESCE(DATE_FORMAT(a.last_login,'%Y-%m-%d %H:%i'),'') AS ll, \
                 (SELECT COUNT(*) FROM characters.characters ch WHERE ch.account=a.id) AS nb \
                 FROM realmd.account a LEFT JOIN realmd.account_access aa ON aa.id=a.id GROUP BY a.id ORDER BY a.username",
                |(username, gm, ll, nb): (String, u8, String, u64)| AccountRow { username, gmlevel: gm, last_login: ll, characters: nb },
            )
            .map_err(|e| e.to_string())
        });
        let pool = db.bot_pool_count();
        match r {
            Ok(list) => {
                let mut s = self.shared.lock().unwrap();
                s.accounts = list;
                s.bot_pool = pool;
            }
            Err(e) => self.log(format!("comptes : {e}")),
        }
    }

    fn open_heidi(&mut self) {
        let exe = self.layout.heidisql_exe();
        if !exe.is_file() {
            self.shared.lock().unwrap().notice = Some(fr::DB_HEIDI_MISSING.into());
            return;
        }
        let Some(db) = self.db.as_ref() else {
            self.shared.lock().unwrap().notice = Some(fr::DB_NEED_DB.into());
            return;
        };
        let c = db.creds();
        // Session non enregistrée : pas de -d (sinon HeidiSQL cherche une session sauvegardée de ce nom et ouvre son
        // gestionnaire), adresse sans guillemets (std::process les échapperait et HeidiSQL ne les comprendrait pas).
        let r = std::process::Command::new(&exe)
            .args([
                "-n=0".to_string(),
                "-h=127.0.0.1".to_string(),
                format!("-P={MARIADB_PORT}"),
                format!("-u={}", c.user),
                format!("-p={}", c.password),
                "-db=characters;realmd;mangos;logs".to_string(),
            ])
            .current_dir(exe.parent().unwrap())
            .spawn();
        match r {
            Ok(_) => self.log("HeidiSQL ouvert"),
            Err(e) => self.log(format!("HeidiSQL : {e}")),
        }
    }
}
