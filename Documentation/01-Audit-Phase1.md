# HomeCraft Edition BC — Phase 1 : audit technique

Date : 2026-10-16
Auteur : session Claude Code (poste Windows « Marty »)
Statut : **audit terminé ; composants téléchargés le 2026-10-16 avec accord de l'utilisateur (suite : 02-Construction.md, qui lève les points 1 à 6 de la section 7)**

Toutes les affirmations ci-dessous proviennent de vérifications faites le 2026-10-16 sur GitHub
(pages de dépôt, API GitHub, workflows CI, README). Ce qui n'a pas pu être vérifié est indiqué
explicitement (section 7).

---

## 1. Core serveur 2.4.3 retenu : CMaNGOS `mangos-tbc`

| Critère | Constat vérifié |
|---|---|
| Licence | GPL-2.0 ou ultérieure (README : « either version 2 of the License, or (at your option) any later version ») ; bibliothèques de `dep/` sous leurs licences propres |
| Activité | Dernier commit `e42118d` le **2026-10-16** ; dépôt mis à jour quotidiennement |
| Client | 2.4.3 build **8606** (seul build TBC ; TBC-DB l'indique explicitement) |
| Binaires Windows | **Oui, officiels** : le workflow `.github/workflows/windows-release.yml` tourne chaque jour à 6 h UTC sur `windows-2022` (VS 2022, Boost 1.87.0, x64, `RelWithDebInfo`) et publie sur la release `latest` (« Development Build(AAAA-MM-JJ) ») cinq archives par commit : `default`, `with-all`, `with-playerbot-ahbot`, `with-playerbot`, `with-ahbot`. Les anciennes archives restent disponibles (335 assets le 2026-10-16) |
| Variante choisie | **`with-all`** = `BUILD_EXTRACTORS=ON` + `BUILD_PLAYERBOTS=ON` + `BUILD_AHBOT=ON` + `BUILD_RECASTDEMOMOD=ON` + `BUILD_GIT_ID=ON` : serveur, extracteurs et bots dans un seul zip, **aucune compilation nécessaire** (le poste a VS Build Tools + CMake, mais compiler un core CMaNGOS exige en plus Boost, MySQL dev libs et OpenSSL : inutile ici) |
| Archive figée | `mangos-tbc-with-all-15b6ddb4.zip`, commit `15b6ddb4ec9e443d49f4e438af73782ce5c16491`, publiée le 2026-10-10, 86 344 468 octets, SHA-256 `ff1f26c7…207f` (digest fourni par l'API GitHub, à recontrôler au téléchargement) |
| Contenu de l'archive | Le workflow zippe le dossier `bin/x64_RelWithDebInfo` entier. **À vérifier au premier téléchargement** : présence de `realmd.exe`, `mangosd.exe`, `ad.exe`, `vmap_extractor.exe`, `vmap_assembler.exe`, `MoveMapGen.exe`, des DLL (libmysql, OpenSSL ?) et des `*.conf.dist` (sinon les prendre dans les sources) |
| Bases | 4 bases : `tbcrealmd`, `tbccharacters`, `tbcmangos`, `tbclogs` (convention `<era><base>` du guide d'installation) ; schémas dans `sql/base/{realmd,characters,mangos,logs}.sql` des sources, mises à jour dans `sql/updates/` |
| MariaDB | Le guide officiel parle de « MySQL server » ; CMaNGOS est utilisé couramment avec MariaDB (images Docker communautaires sur MariaDB). **À vérifier à l'import** : collations, `sql_mode`, fonctions MySQL 8 éventuelles dans TBC-DB |
| Comptes | Console `mangosd` : `account create <nom> <mdp>`, `account set addon <nom> 1` (0 = Classic, **1 = TBC**, 2 = WotLK), `account set gmlevel <nom> 0-3`, `account delete <nom>` |
| AHBot | Intégré au core (`BUILD_AHBOT`), fichier `ahbot.conf` (réglages par qualité d'objet, pas de table d'objets comme en Vanilla) |

### Alternatives écartées

- **Compilation locale de mangos-tbc** : possible (CMake + VS 2022 présents dans `C:\BuildTools`) mais inutile puisque le build quotidien officiel couvre toutes les options ; à garder en secours si un build quotidien casse.
- **Forks communautaires** (`yushicsm/mangos-tbc` avec anticheat/Eluna, builds Windows de `celguar/mangosbot-bots` r-2026-10-15) : non officiels, sans garantie de suivi ; écartés.
- **TrinityCore / AzerothCore** : pas de branche 2.4.3 maintenue.

## 2. Base de contenu : `cmangos/tbc-db`

| Critère | Constat vérifié |
|---|---|
| Licence | **GPL-3.0** (README : `LICENSE.md` à joindre à tout package redistribuable ; `COPYRIGHT.md` pour les éléments hors GPL) |
| Activité | Release quotidienne `latest` « Development Build(2026-10-16) », commit `87af76a0e12cbd0a29d5ccc2be56f52c2b0b199f` |
| Assets | `tbc-all-db.zip` (65,6 Mo, SHA-256 `2fdd8743…9278`), `tbc-world-db.zip` (65,2 Mo), `tbc-db-mysql-server.zip` (424 Mo, serveur MySQL préchargé : inutile), `tbc-sqlite-db.zip` (99,9 Mo, inutile) |
| Choix | **`tbc-all-db.zip`** (toutes les bases). Contenu exact à vérifier au téléchargement : dumps prêts à importer ou arborescence `InstallFullDB.sh` + `Full_DB/` + `updates/` |
| Méthode officielle | `InstallFullDB.sh` (bash, lit `InstallFullDB.config` : base, utilisateur, options bots / dev updates). Le launcher reproduira la séquence en Rust (import du dump puis des mises à jour dans l'ordre), comme en Vanilla |
| Compatibilité | « compatible with mangos-tbc core only », client 2.4.3 build 8606. Le core et la base étant tous deux quotidiens, **prendre une base datée au plus près du commit du core** (ici core du 10/10, base du 16/10 : écart de 6 jours à valider par l'import et un démarrage sans erreur de schéma) |

## 3. Bots : `cmangos/playerbots` (module officiel CMaNGOS, ex-ike3 / celguar)

| Critère | Constat vérifié |
|---|---|
| Expansions | Classic, TBC, WotLK (un seul dépôt, sous-dossiers par expansion) |
| Licence | Le dépôt n'a pas de fichier `LICENSE` à la racine (liste : `.gitattributes`, `.github`, `.gitignore`, `CMakeLists.txt`, `README.md`, `ahbot`, `botpch.*`, `playerbot`, `sql`). Fork de `celguar/mangosbot-bots` (ike3, GPL-2). **À confirmer** dans les en-têtes des sources au téléchargement avant diffusion |
| Activité | Dernier commit `81fea4c` le **2026-10-16** |
| Intégration | Compilé dans le build `with-all` (`BUILD_PLAYERBOTS=ON`). Côté données : SQL de `sql/characters/tbc` → base personnages, `sql/world/tbc` → base monde ; configuration `aiplayerbot<expansion>.conf.dist` à copier en `aiplayerbot.conf` à côté de `mangosd.conf` |
| Risque | Le build quotidien du core compile le module au commit courant du dépôt playerbots ; le SQL et le `.conf.dist` doivent provenir d'un commit proche (ici 16/10 pour un core du 10/10). Problème de compilation signalé en juillet 2026 sur Ubuntu (issue cmangos/issues #4115) : sans objet pour le build Windows officiel, qui produit bien les archives `with-playerbot` |
| Commandes | Non listées dans le README (lien externe « List of Commands ») ; à relever dans les sources (`playerbot/strategy`) pour l'aide-mémoire de l'onglet PlayerBots |

## 4. Composants réutilisés tels quels (déjà téléchargés pour Vanilla, à copier sans retélécharger)

| Composant | Fichier dans `HomeCraft\Build\downloads` | Licence |
|---|---|---|
| MariaDB 11.8.9 winx64 | `mariadb-11.8.9-winx64.zip` (97 016 251 octets, SHA-256 `830c4672…63f5`) | GPL-2 |
| HeidiSQL 12.21 portable | `HeidiSQL_12.21_64_Portable.zip` (29 789 003 octets, SHA-256 `fecb76a6…3fab`) | GPL-2 |
| Rust / Build Tools | déjà installés (`~/.cargo/bin`, `C:\BuildTools`) | — |

VanillaFixes et DXVK ne s'appliquent pas au client 2.4.3 (VanillaFixes vise 1.12.1 ; DXVK pourrait être proposé plus tard en option, hors périmètre de la phase 1).

## 5. Client 2.4.3 de l'utilisateur

- Build attendu : **8606** (2.4.3.8606). Lire `StringFileInfo` de `WoW.exe` (la version fixe peut mentir, piège Vanilla).
- Disposition : `Data\*.MPQ` (common, expansion, patch, patch-2) + `Data\frFR\*.MPQ` (locale, expansion-locale, patch-frFR, patch-frFR-2, speech, expansion-speech) ; `Data\frFR\realmlist.wtf` ; `WTF\Config.wtf`. Liste exacte à relever sur le client réel.
- **Aucun client 2.4.3 n'a été trouvé sur ce poste** (`Desktop\WoW Client FR` est un client 1.12.1). Le dossier du client doit être indiqué par l'utilisateur (ou copié dans `HomeCraft BC\Client\`).

## 6. Ports et identité

| Élément | Valeur |
|---|---|
| MariaDB / authentification / monde | **3317 / 3726 / 8087**, tous sur 127.0.0.1 (Vanilla 3315/3724/8085, WotLK prévu 3316/3725/8086) |
| Royaume | « HomeCraft BC », `realmlist.realmbuilds` doit accepter 8606 |
| Exécutable | `HomeCraft Edition BC.exe` (cargo : `homecraft-bc-launcher`), ProductName « HomeCraft Edition BC », CompanyName « BlaMacfly » |
| Variable d'environnement de test | `HOMECRAFT_BC_ROOT` (remplace `VANILLA_SOLO_ROOT`) |
| Palette | or `#E8C448` (titres), vert fel `#78DC5A` (états opérationnels, « BC » du titre) |

### Identité visuelle (faite le 2026-10-16)

- Logo fourni `HomeCraft Logo BC.png` : 1254×1254, 32 bits, **fond déjà transparent** (alpha 0 autour de l'anneau, vérifié par échantillonnage) : pas de détourage.
- Produits : `Launcher\assets\HomeCraft-original.png`, `logo-512.png`, `logo-256.png`, `homecraft.ico` (7 tailles, entrées PNG, via `Build\New-InstallerImages.ps1` réduit à l'icône), `fond.jpg` (1661×947 natif, JPEG 85, 314 Ko), `SOURCES.md`.
- Launcher compilé (`cargo build --release`, 1 min 12), signé et déployé (`Build\Sign-Launcher.ps1`), fenêtre vérifiée par capture (`Documentation\images\launcher-home.png`, script `Tools\Capture-Launcher.ps1`).
- Signature : certificat auto-signé **du Vanilla** (sujet « HomeCraft Edition Vanilla (auto-signé) », empreinte `52635227…4CEA`). Choix à faire : garder ce certificat ou en créer un « HomeCraft Edition BC ».

## 7. Non vérifié à ce stade (à contrôler au téléchargement, avant tout code serveur)

1. Contenu réel du zip `with-all` : exécutables, DLL, `*.conf.dist`, scripts d'extraction.
2. Contenu réel de `tbc-all-db.zip` : format des dumps, présence des quatre bases, mises à jour.
3. Noms exacts des fichiers playerbots (`aiplayerbot-tbc.conf.dist` ?, chemins SQL) et licence du module.
4. Import de TBC-DB dans MariaDB 11.8 (collations, `sql_mode`, taille du buffer).
5. Chaîne « World initialized » (ou équivalent) dans la sortie de `mangosd` pour `wait_ready` ; commande console d'arrêt (`server shutdown` ?).
6. Sortie des extracteurs (dossiers `dbc`, `maps`, `vmaps`, `mmaps`) et options de `ExtractResources.sh` à transposer en Rust.
7. Tables `locales_*` et couverture frFR de TBC-DB (phase localisation).
8. Gestion des MPQ V2 (2.4.3) par la crate `wow-mpq` 0.7 pour l'onglet Patchs (création depuis un dossier) ; sinon activer/désactiver/ajouter seulement.

## 8. Téléchargements proposés (aucun lancé : accord de l'utilisateur requis)

| # | Fichier | Taille | Source | Usage |
|---|---|---|---|---|
| 1 | `mangos-tbc-with-all-15b6ddb4.zip` | 86,3 Mo | release `latest` cmangos/mangos-tbc | binaires serveur + extracteurs |
| 2 | `mangos-tbc` sources au commit `15b6ddb4` (zip GitHub) | ~40 Mo (estimation) | archive du commit | `sql/base`, `conf.dist`, scripts d'extraction, obligation GPL |
| 3 | `cmangos/playerbots` sources au commit `81fea4c` (zip GitHub) | ~10 Mo (estimation) | archive du commit | SQL bots, `aiplayerbot.conf.dist`, licence |
| 4 | `tbc-all-db.zip` | 65,6 Mo | release `latest` cmangos/tbc-db | bases de données |

MariaDB et HeidiSQL : **copie locale** depuis `HomeCraft\Build\downloads` (empreintes identiques dans les deux manifestes).

## 9. Plan des phases suivantes

1. **Construction** (après accord) : `Get-Components.ps1 -Pin`, listing des archives, mise en place `Server\mangos-tbc`, `Server\Database\{mariadb,sql}`, scripts `Tools\Common.ps1` (ports 3317/3726/8087, bases `tbc*`), import manuel des bases dans MariaDB, génération des `.conf`, extraction depuis le client 2.4.3, premier démarrage `realmd` + `mangosd`, connexion manuelle → `02-Construction.md`.
2. **Launcher** : adapter `paths.rs`, `server.rs` (noms des `.conf`, chaînes de connexion CMaNGOS `127.0.0.1;3317;user;mdp;tbcmangos`, `account set addon`), `client.rs` (2.4.3, `Data\frFR`), `conf_catalog.rs` (clés `mangosd.conf` TBC + `aiplayerbot.conf` + `ahbot.conf`), `repair.rs` (extracteurs CMaNGOS), `i18n.rs` (textes 2.4.3) ; onglets un par un.
3. **Fiabilité, packaging, validation** : comme en Vanilla (`Tests\*.ps1`, `New-Package.ps1`, `10-Validation-Finale.md`).

## Sources

- https://github.com/cmangos/mangos-tbc (README, releases `latest`, `.github/workflows/windows-release.yml`, `cmake/options.cmake`)
- https://api.github.com/repos/cmangos/mangos-tbc/releases/tags/latest (assets, tailles, digests)
- https://github.com/cmangos/tbc-db et https://api.github.com/repos/cmangos/tbc-db/releases/latest
- https://github.com/cmangos/playerbots (README, contenu racine, dernier commit)
- https://github.com/cmangos/issues/wiki/Installation-Instructions (procédure Windows, bases, commandes de compte)
- https://github.com/cmangos/issues/issues/4115 (build Linux avec bots, juillet 2026)
- https://github.com/celguar/mangosbot-bots/releases (builds Windows non officiels, écartés)
