# Phase 3 — Launcher Rust (fork de `../HomeCraft/Launcher`), état au 2026-10-16

Le launcher WoTLK est un fork du launcher de l'édition Vanilla (egui 0.32, 100 % Rust), réécrit module par module
pour AzerothCore (fork Sylvania + mod-playerbots), MySQL 8.4.9 et le double client 3.3.5a / WotLK Classic 3.4.3
(HermesProxy + Arctium). Cahier des charges : `04-Launcher-Exigences.md` (§A tranché, §B brief utilisateur).

## 1. Résultat

- `cargo build --release` passe du premier coup après la réécriture (13 avertissements de code mort, aucun
  avertissement de logique). Exécutable : `Launcher/target/release/homecraft-wotlk-launcher.exe` (10,7 Mo), copié à
  la racine sous le nom **`HomeCraft Edition WoTLK.exe`** (nom aussi utilisé par `Layout::detect` et par
  l'installation depuis un support en lecture seule).
- **Non testé en exécution** : le serveur de l'utilisateur tournait dans un terminal pendant toute la phase ; aucun
  test qui démarre ou arrête MySQL, authserver ou worldserver n'a été lancé (règle du projet). Premier lancement réel
  à faire serveur arrêté : voir §5.
- Visuels : `Build/New-LauncherImages.ps1` a produit `Launcher/assets/logo-512.png` / `logo-256.png` (logo détouré par
  diffusion depuis les coins), `homecraft.ico` (7 tailles) et `fond.jpg` (1659 × 948, taverne). Le fond est peint sur
  le calque d'arrière-plan avec son rapport d'aspect ; le logo est dans la barre de titre et l'icône de fenêtre.
- `LICENCE.txt` à la racine : licence propriétaire HomeCraft de l'édition Vanilla, adaptée (AzerothCore AGPL v3,
  mod-playerbots GPL v2, MySQL Community GPL v2, HermesProxy GPL v3, client 3.3.5a ou 3.4.3).

## 2. Ce qui a changé par rapport à l'édition Vanilla (par module)

| Module | Vanilla (VMaNGOS) | WoTLK (AzerothCore) |
|---|---|---|
| `paths.rs` | realmd/mangosd, MariaDB, ports 3306/3724/8085 | `Server/acore` (binaires + `configs/`), MySQL `Server/Database/mysql` + `Server/Config/my.ini`, ports **3315**/3724/8085, HermesProxy 1119/8081/8084/8086, bases `acore_auth/characters/world/playerbots`, `Tools/HermesProxy`, `short_root()` (racine sans espace : nom court 8.3 ou jonction `<lecteur>:\HomeCraftW`), `core_source_dir()` (`Build/src/sylvaniacore-wotlk` ou `C:\hc`) |
| `db.rs` | `MariaDb` | `MySql` : `--initialize-insecure` + `--init-file` (root@127.0.0.1, `skip-name-resolve`), import `.sql`/`.sql.gz` (flate2) avec `--force`, `import_dumps` par mot-clé + `Server/Database/custom/<base>/*.sql`, `post_import_cleanup` (objets à DEFINER inconnu, `active_arena_season`), `bot_pool()`/`delete_bot_pool()` (comptes `RNDBOT%`), compteurs hors bots |
| `server.rs` | realmd/mangosd, conf dans `Server/Config` | `generate_configs` écrit `authserver.conf`, `worldserver.conf` (+ clés hors modèle `PlayerbotsDatabaseInfo`, `Playerbots.Updates.EnableDatabases`, `Warden.Enabled=0`, `Updates.EnableDatabases=15` si sources présentes sinon 0) et `configs/modules/*.conf` (surcharges `playerbots.conf` : `MinRandomBots/MaxRandomBots/RandomBotAutologin` + catalogue ; `mod_ahbot.conf` : vendeur) ; `ensure_realm` (`INSERT … ON DUPLICATE KEY`, **avant** authserver) ; `AuthServer`, `WorldServer` (console stdin, prêt sur « World Initialized », arrêt `server shutdown 1`), `HermesProxy` (`--set ServerAddress/ServerPort/ServerBuild` + `Settings.hermes_args`) |
| `client.rs` | 1.12.1 | `Detected {None, Wotlk335, Classic343}`, `ClientKind {Auto, Wotlk335, Classic343}` ; 3.3.5a : `Wow.exe` 3.3.5.12340, locale `Data/xxXX`, MPQ requis, `realmlist.wtf` + `WTF/Config.wtf` ; 3.4.3 : `_classic_/WowClassic.exe`, `.build.info` contient 54261, `Arctium WoW Launcher.exe --version=Classic --staticseed`, `_classic_/WTF/Config.wtf` `portal 127.0.0.1` ; `interface_root()` pour AddOns/WTF |
| `conf_catalog.rs` | `mangosd.conf` | `ConfFile {World, Playerbots}` ; `BOT_OPTIONS` (clés `AiPlayerbot.*` vérifiées dans `playerbots.conf.dist`) et `WORLD_OPTIONS` (vérifiées dans `worldserver.conf.dist` du fork) |
| `settings.rs` | partybots, VanillaFixes, DXVK, faction AH | `client_kind`, `random_bots` (40), `ahbot_enabled/account/guid`, `realm_name`, `hermes_args`, `auto_mmaps`, `auto_backup(_keep)` |
| `repair.rs` | extracteurs VMaNGOS | `map_extractor -e 7`, `vmap4_extractor`/`vmap4_assembler`, `mmaps_generator` ; contrôle **DBC 3.3.5a** (`Map.dbc` max ID ≥ 724, `AreaTable` ≥ 4987 : les DBC d'un client frFR recomposé sont 3.3.3a) ; seuils dbc 240 / maps 5000 / vmaps 10000 / mmaps 3000 |
| `patches.rs` | `Data/` seul | `Data/` + `Data/<locale>/` (`patch-<locale>-X.MPQ`) ; sans objet pour le 3.4.3 (CASC) |
| `backup.rs` | characters/realmd | `acore_characters.sql` + `acore_auth.sql` |
| `orchestrator.rs` | GenerateBots, PatchBuild | `Snapshot.auth_up/hermes_up/hermes_installed/bot_pool`, `Command::DeleteBots` (pool rndbot, serveur arrêté), `play()` démarre HermesProxy si client 3.4.3, surveillance de `WowClassic.exe` (Arctium se termine dès le jeu ouvert), GM `account set gmlevel <u> <0-3> -1` |
| `ui.rs` | onglet Bots = génération de bots | onglet **PlayerBots** : curseur « bots aléatoires » (0-500), pool existant (comptes/personnages) + suppression, catalogue `AiPlayerbot.*`, aide-mémoire `.playerbots` / murmures ; **Paramètres** : dossier client, type de client, fenêtré, nom du royaume, AHBot (compte/guid), arguments HermesProxy, mmaps, sauvegarde auto ; **Accueil** : ligne HermesProxy si client 3.4.3, alerte Arctium absent ; **Patchs** : note CASC + AddOns seuls pour le 3.4.3 ; AddOns : interface attendue 30300 / 30403 ; GM 0-3 |
| supprimés | `botgen.rs`, `mpqbuild.rs` | sans objet (bots créés par le serveur ; pas de construction de MPQ) |

Dépendances (`Cargo.toml`) : eframe/egui_extras **0.32** (ne pas monter), `mysql` 28 `minimal-rust`, rfd, zip, sha2,
serde, image, windows-sys 0.61, **flate2** (`rust_backend`, ajouté pour les dumps `.sql.gz`), winresource. `wow-mpq`
retiré.

## 3. Avertissements restants (code mort, à trancher au packaging)

`fr::CLOSE`, `fr::DB_BASE/DB_TABLE/DB_LOAD/DB_SQL/DB_RUN` (anciens textes de l'explorateur), `HERMES_REST_PORT`,
`HERMES_REALM_PORT`, `HERMES_INSTANCE_PORT` (documentaires), `conf_catalog::is_known`, `backup::verify`,
`VerifyReport::data_ok`, `MySql::conn`/`characters_count`. Aucun n'affecte le fonctionnement.

## 4. Points à vérifier au premier lancement réel (serveur arrêté)

1. `Layout::detect` trouve la racine (dossier contenant `Server/` et `Tools/`) depuis l'exe à la racine ; la variable
   `HOMECRAFT_WOTLK_ROOT` force une racine (tests).
2. `short_root()` : la racine contient un espace (`HomeCraft WoTLK`) → nom court 8.3 si disponible, sinon jonction
   `C:\HomeCraftW` créée par `mklink /J` (droits : pas besoin d'administrateur pour une jonction). Le lancement par
   `Tools/Run-World.ps1` utilise lui `C:\hcw` : les deux peuvent coexister.
3. Démarrage MySQL par le launcher alors que la base a été initialisée par `Tools/Init-Database.ps1` : même datadir
   `UserData/Database`, mêmes identifiants (`UserData/db-credentials.json`, utilisateur `acore`). Si MySQL tourne
   déjà, le launcher s'y attache sans la posséder (« non gérée par ce launcher »).
4. `generate_configs` : écrase `Server/acore/configs/*.conf` produits par `Tools/New-ServerConfig.ps1` — vérifier que
   les clés spécifiques (`PlayerbotsDatabaseInfo`, `Warden.Enabled=0`, `Updates.*`, `SourceDirectory`) sont bien
   identiques à celles qui ont permis le premier démarrage (`03-Phase2-Serveur-local.md`).
5. Chargement du monde : délai de 20 minutes maximum avant échec ; 1 min 44 constaté en phase 2.
6. Bouton JOUER avec le client 3.3.5a (`Settings.client_kind = Wotlk335` ou Auto sans `_classic_`), puis avec le
   3.4.3 une fois HermesProxy installé dans `Tools/HermesProxy` (téléchargement à autoriser : `Xian55/HermesProxy`,
   zip Windows, à figer dans `Build/manifest.json`).
7. Onglet PlayerBots : le curseur écrit `AiPlayerbot.MinRandomBots/MaxRandomBots` à la génération des conf (valeur
   0 = `RandomBotAutologin 0`).

## 5. Reste à faire (phases suivantes, modèle Vanilla)

- ~~Tests PowerShell (`Tests/`)~~ : faits le 2026-10-16 (`06-Tests-Launcher.md`) : premier lancement réel réussi,
  `--start`, `--do backup|verify|repair|restore|diagnostic`, `--tab` verts ; correctif `AiPlayerbot.CommandServerPort = 0`
  (port 8888 sur toutes les interfaces). Restent `--do reimportworld|botdel`, `--install-to`.
- Packaging portable (`Build/New-Package.ps1`, `New-FilesManifest.ps1`, dossier `LICENCES/` + `SOURCES.md`, dump
  `acore_playerbots` pour le package), signature (`Sign-Launcher.ps1`), manuel utilisateur.
- HermesProxy : téléchargement, `appsettings.json` de référence, validation du bouton JOUER en 3.4.3.
- Compte/personnage vendeur de l'AHBot (onglet Paramètres) une fois créés en jeu.

## 6. Onglet Modules (ajouté le 2026-10-16, après les tests)

Demande de l'utilisateur : un onglet « Modules » à côté de PlayerBots pour activer ou désactiver chaque module
indépendamment, avec un raccourci vers le fichier de configuration de chacun.

- `src/modules_catalog.rs` : les 24 modules du cœur Sylvania (`Build/src/sylvaniacore-wotlk/modules/`), avec pour
  chacun son fichier dans `Server/acore/configs/modules/`, sa clé d'activation et ses valeurs (`1/0`, ou
  `true/false` pour `ALE.Enabled`), relevées dans les `.conf.dist` du cœur compilé. `mod-arac` et `mod-npc-morph`
  n'ont ni fichier ni clé : affichés « toujours actif ». `mod-ah-bot` reste piloté par l'onglet Paramètres (compte et
  personnage vendeur obligatoires) : l'onglet n'en montre que l'état.
- **État lu dans le `.conf`**, pas dans `settings.json` : la case écrit la clé dans le fichier
  (`server::set_module_enabled`), « Ouvrir » lance l'application associée (Bloc-notes en repli), « Modèle » recopie le
  `.conf.dist` (`server::reset_module_conf`, après confirmation). Le changement prend effet au prochain démarrage du
  serveur (avertissement affiché si les services tournent).
- **Changement du générateur** (`server::generate_configs`) : les `.conf` de modules repartent désormais du `.conf`
  existant et non plus du modèle, pour conserver l'activation et les modifications faites à la main ; le launcher n'y
  réapplique que ses propres clés (`apply_module_overrides` : base playerbots, nombre de bots, port de commandes 0,
  catalogue PlayerBots, AHBot). Si une clé gérée manque dans le `.conf` (fichier abîmé), il repart du modèle.
  `worldserver.conf` et `authserver.conf` restent régénérés intégralement à chaque démarrage.
- Accueil : « PlayerBots : désactivés » aussi quand `AiPlayerbot.Enabled = 0`, et l'onglet Modules prévient que le
  nombre de bots réglé n'aura alors aucun effet.
- Ligne de commande (tests) : `--do modon <id>`, `--do modoff <id>`, `--do modreset <id>` (identifiant = nom du
  dossier du module, ex. `mod-junk-to-gold`) ; `--tab modules`. Vérifié le 2026-10-16 sur `mod-junk-to-gold` (1 → 0
  → 1) et `mod-ale` (true → false → modèle), launcher relancé 8 s sur l'onglet sans erreur. La bascule depuis la case
  de l'interface n'a pas été cliquée à la souris (même code que `--do`).
