//! Profils de lancement : boutons de l'accueil et fenêtre de gestion (création, modification, renommage,
//! suppression, profil par défaut). Le mot de passe saisi n'existe en clair que dans le champ de saisie ; il est
//! chiffré (DPAPI) dès l'enregistrement, et n'est jamais réaffiché.

use super::{App, ERR, OK_COLOR, Pending, WARN};
use crate::i18n::t;
use crate::orchestrator::{Command, Snapshot};
use crate::profiles::{Profile, Profiles};
use eframe::egui::{self, RichText};

/// Profil en cours de modification
pub(super) struct ProfileEdit {
    pub p: Profile,
    pub password: String,
    pub store_password: bool,
    pub clear_password: bool,
    pub error: Option<String>,
}

impl App {
    /// Boutons des profils sur l'accueil : un clic lance le client prérempli (un client par profil).
    pub(super) fn home_profiles(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t("Profils :", "Profiles:")).strong());
            let guest = crate::network::guest_active();
            let can = snap.client.is_ok() && (guest || !snap.state.is_busy());
            let mut list = self.profiles.list.clone();
            list.sort_by_key(|p| p.id != self.profiles.default_id);
            for p in list {
                let running = snap.games.iter().any(|g| g.0.as_deref() == Some(p.id.as_str()));
                let mut text = RichText::new(format!("{}{}", if p.id == self.profiles.default_id { "★ " } else { "" }, p.name)).strong();
                if running {
                    text = text.color(OK_COLOR);
                }
                let hint = format!(
                    "{} {}{}{}",
                    t("Compte", "Account"),
                    p.account,
                    if p.character.is_empty() { String::new() } else { format!(" · {}", p.character) },
                    if p.has_password() { t(" · connexion automatique", " · automatic login") } else { "" }
                );
                let b = ui.add_enabled(can && !running, egui::Button::new(text).min_size(egui::vec2(0.0, 32.0))).on_hover_text(if running { t("client déjà ouvert", "client already open").to_string() } else { hint });
                if b.clicked() {
                    self.send(if guest { Command::PlayGuest(Some(p)) } else { Command::PlayProfile(p) });
                }
            }
            if ui.button(t("Gérer les profils…", "Manage profiles…")).clicked() {
                self.show_profiles = true;
            }
        });
    }

    pub(super) fn profiles_window(&mut self, ctx: &egui::Context, snap: &Snapshot) {
        if !self.show_profiles {
            return;
        }
        let mut open = true;
        egui::Window::new(t("Profils de lancement", "Launch profiles")).open(&mut open).collapsible(false).default_width(560.0).show(ctx, |ui| {
            ui.label(t(
                "Un profil lance le client avec votre compte prérempli et votre personnage préféré présélectionné. Le mot de passe, facultatif, est chiffré pour votre session Windows (DPAPI) et n'est jamais écrit en clair ni envoyé sur le réseau.",
                "A profile starts the client with your account filled in and your preferred character preselected. The optional password is encrypted for your Windows session (DPAPI) and is never written in clear text nor sent over the network.",
            ));
            if !snap.client.is_benilla() && snap.client.is_ok() {
                ui.colored_label(WARN, t(
                    "Client d'origine : le nom de compte est prérempli (Config.wtf) ; le mot de passe enregistré est tapé dans la fenêtre du client quelques secondes après son ouverture (le 1.12.1 n'a pas d'autre moyen) ; pas de présélection du personnage.",
                    "Original client: the account name is filled in (Config.wtf); a stored password is typed into the client window a few seconds after it opens (1.12.1 has no other way); no character preselection.",
                ));
            }
            ui.add_space(6.0);
            let mut action: Option<(u8, String)> = None;
            egui::Grid::new("profils").num_columns(5).striped(true).spacing([12.0, 6.0]).show(ui, |ui| {
                for p in &self.profiles.list {
                    ui.label(RichText::new(&p.name).strong());
                    ui.label(format!("{}{}", p.account, if p.character.is_empty() { String::new() } else { format!(" · {}", p.character) }));
                    ui.label(if p.has_password() { t("mot de passe enregistré", "password stored") } else { "" });
                    if p.id == self.profiles.default_id {
                        ui.label(RichText::new(t("par défaut", "default")).color(OK_COLOR));
                    } else if ui.small_button(t("Par défaut", "Make default")).clicked() {
                        action = Some((0, p.id.clone()));
                    }
                    ui.horizontal(|ui| {
                        if ui.small_button(t("Modifier", "Edit")).clicked() {
                            action = Some((1, p.id.clone()));
                        }
                        if ui.small_button(t("Supprimer", "Delete")).clicked() {
                            action = Some((2, p.id.clone()));
                        }
                    });
                    ui.end_row();
                }
            });
            if self.profiles.list.is_empty() {
                ui.label(t("Aucun profil pour l'instant.", "No profile yet."));
            }
            match action {
                Some((0, id)) => {
                    self.profiles.default_id = id;
                    self.save_profiles();
                }
                Some((1, id)) => {
                    if let Some(p) = self.profiles.get(&id).cloned() {
                        let store = p.has_password();
                        self.prof_edit = Some(ProfileEdit { p, password: String::new(), store_password: store, clear_password: false, error: None });
                    }
                }
                Some((2, id)) => {
                    if let Some(p) = self.profiles.get(&id) {
                        self.pending = Some(Pending::DeleteProfile(id.clone(), p.name.clone()));
                    }
                }
                _ => {}
            }
            ui.add_space(6.0);
            if ui.button(t("Nouveau profil", "New profile")).clicked() {
                let account = self.acc_selected.clone().unwrap_or_default();
                self.prof_edit = Some(ProfileEdit { p: Profile { id: Profiles::new_id(), name: String::new(), account, ..Default::default() }, password: String::new(), store_password: false, clear_password: false, error: None });
            }
            if let Some(e) = &mut self.prof_edit {
                ui.separator();
                let new = self.profiles.get(&e.p.id).is_none();
                ui.label(RichText::new(if new { t("Nouveau profil", "New profile") } else { t("Modifier le profil", "Edit profile") }).strong());
                egui::Grid::new("profil-edition").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.label(t("Nom du profil", "Profile name"));
                    ui.add(egui::TextEdit::singleline(&mut e.p.name).desired_width(220.0));
                    ui.end_row();
                    ui.label(t("Compte", "Account"));
                    ui.add(egui::TextEdit::singleline(&mut e.p.account).desired_width(220.0));
                    ui.end_row();
                    ui.label(t("Mot de passe", "Password"));
                    ui.vertical(|ui| {
                        ui.checkbox(&mut e.store_password, t("enregistrer le mot de passe (chiffré)", "store the password (encrypted)"));
                        if e.store_password {
                            let hint = if e.p.has_password() && !e.clear_password { t("inchangé (déjà enregistré)", "unchanged (already stored)") } else { "" };
                            ui.add(egui::TextEdit::singleline(&mut e.password).password(true).hint_text(hint).desired_width(220.0));
                        }
                    });
                    ui.end_row();
                    ui.label(t("Personnage préféré", "Preferred character"));
                    ui.add(egui::TextEdit::singleline(&mut e.p.character).hint_text(t("facultatif", "optional")).desired_width(220.0));
                    ui.end_row();
                    ui.label("");
                    ui.checkbox(&mut e.p.enter_world, t("entrer directement en jeu (Benilla, mot de passe enregistré)", "enter the world directly (Benilla, stored password)"));
                    ui.end_row();
                    ui.label(t("Arguments du client", "Client arguments"));
                    ui.add(egui::TextEdit::singleline(&mut e.p.extra_args).hint_text(t("facultatif", "optional")).desired_width(220.0));
                    ui.end_row();
                });
                if let Some(err) = &e.error {
                    ui.colored_label(ERR, err);
                }
                let mut done = None;
                ui.horizontal(|ui| {
                    if ui.button(t("Enregistrer", "Save")).clicked() {
                        done = Some(true);
                    }
                    if ui.button(t("Annuler", "Cancel")).clicked() {
                        done = Some(false);
                    }
                });
                match done {
                    Some(true) => {
                        let e = self.prof_edit.as_mut().unwrap();
                        e.p.name = e.p.name.trim().to_string();
                        e.p.account = e.p.account.trim().to_ascii_uppercase();
                        e.p.character = e.p.character.trim().to_string();
                        if e.p.name.is_empty() {
                            e.error = Some(t("Donnez un nom au profil.", "Give the profile a name.").into());
                        } else if !crate::profiles::valid_account(&e.p.account) {
                            e.error = Some(t("Nom de compte invalide (lettres et chiffres, 32 au plus).", "Invalid account name (letters and digits, 32 at most).").into());
                        } else {
                            let r = if !e.store_password {
                                e.p.password.clear();
                                Ok(())
                            } else if !e.password.is_empty() {
                                e.p.set_password(&e.password)
                            } else {
                                Ok(())
                            };
                            // Le texte saisi est effacé dans tous les cas.
                            e.password.clear();
                            match r {
                                Ok(()) => {
                                    let p = e.p.clone();
                                    self.profiles.upsert(p);
                                    self.save_profiles();
                                    self.prof_edit = None;
                                }
                                Err(err) => e.error = Some(err),
                            }
                        }
                    }
                    Some(false) => self.prof_edit = None,
                    None => {}
                }
            }
        });
        if !open {
            self.show_profiles = false;
            self.prof_edit = None;
        }
    }

    pub(super) fn save_profiles(&mut self) {
        if let Err(e) = self.profiles.save(&self.layout.user_data()) {
            self.settings_notice = Some(format!("profiles.json : {e}"));
        }
    }
}
