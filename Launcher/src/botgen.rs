//! Bots aléatoires du module cmangos/playerbots. Le serveur crée lui-même leurs comptes (RNDBOT0, RNDBOT1…, dix
//! personnages chacun) au démarrage, d'après AiPlayerbot.RandomBotAccountCount (voir `server::random_bot_accounts`),
//! puis les connecte et les fait vivre. Le launcher ne fait que les compter, les distinguer des comptes du joueur
//! et les supprimer.

use crate::db::MariaDb;
use mysql::prelude::*;

/// Préfixe des comptes de bots aléatoires (AiPlayerbot.RandomBotAccountPrefix, imposé par le launcher).
pub const BOT_ACCOUNT_PREFIX: &str = "RNDBOT";

/// Condition SQL « compte de bot aléatoire » sur la colonne `column` (nom d'utilisateur) : le préfixe suivi du numéro.
pub fn bot_account_filter(column: &str) -> String {
    format!("{column} REGEXP '^{BOT_ACCOUNT_PREFIX}[0-9]+$'")
}

/// Nom réservé aux comptes de bots aléatoires (refusé à la création d'un compte joueur).
pub fn is_bot_account(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper.starts_with(BOT_ACCOUNT_PREFIX)
}

/// Personnages joueurs (hors bots) actuellement en jeu.
pub fn players_in_world(db: &MariaDb) -> Result<u64, String> {
    let mut c = db.root_conn()?;
    c.query_first(format!(
        "SELECT COUNT(*) FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE c.online=1 AND NOT {}",
        bot_account_filter("a.username")
    ))
    .map(|v| v.unwrap_or(0))
    .map_err(|e| e.to_string())
}

/// Supprime tous les comptes de bots aléatoires, leurs personnages, leurs guildes et l'état du gestionnaire de bots
/// (serveur arrêté). Au démarrage suivant, le serveur recrée des bots neufs si les bots aléatoires sont activés.
pub fn delete_all(db: &MariaDb) -> Result<u64, String> {
    let mut c = db.root_conn()?;
    let ids: Vec<u32> = c.query(format!("SELECT id FROM realmd.account WHERE {}", bot_account_filter("username"))).map_err(|e| e.to_string())?;
    let n = crate::db::delete_accounts(&mut c, &ids)?;
    // Événements du gestionnaire (prochaine connexion, téléportations…), y compris ceux sans personnage.
    c.query_drop("DELETE FROM characters.ai_playerbot_random_bots").map_err(|e| e.to_string())?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    #[test]
    fn bot_accounts_are_recognised() {
        assert!(super::is_bot_account("RNDBOT12"));
        assert!(super::is_bot_account("rndbotx"));
        assert!(!super::is_bot_account("Marty"));
        assert_eq!(super::bot_account_filter("a.username"), "a.username REGEXP '^RNDBOT[0-9]+$'");
    }
}
