# WoW Vanilla Solo Edition — instructions de session

Projet distinct des serveurs Sylvania (voir `../CLAUDE.md` pour ceux-là). Répondre en français.

## Lire d'abord
1. `Documentation/01-Audit-Phase1.md` — audit, architecture retenue (VMaNGOS + PartyBots + client frFR + launcher Qt), risques, plan.
2. `Documentation/02-Construction.md` — composants figés, disposition des dossiers, ports, scripts, résultats vérifiés.
3. `Documentation/03-Tests-Phase2.md` — journal de tests et procédure de test en jeu.
4. `Documentation/04-Launcher-Exigences.md` — exigences du launcher (Rust, comptes, explorateur de base + HeidiSQL, identité visuelle, signature).
5. `Documentation/05-Tests-Phases4-5.md` — tests du launcher (démarrage/arrêt, sauvegarde, restauration, réparation).
6. `Documentation/06-Packaging-Portable.md`, `07-Perennite-Client.md`, `08-Patchs-Graphiques.md`, `09-Localisation-Inventaire.md`, `10-Validation-Finale.md`.

## Tests (Tests/)
`Test-ServerStartup`, `Test-Launcher`, `Test-Phase5`, `Test-Package` (installation vierge + ISO, dossier C:\HomeCraftTest), `Test-Installer` (installateur Inno Setup : installation/désinstallation silencieuses dans C:\HomeCraftTest-Setup), `Test-Localisation` (inventaire frFR), `Test-Offline` (écoute loopback seulement), `Test-Dependencies` (aucune dépendance à un runtime Visual C++ installé ; à lancer sur chaque package), `Test-Integration` (copie du package dans %TEMP%\hc-integration : comptes, suppression hors ligne, bots serveur en marche, processus résiduels ; à lancer après chaque modification du launcher, package construit).
Distribution : **portable uniquement** (décision utilisateur 2026-10-16 ; l'installateur Inno Setup a été construit, testé puis abandonné parce qu'il aurait dû contenir le client : scripts `Build/New-Installer.ps1`, `Build/HomeCraft.iss` et `Tools/InnoSetup` conservés inactifs, ne pas les relancer sans demande explicite). Le client WoW n'est jamais inclus dans un livrable, quel qu'il soit. Icône de l'exécutable : `Launcher/assets/homecraft.ico` via `Launcher/build.rs` (winresource, rc.exe du SDK Windows). Le launcher accepte `--start`, `--quit-after N`, `--tab <onglet>`, `--do backup|verify|repair|diagnostic|restore <zip>|botgen N|botdel|patchon <nom>|patchoff <nom>|addonadd <zip>|addonon|addonoff|addondel <nom>|accdel <compte>|acccreate <compte> <mdp> [gm]|mpqbuild <dossier>|reimportworld|mmaps`, `--install-to <dossier>`.
Serveur : `mangosd`/`realmd` compilés par `Build/Build-Vmangos.ps1` (VMaNGOS 53d24d5 + `Build/vmangos-patches/homecraft.patch`, voir 02-Construction) ; modifier le patch, jamais les binaires.
Après chaque build : `Build/Sign-Launcher.ps1 -Thumbprint <UserData/code-signing-thumbprint.txt>` (signe et déploie à la racine), puis `Build/New-FilesManifest.ps1`.
**Ne jamais** lancer un test qui démarre ou arrête des services pendant que le launcher de l'utilisateur tourne (processus « HomeCraft Edition Vanilla »).

## Règles du dépôt
- `Client/` contient le client WoW 1.12.1 frFR de l'utilisateur et `Server/Data/` les données qui en dérivent : **jamais redistribués, jamais dans une archive**.
- `UserData/` contient la base MariaDB (personnages) et les identifiants : ne jamais supprimer ni ré-importer sans sauvegarde et accord explicite (`Init-Database.ps1` refuse par défaut).
- Composants externes : uniquement via `Build/manifest.json` (URL, commit, SHA-256 figés). Mettre à jour = changer le manifeste, re-vérifier, re-tester.
- Scripts PowerShell : encodés **UTF-8 avec BOM** (PowerShell 5.1), jamais `2>$null` sur un exécutable natif (passer par `cmd /c … >nul 2>&1`), pas de chemins absolus codés en dur (tout passe par `Tools/Common.ps1`).
- Les `.conf` de `Server/Config/` sont générés : modifier `Tools/New-ServerConfig.ps1`, pas les fichiers.
- Serveur de développement : `Tools/Run-World.ps1` (arrière-plan), console via `Tools/Send-Console.ps1`, arrêt via le fichier `Logs/server/stop`. Vérifier qu'aucun personnage n'est en ligne avant un redémarrage.
- Ports : MariaDB 3315, realmd 3724, mangosd 8085, tous sur 127.0.0.1 uniquement.
- Launcher : `Launcher/` en **Rust** (décision utilisateur), egui + crate `mysql` pur Rust ; voir `Launcher/README.md`. Compiler avec `cargo build --release` (rustup + Build Tools 2022 requis ; `~/.cargo/bin` à ajouter au PATH de la session).
