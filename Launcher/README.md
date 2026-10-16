# Launcher — WoW Vanilla Solo Edition (Rust)

Licence propriétaire HomeCraft (voir `../LICENCE.txt` : usage personnel, lecture du code autorisée, reproduction et modification interdites ; les bibliothèques Rust utilisées restent sous leurs licences MIT/Apache-2.0 et assimilées, aucune dépendance GPL/LGPL n'est liée). Un seul exécutable, 100 % Rust (aucune bibliothèque C à livrer).

## Construction

Prérequis : `rustup` (toolchain stable, cible `x86_64-pc-windows-msvc`) et Visual Studio Build Tools 2022 avec la charge « Développement Desktop C++ » (linker `link.exe`).

```bash
cargo build --release
```

Puis signer (chaque compilation écrase la signature) :

```bash
powershell -ExecutionPolicy Bypass -File ..\Build\Sign-Launcher.ps1 -Thumbprint (Get-Content ..\UserData\code-signing-thumbprint.txt)
```

L'exécutable `target/release/vanilla-solo-launcher.exe` détecte la racine d'installation en remontant depuis son emplacement jusqu'au dossier contenant `Server/` et `Tools/` (ou via la variable `VANILLA_SOLO_ROOT`). En développement, lancer depuis `Launcher/` fonctionne donc directement.

## Modules

| Fichier | Rôle |
|---|---|
| `paths.rs` | disposition des dossiers, ports (3315 / 3724 / 8085), composants attendus |
| `credentials.rs` | compte SQL local (`UserData/db-credentials.json`, même format que les scripts PowerShell) |
| `settings.rs` | réglages utilisateur (`UserData/settings.json`) |
| `db.rs` | MariaDB portable : `my.ini`, initialisation, démarrage, santé réelle (port + `SELECT 1`), `SHUTDOWN`, import initial, requêtes de lecture |
| `server.rs` | génération de `realmd.conf` / `mangosd.conf` depuis les modèles, royaume, realmd, mangosd avec console sur l'entrée standard, arrêt propre (`server exit` + fermeture de stdin) |
| `client.rs` | contrôle du client (version 1.12.1.5875 via l'API Windows, MPQ requis), `realmlist.wtf`, `Config.wtf`, lancement |
| `orchestrator.rs` | machine à états (NON INSTALLÉ … JEU EN COURS, ERREUR), thread de pilotage, surveillance périodique, comptes, console |
| `ui.rs` | interface egui : accueil, Comptes, PlayerBots, Monde, Patchs, Base de données (explorateur en lecture + HeidiSQL), Sauvegardes, Paramètres, Journaux |
| `conf_catalog.rs` | catalogue des clés `mangosd.conf` exposées (PlayerBots, Monde) : libellés, aides, types, défauts |
| `botgen.rs` | génération de personnages bots de peuplement (compte HOMECRAFTBOTS, table `playerbot`) |
| `patches.rs` | gestionnaire des archives `patch-X.MPQ` du client (activer/désactiver/ajouter/supprimer) |
| `backup.rs`, `repair.rs` | sauvegardes et restauration, vérification par manifeste, réparation, extraction, diagnostic |
| `util.rs` | horodatage local, SHA-256, copie d'arborescence |
| `i18n.rs` | toutes les chaînes affichées (français) |

## Machine à états

`NotInstalled → Installed → Verifying → Initializing → StartingDb → StartingAuth → StartingWorld → Ready ⇄ Playing → Stopping → Installed`, `Error` depuis tout état de démarrage. JOUER n'est actif qu'en `Ready` avec un client valide ; la fermeture de la fenêtre arrête d'abord les services.

## Phase 5 : fiabilité

| Fonction | Mise en œuvre |
|---|---|
| SAUVEGARDER | `backup.rs` : `mariadb-dump --lock-tables` de `characters` et `realmd` (cohérent malgré le mélange MyISAM/InnoDB), `settings.json`, manifeste `backup.json` avec SHA-256 de chaque fichier, zip horodaté dans `UserData/Sauvegardes/`. Pendant le jeu : `saveall` envoyé à la console avant l'export. MariaDB est démarrée temporairement si les services sont arrêtés. Aucune suppression automatique. |
| RESTAURER | empreintes vérifiées avant tout import, sauvegarde de sécurité automatique de l'état courant, arrêt de realmd/mangosd (refus si le jeu est lancé), confirmation explicite dans l'interface, puis import. |
| VÉRIFIER LES FICHIERS | `repair.rs` : comparaison taille + SHA-256 avec `Build/files-manifest.json` (généré par `Build/New-FilesManifest.ps1` à chaque packaging), présence des données extraites (dbc/maps/vmaps/mmaps), bases et table `migrations`. Rapport affiché dans une fenêtre. |
| RÉPARER | réimporte les bases manquantes ou sans schéma (**jamais** `characters` si elle contient des personnages), régénère les configurations gérées et le royaume, ré-extrait dbc/maps/vmaps depuis le client si absents. Les fichiers gérés manquants ou altérés sont listés comme irréparables (réinstallation). |
| Diagnostic | zip dans `UserData/` : journaux, configurations avec mots de passe masqués, état des composants. |
| mmaps | génération à la demande depuis Paramètres (longue). |

Mode automatique pour les tests : `--start`, `--quit-after N`, `--do backup|verify|repair|diagnostic|restore <zip>` (`Tests/Test-Launcher.ps1`, `Tests/Test-Phase5.ps1`).

## VanillaFixes et DXVK (option, Paramètres)

`client.rs::apply_vanillafixes` dépose `VanillaFixes.exe` / `VfPatcher.dll` (et `dlls.txt` s'il manque) dans le dossier du client et `launch` démarre `VanillaFixes.exe` au lieu de `WoW.exe` ; DXVK dépose/retire `d3d9.dll` (retrait seulement si l'empreinte est celle de notre copie). Composant figé `vanillafixes` du manifeste, licences MIT et zlib dans `Tools/VanillaFixes/`.

## Limites restantes

- Création de compte uniquement serveur démarré (console mangosd) ; SRP6 hors ligne à venir.
- Explorateur de base en lecture seule ; les écritures passent par HeidiSQL.
- Les fichiers gérés altérés ne sont pas reconstitués depuis les archives de `Build/downloads` (réinstallation).
