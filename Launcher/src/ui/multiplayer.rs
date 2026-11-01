//! Onglet « HomeCraft Multijoueur » : héberger une partie, inviter, rejoindre, joueurs, révocation.

use super::{App, ERR, OK_COLOR, WARN};
use crate::i18n::t;
use crate::network::{GuestLink, NetCmd, Role};
use crate::orchestrator::{Command, Snapshot};
use eframe::egui::{self, Color32, RichText};

impl App {
    pub(super) fn multiplayer(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let st = self.net.status.lock().unwrap().clone();
        ui.label(RichText::new(t("HomeCraft Multijoueur", "HomeCraft Multiplayer")).size(20.0).strong());
        ui.label(t(
            "Partie privée entre amis, directement entre votre launcher et le leur, par une liaison chiffrée (protocole Noise) : seuls le serveur d'authentification et le serveur de jeu de l'hôte sont accessibles aux invités, jamais la base de données ni la console.",
            "Private game between friends, directly between your launcher and theirs, over an encrypted link (Noise protocol): guests can only reach the host's login and game servers, never the database or the console.",
        ));
        ui.add_space(6.0);
        if let Some(e) = &st.error {
            ui.colored_label(ERR, e);
        }
        if !st.message.is_empty() {
            ui.label(&st.message);
        }
        if st.busy {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t("Ouverture du port sur la box (UPnP)…", "Opening the port on the router (UPnP)…"));
            });
            return;
        }
        match st.role {
            Role::Off => self.mp_idle(ui, snap),
            Role::Host => self.mp_host(ui, snap, &st),
            Role::Guest => self.mp_guest(ui, snap, &st),
        }
        ui.add_space(10.0);
        egui::CollapsingHeader::new(t("Réglages du réseau", "Network settings")).show(ui, |ui| self.mp_settings(ui, st.role == Role::Off));
    }

    fn mp_idle(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.add_space(6.0);
        ui.columns(2, |cols| {
            let ui = &mut cols[0];
            ui.label(RichText::new(t("Héberger une partie", "Host a game")).strong());
            ui.label(t("Votre serveur HomeCraft accueille vos amis. Il doit être démarré.", "Your HomeCraft server welcomes your friends. It must be running."));
            let up = snap.state.services_up();
            if !up {
                ui.colored_label(WARN, t("Serveur arrêté : démarrez-le depuis l'accueil.", "Server stopped: start it from the home tab."));
            }
            if ui.add_enabled(up, egui::Button::new(t("Ouvrir ma partie", "Open my game"))).clicked() {
                self.net.send(NetCmd::Host);
            }
            let ui = &mut cols[1];
            ui.label(RichText::new(t("Rejoindre une partie", "Join a game")).strong());
            ui.label(t("Collez le code d'invitation reçu de l'hôte.", "Paste the invitation code you received from the host."));
            ui.add(egui::TextEdit::multiline(&mut self.net_code).desired_rows(3).desired_width(f32::INFINITY).hint_text("HC2-…"));
            let local_up = snap.state.services_up() || snap.state.is_busy();
            if local_up {
                ui.colored_label(WARN, t("Arrêtez d'abord votre propre serveur (mêmes ports).", "Stop your own server first (same ports)."));
            }
            if ui.add_enabled(!local_up && !self.net_code.trim().is_empty(), egui::Button::new(t("Rejoindre", "Join"))).clicked() {
                self.net.send(NetCmd::Join(std::mem::take(&mut self.net_code)));
            }
        });
    }

    fn mp_host(&mut self, ui: &mut egui::Ui, snap: &Snapshot, st: &crate::network::NetStatus) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.colored_label(OK_COLOR, RichText::new(format!("{} · port {}", t("Partie ouverte", "Game open"), st.listen_port)).strong());
            if ui.button(t("Fermer la partie", "Close the game")).clicked() {
                self.net.send(NetCmd::Leave);
            }
        });
        match &st.upnp {
            Some(Ok(ip)) => {
                ui.colored_label(OK_COLOR, format!("{} {ip}", t("Port ouvert automatiquement sur la box (UPnP) ; adresse publique :", "Port opened automatically on the router (UPnP); public address:")));
            }
            Some(Err(e)) => {
                ui.colored_label(WARN, format!(
                    "{} {e}. {}",
                    t("Ouverture automatique du port impossible :", "Automatic port opening failed:"),
                    t("Pour des invités hors de votre réseau local, redirigez le port ci-dessus (TCP) vers ce PC dans la configuration de votre box.", "For guests outside your local network, forward the port above (TCP) to this PC in your router's settings.")
                ));
            }
            None => {
                ui.label(t("Ouverture automatique du port désactivée : pour des invités hors de votre réseau local, redirigez le port ci-dessus (TCP) vers ce PC.", "Automatic port opening disabled: for guests outside your local network, forward the port above (TCP) to this PC."));
            }
        }
        if !snap.state.services_up() {
            ui.colored_label(WARN, t("Votre serveur est arrêté : les invités ne peuvent pas se connecter.", "Your server is stopped: guests cannot connect."));
        }
        ui.add_space(8.0);
        ui.label(RichText::new(t("Inviter un joueur", "Invite a player")).strong());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.net_label).hint_text(t("nom de l'invité", "guest name")).desired_width(180.0));
            if ui.button(t("Créer une invitation", "Create an invitation")).clicked() {
                self.net.send(NetCmd::Invite(std::mem::take(&mut self.net_label)));
            }
        });
        if let Some((label, code)) = &st.last_code {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(format!("{} {label} ({} {} h) :", t("Code pour", "Code for"), t("valable", "valid for"), self.net_cfg.invite_hours));
                let mut c = code.clone();
                ui.add(egui::TextEdit::multiline(&mut c).desired_rows(2).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace));
                ui.horizontal(|ui| {
                    if ui.button(t("Copier le code", "Copy the code")).clicked() {
                        ui.ctx().copy_text(code.clone());
                    }
                    ui.label(RichText::new(t("À transmettre en privé : ce code donne accès à votre partie jusqu'à son expiration ou sa révocation.", "Share it privately: this code gives access to your game until it expires or is revoked.")).color(WARN));
                });
            });
        }
        // Perçage de NAT : l'invité qui ne peut pas joindre ce PC directement renvoie un code de réponse.
        ui.add_space(8.0);
        ui.label(RichText::new(t("Code de réponse d'un invité", "Reply code from a guest")).strong());
        if st.nat_ready {
            ui.label(t(
                "Si un invité reste « à la recherche de l'hôte », il vous envoie le code de réponse affiché dans son launcher : collez-le ici. Les deux launchers se relient alors sans aucune redirection de port.",
                "If a guest stays \"looking for the host\", they send you the reply code shown in their launcher: paste it here. Both launchers then link up without any port forwarding.",
            ));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.net_reply).hint_text("HCR-…").desired_width(360.0));
                if ui.add_enabled(!self.net_reply.trim().is_empty(), egui::Button::new(t("Relier", "Link"))).clicked() {
                    self.net.send(NetCmd::Reply(std::mem::take(&mut self.net_reply)));
                }
            });
        } else {
            ui.colored_label(WARN, t("Perçage de NAT indisponible (adresse publique introuvable) : seuls les invités qui joignent ce PC directement peuvent se relier.", "NAT traversal unavailable (public address not found): only guests that reach this PC directly can link."));
        }
        ui.add_space(8.0);
        ui.label(RichText::new(format!("{} ({} {})", t("Joueurs reliés", "Linked players"), snap.online, t("personnage(s) en jeu sur le serveur", "character(s) in game on the server"))).strong());
        if st.peers.is_empty() {
            ui.label(t("aucun invité relié", "no guest linked"));
        }
        egui::Grid::new("pairs").num_columns(4).striped(true).show(ui, |ui| {
            for p in &st.peers {
                ui.label(&p.name);
                ui.label(format!("{} {}", t("invitation", "invitation"), p.invite));
                ui.label(&p.address);
                ui.label(p.latency_ms.map(|l| format!("{l:.0} ms")).unwrap_or_default());
                ui.end_row();
            }
        });
        if !st.invites.is_empty() {
            ui.add_space(8.0);
            ui.label(RichText::new(t("Invitations", "Invitations")).strong());
            let now = crate::network::now_unix();
            egui::Grid::new("invitations").num_columns(3).striped(true).show(ui, |ui| {
                for inv in &st.invites {
                    ui.label(&inv.label);
                    let state = if inv.revoked {
                        RichText::new(t("révoquée", "revoked")).color(ERR)
                    } else if inv.expires <= now {
                        RichText::new(t("expirée", "expired")).color(WARN)
                    } else {
                        RichText::new(format!("{} {} min", t("valable encore", "valid for"), (inv.expires - now) / 60)).color(OK_COLOR)
                    };
                    ui.label(state);
                    if !inv.revoked && ui.button(t("Révoquer", "Revoke")).on_hover_text(t("Déconnecte ce joueur et invalide son code", "Disconnects this player and invalidates the code")).clicked() {
                        self.net.send(NetCmd::Revoke(inv.id.clone()));
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            });
        }
    }

    fn mp_guest(&mut self, ui: &mut egui::Ui, snap: &Snapshot, st: &crate::network::NetStatus) {
        ui.add_space(4.0);
        let connected = st.guest_link == GuestLink::Connected;
        let (color, text) = match st.guest_link {
            GuestLink::Connected => (OK_COLOR, t("Relié à l'hôte", "Linked to the host")),
            GuestLink::Searching => (WARN, t("Recherche de l'hôte…", "Looking for the host…")),
            GuestLink::Refused => (ERR, t("Invitation refusée par l'hôte", "Invitation refused by the host")),
        };
        ui.horizontal(|ui| {
            ui.colored_label(color, RichText::new(text).strong());
            if ui.button(t("Quitter la partie", "Leave the game")).clicked() {
                self.net.send(NetCmd::Leave);
            }
        });
        if !connected {
            if let Some(code) = &st.reply_code {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(t(
                        "Si la liaison ne s'établit pas d'elle-même, envoyez ce code de réponse à l'hôte : il le colle dans son onglet Multijoueur, et vos deux launchers se relient sans redirection de port.",
                        "If the link does not come up on its own, send this reply code to the host: they paste it in their Multiplayer tab, and both launchers link up without port forwarding.",
                    ));
                    let mut c = code.clone();
                    ui.add(egui::TextEdit::multiline(&mut c).desired_rows(2).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace));
                    if ui.button(t("Copier le code de réponse", "Copy the reply code")).clicked() {
                        ui.ctx().copy_text(code.clone());
                    }
                });
            }
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            let ok = connected && snap.client.is_ok();
            if ui.add_enabled(ok, egui::Button::new(RichText::new(t("JOUER SUR LA PARTIE DE L'HÔTE", "PLAY ON THE HOST'S GAME")).size(18.0).strong()).min_size(egui::vec2(260.0, 40.0))).clicked() {
                self.send(Command::PlayGuest(None));
            }
            for p in self.profiles.list.clone() {
                if ui.add_enabled(ok, egui::Button::new(format!("{} {}", t("Jouer :", "Play:"), p.name))).clicked() {
                    self.send(Command::PlayGuest(Some(p)));
                }
            }
        });
        ui.label(t("Utilisez le compte que l'hôte a créé pour vous sur son serveur.", "Use the account the host created for you on their server."));
        if !snap.games.is_empty() {
            ui.label(format!("{} {}", t("Clients ouverts :", "Open clients:"), snap.games.iter().map(|g| g.1.clone()).collect::<Vec<_>>().join(", ")));
        }
    }

    fn mp_settings(&mut self, ui: &mut egui::Ui, editable: bool) {
        if !editable {
            ui.label(RichText::new(t("Fermez la partie pour modifier ces réglages.", "Close the game to change these settings.")).color(WARN));
        }
        ui.add_enabled_ui(editable, |ui| {
            egui::Grid::new("reseau-reglages").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
                ui.label(t("Votre nom sur le réseau", "Your name on the network"));
                ui.add(egui::TextEdit::singleline(&mut self.net_cfg.player_name).desired_width(200.0));
                ui.end_row();
                ui.label(t("Port d'écoute (hôte)", "Listening port (host)"));
                ui.add(egui::DragValue::new(&mut self.net_cfg.listen_port).range(1024..=65535));
                ui.end_row();
                ui.label(t("Adresse publique annoncée", "Announced public address"));
                ui.add(egui::TextEdit::singleline(&mut self.net_cfg.public_address).hint_text(t("vide : donnée par la box (UPnP)", "empty: given by the router (UPnP)")).desired_width(260.0));
                ui.end_row();
                ui.label(t("Validité des invitations (heures)", "Invitation validity (hours)"));
                ui.add(egui::DragValue::new(&mut self.net_cfg.invite_hours).range(1..=720));
                ui.end_row();
                ui.label(t("Ouverture automatique du port (UPnP)", "Automatic port opening (UPnP)"));
                ui.checkbox(&mut self.net_cfg.upnp, "");
                ui.end_row();
            });
            ui.label(RichText::new(t(
                "La liaison est directe entre l'hôte et chaque invité : aucun serveur intermédiaire n'est utilisé. Windows peut demander d'autoriser HomeCraft dans le pare-feu à la première ouverture d'une partie : acceptez pour les réseaux privés.",
                "The link is direct between the host and each guest: no intermediate server is used. Windows may ask to allow HomeCraft through the firewall the first time a game is opened: accept for private networks.",
            )).color(Color32::GRAY));
            if ui.button(t("Enregistrer", "Save")).clicked() {
                self.net.send(NetCmd::Settings(self.net_cfg.clone()));
            }
        });
    }
}
