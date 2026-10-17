//! Réglages de l'utilisateur (UserData/settings.json).

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AhFaction {
    Auto,
    Alliance,
    Horde,
}

/// Réglages des PlayerBots VMaNGOS (clés PartyBot.* / PlayerBot.* / BattleBot.* de mangosd.conf).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct BotSettings {
    /// 0 = illimité (PartyBot.MaxBots)
    pub max_party_bots: u32,
    /// lève les restrictions (combat, instance, niveau) pour les non-GM (PartyBot.SkipChecks)
    pub skip_checks: bool,
    /// 0 objets de départ, 1 équipement aléatoire adapté, 2 modèle prédéfini (PartyBot.AutoEquip)
    pub auto_equip: u8,
    /// écart de niveau d'objet maximal pour l'équipement aléatoire (PartyBot.RandomGearLevelDifference)
    pub gear_level_diff: u32,
    /// période de réflexion des bots en ms (PlayerBot.UpdateMs) : plus bas = plus réactifs
    pub update_ms: u32,
    /// bots visibles dans /qui (PlayerBot.ShowInWhoList)
    pub show_in_who: bool,
    /// bots de champ de bataille ajoutés automatiquement à l'inscription (BattleBot.AutoJoin)
    pub battlebot_autojoin: bool,
    /// équipement des bots de champ de bataille (BattleBot.AutoEquip)
    pub battlebot_auto_equip: u8,
}

impl Default for BotSettings {
    fn default() -> Self {
        BotSettings { max_party_bots: 0, skip_checks: true, auto_equip: 1, gear_level_diff: 10, update_ms: 1000, show_in_who: true, battlebot_autojoin: true, battlebot_auto_equip: 1 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    /// Dossier du client WoW 1.12.1 (vide = Client/ à la racine)
    pub client_dir: String,
    pub windowed: bool,
    pub ahbot_enabled: bool,
    pub ah_faction: AhFaction,
    pub partybots_enabled: bool,
    pub rate_xp: f32,
    pub bots: BotSettings,
    /// Surcharges de mangosd.conf (clé -> valeur), limitées au catalogue `conf_catalog` (onglets PlayerBots et Monde).
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
            ah_faction: AhFaction::Auto,
            partybots_enabled: true,
            rate_xp: 1.0,
            bots: BotSettings::default(),
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
        std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).map_err(|e| e.to_string())
    }
}
