//! Catalogue des réglages exposés dans l'interface (onglets PlayerBots et Monde).
//! Chaque entrée : fichier cible, clé exacte, libellé et aide en français, type, valeur par défaut (celle du
//! modèle `.conf.dist` du cœur compilé : worldserver.conf.dist d'AzerothCore / fork Sylvania, playerbots.conf.dist
//! de mod-playerbots d'avril 2026). Les valeurs modifiées sont stockées dans UserData/settings.json (`conf`) et
//! appliquées à la génération des .conf. Seules les clés de ce catalogue peuvent être surchargées : les clés gérées
//! (ports, bases, chemins, Warden) ne le sont jamais. Toutes les clés ont été vérifiées présentes dans les modèles
//! le 2026-10-16 (la génération échoue explicitement si une clé disparaît d'un modèle).

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConfFile {
    World,
    Playerbots,
}

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
    pub file: ConfFile,
    pub key: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub kind: Kind,
    pub default: &'static str,
}

const ON_OFF: Kind = Kind::Bool;
const PB: ConfFile = ConfFile::Playerbots;
const W: ConfFile = ConfFile::World;

pub const BOT_OPTIONS: &[Opt] = &[
    // ---- Bots aléatoires (peuplement)
    Opt { file: PB, key: "AiPlayerbot.RandomBotMinLevel", group: "Bots aléatoires (peuplement du monde)", label: "Niveau minimum des bots", help: "Les bots sont créés et téléportés dans des zones de leur niveau.", kind: Kind::Int { min: 1, max: 80 }, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotMaxLevel", group: "Bots aléatoires (peuplement du monde)", label: "Niveau maximum des bots", help: "", kind: Kind::Int { min: 1, max: 80 }, default: "80" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotMaps", group: "Bots aléatoires (peuplement du monde)", label: "Cartes où les bots évoluent", help: "Identifiants de cartes séparés par des virgules : 0 Royaumes de l'Est, 1 Kalimdor, 530 Outreterre, 571 Norfendre.", kind: Kind::Text, default: "0,1,530,571" },
    Opt { file: PB, key: "AiPlayerbot.AutoDoQuests", group: "Bots aléatoires (peuplement du monde)", label: "Les bots font des quêtes", help: "Ils acceptent et rendent des quêtes de leur niveau, comme des joueurs.", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotJoinLfg", group: "Bots aléatoires (peuplement du monde)", label: "Les bots s'inscrivent au Recherche de donjon", help: "Permet de compléter un groupe de donjon avec des bots via l'outil du jeu.", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotJoinBG", group: "Bots aléatoires (peuplement du monde)", label: "Les bots rejoignent les champs de bataille", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotAutoJoinBG", group: "Bots aléatoires (peuplement du monde)", label: "Les bots lancent eux-mêmes des champs de bataille", help: "Sinon ils ne s'inscrivent que pour compléter la file d'un joueur.", kind: ON_OFF, default: "0" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotTalk", group: "Bots aléatoires (peuplement du monde)", label: "Les bots parlent dans les canaux", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotEmote", group: "Bots aléatoires (peuplement du monde)", label: "Les bots font des émotes", help: "", kind: ON_OFF, default: "0" },
    Opt { file: PB, key: "AiPlayerbot.EnableBroadcasts", group: "Bots aléatoires (peuplement du monde)", label: "Annonces des bots (quêtes terminées, niveaux…)", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotShowHelmet", group: "Bots aléatoires (peuplement du monde)", label: "Casques visibles", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotShowCloak", group: "Bots aléatoires (peuplement du monde)", label: "Capes visibles", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotGuildCount", group: "Bots aléatoires (peuplement du monde)", label: "Guildes de bots créées", help: "", kind: Kind::Int { min: 0, max: 100 }, default: "20" },
    Opt { file: PB, key: "AiPlayerbot.EnablePeriodicOnlineOffline", group: "Bots aléatoires (peuplement du monde)", label: "Connexions et déconnexions périodiques", help: "Simule des allées et venues de joueurs.", kind: ON_OFF, default: "0" },
    // ---- Compagnons (vos personnages et bots invités)
    Opt { file: PB, key: "AiPlayerbot.MaxAddedBots", group: "Compagnons (.playerbots bot add)", label: "Nombre maximal de bots invités", help: "Bots aléatoires que vous pouvez inviter dans votre groupe ou raid.", kind: Kind::Int { min: 0, max: 40 }, default: "40" },
    Opt { file: PB, key: "AiPlayerbot.SelfBotLevel", group: "Compagnons (.playerbots bot add)", label: "Contrôle de son propre personnage par l'IA (.playerbots self)", help: "0 désactivé, 1 réservé aux GM, 2 pour tous.", kind: Kind::Choice(&[("0", "Désactivé"), ("1", "Comptes GM"), ("2", "Tous les joueurs")]), default: "1" },
    Opt { file: PB, key: "AiPlayerbot.AllowGuildBots", group: "Compagnons (.playerbots bot add)", label: "Inviter les bots de sa guilde", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.SyncQuestWithPlayer", group: "Compagnons (.playerbots bot add)", label: "Les compagnons prennent les mêmes quêtes que vous", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.AutoPickReward", group: "Compagnons (.playerbots bot add)", label: "Choix automatique des récompenses de quête", help: "yes = toujours, no = jamais, ask = demande au joueur.", kind: Kind::Choice(&[("yes", "Toujours"), ("no", "Jamais"), ("ask", "Demander")]), default: "yes" },
    Opt { file: PB, key: "AiPlayerbot.AutoEquipUpgradeLoot", group: "Compagnons (.playerbots bot add)", label: "Les compagnons équipent les butins meilleurs", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.AutoLearnTrainerSpells", group: "Compagnons (.playerbots bot add)", label: "Apprentissage automatique des sorts de maître", help: "", kind: ON_OFF, default: "1" },
    Opt { file: PB, key: "AiPlayerbot.SummonAtInnkeepersEnabled", group: "Compagnons (.playerbots bot add)", label: "Invocation des bots aux aubergistes", help: "", kind: ON_OFF, default: "1" },
    // ---- Performance
    Opt { file: PB, key: "AiPlayerbot.BotActiveAlone", group: "Performance", label: "Activité des bots sans joueur à proximité (%)", help: "100 = les bots agissent partout, en permanence ; plus bas = moins de charge processeur.", kind: Kind::Int { min: 0, max: 100 }, default: "100" },
    Opt { file: PB, key: "AiPlayerbot.RandomBotsPerInterval", group: "Performance", label: "Bots traités par intervalle", help: "", kind: Kind::Int { min: 1, max: 500 }, default: "60" },
    Opt { file: PB, key: "AiPlayerbot.EnableGreet", group: "Performance", label: "Salut des bots à la connexion", help: "", kind: ON_OFF, default: "0" },
];

pub const WORLD_OPTIONS: &[Opt] = &[
    // ---- Règles du monde
    Opt { file: W, key: "GameType", group: "Règles du monde", label: "Type de royaume", help: "Normal : pas de JcJ non consenti.", kind: Kind::Choice(&[("0", "Normal (JcE)"), ("1", "JcJ"), ("6", "Jeu de rôle"), ("8", "Jeu de rôle JcJ")]), default: "0" },
    Opt { file: W, key: "Expansion", group: "Règles du monde", label: "Extension active", help: "2 = Wrath of the Lich King (complet).", kind: Kind::Choice(&[("0", "Classic"), ("1", "Burning Crusade"), ("2", "Wrath of the Lich King")]), default: "2" },
    Opt { file: W, key: "MaxPlayerLevel", group: "Règles du monde", label: "Niveau maximum", help: "", kind: Kind::Int { min: 1, max: 80 }, default: "80" },
    Opt { file: W, key: "StartPlayerLevel", group: "Règles du monde", label: "Niveau des nouveaux personnages", help: "", kind: Kind::Int { min: 1, max: 80 }, default: "1" },
    Opt { file: W, key: "StartHeroicPlayerLevel", group: "Règles du monde", label: "Niveau des nouveaux chevaliers de la mort", help: "", kind: Kind::Int { min: 55, max: 80 }, default: "55" },
    Opt { file: W, key: "StartPlayerMoney", group: "Règles du monde", label: "Argent de départ (pièces de cuivre)", help: "10000 = 1 pièce d'or.", kind: Kind::Int { min: 0, max: 100_000_000 }, default: "0" },
    Opt { file: W, key: "SkipCinematics", group: "Règles du monde", label: "Cinématique d'introduction", help: "", kind: Kind::Choice(&[("0", "Toujours"), ("1", "Seulement au premier personnage de la race"), ("2", "Jamais")]), default: "0" },
    Opt { file: W, key: "CharactersPerRealm", group: "Règles du monde", label: "Personnages par royaume", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "10" },
    Opt { file: W, key: "Instance.IgnoreLevel", group: "Règles du monde", label: "Ignorer le niveau requis des instances", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "Instance.ResetTimeHour", group: "Règles du monde", label: "Heure de réinitialisation des raids", help: "Heure locale (0-23).", kind: Kind::Int { min: 0, max: 23 }, default: "4" },
    Opt { file: W, key: "Quests.LowLevelHideDiff", group: "Règles du monde", label: "Masquer les quêtes trop basses (écart de niveaux)", help: "-1 = jamais.", kind: Kind::Int { min: -1, max: 80 }, default: "4" },
    Opt { file: W, key: "Quests.HighLevelHideDiff", group: "Règles du monde", label: "Masquer les quêtes trop hautes (écart de niveaux)", help: "-1 = jamais.", kind: Kind::Int { min: -1, max: 80 }, default: "7" },
    Opt { file: W, key: "Quests.IgnoreRaid", group: "Règles du monde", label: "Autoriser les quêtes de raid en groupe simple", help: "Utile en solo avec des compagnons.", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "MaxPrimaryTradeSkill", group: "Règles du monde", label: "Métiers principaux maximum", help: "", kind: Kind::Int { min: 0, max: 11 }, default: "2" },
    Opt { file: W, key: "MinPetitionSigns", group: "Règles du monde", label: "Signatures requises pour fonder une guilde", help: "0 permet de créer une guilde seul.", kind: Kind::Int { min: 0, max: 9 }, default: "9" },
    Opt { file: W, key: "DungeonFinder.OptionsMask", group: "Règles du monde", label: "Recherche de donjon", help: "Masque d'options d'AzerothCore : 1 activé, 2 à l'arrivée, 4 téléportation. 5 = activé + téléportation (défaut).", kind: Kind::Int { min: 0, max: 7 }, default: "5" },
    // ---- Expérience et progression
    Opt { file: W, key: "Rate.XP.Kill", group: "Expérience et progression", label: "Expérience des victoires", help: "Multiplicateur. 1 = règles officielles.", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.XP.Quest", group: "Expérience et progression", label: "Expérience des quêtes", help: "", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.XP.Explore", group: "Expérience et progression", label: "Expérience d'exploration", help: "", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Rest.InGame", group: "Expérience et progression", label: "Repos accumulé en jeu", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Rest.Offline.InTavernOrCity", group: "Expérience et progression", label: "Repos hors ligne en auberge ou en ville", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Rest.Offline.InWilderness", group: "Expérience et progression", label: "Repos hors ligne en extérieur", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Talent", group: "Expérience et progression", label: "Points de talent", help: "", kind: Kind::Float { min: 1.0, max: 10.0 }, default: "1" },
    Opt { file: W, key: "Rate.Reputation.Gain", group: "Expérience et progression", label: "Gain de réputation", help: "", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Reputation.LowLevel.Kill", group: "Expérience et progression", label: "Réputation sur créatures grises", help: "", kind: Kind::Float { min: 0.0, max: 1.0 }, default: "1" },
    Opt { file: W, key: "Rate.Honor", group: "Expérience et progression", label: "Points d'honneur", help: "", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.ArenaPoints", group: "Expérience et progression", label: "Points d'arène", help: "", kind: Kind::Float { min: 0.1, max: 20.0 }, default: "1" },
    Opt { file: W, key: "SkillGain.Crafting", group: "Expérience et progression", label: "Progression des métiers d'artisanat", help: "Points gagnés par fabrication.", kind: Kind::Int { min: 1, max: 10 }, default: "1" },
    Opt { file: W, key: "SkillGain.Gathering", group: "Expérience et progression", label: "Progression des métiers de récolte", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "1" },
    Opt { file: W, key: "SkillGain.Weapon", group: "Expérience et progression", label: "Progression des compétences d'armes", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "1" },
    Opt { file: W, key: "SkillGain.Defense", group: "Expérience et progression", label: "Progression de la défense", help: "", kind: Kind::Int { min: 1, max: 10 }, default: "1" },
    Opt { file: W, key: "MaxGroupXPDistance", group: "Expérience et progression", label: "Distance de partage d'expérience en groupe", help: "En mètres.", kind: Kind::Int { min: 10, max: 300 }, default: "74" },
    // ---- Butin et économie
    Opt { file: W, key: "Rate.Drop.Item.Poor", group: "Butin et économie", label: "Butin : objets gris", help: "Multiplicateurs de chance de butin.", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Drop.Item.Normal", group: "Butin et économie", label: "Butin : objets blancs", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Drop.Item.Uncommon", group: "Butin et économie", label: "Butin : objets verts", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Drop.Item.Rare", group: "Butin et économie", label: "Butin : objets bleus", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Drop.Item.Epic", group: "Butin et économie", label: "Butin : objets épiques", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Drop.Item.Legendary", group: "Butin et économie", label: "Butin : objets légendaires", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Drop.Item.Referenced", group: "Butin et économie", label: "Butin : tables référencées", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Drop.Money", group: "Butin et économie", label: "Argent trouvé sur les créatures", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.RewardBonusMoney", group: "Butin et économie", label: "Argent des quêtes au niveau maximum", help: "", kind: Kind::Float { min: 0.0, max: 20.0 }, default: "1" },
    Opt { file: W, key: "Rate.Auction.Time", group: "Butin et économie", label: "Durée des enchères", help: "", kind: Kind::Float { min: 0.1, max: 10.0 }, default: "1" },
    Opt { file: W, key: "Rate.Auction.Deposit", group: "Butin et économie", label: "Dépôt des enchères", help: "", kind: Kind::Float { min: 0.0, max: 10.0 }, default: "1" },
    Opt { file: W, key: "Rate.Auction.Cut", group: "Butin et économie", label: "Commission de l'hôtel des ventes", help: "", kind: Kind::Float { min: 0.0, max: 10.0 }, default: "1" },
    Opt { file: W, key: "MailDeliveryDelay", group: "Butin et économie", label: "Délai de livraison du courrier (secondes)", help: "3600 = 1 heure (officiel). 0 = immédiat.", kind: Kind::Int { min: 0, max: 86400 }, default: "3600" },
    // ---- Difficulté
    Opt { file: W, key: "Rate.Creature.Normal.Damage", group: "Difficulté des créatures", label: "Dégâts des créatures normales", help: "Multiplicateurs. En solo avec compagnons, 0.8 à 1.0 est confortable.", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Normal.HP", group: "Difficulté des créatures", label: "Points de vie des créatures normales", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Normal.SpellDamage", group: "Difficulté des créatures", label: "Dégâts des sorts des créatures normales", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Elite.Elite.Damage", group: "Difficulté des créatures", label: "Dégâts des élites", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Elite.Elite.HP", group: "Difficulté des créatures", label: "Points de vie des élites", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Elite.Elite.SpellDamage", group: "Difficulté des créatures", label: "Dégâts des sorts des élites", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Elite.RAREELITE.HP", group: "Difficulté des créatures", label: "Points de vie des élites rares", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Elite.RAREELITE.Damage", group: "Difficulté des créatures", label: "Dégâts des élites rares", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Elite.WORLDBOSS.HP", group: "Difficulté des créatures", label: "Points de vie des boss mondiaux", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Elite.WORLDBOSS.Damage", group: "Difficulté des créatures", label: "Dégâts des boss mondiaux", help: "", kind: Kind::Float { min: 0.1, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Rate.Creature.Aggro", group: "Difficulté des créatures", label: "Distance d'agression", help: "Multiplicateur du rayon d'agression. 0 = les créatures n'attaquent jamais d'elles-mêmes.", kind: Kind::Float { min: 0.0, max: 3.0 }, default: "1" },
    Opt { file: W, key: "Rate.Damage.Fall", group: "Difficulté des créatures", label: "Dégâts de chute", help: "", kind: Kind::Float { min: 0.0, max: 5.0 }, default: "1" },
    Opt { file: W, key: "Death.SicknessLevel", group: "Difficulté des créatures", label: "Niveau à partir duquel la faiblesse de résurrection s'applique", help: "11 = officiel. 81 = jamais.", kind: Kind::Int { min: -10, max: 81 }, default: "11" },
    Opt { file: W, key: "Death.CorpseReclaimDelay.PvE", group: "Difficulté des créatures", label: "Délai croissant de récupération du corps (JcE)", help: "", kind: ON_OFF, default: "1" },
    Opt { file: W, key: "Battleground.CastDeserter", group: "Difficulté des créatures", label: "Malus de déserteur en quittant un champ de bataille", help: "", kind: ON_OFF, default: "1" },
    Opt { file: W, key: "DungeonFinder.CastDeserter", group: "Difficulté des créatures", label: "Malus de déserteur en quittant un donjon aléatoire", help: "", kind: ON_OFF, default: "1" },
    // ---- Factions
    Opt { file: W, key: "AllowTwoSide.Accounts", group: "Factions et social", label: "Alliance et Horde sur le même compte", help: "Pratique en solo pour essayer les deux factions.", kind: ON_OFF, default: "1" },
    Opt { file: W, key: "AllowTwoSide.Interaction.Chat", group: "Factions et social", label: "Chat entre factions", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "AllowTwoSide.Interaction.Channel", group: "Factions et social", label: "Canaux communs aux deux factions", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "AllowTwoSide.Interaction.Group", group: "Factions et social", label: "Groupes mixtes", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "AllowTwoSide.Interaction.Guild", group: "Factions et social", label: "Guildes mixtes", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "AllowTwoSide.Interaction.Auction", group: "Factions et social", label: "Hôtel des ventes commun", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "AllowTwoSide.Interaction.Mail", group: "Factions et social", label: "Courrier entre factions", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "AllowTwoSide.WhoList", group: "Factions et social", label: "/qui commun aux deux factions", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "AllowTwoSide.AddFriend", group: "Factions et social", label: "Amis de l'autre faction", help: "", kind: ON_OFF, default: "0" },
    // ---- Maître de jeu
    Opt { file: W, key: "GM.LoginState", group: "Maître de jeu (compte GM)", label: "Mode MJ à la connexion", help: "« Désactivé » conseillé pour jouer normalement : les commandes restent disponibles, le mode MJ s'active avec .gm on.", kind: Kind::Choice(&[("0", "Désactivé"), ("1", "Activé"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { file: W, key: "GM.Visible", group: "Maître de jeu (compte GM)", label: "Visibilité du MJ à la connexion", help: "", kind: Kind::Choice(&[("0", "Invisible"), ("1", "Visible"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { file: W, key: "GM.Chat", group: "Maître de jeu (compte GM)", label: "Badge MJ dans le chat à la connexion", help: "", kind: Kind::Choice(&[("0", "Désactivé"), ("1", "Activé"), ("2", "Dernier état enregistré")]), default: "2" },
    Opt { file: W, key: "GM.AllowInvite", group: "Maître de jeu (compte GM)", label: "Les joueurs peuvent inviter un MJ en groupe", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "GM.AllowFriend", group: "Maître de jeu (compte GM)", label: "Les joueurs peuvent ajouter un MJ en ami", help: "", kind: ON_OFF, default: "0" },
    // ---- Technique
    Opt { file: W, key: "Visibility.Distance.Continents", group: "Technique", label: "Distance d'affichage en extérieur", help: "En mètres (45 à 333). Plus grand = plus de charge.", kind: Kind::Int { min: 45, max: 333 }, default: "100" },
    Opt { file: W, key: "Visibility.Distance.Instances", group: "Technique", label: "Distance d'affichage en instance", help: "", kind: Kind::Int { min: 45, max: 333 }, default: "170" },
    Opt { file: W, key: "Instance.UnloadDelay", group: "Technique", label: "Déchargement des instances vides (ms)", help: "", kind: Kind::Int { min: 60_000, max: 7_200_000 }, default: "1800000" },
    Opt { file: W, key: "Event.Announce", group: "Technique", label: "Annoncer les événements mondiaux", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "PreserveCustomChannels", group: "Technique", label: "Conserver les canaux de discussion personnalisés", help: "", kind: ON_OFF, default: "0" },
    Opt { file: W, key: "MaxWhoListReturns", group: "Technique", label: "Résultats maximum de /qui", help: "", kind: Kind::Int { min: 1, max: 49 }, default: "49" },
];

/// Valeur effective (surcharge ou défaut).
pub fn value<'a>(overrides: &'a std::collections::BTreeMap<String, String>, opt: &'a Opt) -> &'a str {
    overrides.get(opt.key).map(String::as_str).unwrap_or(opt.default)
}

/// Clés surchargeables (liste blanche).
pub fn is_known(key: &str) -> bool {
    BOT_OPTIONS.iter().chain(WORLD_OPTIONS.iter()).any(|o| o.key == key)
}

/// Valeur formatée pour un .conf (les textes entre guillemets).
pub fn conf_value(opt: &Opt, v: &str) -> String {
    match opt.kind {
        Kind::Text => format!("\"{}\"", v.replace('"', "'")),
        _ => v.to_string(),
    }
}

pub fn options_for(file: ConfFile) -> impl Iterator<Item = &'static Opt> {
    BOT_OPTIONS.iter().chain(WORLD_OPTIONS.iter()).filter(move |o| o.file == file)
}
