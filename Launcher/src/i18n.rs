//! Textes de l'interface. Un seul point de maintenance : aucune chaîne visible n'est écrite ailleurs.
//! Les messages techniques venant des serveurs (console mangosd, SQL) restent tels quels dans les journaux.

pub mod fr {
    pub const TITLE: &str = "HOMECRAFT EDITION VANILLA";
    pub const WINDOW_TITLE: &str = "HomeCraft Edition Vanilla";

    // Composants
    pub const COMP_CLIENT: &str = "Client";
    pub const COMP_DB: &str = "Base de données";
    pub const COMP_SERVER: &str = "Serveur";
    pub const COMP_BOTS: &str = "PlayerBots";
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
    pub const MSG_DB_IMPORT: &str = "Première installation : import des données du monde (quelques minutes)…";
    pub const MSG_AUTH: &str = "Démarrage du serveur d'authentification…";
    pub const MSG_WORLD: &str = "Chargement du monde…";
    pub const MSG_READY: &str = "Serveur opérationnel. Vous pouvez jouer.";
    pub const MSG_PLAYING: &str = "Le jeu est lancé. Le serveur reste actif tant que cette fenêtre est ouverte.";
    pub const MSG_STOPPING: &str = "Arrêt propre des services…";
    pub const MSG_STOPPED: &str = "Services arrêtés.";
    pub const MSG_CLOSING: &str = "Fermeture : arrêt des services en cours…";
    pub const MSG_NOT_INSTALLED: &str = "Des composants sont absents : utilisez RÉPARER ou réinstallez.";

    // Comptes
    pub const ACC_TITLE: &str = "Comptes de jeu";
    pub const ACC_USER: &str = "Identifiant";
    pub const ACC_PASS: &str = "Mot de passe";
    pub const ACC_GM: &str = "Niveau GM";
    pub const ACC_LAST: &str = "Dernière connexion";
    pub const ACC_CREATE: &str = "Créer le compte";
    pub const ACC_REFRESH: &str = "Actualiser";
    pub const ACC_NEED_SERVER: &str = "Le serveur de jeu doit être opérationnel pour créer un compte (création hors ligne : à venir).";
    pub const ACC_HINT: &str = "Le niveau GM 6 est nécessaire pour invoquer des compagnons (.partybot).";
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
    pub const SET_CLIENT_DIR: &str = "Dossier du client WoW 1.12.1";
    pub const SET_CLIENT_HINT: &str = "Vide = dossier Client/ de l'installation. Deux clients sont reconnus : le client d'origine (WoW.exe 1.12.1, 32 bits) et Benilla (client libre réécrit en Rust, 64 bits). Le client n'est jamais téléchargé : fournissez le vôtre.";
    pub const SET_CLIENT_DETECTED: &str = "Client détecté :";
    pub const SET_CLIENT_MISSING_MPQ: &str = "archives manquantes dans Data :";
    pub const SET_CLIENT_BENILLA_HINT: &str = r"Benilla : le launcher règle benilla-config\config.toml (royaume 127.0.0.1, fenêtré ou plein écran sans bordure, taille de fenêtre) ; les AddOns vont dans benilla-config\AddOns. VanillaFixes et DXVK ne servent pas (rendu natif 64 bits). Le jeu est en français si les MPQ de Data viennent d'un client frFR. N'y mettez pas de patchs chiffrés (patch-3/4/5 de certains repacks) : Benilla ne sait pas les lire. Pour la première installation, les extracteurs du serveur ont besoin de base.MPQ (que Benilla ignore) : laissez-la dans Data, ou gardez un client d'origine à côté de HomeCraft.";
    pub const SET_CLIENT_SCAN: &str = "Rechercher les clients";
    pub const SET_CLIENT_SCAN_HINT: &str = r"Cherche un client valide dans Client\ et dans les dossiers voisins de l'installation (par exemple sur le Bureau).";
    pub const SET_CLIENT_USE: &str = "Utiliser";
    pub const SET_CLIENT_IN_USE: &str = "Utilisé";
    pub const SET_VF_BENILLA: &str = "Sans objet avec Benilla : il a son propre rendu 64 bits (Vulkan / Direct3D 12).";
    pub const EXTRACT_MISSING: &str = "Extraction des données du serveur impossible : il manque";
    pub const EXTRACT_MISSING_HINT: &str = "Les extracteurs du serveur ont besoin de ces archives du client d'origine (Benilla, lui, n'en a pas besoin). Copiez-les depuis votre client 1.12.1 dans ce dossier Data, puis cliquez sur JOUER ou RÉPARER : elles ne gênent pas Benilla.";
    pub const CLIENT_KIND_ORIGINAL: &str = "Client d'origine (WoW.exe)";
    pub const CLIENT_KIND_BENILLA: &str = "Benilla";
    pub const CLIENT_KIND_NONE: &str = "aucun client reconnu";
    pub const SET_WINDOWED: &str = "Jeu en mode fenêtré (recommandé : changez le mode ici plutôt que dans les options du jeu)";
    pub const SET_WINDOWED_HINT: &str = "Le launcher borne aussi la résolution à votre écran : un client 1.12.1 en fenêtré 1920x1080 sur un écran 1920x1080 affiche un écran noir.";
    pub const BTN_KILL_GAME: &str = "FERMER LE JEU";
    pub const BTN_KILL_GAME_HINT: &str = "Force la fermeture de WoW.exe (écran noir, jeu figé). Les progrès non enregistrés par le serveur depuis la dernière sauvegarde automatique sont perdus.";
    pub const SET_AHBOT: &str = "Hôtel des ventes approvisionné (AHBot)";
    pub const SET_AH_FACTION: &str = "Hôtel des ventes servi";
    pub const SET_AH_AUTO: &str = "Automatique (faction du dernier personnage)";
    pub const SET_AH_ALLIANCE: &str = "Alliance";
    pub const SET_AH_HORDE: &str = "Horde";
    pub const SET_BOTS: &str = "Compagnons PlayerBots (commandes .partybot)";
    pub const SET_RATE_XP: &str = "Multiplicateur d'expérience";
    pub const SET_SAVE: &str = "Enregistrer (appliqué au prochain démarrage du serveur)";
    pub const SET_SAVED: &str = "Réglages enregistrés.";

    // Journaux / diagnostic
    pub const LOG_TITLE: &str = "Journal du launcher et console du serveur";
    pub const LOG_CONSOLE: &str = "Commande console mangosd";
    pub const LOG_SEND: &str = "Envoyer";
    pub const LOG_DIAG: &str = "Diagnostic";

    pub const NOT_YET: &str = "À venir";

    // Sauvegardes
    pub const TAB_BACKUPS: &str = "Sauvegardes";
    pub const BK_TITLE: &str = "Sauvegardes des personnages et des comptes";
    pub const BK_HINT: &str = "Chaque sauvegarde contient les bases characters et realmd (export cohérent, tables verrouillées), vos réglages et un manifeste d'empreintes. Dossier : UserData/Sauvegardes, conservé lors des mises à jour.";
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
    pub const MSG_MMAPS: &str = "Génération des mmaps (plusieurs dizaines de minutes à quelques heures)…";
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
    pub const BTN_MMAPS: &str = "Générer les mmaps (long)";

    // PlayerBots
    pub const TAB_BOTS: &str = "PlayerBots";
    pub const BOT_TITLE: &str = "Compagnons PlayerBots";
    pub const BOT_INTRO: &str = "Les compagnons sont créés dans le jeu, à votre niveau, et rejoignent votre groupe. Réglages appliqués au prochain démarrage du serveur.";
    pub const BOT_ENABLE: &str = "Autoriser les compagnons (.partybot)";
    pub const BOT_MAX: &str = "Nombre maximal de compagnons par groupe (0 = illimité, un groupe compte 5 membres)";
    pub const BOT_SKIP: &str = "Lever les restrictions d'invocation (en combat, en donjon, niveau au choix)";
    pub const BOT_EQUIP: &str = "Équipement des nouveaux compagnons";
    pub const BOT_EQUIP_0: &str = "Objets de départ";
    pub const BOT_EQUIP_1: &str = "Équipement aléatoire adapté au niveau";
    pub const BOT_EQUIP_2: &str = "Modèle prédéfini";
    pub const BOT_GEAR_DIFF: &str = "Écart de niveau d'objet toléré (équipement aléatoire)";
    pub const BOT_UPDATE: &str = "Temps de réaction des compagnons (ms)";
    pub const BOT_WHO: &str = "Compagnons visibles dans la liste /qui";
    pub const BOT_BG: &str = "Champs de bataille : compléter automatiquement avec des bots";
    pub const BOT_BG_EQUIP: &str = "Équipement des bots de champ de bataille";
    pub const BOT_SAVE: &str = "Enregistrer les réglages PlayerBots";
    pub const BOT_CMDS: &str = "Commandes à taper dans le chat du jeu (compte de niveau GM 6)";
    pub const BOT_RANDOM_NOTE: &str = "Les « bots aléatoires » qui peupleraient le monde n'ont pas d'intelligence dans VMaNGOS : ils ne sont pas proposés.";

    // Monde
    pub const TAB_WORLD: &str = "Monde";
    pub const WORLD_TITLE: &str = "Réglages du monde (mangosd.conf)";
    pub const WORLD_INTRO: &str = "Les réglages les plus utiles du serveur de jeu, avec leurs valeurs Vanilla par défaut. Les modifications sont appliquées au prochain démarrage du serveur. Les autres clés du fichier restent gérées automatiquement par le launcher.";
    pub const WORLD_SAVE: &str = "Enregistrer les réglages du monde";
    pub const OPT_DEFAULT: &str = "Défaut";
    pub const OPT_RESET_ALL: &str = "Tout remettre aux valeurs Vanilla";
    pub const OPT_MODIFIED: &str = "modifié";
    pub const BOT_POOL: &str = "Personnages bots enregistrés pour le peuplement (table characters.playerbot)";
    pub const BOT_POOL_NONE: &str = "aucun : générez des personnages bots ci-dessous pour que les bots de peuplement aient quelqu'un à connecter.";
    pub const BOTGEN_TITLE: &str = "Personnages bots";
    pub const BOTGEN_HINT: &str = "Crée des personnages (moitié Alliance, moitié Horde, races, classes, niveaux et apparences variés, objets de départ) sur le compte dédié HOMECRAFTBOTS, et les enregistre pour le peuplement des capitales : Hurlevent et Forgefer pour l'Alliance, Orgrimmar pour la Horde. Ils flânent dans la ville, sans combattre.";
    pub const BOTGEN_COUNT: &str = "Nombre de personnages à générer";
    pub const BOTGEN_RUN: &str = "Générer les personnages bots";
    pub const BOTGEN_DELETE: &str = "Supprimer tous les personnages bots";
    pub const BOTGEN_DELETE_CONFIRM_TITLE: &str = "Supprimer les personnages bots";
    pub const BOTGEN_DELETE_CONFIRM: &str = "Tous les personnages du compte HOMECRAFTBOTS seront supprimés définitivement (objets et enregistrements de peuplement compris). Vos propres personnages ne sont pas concernés.";
    pub const BOTGEN_PROGRESS: &str = "Création des personnages bots…";
    pub const BOTGEN_DONE: &str = "personnages bots créés";
    pub const BOTGEN_FAIL: &str = "Génération impossible";
    pub const BOTGEN_AUTO: &str = "Les réglages de peuplement sont ajustés automatiquement (activés, au plus 300 bots connectés à la fois, minimum = la moitié, bots visibles dans /qui) ; ils prennent effet au prochain démarrage du serveur. Les bots se retrouvent dans les capitales : quartier commerçant de Hurlevent, Forgefer, Orgrimmar.";

    // Patchs client
    pub const TAB_PATCHES: &str = "Patchs";
    pub const PATCH_TITLE: &str = "Patchs du client (archives MPQ supplémentaires)";
    pub const PATCH_INTRO: &str = "Le client charge, après ses archives officielles, les fichiers patch-X.MPQ de Client/Data (X : chiffre de 3 à 9 ou lettre de A à Z ; les suivants ont priorité sur les précédents). Vous apportez vos propres patchs (graphismes HD, interface, sons) : le launcher n'en fournit ni n'en télécharge aucun. Désactiver renomme l'archive en .off sans la supprimer.";
    pub const PATCH_SERVER_NOTE: &str = "Important : les outils d'extraction de VMaNGOS ne lisent que les archives officielles. Un patch n'a donc aucun effet sur le serveur : les modifications purement visuelles ou sonores fonctionnent, mais un patch qui ajoute des races, des objets ou des sorts côté client ne sera pas reconnu par le serveur.";
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
    pub const PATCH_REMOVE_CONFIRM: &str = "L'archive sera définitivement supprimée de Client/Data. Préférez « Désactiver » pour la conserver.";
    pub const PATCH_NONE: &str = "Aucun patch supplémentaire : seules les archives officielles sont présentes.";
    pub const PATCH_BUILD: &str = "Créer un patch depuis un dossier…";
    pub const PATCH_BUILD_HINT: &str = "Transforme une arborescence de fichiers (ex. un dépôt VanillaHD « Patch-A-main » décompressé) en archive patch-X.MPQ au format du client 1.12.1. Plusieurs minutes pour un gros patch.";
    pub const PATCH_BUILDING: &str = "Construction de l'archive MPQ…";
    pub const PATCH_BUILT: &str = "Patch créé :";
    pub const PATCH_LOOSE_NOTE: &str = "Les fichiers libres placés dans Data (ex. Data/Spell.dbc) sont aussi lus par le client, en priorité sur les archives ; ils ne sont pas gérés ici.";

    // AddOns
    pub const ADDON_TITLE: &str = "AddOns du client (Interface\\AddOns, ou benilla-config\\AddOns avec Benilla)";
    pub const ADDON_INTRO: &str = "Installez vos AddOns 1.12.1 (Questie, pfUI, Bagnon…) depuis leur archive zip : le launcher reconnaît le ou les dossiers d'AddOn (fichier Nom.toc) et les place au bon endroit, même si l'archive vient de GitHub (dossier « Nom-master »). Désactiver renomme le dossier en .off ; l'activation finale par personnage se fait dans le jeu, bouton AddOns de l'écran de sélection. Aucun AddOn n'est fourni ni téléchargé.";
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
    pub const ADDON_OLD_INTERFACE: &str = "Interface ≠ 11200 : conçu pour une autre version, peut être signalé « obsolète » par le jeu (cocher « Charger les AddOns obsolètes »).";
    pub const ADDON_PLAYING: &str = "Le jeu est lancé : fermez-le avant de modifier les AddOns.";
    pub const ADDON_ADDED: &str = "AddOn installé :";
    pub const ADDON_ADD_FAIL: &str = "Installation impossible";
    pub const ADDON_REMOVE: &str = "Supprimer";
    pub const ADDON_REMOVE_CONFIRM_TITLE: &str = "Supprimer l'AddOn";
    pub const ADDON_REMOVE_CONFIRM: &str = "Le dossier de l'AddOn sera définitivement supprimé (ses réglages dans WTF sont conservés).";
    pub const ADDON_NONE: &str = "Aucun AddOn installé.";

    // Cartes de déplacement (mmaps) en arrière-plan
    pub const SET_AUTO_MMAPS: &str = "Générer les cartes de déplacement (mmaps) en arrière-plan si elles manquent";
    pub const SET_AUTO_MMAPS_HINT: &str = "Lancé après le démarrage du serveur, sur la moitié des cœurs du processeur. Les compagnons suivent et se déplacent correctement en donjon une fois les mmaps en place (prises en compte au redémarrage suivant du serveur).";
    pub const HOME_MMAPS: &str = "Cartes de déplacement (mmaps)";
    pub const HOME_MMAPS_PRESENT: &str = "présentes";
    pub const HOME_MMAPS_ABSENT: &str = "absentes (déplacement des compagnons dégradé)";
    pub const HOME_MMAPS_RUNNING: &str = "génération en cours depuis";
    pub const HOME_MMAPS_DONE_RESTART: &str = "générées : actives au prochain démarrage du serveur";
    pub const HOME_MMAPS_FAILED: &str = "échec de la génération :";
    pub const MMAPS_ALREADY_RUNNING: &str = "La génération des mmaps est déjà en cours.";
    pub const BTN_MMAPS_HINT: &str = "Régénère les cartes de déplacement en arrière-plan (long : de plusieurs dizaines de minutes à quelques heures).";

    // Sauvegarde automatique
    pub const SET_AUTO_BACKUP: &str = "Sauvegarder automatiquement les personnages à chaque fermeture du jeu";
    pub const SET_AUTO_BACKUP_KEEP: &str = "Sauvegardes automatiques conservées";
    pub const SET_AUTO_BACKUP_HINT: &str = "Les sauvegardes manuelles et de sécurité ne sont jamais supprimées par cette rotation.";
    pub const BK_AUTO_LABEL: &str = "automatique à la fermeture du jeu";

    // Réimport de la base monde
    pub const BTN_WORLD_REIMPORT: &str = "Réimporter la base monde…";
    pub const WORLD_REIMPORT_HINT: &str = "Recrée la base mangos (quêtes, créatures, objets, hôtel des ventes, traductions) depuis les fichiers du package, sans toucher aux personnages ni aux comptes. Utile après une mise à jour de VMaNGOS ou pour annuler des modifications faites dans l'explorateur de base.";
    pub const WORLD_REIMPORT_CONFIRM_TITLE: &str = "Réimporter la base monde";
    pub const WORLD_REIMPORT_CONFIRM: &str = "La base monde (mangos) sera supprimée puis réimportée depuis le package, compléments HomeCraft compris. Les personnages, comptes et sauvegardes sont conservés. Toute modification faite dans la base monde sera perdue. Le serveur de jeu sera arrêté ; l'opération prend une à deux minutes.";
    pub const WORLD_REIMPORT_PLAYING: &str = "Impossible de réimporter la base monde pendant que le jeu est lancé : fermez le jeu d'abord.";
    pub const MSG_WORLD_REIMPORT: &str = "Réimport de la base monde…";
    pub const MSG_WORLD_REIMPORT_DONE: &str = "Base monde réimportée. Vous pouvez redémarrer le serveur.";

    // VanillaFixes / DXVK
    pub const SET_VF: &str = "Lancer le jeu via VanillaFixes (corrige les saccades et le rythme des animations)";
    pub const SET_VF_HINT: &str = "Copie VanillaFixes.exe, VfPatcher.dll et dlls.txt dans le dossier du client et démarre le jeu par ce lanceur (DLL injectée au démarrage, déchargée à l'écran de connexion). Certains antivirus signalent à tort ce type de lanceur. Licence MIT.";
    pub const SET_DXVK: &str = "Rendu DXVK : traduire Direct3D 9 en Vulkan (cartes graphiques récentes)";
    pub const SET_DXVK_HINT: &str = "Copie d3d9.dll (DXVK 2.6.1, licence zlib) à côté de WoW.exe. À essayer si l'affichage pose problème sous Windows 11 ; nécessite un pilote Vulkan. Décocher retire le fichier.";
    pub const SET_VF_MISSING: &str = "VanillaFixes n'est pas présent dans Tools/VanillaFixes.";

    // Support en lecture seule / installation portable
    pub const RO_TITLE: &str = "Ce dossier est en lecture seule (CD, DVD ou dossier protégé).";
    pub const RO_TEXT: &str = "Le jeu a besoin d'écrire sa base de données et les données extraites de votre client. Choisissez un dossier sur votre disque dur : tout le contenu y sera copié, puis le launcher s'ouvrira depuis ce dossier.";
    pub const RO_INSTALL: &str = "Installer sur ce PC…";
    pub const RO_PICK_TITLE: &str = "Choisir le dossier d'installation";
    pub const MSG_INSTALLING: &str = "Copie des fichiers vers le disque…";
    pub const MSG_INSTALLED: &str = "Copie terminée : ouverture du launcher installé.";
    pub const SET_BROWSE: &str = "Parcourir…";
    pub const MSG_CLIENT_NEEDED: &str = "Le client WoW 1.12.1 est introuvable ou invalide : indiquez son dossier dans PARAMÈTRES (il n'est jamais fourni par ce package).";
    pub const MSG_FIRST_EXTRACT: &str = "Première installation : extraction des données de votre client (quelques minutes)…";
}
