//! Interface egui : écran principal, onglets Comptes / Base de données / Sauvegardes / Paramètres / Journaux,
//! fenêtres de confirmation et rapport de vérification.

use crate::conf_catalog;
use crate::i18n::fr;
use crate::orchestrator::{Command, MmapsStatus, Snapshot, State};
use crate::paths::Layout;
use crate::settings::{ClientKind, Settings};
use crate::util;
use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Home,
    Accounts,
    Bots,
    Modules,
    World,
    Patches,
    Database,
    Backups,
    Settings,
    Logs,
}

enum Pending {
    Restore(PathBuf, String),
    Delete(PathBuf, String),
    DeleteBots,
    ResetModule(String, String),
    RemovePatch(String),
    RemoveAddon(String),
    DeleteAccount(String, String),
    /// (base, sql, titre, texte)
    ExecSql(String, String, &'static str, &'static str),
    ReimportWorld,
}

pub struct App {
    layout: Layout,
    shared: Arc<Mutex<Snapshot>>,
    tx: Sender<Command>,
    tab: Tab,
    settings: Settings,
    settings_notice: Option<String>,
    closing: bool,
    quit_at: Option<std::time::Instant>,
    /// Visite de captures (--screenshots) : dossier, index de l'onglet, échéance de la prochaine étape, capture demandée
    shots: Option<(PathBuf, usize, std::time::Instant, bool)>,
    /// Délai d'attente des bots déjà accordé (visite avec --start)
    shots_settled: bool,
    pending: Option<Pending>,
    show_report: bool,
    // Comptes
    acc_user: String,
    acc_pass: String,
    acc_gm: u8,
    acc_selected: Option<String>,
    // Base de données
    db_selected: String,
    table_selected: String,
    sql: String,
    db_tab: u8,
    db_filter: String,
    db_order: Option<(String, bool)>,
    editing: Option<(usize, usize, String)>,
    insert_form: Option<Vec<String>>,
    sql_history: Vec<String>,
    tables_requested_for: Option<String>,
    // Sauvegardes
    backup_selected: Option<PathBuf>,
    backup_label: String,
    // Journaux
    console_cmd: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, layout: Layout, settings: Settings, shared: Arc<Mutex<Snapshot>>, tx: Sender<Command>, quit_after: Option<u64>, start_tab: Option<String>, shots_dir: Option<PathBuf>) -> App {
        let tab = match start_tab.as_deref() {
            Some("accounts") => Tab::Accounts,
            Some("bots") => Tab::Bots,
            Some("modules") => Tab::Modules,
            Some("world") => Tab::World,
            Some("patches") => Tab::Patches,
            Some("database") => Tab::Database,
            Some("backups") => Tab::Backups,
            Some("settings") => Tab::Settings,
            Some("logs") => Tab::Logs,
            _ => Tab::Home,
        };
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.window_fill = Color32::from_rgba_unmultiplied(20, 16, 12, 240);
        visuals.extreme_bg_color = Color32::from_black_alpha(160);
        visuals.faint_bg_color = Color32::from_white_alpha(10);
        cc.egui_ctx.set_visuals(visuals);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        cc.egui_ctx.set_style(style);
        App {
            layout,
            shared,
            tx,
            tab,
            settings,
            settings_notice: None,
            closing: false,
            quit_at: quit_after.map(|s| std::time::Instant::now() + std::time::Duration::from_secs(s)),
            shots: shots_dir.map(|d| (d, 0, std::time::Instant::now() + std::time::Duration::from_secs(6), false)),
            shots_settled: false,
            pending: None,
            show_report: false,
            acc_user: String::new(),
            acc_pass: String::new(),
            acc_gm: 3,
            acc_selected: None,
            db_selected: crate::paths::DB_CHARACTERS.into(),
            table_selected: String::new(),
            sql: "SELECT guid, name, race, class, level, money FROM characters LIMIT 200".into(),
            db_tab: 0,
            db_filter: String::new(),
            db_order: None,
            editing: None,
            insert_form: None,
            sql_history: Vec::new(),
            tables_requested_for: None,
            backup_selected: None,
            backup_label: String::new(),
            console_cmd: String::new(),
        }
    }

    fn send(&self, c: Command) {
        let _ = self.tx.send(c);
    }
}

fn status_color(ok: bool, busy: bool) -> Color32 {
    if ok {
        Color32::from_rgb(90, 200, 90)
    } else if busy {
        Color32::from_rgb(230, 180, 60)
    } else {
        Color32::from_rgb(200, 90, 90)
    }
}

/// Ouvre un fichier texte dans l'application associée (`.conf` : souvent aucune) ; Bloc-notes en repli.
fn open_text_file(path: &std::path::Path) {
    let opened = std::process::Command::new("cmd").args(["/C", "start", "", &path.to_string_lossy()]).creation_flags(crate::db::CREATE_NO_WINDOW).status().map(|s| s.success()).unwrap_or(false);
    if !opened {
        let _ = std::process::Command::new("notepad.exe").arg(path).spawn();
    }
}

const WARN: Color32 = Color32::from_rgb(230, 180, 60);
const ERR: Color32 = Color32::from_rgb(230, 110, 110);

/// Onglets capturés par `--screenshots` (documentation, README), avec le nom du fichier PNG.
const SHOT_TABS: &[(Tab, &str)] = &[
    (Tab::Home, "launcher-home"),
    (Tab::Accounts, "launcher-accounts"),
    (Tab::Bots, "launcher-bots"),
    (Tab::Modules, "launcher-modules"),
    (Tab::World, "launcher-world"),
    (Tab::Patches, "launcher-patches"),
    (Tab::Database, "launcher-database"),
    (Tab::Backups, "launcher-backups"),
    (Tab::Settings, "launcher-settings"),
];

impl App {
    /// `--screenshots <dossier>` : parcourt les onglets et enregistre une capture PNG de chacun, puis ferme. Avec
    /// `--start`, la visite attend que les services soient prêts. Outil de documentation uniquement.
    fn screenshot_tour(&mut self, ctx: &egui::Context, snap: &Snapshot) {
        let Some((dir, idx, next, requested)) = self.shots.as_mut() else { return };
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
        // Réponse à une capture demandée : enregistrement, puis onglet suivant.
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            if *requested {
                let name = SHOT_TABS[*idx].1;
                let [w, h] = img.size;
                let rgba: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
                let _ = std::fs::create_dir_all(&*dir);
                if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, rgba) {
                    let _ = buf.save(dir.join(format!("{name}.png")));
                }
                *requested = false;
                *idx += 1;
                *next = std::time::Instant::now() + std::time::Duration::from_millis(2500);
                if *idx >= SHOT_TABS.len() {
                    self.shots = None;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
                self.tab = SHOT_TABS[*idx].0;
            }
            return;
        }
        if *requested || std::time::Instant::now() < *next {
            return;
        }
        // Attente d'un état stable : services prêts si l'orchestrateur démarre, ou au repos sinon.
        if snap.state.is_busy() {
            *next = std::time::Instant::now() + std::time::Duration::from_secs(1);
            return;
        }
        // Services tout juste prêts : laisser 45 s aux bots pour se connecter avant la première capture.
        if *idx == 0 && snap.state.services_up() && !self.shots_settled {
            self.shots_settled = true;
            if let Some((_, _, next, _)) = self.shots.as_mut() {
                *next = std::time::Instant::now() + std::time::Duration::from_secs(45);
            }
            return;
        }
        let Some((_, idx, next, requested)) = self.shots.as_mut() else { return };
        if self.tab != SHOT_TABS[*idx].0 {
            self.tab = SHOT_TABS[*idx].0;
            *next = std::time::Instant::now() + std::time::Duration::from_millis(2500);
            return;
        }
        *requested = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        let snap = self.shared.lock().unwrap().clone();
        if snap.report_fresh {
            self.shared.lock().unwrap().report_fresh = false;
            self.show_report = true;
        }

        self.screenshot_tour(ctx, &snap);
        if let Some(t) = self.quit_at {
            if std::time::Instant::now() >= t && !self.closing {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if snap.exit_requested && !self.closing {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // Fermeture : on arrête d'abord les services, puis on laisse la fenêtre se fermer.
        if ctx.input(|i| i.viewport().close_requested()) && (snap.state.services_up() || snap.state.is_busy()) {
            if !self.closing {
                self.closing = true;
                self.send(Command::StopAll);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.closing && !(snap.state.services_up() || snap.state.is_busy()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Fond : image de taverne recouvrant toute la fenêtre (aspect conservé), assombrie pour la lisibilité,
        // peinte sur le calque d'arrière-plan avant les panneaux.
        {
            let screen = ctx.screen_rect();
            let painter = ctx.layer_painter(egui::LayerId::background());
            let (iw, ih) = (1659.0_f32, 948.0_f32);
            let scale = (screen.width() / iw).max(screen.height() / ih);
            let rect = egui::Rect::from_center_size(screen.center(), egui::vec2(iw * scale, ih * scale));
            let img = egui::Image::new(egui::include_image!("../assets/fond.jpg"));
            if let Ok(egui::load::TexturePoll::Ready { texture }) = img.load_for_size(ctx, rect.size()) {
                painter.image(texture.id, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
            }
            painter.rect_filled(screen, 0.0, Color32::from_black_alpha(150));
        }
        let panel_frame = egui::Frame::new().inner_margin(8.0).fill(Color32::from_black_alpha(110));

        egui::TopBottomPanel::top("titre").frame(panel_frame).show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add(egui::Image::new(egui::include_image!("../assets/logo-512.png")).fit_to_exact_size(egui::vec2(56.0, 56.0)));
                ui.add_space(8.0);
                ui.label(RichText::new(fr::TITLE).size(26.0).strong().color(Color32::from_rgb(255, 204, 51)));
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Home, fr::TAB_HOME);
                ui.selectable_value(&mut self.tab, Tab::Accounts, fr::TAB_ACCOUNTS);
                ui.selectable_value(&mut self.tab, Tab::Bots, fr::TAB_BOTS);
                ui.selectable_value(&mut self.tab, Tab::Modules, fr::TAB_MODULES);
                ui.selectable_value(&mut self.tab, Tab::World, fr::TAB_WORLD);
                ui.selectable_value(&mut self.tab, Tab::Patches, fr::TAB_PATCHES);
                ui.selectable_value(&mut self.tab, Tab::Database, fr::TAB_DATABASE);
                ui.selectable_value(&mut self.tab, Tab::Backups, fr::TAB_BACKUPS);
                ui.selectable_value(&mut self.tab, Tab::Settings, fr::TAB_SETTINGS);
                ui.selectable_value(&mut self.tab, Tab::Logs, fr::TAB_LOGS);
            });
            ui.add_space(4.0);
        });

        egui::TopBottomPanel::bottom("etat").frame(panel_frame).show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(snap.state.label()).strong().color(status_color(snap.state.services_up(), snap.state.is_busy())));
                ui.separator();
                ui.label(if self.closing { fr::MSG_CLOSING } else { snap.message.as_str() });
            });
            if snap.state.is_busy() {
                ui.add(egui::ProgressBar::new(snap.progress).animate(true));
            }
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().frame(egui::Frame::new().inner_margin(14.0).fill(Color32::from_black_alpha(90))).show(ctx, |ui| match self.tab {
            Tab::Home => self.home(ui, &snap),
            Tab::Accounts => self.accounts(ui, &snap),
            Tab::Bots => self.bots(ui, &snap),
            Tab::Modules => self.modules(ui, &snap),
            Tab::World => self.world(ui, &snap),
            Tab::Patches => self.patches(ui, &snap),
            Tab::Database => self.database(ui, &snap),
            Tab::Backups => self.backups(ui, &snap),
            Tab::Settings => self.settings_tab(ui),
            Tab::Logs => self.logs(ui, &snap),
        });

        self.confirm_window(ctx);
        self.report_window(ctx, &snap);
    }
}

impl App {
    fn home(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let busy = snap.state.is_busy();
        if snap.readonly {
            ui.colored_label(WARN, RichText::new(fr::RO_TITLE).strong());
            ui.label(fr::RO_TEXT);
            ui.add_space(6.0);
            if ui.add_enabled(!busy, egui::Button::new(RichText::new(fr::RO_INSTALL).size(18.0).strong()).min_size(egui::vec2(220.0, 40.0))).clicked() {
                if let Some(dir) = rfd::FileDialog::new().set_title(fr::RO_PICK_TITLE).pick_folder() {
                    let dest = if dir.file_name().map(|n| n.to_string_lossy().contains("HomeCraft")).unwrap_or(false) { dir } else { dir.join("HomeCraft Edition WoTLK") };
                    self.send(Command::InstallTo(dest));
                }
            }
            if let Some(err) = &snap.error {
                ui.colored_label(ERR, err);
            }
            return;
        }
        egui::Grid::new("composants").num_columns(2).spacing([24.0, 8.0]).show(ui, |ui| {
            let client_txt = if snap.client.is_ok() { fr::CLIENT_OK } else if snap.client.exe_present { fr::CLIENT_REPAIR } else { fr::CLIENT_MISSING };
            ui.label(RichText::new(fr::COMP_CLIENT).strong());
            ui.label(RichText::new(client_txt).color(status_color(snap.client.is_ok(), false)));
            ui.end_row();
            ui.label(RichText::new(fr::COMP_DB).strong());
            ui.label(RichText::new(if snap.db_up { fr::DB_RUNNING } else { fr::DB_STOPPED }).color(status_color(snap.db_up, busy)));
            ui.end_row();
            let srv = if snap.world_up { fr::SERVER_RUNNING } else if busy { fr::SERVER_STARTING } else { fr::SERVER_STOPPED };
            ui.label(RichText::new(fr::COMP_SERVER).strong());
            ui.label(RichText::new(srv).color(status_color(snap.world_up, busy)));
            ui.end_row();
            ui.label(RichText::new(fr::COMP_BOTS).strong());
            ui.label(RichText::new(if snap.bots_enabled { fr::BOTS_ON } else { fr::BOTS_OFF }).color(status_color(snap.bots_enabled, false)));
            ui.end_row();
            if snap.client.is_classic() {
                ui.label(RichText::new(fr::COMP_PROXY).strong());
                let (txt, ok) = if snap.hermes_up { (fr::PROXY_RUNNING, true) } else if snap.hermes_installed { (fr::PROXY_READY, true) } else { (fr::PROXY_MISSING, false) };
                ui.label(RichText::new(txt).color(status_color(ok, false)));
                ui.end_row();
            }
        });
        ui.add_space(12.0);

        let can_play = snap.state == State::Ready && snap.client.is_ok();
        let can_start = matches!(snap.state, State::Installed | State::Error);
        let idle = matches!(snap.state, State::Installed | State::Error | State::Ready | State::NotInstalled);
        ui.horizontal_wrapped(|ui| {
            let play = ui.add_enabled(can_play || (can_start && snap.client.is_ok()), egui::Button::new(RichText::new(fr::BTN_PLAY).size(20.0).strong()).min_size(egui::vec2(180.0, 44.0)));
            if play.clicked() {
                self.send(Command::Play);
            }
            if ui.add_enabled(can_start, egui::Button::new(fr::BTN_START_SERVER).min_size(egui::vec2(0.0, 44.0))).clicked() {
                self.send(Command::StartAll);
            }
            if ui.add_enabled(snap.state.services_up(), egui::Button::new(fr::BTN_STOP).min_size(egui::vec2(0.0, 44.0))).clicked() {
                self.send(Command::StopAll);
            }
            if snap.state == State::Playing && ui.add(egui::Button::new(fr::BTN_KILL_GAME).min_size(egui::vec2(0.0, 44.0))).on_hover_text(fr::BTN_KILL_GAME_HINT).clicked() {
                self.send(Command::KillGame);
            }
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(!busy, egui::Button::new(fr::BTN_VERIFY)).clicked() {
                self.send(Command::Verify);
            }
            if ui.add_enabled(idle, egui::Button::new(fr::BTN_REPAIR)).on_hover_text(fr::RPT_REPAIR_HINT).clicked() {
                self.send(Command::Repair);
            }
            if ui.add_enabled(!busy, egui::Button::new(fr::BTN_BACKUP)).clicked() {
                self.send(Command::Backup("manuelle".into()));
                self.tab = Tab::Backups;
            }
            if ui.add_enabled(idle, egui::Button::new(fr::BTN_RESTORE)).clicked() {
                self.tab = Tab::Backups;
            }
            if ui.button(fr::BTN_SETTINGS).clicked() {
                self.tab = Tab::Settings;
            }
            if ui.button(fr::BTN_LOGS).clicked() {
                self.tab = Tab::Logs;
            }
        });
        if let Some(err) = &snap.error {
            ui.add_space(8.0);
            ui.colored_label(ERR, err);
        }
        ui.add_space(12.0);
        ui.separator();
        ui.label(RichText::new(fr::LOG_DIAG).strong());
        egui::Grid::new("diag").num_columns(2).spacing([24.0, 4.0]).show(ui, |ui| {
            ui.label("Dossier d'installation");
            ui.label(self.layout.root.display().to_string());
            ui.end_row();
            ui.label("Version du launcher");
            ui.label(Layout::version());
            ui.end_row();
            ui.label("Client");
            ui.label(format!("{} — version {}", snap.client.dir.display(), snap.client.version_string()));
            ui.end_row();
            if snap.client.is_classic() && !snap.client.arctium_present {
                ui.label("Arctium");
                ui.colored_label(ERR, "Arctium WoW Launcher.exe absent de la racine du client");
                ui.end_row();
            }
            if !snap.client.missing_mpq.is_empty() {
                ui.label("MPQ manquants");
                ui.label(snap.client.missing_mpq.join(", "));
                ui.end_row();
            }
            for (name, ok) in &snap.components {
                ui.label(name);
                ui.label(RichText::new(if *ok { "présent" } else { "ABSENT" }).color(status_color(*ok, false)));
                ui.end_row();
            }
            ui.label(fr::HOME_MMAPS);
            match &snap.mmaps {
                MmapsStatus::Unknown => ui.label("…"),
                MmapsStatus::Present => ui.label(RichText::new(fr::HOME_MMAPS_PRESENT).color(status_color(true, false))),
                MmapsStatus::Absent => ui.label(RichText::new(fr::HOME_MMAPS_ABSENT).color(WARN)),
                MmapsStatus::Running(since) => {
                    let m = since.elapsed().as_secs() / 60;
                    ui.label(RichText::new(format!("{} {m} min (le jeu reste jouable)", fr::HOME_MMAPS_RUNNING)).color(WARN))
                }
                MmapsStatus::DoneRestartNeeded => ui.label(RichText::new(fr::HOME_MMAPS_DONE_RESTART).color(WARN)),
                MmapsStatus::Failed(e) => ui.label(RichText::new(format!("{} {e}", fr::HOME_MMAPS_FAILED)).color(ERR)),
            };
            ui.end_row();
            ui.label("Personnages en ligne");
            ui.label(snap.online.to_string());
            ui.end_row();
        });
    }

    fn accounts(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(fr::ACC_TITLE);
        ui.label(fr::ACC_HINT);
        if !snap.world_up {
            ui.colored_label(WARN, fr::ACC_NEED_SERVER);
        }
        ui.add_space(8.0);
        ui.group(|ui| {
            egui::Grid::new("acc_form").num_columns(2).show(ui, |ui| {
                ui.label(fr::ACC_USER);
                ui.text_edit_singleline(&mut self.acc_user);
                ui.end_row();
                ui.label(fr::ACC_PASS);
                ui.add(egui::TextEdit::singleline(&mut self.acc_pass).password(true));
                ui.end_row();
                ui.label(fr::ACC_GM);
                ui.add(egui::Slider::new(&mut self.acc_gm, 0..=3));
                ui.end_row();
            });
            let valid = !self.acc_user.trim().is_empty() && self.acc_pass.len() >= 4 && self.acc_user.chars().all(|c| c.is_ascii_alphanumeric());
            ui.horizontal(|ui| {
                if ui.add_enabled(snap.world_up && valid, egui::Button::new(fr::ACC_CREATE)).clicked() {
                    self.send(Command::CreateAccount { user: self.acc_user.trim().to_string(), pass: self.acc_pass.clone(), gm: self.acc_gm });
                    self.acc_pass.clear();
                }
                let sel = self.acc_selected.clone();
                if ui.add_enabled(snap.world_up && sel.is_some() && self.acc_pass.len() >= 4, egui::Button::new(fr::ACC_SET_PASS)).clicked() {
                    self.send(Command::SetPassword { user: sel.clone().unwrap(), pass: self.acc_pass.clone() });
                    self.acc_pass.clear();
                }
                if ui.add_enabled(snap.world_up && sel.is_some(), egui::Button::new(fr::ACC_SET_GM)).clicked() {
                    self.send(Command::SetGm { user: sel.unwrap(), gm: self.acc_gm });
                }
                if ui.add_enabled(snap.db_up, egui::Button::new(fr::ACC_REFRESH)).clicked() {
                    self.send(Command::RefreshAccounts);
                }
            });
        });
        if let Some(n) = &snap.notice {
            ui.colored_label(WARN, n);
        }
        ui.add_space(8.0);
        let playing = snap.state == State::Playing;
        let last_login_user = snap.accounts.iter().filter(|a| !a.last_login.is_empty()).max_by(|a, b| a.last_login.cmp(&b.last_login)).map(|a| a.username.clone());
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(140.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(140.0))
            .column(Column::auto().at_least(100.0))
            .column(Column::remainder())
            .header(22.0, |mut h| {
                h.col(|ui| { ui.strong(fr::ACC_USER); });
                h.col(|ui| { ui.strong(fr::ACC_GM); });
                h.col(|ui| { ui.strong(fr::ACC_LAST); });
                h.col(|ui| { ui.strong(fr::BK_CHARS); });
                h.col(|_| {});
            })
            .body(|body| {
                body.rows(22.0, snap.accounts.len(), |mut row| {
                    let a = &snap.accounts[row.index()];
                    let selected = self.acc_selected.as_deref() == Some(a.username.as_str());
                    row.set_selected(selected);
                    row.col(|ui| { if ui.selectable_label(selected, &a.username).clicked() { self.acc_selected = Some(a.username.clone()); } });
                    row.col(|ui| { ui.label(a.gmlevel.to_string()); });
                    row.col(|ui| { ui.label(&a.last_login); });
                    row.col(|ui| { ui.label(a.characters.to_string()); });
                    row.col(|ui| {
                        if ui.add_enabled(!playing && snap.db_up, egui::Button::new(RichText::new(fr::ACC_DELETE).color(ERR))).clicked() {
                            let mut detail = format!("{} — {} {}", a.username, a.characters, fr::BK_CHARS.to_lowercase());
                            if last_login_user.as_deref() == Some(a.username.as_str()) {
                                detail.push_str(&format!("\n{}", fr::ACC_DELETE_SELF_WARN));
                            }
                            self.pending = Some(Pending::DeleteAccount(a.username.clone(), detail));
                        }
                    });
                });
            });
    }

    /// Rendu générique d'un catalogue d'options (voir conf_catalog) sur les surcharges `conf`.
    fn render_options(ui: &mut egui::Ui, catalog: &[conf_catalog::Opt], conf: &mut std::collections::BTreeMap<String, String>, id: &str) {
        let mut groups: Vec<&str> = Vec::new();
        for o in catalog {
            if !groups.contains(&o.group) {
                groups.push(o.group);
            }
        }
        for g in groups {
            ui.add_space(6.0);
            ui.label(RichText::new(g).strong().color(Color32::from_rgb(255, 204, 51)));
            egui::Grid::new(format!("{id}-{g}")).num_columns(3).spacing([14.0, 6.0]).striped(true).show(ui, |ui| {
                for o in catalog.iter().filter(|o| o.group == g) {
                    let current = conf.get(o.key).cloned().unwrap_or_else(|| o.default.to_string());
                    let mut new_value: Option<String> = None;
                    let r = ui.label(o.label);
                    if !o.help.is_empty() {
                        r.on_hover_text(o.help);
                    }
                    match o.kind {
                        conf_catalog::Kind::Bool => {
                            let mut b = current == "1";
                            if ui.checkbox(&mut b, "").changed() {
                                new_value = Some(if b { "1".into() } else { "0".into() });
                            }
                        }
                        conf_catalog::Kind::Int { min, max } => {
                            let mut v: i64 = current.parse().unwrap_or(0);
                            if ui.add(egui::Slider::new(&mut v, min..=max)).changed() {
                                new_value = Some(v.to_string());
                            }
                        }
                        conf_catalog::Kind::Float { min, max } => {
                            let mut v: f64 = current.parse().unwrap_or(1.0);
                            if ui.add(egui::Slider::new(&mut v, min..=max).step_by(0.05).fixed_decimals(2)).changed() {
                                new_value = Some(format!("{v:.2}"));
                            }
                        }
                        conf_catalog::Kind::Choice(items) => {
                            let label = items.iter().find(|(v, _)| *v == current).map(|(_, l)| *l).unwrap_or(current.as_str());
                            egui::ComboBox::from_id_salt(format!("{id}-{}", o.key)).selected_text(label).show_ui(ui, |ui| {
                                for (v, l) in items {
                                    if ui.selectable_label(*v == current, *l).clicked() {
                                        new_value = Some(v.to_string());
                                    }
                                }
                            });
                        }
                        conf_catalog::Kind::Text => {
                            let mut t = current.clone();
                            if ui.add(egui::TextEdit::singleline(&mut t).desired_width(360.0)).changed() {
                                new_value = Some(t);
                            }
                        }
                    }
                    let modified = conf.contains_key(o.key) && conf.get(o.key).map(String::as_str) != Some(o.default);
                    ui.horizontal(|ui| {
                        if modified {
                            ui.small(RichText::new(fr::OPT_MODIFIED).color(WARN));
                            if ui.small_button(fr::OPT_DEFAULT).clicked() {
                                conf.remove(o.key);
                            }
                        }
                    });
                    ui.end_row();
                    if let Some(v) = new_value {
                        if v == o.default {
                            conf.remove(o.key);
                        } else {
                            conf.insert(o.key.to_string(), v);
                        }
                    }
                }
            });
        }
    }

    fn save_settings(&mut self) {
        match self.settings.save(&self.layout.settings_file()) {
            Ok(()) => {
                self.settings_notice = Some(fr::SET_SAVED.into());
                self.send(Command::SettingsChanged(self.settings.clone()));
            }
            Err(e) => self.settings_notice = Some(e),
        }
    }

    fn world(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(fr::WORLD_TITLE);
        ui.label(fr::WORLD_INTRO);
        if snap.state.services_up() {
            ui.colored_label(WARN, fr::SET_SAVE);
        }
        ui.horizontal(|ui| {
            if ui.button(fr::WORLD_SAVE).clicked() {
                self.save_settings();
            }
            if ui.button(fr::OPT_RESET_ALL).clicked() {
                for o in conf_catalog::WORLD_OPTIONS {
                    self.settings.conf.remove(o.key);
                }
            }
            if let Some(n) = &self.settings_notice {
                ui.label(n);
            }
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Self::render_options(ui, conf_catalog::WORLD_OPTIONS, &mut self.settings.conf, "world");
        });
    }

    fn bots(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(fr::BOT_TITLE);
        ui.label(fr::BOT_INTRO);
        if snap.state.services_up() {
            ui.colored_label(WARN, fr::SET_SAVE);
        }
        ui.add_space(6.0);
        ui.group(|ui| {
            ui.label(RichText::new(fr::BOT_COUNT).strong());
            ui.add(egui::Slider::new(&mut self.settings.random_bots, 0..=500));
            ui.small(fr::BOT_COUNT_HINT);
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(fr::BOT_POOL);
                match snap.bot_pool {
                    Some((accounts, chars)) if accounts > 0 || chars > 0 => {
                        ui.label(RichText::new(format!("{accounts} comptes rndbot, {chars} personnages")).strong());
                    }
                    _ => {
                        ui.colored_label(WARN, fr::BOT_POOL_NONE);
                    }
                }
                let has_pool = matches!(snap.bot_pool, Some((a, c)) if a > 0 || c > 0);
                let idle = !snap.state.is_busy() && !snap.state.services_up();
                if ui.add_enabled(idle && has_pool, egui::Button::new(RichText::new(fr::BOTPOOL_DELETE).color(ERR))).on_disabled_hover_text(fr::BOTPOOL_DELETE_RUNNING).clicked() {
                    self.pending = Some(Pending::DeleteBots);
                }
            });
            if let Some(n) = &snap.notice {
                ui.colored_label(WARN, n);
            }
        });
        ui.horizontal(|ui| {
            if ui.button(fr::BOT_SAVE).clicked() {
                self.save_settings();
            }
            if ui.button(fr::OPT_RESET_ALL).clicked() {
                for o in conf_catalog::BOT_OPTIONS {
                    self.settings.conf.remove(o.key);
                }
            }
            if let Some(n) = &self.settings_notice {
                ui.label(n);
            }
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Self::render_options(ui, conf_catalog::BOT_OPTIONS, &mut self.settings.conf, "bots");
            ui.add_space(12.0);
            ui.separator();
            ui.strong(fr::BOT_CMDS);
            egui::Grid::new("bot_cmds").num_columns(2).spacing([16.0, 4.0]).striped(true).show(ui, |ui| {
                for (cmd, desc) in [
                    (".playerbots bot add <nom>", "invite un bot aléatoire (ou un de vos personnages hors ligne) dans votre groupe"),
                    (".playerbots bot addclass tank | heal | dps", "invite un bot aléatoire de ce rôle, à votre niveau"),
                    (".playerbots bot addclass warrior | paladin | hunter | rogue | priest | shaman | mage | warlock | druid | dk", "invite un bot aléatoire de cette classe"),
                    (".playerbots bot init <nom>", "rééquipe et réinitialise le bot ciblé pour son niveau"),
                    (".playerbots bot remove <nom>", "renvoie le bot"),
                    (".playerbots self", "votre propre personnage est joué par l'IA (selon le réglage « Contrôle de son propre personnage »)"),
                    ("/w <bot> follow / stay / flee", "ordres de déplacement au bot (par murmure)"),
                    ("/w <bot> attack / tank attack", "attaquer la cible / tanker la cible"),
                    ("/w <bot> co ? / nc ? / dead ?", "liste les stratégies de combat / hors combat / de mort"),
                    ("/w <bot> co +tank,-dps", "ajoute ou retire des stratégies"),
                    ("/w <bot> summon", "le bot vient à vous (ou vous rejoint via aubergiste)"),
                    ("/w <bot> talents spec <n>", "choisit une spécialisation de talents"),
                    ("/w <bot> e <lien d'objet> / ue <lien>", "équipe / déséquipe un objet"),
                    ("/w <bot> accept / reward", "accepte / rend les quêtes partagées"),
                    ("/w <bot> stats / items / spells", "état, inventaire, sorts du bot"),
                    ("/w <bot> help", "aide complète du bot"),
                ] {
                    ui.monospace(cmd);
                    ui.label(desc);
                    ui.end_row();
                }
            });
        });
    }

    fn modules(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(fr::MOD_TITLE);
        ui.label(fr::MOD_INTRO);
        if snap.state.services_up() {
            ui.colored_label(WARN, fr::SET_SAVE);
        }
        let modules_dir = self.layout.server_config().join("modules");
        ui.horizontal(|ui| {
            if ui.button(fr::MOD_OPEN_DIR).clicked() {
                let _ = std::process::Command::new("explorer.exe").arg(&modules_dir).spawn();
            }
            if ui.button(fr::ACC_REFRESH).clicked() {
                self.send(Command::RefreshModules);
            }
            if let Some(n) = &snap.notice {
                ui.colored_label(WARN, n);
            }
        });
        if snap.modules.iter().any(|m| m.id == "mod-playerbots" && m.enabled == Some(false)) {
            ui.colored_label(WARN, fr::MOD_PLAYERBOTS_OFF);
        }
        ui.add_space(6.0);
        let mut action: Option<Command> = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Grid::new("modules").num_columns(3).striped(true).spacing([14.0, 8.0]).show(ui, |ui| {
                ui.strong(fr::MOD_COL_MODULE);
                ui.strong(fr::MOD_COL_STATE);
                ui.label("");
                ui.end_row();
                for m in &snap.modules {
                    ui.vertical(|ui| {
                        // Largeur fixe : sans elle, egui réduit la colonne au minimum et coupe chaque mot.
                        ui.set_width((ui.available_width() - 260.0).clamp(320.0, 620.0));
                        ui.label(RichText::new(m.label).strong());
                        ui.add(egui::Label::new(RichText::new(m.description).small()).wrap());
                        ui.small(RichText::new(m.id).weak());
                    });
                    if m.id == "mod-ah-bot" {
                        let on = self.settings.ahbot_enabled;
                        let r = ui.colored_label(status_color(on, false), if on { fr::MOD_ON } else { fr::MOD_OFF });
                        r.on_hover_text(fr::MOD_SEE_SETTINGS);
                    } else if !m.switchable {
                        ui.label(fr::MOD_ALWAYS_ON);
                    } else {
                        let mut on = m.enabled.unwrap_or(true);
                        let label = if on { fr::MOD_ON } else { fr::MOD_OFF };
                        if ui.checkbox(&mut on, label).changed() {
                            action = Some(Command::ModuleSetEnabled(m.id.to_string(), on));
                        }
                    }
                    ui.horizontal(|ui| {
                        if let Some(p) = &m.conf_path {
                            if ui.button(fr::MOD_OPEN).on_hover_text(fr::MOD_OPEN_HINT).clicked() {
                                open_text_file(p);
                            }
                            if ui.button(fr::MOD_RESET).on_hover_text(fr::MOD_RESET_HINT).clicked() {
                                self.pending = Some(Pending::ResetModule(m.id.to_string(), m.label.to_string()));
                            }
                        }
                    });
                    ui.end_row();
                }
            });
        });
        if let Some(c) = action {
            self.send(c);
        }
    }

    fn patches(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(fr::PATCH_TITLE);
        if snap.client.is_classic() {
            ui.colored_label(WARN, fr::PATCH_CLASSIC_NOTE);
            ui.add_space(8.0);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                self.addons_section(ui, snap);
            });
            return;
        }
        ui.label(fr::PATCH_INTRO);
        ui.colored_label(WARN, fr::PATCH_SERVER_NOTE);
        let playing = snap.state == State::Playing;
        if playing {
            ui.colored_label(ERR, fr::PATCH_PLAYING);
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.add_enabled(!playing, egui::Button::new(fr::PATCH_ADD)).clicked() {
                if let Some(f) = rfd::FileDialog::new().set_title(fr::PATCH_ADD).add_filter("Archive MPQ", &["mpq", "MPQ"]).pick_file() {
                    self.send(Command::PatchAdd(f));
                }
            }
            if ui.button(fr::PATCH_OPEN_DIR).clicked() {
                let _ = std::process::Command::new("explorer.exe").arg(snap.client.dir.join("Data")).spawn();
            }
            if ui.button(fr::ACC_REFRESH).clicked() {
                self.send(Command::RefreshPatches);
            }
            if let Some(n) = &snap.notice {
                ui.colored_label(WARN, n);
            }
        });
        ui.add_space(6.0);
        if !snap.patches.iter().any(|p| !p.official) {
            ui.label(fr::PATCH_NONE);
        }
        let mut action: Option<Command> = None;
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(160.0))
            .column(Column::auto().at_least(90.0))
            .column(Column::auto().at_least(180.0))
            .column(Column::remainder())
            .header(22.0, |mut h| {
                h.col(|ui| { ui.strong(fr::PATCH_NAME); });
                h.col(|ui| { ui.strong(fr::PATCH_SIZE); });
                h.col(|ui| { ui.strong(fr::PATCH_STATE); });
                h.col(|_| {});
            })
            .body(|body| {
                body.rows(22.0, snap.patches.len(), |mut row| {
                    let p = &snap.patches[row.index()];
                    row.col(|ui| { ui.label(if p.locale { format!("{} {}", p.name, fr::PATCH_LOCALE) } else { p.name.clone() }); });
                    row.col(|ui| { ui.label(util::human_size(p.size)); });
                    row.col(|ui| {
                        if p.official {
                            ui.label(fr::PATCH_OFFICIAL);
                        } else if !p.loaded_by_client {
                            ui.colored_label(WARN, fr::PATCH_NOT_LOADED);
                        } else if p.active {
                            ui.colored_label(status_color(true, false), fr::PATCH_ACTIVE);
                        } else {
                            ui.colored_label(ERR, fr::PATCH_INACTIVE);
                        }
                    });
                    row.col(|ui| {
                        if !p.official {
                            if ui.add_enabled(!playing, egui::Button::new(if p.active { fr::PATCH_DISABLE } else { fr::PATCH_ENABLE })).clicked() {
                                action = Some(Command::PatchSetActive(p.name.clone(), !p.active));
                            }
                            if ui.add_enabled(!playing, egui::Button::new(fr::PATCH_REMOVE)).clicked() {
                                self.pending = Some(Pending::RemovePatch(p.name.clone()));
                            }
                        }
                    });
                });
            });
        if let Some(c) = action {
            self.send(c);
        }
        ui.add_space(8.0);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            self.addons_section(ui, snap);
        });
    }

    fn load_table(&self, offset: u64) {
        if self.table_selected.is_empty() {
            return;
        }
        self.send(Command::LoadTable {
            db: self.db_selected.clone(),
            table: self.table_selected.clone(),
            filter: self.db_filter.clone(),
            order: self.db_order.clone(),
            offset,
        });
    }

    /// Clause WHERE identifiant une ligne par sa clé primaire (None si la table n'en a pas).
    fn pk_where(view: &crate::orchestrator::TableView, row: &[String]) -> Option<String> {
        let parts: Vec<String> = view
            .columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.primary)
            .map(|(i, c)| format!("`{}`={}", c.name, crate::db::sql_literal(row.get(i).map(String::as_str).unwrap_or("NULL"))))
            .collect();
        if parts.is_empty() { None } else { Some(parts.join(" AND ")) }
    }

    fn database(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(fr::DB_TITLE);
        ui.small(fr::DB_INTRO);
        ui.group(|ui| {
            ui.label(RichText::new(fr::DB_CONN_TITLE).strong());
            let port = crate::paths::DB_PORT.to_string();
            let bases = crate::paths::ALL_DATABASES.join(", ");
            ui.horizontal_wrapped(|ui| {
                for (label, value) in [
                    (fr::DB_CONN_HOST, "127.0.0.1"),
                    (fr::DB_CONN_PORT, port.as_str()),
                    (fr::DB_CONN_USER, "root"),
                    (fr::DB_CONN_PASS, crate::credentials::ROOT_PASSWORD),
                ] {
                    ui.label(format!("{label} :"));
                    ui.label(RichText::new(value).monospace().strong().color(Color32::from_rgb(255, 204, 51)));
                    if ui.small_button(fr::DB_CONN_COPY).clicked() {
                        ui.ctx().copy_text(value.to_string());
                    }
                    ui.separator();
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{} :", fr::DB_CONN_BASES));
                ui.label(RichText::new(bases).monospace());
            });
            ui.small(fr::DB_CONN_HINT);
        });
        ui.horizontal(|ui| {
            if ui.add_enabled(snap.db_up, egui::Button::new(fr::DB_OPEN_HEIDI)).clicked() {
                self.send(Command::OpenHeidi);
            }
            let keira_present = self.layout.keira_exe().is_file();
            if ui.add_enabled(snap.db_up && keira_present, egui::Button::new(fr::DB_OPEN_KEIRA)).on_hover_text(fr::DB_KEIRA_HINT).on_disabled_hover_text(if keira_present { fr::DB_NEED_DB } else { fr::DB_KEIRA_MISSING }).clicked() {
                self.send(Command::OpenKeira);
            }
            if !snap.db_up {
                ui.colored_label(WARN, fr::DB_NEED_DB);
            }
            if let Some(n) = &snap.notice {
                ui.colored_label(ERR, n);
            }
        });
        if snap.table_dirty && snap.db_up {
            let off = snap.table_view.as_ref().map(|v| v.offset).unwrap_or(0);
            self.load_table(off);
        }
        // Liste des tables de la base courante chargée dès que la base répond (une seule demande par base).
        if snap.db_up && snap.tables.is_empty() && self.tables_requested_for.as_deref() != Some(self.db_selected.as_str()) {
            self.tables_requested_for = Some(self.db_selected.clone());
            self.send(Command::ListTables(self.db_selected.clone()));
        }
        ui.add_space(4.0);
        egui::SidePanel::left("db_tree").resizable(true).default_width(220.0).show_inside(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for d in snap.databases.clone() {
                    let open = self.db_selected == d;
                    let header = egui::CollapsingHeader::new(RichText::new(&d).strong()).default_open(open).show(ui, |ui| {
                        if open {
                            if snap.tables.is_empty() && snap.db_up {
                                ui.small("…");
                            }
                            for t in &snap.tables {
                                let sel = self.table_selected == *t;
                                if ui.selectable_label(sel, t).clicked() {
                                    self.table_selected = t.clone();
                                    self.db_filter.clear();
                                    self.db_order = None;
                                    self.editing = None;
                                    self.load_table(0);
                                }
                            }
                        }
                    });
                    if header.header_response.clicked() && !open && snap.db_up {
                        self.db_selected = d.clone();
                        self.table_selected.clear();
                        self.tables_requested_for = Some(d.clone());
                        self.send(Command::ListTables(d.clone()));
                    }
                }
            });
        });
        egui::CentralPanel::default().frame(egui::Frame::NONE).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.db_tab, 0, fr::DB_TAB_DATA);
                ui.selectable_value(&mut self.db_tab, 1, fr::DB_TAB_STRUCTURE);
                ui.selectable_value(&mut self.db_tab, 2, fr::DB_TAB_SQL);
                if !self.table_selected.is_empty() {
                    ui.separator();
                    ui.label(RichText::new(format!("{}.{}", self.db_selected, self.table_selected)).strong());
                }
            });
            ui.separator();
            match self.db_tab {
                0 => self.db_data_tab(ui, snap),
                1 => self.db_structure_tab(ui, snap),
                _ => self.db_sql_tab(ui, snap),
            }
        });
    }

    fn db_data_tab(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(view) = snap.table_view.as_ref() else {
            if let Some(Err(e)) = &snap.query {
                ui.colored_label(ERR, e);
            }
            return;
        };
        let has_pk = view.columns.iter().any(|c| c.primary);
        ui.horizontal(|ui| {
            ui.label(fr::DB_FILTER);
            let r = ui.add(egui::TextEdit::singleline(&mut self.db_filter).desired_width(320.0).hint_text("ex. level > 10 AND race = 1"));
            if ui.button(fr::DB_APPLY_FILTER).clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                self.load_table(0);
            }
            ui.separator();
            if ui.add_enabled(view.offset > 0, egui::Button::new(fr::DB_PREV)).clicked() {
                self.load_table(view.offset.saturating_sub(200));
            }
            ui.label(format!("{}–{} / {} {}", view.offset + 1, (view.offset + view.page.rows.len() as u64).min(view.total), view.total, fr::DB_TOTAL));
            if ui.add_enabled(view.offset + 200 < view.total, egui::Button::new(fr::DB_NEXT)).clicked() {
                self.load_table(view.offset + 200);
            }
            ui.separator();
            if ui.add_enabled(has_pk, egui::Button::new(fr::DB_INSERT)).clicked() {
                self.insert_form = Some(view.columns.iter().map(|_| String::new()).collect());
            }
        });
        if !has_pk {
            ui.colored_label(WARN, fr::DB_NO_PK);
        }
        if let Some(Err(e)) = &snap.query {
            ui.colored_label(ERR, e);
        }
        // Formulaire d'insertion
        if let Some(form) = self.insert_form.as_mut() {
            let mut submit = false;
            let mut cancel = false;
            egui::Window::new(fr::DB_INSERT_TITLE).collapsible(false).show(ui.ctx(), |ui| {
                ui.small(fr::DB_INSERT_HINT);
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    egui::Grid::new("insert_form").num_columns(2).show(ui, |ui| {
                        for (i, c) in view.columns.iter().enumerate() {
                            ui.label(format!("{} ({})", c.name, c.col_type));
                            ui.add(egui::TextEdit::singleline(&mut form[i]).desired_width(260.0).hint_text(c.default.clone().unwrap_or_default()));
                            ui.end_row();
                        }
                    });
                });
                ui.horizontal(|ui| {
                    if ui.button(fr::CONFIRM).clicked() { submit = true; }
                    if ui.button(fr::CANCEL).clicked() { cancel = true; }
                });
            });
            if submit {
                let cols: Vec<String> = view.columns.iter().zip(form.iter()).filter(|(_, v)| !v.is_empty()).map(|(c, _)| format!("`{}`", c.name)).collect();
                let vals: Vec<String> = form.iter().filter(|v| !v.is_empty()).map(|v| crate::db::sql_literal(v)).collect();
                let sql = if cols.is_empty() { format!("INSERT INTO `{}` () VALUES ()", view.table) } else { format!("INSERT INTO `{}` ({}) VALUES ({})", view.table, cols.join(","), vals.join(",")) };
                self.send(Command::Exec { db: view.db.clone(), sql });
                self.insert_form = None;
            } else if cancel {
                self.insert_form = None;
            }
        }
        // Grille
        let ncols = view.columns.len().max(1);
        let mut pending_sql: Option<String> = None;
        let mut delete_row: Option<usize> = None;
        let mut sort_click: Option<usize> = None;
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let mut tb = TableBuilder::new(ui).striped(true).resizable(true).vscroll(true).max_scroll_height(f32::INFINITY).column(Column::auto().at_least(28.0));
            for _ in 0..ncols {
                tb = tb.column(Column::auto().at_least(70.0).clip(true));
            }
            tb.header(22.0, |mut h| {
                h.col(|_| {});
                for (ci, c) in view.columns.iter().enumerate() {
                    h.col(|ui| {
                        let mark = match &self.db_order { Some((col, asc)) if *col == c.name => if *asc { " ▲" } else { " ▼" }, _ => "" };
                        if ui.add(egui::Label::new(RichText::new(format!("{}{}{}", if c.primary { "🔑 " } else { "" }, c.name, mark)).strong()).sense(egui::Sense::click())).clicked() {
                            sort_click = Some(ci);
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(20.0, view.page.rows.len(), |mut row| {
                    let ri = row.index();
                    let data = &view.page.rows[ri];
                    row.col(|ui| {
                        if has_pk && ui.add(egui::Button::new("✖").small()).on_hover_text(fr::DB_DELETE_ROW).clicked() {
                            delete_row = Some(ri);
                        }
                    });
                    for (ci, v) in data.iter().enumerate() {
                        row.col(|ui| {
                            let editing_here = matches!(self.editing, Some((r, c, _)) if r == ri && c == ci);
                            if editing_here {
                                let (_, _, text) = self.editing.as_mut().unwrap();
                                let resp = ui.add(egui::TextEdit::singleline(text).desired_width(f32::INFINITY));
                                resp.request_focus();
                                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    if let Some(w) = Self::pk_where(view, data) {
                                        let new_val = text.clone();
                                        if new_val != *v {
                                            pending_sql = Some(format!("UPDATE `{}` SET `{}`={} WHERE {} LIMIT 1", view.table, view.columns[ci].name, crate::db::sql_literal(&new_val), w));
                                        }
                                    }
                                    self.editing = None;
                                } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    self.editing = None;
                                }
                            } else {
                                let r = ui.add(egui::Label::new(v).sense(egui::Sense::click()));
                                if r.double_clicked() && has_pk {
                                    self.editing = Some((ri, ci, v.clone()));
                                }
                            }
                        });
                    }
                });
            });
        });
        if let Some(ci) = sort_click {
            let col = view.columns[ci].name.clone();
            self.db_order = match &self.db_order { Some((c, true)) if *c == col => Some((col, false)), Some((c, false)) if *c == col => None, _ => Some((col, true)) };
            self.load_table(0);
        }
        if let Some(sql) = pending_sql {
            self.send(Command::Exec { db: view.db.clone(), sql });
        }
        if let Some(ri) = delete_row {
            if let Some(w) = Self::pk_where(view, &view.page.rows[ri]) {
                self.pending = Some(Pending::ExecSql(view.db.clone(), format!("DELETE FROM `{}` WHERE {} LIMIT 1", view.table, w), fr::DB_DELETE_ROW_CONFIRM_TITLE, fr::DB_DELETE_ROW_CONFIRM));
            }
        }
    }

    fn db_structure_tab(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(view) = snap.table_view.as_ref() else { return };
        egui::Grid::new("structure").num_columns(6).striped(true).spacing([14.0, 4.0]).show(ui, |ui| {
            for h in [fr::DB_COL_NAME, fr::DB_COL_TYPE, fr::DB_COL_NULL, fr::DB_COL_KEY, fr::DB_COL_DEFAULT, "Extra"] {
                ui.strong(h);
            }
            ui.end_row();
            for c in &view.columns {
                ui.label(&c.name);
                ui.label(&c.col_type);
                ui.label(if c.nullable { "oui" } else { "non" });
                ui.label(if c.primary { "PRIMARY" } else { "" });
                ui.label(c.default.clone().unwrap_or_else(|| "NULL".into()));
                ui.label(&c.extra);
                ui.end_row();
            }
        });
    }

    fn db_sql_tab(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.add(egui::TextEdit::multiline(&mut self.sql).desired_rows(5).desired_width(f32::INFINITY).code_editor());
        ui.horizontal(|ui| {
            let is_read = crate::db::is_read_statement(&self.sql);
            if ui.add_enabled(snap.db_up && is_read, egui::Button::new(fr::DB_RUN_READ)).clicked() {
                self.sql_history.push(self.sql.clone());
                self.send(Command::Query { db: self.db_selected.clone(), sql: self.sql.clone() });
            }
            if ui.add_enabled(snap.db_up && !is_read && !self.sql.trim().is_empty(), egui::Button::new(RichText::new(fr::DB_RUN_WRITE).color(WARN))).clicked() {
                self.sql_history.push(self.sql.clone());
                self.pending = Some(Pending::ExecSql(self.db_selected.clone(), self.sql.clone(), fr::DB_WRITE_CONFIRM_TITLE, fr::DB_WRITE_CONFIRM));
            }
            if !self.sql_history.is_empty() {
                egui::ComboBox::from_id_salt("sql_hist").selected_text(fr::DB_HISTORY).show_ui(ui, |ui| {
                    for h in self.sql_history.iter().rev().take(20) {
                        if ui.selectable_label(false, h.lines().next().unwrap_or("").chars().take(80).collect::<String>()).clicked() {
                            self.sql = h.clone();
                        }
                    }
                });
            }
        });
        match &snap.query {
            None => {}
            Some(Err(e)) => { ui.colored_label(ERR, e); }
            Some(Ok(r)) if r.affected.is_some() => { ui.colored_label(status_color(true, false), format!("{} {}", r.affected.unwrap(), fr::DB_AFFECTED)); }
            Some(Ok(r)) => {
                ui.label(format!("{} {}{}", r.rows.len(), fr::DB_ROWS, if r.truncated { " (tronqué à 500)" } else { "" }));
                let ncols = r.columns.len().max(1);
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    let mut tb = TableBuilder::new(ui).striped(true).resizable(true).vscroll(true).max_scroll_height(f32::INFINITY);
                    for _ in 0..ncols {
                        tb = tb.column(Column::auto().at_least(60.0).clip(true));
                    }
                    tb.header(22.0, |mut h| {
                        for c in &r.columns {
                            h.col(|ui| { ui.strong(c); });
                        }
                    })
                    .body(|body| {
                        body.rows(18.0, r.rows.len(), |mut row| {
                            let data = &r.rows[row.index()];
                            for v in data {
                                row.col(|ui| { ui.label(v); });
                            }
                        });
                    });
                });
            }
        }
    }

    fn backups(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(fr::BK_TITLE);
        ui.label(fr::BK_HINT);
        if snap.state == State::Playing {
            ui.colored_label(WARN, fr::BK_PLAYING_WARN);
        }
        if let Some(n) = &snap.notice {
            ui.colored_label(WARN, n);
        }
        ui.add_space(6.0);
        let busy = snap.state.is_busy();
        let idle = matches!(snap.state, State::Installed | State::Error | State::Ready | State::NotInstalled);
        ui.horizontal_wrapped(|ui| {
            ui.label(fr::BK_LABEL);
            ui.add(egui::TextEdit::singleline(&mut self.backup_label).desired_width(200.0));
            if ui.add_enabled(!busy, egui::Button::new(fr::BK_NOW)).clicked() {
                let label = if self.backup_label.trim().is_empty() { "manuelle".to_string() } else { self.backup_label.trim().to_string() };
                self.send(Command::Backup(label));
                self.backup_label.clear();
            }
            let sel = self.backup_selected.clone();
            let sel_info = sel.as_ref().and_then(|p| snap.backups.iter().find(|b| &b.path == p));
            if ui.add_enabled(idle && sel_info.is_some(), egui::Button::new(fr::BK_RESTORE_SEL)).clicked() {
                let b = sel_info.unwrap();
                self.pending = Some(Pending::Restore(b.path.clone(), format!("{} — {} personnages, {} comptes", b.manifest.created, b.manifest.characters, b.manifest.accounts)));
            }
            if ui.add_enabled(!busy && sel_info.is_some(), egui::Button::new(fr::BK_DELETE_SEL)).clicked() {
                let b = sel_info.unwrap();
                self.pending = Some(Pending::Delete(b.path.clone(), b.manifest.created.clone()));
            }
            if ui.button(fr::BK_OPEN_DIR).clicked() {
                let dir = self.layout.backups();
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::process::Command::new("explorer.exe").arg(&dir).spawn();
            }
            if ui.add_enabled(!busy, egui::Button::new(fr::BK_DIAG)).clicked() {
                self.send(Command::ExportDiagnostic);
            }
            if ui.button(fr::ACC_REFRESH).clicked() {
                self.send(Command::RefreshBackups);
            }
        });
        ui.add_space(8.0);
        if snap.backups.is_empty() {
            ui.label(fr::BK_NONE);
            return;
        }
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(150.0))
            .column(Column::auto().at_least(180.0))
            .column(Column::auto().at_least(90.0))
            .column(Column::auto().at_least(70.0))
            .column(Column::remainder())
            .header(22.0, |mut h| {
                h.col(|ui| { ui.strong(fr::BK_DATE); });
                h.col(|ui| { ui.strong(fr::BK_LABEL); });
                h.col(|ui| { ui.strong(fr::BK_CHARS); });
                h.col(|ui| { ui.strong(fr::BK_ACCOUNTS); });
                h.col(|ui| { ui.strong(fr::BK_SIZE); });
            })
            .body(|body| {
                body.rows(20.0, snap.backups.len(), |mut row| {
                    let b = &snap.backups[row.index()];
                    let selected = self.backup_selected.as_ref() == Some(&b.path);
                    row.set_selected(selected);
                    row.col(|ui| { if ui.selectable_label(selected, &b.manifest.created).clicked() { self.backup_selected = Some(b.path.clone()); } });
                    row.col(|ui| { ui.label(&b.manifest.label); });
                    row.col(|ui| { ui.label(b.manifest.characters.to_string()); });
                    row.col(|ui| { ui.label(b.manifest.accounts.to_string()); });
                    row.col(|ui| { ui.label(util::human_size(b.size)); });
                });
            });
    }

    fn addons_section(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.separator();
        ui.strong(fr::ADDON_TITLE);
        ui.small(fr::ADDON_INTRO);
        let playing = snap.state == State::Playing;
        if playing {
            ui.colored_label(ERR, fr::ADDON_PLAYING);
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(playing == false, egui::Button::new(fr::ADDON_ADD)).clicked() {
                if let Some(f) = rfd::FileDialog::new().set_title(fr::ADDON_ADD).add_filter("Archive zip", &["zip"]).pick_file() {
                    self.send(Command::AddonAdd(f));
                }
            }
            if ui.button(fr::ADDON_OPEN_DIR).clicked() {
                let dir = crate::addons::addons_dir(&snap.client.interface_root());
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::process::Command::new("explorer.exe").arg(dir).spawn();
            }
            if ui.button(fr::ACC_REFRESH).clicked() {
                self.send(Command::RefreshAddons);
            }
            if let Some(n) = &snap.notice {
                ui.colored_label(WARN, n);
            }
        });
        if snap.addons.is_empty() {
            ui.label(fr::ADDON_NONE);
            return;
        }
        let mut action: Option<Command> = None;
        egui::Grid::new("addons").num_columns(6).striped(true).spacing([12.0, 4.0]).show(ui, |ui| {
            ui.strong(fr::ADDON_NAME);
            ui.strong(fr::ADDON_TITLE_COL);
            ui.strong(fr::ADDON_VERSION);
            ui.strong(fr::ADDON_INTERFACE);
            ui.strong(fr::ADDON_STATE);
            ui.label("");
            ui.end_row();
            for a in &snap.addons {
                ui.label(&a.name);
                ui.label(&a.title);
                ui.label(&a.version);
                let r = ui.label(&a.interface);
                let expected = if snap.client.is_classic() { "30403" } else { "30300" };
                if !a.interface.is_empty() && a.interface != expected {
                    r.on_hover_text(fr::ADDON_OLD_INTERFACE);
                }
                if !a.has_toc {
                    ui.colored_label(WARN, fr::ADDON_NO_TOC);
                } else if a.active {
                    ui.colored_label(status_color(true, false), fr::ADDON_ACTIVE);
                } else {
                    ui.colored_label(ERR, fr::ADDON_INACTIVE);
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(!playing, egui::Button::new(if a.active { fr::PATCH_DISABLE } else { fr::PATCH_ENABLE })).clicked() {
                        action = Some(Command::AddonSetActive(a.name.clone(), !a.active));
                    }
                    if ui.add_enabled(!playing, egui::Button::new(fr::ADDON_REMOVE)).clicked() {
                        self.pending = Some(Pending::RemoveAddon(a.name.clone()));
                    }
                });
                ui.end_row();
            }
        });
        if let Some(c) = action {
            self.send(c);
        }
    }

    fn settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(fr::TAB_SETTINGS);
        let snap = self.shared.lock().unwrap().clone();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.label(RichText::new(fr::SET_CLIENT_DIR).strong());
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.settings.client_dir).desired_width(480.0));
                if ui.button(fr::SET_BROWSE).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().set_title(fr::SET_CLIENT_DIR).pick_folder() {
                        self.settings.client_dir = dir.to_string_lossy().into_owned();
                    }
                }
            });
            ui.small(fr::SET_CLIENT_HINT);
            ui.horizontal_wrapped(|ui| {
                ui.label(fr::SET_CLIENT_KIND);
                ui.radio_value(&mut self.settings.client_kind, ClientKind::Auto, fr::SET_CLIENT_AUTO);
                ui.radio_value(&mut self.settings.client_kind, ClientKind::Wotlk335, fr::SET_CLIENT_335);
                ui.radio_value(&mut self.settings.client_kind, ClientKind::Classic343, fr::SET_CLIENT_343);
            });
            ui.add_space(6.0);
            ui.checkbox(&mut self.settings.windowed, fr::SET_WINDOWED);
            ui.small(fr::SET_WINDOWED_HINT);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(fr::SET_REALM_NAME);
                ui.add(egui::TextEdit::singleline(&mut self.settings.realm_name).desired_width(240.0).hint_text(crate::paths::REALM_NAME));
            });
            ui.add_space(6.0);
            ui.checkbox(&mut self.settings.ahbot_enabled, fr::SET_AHBOT);
            ui.add_enabled_ui(self.settings.ahbot_enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.label(fr::SET_AHBOT_ACCOUNT);
                    ui.add(egui::DragValue::new(&mut self.settings.ahbot_account).range(0..=u32::MAX));
                    ui.label(fr::SET_AHBOT_GUID);
                    ui.add(egui::DragValue::new(&mut self.settings.ahbot_guid).range(0..=u32::MAX));
                });
            });
            ui.small(fr::SET_AHBOT_HINT);
            ui.add_space(6.0);
            ui.label(fr::SET_HERMES_ARGS);
            ui.add(egui::TextEdit::singleline(&mut self.settings.hermes_args).desired_width(480.0));
            ui.small(fr::SET_HERMES_HINT);
            if !snap.hermes_installed {
                ui.colored_label(WARN, fr::MSG_HERMES_NEEDED);
            }
            ui.add_space(6.0);
            ui.checkbox(&mut self.settings.auto_mmaps, fr::SET_AUTO_MMAPS);
            ui.small(fr::SET_AUTO_MMAPS_HINT);
            ui.add_space(4.0);
            ui.checkbox(&mut self.settings.auto_backup, fr::SET_AUTO_BACKUP);
            ui.horizontal(|ui| {
                ui.label(fr::SET_AUTO_BACKUP_KEEP);
                ui.add(egui::DragValue::new(&mut self.settings.auto_backup_keep).range(1..=100));
            });
            ui.small(fr::SET_AUTO_BACKUP_HINT);
            ui.add_space(8.0);
            if ui.button(fr::SET_SAVE).clicked() {
                self.save_settings();
            }
            if let Some(n) = &self.settings_notice {
                ui.label(n);
            }
            ui.add_space(12.0);
            ui.separator();
            let idle = matches!(snap.state, State::Installed | State::Error | State::Ready | State::NotInstalled);
            ui.horizontal_wrapped(|ui| {
                let running = matches!(snap.mmaps, MmapsStatus::Running(_));
                if ui.add_enabled(!running && !snap.state.is_busy(), egui::Button::new(fr::BTN_MMAPS)).on_hover_text(fr::BTN_MMAPS_HINT).clicked() {
                    self.send(Command::GenerateMmaps);
                }
                if ui.add_enabled(idle, egui::Button::new(fr::BTN_WORLD_REIMPORT)).on_hover_text(fr::WORLD_REIMPORT_HINT).clicked() {
                    self.pending = Some(Pending::ReimportWorld);
                }
            });
            ui.small(fr::WORLD_REIMPORT_HINT);
            if let Some(n) = &snap.notice {
                ui.colored_label(WARN, n);
            }
        });
    }

    fn logs(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(fr::LOG_TITLE);
        ui.horizontal(|ui| {
            ui.label(fr::LOG_CONSOLE);
            let r = ui.add(egui::TextEdit::singleline(&mut self.console_cmd).desired_width(400.0));
            let submit = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if (ui.add_enabled(snap.world_up, egui::Button::new(fr::LOG_SEND)).clicked() || submit) && !self.console_cmd.trim().is_empty() {
                self.send(Command::Console(self.console_cmd.trim().to_string()));
                self.console_cmd.clear();
            }
        });
        egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
            for l in snap.logs.iter() {
                ui.monospace(l);
            }
        });
    }

    fn confirm_window(&mut self, ctx: &egui::Context) {
        let Some(p) = self.pending.as_ref() else { return };
        let (title, text, detail) = match p {
            Pending::Restore(_, d) => (fr::BK_RESTORE_CONFIRM_TITLE, fr::BK_RESTORE_CONFIRM, d.clone()),
            Pending::Delete(_, d) => (fr::BK_DELETE_CONFIRM_TITLE, fr::BK_DELETE_CONFIRM, d.clone()),
            Pending::DeleteBots => (fr::BOTPOOL_DELETE_CONFIRM_TITLE, fr::BOTPOOL_DELETE_CONFIRM, String::new()),
            Pending::ResetModule(_, label) => (fr::MOD_RESET_CONFIRM_TITLE, fr::MOD_RESET_CONFIRM, label.clone()),
            Pending::RemovePatch(n) => (fr::PATCH_REMOVE_CONFIRM_TITLE, fr::PATCH_REMOVE_CONFIRM, n.clone()),
            Pending::RemoveAddon(n) => (fr::ADDON_REMOVE_CONFIRM_TITLE, fr::ADDON_REMOVE_CONFIRM, n.clone()),
            Pending::DeleteAccount(_, d) => (fr::ACC_DELETE_CONFIRM_TITLE, fr::ACC_DELETE_CONFIRM, d.clone()),
            Pending::ExecSql(_, sql, t, x) => (*t, *x, sql.chars().take(300).collect()),
            Pending::ReimportWorld => (fr::WORLD_REIMPORT_CONFIRM_TITLE, fr::WORLD_REIMPORT_CONFIRM, String::new()),
        };
        let mut decision: Option<bool> = None;
        egui::Window::new(title).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(text);
            ui.label(RichText::new(detail).strong());
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new(fr::CONFIRM).strong()).clicked() {
                    decision = Some(true);
                }
                if ui.button(fr::CANCEL).clicked() {
                    decision = Some(false);
                }
            });
        });
        if let Some(ok) = decision {
            if let Some(p) = self.pending.take() {
                if ok {
                    match p {
                        Pending::Restore(path, _) => self.send(Command::Restore(path)),
                        Pending::Delete(path, _) => {
                            self.backup_selected = None;
                            self.send(Command::DeleteBackup(path));
                        }
                        Pending::DeleteBots => self.send(Command::DeleteBots),
                        Pending::ResetModule(id, _) => self.send(Command::ModuleReset(id)),
                        Pending::RemovePatch(n) => self.send(Command::PatchRemove(n)),
                        Pending::RemoveAddon(n) => self.send(Command::AddonRemove(n)),
                        Pending::DeleteAccount(n, _) => {
                            self.acc_selected = None;
                            self.send(Command::DeleteAccount(n));
                        }
                        Pending::ExecSql(db, sql, _, _) => self.send(Command::Exec { db, sql }),
                        Pending::ReimportWorld => self.send(Command::ReimportWorld),
                    }
                }
            }
        }
    }

    fn report_window(&mut self, ctx: &egui::Context, snap: &Snapshot) {
        if !self.show_report {
            return;
        }
        let Some(r) = snap.report.as_ref() else { return };
        let mut open = true;
        egui::Window::new(fr::RPT_TITLE).open(&mut open).collapsible(false).default_width(560.0).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                if !r.manifest_present {
                    ui.colored_label(WARN, fr::RPT_MANIFEST_MISSING);
                } else if r.files_ok() {
                    ui.colored_label(status_color(true, false), format!("{} ({})", fr::RPT_FILES_OK, r.checked));
                } else {
                    if !r.missing.is_empty() {
                        ui.colored_label(ERR, format!("{} ({})", fr::RPT_MISSING, r.missing.len()));
                        for m in r.missing.iter().take(50) {
                            ui.monospace(m);
                        }
                    }
                    if !r.altered.is_empty() {
                        ui.colored_label(ERR, format!("{} ({})", fr::RPT_ALTERED, r.altered.len()));
                        for m in r.altered.iter().take(50) {
                            ui.monospace(m);
                        }
                    }
                }
                ui.add_space(6.0);
                ui.strong(fr::RPT_DATA);
                for (n, ok) in &r.data {
                    ui.horizontal(|ui| {
                        ui.label(n);
                        ui.label(RichText::new(if *ok { "OK" } else if n == "mmaps" { "absentes (optionnel : pathfinding dégradé)" } else { "MANQUANTES" }).color(status_color(*ok, n == "mmaps")));
                    });
                }
                ui.add_space(6.0);
                ui.strong(fr::RPT_DB);
                ui.label(match r.db_ok {
                    Some(true) => fr::RPT_DB_OK.to_string(),
                    Some(false) => "bases ou schéma incomplets : utilisez RÉPARER".to_string(),
                    None => fr::RPT_DB_UNKNOWN.to_string(),
                });
                for n in &r.notes {
                    ui.add_space(4.0);
                    ui.label(n);
                }
            });
            ui.add_space(6.0);
            ui.label(fr::RPT_REPAIR_HINT);
        });
        if !open {
            self.show_report = false;
        }
    }
}
