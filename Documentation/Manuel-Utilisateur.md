# HomeCraft Edition Vanilla — Manuel d'utilisation

Version du manuel : 1.0 (15 octobre 2026), pour le launcher 0.9.0.

HomeCraft Edition Vanilla vous permet de jouer à World of Warcraft 1.12.1 (« Vanilla ») **seul, hors ligne, en français**, sur votre propre ordinateur, avec des compagnons contrôlés par l'ordinateur, un hôtel des ventes approvisionné et des réglages de confort. Tout est piloté par une seule fenêtre : le launcher. Ce manuel décrit tout ce que le launcher sait faire, dans l'ordre où vous en aurez besoin.

---

## Sommaire

1. [Ce que c'est, ce que ce n'est pas](#1-ce-que-cest-ce-que-ce-nest-pas)
2. [Prérequis](#2-prérequis)
3. [Contenu du dossier](#3-contenu-du-dossier)
4. [Installation](#4-installation)
5. [Premier démarrage pas à pas](#5-premier-démarrage-pas-à-pas)
6. [La fenêtre du launcher](#6-la-fenêtre-du-launcher)
7. [Onglet Accueil](#7-onglet-accueil)
8. [Onglet Comptes](#8-onglet-comptes)
9. [Jouer en solo avec des compagnons](#9-jouer-en-solo-avec-des-compagnons)
10. [Onglet PlayerBots](#10-onglet-playerbots)
11. [Onglet Monde](#11-onglet-monde)
12. [Onglet Patchs](#12-onglet-patchs)
13. [Onglet Base de données](#13-onglet-base-de-données)
14. [Onglet Sauvegardes](#14-onglet-sauvegardes)
15. [Onglet Paramètres](#15-onglet-paramètres)
16. [Onglet Journaux et console du serveur](#16-onglet-journaux-et-console-du-serveur)
17. [Vérifier et réparer l'installation](#17-vérifier-et-réparer-linstallation)
18. [Déplacer, dupliquer, désinstaller](#18-déplacer-dupliquer-désinstaller)
19. [Dépannage](#19-dépannage)
20. [Questions fréquentes](#20-questions-fréquentes)
21. [Où sont mes fichiers ?](#21-où-sont-mes-fichiers-)
22. [Glossaire](#22-glossaire)
23. [Mentions légales](#23-mentions-légales)

---

## 1. Ce que c'est, ce que ce n'est pas

**C'est** un serveur de jeu complet (VMaNGOS, fidèle au patch 1.12 de 2006), une base de données (MariaDB) et un programme de pilotage (le launcher), installés dans un seul dossier, sans rien installer dans Windows. Le serveur n'écoute que sur votre propre machine (adresse 127.0.0.1) : personne d'autre ne peut s'y connecter, et rien ne sort vers Internet.

**Ce n'est pas** le jeu lui-même. Le client World of Warcraft 1.12.1 (le dossier avec `WoW.exe` et `Data\`) appartient à Blizzard Entertainment et n'est pas fourni : vous apportez le vôtre, en version française. HomeCraft n'est pas un service en ligne, pas un serveur privé partagé, et n'a aucun lien avec Blizzard.

**Ce qu'il apporte au jeu de base**

- des **compagnons** (PlayerBots) que vous invoquez dans le jeu pour former un groupe : tank, soigneur, DPS ;
- un **hôtel des ventes** rempli automatiquement (6 392 objets utiles en solo) ;
- des **bots de peuplement** optionnels qui flânent dans les capitales ;
- des **réglages** du monde (expérience, butin, difficulté, factions, etc.) sans toucher à un fichier ;
- des **traductions françaises** complétées là où la base d'origine était en anglais ;
- la gestion de vos **patchs graphiques** et de vos **AddOns** ;
- des **sauvegardes** vérifiées, automatiques à chaque fermeture du jeu, et une restauration en un clic ;
- la **vérification** et la **réparation** de l'installation ;
- deux options de modernisation du client : **VanillaFixes** (fluidité) et **DXVK** (rendu Vulkan).

## 2. Prérequis

| Élément | Minimum |
|---|---|
| Système | Windows 10 ou 11, 64 bits |
| Processeur | 4 cœurs conseillés (le serveur, la base et le jeu tournent en même temps) |
| Mémoire | 8 Go (16 Go confortable) |
| Espace disque | environ 4 Go pour HomeCraft (base de données + données extraites) **en plus** du client (≈ 5 Go, plus vos patchs HD éventuels) |
| Client | World of Warcraft **1.12.1**, version **française** (frFR), build 5875 ou 5880 ; ou **Benilla** (client libre 64 bits) utilisant les archives de ce client |
| Droits | un dossier où vous pouvez écrire (pas `C:\Program Files`, pas un CD) |
| Réseau | aucun : tout fonctionne hors ligne |

Aucun logiciel supplémentaire n'est nécessaire : ni Visual C++, ni .NET, ni MySQL, ni Python.

## 3. Contenu du dossier

```
HomeCraft Edition Vanilla\
├─ HomeCraft Edition Vanilla.exe   le launcher (signé)
├─ LISEZMOI.txt                    démarrage rapide
├─ LICENCE.txt                     licence d'utilisation de HomeCraft
├─ Client\                         VOTRE client 1.12.1 (vide à la livraison)
├─ Server\                         serveur VMaNGOS, MariaDB, dumps de la base, configurations générées,
│                                  données extraites de votre client (Server\Data, créées au premier démarrage)
├─ UserData\                       VOS données : base de données (personnages), réglages, sauvegardes, identifiants
├─ Logs\                           journaux du launcher et du serveur
├─ Tools\                          HeidiSQL, VanillaFixes/DXVK, scripts
├─ Documentation\                  ce manuel et la documentation technique
├─ LICENCES\                       licences des composants tiers et liste des sources
├─ Sources\                        code source du launcher et des scripts de construction
└─ Build\                          manifeste des fichiers (vérification), certificat de signature
```

Deux dossiers vous appartiennent et doivent être préservés : `Client\` (votre jeu) et `UserData\` (vos personnages et sauvegardes).

## 4. Installation

HomeCraft est livré sous forme **portable** : un dossier (ou son archive zip) à copier, sans installateur. Rien n'est écrit dans Windows en dehors de ce dossier.

1. **Copiez** l'intégralité du dossier livré sur votre disque dur, par exemple dans `C:\Jeux\HomeCraft Edition Vanilla\`. Évitez `C:\Program Files` (droits d'écriture) et les dossiers synchronisés en nuage (OneDrive) : la base de données y fonctionne mal.
2. **Placez votre client** 1.12.1 frFR dans le sous-dossier `Client\` (le dossier doit contenir directement `WoW.exe` et `Data\`). Vous pouvez aussi laisser le client où il est et indiquer son emplacement dans Paramètres (voir §15).
3. **Ouvrez** `HomeCraft Edition Vanilla.exe`.

**Avertissement « Éditeur inconnu ».** Windows SmartScreen affiche cet avertissement parce que l'exécutable est signé avec un certificat personnel, non reconnu par Microsoft. Cliquez sur « Informations complémentaires » puis « Exécuter quand même ». La signature garantit au moins que le fichier n'a pas été modifié depuis sa création ; vous pouvez la consulter dans Propriétés > Signatures numériques.

**Antivirus.** Certains antivirus signalent à tort les lanceurs de jeu ou VanillaFixes (qui injecte une bibliothèque dans le jeu). Si c'est le cas, ajoutez le dossier HomeCraft aux exclusions.

**Si vous avez ouvert le launcher depuis un support en lecture seule** (CD, DVD, dossier protégé), il vous le dit et propose « Installer sur ce PC… » : choisissez un dossier de destination, tout y est copié et le launcher s'y rouvre. Ce cas n'est pas l'usage prévu : copiez le dossier d'abord.

## 5. Premier démarrage pas à pas

1. Au premier lancement, l'accueil affiche l'état des composants : Client « vérifié » (ou « introuvable » si le client n'est pas encore en place), Base de données « arrêtée », Serveur « arrêté ».
2. Cliquez sur **JOUER**. Le launcher enchaîne automatiquement, en affichant chaque étape et une barre de progression :
   - *Préparation de la base de données locale* : création de la base MariaDB dans `UserData\Database` et d'identifiants aléatoires (`UserData\db-credentials.json`) ;
   - *Première installation : import des données du monde* : les quatre bases (comptes, personnages, monde, journaux) sont importées, puis les compléments HomeCraft (hôtel des ventes, traductions) ; comptez une à deux minutes ;
   - *Extraction des données du client* : les outils du serveur lisent votre client pour produire les cartes (dbc, maps, vmaps) dans `Server\Data` ; comptez une à trois minutes, une seule fois ;
   - *Démarrage du serveur d'authentification* puis *Chargement du monde* : une dizaine de secondes.
   Au total, le premier démarrage prend de **deux à cinq minutes**. Les suivants prennent quelques secondes.
3. Quand l'état passe à **PRÊT**, le launcher lance le jeu tout seul (c'est le bouton JOUER qui a été utilisé). Le client est configuré automatiquement : adresse du serveur (`realmlist.wtf`), mode fenêtré et résolution adaptée à votre écran (`Config.wtf`).
4. **Créez votre compte** dans l'onglet **Comptes** (voir §8), puis connectez-vous dans le jeu avec cet identifiant et ce mot de passe. Créez un personnage et entrez dans le monde.
5. En arrière-plan, le launcher lance la génération des **cartes de déplacement (mmaps)**, nécessaires aux déplacements fins des compagnons en donjon. Elle dure de plusieurs dizaines de minutes à quelques heures selon votre processeur ; vous pouvez jouer pendant ce temps. L'accueil indique « génération en cours depuis N min » puis « générées : actives au prochain démarrage du serveur ». Si vous fermez le launcher avant la fin, elle reprend au démarrage suivant.

**Quand vous avez fini de jouer :** quittez le jeu normalement (Quitter dans le menu). Le launcher crée une sauvegarde automatique de vos personnages, puis vous pouvez fermer le launcher : il arrête proprement le serveur et la base (quelques secondes, ne forcez pas la fermeture).

## 6. La fenêtre du launcher

La fenêtre est organisée en **onglets** (barre du haut) : Accueil, Comptes, PlayerBots, Monde, Patchs, Base de données, Sauvegardes, Paramètres, Journaux.

En permanence, l'accueil affiche un **état** en lettres capitales :

| État | Signification |
|---|---|
| NON INSTALLÉ | des composants sont absents (dossier incomplet) : RÉPARER ou réinstaller |
| INSTALLÉ | tout est en place, services arrêtés |
| VÉRIFICATION / INITIALISATION / DÉMARRAGE… | démarrage en cours, étape affichée dessous |
| PRÊT | serveur opérationnel, vous pouvez jouer |
| JEU EN COURS | le jeu est lancé ; le serveur reste actif tant que la fenêtre est ouverte |
| ARRÊT EN COURS | fermeture propre des services |
| RÉPARATION | opération longue en cours (réparation, restauration, réimport, construction d'un patch) |
| ERREUR | une étape a échoué : le message explique laquelle ; voir §19 |

Les boutons se grisent quand l'action n'est pas possible dans l'état courant (par exemple, pas de restauration pendant que le jeu tourne). Toute action irréversible demande une **confirmation** dans une petite fenêtre.

**Fermer la fenêtre** arrête d'abord les services : si le jeu est encore ouvert, il sera déconnecté. Fermez le jeu avant.

## 7. Onglet Accueil

C'est le tableau de bord.

**Composants** : Client (vérifié / réparation nécessaire / introuvable, avec sa version), Base de données (arrêtée / opérationnelle), Serveur (arrêté / démarrage / opérationnel), PlayerBots (activés / désactivés).

**Boutons principaux**

- **JOUER** : démarre les services si nécessaire, puis lance le jeu. Actif dès que le client est valide.
- **DÉMARRER LE SERVEUR** : démarre les services sans lancer le jeu (utile pour créer un compte, régler le monde, laisser les bots de peuplement se connecter).
- **ARRÊTER LE SERVEUR** : arrêt propre (les personnages sont enregistrés par le serveur avant l'arrêt).
- **FERMER LE JEU** (visible seulement pendant le jeu) : force la fermeture de `WoW.exe`, pour un écran noir ou un jeu figé. Ce que le serveur n'a pas encore enregistré (au plus quelques minutes) est perdu.

**Boutons secondaires** : VÉRIFIER LES FICHIERS, RÉPARER (voir §17), SAUVEGARDER (sauvegarde manuelle immédiate, puis bascule sur l'onglet Sauvegardes), RESTAURER (ouvre l'onglet Sauvegardes), PARAMÈTRES, JOURNAUX.

**Diagnostic** (bas de l'accueil) : dossier d'installation, version du launcher, dossier et version du client, archives MPQ manquantes le cas échéant, présence de chaque composant, état des **cartes de déplacement (mmaps)** (présentes / absentes / génération en cours / générées, actives au prochain démarrage / échec), nombre de personnages en ligne.

## 8. Onglet Comptes

Un compte de jeu = un identifiant + un mot de passe, auquel sont rattachés jusqu'à 10 personnages. Vous pouvez créer autant de comptes que vous voulez (un par membre de la famille, par exemple).

**Le serveur doit être démarré** pour créer ou modifier un compte (DÉMARRER LE SERVEUR ou JOUER).

**Créer un compte** : saisissez l'identifiant (lettres et chiffres, sans espace), le mot de passe, choisissez le **niveau GM** puis « Créer le compte ».

**Niveau GM** : 0 = joueur normal ; **6 = administrateur**, indispensable pour invoquer des compagnons avec les commandes `.partybot`. En solo, mettez 6. Un compte GM joue exactement comme un autre tant que vous n'activez pas le mode maître de jeu dans le jeu (`.gm on`) ; voir le réglage « Mode MJ à la connexion » dans l'onglet Monde.

**Liste des comptes** : identifiant, niveau GM, dernière connexion, nombre de personnages. Sélectionnez un compte pour :

- **Changer le mot de passe** (saisissez le nouveau dans le champ Mot de passe puis cliquez) ;
- **Appliquer le niveau GM** (après l'avoir changé dans le champ) ;
- **Supprimer** : le compte **et tous ses personnages** (objets, courrier, progression) sont supprimés définitivement, après confirmation. Faites une sauvegarde avant si vous avez un doute. Impossible pendant que le jeu est lancé. Le launcher avertit si c'est le compte connecté le plus récemment.

**Actualiser** relit la liste depuis la base.

**Se connecter dans le jeu** : à l'écran de connexion du jeu, saisissez l'identifiant et le mot de passe tels que créés ici. Le royaume s'appelle « HomeCraft » et apparaît seul dans la liste.

## 9. Jouer en solo avec des compagnons

Les compagnons sont des personnages contrôlés par le serveur qui rejoignent votre groupe, combattent avec un rôle (tank, soigneur, DPS), vous suivent et ramassent leur butin. Ils sont créés à la volée, **à votre niveau**, et disparaissent quand vous les renvoyez ou vous déconnectez.

Toutes les commandes se tapent dans le **chat du jeu**, précédées d'un point, avec un compte de niveau GM 6.

**Former un groupe type (tank, soigneur, DPS)**

```
.partybot add tank
.partybot add healer
.partybot add dps
```

Vous pouvez ajouter jusqu'à quatre compagnons (un groupe compte cinq membres) ou plus en raid si le réglage « Compagnons maximum par groupe » le permet.

**Toutes les commandes**

| Commande | Effet |
|---|---|
| `.partybot add tank` / `healer` / `dps` | invoque un compagnon de ce rôle, classe choisie automatiquement |
| `.partybot add warrior` (ou `paladin`, `hunter`, `rogue`, `priest`, `shaman`, `mage`, `warlock`, `druid`) | invoque un compagnon de cette classe |
| `.partybot clone` | copie la race et la classe du joueur ciblé |
| `.partybot load <nom>` | charge un de **vos** personnages hors ligne comme compagnon (sa progression n'est enregistrée que si le réglage « Enregistrer la progression des personnages chargés » est activé) |
| `.partybot setrole tank` / `healer` / `meleedps` / `rangedps` | change le rôle du compagnon ciblé |
| `.partybot attackstart` / `attackstop` | tous les compagnons attaquent / cessent d'attaquer votre cible |
| `.partybot pull [ms]` | le tank engage la cible, les DPS attendent le délai indiqué |
| `.partybot aoe` | sorts de zone sur la cible |
| `.partybot caststart` / `caststop` | autorise / interdit les sorts (compagnon ciblé ou tous) |
| `.partybot ccmark <marque>` / `focusmark <marque>` | marque à contrôler / à focaliser (`star`, `circle`, `diamond`, `triangle`, `moon`, `square`, `cross`, `skull`) |
| `.partybot clearmarks` | efface les marques |
| `.partybot cometome` | le compagnon ciblé (ou tous) vous rejoint |
| `.partybot usegobject` | les compagnons utilisent l'objet du monde ciblé (levier, porte) |
| `.partybot pause [ms] [all]` / `unpause [all]` | met en pause / reprend (le compagnon ciblé, ou tous avec `all`) |
| `.partybot unequip <lien d'objet>` | retire un objet au compagnon ciblé |
| `.partybot remove` | renvoie le compagnon ciblé |

L'onglet PlayerBots du launcher rappelle cette liste.

**Conseils de jeu**

- Ciblez un compagnon (clic sur son portrait) avant une commande qui ne concerne que lui (`setrole`, `cometome`, `remove`).
- En donjon, laissez le tank engager (`.partybot pull`) puis attaquez. Les compagnons soignent et attaquent d'eux-mêmes.
- Les compagnons ont besoin des **cartes de déplacement (mmaps)** pour contourner les obstacles en intérieur. Tant qu'elles ne sont pas générées puis prises en compte (redémarrage du serveur), ils peuvent rester bloqués derrière un mur : utilisez `.partybot cometome`.
- L'hôtel des ventes de votre faction est approvisionné automatiquement toutes les minutes ; vous y trouverez composants, recettes, armes et armures de tous niveaux à des prix raisonnables.
- En solo, pensez aux réglages de l'onglet Monde (§11) : quêtes de raid autorisées en groupe simple, Alliance et Horde sur le même compte, dégâts des créatures légèrement réduits, cinématiques, etc.

## 10. Onglet PlayerBots

Cet onglet règle les compagnons et les bots de peuplement. Les réglages sont **appliqués au prochain démarrage du serveur** (bouton « Enregistrer les réglages PlayerBots »).

**Compagnons (.partybot)**

- *Autoriser les compagnons* : coche générale.
- *Compagnons maximum par groupe* : 0 = illimité ; un groupe compte 5 membres, un raid 40.
- *Lever les restrictions d'invocation* : permet d'invoquer en combat, en donjon, et de choisir le niveau.
- *Équipement des nouveaux compagnons* : objets de départ, équipement aléatoire adapté au niveau (conseillé) ou modèle prédéfini ; *Écart de niveau d'objet toléré* affine l'aléatoire.

**Bots de peuplement (RandomBot)** : des personnages bots se connectent et flânent dans Hurlevent, Forgefer et Orgrimmar pour donner vie aux villes. Ils ne combattent pas et ne vous suivent pas.

1. Dans la section **Personnages bots**, choisissez un nombre (20 pour commencer) et cliquez « Générer les personnages bots ». Ils sont créés sur un compte technique `HOMECRAFTBOTS` (moitié Alliance, moitié Horde, races, classes, niveaux et apparences variés). Les réglages de peuplement sont ajustés automatiquement (activés, minimum = la moitié, maximum = le total).
2. Si le serveur tourne, les bots se connectent aussitôt : le launcher fait relire sa configuration au serveur (`reload config`, `bot reload`). Sinon, ils se connectent au prochain démarrage. Vérifiez avec `.bot info` dans le jeu (compte de niveau 6) : la ligne « Online » donne le nombre de bots connectés.
3. « Supprimer tous les personnages bots » retire tout, après confirmation ; vos propres personnages ne sont jamais concernés.

Les autres réglages (nombre minimal / maximal de bots, intervalle de renouvellement) sont modifiables ensuite.

**Combien de bots ?** Générer beaucoup de personnages bots ne coûte rien : seuls les bots **connectés** consomment des ressources. Au démarrage, le serveur connecte d'un coup le « nombre minimal », puis toutes les minutes il en ajoute ou en retire un pour rester entre le minimum et le maximum. Mesure sur ce projet : 1 023 bots connectés occupent environ 1,1 Go de mémoire et un cœur de processeur. Après une génération, le launcher règle donc au plus 300 bots connectés (minimum 150) et rend les bots visibles dans `/qui` ; montez ces valeurs dans PlayerBots si votre PC le permet.

**Où sont-ils ?** Chaque bot est placé à sa connexion dans sa capitale : le **quartier commerçant de Hurlevent** (humains et elfes de la nuit, dans un rayon d'environ 50 m), **Forgefer** (nains et gnomes) ou **Orgrimmar** (orcs, morts-vivants, taurens et trolls, dans toute la ville). Vous ne les croiserez ni dans les zones de départ ni à Darnassus, aux Pitons-du-Tonnerre ou à Fossoyeuse. Ils flânent, certains errent au hasard ; ils ne combattent pas et ne répondent pas aux messages.

**Champs de bataille (BattleBot)** : à l'inscription à un champ de bataille, des bots complètent les deux camps pour qu'il démarre.

**Général** : temps de réaction des bots (plus court = plus réactifs), **« Bots visibles dans /qui »** (désactivé par défaut : les bots de peuplement sont alors comptés par le launcher dans « Personnages en ligne » mais n'apparaissent pas dans la liste `/qui` du jeu ; cochez pour les y voir, enregistrez, redémarrez le serveur), enregistrement de la progression des personnages chargés avec `.partybot load`, informations de débogage.

## 11. Onglet Monde

Les réglages les plus utiles du serveur de jeu, regroupés par thème, chacun avec sa **valeur Vanilla par défaut** rappelée (« Défaut ») et marqué « modifié » quand vous l'avez changé. « Tout remettre aux valeurs Vanilla » annule l'ensemble. Enregistrez, puis redémarrez le serveur pour appliquer.

| Groupe | Exemples de réglages |
|---|---|
| Règles du monde | contenu du monde par patch (1.2 Cœur du Magma → 1.12 complet), type de royaume (JcE / JcJ / jeu de rôle), message du jour, niveau et argent de départ, cinématiques, personnages par royaume, niveau requis des instances, heure de réinitialisation des raids, affichage des quêtes selon le niveau, **quêtes de raid autorisées en groupe simple**, nombre de métiers principaux, signatures pour fonder une guilde (0 = guilde solo) |
| Expérience et progression | expérience des victoires / quêtes / exploration, repos, points de talent, réputation, vitesse des métiers (artisanat, récolte, armes, défense), minerai par filon, distance de partage d'expérience |
| Butin et économie | chances de butin par rareté (gris à légendaire), argent trouvé, délai de livraison du courrier |
| Difficulté des créatures | dégâts, points de vie et dégâts des sorts des créatures normales, élites, élites rares et boss mondiaux ; distance d'agression ; faiblesse de résurrection ; délai de récupération du corps ; malus de déserteur |
| Factions et social | **Alliance et Horde sur le même compte**, chat, canaux, groupes, guildes, échanges, courrier, `/qui` et amis entre factions |
| Maître de jeu (compte GM) | mode MJ à la connexion (« Désactivé » conseillé pour jouer normalement), visibilité, invulnérabilité en mode MJ, échanges autorisés pour les comptes GM |
| Technique | distance d'affichage en extérieur et en instance, intervalle de sauvegarde des personnages, déchargement des instances vides, annonce des événements |

**Réglages conseillés pour un solo confortable** : quêtes de raid en groupe simple = oui ; Alliance et Horde sur le même compte = oui ; signatures pour une guilde = 0 ; dégâts des créatures normales et élites = 0,8 à 0,9 si vous jouez sans soigneur ; cinématiques = seulement au premier personnage ; mode MJ à la connexion = Désactivé.

**Contenu du monde par patch** : le serveur reproduit la sortie progressive du contenu de 2004 à 2006. Avec « 1.12 — complet », tout est disponible (Naxxramas compris). Un patch plus ancien retire les donjons, objets et événements sortis après.

## 12. Onglet Patchs

Le client charge, après ses archives officielles, les fichiers `patch-X.MPQ` du dossier `Client\Data` (X : chiffre de 3 à 9 ou lettre de A à Z ; les suivants ont priorité sur les précédents). C'est ainsi que s'installent les patchs de **graphismes HD**, d'interface ou de sons. HomeCraft n'en fournit ni n'en télécharge aucun : vous apportez les vôtres.

- La liste montre chaque archive, sa taille et son état : **officielle** (archives du client, non modifiables), **actif**, **désactivé** (renommé en `.off`, conservé), ou « nom non chargé par le client ».
- **Activer / Désactiver** : bascule sans rien supprimer.
- **Supprimer** : efface l'archive définitivement (après confirmation). Préférez Désactiver.
- **Ajouter un patch…** : choisissez un fichier `.MPQ` ; il est copié sous le prochain nom libre.
- **Créer un patch depuis un dossier…** : transforme une arborescence de fichiers (par exemple le dépôt décompressé d'un patch VanillaHD « Patch-A-main ») en archive MPQ au format du client 1.12.1. Comptez plusieurs minutes pour un gros patch (plus d'un gigaoctet). Le nom est choisi automatiquement et l'archive est relue pour contrôle.
- **Ouvrir le dossier Data** ouvre l'Explorateur Windows.

Fermez le jeu avant de modifier les patchs (fichiers en cours d'utilisation).

**Important** : les outils du serveur ne lisent que les archives officielles. Un patch n'a donc **aucun effet sur le serveur** : les patchs purement visuels ou sonores fonctionnent parfaitement, mais un patch qui ajouterait des races, des objets ou des sorts côté client ne serait pas reconnu par le serveur.

## 13. Onglet Base de données

Un explorateur intégré, dans l'esprit de HeidiSQL, pour consulter et, si vous savez ce que vous faites, modifier les quatre bases : `characters` (personnages, objets, quêtes accomplies, courrier, hôtel des ventes), `realmd` (comptes), `mangos` (le monde : créatures, objets, quêtes, traductions) et `logs`.

La base doit être démarrée (DÉMARRER LE SERVEUR suffit).

- **Arbre** : base > tables. Cliquez une table pour l'afficher.
- **Données** : 200 lignes par page (Précédent / Suivant), **Filtre** = clause WHERE libre (ex. `level >= 40`), tri par clic sur l'en-tête de colonne. **Double-cliquez une cellule** pour la modifier (Entrée valide, Échap annule). « Insérer une ligne » ouvre un formulaire (vide = valeur par défaut, `NULL` pour une valeur nulle) ; « Supprimer la ligne » demande confirmation. Les tables sans clé primaire sont en lecture seule dans la grille.
- **Structure** : colonnes, types, clés, valeurs par défaut.
- **Requête SQL** : tapez une requête. « Exécuter (lecture) » accepte SELECT / SHOW / DESCRIBE ; « Exécuter (écriture) » exécute toute autre instruction **après confirmation**. Un historique conserve vos requêtes de la session.
- **Ouvrir dans HeidiSQL** : lance l'outil externe fourni (`Tools\HeidiSQL`), déjà connecté à la base locale, pour les opérations avancées (export, édition massive).

Toute écriture est journalisée dans Journaux. **Avant de modifier la base, faites une sauvegarde** (onglet Sauvegardes). Pour annuler des modifications du monde, utilisez « Réimporter la base monde » (§15) ; pour annuler des modifications de personnages, restaurez une sauvegarde (§14).

Quelques requêtes utiles (lecture) :

```sql
SELECT name, level, class, race, money FROM characters.characters;
SELECT username, gmlevel, last_login FROM realmd.account;
SELECT COUNT(*) FROM characters.auction;
```

## 14. Onglet Sauvegardes

Une sauvegarde contient les bases `characters` et `realmd` (export cohérent, tables verrouillées le temps de l'export), vos réglages (`settings.json`) et un manifeste d'empreintes SHA-256. C'est une archive zip horodatée dans `UserData\Sauvegardes`, d'environ 1 Mo au début. La base monde n'en fait pas partie : elle se réimporte depuis le package.

**Trois sortes de sauvegardes**

- **manuelles** : bouton « Sauvegarder maintenant » (libellé libre) ou SAUVEGARDER de l'accueil ;
- **automatiques** : à chaque fermeture du jeu, libellé « automatique à la fermeture du jeu » ; seules les 10 plus récentes sont conservées (réglable dans Paramètres) ;
- **de sécurité** : créées automatiquement avant une restauration (« automatique avant restauration »).

Les manuelles et les sauvegardes de sécurité ne sont jamais supprimées automatiquement.

Si le jeu est en cours, une sauvegarde force d'abord l'enregistrement des personnages par le serveur (`saveall`) puis exporte.

**Restaurer** : sélectionnez une sauvegarde dans la liste (date, libellé, nombre de personnages et de comptes, taille), cliquez « Restaurer la sélection », confirmez. Le serveur de jeu est arrêté, les empreintes de l'archive sont vérifiées, une sauvegarde de sécurité de l'état actuel est créée, puis les personnages et comptes sont **remplacés** par ceux de l'archive. Impossible pendant que le jeu est lancé. Redémarrez ensuite le serveur.

**Supprimer la sélection** efface l'archive du disque (confirmation). **Ouvrir le dossier** ouvre `UserData\Sauvegardes` : copiez régulièrement ce dossier sur un autre support (clé USB, disque externe). **Exporter un diagnostic** produit un zip (`UserData\diagnostic-*.zip`) avec les journaux, les configurations (mots de passe masqués) et l'état des fichiers, à joindre à une demande d'aide.

**Conseil** : avant toute manipulation risquée (suppression de compte, modification de la base, changement de version), faites une sauvegarde manuelle avec un libellé parlant.

## 15. Onglet Paramètres

Cliquez **Enregistrer** après vos modifications ; les réglages liés au serveur s'appliquent au prochain démarrage de celui-ci.

**Dossier du client WoW 1.12.1** : vide = `Client\` de l'installation ; sinon « Parcourir… » pour désigner le dossier du client. Le client n'est jamais téléchargé.

**Client détecté** : sous le dossier, le launcher indique le type de client reconnu. Deux sont pris en charge :

- le **client d'origine** : `WoW.exe` 1.12.1 (32 bits) et ses archives dans `Data\` ;
- **Benilla** : client libre réécrit en Rust (64 bits, licences MIT et Apache 2.0), avec un exécutable à la racine (`benilla.exe`, ou renommé, par exemple `HomeCraft.exe`), les douze archives 1.12.1 dans `Data\` et son dossier d'état `benilla-config\`. Le launcher règle alors `benilla-config\config.toml` (royaume local, fenêtré ou plein écran sans bordure, taille de fenêtre), installe les AddOns dans `benilla-config\AddOns` et désactive VanillaFixes et DXVK, inutiles avec Benilla. Le jeu est en français si les archives viennent d'un client frFR. Ne mettez pas dans son `Data\` de patchs chiffrés (patch-3, 4 ou 5 de certains repacks) : Benilla ne sait pas les lire. Rien d'autre à faire : les extracteurs du serveur exigent une archive `base.MPQ` que les dossiers Benilla n'ont pas. Si elle manque, le launcher en crée une de remplacement, vide (aucun contenu du jeu), dans le `Data\` de Benilla, qui l'ignore. Une vraie `base.MPQ` convient aussi.

« **Rechercher les clients** » liste les clients valides trouvés dans `Client\` et dans les dossiers voisins de l'installation (par exemple sur le Bureau) ; « **Utiliser** » en sélectionne un, puis Enregistrer. Vos personnages sont sur le serveur : ce sont les mêmes quel que soit le client utilisé.

**Jeu en mode fenêtré** (recommandé) : changez le mode **ici**, pas dans les options vidéo du jeu. Le launcher borne aussi la résolution à votre écran : un client 1.12.1 en fenêtré à la taille exacte de l'écran affiche un écran noir, le launcher l'évite.

**Lancer le jeu via VanillaFixes** : corrige les saccades et le rythme des animations du vieux moteur sur les machines modernes (copie `VanillaFixes.exe`, `VfPatcher.dll` et `dlls.txt` dans le dossier du client ; la bibliothèque est injectée au démarrage et déchargée à l'écran de connexion). Désactivé par défaut. Certains antivirus signalent ce type de lanceur : ajoutez une exclusion.

**Rendu DXVK** : traduit Direct3D 9 en Vulkan (copie `d3d9.dll` à côté de `WoW.exe`). À essayer si l'affichage pose problème sous Windows 11 ; nécessite un pilote Vulkan à jour. Décocher retire le fichier.

**Hôtel des ventes approvisionné (AHBot)** et **Hôtel des ventes servi** : Automatique (faction du dernier personnage connecté), Alliance ou Horde.

**Générer les cartes de déplacement (mmaps) en arrière-plan** : activé par défaut ; voir §5. **Générer les mmaps** relance la génération à la main (utile après une réinstallation du client ou si l'accueil signale un échec).

**Sauvegarder automatiquement à chaque fermeture du jeu** et **Sauvegardes automatiques conservées** : voir §14.

**Réimporter la base monde…** : supprime et recrée la base `mangos` depuis les fichiers du package, traductions et hôtel des ventes compris, **sans toucher aux personnages ni aux comptes**. Confirmation préalable ; le serveur de jeu est arrêté ; une à deux minutes. À utiliser après une mise à jour de HomeCraft ou pour annuler des modifications faites dans l'explorateur de base. Vos réglages de l'onglet Monde sont conservés (ils sont dans `settings.json`, pas dans la base).

**AddOns du client** : installez vos AddOns 1.12.1 (Questie, pfUI, Bagnon, etc.) depuis leur archive zip avec « Ajouter un AddOn (zip)… » : le launcher reconnaît le ou les dossiers d'AddOn (fichier `Nom.toc`) et les place dans `Client\Interface\AddOns`, même si l'archive vient de GitHub (dossier « Nom-master »). La liste montre dossier, titre, version, interface et état ; « Désactiver » renomme le dossier en `.off`, « Supprimer » l'efface (confirmation). Un AddOn conçu pour une autre version du jeu (Interface différente de 11200) est signalé : dans le jeu, cochez « Charger les AddOns obsolètes » (bouton AddOns de l'écran de sélection des personnages). L'activation finale, personnage par personnage, se fait dans le jeu. Aucun AddOn n'est fourni ni téléchargé.

## 16. Onglet Journaux et console du serveur

Le journal du launcher (horodaté) et les messages du serveur de jeu défilent ici ; tout est aussi écrit dans `Logs\launcher.log` et `Logs\server\`.

Le champ **Commande console mangosd** envoie une commande directement à la console du serveur (serveur démarré). Quelques commandes utiles :

| Commande | Effet |
|---|---|
| `server info` | état du serveur, joueurs connectés, durée de fonctionnement |
| `saveall` | enregistre tous les personnages en ligne |
| `account onlinelist` | comptes connectés |
| `announce <texte>` | message à tous les joueurs |
| `server exit` | arrêt du serveur (préférez ARRÊTER LE SERVEUR) |

Dans le jeu, avec un compte GM 6, les commandes commencent par un point : `.gm on` / `.gm off` (mode maître de jeu, invulnérable), `.gm visible on`, `.levelup 5`, `.modify money 100000` (10 pièces d'or), `.tele Stormwind` (téléportation ; les noms de lieux sont en anglais : `.lookup tele <début du nom>` pour chercher), `.additem <id>`, `.unaura all`, `.revive`, `.commands` (liste). Ces commandes modifient votre partie : utilisez-les en connaissance de cause.

## 17. Vérifier et réparer l'installation

**VÉRIFIER LES FICHIERS** compare chaque fichier géré (launcher, serveur, base, scripts, documentation) à son empreinte SHA-256 dans `Build\files-manifest.json`, contrôle la présence des données extraites (dbc, maps, vmaps, mmaps) et, si la base est démarrée, la présence et le schéma des quatre bases. Comptez deux à trois minutes (500 Mo à lire). Un rapport s'affiche : fichiers manquants, altérés, données, bases.

**RÉPARER** corrige ce qui peut l'être sans réinstaller : réimporte les bases manquantes ou incomplètes (**jamais** `characters` si elle contient des personnages), régénère les fichiers de configuration (`realmd.conf`, `mangosd.conf`, `my.ini`, royaume), ré-extrait les données du client si elles manquent. Les fichiers gérés manquants ou altérés sont listés comme irréparables : réinstallez le package (vos personnages et sauvegardes dans `UserData\` sont conservés, voir §18).

Vous pouvez aussi vérifier la copie du package depuis PowerShell, dans le dossier d'installation :

```
Get-FileHash -Algorithm SHA256 (Get-Content SHA256SUMS.txt | ForEach-Object { ($_ -split '  ',2)[1] })
```

## 18. Déplacer, dupliquer, désinstaller

**Déplacer** : fermez le launcher, déplacez le dossier entier où vous voulez. Aucun chemin n'est inscrit dans Windows ; les fichiers de configuration sont régénérés au démarrage suivant.

**Mettre à jour** vers une nouvelle version de HomeCraft : fermez le launcher, faites une sauvegarde manuelle, copiez le nouveau package dans un nouveau dossier, puis copiez-y votre dossier `UserData\` (et `Client\` si vous l'aviez placé dedans) depuis l'ancienne installation. Au premier démarrage, utilisez « Réimporter la base monde » si la documentation de la version l'indique.

**Dupliquer** (deux installations indépendantes, par exemple une par joueur) : copiez le dossier entier ; chaque copie a ses propres bases et sauvegardes. Ne lancez qu'une installation à la fois : elles utilisent les mêmes ports locaux (3315, 3724, 8085).

**Désinstaller** : fermez le launcher et supprimez le dossier. Rien n'a été écrit ailleurs (ni registre, ni Program Files, ni AppData), à deux exceptions près que vous pouvez conserver ou effacer : les fichiers de configuration écrits dans votre **client** (`realmlist.wtf`, dont l'original est conservé sous `realmlist.wtf.orig`, `WTF\Config.wtf`, et les fichiers VanillaFixes/DXVK si vous les aviez activés), et le certificat de signature si vous l'aviez importé volontairement.

## 19. Dépannage

**« Éditeur inconnu » au lancement** : normal (certificat personnel), voir §4.

**Le launcher dit ERREUR au démarrage**

- *client non valide* / *données du client* : le dossier indiqué ne contient pas `WoW.exe` 1.12.1 ou il manque des archives MPQ (listées sur l'accueil). Vérifiez le dossier dans Paramètres.
- *démarrage MariaDB* : le port 3315 est occupé (une autre instance de HomeCraft, ou un autre MySQL/MariaDB) ou le dossier `UserData\Database` est en lecture seule ou synchronisé en nuage. Fermez l'autre programme ; consultez `Logs\mariadb-error.log`.
- *démarrage realmd* / *démarrage mangosd* : port 3724 ou 8085 occupé par un reste d'une session précédente : patientez trente secondes et réessayez, ou redémarrez l'ordinateur. Consultez `Logs\server\Server.log`.
- *chargement du monde* dépassant dix minutes : base endommagée après un arrêt brutal ; RÉPARER, puis « Réimporter la base monde ».
- *service interrompu* : un des services s'est arrêté seul (plantage, antivirus) ; relancez, et consultez les journaux.

**Le jeu affiche un écran noir ou se fige** : bouton FERMER LE JEU sur l'accueil (ou Alt+F4). Ne changez jamais le mode fenêtré / plein écran ni la résolution dans les options vidéo du jeu : faites-le dans Paramètres. Essayez DXVK, ou désactivez-le s'il est actif.

**Le jeu dit « Impossible de se connecter »** : le serveur n'est pas PRÊT (attendez la fin du démarrage) ou le client a été lancé à la main sans passer par JOUER (le `realmlist.wtf` doit pointer vers 127.0.0.1 : le launcher s'en charge).

**Identifiant ou mot de passe refusé** : identifiant et mot de passe sont insensibles à la casse sur ce serveur ; vérifiez l'absence d'espace ou de caractère accentué, ou changez le mot de passe dans l'onglet Comptes.

**« Vous n'avez pas la permission » sur `.partybot`** : le compte n'est pas de niveau GM 6 ; corrigez dans Comptes puis reconnectez-vous. Vérifiez aussi que « Autoriser les compagnons » est coché dans PlayerBots.

**Les compagnons restent bloqués ou traversent les murs** : les cartes de déplacement ne sont pas encore générées ou pas encore prises en compte (redémarrez le serveur une fois l'accueil passé à « générées »). En attendant : `.partybot cometome`.

**Aucun bot de peuplement en ville** : dans l'ordre, vérifiez que (1) des personnages bots ont été générés (PlayerBots > Personnages bots), (2) les réglages ont été **enregistrés** (serveur en marche, ils s'appliquent aussitôt ; en cas de doute, arrêtez puis relancez le serveur), (3) vous êtes au bon endroit : quartier commerçant de Hurlevent, Forgefer ou Orgrimmar (pas dans une zone de départ), (4) « Activer les bots de peuplement » est coché avec un nombre minimal supérieur à 0 (PlayerBots), et (5) dans le jeu, `.bot info` (compte niveau 6) indique des bots en ligne. Si vous les cherchez avec `/qui`, cochez « Bots visibles dans /qui ». Le compteur « Personnages en ligne » de l'accueil inclut les bots : s'il est élevé, ils sont bien connectés.

**Le launcher annonce N personnages en ligne mais `/qui` n'affiche personne** : les bots de peuplement sont cachés de la liste `/qui` par défaut. PlayerBots > Général > « Bots visibles dans /qui », Enregistrer, puis redémarrez le serveur. Rappel : `/qui` ne montre que votre faction, et les bots sont de toute façon visibles physiquement dans les capitales.

**L'hôtel des ventes est vide** : il se remplit progressivement après le démarrage (une vague par minute) pour la faction choisie dans Paramètres ; vérifiez que l'option est cochée.

**Lenteur, ventilateur bruyant pendant une partie** : la génération des mmaps tourne en arrière-plan (accueil). Elle s'arrête d'elle-même ; vous pouvez la désactiver dans Paramètres et la lancer plus tard.

**Mot de passe oublié** : Comptes > sélectionnez le compte > nouveau mot de passe > « Changer le mot de passe ».

**J'ai supprimé un personnage ou un compte par erreur** : Sauvegardes > restaurez la sauvegarde automatique la plus récente d'avant l'erreur.

**L'antivirus a supprimé un fichier** : rétablissez-le depuis la quarantaine, ajoutez le dossier aux exclusions, puis VÉRIFIER LES FICHIERS.

**« Une stratégie de contrôle d'application a bloqué ce fichier » (os error 4551)** : Windows 11 refuse de lancer un programme du serveur. C'est le **Contrôle intelligent des applications** (Smart App Control), actif sur certaines installations neuves de Windows 11 : il bloque tout programme qui n'est pas signé par un éditeur reconnu, et les programmes du serveur VMaNGOS (`MapExtractor`, `VMapExtractor`, `realmd`, `mangosd`…) ne sont pas signés par leurs auteurs. Contrairement à l'avertissement SmartScreen, aucun bouton « Exécuter quand même » n'existe ; les exclusions de l'antivirus n'y changent rien. Pour jouer à HomeCraft sur ce PC :

1. Ouvrez **Sécurité Windows** > **Contrôle des applications et du navigateur** > **Paramètres du contrôle intelligent des applications**.
2. Choisissez **Désactivé**. Sur certaines versions de Windows 11, ce réglage ne peut plus être réactivé ensuite sans réinitialiser Windows : décidez en connaissance de cause. L'antivirus Microsoft Defender, lui, reste actif.
3. Relancez le launcher et cliquez sur JOUER (ou RÉPARER) : l'extraction reprend depuis le début de l'étape interrompue (le launcher nettoie lui-même ce qu'elle avait laissé).

Sur un PC professionnel, le blocage peut venir d'une stratégie de l'entreprise (WDAC) : seul l'administrateur peut l'autoriser.

**Le launcher ne se ferme pas tout de suite** : il arrête proprement le serveur et la base (jusqu'à une minute si beaucoup de personnages). Ne tuez pas le processus : la base pourrait devoir se réparer au démarrage suivant (ce qu'elle sait faire, mais c'est plus long).

Pour toute demande d'aide, joignez le diagnostic (Sauvegardes > Exporter un diagnostic). Entraide et questions : serveur Discord https://discord.gg/M34APTbZ8R

## 20. Questions fréquentes

**Puis-je jouer à plusieurs sur le même serveur ?** Non. HomeCraft est conçu et concédé pour un usage individuel : le serveur n'écoute que sur votre machine.

**Puis-je avoir des personnages Alliance et Horde ?** Oui, sur des comptes différents, ou sur le même compte en activant « Alliance et Horde sur le même compte » dans Monde.

**Le jeu est-il entièrement en français ?** L'interface, les sorts et les noms de zones viennent de votre client français. Les quêtes, dialogues, objets et créatures viennent de la base du serveur, traduite à plus de 97 % par les traductions officielles d'époque ; HomeCraft a complété le reste (quêtes, dialogues, noms de créatures). Quelques textes techniques invisibles restent en anglais.

**Puis-je changer de client plus tard ?** Oui, indiquez le nouveau dossier dans Paramètres puis RÉPARER (les données extraites sont régénérées). Il doit rester un client 1.12.1 frFR.

**Puis-je utiliser des patchs HD et des AddOns ?** Oui, voir §12 et §15. HomeCraft n'en fournit aucun.

**Mes sauvegardes sont-elles compatibles entre versions de HomeCraft ?** Oui tant que la version du serveur VMaNGOS n'a pas changé de schéma de base ; dans ce cas la documentation de la version l'indique.

**HomeCraft se connecte-t-il à Internet ?** Jamais. Aucun téléchargement, aucune télémétrie, aucune mise à jour automatique.

**Que faire des fichiers `.orig` ?** `realmlist.wtf.orig` dans le client est la copie de votre fichier d'origine ; `mangosd.exe.orig` dans `Server\vmangos` est le binaire officiel avant le correctif de compatibilité du client build 5880. Laissez-les.

**Combien de personnages ?** 10 par compte et par royaume (réglable dans Monde), comptes illimités.

**Que compte « Personnages en ligne » sur l'accueil ?** Tous les personnages connectés au serveur : le vôtre et les bots de peuplement. Les compagnons invoqués par `.partybot` sont aussi comptés tant qu'ils sont dans votre groupe.

## 21. Où sont mes fichiers ?

| Quoi | Où |
|---|---|
| Personnages, comptes, objets, courrier, hôtel des ventes (base de données vivante) | `UserData\Database\` (ne jamais modifier à la main) |
| Sauvegardes | `UserData\Sauvegardes\sauvegarde-AAAAMMJJ-HHMMSS.zip` |
| Réglages du launcher, du monde et des bots | `UserData\settings.json` |
| Identifiants techniques de la base (générés) | `UserData\db-credentials.json` (ne pas diffuser) |
| Diagnostics | `UserData\diagnostic-*.zip` |
| Journal du launcher | `Logs\launcher.log` |
| Journaux du serveur | `Logs\server\Server.log`, `DBErrors.log`, `Realmd.log`, etc. |
| Journal de la base | `Logs\mariadb-error.log` |
| Fichiers de configuration générés (ne pas modifier : régénérés) | `Server\Config\realmd.conf`, `mangosd.conf`, `my.ini` |
| Données extraites du client (dbc, maps, vmaps, mmaps) | `Server\Data\` (régénérables : RÉPARER, Générer les mmaps) |
| Compléments HomeCraft de la base monde (hôtel des ventes, traductions) | `Server\Database\custom\*.sql` |
| Vos patchs | `Client\Data\patch-X.MPQ` |
| Vos AddOns | `Client\Interface\AddOns\` |
| Configuration du client écrite par le launcher | `Client\realmlist.wtf`, `Client\WTF\Config.wtf` |

Ports utilisés, sur 127.0.0.1 uniquement : 3315 (base de données), 3724 (authentification), 8085 (monde).

## 22. Glossaire

- **AddOn** : extension d'interface du jeu, écrite en Lua, placée dans `Interface\AddOns`.
- **AHBot** : robot qui approvisionne l'hôtel des ventes.
- **Client** : le programme du jeu (`WoW.exe`) et ses données (`Data\`), fourni par vous.
- **Compagnon / PartyBot** : personnage contrôlé par le serveur qui rejoint votre groupe.
- **DBC, maps, vmaps, mmaps** : données extraites du client dont le serveur a besoin (tables du jeu, relief, collisions, cartes de déplacement).
- **DXVK** : couche qui traduit Direct3D 9 en Vulkan.
- **GM / MJ** : maître de jeu ; le niveau GM 6 donne accès aux commandes d'administration.
- **HeidiSQL** : outil externe de consultation de base de données, fourni.
- **MariaDB** : le serveur de base de données qui stocke le monde et vos personnages.
- **MPQ** : format d'archive de Blizzard ; les patchs sont des archives MPQ.
- **Patch (contenu)** : étape de la sortie progressive du contenu (1.2 à 1.12).
- **Patch (fichier)** : archive `patch-X.MPQ` chargée par le client.
- **RandomBot / bot de peuplement** : personnage bot qui flâne dans une capitale.
- **realmd / mangosd** : les deux programmes du serveur (authentification / monde).
- **Royaume** : le serveur de jeu tel qu'il apparaît dans la liste du client (« HomeCraft »).
- **VanillaFixes** : correctif de fluidité du client 1.12.1.
- **VMaNGOS** : le logiciel de serveur, fidèle au patch 1.12.

## 23. Mentions légales

HomeCraft Edition Vanilla (launcher, scripts, documentation, compléments de base de données, nom et logo) est concédé selon la **licence d'utilisation** figurant dans `LICENCE.txt` : usage personnel autorisé ; reproduction, diffusion et modification interdites ; aucune garantie ; voir le texte complet.

Les composants tiers (VMaNGOS, MariaDB, HeidiSQL sous GPL-2.0 ; VanillaFixes sous MIT ; DXVK sous zlib ; bibliothèques Rust) restent sous leurs licences propres, reproduites dans `LICENCES\`, avec les liens vers leurs sources dans `LICENCES\SOURCES.md`.

World of Warcraft et Blizzard Entertainment sont des marques de Blizzard Entertainment, Inc. HomeCraft est une création indépendante, sans lien avec Blizzard Entertainment, Inc. Le client du jeu n'est pas fourni : vous devez disposer du vôtre et respecter les conditions qui s'y appliquent.
