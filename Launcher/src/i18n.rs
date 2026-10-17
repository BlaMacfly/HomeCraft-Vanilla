//! Textes de l'interface. Un seul point de maintenance : aucune chaîne visible n'est écrite ailleurs.
//! Les messages techniques venant des serveurs (console worldserver, SQL) restent tels quels dans les journaux.

pub mod fr {
    pub const TITLE: &str = "HOMECRAFT EDITION WOTLK";
    pub const WINDOW_TITLE: &str = "HomeCraft Edition WoTLK";

    // Composants
    pub const COMP_CLIENT: &str = "Client";
    pub const COMP_DB: &str = "Base de données";
    pub const COMP_SERVER: &str = "Serveur";
    pub const COMP_BOTS: &str = "PlayerBots";
    pub const COMP_PROXY: &str = "HermesProxy (client 3.4.3)";
    pub const CLIENT_OK: &str = "vérifié";
    pub const CLIENT_REPAIR: &str = "réparation nécessaire";
    pub const CLIENT_MISSING: &str = "introuvable";
    pub const DB_STOPPED: &str = "arrêtée";
    pub const DB_RUNNING: &str = "opérationnelle";
    pub const SERVER_STOPPED: &str = "arrêté";
    pub const SERVER_STARTING: &str = "démarrage";
    pub const SERVER_RUNNING: &str = "opérationnel";
    pub const BOTS_ON: &str = "activés";
    pub const BOTS_OFF: &str = "désactivés";
    pub const PROXY_MISSING: &str = "non installé (Tools/HermesProxy)";
    pub const PROXY_READY: &str = "installé";
    pub const PROXY_RUNNING: &str = "en service";

    // Boutons
    pub const BTN_PLAY: &str = "JOUER";
    pub const BTN_STOP: &str = "ARRÊTER LE SERVEUR";
    pub const BTN_VERIFY: &str = "VÉRIFIER LES FICHIERS";
    pub const BTN_REPAIR: &str = "RÉPARER";
    pub const BTN_BACKUP: &str = "SAUVEGARDER";
    pub const BTN_RESTORE: &str = "RESTAURER";
    pub const BTN_SETTINGS: &str = "PARAMÈTRES";
    pub const BTN_LOGS: &str = "JOURNAUX";
    pub const BTN_START_SERVER: &str = "DÉMARRER LE SERVEUR";

    // Onglets
    pub const TAB_HOME: &str = "Accueil";
    pub const TAB_ACCOUNTS: &str = "Comptes";
    pub const TAB_DATABASE: &str = "Base de données";
    pub const TAB_SETTINGS: &str = "Paramètres";
    pub const TAB_LOGS: &str = "Journaux";

    // États
    pub const STATE_NOT_INSTALLED: &str = "NON INSTALLÉ";
    pub const STATE_INSTALLED: &str = "INSTALLÉ";
    pub const STATE_VERIFYING: &str = "VÉRIFICATION";
    pub const STATE_INITIALIZING: &str = "INITIALISATION";
    pub const STATE_STARTING_DB: &str = "DÉMARRAGE BASE DE DONNÉES";
    pub const STATE_STARTING_AUTH: &str = "DÉMARRAGE AUTHENTIFICATION";
    pub const STATE_STARTING_WORLD: &str = "DÉMARRAGE SERVEUR DE JEU";
    pub const STATE_READY: &str = "PRÊT";
    pub const STATE_PLAYING: &str = "JEU EN COURS";
    pub const STATE_STOPPING: &str = "ARRÊT EN COURS";
    pub const STATE_ERROR: &str = "ERREUR";
    pub const STATE_REPAIRING: &str = "RÉPARATION";

    // Messages de progression
    pub const MSG_IDLE: &str = "Cliquez sur JOUER pour démarrer le serveur local et lancer le jeu.";
    pub const MSG_VERIFY: &str = "Vérification des fichiers…";
    pub const MSG_INIT: &str = "Préparation de la base de données locale…";
    pub const MSG_DB: &str = "Démarrage de la base de données…";
    pub const MSG_DB_IMPORT: &str = "Première installation : import des données du royaume (deux à cinq minutes)…";
    pub const MSG_AUTH: &str = "Démarrage du serveur d'authentification…";
    pub const MSG_WORLD: &str = "Chargement du monde (une à deux minutes, puis connexion des bots)…";
    pub const MSG_PROXY: &str = "Démarrage de HermesProxy (passerelle pour le client 3.4.3)…";
    pub const MSG_READY: &str = "Serveur opérationnel. Vous pouvez jouer.";
    pub const MSG_PLAYING: &str = "Le jeu est lancé. Le serveur reste actif tant que cette fenêtre est ouverte.";
    pub const MSG_STOPPING: &str = "Arrêt propre des services…";
    pub const MSG_STOPPED: &str = "Services arrêtés.";
    pub const MSG_CLOSING: &str = "Fermeture : arrêt des services en cours…";
    pub const MSG_NOT_INSTALLED: &str = "Des composants sont absents : utilisez RÉPARER ou réinstallez.";
    pub const MSG_DBC_OUTDATED: &str = "Les DBC extraits de ce client ne sont pas au niveau 3.3.5a (Map.dbc sans le sanctum Rubis) : ce client frFR a été recomposé avec des données 3.3.3a. Fournissez un dossier Server/Data/dbc complet (enUS 3.3.5a) ou un autre client.";

    // Comptes
    pub const ACC_TITLE: &str = "Comptes de jeu";
    pub const ACC_USER: &str = "Identifiant";
    pub const ACC_PASS: &str = "Mot de passe";
    pub const ACC_GM: &str = "Niveau GM";
    pub const ACC_LAST: &str = "Dernière connexion";
    pub const ACC_CREATE: &str = "Créer le compte";
    pub const ACC_REFRESH: &str = "Actualiser";
    pub const ACC_NEED_SERVER: &str = "Le serveur de jeu doit être opérationnel pour créer un compte ou changer un mot de passe.";
    pub const ACC_HINT: &str = "Niveaux GM d'AzerothCore : 0 joueur, 1 modérateur, 2 maître de jeu, 3 administrateur. Les commandes des bots (.playerbots) sont accessibles aux joueurs ; le niveau 3 donne toutes les commandes serveur. Les comptes de bots aléatoires (rndbot…) ne sont pas listés.";
    pub const ACC_SET_PASS: &str = "Changer le mot de passe";
    pub const ACC_SET_GM: &str = "Appliquer le niveau GM";
    pub const ACC_DELETE: &str = "Supprimer";
    pub const ACC_DELETE_CONFIRM_TITLE: &str = "Supprimer définitivement ce compte ?";
    pub const ACC_DELETE_CONFIRM: &str = "Le compte et TOUS ses personnages (objets, courrier, progression) seront supprimés définitivement. Cette action est irréversible : faites d'abord une sauvegarde (onglet Sauvegardes) si vous avez un doute.";
    pub const ACC_DELETE_PLAYING: &str = "Impossible de supprimer un compte pendant que le jeu est lancé.";
    pub const ACC_DELETED: &str = "Compte supprimé :";
    pub const ACC_DELETE_FAIL: &str = "Suppression impossible";
    pub const ACC_DELETE_SELF_WARN: &str = "Attention : ce compte est celui qui s'est connecté le plus récemment.";

    // Base de données
    pub const DB_TITLE: &str = "Explorateur de base de données";
    pub const DB_OPEN_HEIDI: &str = "Ouvrir dans HeidiSQL";
    pub const DB_HEIDI_MISSING: &str = "HeidiSQL n'est pas installé dans Tools/HeidiSQL.";
    pub const DB_OPEN_KEIRA: &str = "Ouvrir dans Keira3";
    pub const DB_KEIRA_HINT: &str = "Keira3 : éditeur spécialisé de la base monde AzerothCore (créatures, objets, quêtes, butins, scripts SmartAI, conditions…), avec génération du SQL. Version 3.10.5 compilée depuis ses sources, alignée sur le schéma de votre base. Ses valeurs de connexion pré-remplies sont celles de la base locale : cliquez simplement sur Connect.";
    pub const DB_KEIRA_MISSING: &str = "Keira3 n'est pas installé dans Tools/Keira3 (Keira3.exe).";
    pub const DB_KEIRA_OPENED: &str = "Keira3 ouvert : les valeurs pré-remplies (127.0.0.1, port 3306, root / root, acore_world) sont les bonnes, cliquez sur Connect. Si une ancienne connexion enregistrée s'affiche, remplacez-la par ces valeurs.";
    pub const DB_CONN_TITLE: &str = "Connexion à la base locale (HeidiSQL, Keira3 ou tout client MySQL)";
    pub const DB_CONN_HOST: &str = "Hôte";
    pub const DB_CONN_PORT: &str = "Port";
    pub const DB_CONN_USER: &str = "Identifiant";
    pub const DB_CONN_PASS: &str = "Mot de passe";
    pub const DB_CONN_BASES: &str = "Bases";
    pub const DB_CONN_COPY: &str = "Copier";
    pub const DB_CONN_HINT: &str = "Accessible uniquement depuis ce PC (127.0.0.1). Le serveur de jeu utilise son propre compte SQL (acore), indépendant de root.";
    pub const DB_NEED_DB: &str = "La base de données doit être démarrée.";
    pub const DB_BASE: &str = "Base";
    pub const DB_TABLE: &str = "Table";
    pub const DB_LOAD: &str = "Afficher (200 premières lignes)";
    pub const DB_SQL: &str = "Requête SQL (lecture seule : SELECT / SHOW / DESCRIBE)";
    pub const DB_RUN: &str = "Exécuter";
    pub const DB_ROWS: &str = "lignes";
    pub const DB_READONLY_ONLY: &str = "Cette instruction n'est pas une lecture : utilisez « Exécuter (écriture) » après vérification.";
    pub const DB_INTRO: &str = "Explorateur intégré : choisissez une table dans l'arbre ; double-cliquez une cellule pour la modifier (Entrée valide, Échap annule), insérez ou supprimez des lignes. Les tables sans clé primaire sont en lecture seule. Toute écriture est journalisée.";
    pub const DB_TAB_DATA: &str = "Données";
    pub const DB_TAB_STRUCTURE: &str = "Structure";
    pub const DB_TAB_SQL: &str = "Requête SQL";
    pub const DB_FILTER: &str = "Filtre (clause WHERE)";
    pub const DB_APPLY_FILTER: &str = "Appliquer";
    pub const DB_PREV: &str = "◀ Précédent";
    pub const DB_NEXT: &str = "Suivant ▶";
    pub const DB_INSERT: &str = "Insérer une ligne";
    pub const DB_DELETE_ROW: &str = "Supprimer la ligne";
    pub const DB_DELETE_ROW_CONFIRM_TITLE: &str = "Supprimer cette ligne ?";
    pub const DB_DELETE_ROW_CONFIRM: &str = "La ligne sera supprimée définitivement de la table.";
    pub const DB_NO_PK: &str = "Table sans clé primaire : modification désactivée dans la grille (utilisez l'onglet Requête SQL ou HeidiSQL).";
    pub const DB_RUN_READ: &str = "Exécuter (lecture)";
    pub const DB_RUN_WRITE: &str = "Exécuter (écriture)";
    pub const DB_WRITE_CONFIRM_TITLE: &str = "Exécuter cette instruction d'écriture ?";
    pub const DB_WRITE_CONFIRM: &str = "Elle modifie la base de manière définitive. Faites une sauvegarde en cas de doute.";
    pub const DB_AFFECTED: &str = "ligne(s) affectée(s)";
    pub const DB_HISTORY: &str = "Historique";
    pub const DB_COL_NAME: &str = "Colonne";
    pub const DB_COL_TYPE: &str = "Type";
    pub const DB_COL_NULL: &str = "NULL";
    pub const DB_COL_KEY: &str = "Clé";
    pub const DB_COL_DEFAULT: &str = "Défaut";
    pub const DB_INSERT_TITLE: &str = "Nouvelle ligne";
    pub const DB_INSERT_HINT: &str = "Laissez vide pour utiliser la valeur par défaut ; tapez NULL pour une valeur nulle.";
    pub const DB_TOTAL: &str = "lignes au total";

    // Paramètres
    pub const SET_CLIENT_DIR: &str = "Dossier du client World of Warcraft";
    pub const SET_CLIENT_HINT: &str = "Vide = dossier Client/ de l'installation. Deux clients possibles : 3.3.5a (Data/frFR + Wow.exe d'origine, ou client open source wowdecomp WowClient64.exe / Wow64.exe en 64 bits, choisi en priorité) ou WotLK Classic 3.4.3 (dossier _classic_ + Arctium WoW Launcher.exe, via HermesProxy). Le client n'est jamais téléchargé : fournissez le vôtre.";
    pub const SET_CLIENT_KIND: &str = "Client à lancer";
    pub const SET_CLIENT_AUTO: &str = "Automatique";
    pub const SET_CLIENT_335: &str = "3.3.5a (connexion directe)";
    pub const SET_CLIENT_343: &str = "WotLK Classic 3.4.3 (HermesProxy + Arctium)";
    pub const SET_WINDOWED: &str = "Jeu en mode fenêtré (recommandé : changez le mode ici plutôt que dans les options du jeu)";
    pub const SET_WINDOWED_HINT: &str = "Le launcher borne aussi la résolution du client 3.3.5a à votre écran (écran noir sinon en fenêtré à la résolution exacte de l'écran).";
    pub const BTN_KILL_GAME: &str = "FERMER LE JEU";
    pub const BTN_KILL_GAME_HINT: &str = "Force la fermeture du jeu (écran noir, jeu figé). Les progrès non enregistrés par le serveur depuis la dernière sauvegarde automatique sont perdus.";
    pub const SET_REALM_NAME: &str = "Nom du royaume affiché dans le jeu";
    pub const SET_AHBOT: &str = "Hôtel des ventes approvisionné (mod-ah-bot, vendeur)";
    pub const SET_AHBOT_HINT: &str = "Le module exige un compte et un personnage « vendeur » existants : indiquez l'identifiant du compte (acore_auth.account.id) et le GUID du personnage (acore_characters.characters.guid). Créez-les dans le jeu, puis relevez les numéros dans l'onglet Base de données.";
    pub const SET_AHBOT_ACCOUNT: &str = "Compte vendeur (id)";
    pub const SET_AHBOT_GUID: &str = "Personnage vendeur (guid)";
    pub const SET_HERMES_ARGS: &str = "Arguments supplémentaires de HermesProxy";
    pub const SET_HERMES_HINT: &str = "Le launcher passe déjà --set ServerAddress=127.0.0.1, ServerPort=3724 et ServerBuild=12340. Ajoutez ici d'autres surcharges (--Section:Clé=Valeur) si votre appsettings.json n'est pas celui livré.";
    pub const SET_SAVE: &str = "Enregistrer (appliqué au prochain démarrage du serveur)";
    pub const SET_SAVED: &str = "Réglages enregistrés.";

    // Journaux / diagnostic
    pub const LOG_TITLE: &str = "Journal du launcher et console du serveur";
    pub const LOG_CONSOLE: &str = "Commande console worldserver";
    pub const LOG_SEND: &str = "Envoyer";
    pub const LOG_DIAG: &str = "Diagnostic";

    // Sauvegardes
    pub const TAB_BACKUPS: &str = "Sauvegardes";
    pub const BK_TITLE: &str = "Sauvegardes des personnages et des comptes";
    pub const BK_HINT: &str = "Chaque sauvegarde contient les bases acore_characters et acore_auth (export cohérent, tables verrouillées), vos réglages et un manifeste d'empreintes. Dossier : UserData/Sauvegardes, conservé lors des mises à jour. Les personnages des bots aléatoires en font partie.";
    pub const BK_NOW: &str = "Sauvegarder maintenant";
    pub const BK_RESTORE_SEL: &str = "Restaurer la sélection";
    pub const BK_DELETE_SEL: &str = "Supprimer la sélection";
    pub const BK_OPEN_DIR: &str = "Ouvrir le dossier";
    pub const BK_DIAG: &str = "Exporter un diagnostic";
    pub const BK_DATE: &str = "Date";
    pub const BK_LABEL: &str = "Libellé";
    pub const BK_CHARS: &str = "Personnages";
    pub const BK_ACCOUNTS: &str = "Comptes";
    pub const BK_SIZE: &str = "Taille";
    pub const BK_NONE: &str = "Aucune sauvegarde pour l'instant.";
    pub const BK_PLAYING_WARN: &str = "Le jeu est en cours : les progrès non encore enregistrés par le serveur sont d'abord forcés (saveall), puis exportés.";
    pub const BK_RESTORE_CONFIRM_TITLE: &str = "Confirmer la restauration";
    pub const BK_RESTORE_CONFIRM: &str = "Les personnages et comptes actuels seront REMPLACÉS par ceux de cette sauvegarde. Une sauvegarde de sécurité de l'état actuel est créée avant. Le serveur de jeu sera arrêté.";
    pub const BK_DELETE_CONFIRM_TITLE: &str = "Confirmer la suppression";
    pub const BK_DELETE_CONFIRM: &str = "Cette sauvegarde sera définitivement supprimée du disque.";
    pub const BK_RESTORE_PLAYING: &str = "Impossible de restaurer pendant que le jeu est lancé : fermez le jeu d'abord.";
    pub const CONFIRM: &str = "Confirmer";
    pub const CANCEL: &str = "Annuler";
    pub const CLOSE: &str = "Fermer";

    // Messages
    pub const MSG_BACKUP: &str = "Sauvegarde en cours…";
    pub const MSG_BACKUP_DONE: &str = "Sauvegarde terminée.";
    pub const MSG_RESTORE: &str = "Restauration en cours…";
    pub const MSG_RESTORE_DONE: &str = "Restauration terminée. Vous pouvez redémarrer le serveur.";
    pub const MSG_VERIFYING_FILES: &str = "Vérification des fichiers gérés…";
    pub const MSG_REPAIR: &str = "Réparation en cours…";
    pub const MSG_REPAIR_DONE: &str = "Réparation terminée : voir le rapport.";
    pub const MSG_EXTRACT: &str = "Extraction des données du client…";
    pub const MSG_DIAG_DONE: &str = "Diagnostic exporté : ";

    // Rapport de vérification
    pub const RPT_TITLE: &str = "Rapport de vérification";
    pub const RPT_MANIFEST_MISSING: &str = "Manifeste Build/files-manifest.json absent : seule la présence des composants a été contrôlée.";
    pub const RPT_FILES_OK: &str = "Fichiers gérés : tous présents et conformes";
    pub const RPT_MISSING: &str = "Fichiers manquants";
    pub const RPT_ALTERED: &str = "Fichiers altérés";
    pub const RPT_DATA: &str = "Données extraites du client";
    pub const RPT_DB: &str = "Bases de données";
    pub const RPT_DB_OK: &str = "quatre bases présentes, schéma complet";
    pub const RPT_DB_UNKNOWN: &str = "non contrôlées (base arrêtée)";
    pub const RPT_UNREPAIRABLE: &str = "Fichiers manquants ou altérés que RÉPARER ne peut pas reconstituer : réinstallez le package (vos personnages et sauvegardes sont conservés).";
    pub const RPT_REPAIR_HINT: &str = "RÉPARER régénère les configurations, réimporte les bases manquantes (jamais les personnages) et ré-extrait les données du client si nécessaire.";
    pub const BTN_MMAPS: &str = "Générer les mmaps (~20 min)";

    // PlayerBots
    pub const TAB_BOTS: &str = "PlayerBots";
    pub const BOT_TITLE: &str = "PlayerBots (mod-playerbots)";
    pub const BOT_INTRO: &str = "Des bots aléatoires peuplent le monde (ils questent, voyagent, font des donjons et des champs de bataille) et peuvent rejoindre votre groupe. Le serveur crée lui-même leurs comptes (rndbot…) et personnages au premier démarrage. Réglages appliqués au prochain démarrage du serveur.";
    pub const BOT_COUNT: &str = "Nombre de bots aléatoires connectés en permanence";
    pub const BOT_COUNT_HINT: &str = "0 = aucun bot. Chaque bot coûte de la mémoire et du processeur : 20 à 60 conviennent à un PC de jeu ; le serveur crée 10 bots par compte rndbot (plus 2 comptes de marge). Les mmaps sont indispensables à leurs déplacements.";
    pub const BOT_SAVE: &str = "Enregistrer les réglages PlayerBots";
    pub const BOT_CMDS: &str = "Commandes à taper dans le chat du jeu";
    pub const BOT_POOL: &str = "Comptes et personnages de bots existants";
    pub const BOT_POOL_NONE: &str = "aucun pour l'instant : ils seront créés au prochain démarrage du serveur si le nombre de bots est supérieur à 0.";
    pub const BOTPOOL_DELETE: &str = "Supprimer tous les bots aléatoires";
    pub const BOTPOOL_DELETE_CONFIRM_TITLE: &str = "Supprimer les bots aléatoires";
    pub const BOTPOOL_DELETE_CONFIRM: &str = "Tous les comptes rndbot… et leurs personnages seront supprimés (le serveur les recréera au prochain démarrage). Vos propres comptes et personnages ne sont pas concernés. Le serveur doit être arrêté.";
    pub const BOTPOOL_DELETE_RUNNING: &str = "Arrêtez le serveur avant de supprimer les bots.";

    // Modules
    pub const TAB_MODULES: &str = "Modules";
    pub const MOD_TITLE: &str = "Modules du cœur (AzerothCore / Sylvania)";
    pub const MOD_INTRO: &str = "Chaque module se désactive indépendamment : la case écrit sa clé d'activation dans son fichier de configuration (Server/acore/configs/modules/). « Ouvrir » affiche ce fichier dans votre éditeur de texte pour les réglages fins ; vos modifications sont conservées d'un démarrage à l'autre (le launcher n'y touche que ses propres clés : base de données, nombre de bots, AHBot). « Modèle » rétablit la configuration d'origine du module.";
    pub const MOD_RESTART_HINT: &str = "Modification enregistrée : elle sera appliquée au prochain démarrage du serveur.";
    pub const MOD_OPEN: &str = "Ouvrir";
    pub const MOD_OPEN_HINT: &str = "Ouvre le fichier de configuration du module dans l'éditeur de texte associé aux .conf (Bloc-notes sinon).";
    pub const MOD_RESET: &str = "Modèle";
    pub const MOD_RESET_HINT: &str = "Rétablit la configuration d'origine du module (toutes vos modifications de ce fichier sont perdues).";
    pub const MOD_RESET_CONFIRM_TITLE: &str = "Rétablir la configuration d'origine ?";
    pub const MOD_RESET_CONFIRM: &str = "Le fichier de configuration du module sera remplacé par le modèle livré. Les réglages du launcher (bots, AHBot) seront réappliqués au prochain démarrage.";
    pub const MOD_RESET_DONE: &str = "Configuration du module rétablie depuis le modèle.";
    pub const MOD_FAIL: &str = "Modification impossible";
    pub const MOD_ALWAYS_ON: &str = "toujours actif (pas de réglage)";
    pub const MOD_SEE_SETTINGS: &str = "activé dans Paramètres";
    pub const MOD_ON: &str = "activé";
    pub const MOD_OFF: &str = "désactivé";
    pub const MOD_COL_MODULE: &str = "Module";
    pub const MOD_COL_STATE: &str = "État";
    pub const MOD_OPEN_DIR: &str = "Ouvrir le dossier des configurations";
    pub const MOD_PLAYERBOTS_OFF: &str = "Le module PlayerBots est désactivé : aucun bot ne sera chargé, quel que soit le nombre réglé dans l'onglet PlayerBots.";

    // Monde
    pub const TAB_WORLD: &str = "Monde";
    pub const WORLD_TITLE: &str = "Réglages du monde (worldserver.conf)";
    pub const WORLD_INTRO: &str = "Les réglages les plus utiles du serveur de jeu, avec leurs valeurs par défaut (celles du royaume). Les modifications sont appliquées au prochain démarrage du serveur. Les autres clés du fichier restent gérées automatiquement par le launcher.";
    pub const WORLD_SAVE: &str = "Enregistrer les réglages du monde";
    pub const OPT_DEFAULT: &str = "Défaut";
    pub const OPT_RESET_ALL: &str = "Tout remettre aux valeurs par défaut";
    pub const OPT_MODIFIED: &str = "modifié";

    // Patchs client
    pub const TAB_PATCHES: &str = "Patchs";
    pub const PATCH_TITLE: &str = "Patchs du client 3.3.5a (archives MPQ supplémentaires)";
    pub const PATCH_INTRO: &str = "Le client charge, après ses archives officielles, les fichiers patch-X.MPQ de Data/ et patch-frFR-X.MPQ de Data/frFR/ (X : chiffre de 4 à 9 ou lettre de A à Z ; les suivants ont priorité sur les précédents). Vous apportez vos propres patchs (graphismes HD, interface, sons) : le launcher n'en fournit ni n'en télécharge aucun. Désactiver renomme l'archive en .off sans la supprimer.";
    pub const PATCH_SERVER_NOTE: &str = "Important : les outils d'extraction d'AzerothCore ne lisent que les archives officielles. Un patch n'a donc aucun effet sur le serveur : les modifications purement visuelles ou sonores fonctionnent, mais un patch qui ajoute des races, des objets ou des sorts côté client ne sera pas reconnu par le serveur.";
    pub const PATCH_CLASSIC_NOTE: &str = "Le client WotLK Classic 3.4.3 utilise des données CASC : pas de patchs MPQ. Cet onglet concerne le client 3.3.5a.";
    pub const PATCH_NAME: &str = "Archive";
    pub const PATCH_SIZE: &str = "Taille";
    pub const PATCH_STATE: &str = "État";
    pub const PATCH_OFFICIAL: &str = "officielle";
    pub const PATCH_ACTIVE: &str = "actif";
    pub const PATCH_INACTIVE: &str = "désactivé";
    pub const PATCH_NOT_LOADED: &str = "nom non chargé par le client";
    pub const PATCH_ENABLE: &str = "Activer";
    pub const PATCH_DISABLE: &str = "Désactiver";
    pub const PATCH_REMOVE: &str = "Supprimer";
    pub const PATCH_ADD: &str = "Ajouter un patch…";
    pub const PATCH_OPEN_DIR: &str = "Ouvrir le dossier Data";
    pub const PATCH_PLAYING: &str = "Le jeu est lancé : fermez-le avant de modifier les patchs (fichiers en cours d'utilisation).";
    pub const PATCH_ADDED: &str = "Patch ajouté :";
    pub const PATCH_ADD_FAIL: &str = "Ajout impossible";
    pub const PATCH_REMOVE_CONFIRM_TITLE: &str = "Supprimer le patch";
    pub const PATCH_REMOVE_CONFIRM: &str = "L'archive sera définitivement supprimée du client. Préférez « Désactiver » pour la conserver.";
    pub const PATCH_NONE: &str = "Aucun patch supplémentaire : seules les archives officielles sont présentes.";
    pub const PATCH_LOCALE: &str = "(locale)";

    // AddOns
    pub const ADDON_TITLE: &str = "AddOns du client (Interface\\AddOns)";
    pub const ADDON_INTRO: &str = "Installez vos AddOns depuis leur archive zip : le launcher reconnaît le ou les dossiers d'AddOn (fichier Nom.toc) et les place au bon endroit, même si l'archive vient de GitHub (dossier « Nom-master »). Désactiver renomme le dossier en .off ; l'activation finale par personnage se fait dans le jeu. Aucun AddOn n'est fourni ni téléchargé. Client 3.3.5a : AddOns « Interface 30300 » ; client 3.4.3 : AddOns WotLK Classic (« Interface 30403 »).";
    pub const ADDON_ADD: &str = "Ajouter un AddOn (zip)…";
    pub const ADDON_OPEN_DIR: &str = "Ouvrir le dossier AddOns";
    pub const ADDON_NAME: &str = "Dossier";
    pub const ADDON_TITLE_COL: &str = "Titre";
    pub const ADDON_VERSION: &str = "Version";
    pub const ADDON_INTERFACE: &str = "Interface";
    pub const ADDON_STATE: &str = "État";
    pub const ADDON_ACTIVE: &str = "actif";
    pub const ADDON_INACTIVE: &str = "désactivé";
    pub const ADDON_NO_TOC: &str = "sans .toc (ignoré par le jeu)";
    pub const ADDON_OLD_INTERFACE: &str = "Version d'interface différente de celle du client : peut être signalé « obsolète » par le jeu (cocher « Charger les AddOns obsolètes »).";
    pub const ADDON_PLAYING: &str = "Le jeu est lancé : fermez-le avant de modifier les AddOns.";
    pub const ADDON_ADDED: &str = "AddOn installé :";
    pub const ADDON_ADD_FAIL: &str = "Installation impossible";
    pub const ADDON_REMOVE: &str = "Supprimer";
    pub const ADDON_REMOVE_CONFIRM_TITLE: &str = "Supprimer l'AddOn";
    pub const ADDON_REMOVE_CONFIRM: &str = "Le dossier de l'AddOn sera définitivement supprimé (ses réglages dans WTF sont conservés).";
    pub const ADDON_NONE: &str = "Aucun AddOn installé.";

    // Cartes de déplacement (mmaps) en arrière-plan
    pub const SET_AUTO_MMAPS: &str = "Générer les cartes de déplacement (mmaps) en arrière-plan si elles manquent";
    pub const SET_AUTO_MMAPS_HINT: &str = "Lancé après le démarrage du serveur, sur la moitié des cœurs du processeur (environ 20 minutes sur 14 threads). Indispensables aux déplacements des bots ; prises en compte au redémarrage suivant du serveur.";
    pub const HOME_MMAPS: &str = "Cartes de déplacement (mmaps)";
    pub const HOME_MMAPS_PRESENT: &str = "présentes";
    pub const HOME_MMAPS_ABSENT: &str = "absentes (déplacement des bots dégradé)";
    pub const HOME_MMAPS_RUNNING: &str = "génération en cours depuis";
    pub const HOME_MMAPS_DONE_RESTART: &str = "générées : actives au prochain démarrage du serveur";
    pub const HOME_MMAPS_FAILED: &str = "échec de la génération :";
    pub const MMAPS_ALREADY_RUNNING: &str = "La génération des mmaps est déjà en cours.";
    pub const BTN_MMAPS_HINT: &str = "Régénère les cartes de déplacement en arrière-plan.";

    // Sauvegarde automatique
    pub const SET_AUTO_BACKUP: &str = "Sauvegarder automatiquement les personnages à chaque fermeture du jeu";
    pub const SET_AUTO_BACKUP_KEEP: &str = "Sauvegardes automatiques conservées";
    pub const SET_AUTO_BACKUP_HINT: &str = "Les sauvegardes manuelles et de sécurité ne sont jamais supprimées par cette rotation.";
    pub const BK_AUTO_LABEL: &str = "automatique à la fermeture du jeu";

    // Réimport de la base monde
    pub const BTN_WORLD_REIMPORT: &str = "Réimporter la base monde…";
    pub const WORLD_REIMPORT_HINT: &str = "Recrée la base acore_world (quêtes, créatures, objets, hôtel des ventes, traductions) depuis le dump du royaume livré avec le package, sans toucher aux personnages ni aux comptes. Utile après une mise à jour ou pour annuler des modifications faites dans l'explorateur de base.";
    pub const WORLD_REIMPORT_CONFIRM_TITLE: &str = "Réimporter la base monde";
    pub const WORLD_REIMPORT_CONFIRM: &str = "La base monde (acore_world) sera supprimée puis réimportée depuis le package, compléments compris. Les personnages, comptes et sauvegardes sont conservés. Toute modification faite dans la base monde sera perdue. Le serveur de jeu sera arrêté ; l'opération prend une à deux minutes.";
    pub const WORLD_REIMPORT_PLAYING: &str = "Impossible de réimporter la base monde pendant que le jeu est lancé : fermez le jeu d'abord.";
    pub const MSG_WORLD_REIMPORT: &str = "Réimport de la base monde…";
    pub const MSG_WORLD_REIMPORT_DONE: &str = "Base monde réimportée. Vous pouvez redémarrer le serveur.";

    // Support en lecture seule / installation portable
    pub const RO_TITLE: &str = "Ce dossier est en lecture seule (DVD, clé protégée ou dossier protégé).";
    pub const RO_TEXT: &str = "Le jeu a besoin d'écrire sa base de données et les données extraites de votre client. Choisissez un dossier sur votre disque dur : tout le contenu y sera copié, puis le launcher s'ouvrira depuis ce dossier.";
    pub const RO_INSTALL: &str = "Installer sur ce PC…";
    pub const RO_PICK_TITLE: &str = "Choisir le dossier d'installation";
    pub const MSG_INSTALLING: &str = "Copie des fichiers vers le disque…";
    pub const MSG_INSTALLED: &str = "Copie terminée : ouverture du launcher installé.";
    pub const SET_BROWSE: &str = "Parcourir…";
    pub const MSG_CLIENT_NEEDED: &str = "Le client est introuvable ou invalide : indiquez son dossier dans PARAMÈTRES (il n'est jamais fourni par ce package). Pour extraire les données du serveur, un client 3.3.5a est nécessaire.";
    pub const MSG_CLIENT_335_NEEDED_FOR_DATA: &str = "Les données du serveur (cartes, collisions) s'extraient d'un client 3.3.5a : indiquez-en un dans PARAMÈTRES le temps de la première installation, ou fournissez un dossier Server/Data complet.";
    pub const MSG_FIRST_EXTRACT: &str = "Première installation : extraction des données de votre client (cinq à dix minutes)…";
    pub const MSG_HERMES_NEEDED: &str = "Le client 3.4.3 passe par HermesProxy, absent de Tools/HermesProxy. Installez-le (zip Windows de la release, décompressé dans ce dossier) ou utilisez un client 3.3.5a.";
}
