//! Réglages de l'utilisateur (UserData/settings.json). `#[serde(default)]` partout : un fichier d'une version
//! précédente reste lisible.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Client utilisé pour jouer.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ClientKind {
    /// Détection automatique : 3.4.3 (WotLK Classic, dossier `_classic_`) si présent, sinon 3.3.5a.
    Auto,
    /// World of Warcraft 3.3.5a (build 12340), connexion directe au serveur.
    Wotlk335,
    /// World of Warcraft Classic 3.4.3.54261, via HermesProxy + Arctium WoW Launcher.
    Classic343,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    /// Dossier du client (vide = Client/ à la racine)
    pub client_dir: String,
    pub client_kind: ClientKind,
    pub windowed: bool,
    /// Bots aléatoires qui peuplent le monde (mod-playerbots : AiPlayerbot.MinRandomBots / MaxRandomBots). 0 = aucun.
    pub random_bots: u32,
    /// Hôtel des ventes approvisionné par mod-ah-bot (vendeur).
    pub ahbot_enabled: bool,
    /// mod-ah-bot exige un compte et un personnage « vendeur » existants (identifiants numériques).
    pub ahbot_account: u32,
    pub ahbot_guid: u32,
    /// Surcharges des .conf (clé -> valeur), limitées au catalogue `conf_catalog` (onglets PlayerBots et Monde).
    pub conf: std::collections::BTreeMap<String, String>,
    /// Cartes de déplacement (mmaps) générées automatiquement en arrière-plan après le démarrage du serveur si absentes
    pub auto_mmaps: bool,
    /// Sauvegarde automatique des personnages à chaque fermeture du jeu
    pub auto_backup: bool,
    /// Nombre de sauvegardes automatiques conservées (les plus anciennes sont supprimées)
    pub auto_backup_keep: u32,
    /// Nom du royaume affiché dans la liste des royaumes du client
    pub realm_name: String,
    /// Arguments supplémentaires passés à HermesProxy.exe (ex. surcharges `--Section:Clé=Valeur`)
    pub hermes_args: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            client_dir: String::new(),
            client_kind: ClientKind::Auto,
            windowed: true,
            random_bots: 40,
            ahbot_enabled: false,
            ahbot_account: 0,
            ahbot_guid: 0,
            conf: std::collections::BTreeMap::new(),
            auto_mmaps: true,
            auto_backup: true,
            auto_backup_keep: 10,
            realm_name: crate::paths::REALM_NAME.into(),
            hermes_args: String::new(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).map_err(|e| e.to_string())
    }

    pub fn realm_name(&self) -> &str {
        let n = self.realm_name.trim();
        if n.is_empty() { crate::paths::REALM_NAME } else { n }
    }
}
