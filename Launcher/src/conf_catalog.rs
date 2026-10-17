//! Catalogue des réglages exposés dans l'interface : onglet Monde (mangosd.conf) et onglet PlayerBots
//! (aiplayerbot.conf du module cmangos/playerbots). Chaque entrée : clé exacte du fichier CMaNGOS-tbc, libellé et
//! aide en français, type, valeur par défaut (celle du modèle .conf.dist du build figé 15b6ddb4, relevée le
//! 2026-10-17, sauf mention « HomeCraft »). Les valeurs modifiées par l'utilisateur sont stockées dans
//! UserData/settings.json (`conf`) et appliquées à la génération des .conf.
//! Seules les clés de ce catalogue peuvent être surchargées : les clés gérées (ports, bases, chemins) ne le sont jamais.

#[derive(Clone, Copy)]
pub enum Kind {
    Bool,
    Int { min: i64, max: i64 },
    Float { min: f64, max: f64 },
    Choice(&'static [(&'static str, &'static str)]), // (valeur, libellé)
    Text,
}

#[derive(Clone, Copy)]
pub struct Opt {
    pub key: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub kind: Kind,
    pub default: &'static str,
}

const ON_OFF: Kind = Kind::Bool;
const RATE: Kind = Kind::Float { min: 0.0, max: 20.0 };
const MULT: Kind = Kind::Float { min: 0.1, max: 5.0 };

/// Module PlayerBots (aiplayerbot.conf). Défaut HomeCraft : 50 bots aléatoires au lieu de 1000 (PC solo).
pub const BOT_OPTIONS: &[Opt] = &[
    // ---- Bots aléatoires (peuplement)
    Opt { key: "AiPlayerbot.RandomBotAutologin", group: "Bots aléatoires (peuplement du monde)", label: "Activer les bots aléatoires", help: "Des personnages bots (comptes RNDBOT…) sont créés automatiquement, se connectent, quêtent, groupent et peuplent le monde.", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.MinRandomBots", group: "Bots aléatoires (peuplement du monde)", label: "Nombre minimal de bots", help: "Défaut du module : 1000. HomeCraft : 50, adapté à un PC de jeu. Chaque bot consomme mémoire et processeur.", kind: Kind::Int { min: 0, max: 2000 }, default: "50" },
    Opt { key: "AiPlayerbot.MaxRandomBots", group: "Bots aléatoires (peuplement du monde)", label: "Nombre maximal de bots", help: "Le nombre de comptes de bots est calculé automatiquement (un compte pour 9 bots).", kind: Kind::Int { min: 0, max: 2000 }, default: "50" },
    Opt { key: "AiPlayerbot.RandomBotLoginAtStartup", group: "Bots aléatoires (peuplement du monde)", label: "Connecter tous les bots au démarrage", help: "Sinon ils arrivent progressivement.", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotMinLevel", group: "Bots aléatoires (peuplement du monde)", label: "Niveau minimal des bots", help: "", kind: Kind::Int { min: 1, max: 70 }, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotMaxLevel", group: "Bots aléatoires (peuplement du monde)", label: "Niveau maximal des bots", help: "", kind: Kind::Int { min: 1, max: 70 }, default: "70" },
    Opt { key: "AiPlayerbot.RandomBotMaxLevelChance", group: "Bots aléatoires (peuplement du monde)", label: "Proportion de bots au niveau maximal", help: "0.15 = 15 % des bots au niveau maximal.", kind: Kind::Float { min: 0.0, max: 1.0 }, default: "0.15" },
    Opt { key: "AiPlayerbot.AllowGuildBots", group: "Bots aléatoires (peuplement du monde)", label: "Guildes de bots", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotGuildCount", group: "Bots aléatoires (peuplement du monde)", label: "Nombre de guildes de bots", help: "", kind: Kind::Int { min: 0, max: 100 }, default: "20" },
    Opt { key: "AiPlayerbot.DeleteRandomBotAccounts", group: "Bots aléatoires (peuplement du monde)", label: "Supprimer tous les bots au prochain démarrage", help: "Remet à zéro les bots aléatoires (comptes RNDBOT et leurs personnages). À désactiver ensuite.", kind: ON_OFF, default: "0" },
    // ---- Champs de bataille et donjons
    Opt { key: "AiPlayerbot.RandomBotJoinBG", group: "Champs de bataille et donjons", label: "Bots dans les champs de bataille et arènes", help: "Les bots aléatoires s'inscrivent quand un joueur est en file.", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotAutoJoinBG", group: "Champs de bataille et donjons", label: "Bots qui lancent eux-mêmes des champs de bataille", help: "Goulet des Chanteguerres et arènes 2c2, si le serveur n'est pas surchargé.", kind: ON_OFF, default: "0" },
    Opt { key: "AiPlayerbot.RandomBotJoinLfg", group: "Champs de bataille et donjons", label: "Bots dans la recherche de groupe", help: "Fonction marquée « non terminée » par le module.", kind: ON_OFF, default: "1" },
    // ---- Comportement
    Opt { key: "AiPlayerbot.AutoEquipUpgradeLoot", group: "Comportement des bots", label: "Les bots s'équipent du butin meilleur", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.AutoLearnTrainerSpells", group: "Comportement des bots", label: "Les bots apprennent seuls les sorts des maîtres", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AiPlayerbot.AutoLearnQuestSpells", group: "Comportement des bots", label: "Les bots apprennent les sorts de quête", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AiPlayerbot.EnableGreet", group: "Comportement des bots", label: "Les bots saluent", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomGearLoweringChance", group: "Comportement des bots", label: "Probabilité d'équipement moins bon", help: "0.15 = 15 % d'objets en dessous du niveau optimal.", kind: Kind::Float { min: 0.0, max: 1.0 }, default: "0.15" },
    Opt { key: "AiPlayerbot.BotAutologin", group: "Comportement des bots", label: "Connecter vos autres personnages comme bots", help: "À votre connexion, les autres personnages de votre compte se connectent en bots (commandes /w).", kind: ON_OFF, default: "0" },
];

pub const WORLD_OPTIONS: &[Opt] = &[
    // ---- Règles du monde
    Opt { key: "GameType", group: "Règles du monde", label: "Type de royaume", help: "Normal : pas de JcJ non consenti. Défaut HomeCraft : Normal (modèle : JcJ).", kind: Kind::Choice(&[("0", "Normal (JcE)"), ("1", "JcJ"), ("6", "Jeu de rôle"), ("8", "Jeu de rôle JcJ")]), default: "0" },
    Opt { key: "Motd", group: "Règles du monde", label: "Message du jour", help: "Affiché dans le chat à la connexion.", kind: Kind::Text, default: "Bienvenue dans HomeCraft Edition BC !" },
    Opt { key: "MaxPlayerLevel", group: "Règles du monde", label: "Niveau maximal", help: "70 = The Burning Crusade.", kind: Kind::Int { min: 1, max: 70 }, default: "70" },
    Opt { key: "StartPlayerLevel", group: "Règles du monde", label: "Niveau des nouveaux personnages", help: "", kind: Kind::Int { min: 1, max: 70 }, default: "1" },
    Opt { key: "StartPlayerMoney", group: "Règles du monde", label: "Argent de départ (pièces de cuivre)", help: "10000 = 1 pièce d'or.", kind: Kind::Int { min: 0, max: 100_000_000 }, default: "0" },
    Opt { key: "CharactersPerRealm", group: "Règles du monde", label: "Personnages par royaume", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "10" },
    Opt { key: "MaxPrimaryTradeSkill", group: "Règles du monde", label: "Métiers principaux par personnage", help: "", kind: Kind::Int { min: 0, max: 11 }, default: "2" },
    Opt { key: "AlwaysMaxSkillForLevel", group: "Règles du monde", label: "Compétences d'armes toujours au maximum", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "SkipCinematics", group: "Règles du monde", label: "Cinématiques d'introduction", help: "", kind: Kind::Choice(&[("0", "Toujours"), ("1", "Première race seulement"), ("2", "Jamais")]), default: "0" },
    Opt { key: "ActivateWeather", group: "Règles du monde", label: "Météo", help: "", kind: ON_OFF, default: "1" },
    // ---- Progression
    Opt { key: "Rate.XP.Kill", group: "Progression (taux)", label: "Expérience des monstres", help: "1 = officiel. 2 = deux fois plus vite.", kind: RATE, default: "1" },
    Opt { key: "Rate.XP.Quest", group: "Progression (taux)", label: "Expérience des quêtes", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.XP.Explore", group: "Progression (taux)", label: "Expérience d'exploration", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Rest.InGame", group: "Progression (taux)", label: "Repos accumulé en jeu", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Rest.Offline.InTavernOrCity", group: "Progression (taux)", label: "Repos hors ligne en auberge ou en ville", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Rest.Offline.InWilderness", group: "Progression (taux)", label: "Repos hors ligne en extérieur", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Reputation.Gain", group: "Progression (taux)", label: "Gain de réputation", help: "Utile pour les factions de l'Outreterre (clés des donjons héroïques).", kind: RATE, default: "1" },
    Opt { key: "Rate.Honor", group: "Progression (taux)", label: "Gain d'honneur", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Talent", group: "Progression (taux)", label: "Points de talent", help: "", kind: RATE, default: "1" },
    Opt { key: "SkillGain.Crafting", group: "Progression (taux)", label: "Progression des métiers d'artisanat", help: "Points gagnés par réussite.", kind: Kind::Int { min: 1, max: 20 }, default: "1" },
    Opt { key: "SkillGain.Gathering", group: "Progression (taux)", label: "Progression des métiers de récolte", help: "", kind: Kind::Int { min: 1, max: 20 }, default: "1" },
    Opt { key: "SkillGain.Weapon", group: "Progression (taux)", label: "Progression des compétences d'armes", help: "", kind: Kind::Int { min: 1, max: 20 }, default: "1" },
    Opt { key: "SkillGain.Defense", group: "Progression (taux)", label: "Progression de la défense", help: "", kind: Kind::Int { min: 1, max: 20 }, default: "1" },
    Opt { key: "Rate.Skill.Discovery", group: "Progression (taux)", label: "Découverte de recettes", help: "", kind: RATE, default: "1" },
    // ---- Butin
    Opt { key: "Rate.Drop.Money", group: "Butin", label: "Argent ramassé", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Drop.Item.Poor", group: "Butin", label: "Objets médiocres (gris)", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Drop.Item.Normal", group: "Butin", label: "Objets communs (blancs)", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Drop.Item.Uncommon", group: "Butin", label: "Objets peu communs (verts)", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Drop.Item.Rare", group: "Butin", label: "Objets rares (bleus)", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Drop.Item.Epic", group: "Butin", label: "Objets épiques (violets)", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Drop.Item.Legendary", group: "Butin", label: "Objets légendaires (orange)", help: "", kind: RATE, default: "1" },
    Opt { key: "Rate.Mining.Amount", group: "Butin", label: "Quantité extraite par filon", help: "", kind: RATE, default: "1" },
    // ---- Difficulté
    Opt { key: "Rate.Creature.Normal.HP", group: "Difficulté des créatures", label: "Points de vie des créatures normales", help: "", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Normal.Damage", group: "Difficulté des créatures", label: "Dégâts des créatures normales", help: "", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Elite.Elite.HP", group: "Difficulté des créatures", label: "Points de vie des élites", help: "Concerne aussi les boss de donjon : à baisser pour les faire avec peu de compagnons.", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Elite.Elite.Damage", group: "Difficulté des créatures", label: "Dégâts des élites", help: "", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Elite.Elite.SpellDamage", group: "Difficulté des créatures", label: "Dégâts des sorts des élites", help: "", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Elite.RAREELITE.HP", group: "Difficulté des créatures", label: "Points de vie des élites rares", help: "", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Elite.RAREELITE.Damage", group: "Difficulté des créatures", label: "Dégâts des élites rares", help: "", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Elite.WORLDBOSS.HP", group: "Difficulté des créatures", label: "Points de vie des boss mondiaux", help: "", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Elite.WORLDBOSS.Damage", group: "Difficulté des créatures", label: "Dégâts des boss mondiaux", help: "", kind: MULT, default: "1" },
    Opt { key: "Rate.Creature.Aggro", group: "Difficulté des créatures", label: "Distance d'agression", help: "Multiplicateur du rayon d'agression. 0 = les créatures n'attaquent jamais d'elles-mêmes.", kind: Kind::Float { min: 0.0, max: 3.0 }, default: "1" },
    Opt { key: "Death.SicknessLevel", group: "Difficulté des créatures", label: "Niveau à partir duquel le mal de résurrection s'applique", help: "11 = officiel. 71 = jamais.", kind: Kind::Int { min: -10, max: 71 }, default: "11" },
    Opt { key: "Death.CorpseReclaimDelay.PvE", group: "Difficulté des créatures", label: "Délai croissant de récupération du corps (JcE)", help: "", kind: ON_OFF, default: "1" },
    // ---- Instances
    Opt { key: "Instance.IgnoreLevel", group: "Instances et donjons héroïques", label: "Ignorer le niveau requis des instances", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "Instance.IgnoreRaid", group: "Instances et donjons héroïques", label: "Entrer en raid sans être en groupe de raid", help: "Pratique en solo avec quelques compagnons.", kind: ON_OFF, default: "0" },
    Opt { key: "Instance.ResetTimeHour", group: "Instances et donjons héroïques", label: "Heure de remise à zéro des instances", help: "Remise à zéro quotidienne des héroïques et hebdomadaire des raids.", kind: Kind::Int { min: 0, max: 23 }, default: "4" },
    Opt { key: "Rate.InstanceResetTime", group: "Instances et donjons héroïques", label: "Multiplicateur du délai de remise à zéro", help: "", kind: Kind::Float { min: 0.1, max: 10.0 }, default: "1" },
    Opt { key: "Quests.IgnoreRaid", group: "Instances et donjons héroïques", label: "Quêtes de groupe réalisables en raid", help: "", kind: ON_OFF, default: "0" },
    // ---- Arènes et JcJ
    Opt { key: "Arena.AutoDistributePoints", group: "Arènes et champs de bataille", label: "Distribution automatique des points d'arène", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "Arena.AutoDistributeInterval", group: "Arènes et champs de bataille", label: "Intervalle de distribution (jours)", help: "", kind: Kind::Int { min: 1, max: 30 }, default: "7" },
    Opt { key: "MaxHonorPoints", group: "Arènes et champs de bataille", label: "Points d'honneur maximum", help: "", kind: Kind::Int { min: 0, max: 1_000_000 }, default: "75000" },
    Opt { key: "MaxArenaPoints", group: "Arènes et champs de bataille", label: "Points d'arène maximum", help: "", kind: Kind::Int { min: 0, max: 100_000 }, default: "5000" },
    Opt { key: "StartHonorPoints", group: "Arènes et champs de bataille", label: "Points d'honneur de départ", help: "", kind: Kind::Int { min: 0, max: 1_000_000 }, default: "0" },
    Opt { key: "StartArenaPoints", group: "Arènes et champs de bataille", label: "Points d'arène de départ", help: "", kind: Kind::Int { min: 0, max: 100_000 }, default: "0" },
    Opt { key: "Battleground.CastDeserter", group: "Arènes et champs de bataille", label: "Malus de déserteur en quittant un champ de bataille", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "Battleground.QueueAnnouncer.Join", group: "Arènes et champs de bataille", label: "Annoncer les inscriptions aux champs de bataille", help: "", kind: Kind::Choice(&[("0", "Non"), ("1", "Au joueur"), ("2", "À tous")]), default: "0" },
    // ---- Factions
    Opt { key: "AllowTwoSide.Accounts", group: "Factions et social", label: "Alliance et Horde sur le même compte", help: "Pratique en solo pour essayer les deux factions.", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Chat", group: "Factions et social", label: "Chat entre factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Channel", group: "Factions et social", label: "Canaux communs aux deux factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Group", group: "Factions et social", label: "Groupes mixtes", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Guild", group: "Factions et social", label: "Guildes mixtes", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Auction", group: "Factions et social", label: "Hôtel des ventes commun", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Mail", group: "Factions et social", label: "Courrier entre factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.WhoList", group: "Factions et social", label: "/qui commun aux deux factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.AddFriend", group: "Factions et social", label: "Amis de l'autre faction", help: "", kind: ON_OFF, default: "0" },
    // ---- Maître de jeu
    Opt { key: "GM.LoginState", group: "Maître de jeu (compte GM)", label: "Mode MJ à la connexion", help: "« Désactivé » conseillé pour jouer normalement : les commandes restent disponibles, le mode MJ s'active avec .gm on.", kind: Kind::Choice(&[("0", "Désactivé"), ("1", "Activé"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { key: "GM.Visible", group: "Maître de jeu (compte GM)", label: "Visibilité du MJ à la connexion", help: "", kind: Kind::Choice(&[("0", "Invisible"), ("1", "Visible"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { key: "GM.AcceptTickets", group: "Maître de jeu (compte GM)", label: "Accepter les tickets à la connexion", help: "", kind: Kind::Choice(&[("0", "Non"), ("1", "Oui"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { key: "GM.Chat", group: "Maître de jeu (compte GM)", label: "Badge MJ dans le chat", help: "", kind: Kind::Choice(&[("0", "Non"), ("1", "Oui"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { key: "GM.StartLevel", group: "Maître de jeu (compte GM)", label: "Niveau des nouveaux personnages MJ", help: "", kind: Kind::Int { min: 1, max: 70 }, default: "1" },
    // ---- Technique
    Opt { key: "Visibility.Distance.Continents", group: "Technique", label: "Distance d'affichage en extérieur", help: "En mètres (45 à 333). Plus grand = plus de charge.", kind: Kind::Int { min: 45, max: 333 }, default: "100" },
    Opt { key: "Visibility.Distance.Instances", group: "Technique", label: "Distance d'affichage en instance", help: "", kind: Kind::Int { min: 45, max: 333 }, default: "170" },
    Opt { key: "Visibility.Distance.BGArenas", group: "Technique", label: "Distance d'affichage en champ de bataille et arène", help: "", kind: Kind::Int { min: 45, max: 533 }, default: "533" },
    Opt { key: "PlayerSave.Interval", group: "Technique", label: "Intervalle de sauvegarde des personnages (ms)", help: "900000 = 15 minutes. Les sauvegardes du launcher forcent de toute façon un saveall.", kind: Kind::Int { min: 60_000, max: 3_600_000 }, default: "900000" },
    Opt { key: "Instance.UnloadDelay", group: "Technique", label: "Déchargement des instances vides (ms)", help: "", kind: Kind::Int { min: 60_000, max: 7_200_000 }, default: "1800000" },
    Opt { key: "Event.Announce", group: "Technique", label: "Annoncer les événements mondiaux", help: "", kind: ON_OFF, default: "0" },
];

/// Valeur effective (surcharge ou défaut).
pub fn value<'a>(overrides: &'a std::collections::BTreeMap<String, String>, opt: &'a Opt) -> &'a str {
    overrides.get(opt.key).map(String::as_str).unwrap_or(opt.default)
}

/// Valeur effective d'une clé du catalogue PlayerBots (défaut du catalogue si la clé est inconnue : chaîne vide).
pub fn bot_value<'a>(overrides: &'a std::collections::BTreeMap<String, String>, key: &str) -> &'a str {
    match BOT_OPTIONS.iter().find(|o| o.key == key) {
        Some(o) => overrides.get(o.key).map(String::as_str).unwrap_or(o.default),
        None => "",
    }
}

/// Clés surchargeables (liste blanche).
pub fn is_known(key: &str) -> bool {
    BOT_OPTIONS.iter().chain(WORLD_OPTIONS.iter()).any(|o| o.key == key)
}

/// Valeur formatée pour le .conf (les textes entre guillemets).
pub fn conf_value(opt: &Opt, v: &str) -> String {
    match opt.kind {
        Kind::Text => format!("\"{}\"", v.replace('"', "'")),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    /// Chaque clé du catalogue existe dans le modèle correspondant du build figé (sinon generate_configs échouerait).
    #[test]
    fn keys_exist_in_templates() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("Server").join("mangos-tbc");
        for (file, opts) in [("mangosd.conf.dist", super::WORLD_OPTIONS), ("aiplayerbot.conf.dist", super::BOT_OPTIONS)] {
            let Ok(tpl) = std::fs::read_to_string(root.join(file)) else { continue };
            for o in opts {
                let found = tpl.lines().any(|l| l.starts_with(o.key) && l[o.key.len()..].trim_start().starts_with('='));
                assert!(found, "clé {} absente de {file}", o.key);
            }
        }
    }
}
