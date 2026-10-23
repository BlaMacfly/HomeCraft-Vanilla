# Construction et exploitation (poste développeur)

Dernière mise à jour : 2026-10-14. État : Phase 2 en cours (tâches 2.1 et 2.2 validées, 2.3 en cours).

## Composants figés

Voir `Build/manifest.json`. Récupération et vérification :

```bash
powershell -ExecutionPolicy Bypass -File Build\Get-Components.ps1
```

| Composant | Version figée | Emplacement | Licence |
|---|---|---|---|
| VMaNGOS (realmd, mangosd, extracteurs) | commit `53d24d5` du 2026-10-07, build Windows x64 officiel | `Server/vmangos/` | GPL-2.0 (`Server/Config/LICENSE`) |
| Base VMaNGOS (dumps MySQL) | snapshot `4641790` du 2026-09-28 | `Server/Database/sql/` | GPL-2.0 |
| MariaDB Community Server | 11.8.9 LTS (zip winx64, sans installeur) | `Server/Database/mariadb/` | GPL-2.0 |
| Modèles de configuration | `*.conf.dist.in` du même commit | `Server/Config/` | GPL-2.0 |
| HeidiSQL portable (bouton « Ouvrir dans HeidiSQL ») | 12.21 du 2026-08-03, zip 64 bits, 79 Mo extraits | `Tools/HeidiSQL/` (`gpl.txt`, `license.txt`, `LICENSE-openssl` inclus) | GPL-2.0 |

Le client WoW 1.12.1 frFR est fourni par l'utilisateur (ici : `Client/`, **exclu de tout packaging**).

## Disposition des dossiers et ports

| Élément | Chemin | Remarque |
|---|---|---|
| Données personnelles (base MariaDB, identifiants SQL) | `UserData/` | hors `Server/` : jamais écrasé par une mise à jour |
| Données extraites du client (dbc, maps, vmaps, mmaps) | `Server/Data/` | dérivées des MPQ : non redistribuables, régénérées à l'installation |
| Configurations générées | `Server/Config/realmd.conf`, `mangosd.conf` | régénérées à chaque démarrage par `Tools/New-ServerConfig.ps1` |
| Journaux | `Logs/` | `mariadb-error.log`, `server/`, `extract/`, `tests/` |

| Service | Adresse | Port |
|---|---|---|
| MariaDB | 127.0.0.1 | 3315 (non standard, évite les conflits avec un MySQL/XAMPP existant) |
| realmd (authentification) | 127.0.0.1 | 3724 |
| mangosd (monde) | 127.0.0.1 | 8085 |

Tous les services écoutent uniquement sur l'interface de bouclage : aucune règle de pare-feu, aucun port routeur.

## Scripts (Tools/)

| Script | Rôle |
|---|---|
| `Common.ps1` | chemins, ports, identifiants SQL générés (`UserData/db-credentials.json`), `Invoke-Sql`, contrôle de santé MariaDB |
| `Start-MariaDB.ps1` / `Stop-MariaDB.ps1` | génère `my.ini`, initialise le datadir au premier lancement, démarre/arrête proprement (`shutdown` SQL, jamais de kill) |
| `Init-Database.ps1` | crée le compte SQL `mangos` (loopback), importe realmd/characters/mangos/logs ; **refuse** de ré-importer si des personnages existent sans `-IKnowThisDeletesCharacters` |
| `New-ServerConfig.ps1` | génère les deux `.conf` depuis les modèles (erreur si une clé attendue disparaît du modèle VMaNGOS) |
| `Extract-ClientData.ps1` | vérifie le build 5875 puis lance MapExtractor, VMapExtractor, VMapAssembler, MoveMapGenerator |

Tests (Tests/) : `Test-ServerStartup.ps1` enchaîne MariaDB → realmd → mangosd et vérifie connexion base, migrations, écoute des ports.

## Résultats vérifiés le 2026-10-14

- Import des dumps VMaNGOS (MySQL 5.6) dans **MariaDB 11.8.9** : OK en 12 s. realmd 13 tables / 9 migrations, characters 55 / 8, mangos 175 / 1141, logs 10 / 7.
- `mangosd` révision 53d24d5 : connexion aux 4 bases OK, aucun avertissement de migration, royaume « Vanilla Solo » lu ; arrêt attendu sur l'absence de `maps/` avant extraction.
- Couverture française (loc2) de la base monde : quêtes 4 167 titres / 4 727 (88 %), descriptions 3 621 ; objets 17 556 / 20 034 (88 %) ; objets du monde 7 991 / 9 530 ; créatures 6 565 / 15 217 ; textes diffusés 11 030 / 11 108 ; pages 1 238 / 1 257.
- Client fourni : WoW.exe 1.12.1.5875 frFR, 5,27 Go de MPQ. Particularités : `patch-3.MPQ` (9,6 Mo, non officiel, origine Nostalgeek), `Data/Spell.dbc` libre (15,8 Mo), `realmlist.wtf` pointant vers Nostalgeek, `WoW.dxvk-cache`. À traiter : contenu de `patch-3.MPQ` à inventorier avant de décider de le conserver.

## Comportements VMaNGOS relevés (commit 53d24d5)

- DBC attendues dans `DataDir/5875/dbc` (sous-dossier par build client) ; `DBC.Locale = 255` détecte **frFR** si les DBC viennent du client français.
- VMapAssembler ne crée pas le dossier `vmaps` : le créer avant (fait dans `Extract-ClientData.ps1`).
- MapExtractor tronque les chemins de sortie longs (`-o`) : utiliser des chemins courts.
- Les extracteurs VMaNGOS ne lisent que `model`, `dbc`, `terrain`, `patch` et `patch-2` (`contrib/extractor/System.cpp`) : `patch-3.MPQ` et tout patch supplémentaire sont **ignorés côté serveur** (la comparaison avec/sans patch-3 du 2026-10-14 était donc sans objet) ; `Data/Spell.dbc` libre est identique à la version extraite.
- Console mangosd : la première commande envoyée juste après « World initialized » peut être ignorée (« Cette commande n'existe pas ») ; réémettre. `account create` et `account set gmlevel` sont `SEC_CONSOLE` ; le niveau GM va dans `realmd.account_access` (realm 1).
- Arrêt propre : `server exit` sur la console, **puis fermer l'entrée standard**, sinon le processus reste bloqué après « Halting process... » (thread console) ; durée d'arrêt observée < 1 s.
- Toutes les commandes `.partybot` sont `SEC_ADMINISTRATOR` (= gmlevel 6) ; `PartyBot.SkipChecks = 1` lève les restrictions (combat, instance) pour les non-GM.
- **Build client 5880** : le client frFR fourni annonce « 1.12.1 (5880) » à l'écran de connexion (ressource FileVersion : 5875, VS_FIXEDFILEINFO : 5880). realmd n'accepte que les builds de `allowed_clients` (5875 seul pour 1.12.1 dans le snapshot) et n'affiche le royaume que si le build figure dans `realmlist.realmbuilds` (liste séparée par des espaces). Correction (2026-10-15) : ligne `allowed_clients` 5880/Win/x86 ajoutée, `realmbuilds = '5875 5880'`, `StrictVersionCheck = 0` (pas d'empreinte d'intégrité connue pour 5880). Fait dans le launcher (`ensure_realm`) et dans `Run-World.ps1`.
  **Mais mangosd a aussi sa propre liste** : `{5875, 6005, 6141, 0}` (comparaison exacte à `CMSG_AUTH_SESSION`, réponse `AUTH_VERSION_MISMATCH`), trouvée dans le binaire à l'offset 6 193 056, et il réécrit `realmlist.realmbuilds` avec cette liste à chaque démarrage. Correctif : `Build/Patch-MangosdBuilds.ps1` remplace 6141 (1.12.3, inutile) par 5880 dans `mangosd.exe` (original conservé en `.orig`, modification déclarée dans `LICENCES/SOURCES.md`). À terme : recompiler VMaNGOS avec ce changement dans le source (le fichier qui définit la liste n'a pas été localisé par lecture à distance ; à faire lors de la mise en place de la compilation du cœur).
- AHBot : `sAuctionHouseBotMgr.Load()` au démarrage, `Update()` sur le timer `WUPDATE_AUCTIONS` (60 s) ; remplit jusqu'à `AHBot.itemcount` enchères par tirage uniforme avec remise dans `auctionhousebot` ; vendeur 0 ; un seul hôtel (`AHBot.ah.fid`).

## Exigences ajoutées en cours de route

- **AHBot** (demande utilisateur du 2026-10-14) : l'hôtel des ventes doit être approvisionné avec tous les objets utiles en solo. VMaNGOS fournit un AHBot minimal (`AHBot.Enable`, `AHBot.ah.fid` une seule faction, `AHBot.itemcount`, table `auctionhousebot` : 78 objets dans le snapshot). **Fait le 2026-10-14** : `Tools/New-AHBotTable.ps1 -Apply` génère `Server/Database/custom/auctionhousebot_solo.sql` (6 392 objets : consommables, réactifs, composants, recettes, sacs, munitions, familiers/montures, armes et armures vertes/bleues/épiques non liées au ramassage ; prix = prix vendeur ou 4 × prix de revente, piles jusqu'à 20, enchère = 75 % du rachat). `New-ServerConfig.ps1` active `AHBot.Enable = 1`, choisit l'hôtel de la faction du dernier personnage joué (Alliance par défaut) et règle `AHBot.itemcount = 3 × lignes` (≈ 19 000 enchères). Limites : un seul hôtel à la fois (changement au redémarrage), doublons dus au tirage aléatoire, prix non économiques ; amélioration possible par un correctif du cœur (listing déterministe, deux factions).

## Autonomie vis-à-vis des runtimes Visual C++ (0.9.17, 2026-10-20)

Les programmes VMaNGOS demandent MSVCP140 / VCRUNTIME140 (Visual C++ 2015-2022) et leur `libmySQL.dll` (MySQL 5.5.62)
le runtime Visual C++ 2008 (`Microsoft.VC90.CRT`), absents d'un Windows neuf. Après toute (ré)installation du composant
VMaNGOS ou de MariaDB, et après `Build/Patch-MangosdBuilds.ps1`, lancer **`Build/Set-VmangosRuntime.ps1`** :

- copie dans `Server\vmangos` le runtime Visual C++ 14.44 livré par MariaDB (même version que l'éditeur de liens de VMaNGOS) ;
- compile `Build/libmysql-shim` (Build Tools 2022) et remplace `libmySQL.dll` par cette couche (213 fonctions redirigées
  vers `libmariadb.dll`, MariaDB Connector/C 3.4, binairement compatible pour ce qu'utilise VMaNGOS : `MYSQL_FIELD`,
  `MYSQL_BIND` et les options 7 et 9 de `mysql_options`, structures comparées aux en-têtes ; seules `mysql_thread_init`,
  `mysql_thread_end` et `mysql_thread_safe` sont écrites dans la couche) ; l'original reste en `libmySQL.dll.orig`.

Le launcher est lié à la CRT statique (`Launcher/.cargo/config.toml`). Contrôle : `Tests/Test-Dependencies.ps1` (chaque
DLL importée par un programme du package doit être une DLL de Windows ou se trouver à côté de lui).

## VMaNGOS compilé par HomeCraft (0.9.18, 2026-10-23)

`mangosd.exe` et `realmd.exe` ne sont plus les binaires officiels : **`Build/Build-Vmangos.ps1`** clone `vmangos/core`
dans `Build/vmangos-src` (ignoré par git), se place sur le commit figé `53d24d5`, applique
**`Build/vmangos-patches/homecraft.patch`** et compile en Release x64 (CMake + MSBuild des Build Tools 2022, environ
1 min 30 en recompilation, quelques minutes la première fois ; aucune dépendance hors du dépôt VMaNGOS). Les binaires
officiels restent en `.orig` (exclus du package). `Build/Patch-MangosdBuilds.ps1` (patch binaire 6141 → 5880) est
remplacé par la modification source de `cmake/generators/Progression.h.in` (`{5875, 5880, 6005, 6141, 0}` ; `src/shared/Progression.h` en est généré par CMake, le modifier directement ne sert à rien).

Contenu du patch :
- **bots du monde** (`PartyBotAI` avec un constructeur « monde », `PlayerBotAI.cpp` : les trois IA de peuplement
  `*PopulationAI` y sont rattachées) : hors groupe, ville (`HomeCraftBot.CityPercent`, 25 %) ou chasse dans une zone de
  leur niveau (positions d'apparition de monstres ordinaires hostiles aux joueurs, continents 0 et 1, jamais en zone
  de la faction adverse, `AreaEntry::Team` / `AreaLevel`) ; invitation d'un joueur réel acceptée (`SMSG_GROUP_INVITE`
  → `CMSG_GROUP_ACCEPT`), mise au niveau du joueur (`HomeCraftBot.LevelSync`), puis comportement de compagnon ;
  retour à la vie libre quand le joueur part ; `DeleteRandomBot` ne déconnecte jamais un bot groupé avec un joueur ;
- protections contre l'absence de groupe dans `PartyBotAI` et `CombatBotBaseAI::AreOthersOnSameTarget` (ces fonctions
  supposaient toujours un groupe).

Modifier VMaNGOS : travailler dans `Build/vmangos-src`, puis `git -C Build/vmangos-src diff > Build/vmangos-patches/homecraft.patch`
avant de relancer `Build-Vmangos.ps1` (qui repart du commit propre et réapplique le patch). Le patch est livré dans
`Sources\Build` du package (GPL-2.0).
