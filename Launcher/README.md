# Launcher — HomeCraft Edition BC (Rust)

Licence propriétaire HomeCraft (voir `../LICENCE.txt` : usage personnel, lecture du code autorisée, reproduction et
modification interdites ; les bibliothèques Rust utilisées restent sous leurs licences MIT/Apache-2.0 et assimilées,
aucune dépendance GPL/LGPL n'est liée — contrôlé par `Build/New-Package.ps1`). Un seul exécutable, 100 % Rust.

Serveur piloté : CMaNGOS `mangos-tbc` (build Windows officiel `with-all`, module PlayerBots et AHBot compilés),
base TBC-DB, MariaDB portable, client World of Warcraft 2.4.3 build 8606 fourni par l'utilisateur.

## Construction

Prérequis : `rustup` (stable, `x86_64-pc-windows-msvc`) et Visual Studio Build Tools 2022 « Développement Desktop C++ »
(`link.exe`, `rc.exe` pour l'icône).

```bash
cargo build --release
cargo test
```

Puis signer et déployer (chaque compilation écrase la signature), depuis PowerShell :

```powershell
& ..\Build\Sign-Launcher.ps1 -Thumbprint <empreinte du certificat>
& ..\Build\New-FilesManifest.ps1
```

L'exécutable détecte la racine d'installation en remontant jusqu'au dossier contenant `Server/` et `Tools/`
(ou via la variable `HOMECRAFT_BC_ROOT`).

## Modules

| Fichier | Rôle |
|---|---|
| `paths.rs` | disposition, ports 3317 / 3726 / 8087 (127.0.0.1), bases `tbcrealmd` / `tbccharacters` / `tbcmangos` / `tbclogs`, build 8606 |
| `credentials.rs` | compte SQL local (`UserData/db-credentials.json`) |
| `settings.rs` | réglages utilisateur (`UserData/settings.json`) |
| `db.rs` | MariaDB portable : `my.ini`, initialisation, démarrage (base « possédée » ou non), arrêt SQL, import des dumps TBC-DB puis des compléments `Server/Database/custom/*.sql`, requêtes, suppression de compte hors ligne |
| `server.rs` | génération de `realmd.conf`, `mangosd.conf`, `aiplayerbot.conf` (aussi copié dans `Server/mangos-tbc` : le module ignore l'option `-p`), `ahbot.conf` ; royaume ; realmd et mangosd lancés **depuis `Server/mangos-tbc`** (legacy.dll d'OpenSSL) ; anticheat coupé par variables d'environnement ; arrêt `server shutdown 0` + fermeture de stdin |
| `client.rs` | client 2.4.3 (archives officielles `Data` + `Data/frFR`), `realmlist.wtf` à la racine du client, `Config.wtf`, lancement |
| `repair.rs` | vérification par manifeste, extraction `ad` / `vmap_extractor` / `vmap_assembler` / `MoveMapGen` sur une vue des seules archives officielles, mmaps (obligatoires), diagnostic |
| `orchestrator.rs` | machine à états, séquence JOUER (base → configs → extraction et mmaps si absentes → realmd → mangosd, jusqu'à 3 h au premier démarrage pour l'initialisation des PlayerBots), comptes (`account set addon 1`), sauvegardes, surveillance |
| `conf_catalog.rs` | réglages exposés : ~70 clés `mangosd.conf` (onglet Monde), 19 clés `aiplayerbot.conf` (onglet PlayerBots) |
| `botgen.rs` | nombre de bots aléatoires, suppression des bots (serveur arrêté) |
| `patches.rs`, `addons.rs`, `mpqbuild.rs` | patchs du client (`Data` et `Data/frFR`), AddOns (interface 20400), construction MPQ |
| `backup.rs` | sauvegardes `tbccharacters` + `tbcrealmd`, restauration vérifiée |
| `ui.rs`, `i18n.rs` | interface egui 0.32 ; toutes les chaînes en français |

Historique et pièges : `Documentation/03-Launcher-Adaptation-BC.md`.
