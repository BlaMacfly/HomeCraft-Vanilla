//! Réglages de l'utilisateur (UserData/settings.json).

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    /// Dossier du client WoW 1.12.1 (vide = Client/ à la racine)
    pub client_dir: String,
    pub windowed: bool,
    pub ahbot_enabled: bool,
    /// Module playerbots actif (AiPlayerbot.Enabled)
    pub partybots_enabled: bool,
    /// Surcharges de mangosd.conf et aiplayerbot.conf (clé -> valeur), limitées au catalogue `conf_catalog`
    /// (onglets PlayerBots et Monde).
    pub conf: std::collections::BTreeMap<String, String>,
    /// Lancer le jeu via VanillaFixes (correctif des saccades ; injecte VfPatcher.dll au démarrage)
    pub vanillafixes: bool,
    /// Rendu Direct3D 9 traduit en Vulkan par DXVK (d3d9.dll copié dans le dossier du client)
    pub dxvk: bool,
    /// Cartes de déplacement (mmaps) générées automatiquement en arrière-plan après le démarrage du serveur si absentes
    pub auto_mmaps: bool,
    /// Sauvegarde automatique des personnages à chaque fermeture du jeu
    pub auto_backup: bool,
    /// Nombre de sauvegardes automatiques conservées (les plus anciennes sont supprimées)
    pub auto_backup_keep: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            client_dir: String::new(),
            windowed: true, // plus sûr sous Windows 11 (plein écran exclusif D3D9 capricieux) ; modifiable dans Paramètres
            ahbot_enabled: true,
            partybots_enabled: true,
            conf: std::collections::BTreeMap::new(),
            vanillafixes: false,
            dxvk: false,
            auto_mmaps: true,
            auto_backup: true,
            auto_backup_keep: 10,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        let mut s: Settings = std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default();
        // Réglages d'une version VMaNGOS (clés PartyBot.*, RandomBot.*, WowPatch…) : sans objet ici.
        s.conf.retain(|k, _| crate::conf_catalog::is_known(k));
        s
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).map_err(|e| e.to_string())
    }
}
