//! HomeCraft AI Bridge : module PlayerBots ↔ pont local ↔ fournisseur de modèle de langage.
//!
//! Le module playerbots n'appelle jamais le fournisseur : il envoie sa demande (contexte du bot, message du joueur) à ce
//! pont, sur 127.0.0.1 seulement, avec un jeton local. Le pont ajoute la personnalité et les consignes, appelle le
//! fournisseur choisi (clé d'API déchiffrée en mémoire, jamais journalisée), nettoie la réponse et ne laisse passer
//! qu'une intention de jeu de la liste blanche (`[hc:follow]`…), revalidée par le module avant exécution dans le fil du
//! monde. En cas d'échec il répond par une erreur HTTP : le module repasse alors aux réponses classiques des bots.
//! En multijoueur, seul l'hôte (qui fait tourner le serveur) appelle le fournisseur ; aucune clé n'est transmise.

use crate::paths::Layout;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const BRIDGE_PORT: u16 = 15_910;
pub const BRIDGE_PATH: &str = "/homecraft/generate";
/// Intentions que le modèle peut proposer (le module applique la même liste).
pub const INTENTS: [&str; 8] = ["follow", "stay", "attack", "wave", "bow", "cheer", "laugh", "salute"];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAi,
    Anthropic,
    Mistral,
    OpenRouter,
    Ollama,
    #[default]
    LmStudio,
    /// Tout service compatible avec l'API OpenAI (URL à saisir)
    Compatible,
}

impl Provider {
    pub const ALL: [Provider; 7] = [Provider::LmStudio, Provider::Ollama, Provider::OpenAi, Provider::Anthropic, Provider::Mistral, Provider::OpenRouter, Provider::Compatible];
    pub fn label(self) -> &'static str {
        match self {
            Provider::OpenAi => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::Mistral => "Mistral",
            Provider::OpenRouter => "OpenRouter",
            Provider::Ollama => "Ollama (local)",
            Provider::LmStudio => "LM Studio (local)",
            Provider::Compatible => "Compatible OpenAI",
        }
    }
    pub fn default_url(self) -> &'static str {
        match self {
            Provider::OpenAi => "https://api.openai.com/v1",
            Provider::Anthropic => "https://api.anthropic.com/v1",
            Provider::Mistral => "https://api.mistral.ai/v1",
            Provider::OpenRouter => "https://openrouter.ai/api/v1",
            Provider::Ollama => "http://127.0.0.1:11434/v1",
            Provider::LmStudio => "http://127.0.0.1:1234/v1",
            Provider::Compatible => "",
        }
    }
    pub fn local(self) -> bool {
        matches!(self, Provider::Ollama | Provider::LmStudio)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AiSettings {
    pub enabled: bool,
    pub provider: Provider,
    pub model: String,
    /// vide : URL par défaut du fournisseur
    pub base_url: String,
    /// Clé d'API chiffrée DPAPI (hexadécimal) ; vide : aucune
    pub api_key: String,
    pub temperature: f32,
    /// Jetons de réponse (les modèles à raisonnement en consomment une partie avant de répondre)
    pub max_tokens: u32,
    pub timeout_secs: u32,
    /// Générations simultanées au plus
    pub concurrency: u32,
    /// Fréquence (0-100) des paroles spontanées des bots (discussions avec les PNJ)
    pub spontaneous: u32,
    /// Personnalité commune à tous les bots (en plus de llm_character_card.txt du module)
    pub personality: String,
    /// Intentions de jeu autorisées (suivre, rester, attaquer, gestes)
    pub intents: bool,
    /// Langue des réponses : "fr" ou "en"
    pub language: String,
}

impl Default for AiSettings {
    fn default() -> Self {
        AiSettings { enabled: false, provider: Provider::LmStudio, model: String::new(), base_url: String::new(), api_key: String::new(), temperature: 0.8, max_tokens: 300, timeout_secs: 45, concurrency: 2, spontaneous: 0, personality: String::new(), intents: true, language: "fr".into() }
    }
}

impl AiSettings {
    pub fn path(layout: &Layout) -> std::path::PathBuf {
        layout.user_data().join("ai-settings.json")
    }
    pub fn load(layout: &Layout) -> AiSettings {
        std::fs::read_to_string(Self::path(layout)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }
    pub fn save(&self, layout: &Layout) -> Result<(), String> {
        std::fs::create_dir_all(layout.user_data()).map_err(|e| e.to_string())?;
        std::fs::write(Self::path(layout), serde_json::to_string_pretty(self).unwrap()).map_err(|e| e.to_string())
    }
    pub fn url(&self) -> String {
        let u = if self.base_url.trim().is_empty() { self.provider.default_url().to_string() } else { self.base_url.trim().to_string() };
        u.trim_end_matches('/').to_string()
    }
    pub fn key(&self) -> Result<Option<String>, String> {
        if self.api_key.is_empty() {
            return Ok(None);
        }
        crate::secret::unprotect(&self.api_key).map(Some)
    }
}

/// Jeton local entre mangosd et le pont (pas une clé de fournisseur) : UserData\ai-bridge-token, créé une fois.
pub fn bridge_token(layout: &Layout) -> String {
    let p = layout.user_data().join("ai-bridge-token");
    if let Ok(t) = std::fs::read_to_string(&p) {
        let t = t.trim().to_string();
        if t.len() >= 24 {
            return t;
        }
    }
    let t = crate::secret::random_token(40);
    let _ = std::fs::create_dir_all(layout.user_data());
    let _ = std::fs::write(&p, &t);
    t
}

/// Clés de aiplayerbot.conf écrites par `server::generate_configs`.
pub fn conf_values(layout: &Layout) -> Vec<(&'static str, String)> {
    let s = AiSettings::load(layout);
    if !s.enabled {
        return vec![("AiPlayerbot.LLMEnabled", "0".into())];
    }
    vec![
        // 2 : stratégie « ai chat » pour tous les bots ; ils répondent aux vrais joueurs (dire, groupe, chuchotement).
        ("AiPlayerbot.LLMEnabled", "2".into()),
        ("AiPlayerbot.LLMApiEndpoint", format!("http://127.0.0.1:{BRIDGE_PORT}{BRIDGE_PATH}")),
        ("AiPlayerbot.LLMApiKey", bridge_token(layout)),
        ("AiPlayerbot.LLMApiJson", r#"{"system": "<pre prompt>", "context": "<context>", "prompt": "<prompt>", "bot": "<bot name>"}"#.into()),
        ("AiPlayerbot.LLMResponseStartPattern", r#"("text":\s*")"#.into()),
        ("AiPlayerbot.LLMResponseEndPattern", r#"(")"#.into()),
        ("AiPlayerbot.LLMResponseDeletePattern", r"(\\n)".into()),
        ("AiPlayerbot.LLMGenerationTimeout", (s.timeout_secs + 15).to_string()),
        ("AiPlayerbot.LLMMaxSimultaniousGenerations", (s.concurrency.max(1) * 4).to_string()),
        ("AiPlayerbot.LLMBotToBotChatChance", "0".into()),
        ("AiPlayerbot.LLMRpgAIChatChance", s.spontaneous.min(100).to_string()),
    ]
}

// ------------------------------------------------------------------------------------------------ fournisseurs

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(timeout)).http_status_as_error(false).build().into()
}

fn read_json(resp: ureq::http::Response<ureq::Body>) -> Result<Value, String> {
    let status = resp.status().as_u16();
    let text = resp.into_body().read_to_string().map_err(|e| format!("réponse illisible : {e}"))?;
    if status != 200 {
        // Message d'erreur du fournisseur (jamais d'en-têtes ni de clé)
        let msg = serde_json::from_str::<Value>(&text).ok().and_then(|v| v["error"]["message"].as_str().or(v["error"].as_str()).map(String::from)).unwrap_or_else(|| text.chars().take(160).collect());
        return Err(format!("HTTP {status} : {msg}"));
    }
    serde_json::from_str(&text).map_err(|e| format!("réponse non JSON : {e}"))
}

/// Une génération : consignes (system) et message du joueur (user). Renvoie le texte brut du modèle.
pub fn complete(s: &AiSettings, key: Option<&str>, system: &str, user: &str) -> Result<String, String> {
    let url = s.url();
    if url.is_empty() {
        return Err("adresse du fournisseur non renseignée".into());
    }
    if s.model.trim().is_empty() {
        return Err("aucun modèle choisi".into());
    }
    let a = agent(Duration::from_secs(s.timeout_secs.clamp(5, 600) as u64));
    let r = if s.provider == Provider::Anthropic {
        let body = json!({"model": s.model, "max_tokens": s.max_tokens.max(16), "temperature": s.temperature, "system": system, "messages": [{"role": "user", "content": user}]});
        let mut req = a.post(&format!("{url}/messages")).header("anthropic-version", "2023-06-01").header("content-type", "application/json");
        if let Some(k) = key {
            req = req.header("x-api-key", k);
        }
        req.send(body.to_string())
    } else {
        let body = json!({"model": s.model, "max_tokens": s.max_tokens.max(16), "temperature": s.temperature, "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}]});
        let mut req = a.post(&format!("{url}/chat/completions")).header("content-type", "application/json");
        if let Some(k) = key {
            req = req.header("authorization", &format!("Bearer {k}"));
        }
        if s.provider == Provider::OpenRouter {
            req = req.header("x-title", "HomeCraft Edition Vanilla");
        }
        req.send(body.to_string())
    };
    let v = read_json(r.map_err(|e| format!("fournisseur injoignable : {e}"))?)?;
    let text = if s.provider == Provider::Anthropic {
        v["content"].as_array().and_then(|a| a.iter().find_map(|c| c["text"].as_str())).map(String::from)
    } else {
        v["choices"][0]["message"]["content"].as_str().map(String::from)
    };
    text.filter(|t| !t.trim().is_empty()).ok_or_else(|| "réponse vide du modèle (jetons de réponse insuffisants ?)".into())
}

/// Modèles proposés par le fournisseur (liste locale pour LM Studio / Ollama) : rien n'est téléchargé.
pub fn list_models(s: &AiSettings, key: Option<&str>) -> Result<Vec<String>, String> {
    let url = s.url();
    if url.is_empty() {
        return Err("adresse du fournisseur non renseignée".into());
    }
    let a = agent(Duration::from_secs(15));
    let mut req = a.get(&format!("{url}/models"));
    if let Some(k) = key {
        req = if s.provider == Provider::Anthropic { req.header("x-api-key", k).header("anthropic-version", "2023-06-01") } else { req.header("authorization", &format!("Bearer {k}")) };
    }
    let v = read_json(req.call().map_err(|e| format!("fournisseur injoignable : {e}"))?)?;
    let mut models: Vec<String> = v["data"].as_array().map(|a| a.iter().filter_map(|m| m["id"].as_str().map(String::from)).collect()).unwrap_or_default();
    // Modèles d'embedding : inutilisables pour dialoguer.
    models.retain(|m| !m.to_ascii_lowercase().contains("embed"));
    models.sort();
    Ok(models)
}

// ------------------------------------------------------------------------------------------------ consignes et réponse

pub fn system_prompt(s: &AiSettings, module_system: &str, context: &str) -> String {
    let mut out = String::new();
    out.push_str(module_system.trim());
    if !s.personality.trim().is_empty() {
        out.push_str("\n");
        out.push_str(s.personality.trim());
    }
    if s.language == "en" {
        out.push_str("\nAlways answer in English, in one or two short sentences (under 200 characters), without quotation marks and without prefixing your name.");
    } else {
        out.push_str("\nRéponds toujours en français, en une ou deux phrases courtes (moins de 200 caractères), sans guillemets et sans écrire ton nom devant.");
    }
    if s.intents {
        out.push_str(if s.language == "en" {
            "\nIf the player clearly asks you to act, end your answer with ONE tag among: [hc:follow] (follow them), [hc:stay] (stay here), [hc:attack] (attack their target), [hc:wave] [hc:bow] [hc:cheer] [hc:laugh] [hc:salute] (gestures). Never invent another tag; otherwise add none."
        } else {
            "\nSi le joueur te demande clairement d'agir, termine ta réponse par UNE seule balise parmi : [hc:follow] (le suivre), [hc:stay] (rester ici), [hc:attack] (attaquer sa cible), [hc:wave] [hc:bow] [hc:cheer] [hc:laugh] [hc:salute] (gestes). N'invente jamais d'autre balise ; sinon n'en mets aucune."
        });
    }
    if !context.trim().is_empty() {
        out.push_str(if s.language == "en" { "\nEarlier conversation: " } else { "\nConversation précédente : " });
        out.push_str(context.trim());
    }
    out
}

/// Nettoie la réponse du modèle pour le module : une ligne, sans guillemets ni balises inventées, ponctuation finale
/// (le module ne garde que des phrases terminées), et au plus une intention de la liste blanche, en fin de texte.
pub fn postprocess(raw: &str, bot_name: &str, intents: bool) -> String {
    let mut text = raw.to_string();
    // Raisonnement apparent de certains modèles locaux
    while let (Some(a), Some(b)) = (text.find("<think>"), text.find("</think>")) {
        if b < a {
            break;
        }
        text.replace_range(a..b + "</think>".len(), " ");
    }
    let mut intent: Option<String> = None;
    let mut out = String::new();
    let mut rest = text.as_str();
    while let Some(i) = rest.find('[') {
        out.push_str(&rest[..i]);
        match rest[i..].find(']') {
            Some(j) => {
                let tag = &rest[i + 1..i + j];
                if let Some(name) = tag.trim().strip_prefix("hc:") {
                    let name = name.trim().to_ascii_lowercase();
                    if intent.is_none() && INTENTS.contains(&name.as_str()) {
                        intent = Some(name);
                    }
                }
                rest = &rest[i + j + 1..];
            }
            None => {
                rest = "";
            }
        }
    }
    out.push_str(rest);
    let mut s: String = out.replace(['"', '\\', '*', '\u{201c}', '\u{201d}'], "").split_whitespace().collect::<Vec<_>>().join(" ");
    if !bot_name.is_empty() {
        if let Some(r) = s.strip_prefix(&format!("{bot_name}:")) {
            s = r.trim().to_string();
        }
    }
    if s.chars().count() > 240 {
        let cut: String = s.chars().take(240).collect();
        s = match cut.rfind(['.', '!', '?']) {
            Some(p) if p > 40 => cut[..=p].to_string(),
            _ => cut,
        };
    }
    let s = s.trim().to_string();
    let mut s = if s.is_empty() || s.ends_with(['.', '!', '?']) { s } else { format!("{s}.") };
    if let (true, Some(i)) = (intents, intent) {
        if s.is_empty() {
            s = "D'accord.".into();
        }
        s.push_str(&format!(" [hc:{i}]"));
    }
    s
}

// ------------------------------------------------------------------------------------------------ serveur local

#[derive(Clone, Debug, Default)]
pub struct BridgeStats {
    pub running: bool,
    pub requests: u64,
    pub failures: u64,
    pub last_latency_ms: u64,
    pub last_error: Option<String>,
}

struct Running {
    stop: Arc<AtomicBool>,
    threads: Vec<std::thread::JoinHandle<()>>,
}

static BRIDGE: Mutex<Option<Running>> = Mutex::new(None);
static STATS: Mutex<BridgeStats> = Mutex::new(BridgeStats { running: false, requests: 0, failures: 0, last_latency_ms: 0, last_error: None });

pub fn stats() -> BridgeStats {
    STATS.lock().unwrap().clone()
}

fn log_line(layout: &Layout, line: &str) {
    let _ = std::fs::create_dir_all(layout.logs());
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(layout.logs().join("ai-bridge.log")) {
        let _ = writeln!(f, "{} {line}", crate::util::time_hms());
    }
}

pub fn stop() {
    if let Some(r) = BRIDGE.lock().unwrap().take() {
        r.stop.store(true, Ordering::SeqCst);
        for t in r.threads {
            let _ = t.join();
        }
    }
    STATS.lock().unwrap().running = false;
}

/// (Re)démarre le pont avec les réglages enregistrés ; ne fait rien si l'IA est désactivée.
pub fn restart(layout: &Layout) -> Result<bool, String> {
    stop();
    let s = AiSettings::load(layout);
    if !s.enabled {
        return Ok(false);
    }
    let key = s.key()?;
    let server = Arc::new(tiny_http::Server::http(("127.0.0.1", BRIDGE_PORT)).map_err(|e| format!("pont IA : port {BRIDGE_PORT} indisponible ({e})"))?);
    let token = Arc::new(bridge_token(layout));
    let stop = Arc::new(AtomicBool::new(false));
    let settings = Arc::new(s.clone());
    let key = Arc::new(key);
    let seq = Arc::new(AtomicU64::new(0));
    let mut threads = Vec::new();
    for _ in 0..s.concurrency.clamp(1, 16) {
        let (server, token, stop, settings, key, layout, seq) = (server.clone(), token.clone(), stop.clone(), settings.clone(), key.clone(), layout.clone(), seq.clone());
        threads.push(std::thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) {
                let Ok(Some(req)) = server.recv_timeout(Duration::from_millis(300)) else { continue };
                handle(req, &settings, key.as_deref(), &token, &layout, seq.fetch_add(1, Ordering::SeqCst) + 1);
            }
        }));
    }
    *BRIDGE.lock().unwrap() = Some(Running { stop, threads });
    *STATS.lock().unwrap() = BridgeStats { running: true, ..Default::default() };
    log_line(layout, &format!("pont IA démarré sur 127.0.0.1:{BRIDGE_PORT} — {} / {} ({} génération(s) simultanée(s), clé {})", s.provider.label(), s.model, s.concurrency.clamp(1, 16), if s.api_key.is_empty() { "aucune" } else { "enregistrée" }));
    Ok(true)
}

fn respond(req: tiny_http::Request, status: u16, body: String) {
    let resp = tiny_http::Response::from_string(body).with_status_code(status).with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap());
    let _ = req.respond(resp);
}

fn handle(mut req: tiny_http::Request, s: &AiSettings, key: Option<&str>, token: &str, layout: &Layout, n: u64) {
    if req.url() == "/homecraft/health" {
        return respond(req, 200, "{\"ok\":true}".into());
    }
    if req.method() != &tiny_http::Method::Post || req.url() != BRIDGE_PATH {
        return respond(req, 404, String::new());
    }
    let auth_ok = req.headers().iter().any(|h| h.field.equiv("Authorization") && h.value.as_str() == format!("Bearer {token}"));
    if !auth_ok {
        log_line(layout, &format!("#{n} refusé : jeton local absent ou faux"));
        return respond(req, 401, String::new());
    }
    let mut body = String::new();
    if req.as_reader().take(256 * 1024).read_to_string(&mut body).is_err() {
        return respond(req, 400, String::new());
    }
    let v: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            log_line(layout, &format!("#{n} demande illisible du module : {e}"));
            return respond(req, 400, String::new());
        }
    };
    let field = |k: &str| v[k].as_str().unwrap_or("").to_string();
    let bot = field("bot");
    let system = system_prompt(s, &field("system"), &field("context"));
    let start = Instant::now();
    let result = complete(s, key, &system, &field("prompt"));
    let ms = start.elapsed().as_millis() as u64;
    {
        let mut st = STATS.lock().unwrap();
        st.requests += 1;
        st.last_latency_ms = ms;
        match &result {
            Ok(_) => st.last_error = None,
            Err(e) => {
                st.failures += 1;
                st.last_error = Some(e.clone());
            }
        }
    }
    match result {
        Ok(raw) => {
            let text = postprocess(&raw, &bot, s.intents);
            let intent = text.rfind("[hc:").map(|i| text[i..].to_string()).unwrap_or_default();
            log_line(layout, &format!("#{n} {bot} : réponse en {ms} ms ({} caractères){}", text.chars().count(), if intent.is_empty() { String::new() } else { format!(", intention {intent}") }));
            respond(req, 200, json!({"results": [{"text": text}]}).to_string())
        }
        Err(e) => {
            log_line(layout, &format!("#{n} {bot} : échec en {ms} ms — {e}"));
            respond(req, 502, String::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nettoyage_et_intentions() {
        assert_eq!(postprocess("Bien sûr, Arthas. [hc:follow]", "Thorin", true), "Bien sûr, Arthas. [hc:follow]");
        // Balises inventées ou hors liste blanche supprimées, une seule intention gardée
        assert_eq!(postprocess("Je lance le sort [hc:cast_pyroblast] puis [hc:stay] et [hc:follow]", "", true), "Je lance le sort puis et. [hc:stay]");
        assert_eq!(postprocess("Thorin: \"Salut\" *sourit*", "Thorin", true), "Salut sourit.");
        assert_eq!(postprocess("<think>raisonnement</think>D'accord !", "", true), "D'accord !");
        // Intentions désactivées : jamais de balise transmise
        assert_eq!(postprocess("Je te suis. [hc:follow]", "", false), "Je te suis.");
        assert_eq!(postprocess("[hc:attack]", "", true), "D'accord. [hc:attack]");
        assert!(postprocess(&"mot ".repeat(200), "", true).chars().count() <= 241);
    }

    #[test]
    fn consignes() {
        let s = AiSettings { personality: "Tu es bourru.".into(), ..Default::default() };
        let p = system_prompt(&s, "You are Thorin.", "Arthas: salut");
        assert!(p.contains("You are Thorin.") && p.contains("bourru") && p.contains("français") && p.contains("[hc:follow]") && p.contains("Arthas: salut"));
    }
}
