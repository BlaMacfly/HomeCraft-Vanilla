# Prompt pour la session « HomeCraft BC » — construire HomeCraft Edition BC (The Burning Crusade 2.4.3) avec son launcher Rust

> À coller tel quel dans la session qui travaille dans `C:\Users\Marty\Desktop\HomeCraft BC`. Il décrit ce qui a été fait dans HomeCraft Edition Vanilla (dossier `C:\Users\Marty\Desktop\HomeCraft`, serveur VMaNGOS 1.12.1, launcher Rust validé par des tests automatisés, publié sur `https://github.com/BlaMacfly/HomeCraft-Vanilla`) et ce qu'il faut reproduire pour la version BC 2.4.3. Tout ce qui est marqué « à vérifier » dépend du core BC choisi et doit être contrôlé dans son code ou sa configuration avant d'être codé. Tu peux lire le projet Vanilla sur ce poste pour t'en inspirer, en particulier `Launcher\src\*.rs`, `Launcher\build.rs`, `Tools\*.ps1`, `Build\*.ps1`, `Tests\*.ps1` et `Documentation\` (le manuel `Manuel-Utilisateur.md` et `04-Launcher-Exigences.md` décrivent chaque fonction).

---

## 0. Ce que tu dois produire, en une phrase

Un **launcher Windows écrit entièrement en Rust** (décision de l'utilisateur, non négociable), fenêtre unique avec un bouton **JOUER**, qui démarre dans l'ordre la base de données portable, le serveur d'authentification et le serveur monde, configure le client 2.4.3 du joueur (realmlist, fenêtré, résolution), lance le jeu, surveille les services, et offre les onglets Comptes, PlayerBots, Monde, Patchs, Base de données, Sauvegardes, Paramètres, Journaux. Aucune compilation ni installation côté joueur, aucun port ouvert vers l'extérieur (tout sur 127.0.0.1), tout en français, distribué en dossier portable (pas d'installateur), le client du jeu n'étant **jamais** inclus.

Le launcher est **le pont** entre le serveur et le client : sans lui, le joueur devrait éditer des `.conf`, lancer MySQL, deux exécutables serveur, écrire son `realmlist.wtf`, créer un compte en console. Le launcher fait tout cela, dans le bon ordre, avec surveillance, sauvegarde et réparation.

## 1. Identité visuelle BC (fournie par l'utilisateur, à installer en premier)

- **Logo** : `C:\Users\Marty\Desktop\HomeCraft BC\HomeCraft Logo BC.png` (1254×1254, PNG 32 bits). Vérifie d'abord si le fond est réellement transparent ou noir opaque : si le canal alpha est plein (fond noir), rends le fond transparent (les pixels de fond sont noirs purs autour de l'anneau : détourage par couleur avec tolérance et lissage du bord, comme cela avait été fait pour le logo Vanilla). Produis ensuite, dans `Launcher\assets\` : `HomeCraft-original.png` (copie de l'original), `logo-512.png` et `logo-256.png` (redimensionnés, fond transparent), et l'icône multi-tailles `homecraft.ico` (16, 24, 32, 48, 64, 128, 256 px, entrées PNG) générée par un script `Build\New-InstallerImages.ps1` repris du projet Vanilla. L'icône est intégrée à l'exécutable par `Launcher\build.rs` (crate `winresource`) avec les informations de version (ProductName « HomeCraft Edition BC », CompanyName « BlaMacfly », LegalCopyright). Sans cette étape, l'Explorateur Windows n'affiche pas de logo sur l'exe.
- **Fond** : `C:\Users\Marty\Desktop\HomeCraft BC\HomeCraft Tavern BC.png` (1661×947). Garde sa taille native (1661×947 : ne pas l'agrandir, l'interface l'étire à la fenêtre en mode couverture), enregistre-le en JPEG qualité 85 sous `Launcher\assets\fond.jpg`, et affiche-le en couverture de la fenêtre avec un assombrissement (voile noir à 55-65 %) pour que les textes restent lisibles, exactement comme le launcher Vanilla (chargé par `egui_extras` avec `include_bytes!`).
- **Titre** : « HOMECRAFT EDITION BC » dans l'en-tête, titre de fenêtre « HomeCraft Edition BC », exécutable déployé `HomeCraft Edition BC.exe`, royaume nommé « HomeCraft BC », message du jour « Bienvenue dans HomeCraft Edition BC ! ». Palette d'accent : reprends les teintes du logo (or et vert fel) pour le titre et les états, au lieu de l'or seul du Vanilla.
- Documente la provenance des deux images dans `Launcher\assets\SOURCES.md` (fournies par l'utilisateur, qui en détient les droits ; identité visuelle couverte par la licence HomeCraft, article 2.2).

## 2. Disposition des dossiers (identique au Vanilla, noms des binaires à adapter)

```
<racine>\
├─ HomeCraft Edition BC.exe           launcher (copie signée de Launcher\target\release\<nom>.exe)
├─ Client\                            client du joueur (2.4.3 frFR, build 8606) – jamais redistribué
├─ Server\
│  ├─ <core>\                         binaires serveur (authentification, monde, extracteurs, DLL)
│  ├─ Config\                         *.conf.dist (modèles) → *.conf GÉNÉRÉS par le launcher ; my.ini généré
│  ├─ Database\mariadb\               MariaDB portable (zip officiel, allégé au packaging)
│  ├─ Database\sql\                   SQL de base du core (auth, characters, world) + mises à jour
│  ├─ Database\custom\*.sql           compléments appliqués après l'import de la base monde (AHBot, traductions ; ordre alphabétique)
│  └─ Data\                           dbc/maps/vmaps/mmaps EXTRAITS du client au premier démarrage – jamais redistribués
├─ UserData\                          Database\ (datadir MariaDB), settings.json, db-credentials.json, Sauvegardes\, diagnostics
├─ Logs\                              launcher.log, server\*.log, mariadb-error.log
├─ Tools\                             HeidiSQL portable, scripts PowerShell (Common.ps1 = chemins, ports, identifiants, Invoke-Sql)
├─ Build\                             manifest.json (composants figés : URL, commit, SHA-256), Get-Components.ps1, Sign-Launcher.ps1,
│                                     New-FilesManifest.ps1, New-Package.ps1, New-Iso.ps1, New-InstallerImages.ps1, files-manifest.json, package\
├─ Tests\                             scripts de test rejouables (voir §10)
├─ Documentation\                     01-Audit … 10-Validation, Manuel-Utilisateur.md, images\ (captures)
├─ Launcher\                          projet cargo (src\, assets\, build.rs, Cargo.toml, README.md)
├─ LICENCE.txt                        licence propriétaire HomeCraft (reprendre celle du Vanilla, version 1.1, en adaptant le nom)
└─ LICENCES\ (dans le package)        licences des composants tiers + SOURCES.md
```

**Ports** : le Vanilla utilise 3315 (MariaDB), 3724 (authentification), 8085 (monde) ; la version WotLK en prépare d'autres. Pour que les trois éditions puissent coexister sur le même PC, prends pour BC **3317 / 3726 / 8087** (MariaDB / authentification / monde), tous sur 127.0.0.1 : le port du monde va dans la table `realmlist`, le port d'authentification dans le `realmlist.wtf` du client (`set realmlist 127.0.0.1:3726`).

## 3. Pile Rust figée (reprendre telle quelle)

- Rust stable MSVC (rustup, `~/.cargo/bin` à ajouter au PATH de la session) + Visual Studio Build Tools 2022 « Développement Desktop C++ » (link.exe, rc.exe). Installés sur ce poste dans `C:\BuildTools`.
- `eframe`/`egui` **0.32** et `egui_extras` 0.32 (feature `image`) : **ne pas monter en 0.36**, l'API a changé et toute l'interface serait à réécrire.
- `mysql` 28 (`default-features = false, features = ["minimal-rust"]`) : client MySQL pur Rust, aucune DLL.
- `rfd` 0.17 (sélecteurs natifs), `zip` 2 (deflate), `sha2` 0.11, `serde`/`serde_json`, `image` 0.25 (png, jpeg), `rand`, `windows-sys` 0.61 (features `Win32_Storage_FileSystem`, `Win32_Foundation`, `Win32_System_Time`, `Win32_UI_WindowsAndMessaging`), `winresource` 0.1 en build-dependency.
- Vanilla seulement : `wow-mpq` 0.7 pour construire des archives MPQ V1 ; en 2.4.3 les patchs sont des MPQ V2 : vérifier que la crate gère cette version, sinon ne proposer que activer/désactiver/ajouter.
- Un seul exécutable, `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`, aucune dépendance GPL/LGPL liée (vérifié avec `cargo metadata`), ce qui permet la licence propriétaire du launcher.

## 4. Architecture interne (à reproduire)

- `main.rs` : détecte la racine (`paths::Layout::detect()` remonte depuis l'exe jusqu'au dossier contenant `Server\` et `Tools\`), charge `settings.json`, lance l'orchestrateur, analyse la ligne de commande, démarre eframe (icône de fenêtre depuis `assets\logo-256.png`). À la fermeture, envoie `Command::Shutdown` et attend 200 ms.
- `orchestrator.rs` : **thread dédié** avec une machine à états `State` (NotInstalled, Installed, Verifying, Initializing, StartingDb, StartingAuth, StartingWorld, Ready, Playing, Stopping, Error, Repairing), un `Snapshot` partagé `Arc<Mutex<…>>` (état, message, progression, services actifs, comptes, bases, tables, résultat de requête, sauvegardes, rapport, patchs, addons, état des mmaps, journaux ≤ 3000 lignes) et un canal `mpsc` de `Command` (StartAll, Play, StopAll, Verify, Repair, Backup, Restore, DeleteBackup, ExportDiagnostic, Console, CreateAccount, SetPassword, SetGm, DeleteAccount, ListTables, Query, Exec, LoadTable, OpenHeidi, SettingsChanged, InstallTo, KillGame, GenerateBots, DeleteBots, PatchSetActive, PatchAdd, PatchRemove, PatchBuild, AddonAdd, AddonSetActive, AddonRemove, GenerateMmaps, ReimportWorld, Shutdown). Boucle : `recv_timeout(1 s)` puis `tick()` (ping base, ports, processus, personnages en ligne, fin du jeu, fin de la génération des mmaps). L'interface ne touche jamais aux processus : elle lit le Snapshot et envoie des commandes.
- `db.rs` (`MariaDb`) : écrit `my.ini` (datadir `UserData\Database`, port, `bind-address=127.0.0.1`, `lower_case_table_names=2`, `innodb_buffer_pool_size` raisonnable), `initialize()` (`mariadb-install-db.exe` puis création de l'utilisateur), `start(timeout) -> Result<bool>` (spawn `mariadbd.exe`, attend port + `SELECT 1` ; **renvoie false si une base répondait déjà : elle n'est alors pas « possédée »**), `shutdown()` (`SHUTDOWN` SQL), `import_dumps(&[bases])` (DROP/CREATE/GRANT puis `mariadb.exe < dump`, puis `Server\Database\custom\*.sql` après la base monde ; **refuse** de recréer la base personnages si elle contient des personnages), `dump_database` (`mariadb-dump --lock-tables`), `query`/`exec`/`table_columns`/`table_rows`, suppression de compte hors ligne, comptage des bots.
- `credentials.rs` : `UserData\db-credentials.json` `{user, password, rootPassword}` générés aléatoirement au premier démarrage (retirer un éventuel BOM `\u{feff}` à la lecture).
- `server.rs` : génère les `.conf` depuis les `.conf.dist` (DataDir, LogsDir, chaînes de connexion `127.0.0.1;3317;user;mdp;base`, port, `BindIP`, Warden désactivé, anticheat désactivé, Motd, plus les surcharges du catalogue §6), `ensure_realm` (ligne `realmlist` id 1, nom « HomeCraft BC », adresse 127.0.0.1, port monde, build client 8606 autorisé), `Realmd`/`Mangosd` (spawn avec `CREATE_NO_WINDOW`, stdout lu dans un thread et renvoyé au journal, **console sur stdin** pour les commandes serveur ; `wait_ready` détecte la ligne « World initialized » ou l'ouverture du port ; arrêt propre par la commande console d'arrêt **puis fermeture de stdin**, kill après délai).
- `client.rs` : inspecte le dossier client (présence de `WoW.exe`, version lue dans `StringFileInfo` et non dans la version fixe, archives MPQ attendues de 2.4.3 : `common.MPQ`, `expansion.MPQ`, `patch.MPQ`, `patch-2.MPQ`, `frFR\locale-frFR.MPQ`, `frFR\expansion-locale-frFR.MPQ`, `frFR\patch-frFR.MPQ`, `frFR\patch-frFR-2.MPQ`, `frFR\speech-frFR.MPQ`, `frFR\expansion-speech-frFR.MPQ` ; liste exacte à vérifier sur le client de l'utilisateur), écrit **`Data\frFR\realmlist.wtf`** (`set realmlist 127.0.0.1:3726` + `set patchlist 127.0.0.1`) en conservant l'original en `.orig`, fusionne `WTF\Config.wtf` clé par clé (`gxWindow`, `gxResolution` bornée à l'écran, `locale "frFR"`, `realmName`), lance `WoW.exe`, kill du jeu.
- `settings.rs` : `UserData\settings.json` (`client_dir`, `windowed=true`, `ahbot_enabled`, `ah_faction`, `partybots_enabled`, `conf: BTreeMap<clé, valeur>` = surcharges du catalogue, `auto_mmaps=true`, `auto_backup=true`, `auto_backup_keep=10`) ; `#[serde(default)]` partout pour la compatibilité ascendante.
- `conf_catalog.rs` : **catalogue** des clés du `.conf` monde exposées (clé exacte, libellé FR, aide, type Bool/Int/Float/Choice/Text, valeur par défaut du `.conf.dist` du commit figé). Seules ces clés peuvent être surchargées ; ports, chemins et chaînes de connexion restent gérés. ~70 clés Monde (rates XP/butin/réputation, difficulté des créatures, AllowTwoSide.*, GM.*, quêtes, instances, mode héroïque, visibilité, Motd…) et ~14 clés bots.
- `backup.rs` : dumps `characters` + base d'authentification (tables verrouillées), `settings.json`, manifeste `backup.json` (SHA-256 par fichier, date, libellé, nombre de personnages/comptes) dans un zip horodaté `UserData\Sauvegardes\sauvegarde-AAAAMMJJ-HHMMSS.zip` ; `restore` = vérification des empreintes, sauvegarde de sécurité automatique, import ; sauvegarde automatique à chaque fermeture du jeu avec rotation (N conservées, libellé dédié).
- `repair.rs` : vérification des fichiers gérés par `Build\files-manifest.json` (taille + SHA-256), `data_status` (dbc/maps/vmaps/mmaps présents, avec marqueur « génération en cours »), extraction des données du client avec les outils du core (`run_tool` avec sortie limitée à une ligne toutes les 5 s), génération des mmaps **en arrière-plan** (moitié des cœurs) après le démarrage du serveur, export de diagnostic (journaux + confs avec mots de passe masqués).
- `botgen.rs` (si le module de bots n'a pas sa propre génération), `patches.rs` (liste, activer/désactiver par renommage `.off`, ajouter, supprimer), `addons.rs` (installation depuis zip avec détection du dossier contenant le `.toc`, interface attendue 20400 pour 2.4.3), `mpqbuild.rs` (si MPQ V2 pris en charge), `i18n.rs` (**toutes** les chaînes visibles, en français, dans un seul module), `util.rs`, `ui.rs` (onglets, fenêtre de confirmation `Pending` pour toute action irréversible, fenêtre de rapport).

## 5. Ce que fait le bouton JOUER (séquence exacte `start_all`)

1. Vérifie composants et client (sinon ERREUR explicite).
2. Identifiants SQL (créés au besoin), `my.ini`, initialisation du datadir si absent.
3. Démarre MariaDB, ou s'attache à une base déjà en service **sans en prendre la propriété** (ne jamais l'arrêter ensuite ; incident réel corrigé en Vanilla).
4. Bases manquantes → import des dumps + compléments `custom\*.sql`, chaque étape journalisée (« import : complément x.sql »).
5. Génère les `.conf`, assure le royaume et le build client autorisé.
6. Données du client absentes → extraction dbc/maps/vmaps (première installation, quelques minutes) ; mmaps en arrière-plan ensuite.
7. Démarre l'authentification (attend le port), puis le monde (attend « World initialized », délai 10 min), rafraîchit les comptes.
8. Configure le client (realmlist, Config.wtf) et lance le jeu ; état JEU EN COURS ; `tick` détecte la fin du jeu → sauvegarde automatique.
9. Fermeture de la fenêtre : arrêt propre monde → authentification → base (seulement si possédée).

## 6. Onglets et fonctions livrées en Vanilla (toutes demandées par l'utilisateur, à reproduire)

- **Accueil** : état des composants, boutons JOUER / DÉMARRER LE SERVEUR / ARRÊTER LE SERVEUR / FERMER LE JEU (kill de WoW.exe), VÉRIFIER LES FICHIERS, RÉPARER, SAUVEGARDER, RESTAURER, diagnostic (dossier, version, client, mmaps, personnages en ligne).
- **Comptes** : création (identifiant, mot de passe, niveau GM, **expansion = 1 pour BC** dans la table des comptes, à vérifier), changement de mot de passe, niveau GM, **suppression définitive avec confirmation, personnages compris**, liste avec dernière connexion et nombre de personnages. Commandes console des cores MaNGOS : `account create`, `account set gmlevel`, `account set addon` (expansion), `account delete` ; niveaux GM 0 à 3 (4 = console) en CMaNGOS, à vérifier.
- **PlayerBots** : réglages du module de bots exposés par catalogue, aide-mémoire des commandes en jeu, génération et suppression de bots de peuplement. En BC, le core n'a pas de bots intégrés : voir §11 pour les candidats ; à exposer de la même façon (compagnons de groupe tank/soigneur/DPS, bots aléatoires en ville, champs de bataille).
- **Monde** : catalogue des réglages du `.conf` monde par groupe (ajoute pour BC : mode héroïque, Outreterre, montures volantes, arènes), valeur par défaut rappelée, « modifié » signalé, « Tout remettre aux valeurs par défaut », appliqué au prochain démarrage.
- **Patchs** : archives `patch-X.MPQ` (dossier `Data\`) et `patch-frFR-X.MPQ` (dossier `Data\frFR\`) du client, activer/désactiver (renommage `.off`), ajouter, supprimer, créer depuis un dossier si MPQ V2 géré. Ordre de chargement 2.4.3 à vérifier.
- **Base de données** : explorateur natif façon HeidiSQL (arbre bases/tables, grille de 200 lignes éditable par double-clic, filtre WHERE, tri, insertion/suppression avec confirmation, onglet Structure, onglet SQL avec lecture libre et écriture confirmée, historique) + bouton « Ouvrir dans HeidiSQL » (portable dans `Tools\HeidiSQL`, arguments `-h=127.0.0.1 -P=3317 -u=… -p=…`, sans guillemets autour de l'hôte).
- **Sauvegardes** : manuelle avec libellé, automatique à la fermeture du jeu (rotation), de sécurité avant restauration ; restauration avec confirmation ; suppression ; ouverture du dossier ; export de diagnostic.
- **Paramètres** : dossier du client (sélecteur), fenêtré (recommandé ; le launcher borne la résolution), options client éventuelles (le client 2.4.3 est 32 bits comme le 1.12.1 ; DXVK en option, VanillaFixes ne s'applique pas), AHBot (activé, faction servie), mmaps automatiques, sauvegarde automatique + nombre conservé, « Réimporter la base monde… », section **AddOns** (installation depuis zip, activer/désactiver/supprimer).
- **Journaux** : journal du launcher + sortie serveur, champ de commande console.
- Ligne de commande pour les tests : `--start`, `--quit-after N`, `--tab <onglet>`, `--do backup|verify|repair|diagnostic|restore <zip>|botgen N|botdel|patchon|patchoff|addonadd|addonon|addonoff|addondel|accdel|mpqbuild|reimportworld|mmaps`, `--install-to <dossier>`.

## 7. Signature, manifeste, déploiement

- Signature Authenticode SHA-256 avec horodatage DigiCert, certificat **auto-signé** (réutilise celui du Vanilla, empreinte dans `C:\Users\Marty\Desktop\HomeCraft\UserData\code-signing-thumbprint.txt`, ou crée-en un pour BC avec `New-SelfSignedCertificate -Type CodeSigningCert`). `Build\Sign-Launcher.ps1 -Thumbprint …` signe **et déploie** l'exe à la racine ; à relancer **après chaque build**, puis `Build\New-FilesManifest.ps1` (le manifeste contient l'empreinte de l'exe). Appeler ce script avec `&` dans une session PowerShell, pas `powershell -File` (sinon `$PSScriptRoot` est vide dans les valeurs par défaut des paramètres).

## 8. Composants et packaging

- Tous les composants externes passent par `Build\manifest.json` (id, source, licence, commit ou version, date, URL, fichier, taille, SHA-256, dossier d'installation) et `Build\Get-Components.ps1 -Pin`. Rien d'autre n'est jamais téléchargé, surtout pas le client. **Demander l'accord de l'utilisateur avant chaque téléchargement.**
- MariaDB 11.8 : zip officiel « winx64 » depuis `archive.mariadb.org` ; extraire avec `tar.exe -xf … --strip-components=1`. Vérifier que les SQL du core BC s'importent dans MariaDB (fonctions ou collations spécifiques à MySQL 8 ?).
- `Build\New-Package.ps1 -Zip` : copie launcher signé, serveur, modèles `.conf.dist`, SQL, compléments, MariaDB allégée, HeidiSQL, scripts, documentation, `LISEZMOI.txt`, `LICENCE.txt`, `LICENCES\` (licences tiers + `SOURCES.md` généré), `Sources\Launcher`, `Build\files-manifest.json`, `SHA256SUMS.txt`. Jamais `Client\`, `Server\Data\`, `UserData\`, `Logs\`.
- Décisions de l'utilisateur : **portable uniquement** (un installateur Inno Setup a été construit et testé en Vanilla puis abandonné parce qu'il aurait dû contenir le client) ; **le client n'est jamais inclus dans un livrable, même si demandé** ; distribution à la famille et aux amis ; publication possible sur GitHub (dépôt public, `.gitignore` excluant client, données, UserData, binaires tiers, package).
- Licence : reprendre `LICENCE.txt` du Vanilla (version 1.1 : usage personnel, lecture du code, consultation et fork sur GitHub, reproduction/diffusion/modification interdites, exclusion de garantie, limitation de responsabilité, droit français, composants tiers exclus). Si le core BC est GPL, joindre ses sources exactes (commit) et sa licence dans le package.

## 9. Pièges rencontrés en Vanilla (ne pas les redécouvrir)

- PowerShell 5.1 : scripts en **UTF-8 avec BOM**, jamais `2>$null` sur un exécutable natif (`cmd /c … >nul 2>&1`), `Start-Process -Wait` attend aussi les enfants graphiques (boucle de sondage), `Set-Content` sans `-Encoding utf8` écrit en ANSI.
- Ne jamais lancer de test qui démarre ou arrête des services pendant qu'un launcher de l'utilisateur tourne (Vanilla, WotLK ou BC) ; d'où les ports distincts et la base « possédée » / « annexe ».
- `WoW.exe` : la version « fixe » de la ressource peut mentir ; lire `StringFileInfo`. Le serveur doit accepter le build réel (`realmlist.gamebuild`, clients autorisés, liste interne du core).
- Console serveur : la première commande envoyée juste après le démarrage peut être ignorée ; après la commande d'arrêt, **fermer stdin**.
- Extraction : vérifier les chemins de sortie exacts attendus par le core, créer les dossiers manquants, chemins courts (les extracteurs tronquent les chemins longs) ; les extracteurs ne lisent que les archives officielles, un patch n'a jamais d'effet sur le serveur.
- Client localisé : en 2.4.3 les fichiers de langue et le `realmlist.wtf` sont dans `Data\frFR\`, pas à la racine de `Data\`.
- Écran noir du client en fenêtré à la résolution exacte de l'écran : borner la résolution ; bouton FERMER LE JEU.
- Les tables du core peuvent avoir des **doublons d'entry** : `COUNT(DISTINCT …)`.
- Les bots de peuplement cachés de `/qui` par défaut ont surpris l'utilisateur : expliquer le réglage dans le manuel.
- egui 0.32 : `DragValue::new(&mut v).range(a..=b)`, `TextEdit::singleline(&mut s).password(true)`, `Grid`, `ScrollArea` ; toutes les chaînes dans `i18n.rs`.

## 10. Tests à écrire (tous rejouables, tous verts avant livraison)

`Test-ServerStartup` (scripts seuls), `Test-Launcher` (`--start --quit-after 90`), `Test-Phase5` (sauvegarde, vérification, réparation, réimport monde, restauration avec compte témoin, diagnostic ; 20 contrôles), `Test-Package` (copie vierge dans `C:\HomeCraftTestBC`, premier démarrage complet, arrêt brutal puis redémarrage, compléments SQL journalisés, aucun orphelin), `Test-Localisation` (couverture frFR des tables du monde → `09-Localisation-Inventaire.md`), `Test-Offline`. Les tests mettent `auto_mmaps:false` dans `settings.json`. Captures d'écran des onglets pour le README : lancer `--tab <onglet> --quit-after 14` et photographier le rectangle DWM de la fenêtre (script `Capture-Launcher.ps1` à reprendre de la session Vanilla).

## 11. Adaptations BC 2.4.3 à vérifier avant de coder

| Sujet | Vanilla (fait) | BC (à vérifier dans le core choisi) |
|---|---|---|
| Core | VMaNGOS, binaires Windows officiels, GPL-2 | Candidats à auditer : **CMaNGOS mangos-tbc** (GPL-2, actif, sans binaire officiel : compilation CMake/VS2022 nécessaire, DLL MySQL/OpenSSL à livrer), autres cores 2.4.3 communautaires. Choisir sur : activité, fidélité, binaires ou facilité de compilation, licence, compatibilité avec un module de bots |
| Bases | `realmd`, `characters`, `mangos`, `logs` | CMaNGOS-tbc : `tbcrealmd`, `tbccharacters`, `tbcmangos`, `tbclogs` + base de contenu séparée (`tbc-db`, à importer avec son script `InstallFullDB`) ; mises à jour SQL à appliquer dans l'ordre |
| Client | 1.12.1 build 5875/5880, MPQ à plat dans `Data\` | 2.4.3 **build 8606**, `Data\` + `Data\frFR\`, `realmlist.wtf` dans `Data\frFR\`, `Config.wtf` dans `WTF\` |
| Extracteurs | MapExtractor, VMapExtractor, VMapAssembler, MoveMapGenerator | CMaNGOS : `ad.exe` (dbc/maps), `vmap_extractor`, `vmap_assembler`, `MoveMapGen` (scripts `ExtractResources.sh` à transposer) ; dossiers `dbc`, `maps`, `vmaps`, `mmaps` |
| Bots | PartyBots intégrés au core | **cmangos/playerbots** (module officiel CMaNGOS, GPL-2, compilé avec le core : `BUILD_PLAYERBOTS`), ou autre fork ike3 ; commandes `/w <bot>`, `.bot add`, base et `.conf` propres ; vérifier la licence et l'état du module pour TBC |
| AHBot | table `auctionhousebot` remplie par script | AHBot intégré à CMaNGOS (`ahbot.conf`, `ahbot.conf.dist`) : réglages par qualité d'objet plutôt que table d'objets ; adapter l'onglet Paramètres |
| GM | `account set gmlevel 6` | niveaux 0-3 (console = 4), `account set addon <compte> 1` pour autoriser BC |
| Localisation | `locales_*` (loc2 = frFR) | tables `locales_*` CMaNGOS (même convention loc2 = frFR, à vérifier) ; mesurer la couverture, compléter via `custom\*.sql` sans écraser l'existant |
| Contenu progressif | `WowPatch` 1.2 → 1.12 | pas d'équivalent natif en CMaNGOS-tbc (à vérifier) ; ne pas exposer de réglage inexistant |

Procède par phases comme pour la version Vanilla : audit (composants, licences, binaires ou compilation) → construction (base, serveur, extraction, test de connexion manuel) → launcher (identité visuelle et icône d'abord, puis démarrage/arrêt, puis onglets un par un) → fiabilité (sauvegarde, restauration, réparation) → packaging portable → validation finale, chaque phase documentée dans `Documentation\` et couverte par un test. Réponds en français, note chaque décision de l'utilisateur et chaque piège dans la documentation, et demande son accord avant tout téléchargement ou compilation longue.
