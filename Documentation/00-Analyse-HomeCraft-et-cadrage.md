# HomeCraft Edition WoTLK — analyse de l'édition Vanilla et cadrage

Date : 2026-10-15. Poste : Windows 11 Famille (« Marty »), 716 Go libres sur C:.
Statut : **cadrage, rien de construit**. Décisions utilisateur prises le 2026-10-15 : cœur = repack du royaume
« The Kingdom of Sylvania » (fork AzerothCore master, dépôt `BlaMacfly/SylvaniaCore-WoTLK`), **playerbots obligatoires**.

---

## 1. Ce que contient HomeCraft Edition Vanilla (`../HomeCraft`)

Édition solo de WoW 1.12.1 livrée en v0.9.0 (validation finale du 2026-10-15, `10-Validation-Finale.md`).

| Couche | Contenu | Réutilisable pour WoTLK |
|---|---|---|
| Serveur | VMaNGOS (binaires officiels Windows x64, commit figé), dumps MySQL, MariaDB 11.8.9 portable sur 127.0.0.1:3315 | MariaDB portable et toute la mécanique « datadir + import + santé » : **oui**. VMaNGOS : **non** (remplacé par AzerothCore) |
| Données client | Extraction dbc/maps/vmaps/mmaps à la première installation, jamais redistribuées | Même principe, extracteurs AzerothCore (`map_extractor`, `vmap4extractor`, `vmap4assembler`, `mmaps_generator`) |
| Launcher | Rust, egui, un seul exécutable (5 399 lignes dans `Launcher/src/`), machine à états NON INSTALLÉ → PRÊT → JEU EN COURS, console du serveur sur stdin, comptes, PlayerBots, Monde (70 clés de conf), Patchs MPQ, explorateur de base natif + HeidiSQL, AddOns, sauvegardes/restauration, vérification par manifeste, réparation, diagnostic, mmaps en arrière-plan, mode `--do …` pour les tests | **Ossature entière réutilisable**. À réécrire : `server.rs` (authserver/worldserver, clés de conf), `conf_catalog.rs` (clés `worldserver.conf` + `playerbots.conf`), `botgen.rs` (sans objet : les random bots du module gèrent leurs personnages), `client.rs` (build 12340, MPQ 3.3.5a, `realmlist.wtf` dans `Data/frFR/`), SRP6 (format AC `salt`/`verifier`), AHBot (natif dans AC, piloté par conf et non par table) |
| Scripts `Tools/` | PowerShell UTF-8 BOM, aucun chemin absolu, `Common.ps1` central | Harnais de développement à reprendre tel quel, clés et noms de binaires à changer |
| Tests `Tests/` | Démarrage/arrêt, Phase 5, package, localisation, hors ligne | Même structure |
| Packaging | `Build/manifest.json` (URL, commit, SHA-256 figés), `New-Package.ps1`, ISO IMAPI2, signature Authenticode auto-signée, licence propriétaire HomeCraft | Tel quel ; manifeste à refaire. Le package WoTLK ne tiendra pas sur un CD (voir §6) |
| Localisation | Client frFR + compléments SQL `custom/` qui ne remplissent que les champs vides | Même méthode sur les tables `*_locale` d'AzerothCore (la base Sylvania est déjà largement en français) |
| Règles de session | `CLAUDE.md` : client jamais redistribué, `UserData/` jamais effacé, composants uniquement par manifeste, pas de `2>$null` sur un exe natif, conf générées et non éditées | Reprises dans `../CLAUDE.md` de ce dossier |

Identité visuelle fournie pour cette édition : logo `HomeCraft WoTLK.png` (1254×1254, H de glace sur fond noir) et fond
`HomeCraft Taverne WoTLK.png` (1680×944, taverne du Fléau), copiés dans `Launcher/assets/` (voir `SOURCES.md`).
Le logo n'a **pas** de fond transparent (fond noir plein) : à détourer avant usage en icône de fenêtre.

---

## 2. Le client 3.4.3.54261 (WotLK Classic) : décision utilisateur, utilisable via une couche de traduction

Le dossier `Documents\Téléchargements MEGA\World of Warcraft (3.4.3.54261)` est un client **WotLK Classic** :
`.build.info` → `Version 3.4.3.54261`, `Product wow_classic` ; `_classic_\WowClassic.exe` 64 bits ; données en
**CASC** (`Data\data` 10,3 Go sur 14,96 Go), pas de MPQ ; `Arctium WoW Launcher.exe` à la racine, raccourci
`Classic.lnk` → `Arctium WoW Launcher.exe --version=Classic` ; `Config.wtf` avec `SET portal "127.0.0.1"`.
**Décision utilisateur (2026-10-15) : garder ce client**, déjà testé par lui sur son autre serveur WotLK.

AzerothCore (master, fork Sylvania, branche Playerbot) ne parle nativement qu'au client 3.3.5a (12340). Un client
3.4.3 se connecte donc **à travers une couche de traduction de protocole**. Deux solutions existent (vérifiées le
2026-10-15) :

| | **HermesProxy** (`Xian55/HermesProxy`) | **azerothcore-343** (`neeme22/azerothcore-343`) |
|---|---|---|
| Principe | Proxy externe en C# : bnetserver moderne + client d'authentification 3.3.5 + worldserver moderne + client monde 3.3.5. **Aucune modification du cœur** | Fork d'AzerothCore **branche Playerbot** (commit `413bea61a` du 2026-09-04) avec un `bnetserver` porté de TrinityCore 3.4.3 et une passerelle `worldgate` (ports 8086/8087) qui traduit 3.4.3 ↔ 3.3.5 et pousse objets/sorts/textes en hotfixes ; arbre TrinityCore 3.4.3 embarqué dans `gateway/` |
| Clients 3.4.3 | `V3_4_3_54261` **seul build supporté**, statut bêta | 3.4.3.54261 exactement |
| Cœur 3.3.5 | TrinityCore (principal), **AzerothCore « broadly working »** (faiblesse connue : cadavres, issue #190), CMaNGOS partiel | AzerothCore Playerbot uniquement |
| Maturité | Fork actif de `WowLegacyCore/HermesProxy` : 72 étoiles, 49 issues ouvertes, releases quasi quotidiennes (v4.5.67 → v4.5.70 du 12 au 14 oct., prérelease v4.6.0 du 15 oct. « 3.4.3 regression-checked against AzerothCore »), **binaires Windows/Linux/macOS publiés** avec sommes SHA-256 | Dépôt créé le **2026-09-25**, 5 commits le même jour, 6 étoiles, 0 issue, un seul auteur ; « there **will** be bugs » |
| Licence | GPL-3.0 | GPL-2.0 |
| Matrice WotLK sur AzerothCore (`wotlk.md`) | ✅ connexion, personnages, combat, sorts, auras, inventaire, vendeurs, butin, HV, courrier, échanges, chat, groupe/raid, talents, glyphes, quêtes, hauts faits, BG, arènes, Joug-d'hiver, LFG, transports, coiffeur, pêche… ⚠️ non revérifiés après correctifs : ready check, icônes de raid, réputations, envoi de courrier, verrous d'instance, fenêtre de ticket GM. ❓ écuries, affectation d'équipe d'arène, déconnexion sporadique sous charge. ❌ **Calendrier** (aucun cœur) | Pas de liste publiée |
| Warden | Doit être **désactivé** côté cœur (le proxy jette les paquets Warden) | `Warden.Enabled = 0` obligatoire |
| Client | `SET portal "127.0.0.1"` + Arctium `--version=Classic --staticseed` ; garder « Optimiser le réseau pour la vitesse » | Idem Arctium ; `mapextractor -e 2` sur le client pour les DB2 |
| Processus à piloter par le launcher | MariaDB, authserver, worldserver, **HermesProxy**, Arctium → client | MariaDB, worldserver, bnetserver, worldgate, Arctium → client |
| Compatibilité avec « repack du fork Sylvania » | **Totale** : le cœur reste le fork Sylvania + patch Playerbot (§4) | **Faible** : il faudrait soit reporter Sylvania sur ce fork (stratégie B), soit extraire bnetserver + worldgate + l'arbre TC 3.4.3 et les greffer sur le fork Sylvania |

**Décision utilisateur (2026-10-16) : HermesProxy + Arctium**, déjà utilisés avec succès sur son autre serveur
WotLK. Le proxy n'impose rien au cœur, se met à jour indépendamment (zip Windows figé par SHA-256 dans le
manifeste, comme VMaNGOS en Vanilla) et son statut AzerothCore est documenté. `azerothcore-343` reste une
référence utile (sa passerelle montre quelles données 3.4.3 doivent venir en hotfixes) et un repli si le proxy
s'avère insuffisant. À relever auprès de l'utilisateur : la version de HermesProxy et le `appsettings.json`
utilisés sur l'autre serveur, pour repartir d'une configuration connue.

**Conséquences à assumer avec ce client :**

- **Langue** : les tags de `.build.info` ne listent que `enUS text/speech` et `ruRU text/speech`, et `Config.wtf`
  est en `textLocale enUS`. **Ce client n'a pas de données françaises** : interface, sorts, objets et textes du jeu
  seront en anglais (les `.pak` de `_classic_\Utils\LOCALES` ne concernent que le navigateur intégré). Les tables
  `*_locale` frFR du serveur passent par les hotfixes/paquets traduits, mais tout ce qui vient des DB2 du client
  reste en anglais. Obtenir des données frFR 3.4.3 signifie un autre client (CDN Blizzard d'un build retiré :
  non garanti). L'objectif « français intégral » de l'édition Vanilla n'est donc **pas atteignable** tel quel.
- **Arctium WoW Launcher** : exécutable tiers fermé, indispensable pour rediriger le client ; à figer (SHA-256 de
  la copie fournie, 5,5 Mo) et à documenter comme composant non libre.
- **Deux chaînes de données** : le serveur a toujours besoin des dbc/maps/vmaps/mmaps **3.3.5a** (extraits d'un client
  12340 : `wotlk moderna\ref-client`, 16,07 Go frFR, est disponible pour ça), le proxy/la passerelle des **DB2
  3.4.3** du client Classic. Le launcher devra gérer les deux extractions.
- **Trois processus de plus** à superviser (proxy, Arctium, client 64 bits), un port Battle.net (1119) et REST
  (8081) en plus de 3724/8085, le tout sur 127.0.0.1.
- Addons : ceux du client 3.4.3 (`_classic_\Interface\AddOns`, Interface 30403), pas ceux de 3.3.5a.

Comme en Vanilla, le client n'est jamais inclus dans le package : l'utilisateur fournit le sien.

---

## 3. Le cœur : fork Sylvania d'AzerothCore master

- Le dépôt `github.com/BlaMacfly/SylvaniaCore-WoTLK` répond **404 sans authentification** et n'apparaît pas dans la
  liste publique des dépôts de BlaMacfly (qui contient `SylvaniaCore` Legion 7.3.5, `SylvaniaLauncher`,
  `mod-gunship-skip`, `mod-pet-loot`, `WotlkBot`, `Unbot-FR`). Il est donc privé ou pas encore poussé :
  **il faut un accès** (dépôt rendu public, jeton, ou archive des sources) avant toute compilation.
- Ce que la mémoire de la session Linux en dit (`claude-memory/fork-azerothcore-sylvania.md`,
  `pvp-bg-arene-sylvania.md`) : clone **shallow** de `azerothcore-wotlk` master (2026-04-02), commits maison
  `[Sylvania] …`, cherry-picks sélectifs (des correctifs upstream manquent), **schéma `creature.id1/id2/id3`**
  alors que l'upstream est passé à `id` + `creature_multispawn`, `updates` plafonnant à `2026_03_31_00`,
  7 modules maison + modules publics (`mod-arac`, `mod-cfbg`, `mod-1v1-arena`, `mod-solo-lfg`, `mod-npc-morph`,
  `mod-level-one-mounts`, `mod-customlogin`, `mod-mythic-plus`, `mod-skip-dk-starting-area`, `mod-gunship-skip`,
  `mod-pet-loot` : liste à confirmer sur le dépôt), **aucun playerbot** (démontés le 2026-05-31).
- Deux bases monde : `acore_world` (royaume 1, auditée saine le 2026-08-15) et `acore_world_sylvania`
  (royaume 2, contenu « Kingdom of Sylvania », corruption partielle relevée). **À décider : laquelle est « le royaume »
  à repacker** (le nom choisi désigne le royaume 2).

---

## 4. Playerbots sur master : mesure de l'écart et stratégie

Le module `liyunfan1223/mod-playerbots` (GPL-2.0, poussé le 2026-10-12, 1 171 étoiles) exige le fork
`mod-playerbots/azerothcore-wotlk` branche `Playerbot` (« The standard AzerothCore repository will not work »).
Mesure faite le 2026-10-15 avec l'API de comparaison GitHub, `azerothcore:master...mod-playerbots:Playerbot`
(base commune du 2026-10-04, la branche est synchronisée sur master toutes les semaines environ) :

| Zone touchée par la branche Playerbot | Fichiers | + / − |
|---|---|---|
| `src/server/database/` (`ModuleDatabasePool` : pool de base dédié `acore_playerbots`, `MySQLConnection`, `SQLOperation`, `DatabaseWorkerPool`) | 5 | 414 / 87 |
| `src/server/game/` (`PathGenerator` : options de chemin pour les bots, `WorldSession.h`, `Unit.cpp` 1 ligne) | 4 | 44 / 5 |
| `src/tools/mmaps_generator/` + `src/common/Collision/Maps/MapDefines.h` (génération mmaps adaptée) | 5 | 52 / 13 |
| `modules/CMakeLists.txt`, `worldserver.conf.dist`, `.gitignore`, `docker-compose.yml` | 4 | 26 / 1 |
| `.github/`, `apps/ci`, `doc` | 15 | 218 / 43 |
| **Total** | **33** | **754 / 149** |

Autrement dit, « rendre les playerbots compatibles avec master » ne demande pas de réécrire le module : **le cœur
n'a besoin que d'un patch de ~900 lignes**, dont la moitié est de l'intégration continue sans intérêt pour nous.

**Patch produit le 2026-10-16** : `Build/playerbot-core.patch` (61 Ko, diff GitHub `master...Playerbot`, 33 fichiers,
+754/−149). `git apply --check --include='src/*' --include='modules/CMakeLists.txt'` **réussit sur le master du jour**
(`01339ccf`). Lecture du patch : `modules/CMakeLists.txt` définit `MOD_PLAYERBOTS` pour `database` et
`game-interface` quand le module est présent ; `worldserver.conf.dist` ajoute l'appender/logger `Playerbots` ;
`MapDefines.h` renomme `NAV_UNUSED1` en **`NAV_GROUND_STEEP`** (0x10), posé par le `mmaps_generator` patché et lu par
`PathGenerator` → **les mmaps doivent être régénérées avec cet outil** (celles du VPS ou d'un AzerothCore stock
n'ont pas ce drapeau) ; `Unit.cpp` et `WorldSession.h` ne sont que cosmétiques. Clones de référence dans
`Build/src/` : `azerothcore-master` (profondeur 50), `azerothcore-playerbot` (`2a2211cd`, 2026-10-09),
`mod-playerbots` (`79bd4281`, 2026-10-09 ; le dépôt `liyunfan1223` redirige vers l'organisation `mod-playerbots`).

**Stratégie A (recommandée) — patcher le fork Sylvania**

1. Produire le patch cœur : `git diff <base-commune> mod-playerbots/Playerbot -- src modules/CMakeLists.txt` (hors `.github`, `apps`, `doc`, `docker-compose.yml`).
2. L'appliquer sur `SylvaniaCore-WoTLK` avec `git apply -3` (le clone shallow interdit `cherry-pick`, pas `apply`).
   Conflits attendus là où le fork est en retard sur master : surtout `ModuleDatabasePool.cpp` (réécrit récemment
   upstream). Si le fork date d'avant cette réécriture, reporter d'abord les commits upstream de ce fichier.
3. Cloner `mod-playerbots` dans `modules/`, compiler, importer `sql/playerbots/base` + `updates` dans `acore_playerbots`.
4. Régénérer les **mmaps avec le `mmaps_generator` patché** (les bots en dépendent ; les mmaps du VPS ne sont pas
   réutilisables telles quelles si le format a changé : à vérifier sur `MapDefines.h`).
5. Vérifier les SQL du module qui touchent `characters`/`world` contre le schéma `id1/id2/id3` du fork.

**Stratégie B (repli) — partir de la branche Playerbot et y reporter Sylvania**

Reporter les commits `[Sylvania]` et les modules sur la branche Playerbot, puis convertir la base monde Sylvania au
schéma upstream (`id1` → `id`, `id2/id3` → `creature_multispawn`, rattrapage de `updates`). Plus propre à long terme
(fork aligné sur master, mises à jour futures faciles), mais coût initial bien plus élevé et dépendant de la liste
exacte des commits maison. À choisir si la stratégie A bute sur un fork trop ancien.

**Ce que le module apporte à l'édition solo** (README vérifié) : alts du compte jouables comme compagnons
(`.playerbots bot add`), bots aléatoires qui peuplent le monde, questent, font donjons/raids/BG, stratégies par
commandes de chat et addons communautaires d'interface. Réglages exposés par `playerbots.conf` (nombre de random
bots, niveaux, etc.) : ils remplaceront l'onglet PlayerBots/`botgen.rs` de l'édition Vanilla.

Pistes écartées : `WotlkBot` (C#, bots-clients externes : lourd, IA limitée), `Unbot` (patch cœur binaire fermé),
`mod-npcbots` (exige aussi un fork du cœur, bots non-joueurs).

---

## 5. Compilation : il n'y a pas de binaire AzerothCore prêt, il faut compiler sur ce poste

Contrairement à VMaNGOS, AzerothCore ne publie pas de binaires Windows ; le fork Sylvania encore moins.
État du poste le 2026-10-15 :

| Prérequis AzerothCore Windows (wiki officiel) | Présent |
|---|---|
| Visual Studio 2022 « Développement Desktop C++ » | **Oui** (`C:\BuildTools`, déjà utilisé pour le launcher Rust) |
| CMake ≥ 3.16 (MSVC) | **Non** (seul `C:\msys64\mingw64\bin\cmake.exe` existe : MinGW, non supporté par AzerothCore) |
| Boost ≥ 1.78 (binaires VS 2022) | **Non** |
| MySQL ≥ 8.0 (bibliothèques client pour la compilation) | **Non** |
| OpenSSL 3.x 64 bits (non « light ») | **Non** |
| Git | **Non** |
| Rust stable MSVC (launcher) | Oui, 1.99.0 |

**Installés le 2026-10-16** (autorisation utilisateur), versions calées sur l'intégration continue Windows
d'AzerothCore, détail et empreintes dans `Build/build-tools.json`, installeurs conservés dans `Build/downloads/`,
journaux dans `Logs/install-build-tools.log` et `Logs/download-build-tools.log` :

| Outil | Version | Emplacement | Vérification |
|---|---|---|---|
| Git | 2.55.0.5 | `C:\Program Files\Git` | winget (empreinte de l'installeur vérifiée) |
| CMake | 4.4.4 | zip portable winget (`%LOCALAPPDATA%\Microsoft\WinGet\Packages\Kitware.CMake_…\cmake-4.4.4-windows-x86_64\bin`) | winget |
| Boost | 1.87.0 (msvc-14.3, 64 bits) | `C:\local\boost_1_87_0` (`BOOST_ROOT` utilisateur) | installeur SourceForge non signé, SHA-256 relevé |
| OpenSSL | 3.5.9 LTS, complet | `C:\Program Files\OpenSSL-Win64` (`OPENSSL_ROOT_DIR` utilisateur) | SHA-256 conforme à `slproweb/opensslhashes` |
| MySQL (bibliothèques client) | 8.4.9, zip | `C:\tools\mysql\current` | MD5 conforme à la page d'archives MySQL |

Validation : configuration CMake à blanc d'`azerothcore-wotlk` master (`01339ccf`, 2026-10-15) avec le générateur
Visual Studio 17 2022 x64, `-DTOOLS_BUILD=all -DSCRIPTS=static -DMODULES=static` → Boost 1.87.0, OpenSSL 3.5.9 et
MySQL trouvés, « Generating done » (`Logs/cmake-configure-master.log`). Remarques : CMake 4.4 avertit sur la
politique CMP0167 (FindBoost retiré) mais trouve Boost par ses fichiers de configuration ; Git doit être ajouté au
PATH de la session (`C:\Program Files\Git\cmd`) ; MySQL est passé explicitement par `-DMYSQL_INCLUDE_DIR` /
`-DMYSQL_LIBRARY` puisque hors des dossiers standard.

Ces outils ne servent qu'au poste de build ; le package livré reste sans dépendance (DLL MySQL/OpenSSL copiées).
Le serveur de jeu utilisera **MariaDB portable** comme en Vanilla, sous réserve de vérifier la version minimale
acceptée par le fork (AzerothCore exige MySQL 8 ou MariaDB récente ; à tester avec les dumps Sylvania).

---

## 6. Différences de volume et de ressources à anticiper

| Élément | Vanilla (mesuré) | WoTLK (estimé) |
|---|---|---|
| Binaires serveur | 6 Mo | ~60 Mo (worldserver + authserver + DLL) |
| Dumps de bases | 30 Mo | ~300 Mo compressés (`acore_world` ≈ 1 Go à plat) + `acore_playerbots` |
| Données extraites (dbc/maps/vmaps/mmaps) | 1 à 3 Go | **6 à 9 Go** ; mmaps : plusieurs heures |
| Package | 450 Mo (CD) | 1 à 1,5 Go : **DVD ou clé USB**, plus de CD (`New-Iso.ps1` le signale déjà) |
| RAM worldserver | ~300 Mo | 2 à 3 Go avec quelques centaines de random bots (relevé VPS : 2,9 Go RSS) |

---

## 7. Disposition proposée (calquée sur l'édition Vanilla)

```
HomeCraft WoTLK/
  Build/            manifest.json, Get-Components, New-Package, New-Iso (DVD), Sign-Launcher, playerbot-core.patch
  Server/acore/     authserver.exe, worldserver.exe, DLL, extracteurs (compilés ici)
  Server/Config/    *.conf.dist (worldserver, authserver, modules dont playerbots) → .conf générés
  Server/Database/  mariadb/, sql/ (auth, characters, world Sylvania, playerbots), custom/ (compléments HomeCraft)
  Server/Data/      dbc, maps, vmaps, mmaps, Cameras (extraits du client, non redistribués)
  Launcher/         fork du launcher Rust (HomeCraft Edition WoTLK), assets/ (logo, fond)
  Tools/, Tests/, Documentation/, UserData/, Logs/, Client/ (client 3.3.5a frFR de l'utilisateur, exclu)
```

Ports : MariaDB 3315, authserver 3724, worldserver 8085, HermesProxy 1119 (Battle.net) / 8081 (REST) / 8084
(realm) / 8086 (instance), SOAP désactivé (console stdin comme en Vanilla), tout sur 127.0.0.1.
Bases : `acore_auth`, `acore_characters`, `acore_world`, `acore_playerbots`.
Composants supplémentaires dans `Tools/` : `HermesProxy/` (zip Windows figé) et `Arctium/` (copie de l'utilisateur,
non libre, figée par empreinte).

---

## 8. Ce qu'il faut obtenir avant la Phase 2

État au 2026-10-16 01:00 : outils installés et validés (§5), **dépôt cloné et analysé, patch Playerbot d'avril
2026 appliqué, configuration CMake réussie, compilation lancée** → voir `02-Coeur-Playerbots.md` (qui remplace
l'analyse « stratégie A/B » du §4 : le patch du jour ne s'applique pas, la voie retenue est « Playerbot figé à la
date réelle du fork, 2026-03-31 / 2026-04-03 »). **Restent bloquants** :

1. **Dumps des bases** : ce sont les releases GitHub `royaume1-db-2026-10-15` et `royaume2-db-2026-10-15` du
   dépôt, qui est resté **privé** (Git a demandé une authentification ; le clone est passé par Git Credential
   Manager dans le terminal de l'utilisateur). À télécharger par l'utilisateur, ou dépôt rendu public.
2. Choix du royaume repacké (1 ou 2).
3. Client : **3.4.3.54261 + HermesProxy + Arctium confirmés** (2026-10-16). Récupérer la version de HermesProxy
   et le `appsettings.json` de l'autre serveur. Jeu en anglais accepté de fait (§2).
4. Décision stratégie A ou B (§4) après lecture du dépôt.
5. Installation des outils de compilation (§5) : CMake, Boost, MySQL, OpenSSL, Git.
6. Un client 3.3.5a (12340) reste nécessaire au poste de build pour les dbc/maps/vmaps/mmaps du serveur
   (`wotlk moderna\ref-client`) ; l'utilisateur final devra lui aussi en fournir un, ou recevoir ces données
   autrement (non redistribuables : même règle qu'en Vanilla).

## 9. Plan de phases

| Phase | Contenu | Validation |
|---|---|---|
| 1 | Cadrage (ce document), accès dépôt + dumps, outils de build installés | Liste §8 cochée |
| 2 | Compilation du fork + patch Playerbot + `mod-playerbots` ; MariaDB portable ; import des bases ; extraction 3.3.5a (serveur) ; HermesProxy figé + Arctium ; premier login du client 3.4.3, `.playerbots bot add` en donjon ; relevé de ce qui casse à travers le proxy (cadavres, courrier, instances) | Compte-rendu de test |
| 3 | Inventaire localisation (`*_locale`, `broadcast_text_locale`, `quest_template_locale`) et compléments | Tableau de couverture |
| 4 | Launcher : fork du code Vanilla, identité WoTLK, `server.rs`/`client.rs`/`conf_catalog.rs`/SRP6 réécrits, onglet PlayerBots sur `playerbots.conf` | `Test-Launcher` 10/10 |
| 5 | Sauvegardes, réparation, mmaps en arrière-plan (générateur patché) | `Test-Phase5` |
| 6 | Packaging DVD/USB, signature, LISEZMOI, licences (GPL-2 AzerothCore + module, licence HomeCraft) | `Test-Package` |
| 7 | Validation finale | Rapport |

## 10. Sources consultées le 2026-10-15

- `../HomeCraft/` : `CLAUDE.md`, `README.md`, `Documentation/01, 02, 04, 06, 10`, `Launcher/README.md`, `Launcher/src/{paths,client,server}.rs`, `Build/manifest.json`, `Launcher/Cargo.toml`
- `../claude-memory/` : `MEMORY.md`, `fork-azerothcore-sylvania.md`, `pvp-bg-arene-sylvania.md`, `exploitation-royaumes-vps1.md`
- Client 3.4.3 : `.build.info`, `_classic_/.flavor.info`, `_classic_/WTF/Config.wtf`, `extractor.bat`
- https://api.github.com/users/BlaMacfly/repos ; https://github.com/BlaMacfly/SylvaniaCore-WoTLK (404)
- https://github.com/liyunfan1223/mod-playerbots (README, API) ; https://api.github.com/repos/mod-playerbots/azerothcore-wotlk ; API compare `azerothcore:master...mod-playerbots:Playerbot`
- https://api.github.com/repos/TrinityCore/TrinityCore (branches, commits `wotlk_classic`, `sql/base/auth_database.sql`) ; https://talk.trinitycore.org/t/cata-master-and-wotlk-classic-not-supported/33317
- https://www.azerothcore.org/wiki/windows-requirements
- https://github.com/Xian55/HermesProxy (API, releases), https://github.com/blinkysc/HermesProxy (README, `wotlk.md`, `docs/known-limitations.md`)
- https://github.com/neeme22/azerothcore-343 (README, API, commits, `gateway/`, `src/server/apps`)
- https://forum.ragezone.com/threads/wow-wotlk-3-4-3-classic-working-client-for-any-server-with-new-hermesproxy.1263743/ (guide communautaire HermesProxy + 3.4.3, non lu en détail : accès refusé)
