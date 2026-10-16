# Phase 6 — version portable, installateur et support physique

Décision utilisateur (2026-10-14) : pas d'installateur, une version portable destinée à un CD. **Révisée le 2026-10-15** : HomeCraft ne sera pas lancé depuis un disque, et il sera **distribué à la famille et aux amis** → un **installateur Inno Setup** est ajouté, le portable (zip) restant disponible ; l'ISO devient une simple commodité.

## Installateur Windows (2026-10-15, abandonné le 2026-10-16)

**Décision utilisateur du 2026-10-16 : retour au portable seul.** L'installateur avait été demandé pour une distribution à la famille et aux amis, mais l'utilisateur souhaitait y inclure le client World of Warcraft, ce qui serait une redistribution de contenu Blizzard (exclue par le cahier des charges et par `LICENCE.txt`). Le fichier `-setup.exe` a été supprimé ; les scripts (`HomeCraft.iss`, `New-Installer.ps1`, `New-InstallerImages.ps1`, `Tests/Test-Installer.ps1`) et Inno Setup dans `Tools/InnoSetup` sont conservés, inactifs, au cas où. L'icône intégrée à l'exécutable (`build.rs`) reste en place. La description ci-dessous est celle de l'installateur testé (14/14) avant abandon.

```bash
powershell -ExecutionPolicy Bypass -File Build\New-Installer.ps1 -Thumbprint <empreinte> -Rebuild
```

- Outil : **Inno Setup 6.7.3** (composant `innosetup` du manifeste, installateur officiel signé Pyrsys B.V., installé sur le poste de construction dans `Tools/InnoSetup` avec `/VERYSILENT /CURRENTUSER /NOICONS /DIR=…` ; jamais redistribué). Script `Build/HomeCraft.iss`, source = le package portable de `Build/package` (reconstruit avec `-Rebuild`).
- Assistant en français, style moderne : page de licence (`LICENCE.txt`), page d'informations (`Build/package-files/AVANT-INSTALLATION.txt` : client non fourni, dossier inscriptible, SmartScreen), choix du dossier (défaut `C:\Jeux\HomeCraft Edition Vanilla`, hors Program Files et sans droits administrateur grâce à `PrivilegesRequired=lowest` avec dialogue de choix), **page « Client World of Warcraft 1.12.1 »** facultative qui écrit `UserData\settings.json` (`client_dir`, UTF-8) si aucun réglage n'existe, raccourci Bureau optionnel, raccourcis menu Démarrer (launcher, manuel, désinstallation), lancement à la fin.
- Mise à jour par-dessus une installation existante : `UsePreviousAppDir`, `CloseApplications` (Restart Manager ferme le launcher et ses services), réglages et `UserData` conservés.
- Désinstallation : retire le programme, `Server\Data`, `Logs` et les configurations générées ; **ne touche jamais `UserData` ni `Client`** (`uninsneveruninstall`) et le dit dans un message final.
- Visuels : `Build/New-InstallerImages.ps1` génère le bandeau de l'assistant (plusieurs densités, recadrage « cover » du visuel fourni par l'utilisateur `Build/installer-assets/visuel-source.png`), le petit logo, et l'icône multi-tailles `Launcher/assets/homecraft.ico` (16 à 256 px) intégrée à l'exécutable par `Launcher/build.rs` (crate `winresource`, `rc.exe` du SDK Windows) avec les informations de version (produit, éditeur, copyright). Le visuel source dérive d'illustrations Blizzard : il n'est **pas** copié dans `Sources\Build` du package (exclusion `installer-assets\*`) et n'apparaît que dans l'assistant.
- Signature : installateur et désinstallateur signés par `SignTool=hc` (signtool, SHA-256, horodatage DigiCert) avec le même certificat que le launcher ; SmartScreen affichera « éditeur inconnu » tant que le certificat est auto-signé.
- Compression lzma2/ultra64 solide. Ligne de commande utile : `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /DIR="…" /ClientDir="…"` (tests).
- Test : `Tests/Test-Installer.ps1` (installation silencieuse dans `C:\HomeCraftTest-Setup` avec le client du projet, raccourcis, `settings.json` écrit, premier démarrage complet du launcher installé, arrêt propre par fermeture de fenêtre, désinstallation silencieuse, `UserData` conservé).

## Construction (poste développeur)

```bash
powershell -ExecutionPolicy Bypass -File Build\New-Package.ps1 -Zip
```

Prérequis : launcher compilé **et signé** (le script refuse un exécutable non signé). Produit `Build/package/HomeCraft-Edition-Vanilla-v<version>/` et l'archive zip. Puis, pour le support physique :

```bash
powershell -ExecutionPolicy Bypass -File Build\New-Iso.ps1
```

`New-Iso.ps1` utilise l'API IMAPI2 de Windows (UDF + Joliet + ISO 9660), sans outil externe ; l'image se grave avec l'outil intégré à Windows. Taille mesurée le 2026-10-14 : **446 Mo** (289 fichiers), donc un CD-ROM de 700 Mo suffit ; au-delà de 700 Mo le script signale qu'un DVD est requis.

## Contenu du package

| Élément | Contenu |
|---|---|
| `HomeCraft Edition Vanilla.exe` | launcher signé (auto-signé pour l'instant) |
| `Server/vmangos`, `Server/Config/*.dist.in`, `Server/Database/sql`, `Server/Database/custom` | serveur, modèles, dumps, compléments (AHBot) |
| `Server/Database/mariadb` | MariaDB **allégée** : sans `.pdb` (73 Mo), `include/`, `.lib`, tests |
| `Tools/HeidiSQL`, `Tools/Scripts` | explorateur externe, scripts PowerShell d'exploitation |
| `Documentation/`, `LISEZMOI.txt` | documentation projet et mode d'emploi utilisateur |
| `LICENCE.txt`, `LICENCES/` | Licence propriétaire HomeCraft (racine et `LICENCES/HomeCraft-Licence.txt`), GPL-2 (VMaNGOS, MariaDB, HeidiSQL), MIT (VanillaFixes), zlib (DXVK), `SOURCES.md` (versions exactes, liens, liste des crates Rust et leurs licences) |
| `Sources/Launcher`, `Sources/Build` | sources du launcher et scripts de construction (obligation GPL) |
| `Build/manifest.json`, `Build/files-manifest.json`, `SHA256SUMS.txt` | composants figés, empreintes de chaque fichier (VÉRIFIER LES FICHIERS, contrôle après gravure) |
| `autorun.inf` | ouverture du launcher (si l'exécution automatique est autorisée) |

Jamais inclus : `Client/` (Blizzard), `Server/Data/` (dérivé des MPQ), `UserData/`, `Logs/`, `Build/downloads/`.

## Comportement sur un support en lecture seule

Au démarrage, le launcher teste l'écriture dans `UserData/`. Sur un CD (ou un dossier protégé) il passe en état NON INSTALLÉ, explique la situation et propose « Installer sur ce PC… » : sélecteur de dossier natif, copie intégrale (hors `UserData`, `Logs`), suppression de l'attribut lecture seule hérité du CD, puis lancement du launcher copié et fermeture. Mode test : `--install-to <dossier>`.

## Premier démarrage sur un PC vierge

JOUER (ou DÉMARRER LE SERVEUR) enchaîne : initialisation du datadir MariaDB → import des quatre bases + compléments `custom/*.sql` → extraction dbc/maps/vmaps depuis le client de l'utilisateur (dossier `Client/` ou chemin choisi dans PARAMÈTRES, sélecteur natif) → realmd → mangosd. Sans client valide, le démarrage s'arrête avec un message explicite. Les mmaps (pathfinding complet) se génèrent à la demande depuis PARAMÈTRES.

## Tests

`Tests/Test-Package.ps1` : copie vierge du package dans `C:\HomeCraftTest`, premier démarrage complet, arrêt brutal simulé puis redémarrage propre, création de l'ISO, montage en lecture seule et copie assistée. Résultats dans `05-Tests-Phases4-5.md` (section phase 6).
