# Keira3 — éditeur de la base monde (2026-10-17)

Demande de l'utilisateur : intégrer Keira3 (éditeur de base de données d'AzerothCore) au launcher, puis, après
réflexion, **un simple bouton dans l'onglet Base de données** plutôt qu'un onglet dédié. Puis : vérifier l'historique
du projet et récupérer la version compatible avec le cœur.

## 1. Pourquoi un bouton et pas une interface intégrée

- Keira3 est une application Angular 22 / Electron 41 (TypeScript) : egui ne peut pas l'afficher.
- Licence **AGPL-3.0** : reprendre son code dans le launcher le placerait sous AGPL, incompatible avec la licence
  propriétaire HomeCraft. Livrer l'exécutable **inchangé** à côté du launcher reste permis (agrégat), avec sa licence
  (`Tools/Keira3/LICENSE.Keira3.txt`) et un lien vers les sources (à reporter dans `LICENCES/SOURCES.md` au packaging).

## 2. Quelle version est compatible avec le cœur Sylvania

Méthode : clone complet `Build/src/keira3` (référence, jamais dans le package). Pour chaque version, extraction des
colonnes déclarées par les entités `libs/shared/acore-world-model/src/entities/*.type.ts`, comparées aux 4 326
colonnes (331 tables) du dump `Server/Database/sql/acore_world.sql.gz`. Keira3 n'a pas de grille de compatibilité :
son tableau de bord lit seulement la table `version` (préfixes « AzerothCore » et « ACDB »).

| Version | Colonnes MySQL attendues absentes de la base | Verdict |
|---|---|---|
| master (2026-10) | `creature.id` (la base a `id1/id2/id3`) | incompatible |
| **binaire officiel « 3.10.5 »** | `creature.id` : reconstruit le 2026-06-19 depuis master (constaté dans `app.asar`) | **incompatible** |
| **sources de la balise v3.10.5** (2026-05-10, `c5c01ddb`) | aucune | **compatible** |
| v3.10.4 (2026-01-18) | `creature_template.scale`, `mechanic_immune_mask`, `spell_school_immune_mask` | partiel |
| v3.10.3 (2025-11-10) | idem + colonnes formateur (`trainer_*`, ancien `npc_trainer`) | partiel |

Le tournant : AzerothCore a normalisé la table `creature` le 2026-06-16 (`cf0d86a9f`, « extracting multi-ID
spawns ») et Keira3 s'est aligné le 2026-06-18 (`100f966e`, #3779), commit contenu dans aucune version publiée. La
v3.10.5 contient le correctif « ac compatibility » du 2026-04-15 (#3716), qui retire les trois colonnes de
`creature_template` absentes du fork (base upstream du 2026-03-31).

Les 11 tables « absentes » communes à toutes les versions (`areas_and_zones`, `factions`, `holiday`,
`item_enchantment`, `item_extended_cost`, `item_limit_category`, `languages`, `maps`, `skills`, `sound_entries`,
`spells`) sont les tables de référence DBC que Keira3 embarque dans sa base SQLite (`resources/app.asar`) : pas de
MySQL. Les tables maison Sylvania (mythic+, AHBot de marché, défis…) n'ont pas d'éditeur dans Keira3 : explorateur
intégré ou HeidiSQL.

## 3. Compilation

Checkout de la balise dans `Build/src/keira3-v3.10.5` (`git worktree`), `npm ci` puis `npm run electron:windows`
(Node 24.20.0). Le build Angular et `release/win-unpacked` réussissent ; l'étape finale (exécutable portable unique)
échoue : electron-builder décompresse son outil de signature `winCodeSign`, qui contient des liens symboliques macOS
que Windows refuse sans mode développeur. Sans incidence : `win-unpacked` est l'application complète, sans
installation, copiée dans **`Tools/Keira3/`** (384 Mo, `Keira3.exe`). Empreintes figées dans `Build/manifest.json`.
Vérifié dans le build : `creature` → `id1`.

## 4. Bouton du launcher

Onglet Base de données : « Ouvrir dans Keira3 » à côté de HeidiSQL, actif si la base tourne et si
`Tools/Keira3/Keira3.exe` existe (`Layout::keira_exe`, `Command::OpenKeira`). Keira3 n'accepte aucun paramètre de
connexion en ligne de commande et mémorise la connexion (« Remember me », coché par défaut, stockage local Electron
dans `%APPDATA%\Keira3`) : le bouton copie le mot de passe SQL dans le presse-papiers et affiche les paramètres à
saisir la première fois (127.0.0.1, port 3315, utilisateur `acore`, base `acore_world`). Option de test
`--do keira`.

Testé le 2026-10-17 : MySQL seule démarrée (`Tools/Start-Database.ps1`), launcher `--tab database --do keira` →
« Keira3 ouvert », processus présents ; table `version` = « AzerothCore rev. a7161cd0adb7+ … » / « ACDB 335.16-dev »
(préfixes attendus par Keira3). **Non testé** : la saisie de la connexion et l'édition dans l'interface de Keira3.

## 5. Mise à jour future

Ne pas prendre un binaire Keira3 publié sans vérifier : tant que le cœur garde `creature.id1/id2/id3`, rester sur
v3.10.5 compilée. Si le fork rejoue un jour la normalisation de juin 2026, passer à une version postérieure à
`100f966e` et refaire la comparaison de colonnes (§2).

## 6. Connexion sans saisie : port 3306 et root / root (2026-10-17, décision utilisateur)

Keira3 pré-remplit `127.0.0.1`, port **3306**, `root` / `root`, base `acore_world`. Pour n'avoir rien à saisir, la
base locale passe sur ces valeurs :

- **Port MySQL 3315 → 3306** (`paths::DB_PORT`, `Tools/Common.ps1`). 3315 était aussi celui de l'édition Vanilla
  (conflit si les deux bases tournaient) ; BC utilise 3317 ; 3306 n'est pris par aucune édition HomeCraft.
- **Mot de passe root = `root`** (`credentials::ROOT_PASSWORD`, `Common.ps1`) pour les nouvelles installations. La
  base reste limitée à 127.0.0.1. Le compte du serveur de jeu (`acore`, mot de passe aléatoire) ne change pas.
- **Migration des installations existantes**, automatique au premier lancement du nouveau launcher :
  `load_or_create` garde l'ancien mot de passe dans `legacyRootPassword` et écrit `root`. `MySql::start` applique le
  nouveau : à chaud avec l'ancien si MySQL tourne déjà, sinon par `--init-file` au démarrage (ALTER USER sur
  `root@localhost` et `root@127.0.0.1`). L'ancien mot de passe est ensuite retiré du fichier. `worldserver.conf`,
  `authserver.conf` et `playerbots.conf` prennent le nouveau port à la régénération du démarrage.
- **Onglet Base de données** : encadré « Connexion à la base locale » (hôte, port, identifiant, mot de passe, bases,
  boutons Copier). HeidiSQL est lancé en `root`. Le bouton Keira3 ne copie plus rien : « cliquez sur Connect ».
- **HeidiSQL** était absent de l'édition WoTLK (« HeidiSQL n'est pas installé dans Tools/HeidiSQL ») : copié depuis
  `../HomeCraft/Tools/HeidiSQL` (12.21 portable, même `heidisql.exe`), **sans** `portable_settings.txt`, `Backups`,
  `Snippets` ni `tabs.ini` de l'édition Vanilla (sessions et mots de passe enregistrés). À ajouter au manifeste au
  packaging (entrée `heidisql-portable` du manifeste Vanilla).

Testé le 2026-10-17 après fermeture du launcher de l'utilisateur :
- identifiants sauvegardés avant migration (`UserData/db-credentials.avant-migration-2026-10-17.json`) ;
- `--do backup` : MySQL démarrée sur 3306 par `--init-file` de migration, sauvegarde réussie en root / root,
  `legacyRootPassword` retiré du fichier ;
- `Tools/Start-Database.ps1` puis connexion avec le pilote de Keira3 (`mysql2`, 127.0.0.1:3306, root / root,
  acore_world) : `version` = « ACDB 335.16-dev », `creature.id1` lu, `creature_template.scale` absente
  (ER_BAD_FIELD_ERROR, confirme l'incompatibilité de la 3.10.4) ;
- `Tests/Test-Launcher.ps1` : **18/18**, les trois services sur 3306/3724/8085, arrêt propre.
Non testé : le clic sur Connect dans la fenêtre de Keira3. Une connexion Keira3 enregistrée avec l'ancien port doit
être remplacée par les valeurs par défaut.
