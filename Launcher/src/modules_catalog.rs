//! Catalogue des modules AzerothCore embarqués dans le cœur Sylvania (onglet Modules) : pour chacun, le fichier de
//! configuration dans `Server/acore/configs/modules/` et la clé qui l'active. Les modules sans fichier de
//! configuration (`mod-arac`, `mod-npc-morph`) sont toujours actifs. Clés et fichiers relevés le 2026-10-16 dans
//! les modèles `.conf.dist` du cœur compilé.
//!
//! L'état d'un module est lu dans son `.conf` (pas dans settings.json) : le générateur de configurations repart
//! du `.conf` existant, ce qui conserve aussi les modifications faites à la main via le bouton « Ouvrir ».

#[derive(Clone, Copy)]
pub struct ModuleDef {
    /// Nom du dossier du module dans le cœur (identifiant stable)
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// Fichier dans `configs/modules/` (sans `.dist`), None si le module n'a pas de configuration
    pub conf: Option<&'static str>,
    /// Clé d'activation dans ce fichier et valeurs (activé, désactivé)
    pub enable: Option<(&'static str, &'static str, &'static str)>,
}

const ON_OFF: (&str, &str) = ("1", "0");

const fn m(id: &'static str, label: &'static str, description: &'static str, conf: &'static str, key: &'static str) -> ModuleDef {
    ModuleDef { id, label, description, conf: Some(conf), enable: Some((key, ON_OFF.0, ON_OFF.1)) }
}

pub const MODULES: &[ModuleDef] = &[
    m("mod-1v1-arena", "Arène 1 contre 1", "File d'arène en duel (PNJ d'inscription, commande .1v1).", "1v1arena.conf", "Arena1v1.Enable"),
    ModuleDef { id: "mod-ah-bot", label: "Hôtel des ventes approvisionné (AHBot)", description: "Vendeur automatique à l'hôtel des ventes (prix de marché Sylvania, stock garanti). Activation et vendeur dans l'onglet Paramètres.", conf: Some("mod_ahbot.conf"), enable: None },
    ModuleDef { id: "mod-ale", label: "Moteur de scripts Lua (ALE / Eluna)", description: "Exécute les scripts Lua du dossier lua_scripts.", conf: Some("mod_ale.conf"), enable: Some(("ALE.Enabled", "true", "false")) },
    m("mod-anticheat", "Anticheat", "Détection de triche (vitesse, vol, traversée de murs). Sans objet en solo, mais inoffensif.", "Anticheat.conf", "Anticheat.Enabled"),
    ModuleDef { id: "mod-arac", label: "Toutes races, toutes classes (ARAC)", description: "Lève les restrictions race/classe à la création de personnage (DBC côté client nécessaire).", conf: None, enable: None },
    m("mod-autobalance", "AutoBalance", "Adapte les points de vie et dégâts des créatures d'instance à la taille du groupe : indispensable en solo avec des compagnons.", "AutoBalance.conf", "AutoBalance.Enable.Global"),
    m("mod-cfbg", "Champs de bataille interfactions (CFBG)", "Alliance et Horde mélangées dans les champs de bataille pour remplir les files.", "CFBG.conf", "CFBG.Enable"),
    m("mod-challenge-modes", "Modes défi", "Hardcore, semi-hardcore, objets fabriqués uniquement, niveau d'objets limité… choisis par personnage auprès d'un PNJ.", "challenge_modes.conf", "ChallengeModes.Enable"),
    m("mod-character-services", "Services de personnage", "Changement de nom, d'apparence, de race et de faction via un PNJ.", "character_services.conf", "CharacterServices.Enable"),
    m("mod-combat-lock-rescue", "Secours de combat bloqué", "Sort du mode combat un personnage resté bloqué « en combat ».", "combat_lock_rescue.conf", "CombatLockRescue.Enable"),
    m("mod-dk-acherus-rescue", "Secours d'Acherus (chevalier de la mort)", "Dépanne un chevalier de la mort bloqué dans la zone de départ.", "dk_acherus_rescue.conf", "DKAcherusRescue.Enable"),
    m("mod-junk-to-gold", "Objets gris en or", "Convertit automatiquement les objets gris ramassés en argent.", "JunkToGold.conf", "JunkToGold.Enable"),
    m("mod-morphsummon", "Métamorphose des invocations", "Change l'apparence des familiers et invocations.", "morphsummon.conf", "MorphSummon.Enabled"),
    m("mod-mythic-plus", "Mythique +", "Donjons à difficulté croissante avec clés et affixes (version Sylvania).", "mod_mythic_plus.conf", "MythicPlus.Enable"),
    m("mod-npc-free-professions", "PNJ métiers gratuits", "PNJ qui enseigne gratuitement les métiers.", "mod_npc_free_professions.conf", "NpcFreeProfessions.Enable"),
    ModuleDef { id: "mod-npc-morph", label: "PNJ de métamorphose", description: "PNJ qui change l'apparence du joueur.", conf: None, enable: None },
    m("mod-pet-loot", "Butin par le familier", "Le familier ramasse le butin des créatures tuées.", "mod_pet_loot.conf", "PetLoot.Enable"),
    m("mod-phase-aura-rescue", "Secours de phase", "Retire les auras de phase qui rendent un personnage invisible au monde.", "phase_aura_rescue.conf", "PhaseAuraRescue.Enable"),
    m("mod-playerbots", "PlayerBots", "Bots joueurs (compagnons et population aléatoire). Réglages détaillés dans l'onglet PlayerBots.", "playerbots.conf", "AiPlayerbot.Enabled"),
    m("mod-skip-dk-starting-area", "Sauter la zone de départ des chevaliers de la mort", "Le chevalier de la mort commence directement hors d'Acherus, quêtes de départ validées.", "skip_dk_module.conf", "Skip.Deathknight.Starter.Enable"),
    m("mod-solo-lfg", "Recherche de donjon en solo", "Permet de lancer l'outil Recherche de donjon sans groupe complet.", "SoloLfg.conf", "SoloLFG.Enable"),
    m("mod-solocraft", "Solocraft", "Renforce les joueurs en instance selon la taille du groupe (alternative ou complément d'AutoBalance).", "Solocraft.conf", "Solocraft.Enable"),
    m("mod-sylvania-events", "Événements Sylvania", "Événements saisonniers du royaume Sylvania (désactivés par défaut en solo).", "sylvania_events.conf", "SylvaniaEvents.Enable"),
    m("mod-transmog", "Transmogrification", "Change l'apparence de l'équipement auprès d'un PNJ.", "transmog.conf", "Transmogrification.Enable"),
];

/// Valeur actuelle d'une clé dans un `.conf` (première occurrence hors commentaire).
pub fn read_key(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let t = l.trim_start();
        let rest = t.strip_prefix(key)?;
        let rest = rest.trim_start();
        let v = rest.strip_prefix('=')?;
        Some(v.trim().trim_matches('"').to_string())
    })
}

/// Module activé selon le texte de son `.conf` (None : pas de clé d'activation ou clé absente).
pub fn is_enabled(def: &ModuleDef, text: &str) -> Option<bool> {
    let (key, on, _) = def.enable?;
    let v = read_key(text, key)?;
    Some(v.eq_ignore_ascii_case(on) || v == "1" || v.eq_ignore_ascii_case("true"))
}

/// État d'un module tel que présenté dans l'onglet.
#[derive(Clone, Debug)]
pub struct ModuleState {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// Chemin du `.conf` (généré) s'il existe, sinon du `.conf.dist`
    pub conf_path: Option<std::path::PathBuf>,
    /// None : module sans interrupteur (toujours actif) ou piloté ailleurs (AHBot)
    pub enabled: Option<bool>,
    pub switchable: bool,
}

/// Lit l'état de tous les modules depuis `configs/modules/` (le `.conf` s'il existe, sinon le modèle).
pub fn states(modules_dir: &std::path::Path) -> Vec<ModuleState> {
    MODULES
        .iter()
        .map(|d| {
            let conf_path = d.conf.and_then(|c| {
                let p = modules_dir.join(c);
                if p.is_file() {
                    Some(p)
                } else {
                    let dist = modules_dir.join(format!("{c}.dist"));
                    dist.is_file().then_some(dist)
                }
            });
            let text = conf_path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
            let enabled = is_enabled(d, &text);
            ModuleState { id: d.id, label: d.label, description: d.description, conf_path, enabled, switchable: d.enable.is_some() }
        })
        .collect()
}

pub fn find(id: &str) -> Option<&'static ModuleDef> {
    MODULES.iter().find(|m| m.id == id)
}
