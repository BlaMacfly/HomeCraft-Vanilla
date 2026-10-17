# HomeCraft Edition BC — Phase 2 : construction

Date : 2026-10-16
Statut : **composants installés, bases importées, realmd opérationnel, mangosd validé jusqu'au chargement des cartes.
Extraction des données et test de connexion en jeu en attente du client 2.4.3** (téléchargement de l'utilisateur en cours).

Décision de l'utilisateur : accord donné le 2026-10-16 pour les quatre téléchargements du manifeste (« reprend »).

---

## 1. Composants figés (Build/manifest.json, vérifiés par Get-Components.ps1 -Pin)

| id | Fichier | Taille | SHA-256 | Installé dans |
|---|---|---|---|---|
| mangos-tbc-windows | `mangos-tbc-with-all-15b6ddb4.zip` | 86 344 468 | `ff1f26c7…207f` (digest GitHub, conforme) | `Server/mangos-tbc` (dossier `x64_RelWithDebInfo` aplati, `*.pdb` exclus) |
| mangos-tbc-source | `mangos-tbc-src-15b6ddb4.zip` | — | `1e6482bc…bb9a` (figé) | `Build/tmp/mangos-tbc-src` (référence, obligation GPL) |
| cmangos-playerbots-source | `cmangos-playerbots-81fea4cd.zip` | — | `b2658ec0…cae1` (figé) | `Build/tmp/playerbots-src` (référence, obligation GPL) |
| tbc-db | `tbc-all-db-87af76a.zip` | 65 628 423 | `2fdd8743…9278` (digest GitHub, conforme) | `Server/Database/sql` |
| mariadb-server-windows | `mariadb-11.8.9-winx64.zip` | 97 016 251 | `830c4672…63f5` | `Server/Database/mariadb` (copie du téléchargement Vanilla) |
| heidisql-portable | `HeidiSQL_12.21_64_Portable.zip` | 29 789 003 | `fecb76a6…3fab` | `Tools/HeidiSQL` (copie du téléchargement Vanilla) |

Extraction : **`tar.exe` de Windows** (`%SystemRoot%\System32\tar.exe`, bsdtar). Le `tar` de Git Bash (GNU) ne lit pas
les zip (« This does not look like a tar archive »).

## 2. Contenu vérifié des archives (points « à vérifier » de l'audit levés)

### Build Windows `with-all`
`realmd.exe`, `mangosd.exe` (31 Mo), `libmysql.dll` (client MySQL 8.4.0), `libcrypto-3-x64.dll`, `libssl-3-x64.dll`,
`legacy.dll` (fournisseur OpenSSL 3 « legacy »), `realmd.conf.dist`, `mangosd.conf.dist`, `aiplayerbot.conf.dist`,
`ahbot.conf.dist`, `anticheat.conf.dist`, `warden_modules/`, et `Extractors/` : `ad.exe`, `vmap_extractor.exe`,
`vmap_assembler.exe`, `MoveMapGen.exe`, `config.json`, `offmesh.txt`, `ExtractResources.sh`, `MoveMapGen.sh`.
Commit compilé `15b6ddb4…` (bannière : « Built on Oct 10 2026 », « CMaNGOS TBC World server v0.18 »).

### TBC-DB `tbc-all-db.zip`
Quatre **dumps complets** `mysqldump` (MySQL 8.0.46) : `tbcrealmd.sql` (18 Ko), `tbccharacters.sql` (2,1 Mo),
`tbcmangos.sql` (283 Mo), `tbclogs.sql` (4 Ko). Aucun script `InstallFullDB.sh` ni mise à jour à appliquer :
les dumps sont déjà à jour. Ils contiennent **déjà** les tables PlayerBots (`ai_playerbot_*` dans tbccharacters et
tbcmangos, `playerbot_saved_data`) et AHBot (`ahbot_*`). Jeux de caractères : `utf8mb3` (288 tables) et `latin1` (7),
aucune collation MySQL 8 (`utf8mb4_0900_*`) : **import MariaDB 11.8 sans erreur**.
Version annoncée par mangosd : « TBC-DB 1.11.0 'Vengeance One: A Cmangos Story' », EventAI « ACID 2.4.3 ».

### PlayerBots
Licence : en-tête GPL-2.0 « or any later version » (copyright MaNGOS project) dans `CMakeLists.txt` du module ;
pas de fichier `LICENSE` à la racine du dépôt. Joindre la licence GPL-2 et les sources exactes au package.
Configuration : `aiplayerbot.conf.dist` (déjà dans le build, variante TBC). Défauts du module : **1000 bots aléatoires**
sur 200 comptes, connectés au démarrage : réduit à 50 bots par `New-ServerConfig.ps1 -RandomBots` (réglage à exposer
dans l'onglet PlayerBots).

## 3. Disposition et ports

| Élément | Valeur |
|---|---|
| Binaires | `Server/mangos-tbc/` |
| Modèles | `Server/Config/*.conf.dist` (copiés du build ; recopiés si leur empreinte change) |
| Configurations générées | `Server/Config/{realmd,mangosd,aiplayerbot,ahbot}.conf`, `Server/Config/my.ini` |
| Données client | `Server/Data/{dbc,maps,vmaps,mmaps,Buildings,Cameras}` (`DataDir`) |
| Datadir MariaDB | `UserData/Database` |
| Journaux | `Logs/` (`mariadb-error.log`), `Logs/server/` (`Realmd.log`, `Server.log`, `DBErrors.log`, …) |
| MariaDB / realmd / mangosd | 127.0.0.1 : **3317 / 3726 / 8087** (vérifié : realmd n'écoute que sur 127.0.0.1:3726) |
| Bases | `tbcrealmd`, `tbccharacters`, `tbcmangos`, `tbclogs` (utf8mb3) |
| Royaume | `realmlist` id 1 « HomeCraft BC », 127.0.0.1:8087, icon 0 (Normal), `realmbuilds = '8606'` |

## 4. Scripts (Tools/, UTF-8 avec BOM)

| Script | Rôle |
|---|---|
| `Common.ps1` | chemins, ports, noms de bases (`$Databases`), identifiants, `Invoke-Sql`, `Test-Port`, `Test-MariaDbAlive` |
| `Start-MariaDB.ps1` / `Stop-MariaDB.ps1` | `my.ini` géré (`lower_case_table_names=2`, buffer pool 512 Mo, paquets 256 Mo), initialisation du datadir, démarrage, arrêt SQL |
| `Init-Database.ps1` | compte SQL local, import des 4 dumps (refus si personnages sans `-IKnowThisDeletesCharacters`), compléments `custom/*.sql` après la base monde, suppression des comptes d'exemple |
| `New-ServerConfig.ps1` | génération des `.conf` (chaînes `127.0.0.1;3317;user;mdp;base`, BindIP 127.0.0.1, ports, Motd, `GameType = 0`, `Expansion = 1`, console activée, RA/SOAP désactivés, bots aléatoires) |
| `Run-World.ps1` / `Send-Console.ps1` | harnais de développement (royaume, realmd, mangosd avec console sur stdin, arrêt `server shutdown 0` + fermeture de stdin) |
| `Extract-ClientData.ps1` | transposition de `ExtractResources.sh` : `ad -i <client> -o <Data>`, `vmap_extractor -d <client>\Data -o <Data>`, `vmap_assembler Buildings vmaps`, `MoveMapGen --configInputPath config.json --offMeshInput offmesh.txt --workdir <Data>\ --threads N --buildGameObjects` (**non testé** : client absent) |
| `Capture-Launcher.ps1` | captures des onglets du launcher |

## 5. Résultats vérifiés le 2026-10-16

| Étape | Résultat |
|---|---|
| Initialisation MariaDB + import des 4 bases | 33 s au total (tbcmangos 28 s), aucune erreur ; 13 / 82 / 197 / 3 tables ; 109 357 créatures placées, 6 599 quêtes |
| realmd | démarre, « Added realm id 1, name 'HomeCraft BC' », écoute **127.0.0.1:3726** uniquement |
| mangosd | se connecte aux 4 bases (MariaDB 11.8.9 via libmysql 8.4.0), accepte le schéma, s'arrête au contrôle des cartes : « Correct *.map files not found » (attendu sans extraction) |

## 6. Pièges rencontrés (à reporter dans le launcher)

1. **Dossier courant obligatoire = `Server/mangos-tbc`**. Le core appelle
   `OSSL_PROVIDER_set_default_search_path(nullptr, current_path())` puis charge le fournisseur OpenSSL « legacy » :
   lancé depuis un autre dossier, realmd **et** mangosd s'arrêtent aussitôt (« OpenSSL3: Failed to load Legacy
   provider: … could not load the shared library »), sans autre message. `server.rs` doit passer
   `current_dir(Server/mangos-tbc)`.
2. `anticheat.conf` est lu dans le dossier courant (chemin relatif codé). Pour ne rien écrire dans le dossier du
   programme (vérifié par `files-manifest.json`), le fichier n'est pas généré : défauts du module à 0
   (`Enable`, `Warden.Enable`, `Antispam.Enable` valent `false` dans `config.cpp`) et surcharge explicite par
   variables d'environnement `Anticheat_Enable=0`, `Anticheat_Warden_Enable=0`, `Anticheat_Antispam_Enable=0`
   (mécanisme `EnvVarForIniKey` du core : préfixe + clé, points remplacés par `_`). Le journal affiche
   « [Anticheat] Could not find configuration file anticheat.conf » : sans conséquence.
3. **Pas de Warden dans `mangosd.conf`** en TBC (contrairement à VMaNGOS) : il n'existe que dans le module anticheat.
4. Clés de connexion sans point (`LoginDatabaseInfo`, `WorldDatabaseInfo`…) dans realmd **et** mangosd (VMaNGOS : `LoginDatabase.Info`).
5. Arrêt console : `server shutdown <secondes>` (`server exit` existe aussi mais exige `SEC_CONSOLE`) ; fermer stdin ensuite.
6. Ligne « CMANGOS: World initialized » dans la sortie de mangosd (`World.cpp:1506`) : marqueur de `wait_ready`.
7. Les dumps contiennent des comptes d'exemple (ADMINISTRATOR, GAMEMASTER, MODERATOR, PLAYER) aux mots de passe
   publics : supprimés à l'import.
8. Les options `-a` (ahbot) et `-p` (aiplayerbot) de mangosd prennent un chemin ; sans elles, les fichiers sont
   cherchés dans le dossier courant.

## 7. Client 2.4.3 de l'utilisateur (reçu le 2026-10-17)

- Archive `Téléchargements\TGCfrFR.zip` (8,6 Go, 208 entrées, dossier racine `WoW2.4.3FR`), extraite avec `tar.exe`
  dans `Client\` (170 fichiers, 8,12 Go, 22 s). Origine : client distribué par le serveur privé « The Geek Crusade ».
- `Wow.exe` (avec cette casse) : version 2.4.3.8606, identique dans StringFileInfo et dans la version fixe.
  SHA-256 `872FC86BFAE202A1529A59E45CD2F76E0C76EFCC5E9AE6CD100E8177A158E099`.
- Archives : `Data\common.MPQ`, `expansion.MPQ`, `patch.MPQ`, `patch-2.MPQ` ; `Data\frFR\locale-frFR`, `expansion-locale-frFR`,
  `patch-frFR`, `patch-frFR-2`, `speech-frFR`, `expansion-speech-frFR`, `base-frFR`, `backup-frFR` (officielles)
  **et `patch-frFR-3.MPQ`** (7,1 Mo, daté du 2026-10-14, non officiel, ajouté par le serveur tiers).
- **`realmlist.wtf` est à la racine du client** en 2.4.3 (le déplacement dans `Data\<locale>\` date de WotLK) :
  le cahier des charges supposait `Data\frFR\` ; c'est la racine qui fait foi. L'original (`set realmlist
  auth.thegeekcrusade-serveur.com`) est conservé en `realmlist.wtf.orig`.
- `WTF\` est vide à l'origine : `Config.wtf` est créé (locale frFR, fenêtré, 1600x900 sur un écran 1920x1080).

### Piège majeur : patchs tiers et extraction

`ad.exe` charge **toutes** les archives `patch-frFR-*.MPQ` (journal : « Opening …/patch-frFR-3.MPQ »). Contrairement
au constat Vanilla (« un patch n'a jamais d'effet sur le serveur »), un patch tiers modifie ici les données serveur.
Comparaison des DBC extraits avec et sans `patch-frFR-3.MPQ` (extraction témoin `ad -e 2`) : 185 DBC dans les deux
cas, **5 différents** :

| DBC | avec patch-frFR-3 | sans |
|---|---|---|
| Spell.dbc | 26 989 414 o | 25 991 940 o |
| ItemDisplayInfo.dbc | 4 764 584 o | 4 762 976 o |
| CreatureDisplayInfo.dbc | 993 177 o | 992 346 o |
| CreatureModelData.dbc | 101 514 o | 100 777 o |
| SpellIcon.dbc | 78 045 o | 77 771 o |

Décision : les données du serveur sont extraites **des seules archives officielles** (elles doivent correspondre à
TBC-DB). `Extract-ClientData.ps1` construit une vue du client par liens physiques (`UserData\extract-view`) sans les
archives non officielles ; le client n'est pas modifié et continue de charger `patch-frFR-3.MPQ` côté jeu (à
désactiver dans l'onglet Patchs si des incohérences visuelles apparaissent). Le launcher (`repair.rs`) doit
reproduire cette vue.

Autre piège : avec un chemin de ~150 caractères (dossier temporaire de session), `ad.exe` affiche « No locales
detected » et n'extrait rien (code de sortie 0 malgré tout) : contrôler la présence de `dbc\` après l'extraction.

### Extraction réalisée (2026-10-17, archives officielles seules)

`Tools/Extract-ClientData.ps1` (vue `UserData\extract-view`, `patch-frFR-3.MPQ` ignoré) : ad 26 s, vmap_extractor
2 min, vmap_assembler 1 min. `Server/Data` : dbc 185 fichiers (48 Mo, `Spell.dbc` = 25 991 940 o, version officielle),
maps 3 586 (196 Mo), vmaps 8 607 (399 Mo), Buildings 5 431 (210 Mo), Cameras 13. mmaps non générées.
Un dossier témoin `C:\HCBC-tmp` (comparaison des DBC) reste à supprimer par l'utilisateur : la suppression à la racine
de `C:\` est bloquée pour la session.

## 8. Reste à faire dans cette phase

1. Relever la version de `WoW.exe` (StringFileInfo) et la liste exacte des MPQ de `Data\` et `Data\frFR\`.
2. `Extract-ClientData.ps1` (dbc/maps/vmaps), puis démarrage complet de mangosd jusqu'à « World initialized ».
3. Compte de test : `account create`, `account set addon <compte> 1`, `account set gmlevel <compte> 3`.
4. `Data\frFR\realmlist.wtf` = `set realmlist 127.0.0.1:3726`, connexion manuelle, création de personnage, bots.
5. `MoveMapGen` (mmaps) en arrière-plan : mesurer la durée.
