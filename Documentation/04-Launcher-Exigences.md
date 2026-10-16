# Launcher — exigences fonctionnelles (Phase 4)

Dernière mise à jour : 2026-10-14.

## Technologie : Rust (décision utilisateur du 2026-10-14, remplace la recommandation C++/Qt de l'audit §7)

Le launcher est écrit **entièrement en Rust**. Licence : GPL-3.0 à l'origine, remplacée le 2026-10-15 (décision utilisateur) par la **licence propriétaire HomeCraft** (`LICENCE.txt` à la racine : usage personnel, lecture autorisée, reproduction/diffusion/modification interdites, exclusion de garantie et limitation de responsabilité, droit français). Elle ne couvre que les créations de l'auteur (launcher, scripts, documentation, compléments SQL, identité visuelle) ; VMaNGOS, MariaDB, HeidiSQL, VanillaFixes, DXVK et les crates Rust restent sous leurs licences propres (vérifié : aucune crate GPL/LGPL n'est liée ; `r-efi` est en triple licence MIT/Apache/LGPL, MIT retenue). Le texte a été rédigé sans conseil juridique : à faire relire par un avocat avant toute diffusion, et compléter l'article 1.1 (identité et coordonnées du titulaire). Pile proposée, 100 % Rust (aucune bibliothèque C à lier, un seul exécutable) :

| Besoin | Crate | Licence |
|---|---|---|
| Interface graphique (fenêtre principale, sections Comptes et Base de données, grilles, barre de progression) | `eframe` / `egui` + `egui_extras` (tables virtualisées) | MIT / Apache-2.0 |
| Accès MariaDB (client MySQL pur Rust, pas de libmysql) | `mysql` | MIT / Apache-2.0 |
| Processus realmd / mangosd / mariadbd, entrée standard de la console | `std::process` + threads | — |
| Contrôles de santé (ports 3315 / 3724 / 8085) | `std::net::TcpStream` | — |
| Empreintes et manifeste | `sha2`, `serde`, `serde_json` | MIT / Apache-2.0 |
| Archives (sauvegardes, composants) | `zip` | MIT |
| Création de compte serveur arrêté (SRP6 : `v`, `s`) | `num-bigint`, `sha1`, `rand` | MIT / Apache-2.0 |
| Journalisation | `tracing` + `tracing-appender` | MIT |
| Textes français | fichiers de ressources intégrés (`include_str!`), pas de chaînes codées en dur | — |

Build : `cargo build --release` (cible `x86_64-pc-windows-msvc`), installateur Inno Setup en Phase 6.
Les scripts PowerShell de `Tools/` restent le harnais de développement et la spécification de référence des comportements (ports, ordre de démarrage, arrêt propre, génération des configurations) ; le launcher Rust les remplace pour l'utilisateur final.

Prérequis poste de build (absents le 2026-10-14) : `rustup` (toolchain stable) et Visual Studio Build Tools 2022 (composant C++ pour le linker MSVC).

## Identité visuelle (demande utilisateur du 2026-10-14)

- Titre : **HomeCraft Edition Vanilla** (fenêtre et bandeau).
- Logo : `HomeCraft.png` fourni par l'utilisateur (version à fond transparent du 2026-10-14 23:34) → `Launcher/assets/logo-512.png` (bandeau) et `logo-256.png` (icône de fenêtre), transparence conservée.
- Fond : `Taverne.png` du Sylvania Launcher (même auteur, qui en détient les droits) → `Launcher/assets/fond.jpg` (1920×1095, JPEG 85), affiché en couverture avec assombrissement.
- Provenance consignée dans `Launcher/assets/SOURCES.md`. Les images sont intégrées à l'exécutable à la compilation.

## Signature numérique (demande utilisateur du 2026-10-14)

Le launcher distribué doit être signé (Authenticode SHA-256 + horodatage). Script : `Build/Sign-Launcher.ps1` (signtool du SDK Windows, présent avec les Build Tools), à partir d'un PFX ou d'un certificat du magasin utilisateur.
Prérequis : **un certificat de signature de code**. Aucun n'est présent sur le poste (2026-10-14). Options :
1. certificat OV/EV d'une autorité reconnue (Certum, Sectigo, DigiCert…) ou Azure Trusted Signing : pas d'avertissement SmartScreen après acquisition de réputation ;
2. certificat auto-signé : garantit l'intégrité et l'auteur pour un usage personnel, mais Windows affiche l'avertissement « éditeur inconnu ».
La signature s'applique à l'exécutable du launcher et à l'installateur (Phase 6).

**État au 2026-10-14 (décision utilisateur : auto-signé pour l'instant)** : certificat `CN=HomeCraft Edition Vanilla (auto-signé)`, RSA 3072, SHA-256, valable jusqu'au 2031-10-14, clé privée dans le magasin `Cert:\CurrentUser\My` du poste de build (exportable, **jamais** copiée dans le projet). Empreinte dans `UserData/code-signing-thumbprint.txt`, partie publique dans `Build/HomeCraft-CodeSigning.cer`. Le launcher release est signé et horodaté (DigiCert). Pour supprimer l'avertissement sur une machine donnée, l'utilisateur peut importer le `.cer` dans « Éditeurs approuvés » et « Autorités de certification racines de confiance » (opération manuelle, non faite par l'installateur). Passage à un certificat reconnu : relancer `Build/Sign-Launcher.ps1` avec le nouveau PFX ou l'empreinte.

## Écran principal (cahier des charges §7)

Titre « HOMECRAFT EDITION VANILLA ». États : Client, Base de données, Serveur, PlayerBots.
Boutons : JOUER, ARRÊTER LE SERVEUR, VÉRIFIER LES FICHIERS, RÉPARER, SAUVEGARDER, RESTAURER, PARAMÈTRES, JOURNAUX.
JOUER n'est actif qu'après contrôle de santé réel (port 3724 + 8085 en écoute, requête SQL OK, « World initialized » lu sur la console).

## Section « Comptes » (demande utilisateur du 2026-10-14)

- Liste des comptes (`realmd.account` + `account_access`) : nom, niveau GM, dernière connexion, nombre de personnages.
- Créer un compte (nom, mot de passe, niveau GM par défaut 6 pour permettre `.partybot`), changer le mot de passe, supprimer (avec avertissement : supprime les personnages).
- Mise en œuvre :
  - serveur démarré : commandes console `account create`, `account set password`, `account set gmlevel`, `account delete` envoyées sur l'entrée standard de mangosd (validé le 2026-10-14) ;
  - serveur arrêté : calcul SRP6 (`v`, `s`) en C++ et écriture directe en base. Algorithme à reprendre de `src/shared/Auth/` de VMaNGOS (commit figé) et à valider par un test de connexion client.
- Le compte créé à l'installation est proposé automatiquement à l'écran de connexion (`Config.wtf` : `accountName`).

## Onglets « PlayerBots » et « Monde » (demandes utilisateur des 2026-10-14 et 2026-10-15)

Les deux onglets reposent sur un **catalogue** (`Launcher/src/conf_catalog.rs`) : clé exacte de `mangosd.conf`, libellé et aide en français, type (booléen, entier borné, réel borné, choix, texte), valeur par défaut du modèle VMaNGOS. Les valeurs modifiées sont stockées dans `UserData/settings.json` (`conf`) et appliquées à la génération de `mangosd.conf` ; seules les clés du catalogue sont surchargeables, jamais les clés gérées (ports, base, chemins). Chaque réglage modifié est signalé et peut revenir au défaut ; un bouton remet tout aux valeurs Vanilla.

- **PlayerBots** (entre Comptes et Monde) : compagnons (`PartyBot.MaxBots`, `SkipChecks`, `AutoEquip`, `RandomGearLevelDifference`), **bots de peuplement** (`RandomBot.Enable/MinBots/MaxBots/Refresh` : nombre de bots connectés sur le serveur), champs de bataille (`BattleBot.AutoJoin/AutoEquip`), général (`PlayerBot.UpdateMs/ShowInWhoList/AllowSaving/Debug`), plus l'aide-mémoire des commandes `.partybot`. Limite vérifiée dans le code VMaNGOS (`PlayerBotMgr.cpp`, `PlayerBotAI.cpp`) : les bots de peuplement sont tirés dans la table `characters.playerbot` (personnage + IA : `StormwindPopulationAI`, `IronforgePopulationAI`, `OrgrimmarPopulationAI`, `PlayerBotFleeingAI`, sinon IA vide) ; sans personnages bots enregistrés, le réglage n'a aucun effet. L'onglet affiche le nombre de personnages bots disponibles et permet de **générer des personnages bots** (`Launcher/src/botgen.rs`, 2026-10-15) : compte dédié `HOMECRAFTBOTS` (sans mot de passe, locale frFR), personnages moitié Alliance / moitié Horde avec races et classes valides, sexe et apparence aléatoires, niveau 1 à 56 centré vers 25, position et point de rappel de `playercreateinfo`, objets de départ de `playercreateinfo_item` équipés selon leur type (lignes `item_instance` + `character_inventory`, cache d'équipement), entrée `playerbot` avec l'IA de peuplement selon la race (Hurlevent : humains et elfes ; Forgefer : nains et gnomes ; Orgrimmar : Horde). Noms générés par syllabes, uniques. Les réglages `RandomBot.*` sont ajustés automatiquement à la génération ; si le serveur tourne, `bot reload` est envoyé à la console. Suppression de tous les personnages bots avec confirmation. Test du 2026-10-15 : 20 personnages créés (145 objets), serveur démarré avec 10 à 20 bots demandés : 12 connectés à t+60 s, 15 à t+140 s.
- **Monde** : règles (patch de contenu `WowPatch`, type de royaume, message du jour, niveau et argent de départ, cinématiques, instances, affichage des quêtes, métiers, guildes), expérience et progression (`Rate.XP.*`, repos, talents, réputation, métiers, minerai, partage d'XP), butin et économie (`Rate.Drop.*`, courrier), difficulté des créatures (`Rate.Creature.*`, agression, faiblesse de résurrection, déserteur), factions et social (`AllowTwoSide.*`), maître de jeu (`GM.LoginState`, `GM.Visible`, `GM.CheatGod`, `GM.AllowTrades`), technique (distances d'affichage, intervalle de sauvegarde, déchargement des instances, annonces). 70 clés, toutes vérifiées présentes dans le modèle du commit figé (la génération échoue explicitement si une clé disparaît).

## Suppression de compte (demande utilisateur du 2026-10-15)

Bouton « Supprimer » sur chaque ligne de la liste des comptes, fenêtre de confirmation rappelant que le compte et **tous ses personnages** (objets, courrier, progression) sont supprimés définitivement, avec avertissement supplémentaire si c'est le compte connecté le plus récemment. Refusé pendant que le jeu tourne. Serveur démarré : commande console `account delete` (le serveur supprime les personnages). Serveur arrêté : `MariaDb::delete_account_offline` supprime les personnages dans toutes les tables de `characters` ayant une colonne `guid`, dans `item_instance` (owner_guid) et `playerbot`, puis le compte (`account`, `account_access`, `account_banned`, `realmcharacters`). Aucun personnage orphelin n'est laissé.

## Section « AddOns » dans Paramètres (demande utilisateur du 2026-10-15)

`Launcher/src/addons.rs` : liste de `Client/Interface/AddOns` (dossier, titre, version et Interface lus dans le `.toc`, état), ajout depuis une archive zip avec détection du ou des dossiers d'AddOn (fichier `Nom.toc` ; archive GitHub `Nom-master/` renommée ; bibliothèques imbriquées ignorées comme AddOns distincts), désactivation par renommage `.off`, suppression avec confirmation, ouverture du dossier, avertissement si `Interface ≠ 11200`. Refusé pendant que le jeu tourne. Aucun AddOn fourni. Test du 2026-10-15 : archive `TestAddon-master.zip` (toc à la racine du dossier + Libs/LibStub avec son propre toc) → installée dans `AddOns/TestAddon` avec sa bibliothèque, sans dossier parasite.

## Explorateur de base de données natif (demande utilisateur du 2026-10-15)

Demande : intégrer le lecteur SQL de HeidiSQL dans l'onglet. HeidiSQL étant écrit en Delphi, son code n'est pas réutilisable dans un launcher Rust ; l'onglet a donc été réécrit nativement sur le modèle de HeidiSQL : arbre des quatre bases et de leurs tables, onglet **Données** (grille paginée par 200 lignes, filtre WHERE, tri par clic sur l'en-tête, **édition de cellule** par double-clic validée par Entrée, insertion de ligne par formulaire, suppression de ligne avec confirmation ; tables sans clé primaire en lecture seule), onglet **Structure** (colonnes, types, NULL, clé, défaut, extra), onglet **Requête SQL** (lecture immédiate ; écriture après confirmation ; historique). Toute écriture est journalisée dans `Logs/launcher.log`. Si la base est déjà en service au lancement du launcher, l'explorateur s'y attache sans en prendre la propriété.

Correctif HeidiSQL (bouton « Ouvrir dans HeidiSQL ») : la connexion automatique échouait à cause d'un nom de session inexistant (`-d=…`) et de guillemets échappés autour de l'adresse ; arguments corrigés et vérifiés (connexion visible côté MariaDB).

## Constructeur d'archives MPQ (demande utilisateur du 2026-10-15)

Les patchs VanillaHD sont distribués comme arborescences de fichiers. Python n'étant pas installé sur le poste, le constructeur est intégré au launcher (`Launcher/src/mpqbuild.rs`, bibliothèque `wow-mpq` MIT/Apache-2.0) : bouton « Créer un patch depuis un dossier… » de l'onglet Patchs (ou `--do mpqbuild <dossier>`), format MPQ **V1** du client 1.12.1, secteurs de 4 Ko, compression zlib, liste de fichiers générée, chemins internes avec « \ », fichiers de service du dépôt ignorés, descente automatique dans le dossier unique « Patch-X-main ». Nom choisi automatiquement (prochain `patch-X.MPQ` libre), relecture de contrôle après construction.

## Compléments de robustesse (demande utilisateur du 2026-10-15, soir)

Quatre manques relevés lors de la revue « le launcher est-il complet ? » et livrés le jour même :

1. **Cartes de déplacement (mmaps) en arrière-plan** : si elles manquent après le démarrage du serveur, `MoveMapGenerator` est lancé en tâche de fond sur la moitié des cœurs (réglage « Générer les cartes de déplacement » de Paramètres, activé par défaut). L'accueil affiche l'état (absentes / génération en cours depuis N min / générées, actives au prochain démarrage / présentes). Un marqueur `Server/Data/mmaps/.generation-en-cours` est posé pendant la génération : s'il reste (launcher fermé avant la fin), les mmaps sont considérées absentes et la génération reprend au démarrage suivant. Le bouton « Générer les mmaps » de Paramètres utilise le même mécanisme (`--do mmaps`).
2. **Sauvegarde automatique** à chaque fermeture du jeu (libellé « automatique à la fermeture du jeu »), avec rotation : seules les N plus récentes de ce libellé sont conservées (N réglable, 10 par défaut) ; les sauvegardes manuelles et de sécurité ne sont jamais touchées par la rotation.
3. **Réimport de la base monde** (bouton « Réimporter la base monde… » de Paramètres, `--do reimportworld`) : supprime et recrée `mangos` depuis les dumps du package, compléments `Server/Database/custom/*.sql` compris, sans toucher à `characters` ni `realmd`. Confirmation préalable, serveur de jeu arrêté avant, refus si le jeu est lancé. Usage : suivre une nouvelle version de VMaNGOS ou annuler des modifications faites dans l'explorateur de base.
4. **Journalisation des imports** : chaque étape d'import (bases et compléments) est désormais tracée dans `Logs/launcher.log` (« import : complément locales_quest_homecraft.sql »), ce que `Test-Package` et `Test-Phase5` vérifient.

## Section « Base de données » (demande utilisateur du 2026-10-14)

Objectif : un explorateur de base intégré au launcher, « comme HeidiSQL ».

Constat vérifié le 2026-10-14 : HeidiSQL est sous **GPL-2.0**, écrit en **Delphi 12.1** (composants Delphi SynEdit, VirtualTreeView ; icônes Icons8 sous permission spéciale limitée à HeidiSQL). Ses composants ne sont donc **pas intégrables** dans une application C++/Qt. **Décision utilisateur (2026-10-14) : les deux voies sont retenues et seront livrées ensemble.**

1. **Explorateur natif Qt intégré** (QSqlDatabase + pilote QMARIADB/QMYSQL de Qt, client MariaDB Connector/C en LGPL-2.1) :
   - arbre des 4 bases et de leurs tables, grille éditable (`QSqlTableModel`), filtre, tri, pagination ;
   - éditeur SQL avec exécution et affichage des résultats, historique ;
   - raccourcis utiles : personnages, comptes, enchères, objets ;
   - garde-fous : confirmation avant toute écriture, pas d'écriture sur `mangos` (monde) sans mode avancé, refus si le serveur est en cours d'arrêt.
2. **Bouton « Ouvrir dans HeidiSQL »** : HeidiSQL portable redistribué tel quel dans `Tools/HeidiSQL/` (GPL-2.0 : joindre la licence et le lien vers les sources de la version exacte), lancé avec la connexion préremplie (paramètres de ligne de commande `--host/--port/--user/--password` à vérifier sur la version figée). Processus séparé : aucune contrainte de liaison de code.

Le lecteur intégré doit refléter l'état réel (connexion au port 3315 local), jamais une copie.

## Autres sections déjà prévues

- Paramètres : faction de l'hôtel des ventes (AHBot), rates XP/or/réputation, fenêtré/plein écran, chemin du client.
- Sauvegardes : `mariadb-dump --single-transaction` de `characters` et `realmd`, horodatées, conservées hors de `Server/`.
- Diagnostic : versions des composants (manifeste), ports, derniers journaux, test de connexion base.
