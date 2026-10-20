//! Génération de personnages bots pour le peuplement des capitales (table characters.playerbot, IA
//! StormwindPopulationAI / IronforgePopulationAI / OrgrimmarPopulationAI de VMaNGOS).
//! Les personnages sont créés en base comme le ferait le serveur à la création (position, objets et
//! points de rappel de `playercreateinfo*`), sur un compte dédié HOMECRAFTBOTS sans mot de passe.

use crate::db::MariaDb;
use mysql::prelude::*;
use std::collections::HashSet;

pub const BOT_ACCOUNT: &str = "HOMECRAFTBOTS";

/// Masque de phase par défaut d'un personnage (VMaNGOS `WORLD_DEFAULT_CHAR` = 0x01). À 0, le bot n'apparaît pas
/// dans /qui (`Player::IsVisibleGloballyFor` → `CanSeeInWorld`) ; la visibilité en jeu, elle, n'en dépend pas.
const WORLD_DEFAULT_MASK: u32 = 1;

/// Bots créés jusqu'à la version 0.9.12 avec un masque de phase à 0 : invisibles dans /qui. Renvoie le nombre corrigé.
pub fn fix_phase_mask(db: &MariaDb) -> Result<u64, String> {
    let mut c = db.root_conn()?;
    c.query_drop(format!(
        "UPDATE characters.characters c JOIN realmd.account a ON a.id=c.account SET c.world_phase_mask={WORLD_DEFAULT_MASK} \
         WHERE a.username='{BOT_ACCOUNT}' AND c.world_phase_mask=0"
    ))
    .map_err(|e| e.to_string())?;
    Ok(c.affected_rows())
}

/// Personnages joueurs (hors compte des bots) actuellement en jeu.
pub fn players_in_world(db: &MariaDb) -> Result<u64, String> {
    let mut c = db.root_conn()?;
    c.query_first::<u64, _>(format!(
        "SELECT COUNT(*) FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE c.online=1 AND a.username<>'{BOT_ACCOUNT}'"
    ))
    .map_err(|e| e.to_string())
    .map(|n| n.unwrap_or(0))
}

/// Jusqu'à 0.9.13, des bots générés serveur en marche pouvaient porter le numéro du prochain personnage créé en jeu :
/// le serveur écrasait le bot par le personnage du joueur, mais l'inscription au peuplement restait. Retire toute
/// inscription qui ne désigne pas un personnage du compte des bots (sinon le serveur connecterait le personnage du
/// joueur comme bot). Renvoie le nombre retiré.
pub fn drop_foreign_entries(db: &MariaDb) -> Result<u64, String> {
    let mut c = db.root_conn()?;
    // Sans base par défaut sur la connexion, MariaDB refuse un DELETE multi-tables par alias (erreur 1046) :
    // sous-requête sur des noms complets.
    c.query_drop(format!(
        "DELETE FROM characters.playerbot WHERE char_guid NOT IN (SELECT c.guid FROM characters.characters c \
         JOIN realmd.account a ON a.id=c.account WHERE a.username='{BOT_ACCOUNT}')"
    ))
    .map_err(|e| e.to_string())?;
    Ok(c.affected_rows())
}

#[derive(Clone, Debug, Default)]
pub struct GenReport {
    pub created: usize,
    pub alliance: usize,
    pub horde: usize,
}

// (race, classes valides en 1.12) — races : 1 humain, 2 orc, 3 nain, 4 elfe de la nuit, 5 mort-vivant, 6 tauren, 7 gnome, 8 troll
const ALLIANCE: &[(u8, &[u8])] = &[(1, &[1, 2, 4, 5, 8, 9]), (3, &[1, 2, 3, 4, 5]), (4, &[1, 3, 4, 5, 11]), (7, &[1, 4, 8, 9])];
const HORDE: &[(u8, &[u8])] = &[(2, &[1, 3, 4, 7, 9]), (5, &[1, 4, 5, 8, 9]), (6, &[1, 3, 7, 11]), (8, &[1, 3, 4, 5, 7, 8])];

fn ai_for_race(race: u8) -> &'static str {
    match race {
        3 | 7 => "IronforgePopulationAI",
        1 | 4 => "StormwindPopulationAI",
        _ => "OrgrimmarPopulationAI",
    }
}

/// Emplacement d'équipement (slot) selon inventory_type ; None = sac à dos.
fn equip_slot(inventory_type: u32) -> Option<u8> {
    Some(match inventory_type {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        5 | 20 => 4,
        6 => 5,
        7 => 6,
        8 => 7,
        9 => 8,
        10 => 9,
        11 => 10,
        12 => 12,
        16 => 14,
        13 | 17 | 21 => 15,
        14 | 22 | 23 => 16,
        15 | 25 | 26 | 28 => 17,
        19 => 18,
        _ => return None,
    })
}

struct Rng(u64);
impl Rng {
    fn new() -> Rng {
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(42) ^ rand::random::<u64>();
        Rng(seed | 1)
    }
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

const SYL_A: &[&str] = &["Al", "Bre", "Ca", "Da", "El", "Fa", "Ga", "Ha", "Il", "Jo", "Ka", "Le", "Ma", "Na", "Or", "Pe", "Ra", "Sa", "Ta", "Ul", "Va", "Wi", "Ys", "Ze"];
const SYL_B: &[&str] = &["dan", "lin", "mir", "nor", "ras", "thel", "vin", "wen", "dor", "gar", "lis", "mon", "rel", "sin", "tan", "ven", "zar", "bel", "cor", "fin", "lor", "nis", "rin", "sol"];
const SYL_C: &[&str] = &["", "", "a", "e", "is", "us", "on", "ia", "ar", "el", "or", "in"];

/// Nom unique, lettres seulement (le jeu refuse les chiffres). Trois syllabes donnent environ 6 000 noms ; au-delà
/// (plusieurs milliers de bots), une syllabe de plus porte l'éventail à environ 150 000.
fn make_name(rng: &mut Rng, used: &mut HashSet<String>) -> String {
    for attempt in 0..20_000 {
        let mut n = if attempt < 50 {
            format!("{}{}{}", rng.pick(SYL_A), rng.pick(SYL_B), rng.pick(SYL_C))
        } else {
            format!("{}{}{}{}", rng.pick(SYL_A), rng.pick(SYL_B), rng.pick(SYL_B), rng.pick(SYL_C))
        };
        n.truncate(12);
        let key = n.to_ascii_uppercase();
        if n.len() >= 3 && !used.contains(&key) {
            used.insert(key);
            return n;
        }
    }
    // Repli théorique : suffixe en lettres dérivé d'un compteur.
    let mut i = used.len();
    loop {
        let mut suffix = String::new();
        let mut k = i;
        loop {
            suffix.push((b'a' + (k % 26) as u8) as char);
            k /= 26;
            if k == 0 {
                break;
            }
        }
        let n = format!("Zor{suffix}");
        if used.insert(n.to_ascii_uppercase()) {
            return n;
        }
        i += 1;
    }
}

#[derive(Clone)]
struct StartItem {
    entry: u32,
    amount: u32,
    inventory_type: u32,
    max_durability: u32,
    charges: String,
    flags: u32,
}

/// Crée `count` personnages bots (moitié Alliance, moitié Horde). `progress(i, n)`.
pub fn generate(db: &MariaDb, count: u32, progress: &dyn Fn(usize, usize)) -> Result<GenReport, String> {
    let mut c = db.root_conn()?;
    let err = |e: mysql::Error| e.to_string();

    // Compte dédié
    let account: u32 = match c.query_first::<u32, _>(format!("SELECT id FROM realmd.account WHERE username='{BOT_ACCOUNT}'")).map_err(err)? {
        Some(id) => id,
        None => {
            c.query_drop(format!("INSERT INTO realmd.account (username, v, s, locale, gmlevel) VALUES ('{BOT_ACCOUNT}', '', '', 2, 0)")).map_err(err)?;
            c.query_first::<u32, _>("SELECT LAST_INSERT_ID()").map_err(err)?.ok_or("compte bots : identifiant introuvable")?
        }
    };

    // Données de création
    let starts: Vec<(u8, u8, u32, u32, f32, f32, f32, f32)> = c
        .query("SELECT race, class, map, zone, position_x, position_y, position_z, orientation FROM mangos.playercreateinfo")
        .map_err(err)?;
    let raw_items: Vec<(u8, u8, u32, u32)> = c.query("SELECT race, class, itemid, amount FROM mangos.playercreateinfo_item").map_err(err)?;
    let templates: Vec<(u32, u32, u32, i32, i32, i32, i32, i32, u32)> = c
        .query(
            "SELECT t.entry, t.inventory_type, t.max_durability, t.spellcharges_1, t.spellcharges_2, t.spellcharges_3, t.spellcharges_4, t.spellcharges_5, t.flags \
             FROM mangos.item_template t JOIN (SELECT entry, MAX(patch) AS patch FROM mangos.item_template WHERE patch <= 10 GROUP BY entry) l ON l.entry=t.entry AND l.patch=t.patch \
             WHERE t.entry IN (SELECT DISTINCT itemid FROM mangos.playercreateinfo_item)",
        )
        .map_err(err)?;
    let template = |entry: u32| templates.iter().find(|t| t.0 == entry);
    let items_for = |race: u8, class: u8| -> Vec<StartItem> {
        raw_items
            .iter()
            .filter(|(r, cl, _, _)| *r == race && *cl == class)
            .filter_map(|(_, _, entry, amount)| {
                let t = template(*entry)?;
                Some(StartItem { entry: *entry, amount: *amount, inventory_type: t.1, max_durability: t.2, charges: format!("{} {} {} {} {}", t.3, t.4, t.5, t.6, t.7), flags: t.8 })
            })
            .collect()
    };
    let stats: Vec<(u8, u8, u32, u32)> = c.query("SELECT class, level, basehp, basemana FROM mangos.player_classlevelstats").map_err(err)?;

    let existing: Vec<String> = c.query("SELECT name FROM characters.characters").map_err(err)?;
    let mut used: HashSet<String> = existing.into_iter().map(|n| n.to_ascii_uppercase()).collect();
    let mut next_guid: u32 = c.query_first::<Option<u32>, _>("SELECT MAX(guid) FROM characters.characters").map_err(err)?.flatten().unwrap_or(0) + 1;
    let mut next_item: u32 = c.query_first::<Option<u32>, _>("SELECT MAX(guid) FROM characters.item_instance").map_err(err)?.flatten().unwrap_or(0) + 1;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

    let mut rng = Rng::new();
    let mut report = GenReport::default();
    let total = count as usize;
    for i in 0..total {
        progress(i + 1, total);
        let alliance = i % 2 == 0;
        let (race, classes) = *rng.pick(if alliance { ALLIANCE } else { HORDE });
        let class = *rng.pick(classes);
        let gender = rng.below(2) as u8;
        let level = (1 + rng.below(12) + rng.below(12) + rng.below(12) + rng.below(12) + rng.below(12)) as u8; // 1..=56, centré vers 25
        let name = make_name(&mut rng, &mut used);
        let Some(start) = starts.iter().find(|s| s.0 == race && s.1 == class) else { continue };
        let (basehp, basemana) = stats.iter().find(|s| s.0 == class && s.1 == level).map(|s| (s.2, s.3)).unwrap_or((100, 100));
        let uses_mana = matches!(class, 2 | 3 | 5 | 7 | 8 | 9 | 11);
        let guid = next_guid;
        next_guid += 1;

        // Objets de départ : équipés selon leur type, sinon dans le sac à dos (emplacements 23+)
        let mut equipment = vec![(0u32, 0u32); 19];
        let mut backpack_slot = 23u8;
        let mut item_rows: Vec<String> = Vec::new();
        let mut inv_rows: Vec<String> = Vec::new();
        for it in items_for(race, class) {
            let slot = match equip_slot(it.inventory_type) {
                Some(s) if equipment[s as usize].0 == 0 => s,
                _ => {
                    let s = backpack_slot;
                    backpack_slot += 1;
                    if s > 38 {
                        continue;
                    }
                    s
                }
            };
            if slot < 19 {
                equipment[slot as usize] = (it.entry, 0);
            }
            let item_guid = next_item;
            next_item += 1;
            let enchants = "0 ".repeat(21);
            item_rows.push(format!("({item_guid},{},{guid},0,0,{},0,'{}',{},'{enchants}',0,{},0,0)", it.entry, it.amount, it.charges, it.flags, it.max_durability));
            inv_rows.push(format!("({guid},0,{slot},{item_guid},{})", it.entry));
        }
        let equipment_cache: String = equipment.iter().map(|(e, en)| format!("{e} {en} ")).collect();
        let explored: String = "0 ".repeat(64);
        let taxi = if alliance { "2 0 0 0 0 0 0 0 " } else { "0 0 0 0 0 0 0 0 " };

        c.query_drop(format!(
            "INSERT INTO characters.characters (guid, account, name, race, class, gender, skin, face, hair_style, hair_color, facial_hair, level, xp, money, zone, map, position_x, position_y, position_z, orientation, known_taxi_mask, current_taxi_path, online, create_time, logout_time, health, power1, power2, power3, power4, power5, explored_zones, equipment_cache, action_bars, world_phase_mask) \
             VALUES ({guid},{account},'{name}',{race},{class},{gender},{},{},{},{},{},{level},0,{},{},{},{},{},{},{},'{taxi}','',0,{now},{now},{},{},0,0,{},0,'{explored}','{equipment_cache}',0,{WORLD_DEFAULT_MASK})",
            rng.below(4), rng.below(5), rng.below(6), rng.below(6), rng.below(4),
            rng.below(5000), start.3, start.2, start.4, start.5, start.6, start.7,
            basehp.max(10) * 2, if uses_mana { basemana.max(10) * 2 } else { 0 }, if class == 4 { 100 } else { 0 }
        ))
        .map_err(err)?;
        c.query_drop(format!("INSERT INTO characters.character_homebind (guid, map, zone, position_x, position_y, position_z) VALUES ({guid},{},{},{},{},{})", start.2, start.3, start.4, start.5, start.6)).map_err(err)?;
        if !item_rows.is_empty() {
            c.query_drop(format!("INSERT INTO characters.item_instance (guid, item_id, owner_guid, creator_guid, gift_creator_guid, count, duration, charges, flags, enchantments, random_property_id, durability, text, generated_loot) VALUES {}", item_rows.join(","))).map_err(err)?;
            c.query_drop(format!("INSERT INTO characters.character_inventory (guid, bag, slot, item_guid, item_id) VALUES {}", inv_rows.join(","))).map_err(err)?;
        }
        c.query_drop(format!("INSERT INTO characters.playerbot (char_guid, chance, comment, ai) VALUES ({guid},10,'HomeCraft peuplement','{}')", ai_for_race(race))).map_err(err)?;
        report.created += 1;
        if alliance { report.alliance += 1 } else { report.horde += 1 }
    }
    Ok(report)
}

/// Supprime tous les personnages du compte bots (et leurs objets, rappels, entrées playerbot).
pub fn delete_all(db: &MariaDb) -> Result<usize, String> {
    let mut c = db.root_conn()?;
    let err = |e: mysql::Error| e.to_string();
    let Some(account) = c.query_first::<u32, _>(format!("SELECT id FROM realmd.account WHERE username='{BOT_ACCOUNT}'")).map_err(err)? else { return Ok(0) };
    let guids: Vec<u32> = c.query(format!("SELECT guid FROM characters.characters WHERE account={account}")).map_err(err)?;
    if guids.is_empty() {
        return Ok(0);
    }
    let list = guids.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(",");
    for sql in [
        format!("DELETE FROM characters.item_instance WHERE owner_guid IN ({list})"),
        format!("DELETE FROM characters.character_inventory WHERE guid IN ({list})"),
        format!("DELETE FROM characters.character_homebind WHERE guid IN ({list})"),
        format!("DELETE FROM characters.playerbot WHERE char_guid IN ({list})"),
        format!("DELETE FROM characters.characters WHERE guid IN ({list})"),
    ] {
        c.query_drop(sql).map_err(err)?;
    }
    Ok(guids.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_thousand_unique_letter_only_names() {
        let mut rng = Rng::new();
        let mut used = HashSet::new();
        for _ in 0..10_000 {
            let n = make_name(&mut rng, &mut used);
            assert!(n.len() >= 3 && n.len() <= 12 && n.chars().all(|c| c.is_ascii_alphabetic()), "{n}");
        }
        assert_eq!(used.len(), 10_000);
    }
}
