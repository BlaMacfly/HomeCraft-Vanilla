# HomeCraft Edition BC — instructions de session

Projet frère de HomeCraft Edition Vanilla (`..\HomeCraft`, VMaNGOS 1.12.1) : même launcher Rust, même disposition, adapté à
The Burning Crusade 2.4.3 (client build 8606) sur CMaNGOS `mangos-tbc`. Répondre en français.

## Lire d'abord
1. `Documentation/Prompt-Session-BC.md` (copie du prompt initial, dans `..\HomeCraft\Documentation\`) — cahier des charges complet.
2. `Documentation/01-Audit-Phase1.md` — core retenu (build Windows quotidien officiel `with-all`), TBC-DB, playerbots, ports, identité, points non vérifiés.
3. Les documents suivants (`02-Construction.md`, …) au fur et à mesure des phases.

## Décisions de l'utilisateur
- Launcher **entièrement en Rust** (eframe/egui 0.32, ne pas monter en 0.36), un seul exécutable, portable uniquement, aucun installateur.
- Le client WoW n'est **jamais** inclus dans un livrable ; `Client/`, `Server/Data/`, `UserData/`, `Logs/` jamais archivés.
- Tout composant externe passe par `Build/manifest.json` + `Build/Get-Components.ps1 -Pin` ; **demander l'accord avant chaque téléchargement ou compilation longue**.
- Ports BC : MariaDB **3317**, authentification **3726**, monde **8087**, 127.0.0.1 seulement (Vanilla 3315/3724/8085, WotLK 3316/3725/8086).
- Royaume « HomeCraft BC », exécutable `HomeCraft Edition BC.exe`, MOTD « Bienvenue dans HomeCraft Edition BC ! ».

## Construction du launcher
- `Launcher/` : `cargo build --release` (PATH : `~/.cargo/bin`), exe `target/release/homecraft-bc-launcher.exe`.
- Après chaque build : `& Build\Sign-Launcher.ps1 -Thumbprint <empreinte>` (signe **et** déploie à la racine ; appeler avec `&`, pas `powershell -File`), puis `Build\New-FilesManifest.ps1`.
- Icône : `Launcher/assets/homecraft.ico` (générée par `Build/New-InstallerImages.ps1`), intégrée par `build.rs` (winresource, rc.exe).
- Captures : `Tools/Capture-Launcher.ps1 -Tab <onglet>` → `Documentation/images/launcher-<onglet>.png`.
- Ligne de commande de test : `--start`, `--quit-after N`, `--tab <onglet>`, `--do …`, `--install-to <dossier>` ; racine forcée par `HOMECRAFT_BC_ROOT`.

## Règles
- **Ne jamais** lancer un test qui démarre ou arrête des services pendant qu'un launcher de l'utilisateur tourne (Vanilla, WotLK ou BC).
- Scripts PowerShell en **UTF-8 avec BOM**, jamais `2>$null` sur un exécutable natif, `Set-Content -Encoding utf8`, pas de chemin absolu codé en dur (tout passe par `Tools/Common.ps1`).
- Les `.conf` de `Server/Config/` et `my.ini` sont générés par le launcher : ne pas les éditer à la main.
- Vérifier chaque point « à vérifier » de l'audit dans le code ou la configuration du core avant de le coder.

## Serveur (voir Documentation/02-Construction.md)
- Core : `Server/mangos-tbc/` (build Windows officiel `with-all` 15b6ddb4), bases `tbcrealmd`, `tbccharacters`, `tbcmangos`, `tbclogs` importées par `Tools/Init-Database.ps1`.
- **realmd et mangosd doivent être lancés avec `Server/mangos-tbc` comme dossier courant** (legacy.dll d'OpenSSL y est cherché, sinon arrêt immédiat). mangosd : `-c mangosd.conf -a ahbot.conf -p aiplayerbot.conf` ; anticheat désactivé par les variables `Anticheat_Enable=0`, `Anticheat_Warden_Enable=0`, `Anticheat_Antispam_Enable=0`.
- Harnais : `Tools/Run-World.ps1` (arrêt : fichier `Logs/server/stop`), console : `Tools/Send-Console.ps1`, arrêt console `server shutdown 0` puis fermeture de stdin.

## Client, données, launcher, français
- Client 2.4.3.8606 frFR dans `Client/` (venu de `Téléchargements\TGCfrFR.zip`, serveur tiers The Geek Crusade) ; `realmlist.wtf` **à la racine** du client en 2.4.3 ; `patch-frFR-3.MPQ` est non officiel (modifie Spell.dbc) : l'extraction n'utilise que les archives officielles (vue par liens physiques, `Tools/Extract-ClientData.ps1`, `repair.rs`).
- Launcher adapté à CMaNGOS (voir `Documentation/03-Launcher-Adaptation-BC.md`) ; tests unitaires : `cargo test` dans `Launcher/`.
- Français : `Documentation/09-Localisation-Inventaire.md` ; complément `Server/Database/custom/50_homecraft_frFR.sql` généré par `Tools/Localisation/frmerge` (sources locales VMaNGOS, AzerothCore/SylvaniaCore, mod-playerbots ; dérivé GPL/AGPL, pas sous licence HomeCraft).
