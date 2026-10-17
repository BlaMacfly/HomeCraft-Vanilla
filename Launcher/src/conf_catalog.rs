//! Catalogue des réglages de mangosd.conf exposés dans l'interface (onglets PlayerBots et Monde).
//! Chaque entrée : clé exacte du fichier VMaNGOS, libellé et aide en français, type, valeur par défaut
//! (celle du modèle mangosd.conf.dist.in du commit figé). Les valeurs modifiées par l'utilisateur sont
//! stockées dans UserData/settings.json (`conf`) et appliquées à la génération de mangosd.conf.
//! Seules les clés de ce catalogue peuvent être surchargées : les clés gérées (ports, base, chemins) ne le sont jamais.

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

pub const BOT_OPTIONS: &[Opt] = &[
    // ---- Compagnons
    Opt { key: "PartyBot.MaxBots", group: "Compagnons (.partybot)", label: "Compagnons maximum par groupe", help: "0 = illimité. Un groupe compte 5 membres, un raid 40.", kind: Kind::Int { min: 0, max: 39 }, default: "0" },
    Opt { key: "PartyBot.SkipChecks", group: "Compagnons (.partybot)", label: "Lever les restrictions d'invocation", help: "Permet d'invoquer en combat, en donjon, et de choisir le niveau du compagnon, même sans être GM.", kind: ON_OFF, default: "0" },
    Opt { key: "PartyBot.AutoEquip", group: "Compagnons (.partybot)", label: "Équipement des nouveaux compagnons", help: "Objets de départ, équipement aléatoire adapté au niveau, ou modèle prédéfini.", kind: Kind::Choice(&[("0", "Objets de départ"), ("1", "Aléatoire adapté au niveau"), ("2", "Modèle prédéfini")]), default: "1" },
    Opt { key: "PartyBot.RandomGearLevelDifference", group: "Compagnons (.partybot)", label: "Écart de niveau d'objet toléré", help: "Pour l'équipement aléatoire : plus la valeur est basse, plus l'équipement colle au niveau du compagnon.", kind: Kind::Int { min: 0, max: 60 }, default: "10" },
    // ---- Bots aléatoires
    Opt { key: "RandomBot.Enable", group: "Bots de peuplement (RandomBot)", label: "Activer les bots de peuplement", help: "Connecte automatiquement des personnages bots enregistrés dans la table characters.playerbot (avec une IA de peuplement : StormwindPopulationAI, IronforgePopulationAI, OrgrimmarPopulationAI). Sans personnages bots enregistrés, rien n'apparaît.", kind: ON_OFF, default: "0" },
    Opt { key: "RandomBot.MinBots", group: "Bots de peuplement (RandomBot)", label: "Nombre minimal de bots connectés", help: "Au démarrage, le serveur connecte d'un coup ce nombre de bots, puis toutes les « intervalle de renouvellement » il en ajoute ou en retire un pour rester entre le minimum et le maximum. Chaque bot coûte environ 1 Mo de mémoire et un peu de processeur : 1 000 bots connectés occupent environ 1 Go et un cœur.", kind: Kind::Int { min: 0, max: 5000 }, default: "0" },
    Opt { key: "RandomBot.MaxBots", group: "Bots de peuplement (RandomBot)", label: "Nombre maximal de bots connectés", help: "Plafonné par le serveur au nombre de personnages bots enregistrés. Après une génération, le launcher le règle à 300 au plus : montez-le ici si votre PC le permet.", kind: Kind::Int { min: 0, max: 5000 }, default: "0" },
    Opt { key: "RandomBot.Refresh", group: "Bots de peuplement (RandomBot)", label: "Intervalle de renouvellement (ms)", help: "Fréquence à laquelle le serveur ajoute ou retire un bot pour rester dans la fourchette.", kind: Kind::Int { min: 5000, max: 3_600_000 }, default: "60000" },
    // ---- Champs de bataille
    Opt { key: "BattleBot.AutoJoin", group: "Champs de bataille (BattleBot)", label: "Compléter automatiquement les champs de bataille", help: "À l'inscription d'un joueur, des bots sont ajoutés pour que le champ de bataille démarre (Alterac, Arathi, Warsong).", kind: ON_OFF, default: "0" },
    Opt { key: "BattleBot.AutoEquip", group: "Champs de bataille (BattleBot)", label: "Équipement des bots de champ de bataille", help: "", kind: Kind::Choice(&[("0", "Objets de départ"), ("1", "Aléatoire adapté au niveau"), ("2", "Modèle prédéfini")]), default: "1" },
    // ---- Général
    Opt { key: "PlayerBot.UpdateMs", group: "Général", label: "Temps de réaction des bots (ms)", help: "Période de mise à jour de l'IA : plus court = bots plus réactifs, un peu plus de charge.", kind: Kind::Int { min: 100, max: 5000 }, default: "1000" },
    Opt { key: "PlayerBot.ShowInWhoList", group: "Général", label: "Bots visibles dans /qui", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "PlayerBot.AllowSaving", group: "Général", label: "Enregistrer la progression des personnages chargés comme bots", help: "Concerne .partybot load : les changements (niveau, objets) du personnage utilisé comme compagnon sont sauvegardés.", kind: ON_OFF, default: "0" },
    Opt { key: "PlayerBot.Debug", group: "Général", label: "Informations de débogage des bots dans la console", help: "", kind: ON_OFF, default: "0" },
];

pub const PATCHES: &[(&str, &str)] = &[
    ("0", "1.2 — Cœur du Magma"), ("1", "1.3", ), ("2", "1.4 — Honneur"), ("3", "1.5 — Champs de bataille"), ("4", "1.6 — Repaire de l'Aile noire"),
    ("5", "1.7 — Zul'Gurub"), ("6", "1.8 — Dragons du Cauchemar"), ("7", "1.9 — Ahn'Qiraj"), ("8", "1.10 — Tempêtes"), ("9", "1.11 — Naxxramas"), ("10", "1.12 — Tambours de guerre (complet)"),
];

pub const WORLD_OPTIONS: &[Opt] = &[
    // ---- Règles du monde
    Opt { key: "WowPatch", group: "Règles du monde", label: "Contenu du monde (patch)", help: "Progression VMaNGOS : le contenu, les objets et les événements correspondent au patch choisi. 1.12 = tout Vanilla.", kind: Kind::Choice(PATCHES), default: "10" },
    Opt { key: "GameType", group: "Règles du monde", label: "Type de royaume", help: "Normal : pas de JcJ non consenti.", kind: Kind::Choice(&[("0", "Normal (JcE)"), ("1", "JcJ"), ("6", "Jeu de rôle"), ("8", "Jeu de rôle JcJ")]), default: "1" },
    Opt { key: "Motd", group: "Règles du monde", label: "Message du jour", help: "Affiché dans le chat à la connexion.", kind: Kind::Text, default: "Bienvenue dans HomeCraft Edition Vanilla !" },
    Opt { key: "StartPlayerLevel", group: "Règles du monde", label: "Niveau des nouveaux personnages", help: "", kind: Kind::Int { min: 1, max: 60 }, default: "1" },
    Opt { key: "StartPlayerMoney", group: "Règles du monde", label: "Argent de départ (pièces de cuivre)", help: "10000 = 1 pièce d'or.", kind: Kind::Int { min: 0, max: 100_000_000 }, default: "0" },
    Opt { key: "SkipCinematics", group: "Règles du monde", label: "Cinématique d'introduction", help: "", kind: Kind::Choice(&[("0", "Toujours"), ("1", "Seulement au premier personnage de la race"), ("2", "Jamais")]), default: "0" },
    Opt { key: "CharactersPerRealm", group: "Règles du monde", label: "Personnages par royaume", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "10" },
    Opt { key: "Instance.IgnoreLevel", group: "Règles du monde", label: "Ignorer le niveau requis des instances", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "Instance.ResetTimeHour", group: "Règles du monde", label: "Heure de réinitialisation des raids", help: "Heure locale (0-23).", kind: Kind::Int { min: 0, max: 23 }, default: "4" },
    Opt { key: "Quests.LowLevelHideDiff", group: "Règles du monde", label: "Masquer les quêtes trop basses (écart de niveaux)", help: "Les points d'exclamation des quêtes inférieures de plus de N niveaux ne sont plus affichés. -1 = jamais.", kind: Kind::Int { min: -1, max: 60 }, default: "4" },
    Opt { key: "Quests.HighLevelHideDiff", group: "Règles du monde", label: "Masquer les quêtes trop hautes (écart de niveaux)", help: "-1 = jamais.", kind: Kind::Int { min: -1, max: 60 }, default: "7" },
    Opt { key: "Quests.IgnoreRaid", group: "Règles du monde", label: "Autoriser les quêtes de raid en groupe simple", help: "Utile en solo avec des compagnons.", kind: ON_OFF, default: "0" },
    Opt { key: "MaxPrimaryTradeSkill", group: "Règles du monde", label: "Métiers principaux maximum", help: "", kind: Kind::Int { min: 0, max: 11 }, default: "2" },
    Opt { key: "MinPetitionSigns", group: "Règles du monde", label: "Signatures requises pour fonder une guilde", help: "0 permet de créer une guilde seul.", kind: Kind::Int { min: 0, max: 9 }, default: "9" },
    // ---- Expérience et progression
    Opt { key: "Rate.XP.Kill", group: "Expérience et progression", label: "Expérience des victoires", help: "Multiplicateur. 1 = Vanilla.", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { key: "Rate.XP.Quest", group: "Expérience et progression", label: "Expérience des quêtes", help: "", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { key: "Rate.XP.Explore", group: "Expérience et progression", label: "Expérience d'exploration", help: "", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Rest.InGame", group: "Expérience et progression", label: "Repos accumulé en jeu", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Rest.Offline.InTavernOrCity", group: "Expérience et progression", label: "Repos hors ligne en auberge ou en ville", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Rest.Offline.InWilderness", group: "Expérience et progression", label: "Repos hors ligne en extérieur", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Talent", group: "Expérience et progression", label: "Points de talent", help: "", kind: Kind::Float { min: 1.0, max: 10.0 }, default: "1" },
    Opt { key: "Rate.Reputation.Gain", group: "Expérience et progression", label: "Gain de réputation", help: "", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Reputation.LowLevel.Kill", group: "Expérience et progression", label: "Réputation sur créatures grises", help: "", kind: Kind::Float { min: 0.0, max: 1.0 }, default: "0.2" },
    Opt { key: "SkillGain.Crafting", group: "Expérience et progression", label: "Progression des métiers d'artisanat", help: "Points gagnés par fabrication.", kind: Kind::Int { min: 1, max: 10 }, default: "1" },
    Opt { key: "SkillGain.Gathering", group: "Expérience et progression", label: "Progression des métiers de récolte", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "1" },
    Opt { key: "SkillGain.Weapon", group: "Expérience et progression", label: "Progression des compétences d'armes", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "1" },
    Opt { key: "SkillGain.Defense", group: "Expérience et progression", label: "Progression de la défense", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "1" },
    Opt { key: "Rate.Mining.Amount", group: "Expérience et progression", label: "Quantité de minerai par filon", help: "", kind: Kind::Float { min: 0.5, max: 10.0 }, default: "1" },
    Opt { key: "MaxGroupXPDistance", group: "Expérience et progression", label: "Distance de partage d'expérience en groupe", help: "En mètres.", kind: Kind::Int { min: 10, max: 300 }, default: "74" },
    // ---- Butin et économie
    Opt { key: "Rate.Drop.Item.Poor", group: "Butin et économie", label: "Butin : objets gris", help: "Multiplicateurs de chance de butin.", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Drop.Item.Normal", group: "Butin et économie", label: "Butin : objets blancs", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Drop.Item.Uncommon", group: "Butin et économie", label: "Butin : objets verts", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Drop.Item.Rare", group: "Butin et économie", label: "Butin : objets bleus", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Drop.Item.Epic", group: "Butin et économie", label: "Butin : objets épiques", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Drop.Item.Legendary", group: "Butin et économie", label: "Butin : objets légendaires", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Drop.Item.Referenced", group: "Butin et économie", label: "Butin : tables référencées", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "Rate.Drop.Money", group: "Butin et économie", label: "Argent trouvé sur les créatures", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { key: "MailDeliveryDelay", group: "Butin et économie", label: "Délai de livraison du courrier (secondes)", help: "3600 = 1 heure (Vanilla). 0 = immédiat.", kind: Kind::Int { min: 0, max: 86400 }, default: "3600" },
    // ---- Difficulté
    Opt { key: "Rate.Creature.Normal.Damage", group: "Difficulté des créatures", label: "Dégâts des créatures normales", help: "Multiplicateurs. En solo avec compagnons, 0.8 à 1.0 est confortable.", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Normal.HP", group: "Difficulté des créatures", label: "Points de vie des créatures normales", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Normal.SpellDamage", group: "Difficulté des créatures", label: "Dégâts des sorts des créatures normales", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Elite.Elite.Damage", group: "Difficulté des créatures", label: "Dégâts des élites", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Elite.Elite.HP", group: "Difficulté des créatures", label: "Points de vie des élites", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Elite.Elite.SpellDamage", group: "Difficulté des créatures", label: "Dégâts des sorts des élites", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Elite.RAREELITE.HP", group: "Difficulté des créatures", label: "Points de vie des élites rares", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Elite.RAREELITE.Damage", group: "Difficulté des créatures", label: "Dégâts des élites rares", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Elite.WORLDBOSS.HP", group: "Difficulté des créatures", label: "Points de vie des boss mondiaux", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Elite.WORLDBOSS.Damage", group: "Difficulté des créatures", label: "Dégâts des boss mondiaux", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { key: "Rate.Creature.Aggro", group: "Difficulté des créatures", label: "Distance d'agression", help: "Multiplicateur du rayon d'agression. 0 = les créatures n'attaquent jamais d'elles-mêmes.", kind: Kind::Float { min: 0.0, max: 3.0 }, default: "1" },
    Opt { key: "Death.SicknessLevel", group: "Difficulté des créatures", label: "Niveau à partir duquel la faiblesse de résurrection s'applique", help: "11 = Vanilla. 61 = jamais.", kind: Kind::Int { min: -10, max: 61 }, default: "11" },
    Opt { key: "Death.CorpseReclaimDelay.PvE", group: "Difficulté des créatures", label: "Délai croissant de récupération du corps (JcE)", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "Battleground.CastDeserter", group: "Difficulté des créatures", label: "Malus de déserteur en quittant un champ de bataille", help: "", kind: ON_OFF, default: "1" },
    // ---- Factions
    Opt { key: "AllowTwoSide.Accounts", group: "Factions et social", label: "Alliance et Horde sur le même compte", help: "Pratique en solo pour essayer les deux factions.", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Chat", group: "Factions et social", label: "Chat entre factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Channel", group: "Factions et social", label: "Canaux communs aux deux factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Group", group: "Factions et social", label: "Groupes mixtes", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Guild", group: "Factions et social", label: "Guildes mixtes", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Trade", group: "Factions et social", label: "Échanges entre factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.Interaction.Mail", group: "Factions et social", label: "Courrier entre factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.WhoList", group: "Factions et social", label: "/qui commun aux deux factions", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AllowTwoSide.AddFriend", group: "Factions et social", label: "Amis de l'autre faction", help: "", kind: ON_OFF, default: "0" },
    // ---- Maître de jeu
    Opt { key: "GM.LoginState", group: "Maître de jeu (compte GM)", label: "Mode MJ à la connexion", help: "Un compte de niveau GM 6 (nécessaire aux compagnons) se connecte en mode MJ selon ce réglage. « Désactivé » conseillé pour jouer normalement : les commandes restent disponibles, le mode MJ s'active avec .gm on.", kind: Kind::Choice(&[("0", "Désactivé"), ("1", "Activé"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { key: "GM.Visible", group: "Maître de jeu (compte GM)", label: "Visibilité du MJ à la connexion", help: "", kind: Kind::Choice(&[("0", "Invisible"), ("1", "Visible"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { key: "GM.CheatGod", group: "Maître de jeu (compte GM)", label: "Invulnérabilité en mode MJ", help: "Ne s'applique qu'en mode MJ actif (.gm on).", kind: ON_OFF, default: "1" },
    Opt { key: "GM.AllowTrades", group: "Maître de jeu (compte GM)", label: "Échanges, courrier et hôtel des ventes autorisés pour les comptes GM", help: "À laisser activé en solo.", kind: ON_OFF, default: "1" },
    // ---- Technique
    Opt { key: "Visibility.Distance.Continents", group: "Technique", label: "Distance d'affichage en extérieur", help: "En mètres (45 à 333). Plus grand = plus de charge.", kind: Kind::Int { min: 45, max: 333 }, default: "100" },
    Opt { key: "Visibility.Distance.Instances", group: "Technique", label: "Distance d'affichage en instance", help: "", kind: Kind::Int { min: 45, max: 333 }, default: "170" },
    Opt { key: "PlayerSave.Interval", group: "Technique", label: "Intervalle de sauvegarde des personnages (ms)", help: "900000 = 15 minutes. Les sauvegardes du launcher forcent de toute façon un saveall.", kind: Kind::Int { min: 60_000, max: 3_600_000 }, default: "900000" },
    Opt { key: "Instance.UnloadDelay", group: "Technique", label: "Déchargement des instances vides (ms)", help: "", kind: Kind::Int { min: 60_000, max: 7_200_000 }, default: "1800000" },
    Opt { key: "Event.Announce", group: "Technique", label: "Annoncer les événements mondiaux", help: "", kind: ON_OFF, default: "0" },
];

/// Valeur effective (surcharge ou défaut).
pub fn value<'a>(overrides: &'a std::collections::BTreeMap<String, String>, opt: &'a Opt) -> &'a str {
    overrides.get(opt.key).map(String::as_str).unwrap_or(opt.default)
}

/// Clés surchargeables (liste blanche).
pub fn is_known(key: &str) -> bool {
    BOT_OPTIONS.iter().chain(WORLD_OPTIONS.iter()).any(|o| o.key == key)
}

/// Valeur formatée pour mangosd.conf (les textes entre guillemets).
pub fn conf_value(opt: &Opt, v: &str) -> String {
    match opt.kind {
        Kind::Text => format!("\"{}\"", v.replace('"', "'")),
        _ => v.to_string(),
    }
}
