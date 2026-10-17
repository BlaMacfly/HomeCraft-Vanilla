# HomeCraft Edition WoTLK — instructions de session

Édition solo de World of Warcraft 3.3.5a, déclinaison de `../HomeCraft` (édition Vanilla, VMaNGOS) sur le cœur
du royaume « The Kingdom of Sylvania » (fork AzerothCore master `BlaMacfly/SylvaniaCore-WoTLK`) avec `mod-playerbots`.
Projet distinct des serveurs Sylvania en production (voir `../CLAUDE.md`). Répondre en français.

## Lire d'abord
1. `Documentation/00-Analyse-HomeCraft-et-cadrage.md` — ce qui est repris de l'édition Vanilla, comment le client
   3.4.3 se connecte à AzerothCore (HermesProxy / azerothcore-343), prérequis, plan.
2. `Documentation/02-Coeur-Playerbots.md` — le dépôt (privé, base réelle upstream 2026-03-31, 23 modules, schéma
   `id1/id2/id3`), la voie playerbots retenue (patch `89b810b2f` + module `f76c2863` d'avril 2026 sur la branche
   `homecraft-playerbots` du clone `Build/src/sylvaniacore-wotlk`), la commande CMake et l'état de la compilation.
3. `Documentation/03-Phase2-Serveur-local.md` — MySQL (MariaDB rejetée), jonction `C:\hcw`, extraction du client,
   premier démarrage réussi le 2026-10-16 (20 playerbots), tables Sylvania créées à la main, ce qui reste bloqué.
4. `Documentation/05-Launcher-Rust.md` — launcher Rust compilé le 2026-10-16 (`HomeCraft Edition WoTLK.exe` à la
   racine) : tableau des changements par module, §6 onglet **Modules** (état lu dans `configs/modules/*.conf`, le
   générateur repart du `.conf` existant ; `--do modon/modoff/modreset <id>`), reste à faire (packaging, HermesProxy).
5. `Documentation/06-Tests-Launcher.md` — harnais `Tests/` (porté de Vanilla) et premier lancement réel du launcher
   le 2026-10-16 : les trois services démarrent et s'arrêtent proprement, sauvegarde/vérification/réparation/
   restauration/diagnostic validés ; pièges (port 8888 des playerbots, WM_CLOSE, espaces dans les arguments).
6. `Documentation/07-Keira3.md` — Keira3 (éditeur de la base monde) : **v3.10.5 compilée depuis la balise** dans
   `Tools/Keira3` (le binaire officiel 3.10.5 et master attendent `creature.id`, incompatibles avec le fork) ; bouton
   dans l'onglet Base de données ; AGPL : exécutable livré inchangé, jamais de code Keira3 dans le launcher.
7. `../HomeCraft/CLAUDE.md` et `../HomeCraft/Documentation/` — le modèle : mêmes règles, même disposition, même launcher.

## Décisions utilisateur (2026-10-15)
- Cœur : repack du royaume Kingdom of Sylvania (fork AzerothCore master). **Playerbots obligatoires**
  (`liyunfan1223/mod-playerbots`, patch cœur appliqué sur le fork : stratégie A du cadrage).
- Client : **3.4.3.54261 (WotLK Classic)**, lancé via Arctium, connecté au cœur 3.3.5a par une couche de
  traduction : **HermesProxy** (décision du 2026-10-16, éprouvé par l'utilisateur sur son autre serveur ;
  repli `neeme22/azerothcore-343`). Le client n'a que enUS/ruRU : jeu en anglais.
- **Client 3.3.5a open source (2026-10-16 soir)** : l'utilisateur propose le client décompilé **wowdecomp x64**
  (`../wotlk moderna/wowdecomp/bin/Win/WowClient64.exe`, frFR) « à la place de l'autre ». `Client/` en est un miroir
  (`Tools/New-ClientMirror.ps1`, MPQ en liens physiques) ; le launcher le reconnaît et le lance (JOUER validé jusqu'à
  l'écran de connexion, `06-Tests-Launcher.md` §4 ter). Pas de HermesProxy nécessaire avec ce client.
- Identité : logo `HomeCraft WoTLK.png`, fond `HomeCraft Taverne WoTLK.png` (`Launcher/assets/`).

## Poste de build (installé le 2026-10-16, détail dans `Build/build-tools.json`)
Git 2.55 (`C:\Program Files\Git\cmd`, à ajouter au PATH de la session), CMake 4.4.4 (zip winget sous
`%LOCALAPPDATA%\Microsoft\WinGet\Packages\Kitware.CMake_*\…\bin`), Boost 1.87.0 (`C:\local\boost_1_87_0`),
OpenSSL 3.5.9 (`C:\Program Files\OpenSSL-Win64`), MySQL 8.4.9 client (`C:\tools\mysql\current`), VS Build Tools 2022
(`C:\BuildTools`). Configuration CMake validée sur AzerothCore master (générateur « Visual Studio 17 2022 », x64).
Patch cœur Playerbot **retenu** : `Build/playerbot-core-89b810b2f.patch` (avril 2026, déjà appliqué sur la branche
`homecraft-playerbots`) ; `Build/playerbot-core.patch` (version actuelle, 33 fichiers) ne s'applique pas au fork.
Toujours produire un patch avec `git diff --output=` (jamais `Out-File` : BOM + CRLF).
Clones : `Build/src/` (sylvaniacore-wotlk = le cœur à compiler ; azerothcore-master, azerothcore-playerbot,
mod-playerbots = références, historiques complets) ; jamais dans le package.
Compilation du cœur : `Build/Build-Core.ps1` dans un terminal interactif ; il compile via la jonction **`C:\hc`**
(→ le clone) dans `C:\hc\b`, obligatoire à cause de la longueur des lignes de commande MSVC avec 24 modules.

## Règles (héritées de l'édition Vanilla)
- `Client/` (miroir du client 3.3.5a frFR de l'utilisateur, wowdecomp x64 ; MPQ = liens physiques, ne jamais les modifier en place) et `Server/Data/` (dbc/maps/vmaps/mmaps extraits) : **jamais redistribués, jamais archivés**.
- `UserData/` (base MariaDB, identifiants) : ne jamais supprimer ni ré-importer sans sauvegarde et accord explicite.
- Composants externes uniquement via `Build/manifest.json` (URL, commit, SHA-256 figés).
- Scripts PowerShell en **UTF-8 avec BOM**, jamais `2>$null` sur un exécutable natif, aucun chemin absolu (tout passe par `Tools/Common.ps1`).
- Les `.conf` de `Server/acore/configs/` (et `configs/modules/`) sont générés par `Tools/New-ServerConfig.ps1` :
  modifier le générateur, pas les fichiers. Sous Windows AzerothCore lit les conf de modules dans
  `<dossier courant>/configs/modules/`, et refuse `-c <chemin avec espace>` → les serveurs sont lancés via la
  jonction `C:\hcw` → racine du projet (`Tools/Run-World.ps1`).
- Base : **MySQL 8.4.9 portable** (`Server/Database/mysql`, datadir `UserData/Database`) — AzerothCore rejette MariaDB.
  Scripts : `Start-Database`, `Stop-Database`, `Init-Database` (dumps dans `Server/Database/sql`), `Import-ModuleSql`.
- Ports : MySQL **3306** (depuis le 2026-10-17, root / root : valeurs pré-remplies par Keira3 ; avant 3315), authserver 3724,
  worldserver 8085, tous sur 127.0.0.1 uniquement. HeidiSQL (`Tools/HeidiSQL`) et Keira3 (`Tools/Keira3`) se connectent en root.
- `Server/Data/dbc` vient du VPS (enUS 3.3.5a complet) : les DBC Map/AreaTable des clients frFR locaux sont 3.3.3a.
- Launcher en **Rust** (egui **0.32**, ne pas monter ; crate `mysql` pur Rust), fork de `../HomeCraft/Launcher`.
  Compilation : `cargo build --release` dans `Launcher/` (PATH : `~/.cargo/bin`, VS Build Tools), puis copie de
  `target/release/homecraft-wotlk-launcher.exe` vers `HomeCraft Edition WoTLK.exe` à la racine. Le launcher cherche
  sa racine en remontant depuis l'exe (dossier contenant `Server/` et `Tools/`) ; `HOMECRAFT_WOTLK_ROOT` la force.
- Voir `Documentation/03-Phase2-Serveur-local.md` pour l'état du serveur local et les tables Sylvania créées à la main.
- Ne jamais lancer un test qui démarre ou arrête des services pendant que le launcher de l'utilisateur tourne.
- Le fork Sylvania est un clone shallow avec un schéma `creature.id1/id2/id3` divergent de l'upstream : voir
  `../claude-memory/fork-azerothcore-sylvania.md` avant tout SQL ou patch upstream.
