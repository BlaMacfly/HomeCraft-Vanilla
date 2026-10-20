# Journal des versions

## 0.9.14 — 20 octobre 2026

- Personnage tout juste créé déconnecté à son entrée dans le monde : des bots générés serveur en marche recevaient des numéros (GUID) que mangosd, qui les attribue à partir d'un compteur lu au démarrage, redonnait ensuite aux personnages créés en jeu. Le personnage du joueur écrasait le bot en base, puis le serveur le refusait à la connexion (« Cheat Attempt to login to character on different account » dans `Anticheat.log`), et l'inscription de peuplement du bot désignait désormais le personnage du joueur.
- La génération et la suppression des personnages bots se font maintenant serveur arrêté (boutons grisés et explication sinon).
- Au démarrage du serveur, les inscriptions de peuplement qui ne désignent pas un personnage du compte des bots sont retirées : les personnages touchés se connectent de nouveau normalement.

## 0.9.13 — 19 octobre 2026

- Bots de peuplement absents de `/qui` malgré « Bots visibles dans /qui » : les personnages bots étaient créés avec `world_phase_mask = 0`, alors qu'un personnage normal a 1 (`WORLD_DEFAULT_CHAR`). VMaNGOS écarte de `/qui` tout personnage sans phase commune avec le joueur (`Player::IsVisibleGloballyFor` → `CanSeeInWorld`), sauf en mode MJ (`.gm on`). La visibilité en jeu n'en dépendait pas, d'où des bots visibles en ville mais pas dans la liste. Les bots sont désormais créés avec la phase 1, et ceux déjà générés sont corrigés au démarrage du serveur.

## 0.9.12 — 19 octobre 2026

- Accueil : la génération des cartes de déplacement (mmaps) affiche une barre de progression et le temps restant estimé, mis à jour toutes les 10 secondes (tuiles écrites rapportées aux 1 742 d'un jeu complet, au rythme observé depuis le lancement).

## 0.9.11 — 19 octobre 2026

- PlayerBots : génération et peuplement ont désormais la même limite, 5 000. La génération était limitée à 300 personnages par clic, alors que le nombre de bots connectés montait à 5 000.
- Après une génération, le maximum de bots connectés est réglé sur le total de personnages bots (et non plus plafonné à 300) ; le minimum, connecté d'un coup au démarrage, sur la moitié.
- L'onglet PlayerBots signale quand le peuplement demandé dépasse les personnages bots existants (le serveur ne peut pas en connecter plus), et, au-delà de 1 000 bots connectés, la mémoire et le processeur à prévoir.
- Noms de bots : jusqu'à environ 150 000 noms distincts, lettres seulement. Au-delà de 6 000 bots, l'ancien repli produisait des noms avec chiffres, refusés par le jeu.

## 0.9.10 — 19 octobre 2026

- Bots de peuplement : le monde se figeait (joueur bloqué sur « Connecté », un cœur du processeur à 100 %) quand le serveur démarrait pendant la génération des cartes de déplacement. mangosd charge toute tuile mmaps présente : avec des mmaps à moitié générées, le continent avait une carte de navigation sans la tuile de la capitale, et VMaNGOS cherche sans fin une position de marche pour y placer les bots (`PopulateAreaBotAI::BeforeAddToMap`, boucle sur `Map::GetWalkRandomPosition`). Relevé sur la pile de mangosd : Forgefer, tuile écrite trois minutes après le démarrage du serveur.
- Les mmaps sont désormais générées dans `Server\Data\mmaps-generation` (liens physiques vers maps et vmaps) et n'entrent dans `Server\Data\mmaps` qu'une fois complètes, serveur arrêté (au démarrage suivant). Une génération inachevée d'une version précédente est retirée du serveur et reprise là où elle s'était arrêtée.

## 0.9.9 — 19 octobre 2026

- Bots de peuplement : au-delà de 100 bots connectés, le joueur restait bloqué sur « Connecté ». Chaque bot ouvre une session sur le serveur et la limite de joueurs du modèle VMaNGOS (`PlayerLimit = 100`) mettait le joueur, et les bots suivants, en file d'attente (le client Benilla n'affiche pas la position dans la file). La limite est désormais supprimée (`PlayerLimit = 0`) ; elle est aussi levée à chaud dès qu'on enregistre un réglage serveur en marche.

## 0.9.8 — 19 octobre 2026

- Extraction : VMapExtractor plante de façon intermittente (0xC0000374, corruption du tas : la même commande réussit ensuite). Le launcher le relance jusqu'à trois fois, sur un dossier de travail propre.
- Extraction : seules les archives officielles du client sont lues. Les extracteurs ouvrent aussi `patch-3.MPQ` à `patch-99.MPQ` : un patch graphique (VanillaHD…) injectait ses modèles dans les données du serveur (collisions fausses, 4 002 bâtiments au lieu de 3 913). Les patchs ajoutés sont écartés par un dossier de liens physiques vers les seules archives officielles (`Server\Data\extraction-client`, supprimé ensuite), ou, si le client est sur un autre disque, masqués le temps de l'extraction puis rétablis.
- PlayerBots et Monde : serveur en marche, les réglages enregistrés sont appliqués tout de suite (`mangosd.conf` réécrit, puis `reload config` et `bot reload`). Auparavant `bot reload` relisait l'ancienne configuration en mémoire : les bots de peuplement générés ou activés serveur allumé n'apparaissaient qu'après un redémarrage.
- Génération de bots sur une installation neuve, avant le premier démarrage : message clair (démarrer le serveur une première fois) au lieu d'une erreur SQL.
- Option de test `--do applysettings <fichier>`.

## 0.9.7 — 19 octobre 2026

- Correctif : « VMapExtractor a échoué (code 1) » après une extraction interrompue (programme bloqué par Windows, launcher fermé en cours). VMapExtractor refuse de repartir si son dossier de travail `Server\Data\Buildings` contient encore `dir` ou `dir_bin` (« Your output directory seems to be polluted »). Le launcher vide désormais ce dossier, entièrement régénéré, avant chaque extraction.
- Les erreurs des extracteurs sont transmises au journal sans attendre (20 au plus) et le message d'échec reprend leurs dernières lignes au lieu du seul code de sortie.

## 0.9.6 — 19 octobre 2026

- Programme bloqué par Windows (« Une stratégie de contrôle d'application a bloqué ce fichier », os error 4551, ainsi que 4553, 1260 et 225) : le launcher explique la cause (Contrôle intelligent des applications de Windows 11, qui refuse les programmes non signés : les outils et serveurs VMaNGOS ne le sont pas) et la marche à suivre, au lieu du seul code d'erreur. S'applique aux extracteurs, à realmd, mangosd, MariaDB et au client. Manuel : entrée de dépannage correspondante.

## 0.9.5 — 19 octobre 2026

- Client Benilla sans `base.MPQ` : plus aucune manipulation. L'extracteur VMapExtractor de VMaNGOS plante si cette archive est absente, mais n'en lit pas le contenu (extraction identique octet pour octet avec une archive quelconque, vérifiée sur les 3 913 fichiers). Le launcher crée donc dans le `Data\` de Benilla une `base.MPQ` de remplacement (un fichier texte HomeCraft, aucun contenu du jeu), que Benilla ne monte jamais. La 0.9.4 s'arrêtait sur « il manque base.MPQ » quand aucun client d'origine ne se trouvait à côté de l'installation.

## 0.9.4 — 18 octobre 2026

- Surveillance des services : les ports du serveur ne sont plus sondés qu'une fois toutes les 10 secondes (l'état des processus reste contrôlé chaque seconde). Le launcher ouvrait une connexion au serveur de jeu à chaque seconde, journalisée par mangosd (« Connection accepted/closed », 9 434 lignes sur une journée de test).

## 0.9.3 — 18 octobre 2026

- Bots de peuplement : après une génération, le launcher règle au plus 300 bots connectés (minimum la moitié) au lieu du total généré, et active « Bots visibles dans /qui ». Mesure : avec 2 600 personnages bots, l'ancien réglage (minimum 1 300) faisait connecter plus de 1 000 bots d'un coup (1,1 Go pour le serveur, un cœur de processeur), invisibles dans `/qui`.
- Réglages « Nombre minimal / maximal de bots connectés » : limite relevée de 500 à 5 000, aide détaillée (connexion au démarrage, coût mesuré).
- Manuel : où se trouvent les bots (quartier commerçant de Hurlevent, Forgefer, Orgrimmar), combien en connecter, dépannage pas à pas.
- Nouveau document `Documentation/Commandes-GM.md` : commandes MJ et administrateur utiles en solo (en français) et liste complète des 756 commandes du serveur avec leur niveau, extraite du code de VMaNGOS (`Build/doc-tools/`).
## 0.9.2 — 17 octobre 2026

- Correctif : premier démarrage avec un client Benilla sans `base.MPQ`. VMapExtractor (VMaNGOS) ouvre cette archive et plantait (code 0xC0000005) sans elle. Le launcher vérifie désormais les archives d'extraction ; s'il en manque, il lit celles d'un client d'origine complet trouvé près de l'installation (sans rien copier), sinon il indique clairement l'archive à ajouter. `base.MPQ` peut rester dans le `Data\` de Benilla, qui ne la monte jamais.

## 0.9.1 — 17 octobre 2026

- **Client Benilla 64 bits pris en charge** en plus du client d'origine : reconnaissance d'un exécutable Benilla (`benilla.exe` ou renommé) à côté de `benilla-config\` et des douze archives 1.12.1 (`base.MPQ` non exigé), configuration de `benilla-config\config.toml` (`realmList`, `realmName`, `gxWindow`, `gxResolution`, le reste du fichier conservé), lancement direct, AddOns dans `benilla-config\AddOns`, VanillaFixes et DXVK désactivés pour ce client.
- Paramètres : type du client détecté sous le dossier, bouton « Rechercher les clients » (`Client` et dossiers voisins de l'installation) avec « Utiliser ».
- Accueil et journal : type du client affiché.
- Tests unitaires du module client (fusion du `config.toml`, détection, dossier réel si présent).

## 0.9.0 — 16 octobre 2026 (version candidate)

Première publication. Tout ce qui suit est couvert par les tests automatisés de `Tests/` (tous réussis).

### Jeu
- Serveur VMaNGOS 1.12.1 (commit `53d24d5`, build de développement du 2026-10-07) avec les PartyBots intégrés ; base de données VMaNGOS (snapshot `4641790`) sur MariaDB 11.8.9 portable, tout sur 127.0.0.1.
- Compagnons `.partybot` (tank, soigneur, DPS, classes au choix), bots de peuplement générés par le launcher (compte dédié), champs de bataille complétés par des bots.
- Hôtel des ventes approvisionné : 6 392 objets utiles en solo (`Server/Database/custom/auctionhousebot_solo.sql`).
- Compatibilité du client frFR build 5880 (realm, clients autorisés, correctif reproductible du serveur monde `Build/Patch-MangosdBuilds.ps1`).
- Traductions françaises complétées : 0 quête réelle, 0 dialogue de PNJ, 0 créature visible sans texte français (`locales_quest_homecraft.sql`, `locales_creature_homecraft.sql`, non officielles, jamais écrasantes).

### Launcher (Rust, egui)
- Un bouton JOUER : base → authentification → monde → configuration du client (realmlist, fenêtré, résolution bornée) → jeu. Premier démarrage automatique (import, extraction dbc/maps/vmaps), mmaps en arrière-plan.
- Onglets Accueil, Comptes (création, mot de passe, niveau GM, suppression avec personnages), PlayerBots (réglages + génération de bots), Monde (70 réglages avec défauts Vanilla), Patchs (activer, désactiver, ajouter, construire un MPQ depuis un dossier), Base de données (explorateur natif façon HeidiSQL + bouton HeidiSQL), Sauvegardes (manuelles, automatiques à la fermeture du jeu avec rotation, de sécurité, restauration vérifiée, diagnostic), Paramètres (client, affichage, VanillaFixes 1.5.3, DXVK 2.6.1, AHBot, mmaps, sauvegarde auto, réimport de la base monde, AddOns), Journaux (console serveur).
- Vérification des fichiers par manifeste SHA-256, réparation, bouton FERMER LE JEU, détection de support en lecture seule.
- Exécutable signé (certificat auto-signé, horodatage), icône et informations de version intégrées.

### Distribution
- Package portable `Build/New-Package.ps1` (dossier + zip ≈ 135 Mo, compatible CD-ROM) et image ISO `Build/New-Iso.ps1`. Le client n'est jamais inclus.
- Licence propriétaire `LICENCE.txt` (usage personnel ; consultation et fork sur GitHub autorisés) ; composants tiers sous leurs licences, sources référencées dans `LICENCES/SOURCES.md`.
- Manuel d'utilisation complet (`Documentation/Manuel-Utilisateur.md`) et documentation technique en dix chapitres.

### Connu, non corrigé
- Certificat auto-signé : avertissement SmartScreen « Éditeur inconnu » au premier lancement.
- Les bots de peuplement sont cachés de `/qui` par défaut (réglage dans PlayerBots).
- Le correctif du build 5880 est un patch binaire ; une recompilation de VMaNGOS avec le correctif source reste à faire.
