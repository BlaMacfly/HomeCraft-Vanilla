//! Catalogue des réglages exposés dans l'interface : onglet PlayerBots (aiplayerbot.conf du module cmangos/playerbots)
//! et onglet Monde (mangosd.conf). Chaque entrée : clé exacte du fichier CMaNGOS, libellé et aide en français, type,
//! valeur par défaut (celle du modèle .conf.dist du commit figé, sauf mention HomeCraft). Les valeurs modifiées par
//! l'utilisateur sont stockées dans UserData/settings.json (`conf`) et appliquées à la génération des .conf.
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

/// Plafond du nombre de bots aléatoires connectés réglable dans l'onglet PlayerBots.
pub const RANDOMBOT_MAX: u32 = 5000;

pub const BOT_OPTIONS: &[Opt] = &[
    // ---- Bots aléatoires
    Opt { key: "AiPlayerbot.RandomBotAutologin", group: "Bots aléatoires (monde peuplé)", label: "Connecter les bots aléatoires", help: "Des personnages bots vivent dans le monde : ils montent de niveau, font leurs quêtes, chassent, commercent, forment des groupes et des guildes. Au premier démarrage, le serveur crée leurs comptes et personnages (quelques minutes).", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.MinRandomBots", group: "Bots aléatoires (monde peuplé)", label: "Nombre minimal de bots connectés", help: "Le serveur garde un nombre de bots connectés compris entre le minimum et le maximum, et en renouvelle une partie de temps en temps. Chaque bot coûte de la mémoire et du processeur : 200 bots conviennent à un PC courant, 1 000 demandent une machine puissante. (HomeCraft : 100 ; modèle CMaNGOS : 1000)", kind: Kind::Int { min: 0, max: RANDOMBOT_MAX as i64 }, default: "100" },
    Opt { key: "AiPlayerbot.MaxRandomBots", group: "Bots aléatoires (monde peuplé)", label: "Nombre maximal de bots connectés", help: "Les comptes de bots (10 personnages chacun) sont créés d'après ce nombre au démarrage du serveur. (HomeCraft : 200 ; modèle CMaNGOS : 1000)", kind: Kind::Int { min: 0, max: RANDOMBOT_MAX as i64 }, default: "200" },
    Opt { key: "AiPlayerbot.RandomBotMinLevel", group: "Bots aléatoires (monde peuplé)", label: "Niveau minimal des bots", help: "", kind: Kind::Int { min: 1, max: 60 }, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotMaxLevel", group: "Bots aléatoires (monde peuplé)", label: "Niveau maximal des bots", help: "", kind: Kind::Int { min: 1, max: 60 }, default: "60" },
    Opt { key: "AiPlayerbot.RandomBotMaxLevelChance", group: "Bots aléatoires (monde peuplé)", label: "Part des bots au niveau maximal", help: "Entre 0 et 1. 0,15 = 15 % des bots sont créés au niveau maximal, les autres à un niveau au hasard.", kind: Kind::Float { min: 0.0, max: 1.0 }, default: "0.15" },
    Opt { key: "AiPlayerbot.RandomBotJoinLfg", group: "Bots aléatoires (monde peuplé)", label: "Les bots utilisent la recherche de groupe", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotJoinBG", group: "Bots aléatoires (monde peuplé)", label: "Les bots rejoignent les champs de bataille", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotAutoJoinBG", group: "Bots aléatoires (monde peuplé)", label: "Champs de bataille lancés automatiquement par les bots", help: "Les bots s'inscrivent d'eux-mêmes pour que les champs de bataille tournent en permanence.", kind: ON_OFF, default: "0" },
    Opt { key: "AiPlayerbot.RandomBotGroupNearby", group: "Bots aléatoires (monde peuplé)", label: "Les bots se groupent entre eux", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotGuildNearby", group: "Bots aléatoires (monde peuplé)", label: "Les bots rejoignent des guildes", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotFormGuild", group: "Bots aléatoires (monde peuplé)", label: "Les bots fondent des guildes", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.EnableBroadcasts", group: "Bots aléatoires (monde peuplé)", label: "Les bots parlent dans les canaux", help: "Butin, quêtes, recherche de groupe… Les phrases sont traduites en français pour un client français.", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.EnableGreet", group: "Bots aléatoires (monde peuplé)", label: "Les bots saluent les joueurs", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.RandomBotTeleportNearPlayer", group: "Bots aléatoires (monde peuplé)", label: "Faire venir des bots près de vous", help: "Une partie des bots apparaît dans votre zone plutôt qu'au hasard dans le monde.", kind: ON_OFF, default: "0" },
    // ---- Invitations et groupe
    Opt { key: "AiPlayerbot.RandomBotInvitePlayer", group: "Invitations et groupe", label: "Les bots vous invitent dans leur groupe", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.LevelCheck", group: "Invitations et groupe", label: "Écart de niveau maximal pour accepter une invitation", help: "Un bot refuse votre invitation si son niveau dépasse le vôtre de plus de ce nombre. 60 = il accepte toujours, quel que soit votre niveau. (HomeCraft : 60 ; modèle CMaNGOS : 30)", kind: Kind::Int { min: 0, max: 60 }, default: "60" },
    Opt { key: "AiPlayerbot.GearScoreCheck", group: "Invitations et groupe", label: "Refus si votre équipement est trop faible", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AiPlayerbot.SyncQuestWithPlayer", group: "Invitations et groupe", label: "Les bots du groupe prennent vos quêtes", help: "", kind: ON_OFF, default: "0" },
    Opt { key: "AiPlayerbot.AutoDoQuests", group: "Invitations et groupe", label: "Les bots font leurs quêtes d'eux-mêmes", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.AutoEquipUpgradeLoot", group: "Invitations et groupe", label: "Les bots équipent le butin meilleur que le leur", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.AllowGuildBots", group: "Invitations et groupe", label: "Commander les bots de votre guilde", help: "", kind: ON_OFF, default: "1" },
    Opt { key: "AiPlayerbot.BotCheats", group: "Invitations et groupe", label: "Facilités accordées à vos bots", help: "Liste séparée par des virgules : taxi (vols gratuits), item (composants de sorts et munitions), breath (respiration sous l'eau). Vide = aucune.", kind: Kind::Text, default: "taxi,item,breath" },
    // ---- Performances
    Opt { key: "AiPlayerbot.botActiveAlone", group: "Performances", label: "Part des bots actifs loin de vous (%)", help: "Les bots éloignés de tout joueur ne réfléchissent qu'une partie du temps. Plus haut = monde plus vivant, plus de charge.", kind: Kind::Int { min: 0, max: 100 }, default: "10" },
    Opt { key: "AiPlayerbot.RandomBotsMaxLoginsPerInterval", group: "Performances", label: "Bots connectés par vague", help: "Au démarrage, les bots se connectent par vagues de cette taille.", kind: Kind::Int { min: 1, max: 200 }, default: "10" },
];

/// Entrée du catalogue des bots par sa clé (clé connue à la compilation).
pub fn bot_opt(key: &str) -> &'static Opt {
    BOT_OPTIONS.iter().find(|o| o.key == key).expect("clé du catalogue des bots")
}

pub const WORLD_OPTIONS: &[Opt] = &[
    // ---- Règles du monde
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
    Opt { key: "Quests.IgnoreRaid", group: "Règles du monde", label: "Autoriser les quêtes de raid en groupe simple", help: "Utile en solo avec des bots.", kind: ON_OFF, default: "0" },
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
    // ---- Sessions (options ajoutées par homecraft.patch)
    Opt { key: "HomeCraft.DisableAfkKick", group: "Sessions", label: "Pas de déconnexion pour inactivité", help: "Un personnage absent (AFK) n'est plus renvoyé à la sélection des personnages au bout de 15 minutes. Une connexion réellement perdue est toujours détectée et nettoyée. (HomeCraft : activé ; CMaNGOS : désactivé)", kind: ON_OFF, default: "1" },
    Opt { key: "HomeCraft.AllowMultipleSessionsPerAccount", group: "Sessions", label: "Plusieurs personnages d'un même compte en même temps (multiboxing)", help: "Ouvrez plusieurs fois le jeu et connectez un personnage différent du même compte dans chacun. Sans cette option, la connexion d'un deuxième client déconnecte le premier. (HomeCraft : activé)", kind: ON_OFF, default: "1" },
    Opt { key: "HomeCraft.MaxSessionsPerAccount", group: "Sessions", label: "Sessions simultanées par compte", help: "Nombre maximal de clients connectés en même temps sur un même compte (1 à 10).", kind: Kind::Int { min: 1, max: 10 }, default: "4" },
    // ---- Maître de jeu
    Opt { key: "GM.LoginState", group: "Maître de jeu (compte GM)", label: "Mode MJ à la connexion", help: "Un compte de niveau MJ (2 ou 3) se connecte en mode MJ selon ce réglage. « Désactivé » conseillé pour jouer normalement : les commandes restent disponibles, le mode MJ s'active avec .gm on.", kind: Kind::Choice(&[("0", "Désactivé"), ("1", "Activé"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { key: "GM.Visible", group: "Maître de jeu (compte GM)", label: "Visibilité du MJ à la connexion", help: "", kind: Kind::Choice(&[("0", "Invisible"), ("1", "Visible"), ("2", "Dernier état enregistré")]), default: "2" },
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
