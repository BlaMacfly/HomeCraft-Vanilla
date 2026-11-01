//! Réglages « IA des bots » (PlayerBots AI Settings) : fournisseur, modèle, clé, paramètres, test.
//! La clé d'API n'est jamais réaffichée : elle est chiffrée (DPAPI) à l'enregistrement, seul un aperçu masqué reste.

use super::{App, ERR, OK_COLOR, WARN};
use crate::ai_bridge::{AiSettings, Provider};
use crate::i18n::t;
use crate::orchestrator::{Command, Snapshot};
use eframe::egui::{self, RichText};
use std::sync::{Arc, Mutex};

/// Résultat d'une opération en arrière-plan (test du fournisseur, liste des modèles)
pub(super) type Async<T> = Arc<Mutex<Option<Result<T, String>>>>;

pub(super) struct AiUi {
    pub cfg: AiSettings,
    pub key_input: String,
    pub key_preview: String,
    pub models: Async<Vec<String>>,
    pub models_running: bool,
    pub test: Async<String>,
    pub test_running: bool,
    pub notice: Option<String>,
}

impl AiUi {
    pub fn new(layout: &crate::paths::Layout) -> AiUi {
        let cfg = AiSettings::load(layout);
        let key_preview = cfg.key().ok().flatten().map(|k| crate::secret::masked(&k)).unwrap_or_default();
        AiUi { cfg, key_input: String::new(), key_preview, models: Arc::new(Mutex::new(None)), models_running: false, test: Arc::new(Mutex::new(None)), test_running: false, notice: None }
    }
}

impl App {
    pub(super) fn ai_section(&mut self, ui: &mut egui::Ui, _snap: &Snapshot) {
        egui::CollapsingHeader::new(RichText::new(t("IA des bots (modèle de langage)", "Bot AI (language model)")).strong()).default_open(false).show(ui, |ui| self.ai_body(ui));
    }

    fn ai_body(&mut self, ui: &mut egui::Ui) {
        let a = &mut self.ai;
        ui.label(t(
            "Les bots répondent aux vrais joueurs (dire, groupe, chuchotement) avec un modèle de langage. Le serveur ne parle qu'au pont local du launcher, qui appelle le fournisseur : la clé n'est jamais transmise au serveur ni aux invités d'une partie multijoueur. Si le fournisseur ne répond pas, les bots reprennent leurs réponses classiques.",
            "Bots answer real players (say, party, whisper) with a language model. The server only talks to the launcher's local bridge, which calls the provider: the key is never given to the server nor to multiplayer guests. If the provider does not answer, bots fall back to their classic replies.",
        ));
        ui.add_space(4.0);
        egui::Grid::new("ia-reglages").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
            ui.label(t("Activer l'IA", "Enable AI"));
            ui.checkbox(&mut a.cfg.enabled, "");
            ui.end_row();
            ui.label(t("Fournisseur", "Provider"));
            egui::ComboBox::from_id_salt("ia-fournisseur").selected_text(a.cfg.provider.label()).show_ui(ui, |ui| {
                for p in Provider::ALL {
                    if ui.selectable_value(&mut a.cfg.provider, p, p.label()).changed() {
                        a.cfg.base_url.clear();
                        a.cfg.model.clear();
                        *a.models.lock().unwrap() = None;
                    }
                }
            });
            ui.end_row();
            ui.label(t("Adresse (URL)", "Address (URL)"));
            ui.add(egui::TextEdit::singleline(&mut a.cfg.base_url).hint_text(a.cfg.provider.default_url()).desired_width(300.0));
            ui.end_row();
            ui.label(t("Modèle", "Model"));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut a.cfg.model).desired_width(220.0));
                let models = a.models.lock().unwrap().clone();
                if let Some(Ok(list)) = &models {
                    egui::ComboBox::from_id_salt("ia-modeles").selected_text(t("choisir…", "pick…")).show_ui(ui, |ui| {
                        for m in list {
                            if ui.selectable_label(a.cfg.model == *m, m).clicked() {
                                a.cfg.model = m.clone();
                            }
                        }
                    });
                }
                if ui.add_enabled(!a.models_running, egui::Button::new(t("Rechercher les modèles", "Find models"))).on_hover_text(t("Liste les modèles disponibles chez le fournisseur ; rien n'est téléchargé.", "Lists the models available at the provider; nothing is downloaded.")).clicked() {
                    a.models_running = true;
                    let (cfg, out) = (a.cfg.clone(), a.models.clone());
                    let key = typed_or_stored_key(&a.key_input, &cfg);
                    *out.lock().unwrap() = None;
                    std::thread::spawn(move || {
                        let r = crate::ai_bridge::list_models(&cfg, key.as_deref());
                        *out.lock().unwrap() = Some(r);
                    });
                }
            });
            ui.end_row();
            ui.label(t("Clé d'API", "API key"));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut a.key_input).password(true).hint_text(if a.key_preview.is_empty() { t("aucune", "none").to_string() } else { format!("{} {}", t("enregistrée :", "stored:"), a.key_preview) }).desired_width(220.0));
                if !a.cfg.api_key.is_empty() && ui.small_button(t("Effacer la clé", "Remove the key")).clicked() {
                    a.cfg.api_key.clear();
                    a.key_preview.clear();
                }
            });
            ui.end_row();
            ui.label(t("Température", "Temperature"));
            ui.add(egui::Slider::new(&mut a.cfg.temperature, 0.0..=1.5).step_by(0.05));
            ui.end_row();
            ui.label(t("Jetons de réponse", "Response tokens"));
            ui.add(egui::DragValue::new(&mut a.cfg.max_tokens).range(32..=4000));
            ui.end_row();
            ui.label(t("Délai maximal (s)", "Timeout (s)"));
            ui.add(egui::DragValue::new(&mut a.cfg.timeout_secs).range(5..=600));
            ui.end_row();
            ui.label(t("Générations simultanées", "Concurrent generations"));
            ui.add(egui::DragValue::new(&mut a.cfg.concurrency).range(1..=16));
            ui.end_row();
            ui.label(t("Paroles spontanées (%)", "Spontaneous talk (%)"));
            ui.add(egui::Slider::new(&mut a.cfg.spontaneous, 0..=100)).on_hover_text(t("Probabilité qu'un bot parle de lui-même à un PNJ (chaque parole est une génération : attention au coût d'un fournisseur payant).", "Chance that a bot talks to an NPC on its own (each line is a generation: mind the cost of a paid provider)."));
            ui.end_row();
            ui.label(t("Intentions de jeu", "Game intents"));
            ui.checkbox(&mut a.cfg.intents, t("autoriser suivre, rester, attaquer, gestes (liste blanche)", "allow follow, stay, attack, gestures (whitelist)"));
            ui.end_row();
            ui.label(t("Langue des réponses", "Reply language"));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut a.cfg.language, "fr".to_string(), "Français");
                ui.selectable_value(&mut a.cfg.language, "en".to_string(), "English");
            });
            ui.end_row();
            ui.label(t("Personnalité commune", "Shared personality"));
            ui.add(egui::TextEdit::multiline(&mut a.cfg.personality).desired_rows(2).hint_text(t("ex. : Tu es un aventurier chaleureux et un peu taquin.", "e.g.: You are a warm, slightly teasing adventurer.")).desired_width(300.0));
            ui.end_row();
        });
        if a.cfg.provider.local() {
            ui.label(RichText::new(t("Fournisseur local : le modèle doit être déjà téléchargé et chargé dans LM Studio ou Ollama ; HomeCraft ne télécharge jamais de modèle.", "Local provider: the model must already be downloaded and loaded in LM Studio or Ollama; HomeCraft never downloads a model.")).color(WARN));
        }
        if let Some(Err(e)) = a.models.lock().unwrap().as_ref() {
            ui.colored_label(ERR, e);
        }
        if a.models_running && a.models.lock().unwrap().is_some() {
            a.models_running = false;
        }
        ui.add_space(6.0);
        let mut save = false;
        ui.horizontal(|ui| {
            if ui.add_enabled(!a.test_running, egui::Button::new(t("Tester", "Test"))).clicked() {
                a.test_running = true;
                let (cfg, out) = (a.cfg.clone(), a.test.clone());
                let key = typed_or_stored_key(&a.key_input, &cfg);
                *out.lock().unwrap() = None;
                std::thread::spawn(move || {
                    let start = std::time::Instant::now();
                    let sys = crate::ai_bridge::system_prompt(&cfg, "You are Thorin, a level 12 dwarf warrior in World of Warcraft. The human paladin Arthas speaks to you directly.", "");
                    let r = crate::ai_bridge::complete(&cfg, key.as_deref(), &sys, "Arthas: Salut Thorin, tu peux me suivre ?")
                        .map(|raw| format!("{} ms — {}", start.elapsed().as_millis(), crate::ai_bridge::postprocess(&raw, "Thorin", cfg.intents)));
                    *out.lock().unwrap() = Some(r);
                });
            }
            if ui.button(RichText::new(t("Enregistrer", "Save")).strong()).clicked() {
                save = true;
            }
            if a.test_running {
                ui.spinner();
                ui.label(t("génération en cours (le premier appel à un modèle local peut prendre une minute)…", "generating (the first call to a local model can take a minute)…"));
            }
        });
        if a.test_running && a.test.lock().unwrap().is_some() {
            a.test_running = false;
        }
        match a.test.lock().unwrap().as_ref() {
            Some(Ok(r)) => {
                ui.colored_label(OK_COLOR, format!("{} {r}", t("Réponse :", "Reply:")));
            }
            Some(Err(e)) => {
                ui.colored_label(ERR, format!("{} {e}", t("Échec :", "Failure:")));
            }
            None => {}
        }
        if let Some(n) = &a.notice {
            ui.label(n);
        }
        if save {
            self.save_ai();
        }
        let st = crate::ai_bridge::stats();
        if st.running {
            ui.label(format!("{} {} · {} {} · {} {} ms", t("Pont actif — demandes :", "Bridge running — requests:"), st.requests, t("échecs :", "failures:"), st.failures, t("dernière :", "last:"), st.last_latency_ms));
            if let Some(e) = &st.last_error {
                ui.colored_label(WARN, e);
            }
        }
    }

    fn save_ai(&mut self) {
        let a = &mut self.ai;
        if !a.key_input.is_empty() {
            match crate::secret::protect(a.key_input.trim()) {
                Ok(hex) => {
                    a.key_preview = crate::secret::masked(a.key_input.trim());
                    a.cfg.api_key = hex;
                }
                Err(e) => {
                    a.notice = Some(e);
                    return;
                }
            }
            a.key_input.clear();
        }
        a.notice = Some(match a.cfg.save(&self.layout) {
            Ok(()) => t("Réglages de l'IA enregistrés (appliqués au serveur s'il est en marche).", "AI settings saved (applied to the server if it is running).").into(),
            Err(e) => format!("ai-settings.json : {e}"),
        });
        self.send(Command::AiSettingsChanged);
    }
}

/// Clé saisie (pas encore enregistrée) ou clé enregistrée, pour un test.
fn typed_or_stored_key(typed: &str, cfg: &AiSettings) -> Option<String> {
    if !typed.trim().is_empty() {
        return Some(typed.trim().to_string());
    }
    cfg.key().ok().flatten()
}
