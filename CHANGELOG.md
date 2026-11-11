# Journal des versions

## 1.0.1 — 11 novembre 2026 (édition CMaNGOS)

- **Bouton « Low Computer »** (accueil, à bascule) pour les PC modestes : serveur allégé (mise à jour des cartes toutes les 200 ms au lieu de 100, zones vides déchargées au bout d'une minute, distance de visibilité 60 / 100 m, mémoire de la base 128 Mo) et **PlayerBots désactivés** ; l'onglet PlayerBots est grisé avec une infobulle, l'IA des bots ne démarre pas. Mesuré : le serveur occupe environ 500 Mo (contre plus de 3 Go avec 100 à 200 bots) et démarre en une quarantaine de secondes. Réponse à un retour de joueur : sur un portable de 4 Go (Celeron N4120), le serveur saturé faisait échouer la connexion au monde, puis le client manquait de mémoire graphique au chargement.
- Premier démarrage sur un PC lent : le module PlayerBots prépare une seule fois l'équipement des bots (10 minutes mesurées ici, davantage sur un petit processeur) sans rien afficher ; le launcher abandonnait au bout de 10 minutes et le premier démarrage échouait à chaque essai. L'attente est maintenant prolongée pendant cette étape (jusqu'à une heure) et l'accueil l'annonce.
- Profils, client d'origine : le mot de passe est saisi quand l'écran de connexion est réellement prêt (au premier lancement, il était tapé avant son affichage).
- VÉRIFIER LES FICHIERS ne signale plus 34 fichiers « manquants » : le package ne contient plus que la documentation des joueurs, et son manifeste en tient compte.

## 1.0.0 — 1er novembre 2026 (édition CMaNGOS)

- **HomeCraft Multijoueur** (nouvel onglet) : partie privée entre amis sur le serveur de l'hôte, **intégrée au launcher** (aucun programme à installer, aucun droit administrateur, aucun serveur intermédiaire). Liaison chiffrée et authentifiée (protocole Noise : X25519, ChaCha20-Poly1305, BLAKE2s). Invitations `HC2-…` : une clé aléatoire par joueur, durée limitée, **révocables** (connexions coupées immédiatement) ; rien de persistant. Les invités n'atteignent que l'authentification et le monde de l'hôte : MariaDB, la console et le reste du PC sont injoignables par construction. Pas de redirection de port dans la plupart des cas : ouverture automatique par la box (**UPnP**), sinon **perçage de NAT** (STUN public, QUIC avec certificat épinglé, code de réponse `HCR-…` signé). Les invités se connectent « en local » : aucun réglage de royaume ni de client.
- **Plusieurs personnages d'un même compte en même temps** (multiboxing) : nouveau groupe « Sessions » dans l'onglet Monde (activé par défaut, 4 sessions par compte, 1 à 10) ; la session en trop est refusée sans déconnecter les autres. Correctif du serveur : sessions indexées par compte en multimap, requêtes asynchrones rattachées à leur session, reprise d'une session coupée.
- **Plus d'expulsion pour inactivité** (AFK au bout de 15 minutes), réglable ; une connexion réellement perdue reste détectée, et le personnage est déconnecté et sauvegardé.
- **Profils de lancement** : un bouton par profil sur l'accueil ; compte prérempli, personnage préféré présélectionné, mot de passe facultatif **chiffré pour votre session Windows** (DPAPI), jamais en clair ni journalisé. Benilla : connexion automatique par son mécanisme prévu (et entrée directe en jeu possible) ; client d'origine : compte dans Config.wtf et mot de passe tapé dans la fenêtre du jeu. Un jeu par profil, plusieurs profils ouverts à la fois.
- **IA des bots** (section de l'onglet PlayerBots) : les bots répondent aux joueurs avec un modèle de langage, par un pont local du launcher. Fournisseurs : LM Studio, Ollama, OpenAI, Anthropic, Mistral, OpenRouter, compatible OpenAI ; liste des modèles (jamais de téléchargement), test, clé chiffrée, personnalité, langue, température, jetons, délai, simultanéité, paroles spontanées. Au plus une action par réponse, dans une liste fermée (suivre, rester, attaquer, gestes), exécutée avec les droits d'un ordre chuchoté par le joueur. Si le fournisseur ne répond pas, retour aux réponses classiques pendant une minute. Correctif du module playerbots.
- **Interface en anglais** : sélecteur Langue / Language dans Paramètres (catalogue de réglages compris).
- **Manuel d'utilisation réécrit**, avec schémas (architecture, premier démarrage, multijoueur, perçage de NAT, IA, profils, sauvegardes, dossiers) ; liste des commandes MJ régénérée pour CMaNGOS (517 commandes).
- Correction : le bouton « Supprimer » d'un compte restait grisé serveur démarré ; la suppression est désormais seulement refusée si un personnage du compte est en jeu.
- Tests : client du protocole 1.12 en Rust (`Tests\wowclient`) ; tests de bout en bout du multiboxing et de l'AFK, du multijoueur (TCP et QUIC, deux launchers), des profils (Benilla et client d'origine), de l'IA ; 36 tests unitaires. Rapport : `docs\IMPLEMENTATION_VALIDATION.md`.

## 0.10.3 — 25 octobre 2026 (édition CMaNGOS)

- **Correction : plantage de mangosd quelques dizaines de secondes après le démarrage** (« service interrompu : serveur de jeu ne répond plus »). Analyse des vidages mémoire de Windows : violation d'accès dans `dtNavMeshQuery::queryPolygons`, appelée par l'action des bots `SetAvoidAreaAction` (zones de monstres à éviter) via `PathFinder::setArea`, sur une carte **sans cartes de déplacement chargées**. Le module playerbots suppose que les mmaps existent ; or, sur une installation neuve, le serveur démarrait avant leur génération. Avec 100 à 200 bots (réglage par défaut), le plantage survenait en moins d'une minute.
  - Les cartes de déplacement sont désormais **générées pendant la première installation, avant le premier démarrage du serveur** (tous les cœurs, une quinzaine à une trentaine de minutes, une seule fois, progression affichée). Fermer le launcher l'interrompt ; elle reprend au démarrage suivant. Le réglage « générer les mmaps en arrière-plan » disparaît.
  - Serveur (`homecraft.patch`) : `PathFinder::setArea`, `getArea` et `getFlags` ne plantent plus sur une carte sans navigation ; module (`playerbots.patch`) : `WorldPosition::ClosestCorrectPoint` renvoie « pas de point » au lieu d'arrêter le serveur.
- Test d'intégration avec les réglages par défaut (100 à 200 bots) et génération des mmaps au premier démarrage.

## 0.10.2 — 25 octobre 2026 (édition CMaNGOS)

- **Surveillance plus tolérante** : une seule connexion d'essai (délai de 0,4 s) sans réponse suffisait à déclarer le serveur perdu et à tout arrêter. *Ce n'était pas la cause des arrêts constatés après le démarrage : mangosd plantait réellement, voir 0.10.3.* Désormais : délai de 2 s, et un service n'est déclaré perdu qu'après trois échecs consécutifs (chaque échec est noté dans le journal) ; un processus qui s'arrête reste détecté à la seconde.
- La génération des mmaps tourne en priorité « inférieure à la normale » : le serveur et le jeu passent avant.
- **Après une fermeture forcée du launcher** (plantage, fin de tâche), realmd, mangosd et MariaDB restaient en marche et le démarrage suivant échouait (« le port 3724 est déjà utilisé »). Le launcher arrête désormais les serveurs restés en marche **depuis le dossier de cette installation** (jamais un autre serveur du PC) et reprend en main sa propre MariaDB.
- **Un seul launcher par installation** : un second launcher ouvert sur le même dossier affiche un message et se ferme, au lieu de risquer d'arrêter les services du premier.
- Test d'intégration : premier démarrage avec la génération des mmaps en cours, 90 secondes de fonctionnement sous charge.

## 0.10.1 — 25 octobre 2026 (édition CMaNGOS)

- **Correction : le serveur ne démarrait pas sur une installation neuve** (« chargement du monde : mangosd s'est arrêté pendant le chargement »). Les cartes de déplacement (mmaps) ne sont générées qu'après le premier démarrage, et CMaNGOS s'arrêtait net en leur absence (« Critical Error … MMapManager::loadMap »), même désactivées : il demande toujours les tuiles de navigation et considérait l'absence de données comme impossible. Corrigé dans le serveur (`homecraft.patch` : carte sans mmaps = pas de navigation, comme le prévoit déjà le reste du code) et dans le launcher (mmaps activées seulement présentes et complètes ; celles générées en arrière-plan sont installées avant l'écriture de la configuration du démarrage suivant).
- Test d'intégration : premier démarrage sans mmaps (cas d'une vraie installation), puis avec.

## 0.10.0 — 23 octobre 2026 (édition CMaNGOS, en développement)

**Nouveau serveur : CMaNGOS Classic avec le module playerbots**, le même système de bots que les playerbots d'AzerothCore (lignée ike3 / celguar). L'édition VMaNGOS (0.9.18) reste disponible ; cette édition est développée à part.

- **Un monde peuplé de bots qui jouent comme des joueurs** : ils montent de niveau, font leurs quêtes, chassent, commercent, vont en donjon et en champ de bataille, forment des groupes et fondent des guildes. Le serveur crée lui-même leurs comptes (RNDBOT…) et leurs personnages au premier démarrage, puis en connecte entre le minimum et le maximum réglés (100 à 200 par défaut). Mesuré sur un test de 10 minutes avec 40 bots : déplacements vers leurs zones de niveau, la moitié avec de l'expérience gagnée, 20 guildes fondées.
- **Invitations** : un bot accepte votre invitation quel que soit votre niveau (réglage « Écart de niveau maximal », 60 par défaut, au lieu de 30 dans CMaNGOS), vous suit, combat avec vous et obéit aux ordres du canal de groupe (follow, stay, attack, flee, summon, leave…). `.bot group` complète votre groupe avec des bots de classes complémentaires à votre niveau ; `.bot add Nom` connecte un autre personnage de votre compte comme bot.
- **Textes en français** : les traductions officielles 1.12 de la base VMaNGOS sont reprises dans la base CMaNGOS (quêtes 100 %, objets 99 %, créatures 91 %, dialogues 100 %) ; les phrases des bots sont aussi traduites.
- **Onglet PlayerBots** refait : nombre de bots, niveaux, groupes, guildes, discussions, invitations, performances ; réglages appliqués à chaud (`rndbot reload`). Bouton de suppression des bots aléatoires (serveur arrêté puis relancé automatiquement).
- **Comptes** : niveaux GM 0 à 3 (joueur, modérateur, maître de jeu, administrateur) ; les comptes RNDBOT sont réservés aux bots et masqués de la liste.
- **Hôtel des ventes** : AHBot du cœur CMaNGOS, approvisionne les trois hôtels des ventes.
- **Données du client** : extraites avec les outils CMaNGOS (ad, vmap_extractor, vmap_assembler, MoveMapGen). Des données d'une édition VMaNGOS sont reconnues et réextraites automatiquement. Cartes de déplacement complètes en 13 minutes (12 threads).

Construction : `Build\Build-Cmangos.ps1` (commits figés, Boost 1.87, runtime Visual C++ local) et `Build\Build-CmangosDb.ps1` (installateur officiel classic-db, traductions, comptes de démonstration supprimés). Modifications publiées dans `Build\cmangos-patches` : build client 5880, option `-p` de mangosd réellement prise en compte par le module, console `rndbot reload`.

**Licence** : le module playerbots ne publie aucune licence. Cette édition reste à usage personnel tant que ses auteurs n'ont pas donné leur accord (demande préparée dans `Documentation\11-Licence-PlayerBots.md`).

Personnages : les personnages de l'édition VMaNGOS ne sont pas repris (schéma de base différent).

## 0.9.18 — 23 octobre 2026

**Bots du monde.** Le serveur VMaNGOS est désormais compilé par HomeCraft (même commit 53d24d5, modifications publiées dans `Sources\Build\vmangos-patches\homecraft.patch`, GPL-2.0), pour donner une vraie vie aux bots de peuplement :

- **Invitation acceptée** : un bot invité par un joueur accepte toujours, quel que soit son niveau (sauf s'il accompagne déjà quelqu'un). Il prend le niveau du joueur, avec talents, sorts et équipement adaptés (réglage « Bot invité mis à votre niveau »), le rejoint par téléportation et combat avec lui comme un compagnon `.partybot` (suivi, attaque de sa cible, soins, donjons ; les commandes `.partybot` fonctionnent sur lui). Renvoyé du groupe, il reprend sa vie.
- **Chasse** : hors groupe, les bots partent dans une zone de leur niveau sur les deux continents, jamais en territoire adverse. Ils y combattent les monstres ordinaires avec les sorts de leur classe, sans toucher à ceux qu'un autre a déjà engagés, se reposent, gagnent de l'expérience et des niveaux, et changent de zone quand il n'y a plus rien à chasser.
- **En ville** : une partie reste dans les capitales (« Part des bots restant en ville », 25 % par défaut) et y flâne.
- Le gestionnaire de peuplement ne déconnecte jamais un bot qui accompagne un joueur.
- Les bots ne font pas encore de quêtes par eux-mêmes.

Corrections dans le serveur, trouvées en testant :

- Un bot jamais sauvegardé « regardait » la cinématique d'introduction de sa race à chaque connexion, sans client pour la terminer : les monstres ne pouvaient pas l'attaquer et abandonnaient aussitôt le combat.
- Avec `PlayerBot.AllowSaving`, les bots de peuplement étaient réenregistrés sous le numéro de compte fictif de leur session et quittaient le compte des bots.
- Plusieurs fonctions des compagnons supposaient un groupe et auraient planté le serveur pour un bot seul.
- Le build client 5880 est accepté dans les sources (`cmake/generators/Progression.h.in`) au lieu d'un patch binaire. Le contrôle est automatique : à chaque démarrage du test d'intégration, la liste écrite par le serveur doit contenir 5880.

Outils : `Build\Build-Vmangos.ps1` (clone, patch, compilation CMake/MSBuild, installation ; vérifie que chaque fichier du patch est appliqué). La phase 5 du test d'intégration vérifie les bots du monde : 60 bots pendant 5 minutes, chasse hors des capitales, expérience gagnée, charge du serveur, arrêt propre.

## 0.9.17 — 20 octobre 2026

HomeCraft ne dépend plus d'aucun « Microsoft Visual C++ Redistributable » installé dans Windows. Auparavant, sur un Windows où ces runtimes manquent (installation neuve), le launcher ne démarrait pas (VCRUNTIME140.dll introuvable) et le serveur non plus.

- Launcher compilé avec la bibliothèque C intégrée (CRT statique).
- Programmes VMaNGOS (mangosd, realmd, extracteurs) : runtime Visual C++ 2015-2022 (14.44, la version de leur compilation) placé à côté d'eux, tiré de MariaDB.
- `libmySQL.dll` de VMaNGOS (client MySQL 5.5.62, qui exigeait le runtime Visual C++ **2008**) remplacé par une couche de compatibilité qui redirige ses fonctions vers MariaDB Connector/C (`libmariadb.dll`, déjà livré avec la base). Structures et options utilisées par VMaNGOS vérifiées identiques ; seules trois fonctions de threads, devenues sans effet, sont fournies par la couche.
- Nouveau contrôle `Tests\Test-Dependencies.ps1` : chaque DLL demandée par un programme du package est une DLL de Windows ou se trouve à côté de lui (22 manques détectés sur le package 0.9.16, 0 sur la 0.9.17).

## 0.9.16 — 20 octobre 2026

Audit complet du code du launcher.

- **Suppression d'un compte serveur arrêté** : elle effaçait toutes les tables de la base `characters` ayant une colonne `guid`. Or dans `item_instance`, `item_loot`, `corpse`, `creature_respawn`, `gameobject_respawn` et `pet_*`, cette colonne est le numéro d'un objet, d'un cadavre, d'une créature ou d'un familier : des objets et familiers d'autres personnages pouvaient disparaître. Chaque table est maintenant désignée par la colonne qui porte vraiment le numéro du personnage (propriétaire, destinataire, membre…). La suppression des bots utilise la même fonction.
- **Démarrage** : en cas d'échec (import, configuration, chargement du monde trop long), MariaDB et mangosd sont toujours arrêtés, au lieu de rester en arrière-plan et d'occuper les ports. Une base lancée par le launcher et laissée en service (après une restauration ou un réimport) reste sous sa responsabilité : elle est arrêtée à la fermeture.
- **Fermeture** : le launcher attend la fin réelle de l'arrêt des services (jusqu'à deux minutes) au lieu de 0,2 seconde.
- **Comptes** : le niveau MJ est attribué une fois le compte réellement enregistré par le serveur (« Compte inexistant » et comptes créés deux fois auparavant) ; un compte existant est signalé ; identifiant de 1 à 16 lettres ou chiffres, mot de passe de 4 à 16 caractères sans espace.
- **Mots de passe** masqués dans les journaux (création de compte, changement de mot de passe, commandes tapées dans la console).
- **Fermer le jeu** : seuls les WoW.exe du dossier client HomeCraft sont fermés (auparavant tous les WoW.exe du PC).
- **Restauration** : l'interface relit les réglages restaurés (elle réécrivait les anciens au prochain « Enregistrer »).
- **Hôtel des ventes** en faction automatique : calculée sur vos personnages, plus sur les bots.
- **Journaux** : messages de la console du serveur décodés correctement (page de code 850 : « Compte créé » au lieu de « Compte cr�� »).
- **Archives** : noms de fichiers des sauvegardes et des AddOns contrôlés (rien ne peut être écrit hors du dossier prévu) ; un patch ajouté est copié sous un nom provisoire puis renommé (jamais d'archive tronquée dans `Data`).
- **Scripts du package** (`Tools\Scripts`) : ils calculaient la racine du dossier depuis `Tools` et ne fonctionnaient pas ; la racine est maintenant le premier dossier parent contenant `Server`.
- Tests : `Tests\Test-Integration.ps1` (copie du package, premier démarrage, comptes, suppression sans dommage pour les autres comptes, bots serveur en marche, aucun processus résiduel) ; option `--do acccreate <compte> <mot de passe> [niveau]`.

## 0.9.15 — 20 octobre 2026

- PlayerBots : « Générer les personnages bots » et « Supprimer tous les personnages bots » ne sont plus grisés serveur en marche. Le launcher arrête le serveur le temps de l'opération puis le relance ; c'est refusé, avec explication, si un personnage joueur est en jeu.
- Correctif de la 0.9.14 : le retrait des inscriptions de peuplement qui désignent un personnage joueur échouait (« No database selected », DELETE multi-tables par alias sans base par défaut). Requête réécrite avec une sous-requête.

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
