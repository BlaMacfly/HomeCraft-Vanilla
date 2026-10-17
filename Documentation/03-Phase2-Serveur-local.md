# Phase 2 — serveur local : base de données, données client, premier démarrage

Dernière mise à jour : 2026-10-16 matin. Royaume retenu par l'utilisateur : **royaume 1, « The Kingdom of
Sylvania Semi Blizzlike »** (dumps = release GitHub `royaume1-db-2026-10-15` du dépôt privé, **pas encore
téléchargés** : voir §5).

## 1. Base de données : MariaDB rejetée, MySQL 8.4 retenu

L'édition Vanilla utilise MariaDB 11.8.9 portable. **AzerothCore la refuse** : `DatabaseWorkerPool.cpp`
(`DatabaseIncompatibleVersion`, identique dans upstream master du 2026-10-15) compare caractère par caractère la
chaîne `mysql_get_server_info()` à `MIN_MYSQL_SERVER_VERSION "8.0.0"` ; « 11.8.9-MariaDB » commence par « 1 » <
« 8 » → « AzerothCore does not support MySQL versions below 8.0 » puis `ABORT` (sous Windows le processus reste
figé sur la boîte de dialogue d'erreur masquée, 0 % CPU). Toute MariaDB (10.x, 11.x) est donc exclue sans patch
du cœur. Décision : **MySQL Community 8.4.9** (zip `mysql-8.4.9-winx64.zip` déjà vérifié par MD5 pour la
compilation, GPL-2.0 avec exception FOSS), copié sans `docs/`, `include/`, `lib/` (sauf `lib/private`, données
ICU) ni `mysqld-debug.exe` dans `Server/Database/mysql` (860 Mo ; à élaguer encore pour le package :
`mysql_configurator`, `mysqlslap`, `myisam*`, outils keyring…).

Particularités réglées dans `Tools/Start-Database.ps1` :

- `mysqld --initialize-insecure` crée `root@localhost` sans mot de passe ; avec `skip-name-resolve`, ce compte
  **n'accepte pas les connexions TCP** sur 127.0.0.1. Le premier démarrage passe donc par `--init-file`
  (exécuté par le serveur sans connexion) qui pose le mot de passe généré et crée `root@127.0.0.1` ; le fichier
  est supprimé aussitôt.
- `my.ini` géré (régénéré à chaque démarrage) : port 3315, `bind-address=127.0.0.1`, `mysqlx=OFF`, utf8mb4 /
  `utf8mb4_unicode_ci`, `max_allowed_packet=1G`, `innodb_buffer_pool_size=1G`, `innodb_redo_log_capacity=512M`,
  `sql_mode=NO_ENGINE_SUBSTITUTION`, `log_bin=OFF`, journal `Logs/mysql-error.log`.
- Identifiants : `UserData/db-credentials.json` (`acore` + mot de passe root), même format que l'édition Vanilla.
- Le client `mysql.exe` avertit « Using a password on the command line » à chaque appel : cosmétique, à régler
  (variable `MYSQL_PWD` ou fichier d'options) dans le launcher.

## 2. Chemins : la jonction `C:\hcw`

`authserver -c "C:\…\HomeCraft WoTLK\Server\Config\authserver.conf"` échoue avec « the argument … for option
'--config' is invalid » : `boost::program_options` convertit l'argument en `fs::path` par `lexical_cast`, qui
s'arrête au premier espace. Comme pour la compilation (`C:\hc`), une **jonction NTFS `C:\hcw` → la racine du
projet** (créée par `Run-World.ps1`, `Layout.ShortRoot`) sert à lancer les deux serveurs. Les chemins *dans* les
`.conf` (DataDir, LogsDir) supportent les espaces. Le launcher devra reproduire cette astuce (jonction, nom court
8.3 ou répertoire de travail `configs/`).

Autre prérequis d'exécution découvert : `legacy.dll` (fournisseur OpenSSL legacy, `C:\Program Files\OpenSSL-Win64\bin`)
doit être à côté des exécutables, sinon `authserver` s'arrête (« Not found 'legacy.dll' »). Copié dans
`Server/acore/` avec `libmysql.dll`, `libcrypto-3-x64.dll`, `libssl-3-x64.dll`.

## 3. Données client (3.3.5a frFR, `Desktop\wotlk moderna\ref-client`)

`Tools/Extract-ClientData.ps1 -ClientDir … -Steps dbcmaps,vmaps` (extracteurs compilés, `Server/acore`) :

| Étape | Durée | Résultat dans `Server/Data` |
|---|---|---|
| `map_extractor -i <client> -o <data> -e 7` | 1 min | `dbc/` 247 fichiers (87 Mo, locale frFR détectée), `maps/` 5 703 (275 Mo), `Cameras/` 14 |
| `vmap4_extractor -l -d <client>\Data` + `vmap4_assembler Buildings vmaps` | ~2,5 min | `vmaps/` 12 465 fichiers (1 259 Mo) ; `Buildings/` (223 Mo) peut être supprimé ensuite |
| `mmaps_generator --threads 14 --silent` | **19 min** (09:33 → 09:52, 14 threads) | `mmaps/` 3 738 fichiers (2 126 Mo) — indispensables aux playerbots (déplacements) |

`Buildings/` (553 Mo, intermédiaire des vmaps) supprimé après coup. Total `Server/Data` ≈ 3,75 Go.

Piège : appelé avec `powershell -File … -Steps dbcmaps,vmaps`, le paramètre arrive en une seule chaîne ; le
script la découpe lui-même.

## 4. Premier démarrage (sans les dumps : amorçage depuis le dépôt)

`Tools/New-ServerConfig.ps1` génère `authserver.conf`, `worldserver.conf` et `modules/*.conf` (22) depuis les
`.conf.dist` : bases sur 127.0.0.1:3315, `BindIP 127.0.0.1`, ports 3724/8085, `Console.Enable 1`, `Ra`/`SOAP`
désactivés, **`Warden.Enabled 0`** (HermesProxy), `Updates.EnableDatabases 15` (auth+characters+world+playerbots),
`Updates.AutoSetup 1`, `SourceDirectory C:/hc` et `MySQLExecutable` → l'updater importe lui-même
`data/sql/base/*`, les `updates/`, les `pending_*` du fork et les SQL du module. `playerbots.conf` :
`PlayerbotsDatabaseInfo`, `MinRandomBots = MaxRandomBots = <paramètre>` (20 pour le premier test).
Le message du jour n'est pas dans la conf (table `acore_auth.motd`).

`Tools/Init-Database.ps1` : compte `acore`, création des quatre bases (import des dumps de `Server/Database/sql`
quand ils existent ; refus de ré-import si des personnages existent, comme en Vanilla).

`Tools/Run-World.ps1 [-RandomBots N] [-RealmName …]` : MySQL → conf → authserver → `realmlist` (id 1,
127.0.0.1:8085, build 12340) → worldserver avec console sur stdin ; `Send-Console.ps1 "<commande>"` ; arrêt par
le fichier `Logs/server/stop` (« server shutdown 1 »).

Résultats du 2026-10-16 :

- `authserver` : amorçage de `acore_auth` (18 tables, 8 updates) et écoute sur 3724 en 3 s. ✔
- `worldserver` : amorçage de `acore_world` depuis `C:/hc/data/sql/base/db_world` (304 fichiers, ~2,5 min), puis
  **1 090 updates + 1 384 archivées** (dont les 28 `pending_db_world` du fork) ✔ ; `acore_characters` 109 tables,
  `acore_playerbots` 29 tables (SQL du module) ✔.
- Trois arrêts successifs, corrigés :
  1. « Database Playerbots not specified » : le patch Playerbot d'avril lit `PlayerbotsDatabaseInfo` et
     `Playerbots.Updates.EnableDatabases` dans **`worldserver.conf`** (avant les conf de modules) → ajoutés par le
     générateur en fin de fichier (clés hors modèle).
  2. « Missing property CFBG.Enable… » (toutes les clés de modules) : sous Windows, AzerothCore cherche les conf de
     modules dans **`<dossier courant>/configs/modules/`**, pas à côté du fichier passé par `-c`
     (`ConfigMgr::GetConfigPath()` = `"configs/"`). Les configurations vivent donc désormais dans
     **`Server/acore/configs/`** (+ `modules/`) ; `Server/Config/` ne garde que `my.ini`.
  3. « You have _outdated_ DBC data » : message rendu explicite (commit à venir dans la branche :
     `DBCStores.cpp` journalise chaque contrôle) → **`AreaTable.dbc` : 4 911 lignes, entrée 4987 absente ;
     `Map.dbc` : 719 lignes, entrée 724 absente** (max réel 718 = `Transport201834`, pas de `TheRubySanctum`).
     Les DBC extraits du client frFR sont donc au niveau **3.3.3a** pour ces fichiers (alors que `Spell.dbc` contient
     bien 80864) : les `patch-frFR-*.MPQ` de ce client (identiques octet pour octet dans `wotlk-copie`) n'apportent
     pas les DBC 3.3.5. C'est un client frFR « 3.3.5a » recomposé, fréquent sur les miroirs. Conséquence : **les DBC
     du serveur ne peuvent pas venir de ce client**. Voie standard AzerothCore : le pack de données de référence
     `wowgaming/client-data` (release **v20.0** du 2026-07-19, `Data.zip` 1 196 168 257 o : dbc enUS, maps, vmaps,
     **mmaps**, Cameras), qui rendrait aussi inutile la génération locale des mmaps. Alternative sans téléchargement :
     compléter `map_dbc` / `areatable_dbc` en base (mécanisme de surcharge DBC d'AzerothCore) avec les lignes
     manquantes — il faut quand même une source 3.3.5a pour ces lignes.

## 4 bis. Premier démarrage réussi (2026-10-16, 10:20)

Après les DBC enUS 3.3.5a du VPS (dossier `dbc` du serveur de production, copié par SSH → `Server/Data/dbc`,
247 fichiers, 86 Mo, `component.wow-enUS.txt` build 12340 ; le `Spell.dbc` y porte une retouche Sylvania du
2026-06-26) et les compléments ci-dessous, `Run-World.ps1 -RandomBots 20` aboutit à :

- `WORLD: World Initialized In 0 Minutes 13 Seconds`, port 8085 ouvert, worldserver ≈ 3,8 Go de RAM ;
- mod-playerbots : 52 comptes `rndbot*` et 520 personnages créés automatiquement, **20 bots connectés** qui
  acceptent des quêtes, prennent des vols et se téléportent (`Logs/server/Playerbots.log`) ;
- console opérationnelle : `Send-Console.ps1 "server info"`, `account create homecraft <mdp>` puis
  `account set gmlevel homecraft 3 -1` → compte de test GM créé (mot de passe dans `Logs/server/console.in`, à
  changer) ;
- AHBot Sylvania chargé (12 139 prix de marché, 11 866 objets désactivés), transmog, anticheat, etc.

Compléments de base nécessaires à l'amorçage « depuis le dépôt » (le dump du royaume les contient déjà) :

| Manque constaté | Origine | Correctif |
|---|---|---|
| `acore_characters.mythic_plus_dungeon` et 46 autres tables/données de modules | SQL de modules placés directement dans `data\sql\db-world|db-characters|db-auth` (ancienne disposition, pas de `base/`/`updates/`) : l'updater les ignore | `Tools/Import-ModuleSql.ps1` : 47 fichiers importés, journal `UserData/module-sql-applied.json` |
| `acore_world.mod_auctionhousebot_market_prices` | `modules/mod-ah-bot/sylvania_market/market_prices.sql` (12 139 lignes) | importé à la main |
| `acore_world.mod_auctionhousebot_guaranteed_items` | **aucun SQL dans le dépôt** (table créée à la main sur le VPS) | créée vide (`item`, `mincount`, `comment`) |
| `acore_characters.character_martyr_mode`, `character_challenges` | mod-challenge-modes version Sylvania (« Mode Martyr », Hardcore) : requêtes dans `ChallengeModes.cpp`, aucun `CREATE TABLE` | créées au schéma déduit des requêtes |
| `acore_auth.account.vp` | mod-challenge-modes (points de vote) | colonne ajoutée |

Pièges rencontrés : l'outil de recherche de la session respecte `.gitignore`, qui exclut `modules/*` → chercher
explicitement dans `modules/<module>` ; MySQL 8 n'accepte pas `ADD COLUMN IF NOT EXISTS`.

**Test possible dès maintenant avec un client 3.3.5a** (`wotlk moderna\ref-client` ou `wotlk-copie`) : `Data\frFR\realmlist.wtf`
→ `set realmlist 127.0.0.1`, compte `homecraft`. Le serveur reste lancé dans l'onglet de terminal « clone SylvaniaCore »
(arrêt : fichier `Logs\server\stop` ou Ctrl+C).

## 4 ter. Base du royaume 1 importée (2026-10-16, 10:26 → 10:45)

Dumps fournis par l'utilisateur (release `royaume1-db-2026-10-15`, déposés dans `Downloads` puis copiés dans
`Server/Database/sql`) : `acore_world.sql.gz` (80 Mo → 304 Mo, **complet**, 331 tables, MySQL 8.4.7 côté VPS,
« Dump completed on 2026-10-15 23:30:10 »), `acore_auth_schema.sql.gz` (46 tables) et
`acore_characters_schema.sql.gz` (129 tables) **en schéma seul mais avec leurs tables `updates`** (donc pas de
ré-application des updates par le serveur). Toutes les tables Sylvania y sont (`mod_auctionhousebot_guaranteed_items`
203 lignes, `market_prices` 12 139, `character_martyr_mode`, `character_challenges`, mythic+, `account.vp`…).

`Init-Database.ps1 -Force -IKnowThisDeletesCharacters` (seuls des personnages de bots existaient) : import en 75 s.
Trois compléments nécessaires parce que auth/characters sont en schéma seul :

1. `playerbots_names` / `playerbots_guild_names` / `playerbots_arena_team_names` (characters) vides alors que la
   table `updates` du dump les déclare appliquées → remplies depuis `modules/mod-playerbots/data/sql/characters/base`
   (100 000 / 400 / 300 lignes). À automatiser dans `Init-Database.ps1`.
2. `realmlist` vide → `authserver` refuse de démarrer (« No valid realms specified ») : `Run-World.ps1` insère
   maintenant le royaume **avant** de lancer authserver (`INSERT … ON DUPLICATE KEY UPDATE`).
3. `active_arena_season` vide → assertion `ArenaSeasonMgr::LoadActiveSeason` (code de sortie 0xC0000420) :
   `INSERT (season_id 8, season_state 1)`, valeur relevée sur les royaumes Sylvania.

4. **Triggers à définisseur étranger** : le schéma auth embarque trois triggers sur `account`
   (`ts_force_local_ip_insert`/`_update` : forçage d'IP propre à la production ; `trg_account_vp_notify` : notification de
   points de vote) avec `DEFINER='blamacfly'@'%'` / `@'localhost'`, utilisateur inexistant ici. MySQL refuse alors
   **toute écriture sur `account`** (erreur 1449 « The user specified as a definer does not exist »), sans rien dans
   `DBErrors.log` : la console disait « Account created » mais la table restait vide, `account_access` recevait une
   ligne `id 0`, et les bots annonçaient « 2 more accounts needed ». Triggers supprimés (sans objet en solo) ;
   `Init-Database.ps1` supprime désormais après import tout trigger/routine/vue/événement dont le définisseur n'existe
   pas, remplit les tables de noms playerbots vides et insère la saison d'arène si absente.

Windows a planté pendant ces essais (cause externe) ; au retour, MySQL a fait sa récupération InnoDB/XA sans perte
(331 tables, données intactes). Résultat final : **« World Initialized In 1 Minutes 44 Seconds »** sur la base du
royaume 1 (29 961 `creature_template`), 52 comptes / 520 personnages de bots créés, 20 bots connectés, compte
`homecraft` (GM 3). Le serveur reste lancé dans l'onglet de terminal « serveur HomeCraft ».

Reste à décider : nom affiché du royaume (`HomeCraft` par défaut dans `Run-World.ps1 -RealmName`), compte/joueur de
l'AHBot (`mod_ahbot.conf`, « Account id and player id missing »).

## 5. Ce qui reste bloqué par l'utilisateur

- ~~Dumps du royaume~~ importés (§4 ter). ~~DBC~~ rapatriés du VPS (§4 bis).
- Test en jeu avec le client 3.3.5a (en cours côté utilisateur : realmlist 127.0.0.1, compte `homecraft`).
- HermesProxy : version et `appsettings.json` utilisés sur l'autre serveur ; autorisation de télécharger le zip
  Windows (`Xian55/HermesProxy`, release v4.5.70 ou v4.6.0) pour le figer dans le manifeste.
