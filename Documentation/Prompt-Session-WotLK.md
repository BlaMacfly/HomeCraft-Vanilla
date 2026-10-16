# Prompt pour la session « HomeCraft WotLK Version » — construire le launcher Rust qui fait le pont serveur ↔ client

> À coller tel quel dans la session WotLK. Il décrit ce qui a été fait dans HomeCraft Edition Vanilla (dossier `C:\Users\Marty\Desktop\HomeCraft`, serveur VMaNGOS 1.12.1, launcher Rust validé par des tests automatisés et livré en package portable) et ce qu'il faut reproduire pour la version WotLK 3.3.5a. Tout ce qui est marqué « à vérifier » dépend du core WotLK choisi et doit être contrôlé dans son code ou sa configuration avant d'être codé. Tu peux lire le projet Vanilla sur ce poste pour t'en inspirer, en particulier `Launcher\src\*.rs`, `Tools\*.ps1`, `Build\*.ps1`, `Tests\*.ps1` et `Documentation\`.

---

## 0. Ce que tu dois produire, en une phrase

Un **launcher Windows écrit entièrement en Rust** (décision de l'utilisateur, non négociable), fenêtre unique avec un bouton **JOUER**, qui démarre dans l'ordre la base de données portable, le serveur d'authentification et le serveur monde, configure le client du joueur (realmlist, fenêtré, résolution), lance le jeu, surveille les services, et offre les onglets Comptes, PlayerBots, Monde, Patchs, Base de données, Sauvegardes, Paramètres, Journaux. Aucune compilation ni installation côté joueur, aucun port ouvert vers l'extérieur (tout sur 127.0.0.1), tout en français, distribué en dossier portable (pas d'installateur), le client du jeu n'étant **jamais** inclus.

Le launcher est **le pont** entre le serveur et le client : sans lui, le joueur devrait éditer des `.conf`, lancer MySQL, deux exécutables serveur, écrire son `realmlist.wtf`, créer un compte en console. Le launcher fait tout cela, dans le bon ordre, avec surveillance, sauvegarde et réparation.

## 1. Disposition des dossiers (identique en WotLK, noms des binaires à adapter)

```
<racine>\
├─ HomeCraft Edition WotLK.exe        launcher (copie signée de Launcher\target\release\<nom>.exe)
├─ Client\                            client du joueur (3.3.5a frFR, build 12340) – jamais redistribué
├─ Server\
│  ├─ <core>\                         binaires serveur (authserver.exe, worldserver.exe, extracteurs, DLL)
│  ├─ Config\                         *.conf.dist (modèles) → *.conf GÉNÉRÉS par le launcher ; my.ini généré
│  ├─ Database\mariadb\               MariaDB portable (zip officiel, allégé au packaging)
│  ├─ Database\sql\                   dumps SQL du core (auth, characters, world)
│  ├─ Database\custom\*.sql           compléments appliqués après l'import de la base monde (AHBot, traductions ; ordre alphabétique)
│  └─ Data\                           dbc/maps/vmaps/mmaps EXTRAITS du client au premier démarrage – jamais redistribués
├─ UserData\                          Database\ (datadir MariaDB), settings.json, db-credentials.json, Sauvegardes\, diagnostics
├─ Logs\                              launcher.log, server\*.log, mariadb-error.log
├─ Tools\                             HeidiSQL portable, scripts PowerShell (Common.ps1 = chemins, ports, identifiants, Invoke-Sql)
├─ Build\                             manifest.json (composants figés : URL, commit, SHA-256), Get-Components.ps1, Sign-Launcher.ps1,
│                                     New-FilesManifest.ps1, New-Package.ps1, New-Iso.ps1, files-manifest.json, package\
├─ Tests\                             scripts de test rejouables (voir §9)
├─ Documentation\                     01-Audit … 10-Validation, Manuel-Utilisateur.md
├─ Launcher\                          projet cargo (src\, assets\, build.rs, Cargo.toml, README.md)
├─ LICENCE.txt                        licence propriétaire HomeCraft (usage personnel, pas de reproduction)
└─ LICENCES\ (dans le package)        licences des composants tiers + SOURCES.md
```

Ports Vanilla : MariaDB 3315, auth 3724, monde 8085, tous sur 127.0.0.1. En WotLK, authserver 3724 et worldserver 8085 sont les mêmes par défaut. Si les deux éditions doivent coexister sur le même PC, décale les ports WotLK (ex. 3316 / 3725 / 8086) : le client 3.3.5a prend le port du monde dans la table `realmlist`, et le port d'authentification se met dans `realmlist.wtf` (`set realmlist 127.0.0.1:3725`).

## 2. Pile Rust figée (reprendre telle quelle)

- Rust stable MSVC (rustup, `~/.cargo/bin` à ajouter au PATH de la session) + Visual Studio Build Tools 2022 « Développement Desktop C++ » (link.exe, rc.exe). Installés sur ce poste dans `C:\BuildTools`.
- `eframe`/`egui` **0.32** et `egui_extras` 0.32 (feature `image`) : **ne pas monter en 0.36**, l'API a changé et toute l'interface serait à réécrire.
- `mysql` 28 (`default-features = false, features = ["minimal-rust"]`) : client MySQL pur Rust, aucune DLL.
- `rfd` 0.17 (sélecteurs natifs), `zip` 2 (deflate), `sha2` 0.11, `serde`/`serde_json`, `image` 0.25 (png, jpeg), `rand`, `windows-sys` 0.61 (features `Win32_Storage_FileSystem`, `Win32_Foundation`, `Win32_System_Time`, `Win32_UI_WindowsAndMessaging`), `winresource` 0.1 en build-dependency (icône + informations de version via `build.rs`).
- Vanilla seulement : `wow-mpq` 0.7 pour construire des archives MPQ V1 ; en WotLK les patchs sont des MPQ V2/V3 : vérifier que la crate gère ces versions, sinon ne proposer que activer/désactiver/ajouter.
- Un seul exécutable, `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`, aucune dépendance GPL/LGPL liée (vérifié avec `cargo metadata`), ce qui permet la licence propriétaire du launcher.

## 3. Architecture interne (à reproduire)

- `main.rs` : détecte la racine (`paths::Layout::detect()` remonte depuis l'exe jusqu'au dossier contenant `Server\` et `Tools\`), charge `settings.json`, lance l'orchestrateur, analyse la ligne de commande, démarre eframe. À la fermeture, envoie `Command::Shutdown` et attend 200 ms.
- `orchestrator.rs` : **thread dédié** avec une machine à états `State` (NotInstalled, Installed, Verifying, Initializing, StartingDb, StartingAuth, StartingWorld, Ready, Playing, Stopping, Error, Repairing), un `Snapshot` partagé `Arc<Mutex<…>>` (état, message, progression, services actifs, comptes, bases, tables, résultat de requête, sauvegardes, rapport, patchs, addons, état des mmaps, journaux ≤ 3000 lignes) et un canal `mpsc` de `Command` (StartAll, Play, StopAll, Verify, Repair, Backup, Restore, DeleteBackup, ExportDiagnostic, Console, CreateAccount, SetPassword, SetGm, DeleteAccount, ListTables, Query, Exec, LoadTable, OpenHeidi, SettingsChanged, InstallTo, KillGame, GenerateBots, DeleteBots, PatchSetActive, PatchAdd, PatchRemove, PatchBuild, AddonAdd, AddonSetActive, AddonRemove, GenerateMmaps, ReimportWorld, Shutdown). Boucle : `recv_timeout(1 s)` puis `tick()` (ping base, ports, processus, personnages en ligne, fin du jeu, fin de la génération des mmaps). L'interface ne touche jamais aux processus : elle lit le Snapshot et envoie des commandes.
- `db.rs` (`MariaDb`) : écrit `my.ini` (datadir `UserData\Database`, port, `bind-address=127.0.0.1`, `lower_case_table_names=2`, `innodb_buffer_pool_size` raisonnable), `initialize()` (`mariadb-install-db.exe` puis création de l'utilisateur), `start(timeout) -> Result<bool>` (spawn `mariadbd.exe`, attend port + `SELECT 1` ; **renvoie false si une base répondait déjà : elle n'est alors pas « possédée »**), `shutdown()` (`SHUTDOWN` SQL), `import_dumps(&[bases])` (DROP/CREATE/GRANT puis `mariadb.exe < dump`, puis `Server\Database\custom\*.sql` après la base monde ; **refuse** de recréer la base personnages si elle contient des personnages), `dump_database` (`mariadb-dump --lock-tables`), `query`/`exec`/`table_columns`/`table_rows`, suppression de compte hors ligne, comptage des bots.
- `credentials.rs` : `UserData\db-credentials.json` `{user, password, rootPassword}` générés aléatoirement au premier démarrage (retirer un éventuel BOM `\u{feff}` à la lecture).
- `server.rs` : génère les `.conf` depuis les `.conf.dist` (DataDir, LogsDir, chaînes de connexion `127.0.0.1;3315;user;mdp;base`, port, `BindIP`, Warden désactivé, anticheat désactivé, Motd, plus les surcharges du catalogue §5), `ensure_realm` (ligne `realmlist` id 1, nom « HomeCraft », adresse 127.0.0.1, port monde, build client autorisé), `Realmd`/`Mangosd` (spawn avec `CREATE_NO_WINDOW`, stdout lu dans un thread et renvoyé au journal, **console sur stdin** pour les commandes serveur ; `wait_ready` détecte la ligne « World initialized » ou l'ouverture du port ; arrêt propre par la commande console d'arrêt **puis fermeture de stdin**, kill après délai).
- `client.rs` : inspecte le dossier client (présence de `WoW.exe`, version lue dans `StringFileInfo` et non dans la version fixe, archives MPQ attendues), écrit `realmlist.wtf` (`set realmlist 127.0.0.1`) en conservant l'original en `.orig`, fusionne `WTF\Config.wtf` clé par clé (`gxWindow`, `gxResolution` bornée à l'écran, `locale`, `realmName`), lance `WoW.exe`, kill du jeu.
- `settings.rs` : `UserData\settings.json` (`client_dir`, `windowed=true`, `ahbot_enabled`, `ah_faction`, `partybots_enabled`, `conf: BTreeMap<clé, valeur>` = surcharges du catalogue, options client, `auto_mmaps=true`, `auto_backup=true`, `auto_backup_keep=10`) ; `#[serde(default)]` partout pour la compatibilité ascendante.
- `conf_catalog.rs` : **catalogue** des clés du `.conf` monde exposées (clé exacte, libellé FR, aide, type Bool/Int/Float/Choice/Text, valeur par défaut du `.conf.dist` du commit figé). Seules ces clés peuvent être surchargées ; ports, chemins et chaînes de connexion restent gérés. ~70 clés Monde (rates XP/butin/réputation, difficulté des créatures, AllowTwoSide.*, GM.*, quêtes, instances, visibilité, Motd…) et ~14 clés bots.
- `backup.rs` : dumps `characters` + `auth` (tables verrouillées), `settings.json`, manifeste `backup.json` (SHA-256 par fichier, date, libellé, nombre de personnages/comptes) dans un zip horodaté `UserData\Sauvegardes\sauvegarde-AAAAMMJJ-HHMMSS.zip` ; `restore` = vérification des empreintes, sauvegarde de sécurité automatique, import ; sauvegarde automatique à chaque fermeture du jeu avec rotation (N conservées, libellé dédié).
- `repair.rs` : vérification des fichiers gérés par `Build\files-manifest.json` (taille + SHA-256), `data_status` (dbc/maps/vmaps/mmaps présents, avec marqueur « génération en cours »), extraction des données du client avec les outils du core (`run_tool` avec sortie limitée à une ligne toutes les 5 s), génération des mmaps **en arrière-plan** (moitié des cœurs) après le démarrage du serveur, export de diagnostic (journaux + confs avec mots de passe masqués).
- `botgen.rs` (vanilla : création SQL directe de personnages bots de peuplement sur un compte dédié), `patches.rs` (liste, activer/désactiver par renommage `.off`, ajouter, supprimer), `addons.rs` (installation depuis zip avec détection du dossier contenant le `.toc`, activer/désactiver/supprimer), `mpqbuild.rs`, `i18n.rs` (**toutes** les chaînes visibles, en français, dans un seul module), `util.rs` (horodatage, SHA-256, copie d'arborescence, tailles lisibles), `ui.rs` (onglets, fenêtre de confirmation `Pending` pour toute action irréversible, fenêtre de rapport).

## 4. Ce que fait le bouton JOUER (séquence exacte `start_all`)

1. Vérifie composants et client (sinon ERREUR explicite).
2. Identifiants SQL (créés au besoin), `my.ini`, initialisation du datadir si absent.
3. Démarre MariaDB, ou s'attache à une base déjà en service **sans en prendre la propriété** (ne jamais l'arrêter ensuite ; incident réel corrigé).
4. Bases manquantes → import des dumps + compléments `custom\*.sql`, chaque étape journalisée (« import : complément x.sql »).
5. Génère les `.conf`, assure le royaume et le build client autorisé.
6. Données du client absentes → extraction dbc/maps/vmaps (première installation, quelques minutes) ; mmaps en arrière-plan ensuite.
7. Démarre l'authentification (attend le port), puis le monde (attend « World initialized », délai 10 min), rafraîchit les comptes.
8. Configure le client (realmlist, Config.wtf) et lance le jeu ; état JEU EN COURS ; `tick` détecte la fin du jeu → sauvegarde automatique.
9. Fermeture de la fenêtre : arrêt propre monde → authentification → base (seulement si possédée).

## 5. Onglets et fonctions livrées (toutes demandées par l'utilisateur, à reproduire)

- **Accueil** : état des composants, boutons JOUER / DÉMARRER LE SERVEUR / ARRÊTER LE SERVEUR / FERMER LE JEU (kill de WoW.exe), VÉRIFIER LES FICHIERS, RÉPARER, SAUVEGARDER, RESTAURER, diagnostic (dossier, version, client, mmaps, personnages en ligne).
- **Comptes** : création (identifiant, mot de passe, niveau GM), changement de mot de passe, niveau GM, **suppression définitive avec confirmation, personnages compris** (via console serveur si le monde tourne, sinon SQL hors ligne), liste avec dernière connexion et nombre de personnages. Commandes console vanilla : `account create`, `account set gmlevel`, `account delete` ; en WotLK, syntaxe et niveaux GM (0 à 3, table `account_access`) à vérifier ; le serveur gère SRP6.
- **PlayerBots** : réglages du module de bots exposés par catalogue, aide-mémoire des commandes en jeu, génération et suppression de bots de peuplement. En WotLK, le module de bots a généralement sa propre base, son propre `.conf` et ses propres commandes de chat : à vérifier dans le module choisi et à exposer de la même façon.
- **Monde** : catalogue des réglages du `.conf` monde par groupe, valeur par défaut rappelée, « modifié » signalé, « Tout remettre aux valeurs par défaut », appliqué au prochain démarrage.
- **Patchs** : archives `patch-X.MPQ` du client, activer/désactiver (renommage `.off`), ajouter, supprimer, créer depuis un dossier. WotLK : dossiers `Data\` et `Data\frFR\`, nommage `patch-X.MPQ` et `patch-frFR-X.MPQ`, priorité par lettre et chiffre, à vérifier.
- **Base de données** : explorateur natif façon HeidiSQL (arbre bases/tables, grille de 200 lignes éditable par double-clic, filtre WHERE, tri, insertion/suppression avec confirmation, onglet Structure, onglet SQL avec lecture libre et écriture confirmée, historique) + bouton « Ouvrir dans HeidiSQL » (portable dans `Tools\HeidiSQL`, arguments `-h=127.0.0.1 -P=3315 -u=… -p=…`, sans guillemets autour de l'hôte).
- **Sauvegardes** : manuelle avec libellé, automatique à la fermeture du jeu (rotation), de sécurité avant restauration ; restauration avec confirmation ; suppression ; ouverture du dossier ; export de diagnostic.
- **Paramètres** : dossier du client (sélecteur), fenêtré (recommandé ; le launcher borne la résolution), options client éventuelles, AHBot (activé, faction servie), mmaps automatiques, sauvegarde automatique + nombre conservé, « Réimporter la base monde… » (recrée la base monde depuis les dumps + compléments sans toucher aux personnages ni aux comptes), section **AddOns** (installation depuis zip, activer/désactiver/supprimer).
- **Journaux** : journal du launcher + sortie serveur, champ de commande console.
- Ligne de commande pour les tests : `--start`, `--quit-after N`, `--tab <onglet>`, `--do backup|verify|repair|diagnostic|restore <zip>|botgen N|botdel|patchon|patchoff|addonadd|addonon|addonoff|addondel|accdel|mpqbuild|reimportworld|mmaps`, `--install-to <dossier>`.

## 6. Identité, signature, icône

- Titre « HomeCraft Edition WotLK » (à adapter), logo de l'utilisateur (`assets\logo-512.png`, `logo-256.png`), fond d'accueil assombri, chaînes toutes en français.
- `build.rs` + `winresource` : icône multi-tailles `assets\homecraft.ico` (générée depuis le logo par `Build\New-InstallerImages.ps1`, 16 à 256 px, entrées PNG) + ProductName, CompanyName, LegalCopyright. Sans cela l'Explorateur n'affiche pas de logo (signalé par l'utilisateur).
- Signature Authenticode SHA-256 avec horodatage DigiCert, certificat **auto-signé** créé avec `New-SelfSignedCertificate -Type CodeSigningCert`, empreinte dans `UserData\code-signing-thumbprint.txt`, `.cer` public dans `Build\`. `Build\Sign-Launcher.ps1 -Thumbprint …` signe **et déploie** l'exe à la racine ; à relancer **après chaque build**, puis `Build\New-FilesManifest.ps1` (le manifeste contient l'empreinte de l'exe). Appeler ce script avec `&` dans une session PowerShell, pas `powershell -File` (sinon `$PSScriptRoot` est vide dans les valeurs par défaut des paramètres).

## 7. Composants et packaging

- Tous les composants externes passent par `Build\manifest.json` (id, source, licence, commit ou version, date, URL, fichier, taille, SHA-256, dossier d'installation) et `Build\Get-Components.ps1 -Pin` (télécharge dans `Build\downloads`, vérifie, fige l'empreinte). Rien d'autre n'est jamais téléchargé, surtout pas le client. Demander l'accord de l'utilisateur avant chaque téléchargement.
- MariaDB : zip officiel « winx64 » depuis `archive.mariadb.org` (le site principal était injoignable) ; extraire avec `tar.exe -xf … --strip-components=1` (chemins trop longs pour Expand-Archive). Les dumps MySQL 5.6/5.7/8 s'importent dans MariaDB 11 ; vérifier les éventuelles fonctions ou collations spécifiques à MySQL 8 dans les SQL du core WotLK.
- `Build\New-Package.ps1 -Zip` : copie launcher signé, serveur, modèles `.conf.dist`, dumps, compléments, MariaDB allégée (sans pdb, include, lib, mysql-test), HeidiSQL, scripts, documentation, `LISEZMOI.txt`, `LICENCE.txt`, `LICENCES\` (licences tiers + `SOURCES.md` généré avec versions, liens et liste des crates), `Sources\Launcher` (code), `Build\files-manifest.json`, `SHA256SUMS.txt`, `autorun.inf`. Jamais `Client\`, `Server\Data\`, `UserData\`, `Logs\`. Taille vanilla : 452 Mo, zip 135 Mo.
- Décisions de l'utilisateur : **portable uniquement** ; un installateur Inno Setup a été construit et testé puis **abandonné** parce que l'utilisateur voulait y inclure le client, ce qui serait une redistribution de contenu Blizzard. Ne pas inclure le client, jamais, même si demandé ; expliquer pourquoi.
- Licence : `LICENCE.txt` = contrat de licence propriétaire (usage personnel, lecture du code autorisée, reproduction/diffusion/modification interdites, exclusion de garantie, limitation de responsabilité, droit français) couvrant uniquement les créations de l'utilisateur ; composants tiers (core, MariaDB, HeidiSQL, crates) explicitement exclus et joints avec leurs sources. Si le core WotLK est sous AGPL-3.0, redistribuer ses binaires impose de fournir les sources exactes (commit) et la licence, comme pour VMaNGOS en GPL-2.

## 8. Pièges rencontrés (ne pas les redécouvrir)

- PowerShell 5.1 : scripts en **UTF-8 avec BOM** (sinon accents cassés), jamais `2>$null` sur un exécutable natif (passer par `cmd /c … >nul 2>&1`), `Start-Process -Wait` attend aussi les processus enfants graphiques (utiliser une boucle de sondage), `Set-Content` sans `-Encoding utf8` écrit en ANSI.
- Ne jamais lancer de test qui démarre ou arrête des services pendant que le launcher de l'utilisateur tourne ; une instance de test avait arrêté la base sous le launcher de l'utilisateur, d'où la distinction base « possédée » (`db_owned`) et base « annexe » (démarrée temporairement pour une sauvegarde, arrêtée ensuite seulement si possédée).
- `WoW.exe` : la version « fixe » de la ressource peut mentir (vanilla : 5875 affiché, build réel 5880) ; lire `StringFileInfo`. Le serveur doit accepter le build réel (`realmlist.gamebuild`, table des clients autorisés, liste interne du core).
- Console serveur : la première commande envoyée juste après le démarrage peut être ignorée ; après la commande d'arrêt, **fermer stdin**, sinon le processus ne se termine pas.
- Extraction : vérifier les chemins de sortie exacts attendus par le core (vanilla : `Data\5875\dbc`), créer les dossiers manquants, utiliser des chemins courts (les extracteurs tronquent les chemins longs) ; les extracteurs ne lisent que les archives officielles du client, un patch n'a jamais d'effet sur le serveur.
- Écran noir du client en fenêtré à la résolution exacte de l'écran : borner la résolution ; bouton FERMER LE JEU dans le launcher.
- Les tables du core peuvent avoir des **doublons d'entry** (contenu progressif) : `COUNT(DISTINCT …)`.
- egui 0.32 : `DragValue::new(&mut v).range(a..=b)`, `TextEdit::singleline(&mut s).password(true)`, `Grid`, `ScrollArea` ; toutes les chaînes dans `i18n.rs`.

## 9. Tests à écrire (tous rejouables, tous verts avant livraison)

`Test-ServerStartup` (scripts seuls), `Test-Launcher` (`--start --quit-after 90` : base, authentification, monde, arrêt propre, ports libérés, aucun orphelin, aucune ERREUR), `Test-Phase5` (sauvegarde, vérification 0 manquant / 0 altéré, réparation, réimport monde, restauration avec compte témoin, diagnostic ; 20 contrôles), `Test-Package` (copie vierge du package dans `C:\HomeCraftTest`, premier démarrage complet, arrêt brutal puis redémarrage, compléments SQL journalisés, aucun orphelin ; 10 contrôles), `Test-Localisation` (inventaire des tables de localisation frFR → `09-Localisation-Inventaire.md`), `Test-Offline` (services sur 127.0.0.1 seulement, aucune crate réseau). Les tests mettent `auto_mmaps:false` dans `settings.json` pour éviter la génération longue.

## 10. Adaptations WotLK à vérifier avant de coder

| Sujet | Vanilla (fait) | WotLK (à vérifier dans le core choisi) |
|---|---|---|
| Core | VMaNGOS, binaires Windows officiels, GPL-2 | Core 3.3.5a retenu par la session (binaires disponibles ? licence ? DLL à livrer ? modèles `.conf.dist` ?) |
| Bases | `realmd`, `characters`, `mangos`, `logs` | bases auth / characters / world du core (+ base des bots si module) ; mises à jour SQL appliquées par le serveur monde au démarrage ? |
| Client | 1.12.1 build 5875/5880, MPQ à plat dans `Data\` | 3.3.5a build 12340, `Data\` + `Data\frFR\` (`locale-frFR.MPQ`, `patch-frFR-*.MPQ`), `realmlist.wtf` dans `Data\frFR\` |
| Extracteurs | MapExtractor, VMapExtractor, VMapAssembler, MoveMapGenerator | outils map / vmap / mmap du core, dossiers de sortie `dbc`, `maps`, `vmaps`, `mmaps` (+ `Cameras` ?) |
| Bots | PartyBots intégrés au core (`.partybot add tank`) | module de bots du core : configuration, base, commandes, comptes de bots aléatoires |
| AHBot | table `auctionhousebot` remplie par script | AHBot intégré ou module ; tables et réglages à identifier |
| GM | `account set gmlevel 6` | niveaux GM du core, table `account_access` |
| Localisation | `locales_*` (loc2 = frFR) | tables `*_locale` avec colonne `locale='frFR'` ; mesurer la couverture et compléter via `custom\*.sql` |

Procède par phases comme pour la version Vanilla : audit (composants, licences, binaires disponibles) → construction (base, serveur, extraction, test de connexion manuel) → launcher (démarrage/arrêt, puis onglets un par un) → fiabilité (sauvegarde, restauration, réparation) → packaging portable → validation finale, chaque phase documentée dans `Documentation\` et couverte par un test. Réponds en français, note chaque décision de l'utilisateur et chaque piège dans la documentation, et demande son accord avant tout téléchargement.
