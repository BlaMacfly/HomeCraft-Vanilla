# WoW Vanilla Solo Edition — instructions de session

Projet distinct des serveurs Sylvania (voir `../CLAUDE.md` pour ceux-là). Répondre en français.

## Lire d'abord
1. `Documentation/01-Audit-Phase1.md` — audit, architecture retenue (VMaNGOS + PartyBots + client frFR + launcher Qt), risques, plan.
2. `Documentation/02-Construction.md` — composants figés, disposition des dossiers, ports, scripts, résultats vérifiés.
3. `Documentation/03-Tests-Phase2.md` — journal de tests et procédure de test en jeu.
4. `Documentation/04-Launcher-Exigences.md` — exigences du launcher (Rust, comptes, explorateur de base + HeidiSQL, identité visuelle, signature).
5. `Documentation/05-Tests-Phases4-5.md` — tests du launcher (démarrage/arrêt, sauvegarde, restauration, réparation).
6. `Documentation/06-Packaging-Portable.md`, `07-Perennite-Client.md`, `08-Patchs-Graphiques.md`, `09-Localisation-Inventaire.md`, `10-Validation-Finale.md`, `11-Licence-PlayerBots.md` (édition CMaNGOS).

## Tests (Tests/)
Branche `cmangos` (édition CMaNGOS Classic + module playerbots, version 0.10.x ; l'édition VMaNGOS 0.9.18 reste sur `main`) : tests validés `Test-Dependencies` (aucune dépendance à un runtime Visual C++ installé ; à lancer sur chaque package) et `Test-Integration` (copie du package dans %TEMP%\hc-integration, données CMaNGOS prises dans `-DataDir`, par défaut %TEMP%\hc-cm\Server\Data : premier démarrage et création des bots, comptes, suppression hors ligne, vie des bots, réglages à chaud, suppression des bots serveur en marche, processus résiduels ; à lancer après chaque modification du launcher, package construit). `Test-Launcher`, `Test-Phase5`, `Test-Package`, `Test-Offline`, `Test-Localisation`, `Test-Installer` travaillent sur le dossier de développement (édition VMaNGOS) : à adapter avant usage sur cette branche.
**Dossier de développement = édition VMaNGOS en usage par l'utilisateur** (UserData, Server\Data, launcher à la racine) : ne jamais y lancer le launcher CMaNGOS. Tests manuels dans un dossier isolé (%TEMP%\hc-cm : Server\cmangos, Server\Database\{mariadb,cmangos-sql}, Server\Data au format CMaNGOS, `VANILLA_SOLO_ROOT`), arrêt par `--quit-after`.
Distribution : **portable uniquement** (décision utilisateur 2026-10-16 ; l'installateur Inno Setup a été construit, testé puis abandonné parce qu'il aurait dû contenir le client : scripts `Build/New-Installer.ps1`, `Build/HomeCraft.iss` et `Tools/InnoSetup` conservés inactifs, ne pas les relancer sans demande explicite). Le client WoW n'est jamais inclus dans un livrable, quel qu'il soit. Icône de l'exécutable : `Launcher/assets/homecraft.ico` via `Launcher/build.rs` (winresource, rc.exe du SDK Windows). Le launcher accepte `--start`, `--quit-after N`, `--tab <onglet>`, `--do backup|verify|repair|diagnostic|restore <zip>|botdel|patchon <nom>|patchoff <nom>|addonadd <zip>|addonon|addonoff|addondel <nom>|accdel <compte>|acccreate <compte> <mdp> [gm]|mpqbuild <dossier>|reimportworld|mmaps|console "<commande>"`, `--install-to <dossier>`.
Serveur : `Build/Build-Cmangos.ps1` (mangos-classic + playerbots aux commits du manifeste, Boost 1.87, patchs `Build/cmangos-patches/*.patch`, runtime VC copié ; installe dans `Server/cmangos`) ; bases : `Build/Build-CmangosDb.ps1` (instance MariaDB temporaire 3316, classic-db, traductions loc2 reprises de la base VMaNGOS, export `Server/Database/cmangos-sql`). Modifier les sources dans `Build/cmangos-src` puis réexporter le patch (`git -C … diff`), jamais les binaires. Licence du module playerbots : `Documentation/11-Licence-PlayerBots.md` (pas de diffusion sans accord des auteurs).
Après chaque build : `Build/Sign-Launcher.ps1 -Thumbprint <UserData/code-signing-thumbprint.txt> -NoDeploy` (sur cette branche : ne pas remplacer le launcher VMaNGOS de la racine), puis `Build/New-Package.ps1`.
**Ne jamais** lancer un test qui démarre ou arrête des services pendant que le launcher de l'utilisateur tourne (processus « HomeCraft Edition Vanilla »).

## Règles du dépôt
- `Client/` contient le client WoW 1.12.1 frFR de l'utilisateur et `Server/Data/` les données qui en dérivent : **jamais redistribués, jamais dans une archive**.
- `UserData/` contient la base MariaDB (personnages) et les identifiants : ne jamais supprimer ni ré-importer sans sauvegarde et accord explicite (le launcher refuse de recréer `characters` si elle contient des personnages).
- Composants externes : uniquement via `Build/manifest.json` (URL, commit, SHA-256 figés). Mettre à jour = changer le manifeste, re-vérifier, re-tester.
- Scripts PowerShell : encodés **UTF-8 avec BOM** (PowerShell 5.1), jamais `2>$null` sur un exécutable natif (passer par `cmd /c … >nul 2>&1`), pas de chemins absolus codés en dur (tout passe par `Tools/Common.ps1`).
- Les `.conf` de `Server/Config/` (mangosd, realmd, aiplayerbot, ahbot) sont générés par le launcher (`server::generate_configs`) depuis les modèles `.conf.dist` de `Server/cmangos` : modifier le code ou le catalogue (`conf_catalog.rs`), pas les fichiers. mangosd tourne avec `Server/cmangos` comme dossier courant (module OpenSSL legacy.dll).
- Serveur de développement : le launcher lui-même (`--start`, `--do console "<commande>"`, `--quit-after N`). Vérifier qu'aucun personnage n'est en ligne avant un redémarrage.
- Ports : MariaDB 3315, realmd 3724, mangosd 8085, tous sur 127.0.0.1 uniquement.
- Launcher : `Launcher/` en **Rust** (décision utilisateur), egui + crate `mysql` pur Rust ; voir `Launcher/README.md`. Compiler avec `cargo build --release` (rustup + Build Tools 2022 requis ; `~/.cargo/bin` à ajouter au PATH de la session).
