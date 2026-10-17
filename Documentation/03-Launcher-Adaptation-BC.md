# HomeCraft Edition BC — Phase 3 : adaptation du launcher Rust à CMaNGOS-tbc

Date : 2026-10-17
Statut : **code adapté, compilé (release), 3 tests unitaires verts, signé et déployé (`HomeCraft Edition BC.exe`),
captures des onglets faites sans démarrer de service. Aucun test avec services : votre HomeCraft Edition Vanilla
tournait (décision de l'utilisateur : « Attends : je ferme le Vanilla et je te préviens »).**

Point de départ : copie du launcher Vanilla (VMaNGOS 1.12.1). Pile inchangée : eframe/egui 0.32, `mysql` 28 pur Rust,
un seul exécutable, aucune dépendance GPL liée.

## 1. Modifications par module

| Module | Changement |
|---|---|
| `paths.rs` | ports 3317 / 3726 / 8087, build client 2.4.3.8606, royaume « HomeCraft BC », `HOMECRAFT_BC_ROOT` ; constantes `DB_REALMD/DB_CHARACTERS/DB_WORLD/DB_LOGS` = `tbcrealmd/tbccharacters/tbcmangos/tbclogs` ; tables de version `*_db_version` (remplacent `migrations`) ; `Server/mangos-tbc`, `extractors()`, `extract_view()` ; composants attendus : exe, 4 `.conf.dist`, `ad.exe`, `tbcmangos.sql` |
| `db.rs` | toutes les requêtes sur les bases `tbc*` ; dumps TBC-DB (`tbcrealmd.sql`…) ; comptes d'exemple du dump supprimés à l'import ; suppression de compte hors ligne selon le schéma CMaNGOS (`account_banned.account_id`, `account_logons.accountId`, `realmcharacters.acctid`, plus de `account_access` ni de `playerbot`) ; comptage des bots = personnages des comptes `RNDBOT…` ; plus de table `auctionhousebot` |
| `server.rs` | modèles `*.conf.dist` du build recopiés dans `Server/Config` s'ils changent ; génération de `realmd.conf`, `mangosd.conf` (clés sans point : `LoginDatabaseInfo`…), `aiplayerbot.conf`, `ahbot.conf` ; `Expansion = 1`, `DBC.Locale = 255`, RA/SOAP coupés ; royaume `realmbuilds = '8606'` (pas de table `allowed_clients`) ; mangosd lancé avec `-c -a -p`, **dossier courant `Server/mangos-tbc`** (legacy.dll d'OpenSSL) et variables `Anticheat_Enable=0`, `Anticheat_Warden_Enable=0`, `Anticheat_Antispam_Enable=0` ; arrêt `server shutdown 0` puis fermeture de stdin |
| `client.rs` | archives officielles 2.4.3 (`Data\` + `Data\frFR\`, `is_official_mpq`) ; version exigée 2.4.3.8606 ; **`realmlist.wtf` à la racine du client** avec le port (`set realmlist 127.0.0.1:3726`), original en `.orig` ; `Config.wtf` : `realmList 127.0.0.1:3726` ; VanillaFixes et DXVK retirés |
| `repair.rs` | seuils de données relevés sur le client réel (185 DBC, 3 586 cartes, 8 607 vmaps) ; extraction `ad` / `vmap_extractor` / `vmap_assembler` / `MoveMapGen` (arguments de `ExtractResources.sh` et `MoveMapGen.sh`) sur une **vue des seules archives officielles** (liens physiques dans `UserData\extract-view`, repli `<lecteur>:\HCBC-extract`, chemin ≤ 100 caractères) ; contrôle explicite du nombre de DBC après `ad.exe` (il sort en 0 même sans rien extraire) ; diagnostic : `aiplayerbot.conf` et `ahbot.conf` ajoutés |
| `conf_catalog.rs` | réécrit : ~70 clés `mangosd.conf` TBC (progression, butin, difficulté, instances et héroïques, arènes, factions, MJ, technique) et 19 clés `aiplayerbot.conf` (bots aléatoires, champs de bataille, comportement) ; défauts relevés dans les `.conf.dist` du build ; défauts HomeCraft : 50 bots (au lieu de 1000), royaume Normal, Motd français ; test unitaire : chaque clé existe dans son modèle |
| `botgen.rs` | réécrit : le module PlayerBots crée lui-même ses bots ; le launcher règle leur nombre (`MinRandomBots`/`MaxRandomBots`, nombre de comptes = ⌈n/9⌉) et sait les supprimer serveur arrêté (comptes RNDBOT, personnages, `ai_playerbot_random_bots`, guildes orphelines) |
| `patches.rs` | réécrit : `Data\patch-X.MPQ` **et** `Data\frFR\patch-frFR-X.MPQ` ; officielles = liste de `client.rs` ; `patch-frFR-3.MPQ` (serveur tiers) apparaît comme patch désactivable |
| `orchestrator.rs` | bases `tbc*` ; création de compte : `account create`, `account set gmlevel`, **`account set addon <compte> 1`** (extension TBC) ; liste des comptes : `account.gmlevel`, dernière connexion = `MAX(account_logons.loginTime)`, comptes RNDBOT masqués ; réimport du monde = `tbcmangos` ; HeidiSQL `-db=tbccharacters;tbcrealmd;tbcmangos;tbclogs` ; plus de VanillaFixes |
| `backup.rs` | sauvegarde de `tbccharacters` et `tbcrealmd` (`tbccharacters.sql`, `tbcrealmd.sql` dans l'archive) |
| `ui.rs` | onglet PlayerBots : nombre de bots aléatoires, suppression (désactivée serveur démarré), aide-mémoire des commandes **`.bot`** du module (`.bot group`, `.bot create`, `.bot add/remove`, `.bot prepare`, `/w <bot> help`…) ; niveaux GM 0 à 3 ; AHBot sans choix de faction ; AddOns : interface attendue 20400 ; base par défaut de l'explorateur : `tbccharacters` |
| `i18n.rs` | tous les textes 1.12.1 / VMaNGOS / VanillaFixes / `.partybot` remplacés ou retirés ; nouveaux textes BC |
| `mpqbuild.rs` | inchangé (archives MPQ V1, lues aussi par le client 2.4.3 : **à vérifier en jeu**) |

## 2. Vérifications faites (sans services)

- `cargo build --release` : 0 erreur ; `cargo test` : 3 tests verts (`keys_exist_in_templates`, `official_archives`, `noms_de_patchs`).
- Exécutable signé (certificat auto-signé du Vanilla) et déployé ; `Build/New-FilesManifest.ps1` corrigé
  (`Server\mangos-tbc`, plus de `.conf.dist.in`) : 916 fichiers gérés.
- Captures `Tools/Capture-Launcher.ps1` (sans `--start`) : accueil (client 2.4.3.8606 « vérifié », tous les
  composants « présents »), PlayerBots, Monde, Patchs, Paramètres, Comptes → `Documentation/images/`.
- Ouvrir le launcher ne démarre rien : la base n'est démarrée que par JOUER / DÉMARRER, ou par une action explicite
  (sauvegarde, réparation, suppression hors ligne).

## 3. Premier démarrage complet (2026-10-17, Vanilla arrêté par l'utilisateur)

Complément français appliqué à la base installée sans erreur (1 s ; `mangos_string` 0 → 618 textes frFR, remises de
quêtes 2 081 → 4 193, phrases des bots 500 → 771, noms d'objets 23 870 → 24 590). Puis
`HomeCraft Edition BC.exe --start` : MariaDB, realmd, puis mangosd jusqu'au « Starting Map System », où il s'arrête.
Le launcher a tout arrêté proprement (realmd puis MariaDB) et affiché ERREUR. Deux causes :

1. **Schéma TBC-DB en retard sur le core** : `locales_gameobject` a encore `castbarcaption_loc1..8` alors que le core
   15b6ddb4 lit `opening_text_loc1..8` et `closing_text_loc1..8` (partie « locales » de la mise à jour core
   `s2485_01_mangos_closing_text`, non appliquée par TBC-DB bien que `db_version` annonce `required_s2492`).
   Effet : « query ERROR: Unknown column 'opening_text_loc1' », noms français des objets du monde non chargés.
   Correctif : `Server/Database/custom/00_homecraft_schema_locales_gameobject.sql` (idempotent, `CHANGE COLUMN IF
   EXISTS` / `ADD COLUMN IF NOT EXISTS`, appliqué deux fois sans erreur), passé avant le complément français.
   Seule table `locales_*` concernée (contrôle de toutes les colonnes lues par `ObjectMgr`).
2. **mmaps obligatoires** : « Critical Error: A condition which must never be false » juste après
   « MMAP:loadMapData: Error: Could not open mmap file …/000.mmap ». `MMapManager::loadMap` contient
   `MANGOS_ASSERT(itr != loadedMMaps.end())` et est appelé par `GridMap` dès qu'une grille est chargée, **sans tenir
   compte de `mmap.enabled`** (vérifié dans les sources 15b6ddb4 et sur `master` le 2026-10-17). Sans mmaps,
   mangosd s'arrête donc toujours. Conséquences :
   - le launcher génère les mmaps **avant** le démarrage du monde si elles manquent (bloquant, tous les cœurs
     sauf deux, marqueur « génération en cours » retiré seulement en cas de succès) ; la génération automatique en
     arrière-plan après démarrage (Vanilla) est supprimée, la case correspondante remplacée par une explication ;
   - `MoveMapGen --workdir` reçoit un chemin à barres obliques terminé par « / » : avec un antislash final, le
     guillemet fermant était échappé et MoveMapGen recevait « …\Data" --threads 12 --buildGameObjects /maps »
     (code -3). Corrigé dans `repair.rs` et `Tools/Extract-ClientData.ps1`.

### Deuxième démarrage (mmaps générées : 14 min sur 12 threads, 2 838 fichiers, 72 cartes, 2,4 Go)

- Monde opérationnel en 5 s après realmd (« CMANGOS: World initialized »), ports 3317 / 3726 / 8087 sur 127.0.0.1
  uniquement, royaume « HomeCraft BC » build 8606, AHBot actif (vente et achat), plus aucune erreur de colonne.
- Arrêt automatique (`--quit-after`) **propre** : `server shutdown 0` (« Halting process... »), realmd, MariaDB,
  aucun processus restant.
- **PlayerBots désactivé** : « AI Playerbot is Disabled. Unable to open configuration file aiplayerbot.conf ». Bug du
  core : l'option `-p` de mangosd est affectée à `_PLAYERBOT_CONFIG`, alors que `PlayerbotAIConfig::Initialize` lit la
  constante `_D_AIPLAYERBOT_CONFIG = "aiplayerbot.conf"`, relative au dossier courant. Correctif : le launcher (et
  `Tools/New-ServerConfig.ps1`) écrit aussi `Server/mangos-tbc/aiplayerbot.conf` ; `New-FilesManifest.ps1` ignore les
  `.conf` du dossier du programme.

### Troisième démarrage : première initialisation des bots

- Module actif : « 6 random bot accounts with 54 characters available », puis « Building equipment cache for 12
  classes, 32 specs, 70 levels, 19 slots, 6 quality from 39657 items » : **construction unique** de la table
  `ai_playerbot_equip_cache` (plusieurs millions de lignes, ~3 000 lignes/s, un seul cœur, mangosd jusqu'à ~6 Go de
  mémoire). Le module recharge le cache tel quel s'il contient des lignes : une construction interrompue laisserait des
  bots mal équipés pour toujours.
- Le délai d'attente du monde (600 s) a expiré pendant cette construction. Deux défauts du launcher en sont sortis,
  corrigés :
  1. délai porté à 3 h tant que mangosd tourne, avec le message « Première initialisation des PlayerBots… » dès la
     ligne « Building equipment cache » ;
  2. sur échec du chargement, mangosd n'était pas arrêté (processus local pas encore confié à l'orchestrateur ;
     abandonner un `Child` Rust ne tue pas le processus) : il restait orphelin pendant que la base était arrêtée. Il
     est désormais arrêté avant l'abandon.
- Caches `ai_playerbot_*_cache` vidés (1 454 350 lignes d'équipement partielles, 15 878 d'objets) pour reconstruction
  complète.
- mangosd écrit ses rapports de plantage dans `Server/mangos-tbc/Crashes/` (noms avec crochets : `-LiteralPath`
  obligatoire en PowerShell, sinon `New-FilesManifest.ps1` échouait). Dossier exclu du manifeste ; à ajouter à
  l'export de diagnostic.

### Quatrième démarrage : réussite complète (2026-10-17)

| Étape | Résultat |
|---|---|
| Démarrage `--start` | MariaDB 11:17:02, realmd 11:17:04, cache d'équipement des bots construit en 10 min 30 (1 560 522 lignes), « CMANGOS: World initialized » 11:27:44 |
| Bots | 54 personnages sur 6 comptes RNDBOT, **50 en ligne** une minute après, niveaux 1 à 70, 12 elfes de sang / draeneï, 20 guildes de bots |
| Fermeture de la fenêtre | arrêt propre en 1 s : mangosd (console), realmd, MariaDB ; aucun processus restant |
| Démarrages suivants | 45 s jusqu'au monde prêt (harnais `Run-World.ps1`), le cache étant en base |
| Console (comptes) | `account create`, `account set gmlevel 3`, `account set addon 1`, `account set password`, `account delete` : effectifs en base (`gmlevel = 3`, `expansion = 1`, compte supprimé) ; **réponses de la console en français** (« Compte créé: HCTEST », « Vous modifiez le niveau de sécurité du compte… ») grâce au complément `mangos_string` |

## 4. Reste à valider avec les services

1. `--start --quit-after 600` : démarrage complet, « CMANGOS: World initialized », arrêt propre (`server shutdown 0`).
2. Création de compte depuis l'onglet Comptes (addon 1), connexion en jeu, création de personnage BC (draeneï, elfe de sang).
3. `.bot group` en jeu, bots aléatoires visibles, AHBot.
4. Sauvegarde / restauration / réparation / réimport du monde (avec le complément français).
5. Génération des mmaps (`MoveMapGen`) en arrière-plan : durée.
6. Patchs en lettres (`patch-A.MPQ`) et archives MPQ V1 construites : chargées par le client 2.4.3 ?
