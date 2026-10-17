//! Bots aléatoires du module cmangos/playerbots. Contrairement au Vanilla (VMaNGOS, personnages bots générés par le
//! launcher dans characters.playerbot), le module crée lui-même ses comptes (préfixe RNDBOT) et ses personnages au
//! démarrage du serveur, selon AiPlayerbot.MinRandomBots / MaxRandomBots (onglet PlayerBots). Le launcher se limite à :
//! - régler ce nombre (appliqué au prochain démarrage) ;
//! - supprimer tous les bots, serveur arrêté : comptes RNDBOT, leurs personnages, la table ai_playerbot_random_bots
//!   et les guildes devenues orphelines.

use crate::db::{MariaDb, BOT_ACCOUNT_PREFIX};
use crate::paths::DB_CHARACTERS;
use crate::paths::DB_REALMD;
use mysql::prelude::*;

/// Réglages du catalogue à mettre à jour pour demander `count` bots aléatoires.
pub fn count_overrides(count: u32) -> [(&'static str, String); 2] {
    [("AiPlayerbot.MinRandomBots", count.to_string()), ("AiPlayerbot.MaxRandomBots", count.to_string())]
}

/// Supprime tous les bots aléatoires (serveur arrêté). Renvoie le nombre de personnages supprimés.
pub fn delete_all(db: &MariaDb) -> Result<u64, String> {
    let err = |e: mysql::Error| e.to_string();
    let accounts: Vec<String> = {
        let mut c = db.root_conn()?;
        c.query(format!("SELECT username FROM {DB_REALMD}.account WHERE username LIKE '{BOT_ACCOUNT_PREFIX}%'")).map_err(err)?
    };
    let mut chars = 0;
    for a in &accounts {
        chars += db.delete_account_offline(a)?;
    }
    let mut c = db.root_conn()?;
    c.query_drop(format!("DELETE FROM {DB_CHARACTERS}.ai_playerbot_random_bots")).map_err(err)?;
    // Guildes de bots : guildes dont le chef n'existe plus, puis toutes les données rattachées à une guilde disparue.
    c.query_drop(format!("DELETE FROM {DB_CHARACTERS}.guild WHERE leaderguid NOT IN (SELECT guid FROM {DB_CHARACTERS}.characters)")).map_err(err)?;
    let tables: Vec<String> = c
        .query(format!("SELECT table_name FROM information_schema.columns WHERE table_schema='{DB_CHARACTERS}' AND column_name='guildid' AND table_name<>'guild'"))
        .map_err(err)?;
    for t in tables {
        c.query_drop(format!("DELETE FROM {DB_CHARACTERS}.`{t}` WHERE guildid NOT IN (SELECT guildid FROM {DB_CHARACTERS}.guild)")).map_err(err)?;
    }
    Ok(chars)
}
