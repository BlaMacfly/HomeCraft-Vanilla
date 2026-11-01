//! Traduction anglaise du catalogue de réglages (`conf_catalog`) : libellé et aide par clé, groupes et choix.
//! Une clé absente garde son texte français (test `toutes_les_cles_traduites`).

use crate::conf_catalog::Opt;
use crate::i18n::is_en;

const OPTS: &[(&str, &str, &str)] = &[
    ("AiPlayerbot.RandomBotAutologin", "Log in the random bots", "Bot characters live in the world: they level up, do their quests, hunt, trade, form groups and guilds. On the first start, the server creates their accounts and characters (a few minutes)."),
    ("AiPlayerbot.MinRandomBots", "Minimum number of bots online", "The server keeps a number of bots online between the minimum and the maximum, and renews some of them from time to time. Each bot costs memory and CPU: 200 bots suit an ordinary PC, 1,000 need a powerful machine. (HomeCraft: 100; CMaNGOS template: 1000)"),
    ("AiPlayerbot.MaxRandomBots", "Maximum number of bots online", "Bot accounts (10 characters each) are created from this number when the server starts. (HomeCraft: 200; CMaNGOS template: 1000)"),
    ("AiPlayerbot.RandomBotMinLevel", "Minimum bot level", ""),
    ("AiPlayerbot.RandomBotMaxLevel", "Maximum bot level", ""),
    ("AiPlayerbot.RandomBotMaxLevelChance", "Share of bots at maximum level", "Between 0 and 1. 0.15 = 15% of the bots are created at the maximum level, the others at a random level."),
    ("AiPlayerbot.RandomBotJoinLfg", "Bots use the group finder", ""),
    ("AiPlayerbot.RandomBotJoinBG", "Bots join battlegrounds", ""),
    ("AiPlayerbot.RandomBotAutoJoinBG", "Battlegrounds started automatically by bots", "Bots queue on their own so that battlegrounds keep running."),
    ("AiPlayerbot.RandomBotGroupNearby", "Bots group with each other", ""),
    ("AiPlayerbot.RandomBotGuildNearby", "Bots join guilds", ""),
    ("AiPlayerbot.RandomBotFormGuild", "Bots found guilds", ""),
    ("AiPlayerbot.EnableBroadcasts", "Bots talk in the channels", "Loot, quests, group search… Sentences are translated into French for a French client."),
    ("AiPlayerbot.EnableGreet", "Bots greet players", ""),
    ("AiPlayerbot.RandomBotTeleportNearPlayer", "Bring bots near you", "Some bots appear in your zone instead of at random in the world."),
    ("AiPlayerbot.RandomBotInvitePlayer", "Bots invite you into their group", ""),
    ("AiPlayerbot.LevelCheck", "Maximum level gap to accept an invitation", "A bot refuses your invitation if its level exceeds yours by more than this number. 60 = it always accepts, whatever your level. (HomeCraft: 60; CMaNGOS template: 30)"),
    ("AiPlayerbot.GearScoreCheck", "Refuse if your gear is too weak", ""),
    ("AiPlayerbot.SyncQuestWithPlayer", "Group bots take your quests", ""),
    ("AiPlayerbot.AutoDoQuests", "Bots do their quests on their own", ""),
    ("AiPlayerbot.AutoEquipUpgradeLoot", "Bots equip loot better than theirs", ""),
    ("AiPlayerbot.AllowGuildBots", "Command the bots of your guild", ""),
    ("AiPlayerbot.BotCheats", "Perks given to your bots", "Comma-separated list: taxi (free flights), item (spell reagents and ammunition), breath (underwater breathing). Empty = none."),
    ("AiPlayerbot.botActiveAlone", "Share of bots active far from you (%)", "Bots far from any player only think part of the time. Higher = livelier world, more load."),
    ("AiPlayerbot.RandomBotsMaxLoginsPerInterval", "Bots logged in per wave", "At startup, bots log in by waves of this size."),
    ("GameType", "Realm type", "Normal: no unwanted PvP."),
    ("Motd", "Message of the day", "Shown in the chat at login."),
    ("StartPlayerLevel", "Level of new characters", ""),
    ("StartPlayerMoney", "Starting money (copper coins)", "10000 = 1 gold coin."),
    ("SkipCinematics", "Introduction cinematic", ""),
    ("CharactersPerRealm", "Characters per realm", ""),
    ("Instance.IgnoreLevel", "Ignore the level required by instances", ""),
    ("Instance.ResetTimeHour", "Raid reset hour", "Local time (0-23)."),
    ("Quests.LowLevelHideDiff", "Hide quests that are too low (level gap)", "Exclamation marks of quests more than N levels below are no longer shown. -1 = never."),
    ("Quests.HighLevelHideDiff", "Hide quests that are too high (level gap)", "-1 = never."),
    ("Quests.IgnoreRaid", "Allow raid quests in a normal group", "Handy solo with bots."),
    ("MaxPrimaryTradeSkill", "Maximum primary professions", ""),
    ("MinPetitionSigns", "Signatures required to found a guild", "0 lets you create a guild alone."),
    ("Rate.XP.Kill", "Experience from kills", "Multiplier. 1 = Vanilla."),
    ("Rate.XP.Quest", "Experience from quests", ""),
    ("Rate.XP.Explore", "Exploration experience", ""),
    ("Rate.Rest.InGame", "Rest gained in game", ""),
    ("Rate.Rest.Offline.InTavernOrCity", "Offline rest in an inn or a city", ""),
    ("Rate.Rest.Offline.InWilderness", "Offline rest outdoors", ""),
    ("Rate.Talent", "Talent points", ""),
    ("Rate.Reputation.Gain", "Reputation gain", ""),
    ("Rate.Reputation.LowLevel.Kill", "Reputation from grey creatures", ""),
    ("SkillGain.Crafting", "Crafting profession progress", "Points gained per item crafted."),
    ("SkillGain.Gathering", "Gathering profession progress", ""),
    ("SkillGain.Weapon", "Weapon skill progress", ""),
    ("SkillGain.Defense", "Defense progress", ""),
    ("Rate.Mining.Amount", "Ore per mining node", ""),
    ("MaxGroupXPDistance", "Group experience sharing distance", "In metres."),
    ("Rate.Drop.Item.Poor", "Loot: grey items", "Loot chance multipliers."),
    ("Rate.Drop.Item.Normal", "Loot: white items", ""),
    ("Rate.Drop.Item.Uncommon", "Loot: green items", ""),
    ("Rate.Drop.Item.Rare", "Loot: blue items", ""),
    ("Rate.Drop.Item.Epic", "Loot: epic items", ""),
    ("Rate.Drop.Item.Legendary", "Loot: legendary items", ""),
    ("Rate.Drop.Item.Referenced", "Loot: referenced tables", ""),
    ("Rate.Drop.Money", "Money found on creatures", ""),
    ("MailDeliveryDelay", "Mail delivery delay (seconds)", "3600 = 1 hour (Vanilla). 0 = immediate."),
    ("Rate.Creature.Normal.Damage", "Damage of normal creatures", "Multipliers. Solo with companions, 0.8 to 1.0 is comfortable."),
    ("Rate.Creature.Normal.HP", "Health of normal creatures", ""),
    ("Rate.Creature.Normal.SpellDamage", "Spell damage of normal creatures", ""),
    ("Rate.Creature.Elite.Elite.Damage", "Damage of elites", ""),
    ("Rate.Creature.Elite.Elite.HP", "Health of elites", ""),
    ("Rate.Creature.Elite.Elite.SpellDamage", "Spell damage of elites", ""),
    ("Rate.Creature.Elite.RAREELITE.HP", "Health of rare elites", ""),
    ("Rate.Creature.Elite.RAREELITE.Damage", "Damage of rare elites", ""),
    ("Rate.Creature.Elite.WORLDBOSS.HP", "Health of world bosses", ""),
    ("Rate.Creature.Elite.WORLDBOSS.Damage", "Damage of world bosses", ""),
    ("Rate.Creature.Aggro", "Aggro distance", "Multiplier of the aggro radius. 0 = creatures never attack on their own."),
    ("Death.SicknessLevel", "Level from which resurrection sickness applies", "11 = Vanilla. 61 = never."),
    ("Death.CorpseReclaimDelay.PvE", "Increasing corpse reclaim delay (PvE)", ""),
    ("Battleground.CastDeserter", "Deserter debuff when leaving a battleground", ""),
    ("AllowTwoSide.Accounts", "Alliance and Horde on the same account", "Handy solo to try both factions."),
    ("AllowTwoSide.Interaction.Chat", "Chat between factions", ""),
    ("AllowTwoSide.Interaction.Channel", "Channels shared by both factions", ""),
    ("AllowTwoSide.Interaction.Group", "Mixed groups", ""),
    ("AllowTwoSide.Interaction.Guild", "Mixed guilds", ""),
    ("AllowTwoSide.Interaction.Trade", "Trade between factions", ""),
    ("AllowTwoSide.Interaction.Mail", "Mail between factions", ""),
    ("AllowTwoSide.WhoList", "/who shared by both factions", ""),
    ("AllowTwoSide.AddFriend", "Friends from the other faction", ""),
    ("HomeCraft.DisableAfkKick", "No disconnection for inactivity", "An away (AFK) character is no longer sent back to the character screen after 15 minutes. A connection that is really lost is still detected and cleaned up. (HomeCraft: on; CMaNGOS: off)"),
    ("HomeCraft.AllowMultipleSessionsPerAccount", "Several characters of one account at the same time (multiboxing)", "Open the game several times and log a different character of the same account into each. Without this option, a second client logging in disconnects the first one. (HomeCraft: on)"),
    ("HomeCraft.MaxSessionsPerAccount", "Simultaneous sessions per account", "Maximum number of clients connected at the same time on one account (1 to 10)."),
    ("GM.LoginState", "GM mode at login", "A GM-level account (2 or 3) logs in with GM mode according to this setting. \"Off\" is advised to play normally: commands stay available, GM mode is turned on with .gm on."),
    ("GM.Visible", "GM visibility at login", ""),
    ("Visibility.Distance.Continents", "Outdoor view distance", "In metres (45 to 333). Higher = more load."),
    ("Visibility.Distance.Instances", "View distance in instances", ""),
    ("PlayerSave.Interval", "Character save interval (ms)", "900000 = 15 minutes. Launcher backups force a saveall anyway."),
    ("Instance.UnloadDelay", "Unloading of empty instances (ms)", ""),
    ("Event.Announce", "Announce world events", ""),
];

const GROUPS: &[(&str, &str)] = &[
    ("Bots aléatoires (monde peuplé)", "Random bots (populated world)"),
    ("Invitations et groupe", "Invitations and group"),
    ("Performances", "Performance"),
    ("Règles du monde", "World rules"),
    ("Expérience et progression", "Experience and progression"),
    ("Butin et économie", "Loot and economy"),
    ("Difficulté des créatures", "Creature difficulty"),
    ("Factions et social", "Factions and social"),
    ("Sessions", "Sessions"),
    ("Maître de jeu (compte GM)", "Game master (GM account)"),
    ("Technique", "Technical"),
];

const CHOICES: &[(&str, &str)] = &[
    ("Normal (JcE)", "Normal (PvE)"),
    ("JcJ", "PvP"),
    ("Jeu de rôle", "Role-playing"),
    ("Jeu de rôle JcJ", "Role-playing PvP"),
    ("Toujours", "Always"),
    ("Seulement au premier personnage de la race", "Only for the first character of the race"),
    ("Jamais", "Never"),
    ("Désactivé", "Off"),
    ("Activé", "On"),
    ("Dernier état enregistré", "Last saved state"),
    ("Invisible", "Invisible"),
    ("Visible", "Visible"),
];

pub fn label(o: &Opt) -> &'static str {
    if is_en() { OPTS.iter().find(|x| x.0 == o.key).map(|x| x.1).unwrap_or(o.label) } else { o.label }
}

pub fn help(o: &Opt) -> &'static str {
    if is_en() { OPTS.iter().find(|x| x.0 == o.key).map(|x| x.2).unwrap_or(o.help) } else { o.help }
}

pub fn group(g: &'static str) -> &'static str {
    if is_en() { GROUPS.iter().find(|x| x.0 == g).map(|x| x.1).unwrap_or(g) } else { g }
}

pub fn choice(c: &'static str) -> &'static str {
    if is_en() { CHOICES.iter().find(|x| x.0 == c).map(|x| x.1).unwrap_or(c) } else { c }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conf_catalog::{BOT_OPTIONS, Kind, WORLD_OPTIONS};

    #[test]
    fn toutes_les_cles_traduites() {
        for o in BOT_OPTIONS.iter().chain(WORLD_OPTIONS) {
            let t = OPTS.iter().find(|x| x.0 == o.key).unwrap_or_else(|| panic!("clé non traduite : {}", o.key));
            assert_eq!(t.2.is_empty(), o.help.is_empty(), "aide à traduire : {}", o.key);
            assert!(GROUPS.iter().any(|g| g.0 == o.group), "groupe non traduit : {}", o.group);
            if let Kind::Choice(c) = o.kind {
                for (_, l) in c {
                    assert!(CHOICES.iter().any(|x| x.0 == *l), "choix non traduit : {l}");
                }
            }
        }
    }
}
