# Commandes MJ et administrateur — HomeCraft Edition Vanilla

Liste des commandes du serveur VMaNGOS livré avec HomeCraft (commit `53d24d5`), extraite de son code source (`src/game/Chat/Chat.cpp`) : **756 commandes**. La première partie décrit en français celles qui servent vraiment en solo ; l'annexe donne la liste complète avec le niveau requis.

## Comment les utiliser

- **Dans le jeu** : tapez la commande dans la fenêtre de discussion, précédée d'un point, par exemple `.gm on`. Il faut être connecté avec un compte dont le niveau MJ est suffisant (onglet **Comptes** du launcher).
- **Dans la console du serveur** : onglet **Journaux** du launcher, champ « Commande console mangosd », **sans** le point (`account onlinelist`). Seules les commandes marquées « Console : oui » y fonctionnent ; la console a tous les droits.
- `.commands` affiche les commandes accessibles à votre compte. `.help <commande>` existe, mais l'aide intégrée de ce serveur est presque vide : ce document la remplace.
- Les paramètres entre `< >` sont obligatoires, entre `[ ]` facultatifs. Une commande qui vise un personnage s'applique à la **cible sélectionnée** (ou à vous si rien n'est sélectionné), sauf mention d'un nom.

## Niveaux d'accès

| Niveau | Nom dans VMaNGOS | Commandes réservées à ce niveau ou moins |
|---|---|---|
| 0 | Joueur | 15 (`.save`, `.dismount`, `.unstuck`, `.start`, `.server info`, `.commands`…) |
| 1 | Modérateur | 42 |
| 2 | Maître du jeu des tickets | 136 |
| 3 | Maître de jeu | 170 |
| 4 | Administrateur de base | 70 |
| 5 | Développeur | 196 |
| 6 | Administrateur | 113 (dont toutes les commandes `.partybot`) |
| 7 | Console | 14 (création et gestion des comptes, `server exit`) : console du serveur uniquement |

Un compte de **niveau 6**, celui que recommande le launcher pour les compagnons, a accès à toutes les commandes du jeu. Seules celles du niveau 7 restent réservées à la console (onglet Journaux), et le launcher les utilise lui-même pour l'onglet Comptes.

> **Attention** : ces commandes modifient votre partie, parfois définitivement (objets, niveaux, quêtes, monde). Faites une sauvegarde (onglet Sauvegardes) avant de vous en servir sérieusement. Les commandes `.npc spawn`, `.gobject add/delete` et `.tele add` écrivent dans la base monde : « Réimporter la base monde » (Paramètres) les annule.

---

## Les plus utiles en solo

### Mode maître de jeu

| Commande | Niveau | Effet |
|---|---|---|
| `.gm on` / `.gm off` | 2 | Active ou désactive le mode MJ (invulnérable, ignoré par les monstres, balise <MJ>). Le réglage « Mode MJ à la connexion » de l'onglet Monde décide de l'état à la connexion. |
| `.gm visible on` / `off` | 2 | Rend le MJ visible ou invisible des autres. |
| `.gm fly on` / `off` | 3 | Vol libre (comme `.cheat fly`). |
| `.gm ingame` | 0 | Liste les MJ connectés. |
| `.gm chat on` / `off` | 1 | Balise <MJ> dans la discussion. |

### Se déplacer

| Commande | Niveau | Effet |
|---|---|---|
| `.tele <lieu>` | 2 | Téléportation vers un lieu nommé de la table `game_tele` (noms en anglais : `.tele Stormwind`, `.tele Orgrimmar`, `.tele Ironforge`). |
| `.lookup tele <texte>` | 2 | Cherche les lieux dont le nom contient ce texte. |
| `.tele name <joueur> <lieu>` | 2 | Téléporte un autre personnage. |
| `.tele group <lieu>` | 2 | Téléporte tout votre groupe, compagnons compris. |
| `.tele add <nom>` | 5 | Enregistre votre position comme lieu de téléportation (écrit dans la base monde). |
| `.go xyz <x> <y> [z] [carte]` | 2 | Téléportation à des coordonnées. |
| `.go creature <guid>` / `.go object <guid>` | 2 | Va jusqu'à une créature ou un objet du monde. |
| `.goname <joueur>` | 2 | Vous rend auprès d'un personnage. |
| `.namego <joueur>` | 2 | Fait venir un personnage auprès de vous. |
| `.groupgo <joueur>` | 2 | Fait venir le groupe d'un personnage. |
| `.recall` | 1 | Revient à l'endroit d'avant la dernière téléportation. |
| `.gps` | 1 | Affiche votre position (carte, zone, coordonnées). |
| `.start` | 0 | Retour au point de départ de votre race. |
| `.unstuck` | 0 | Débloque un personnage coincé. |
| `.taxicheat on` / `off` | 2 | Tous les trajets aériens connus. |
| `.explorecheat 1` / `0` | 2 | Révèle (1) ou masque (0) toute la carte du monde. |

### Personnage

| Commande | Niveau | Effet |
|---|---|---|
| `.levelup [n]` | 3 | Gagne `n` niveaux (1 par défaut ; négatif pour descendre). |
| `.modify money <cuivre>` | 4 | Ajoute de l'argent en pièces de cuivre : `10000` = 1 or. Négatif pour retirer. |
| `.modify speed <valeur>` | 2 | Vitesse de déplacement (1 = normale). `.modify aspeed` : toutes les vitesses. |
| `.modify hp <n>` / `.modify mana <n>` | 3 | Points de vie / de mana actuels. |
| `.modify scale <valeur>` | 3 | Taille du personnage (1 = normale). |
| `.modify morph <id d'apparence>` / `.demorph` | 3 | Change d'apparence / reprend la sienne. |
| `.modify rep <faction> <valeur>` | 4 | Réputation auprès d'une faction (identifiant de faction). |
| `.modify gender` | 3 | Change le sexe du personnage. |
| `.revive` | 3 | Ressuscite (la cible ou vous). |
| `.die` | 3 | Tue la cible. |
| `.unaura <id\|all>` | 3 | Retire une aura ou toutes. |
| `.aura <id de sort>` | 4 | Applique l'aura d'un sort. |
| `.cast <id de sort>` | 5 | Lance un sort sur la cible. |
| `.cooldown clear` | 3 | Réinitialise les temps de recharge. |
| `.repairitems` | 3 | Répare tout l'équipement. |
| `.bank` / `.stable` | 2 | Ouvre la banque / l'écurie de familiers à distance. |
| `.dismount` / `.mount <apparence>` | 0 / 3 | Descend de monture / monture d'apparence donnée. |
| `.reset talents` / `spells` / `stats` / `level` | 3 / 5 | Réinitialise talents, sorts, caractéristiques ou niveau. |
| `.honor add <points>` | 4 | Ajoute des points d'honneur. |
| `.pinfo [nom]` | 2 | Informations sur un personnage (compte, niveau, argent, temps de jeu). |
| `.save` | 0 | Enregistre votre personnage maintenant. |

### Objets, sorts, métiers, quêtes

| Commande | Niveau | Effet |
|---|---|---|
| `.lookup item <texte>` | 2 | Cherche un objet par nom et donne son identifiant (cliquable). |
| `.additem <id ou lien> [nombre]` | 3 | Donne un objet (glissez un lien d'objet dans la commande ou tapez son identifiant). |
| `.additemset <id d'ensemble>` | 3 | Donne toutes les pièces d'un ensemble. |
| `.lookup spell <texte>` | 2 | Cherche un sort et donne son identifiant. |
| `.learn <id de sort>` | 5 | Apprend un sort. `.unlearn <id>` (3) l'oublie. |
| `.learn all_myclass` | 5 | Tous les sorts de votre classe. Variantes : `all_myspells`, `all_mytalents`. |
| `.learn all_recipes <métier>` | 3 | Toutes les recettes d'un métier. |
| `.maxskill` | 3 | Toutes vos compétences au maximum pour votre niveau. |
| `.setskill <id> <niveau> [max]` | 3 | Fixe une compétence (métier, arme). |
| `.lookup quest <texte>` | 2 | Cherche une quête et donne son identifiant. |
| `.quest add <id>` / `.quest complete <id>` / `.quest remove <id>` | 4 / 4 / 3 | Ajoute, termine ou retire une quête du journal. |
| `.lookup creature <texte>` / `.lookup object <texte>` / `.lookup area <texte>` | 2 | Cherche une créature, un objet du monde, une zone. |

### Créatures et monde

| Commande | Niveau | Effet |
|---|---|---|
| `.npc info` | 1 | Informations sur la créature ciblée (identifiant, guid, faction…). |
| `.npc summon <id>` | 3 | Fait apparaître une créature temporaire. |
| `.npc spawn add <id>` / `.npc spawn delete` | 5 | Ajoute / supprime une créature **permanente** (base monde). |
| `.npc say` / `yell` / `whisper <texte>` | 2 | Fait parler la créature ciblée. |
| `.npc follow` / `unfollow` | 3 | La créature ciblée vous suit. |
| `.npc tame` | 3 | Apprivoise la bête ciblée (chasseur). |
| `.respawn` | 4 | Fait réapparaître les créatures et objets proches. |
| `.combatstop` | 3 | Sort du combat. |
| `.damage <n>` | 3 | Inflige des dégâts à la cible. |
| `.instance listbinds` / `.instance unbind <carte\|all>` | 3 | Liste / supprime vos sauvegardes d'instances (donjons et raids). |
| `.event list` / `.event start <id>` / `.event stop <id>` | 3 / 4 / 4 | Événements du monde (fêtes, invasions…). |
| `.showarea <id>` | 2 | Révèle une zone de la carte. |

### Triches de confort (`.cheat`)

Tous de niveau 3, avec `on` / `off` : `.cheat god` (invulnérable), `.cheat cooldown` (sans recharge), `.cheat casttime` (incantation instantanée), `.cheat powercost` (sorts gratuits), `.cheat fly` (vol), `.cheat waterwalk` (marche sur l'eau), `.cheat wallclimb` (escalade), `.cheat untargetable`, `.cheat immunenpc`, `.cheat beastmaster`. `.cheat status` affiche ceux qui sont actifs.

### Compagnons et bots

| Commande | Niveau | Effet |
|---|---|---|
| `.partybot add tank` / `healer` / `dps` (ou une classe) | 6 | Invoque un compagnon. Liste complète des 19 commandes `.partybot` : manuel, §9, et onglet PlayerBots du launcher. |
| `.battlebot add warsong` / `arathi` / `alterac` | 6 | Ajoute un bot dans la file du champ de bataille. `.battlebot remove` / `removeall` les retire. |
| `.bot info` | 6 | État des bots de peuplement (en ligne, en attente). Console aussi : `bot info`. |
| `.bot start` / `.bot stop` | 6 | Démarre / arrête la connexion automatique des bots de peuplement. |
| `.bot add` / `.bot ranadd` / `.bot delete` / `.bot reload` | 6 | Connecte un bot donné / un bot au hasard, retire un bot, recharge la table `playerbot`. |
| `.ahbot update` / `.ahbot reload` | 6 | Réapprovisionne l'hôtel des ventes / recharge sa table. |

### Serveur (jeu ou console)

| Commande | Niveau | Effet |
|---|---|---|
| `.server info` | 0 | Version, joueurs connectés, temps de fonctionnement. |
| `.saveall` | 6 | Enregistre tous les personnages connectés. |
| `.announce <texte>` / `.notify <texte>` | 4 | Message à tous (discussion / fenêtre). |
| `.kick <nom>` | 2 | Déconnecte un personnage. |
| `.mute <nom> <minutes>` / `.unmute <nom>` | 1 / 2 | Rend muet / rend la parole. |
| `.freeze` / `.unfreeze` | 2 | Fige / libère la cible. |
| `.server shutdown <secondes>` / `.server restart <secondes>` | 6 | Arrêt / redémarrage différé. Préférez le bouton ARRÊTER LE SERVEUR du launcher. |

### Console du serveur uniquement (niveau 7, onglet Journaux, sans point)

| Commande | Effet |
|---|---|
| `account create <nom> <mot de passe>` | Crée un compte (le launcher le fait depuis l'onglet Comptes). |
| `account set gmlevel <nom> <niveau>` | Change le niveau MJ d'un compte. |
| `account set password <nom> <mdp> <mdp>` | Change le mot de passe. |
| `account delete <nom>` | Supprime le compte et ses personnages. |
| `account onlinelist` | Comptes connectés. |
| `server exit` | Arrête le serveur (préférez ARRÊTER LE SERVEUR). |

---

## Annexe : les 756 commandes du serveur

Générée depuis `src/game/Chat/Chat.cpp` de VMaNGOS au commit `53d24d5` (`Build/doc-tools/`). « Niveau » = niveau MJ minimal ; « Console : oui » = utilisable aussi depuis la console du serveur. Les commandes non décrites plus haut sont surtout destinées au développement du serveur (`.debug`, `.reload`, `.wp`, `.mmap`…).

### Commandes simples

| Commande | Niveau | Console |
|---|---|---|
| `.additem` | 3 |  |
| `.additemset` | 3 |  |
| `.angle` | 1 |  |
| `.announce` | 4 | oui |
| `.anticheat` | 2 |  |
| `.aoedamage` | 3 |  |
| `.aura` | 4 |  |
| `.bank` | 2 |  |
| `.charge` | 3 |  |
| `.combatstop` | 3 |  |
| `.cometome` | 3 |  |
| `.commands` | 0 | oui |
| `.damage` | 3 |  |
| `.deleteitem` | 3 |  |
| `.demorph` | 3 |  |
| `.deplenish` | 3 |  |
| `.die` | 3 |  |
| `.dismount` | 0 |  |
| `.distance` | 1 |  |
| `.explorecheat` | 2 |  |
| `.fear` | 3 |  |
| `.freeze` | 2 |  |
| `.gocorpse` | 2 |  |
| `.goname` | 2 |  |
| `.gps` | 1 |  |
| `.groupgo` | 2 |  |
| `.groupinfo` | 2 | oui |
| `.guid` | 1 |  |
| `.help` | 0 | oui |
| `.hidearea` | 2 |  |
| `.hover` | 2 |  |
| `.itemmove` | 3 |  |
| `.kick` | 2 | oui |
| `.knockback` | 3 |  |
| `.levelup` | 3 |  |
| `.linkgrave` | 5 |  |
| `.log` | 3 | oui |
| `.maxskill` | 3 |  |
| `.mount` | 3 |  |
| `.movegens` | 2 |  |
| `.mute` | 1 | oui |
| `.nameaura` | 4 |  |
| `.namedie` | 3 |  |
| `.namego` | 2 |  |
| `.neargrave` | 2 |  |
| `.notify` | 4 | oui |
| `.pinfo` | 2 |  |
| `.possess` | 3 |  |
| `.pvp` | 3 |  |
| `.quit` | 7 | oui |
| `.recall` | 1 |  |
| `.removeriding` | 3 |  |
| `.repairitems` | 3 | oui |
| `.replenish` | 3 |  |
| `.respawn` | 4 |  |
| `.revive` | 3 | oui |
| `.save` | 0 |  |
| `.saveall` | 6 | oui |
| `.setskill` | 3 |  |
| `.showarea` | 2 |  |
| `.sniff` | 6 |  |
| `.stable` | 2 |  |
| `.start` | 0 |  |
| `.taxicheat` | 2 |  |
| `.unaura` | 3 |  |
| `.unfreeze` | 2 |  |
| `.unmute` | 2 | oui |
| `.unstuck` | 0 |  |
| `.variable` | 5 | oui |
| `.wchange` | 4 |  |
| `.whispers` | 1 |  |
| `.wr` | 0 |  |

### .account (12)

| Commande | Niveau | Console |
|---|---|---|
| `.account` | 0 | oui |
| `.account characters` | 3 | oui |
| `.account cleardata` | 0 |  |
| `.account create` | 7 | oui |
| `.account delete` | 7 | oui |
| `.account lock` | 0 | oui |
| `.account onlinelist` | 7 | oui |
| `.account password` | 0 | oui |
| `.account set addon` | 7 | oui |
| `.account set gmlevel` | 7 | oui |
| `.account set locked` | 6 | oui |
| `.account set password` | 7 | oui |

### .ahbot (2)

| Commande | Niveau | Console |
|---|---|---|
| `.ahbot reload` | 6 | oui |
| `.ahbot update` | 6 | oui |

### .antispam (4)

| Commande | Niveau | Console |
|---|---|---|
| `.antispam add` | 2 | oui |
| `.antispam remove` | 4 | oui |
| `.antispam removereplace` | 4 | oui |
| `.antispam replace` | 2 | oui |

### .auction (4)

| Commande | Niveau | Console |
|---|---|---|
| `.auction` | 2 |  |
| `.auction alliance` | 2 |  |
| `.auction goblin` | 2 |  |
| `.auction horde` | 2 |  |

### .ban (6)

| Commande | Niveau | Console |
|---|---|---|
| `.ban account` | 3 | oui |
| `.ban allip` | 6 | oui |
| `.ban character` | 3 | oui |
| `.ban ip` | 6 | oui |
| `.ban note` | 2 | oui |
| `.ban warn` | 2 | oui |

### .baninfo (3)

| Commande | Niveau | Console |
|---|---|---|
| `.baninfo account` | 2 | oui |
| `.baninfo character` | 2 | oui |
| `.baninfo ip` | 3 | oui |

### .banlist (3)

| Commande | Niveau | Console |
|---|---|---|
| `.banlist account` | 2 | oui |
| `.banlist character` | 2 | oui |
| `.banlist ip` | 3 | oui |

### .battlebot (7)

| Commande | Niveau | Console |
|---|---|---|
| `.battlebot add alterac` | 6 | oui |
| `.battlebot add arathi` | 6 | oui |
| `.battlebot add warsong` | 6 | oui |
| `.battlebot remove` | 6 |  |
| `.battlebot removeall` | 6 | oui |
| `.battlebot showallpaths` | 6 |  |
| `.battlebot showpath` | 6 |  |

### .bg (4)

| Commande | Niveau | Console |
|---|---|---|
| `.bg` | 3 |  |
| `.bg start` | 3 |  |
| `.bg status` | 3 |  |
| `.bg stop` | 3 |  |

### .bot (8)

| Commande | Niveau | Console |
|---|---|---|
| `.bot add` | 6 | oui |
| `.bot add_all` | 6 | oui |
| `.bot delete` | 6 | oui |
| `.bot info` | 6 | oui |
| `.bot ranadd` | 6 | oui |
| `.bot reload` | 6 | oui |
| `.bot start` | 6 | oui |
| `.bot stop` | 6 | oui |

### .cast (5)

| Commande | Niveau | Console |
|---|---|---|
| `.cast` | 5 |  |
| `.cast back` | 5 |  |
| `.cast dist` | 5 |  |
| `.cast self` | 5 |  |
| `.cast target` | 5 |  |

### .channel (2)

| Commande | Niveau | Console |
|---|---|---|
| `.channel join` | 1 |  |
| `.channel leave` | 1 |  |

### .character (21)

| Commande | Niveau | Console |
|---|---|---|
| `.character aiinfo` | 1 | oui |
| `.character citytitle` | 6 |  |
| `.character clean items` | 6 | oui |
| `.character clean todelete` | 6 | oui |
| `.character deleted delete` | 7 | oui |
| `.character deleted list account` | 3 | oui |
| `.character deleted list name` | 3 | oui |
| `.character deleted old` | 7 | oui |
| `.character deleted restore` | 4 | oui |
| `.character erase` | 7 | oui |
| `.character fillflys` | 3 | oui |
| `.character hasitem` | 2 | oui |
| `.character level` | 5 | oui |
| `.character premade gear` | 4 |  |
| `.character premade savegear` | 5 |  |
| `.character premade savespec` | 5 |  |
| `.character premade spec` | 4 |  |
| `.character race` | 4 | oui |
| `.character rename` | 3 | oui |
| `.character reputation` | 2 | oui |
| `.character skin` | 4 | oui |

### .cheat (20)

| Commande | Niveau | Console |
|---|---|---|
| `.cheat beastmaster` | 3 |  |
| `.cheat castchecks` | 3 |  |
| `.cheat casttime` | 3 |  |
| `.cheat cooldown` | 3 |  |
| `.cheat criticals` | 3 |  |
| `.cheat debuffs` | 3 |  |
| `.cheat debugtargetinfo` | 3 |  |
| `.cheat fixedz` | 3 |  |
| `.cheat fly` | 3 |  |
| `.cheat god` | 3 |  |
| `.cheat ignoretriggers` | 3 |  |
| `.cheat immunenpc` | 3 |  |
| `.cheat immunepc` | 3 |  |
| `.cheat powercost` | 3 |  |
| `.cheat procs` | 3 |  |
| `.cheat status` | 3 |  |
| `.cheat triggerpass` | 3 |  |
| `.cheat untargetable` | 3 |  |
| `.cheat wallclimb` | 3 |  |
| `.cheat waterwalk` | 3 |  |

### .cinematic (3)

| Commande | Niveau | Console |
|---|---|---|
| `.cinematic addwp` | 5 |  |
| `.cinematic gotime` | 5 |  |
| `.cinematic listwp` | 5 |  |

### .cooldown (3)

| Commande | Niveau | Console |
|---|---|---|
| `.cooldown clear` | 3 |  |
| `.cooldown clearclientside` | 3 |  |
| `.cooldown list` | 3 |  |

### .debug (62)

| Commande | Niveau | Console |
|---|---|---|
| `.debug anim` | 3 |  |
| `.debug assert` | 6 | oui |
| `.debug bg` | 6 | oui |
| `.debug bytes1` | 5 | oui |
| `.debug bytes2` | 5 | oui |
| `.debug chatfreeze` | 6 | oui |
| `.debug condition` | 2 |  |
| `.debug control` | 3 |  |
| `.debug faceme` | 3 |  |
| `.debug factionchange_items` | 6 | oui |
| `.debug forceupdate` | 5 |  |
| `.debug getitemstate` | 5 |  |
| `.debug getitemvalue` | 5 |  |
| `.debug getprevplaytime` | 5 |  |
| `.debug getvaluebyindex` | 5 |  |
| `.debug getvaluebyname` | 5 |  |
| `.debug loottable` | 5 | oui |
| `.debug los` | 5 |  |
| `.debug los allow` | 5 |  |
| `.debug los check` | 5 |  |
| `.debug lrecipient` | 3 |  |
| `.debug moditemvalue` | 5 |  |
| `.debug modvalue` | 5 |  |
| `.debug monster` | 3 |  |
| `.debug movedistance` | 5 |  |
| `.debug moveflags` | 3 |  |
| `.debug movemotion` | 5 |  |
| `.debug movespline` | 3 |  |
| `.debug moveto` | 3 |  |
| `.debug play cinematic` | 2 |  |
| `.debug play music` | 2 |  |
| `.debug play sound` | 2 |  |
| `.debug play text` | 2 |  |
| `.debug pvpcredit` | 5 |  |
| `.debug send buyerror` | 5 |  |
| `.debug send channelnotify` | 5 |  |
| `.debug send chanvisual` | 5 | oui |
| `.debug send chanvisualnext` | 5 | oui |
| `.debug send chatmmessage` | 5 |  |
| `.debug send equiperror` | 5 |  |
| `.debug send impact` | 5 | oui |
| `.debug send mailerror` | 5 |  |
| `.debug send opcode` | 5 |  |
| `.debug send openbag` | 3 |  |
| `.debug send poi` | 5 |  |
| `.debug send qinvalidmsg` | 5 |  |
| `.debug send qpartymsg` | 5 |  |
| `.debug send sellerror` | 5 |  |
| `.debug send spellfail` | 5 |  |
| `.debug send visual` | 5 | oui |
| `.debug send worldstate` | 5 |  |
| `.debug setaurastate` | 5 |  |
| `.debug setitemvalue` | 5 |  |
| `.debug setprevplaytime` | 5 |  |
| `.debug setvaluebyindex` | 5 |  |
| `.debug setvaluebyname` | 5 |  |
| `.debug spellcheck` | 7 | oui |
| `.debug spellcoefs` | 5 | oui |
| `.debug spellmods` | 5 |  |
| `.debug time` | 6 | oui |
| `.debug unitstate` | 3 |  |
| `.debug utf8overflow` | 6 | oui |

### .escort (6)

| Commande | Niveau | Console |
|---|---|---|
| `.escort addwp` | 5 |  |
| `.escort clearwp` | 5 |  |
| `.escort create` | 5 |  |
| `.escort hidewp` | 2 |  |
| `.escort modwp` | 5 |  |
| `.escort showwp` | 2 |  |

### .event (6)

| Commande | Niveau | Console |
|---|---|---|
| `.event` | 3 | oui |
| `.event disable` | 5 | oui |
| `.event enable` | 5 | oui |
| `.event list` | 3 | oui |
| `.event start` | 4 | oui |
| `.event stop` | 4 | oui |

### .gm (7)

| Commande | Niveau | Console |
|---|---|---|
| `.gm` | 2 |  |
| `.gm chat` | 1 |  |
| `.gm fly` | 3 |  |
| `.gm ingame` | 0 | oui |
| `.gm list` | 6 | oui |
| `.gm setview` | 2 |  |
| `.gm visible` | 2 |  |

### .go (18)

| Commande | Niveau | Console |
|---|---|---|
| `.go` | 2 |  |
| `.go alterac` | 2 |  |
| `.go arathi` | 2 |  |
| `.go creature` | 2 |  |
| `.go forward` | 2 |  |
| `.go graveyard` | 2 |  |
| `.go grid` | 2 |  |
| `.go object` | 2 |  |
| `.go relative` | 2 |  |
| `.go target` | 2 |  |
| `.go taxinode` | 2 |  |
| `.go trigger` | 2 |  |
| `.go up` | 2 |  |
| `.go warsong` | 2 |  |
| `.go xy` | 2 |  |
| `.go xyz` | 2 |  |
| `.go xyzo` | 2 |  |
| `.go zonexy` | 2 |  |

### .gobject (20)

| Commande | Niveau | Console |
|---|---|---|
| `.gobject add` | 5 |  |
| `.gobject customanim` | 3 |  |
| `.gobject delete` | 5 |  |
| `.gobject despawn` | 3 |  |
| `.gobject despawnanim` | 3 |  |
| `.gobject info` | 2 |  |
| `.gobject move` | 3 |  |
| `.gobject near` | 2 |  |
| `.gobject reset` | 3 |  |
| `.gobject respawn` | 3 |  |
| `.gobject select` | 2 |  |
| `.gobject setgostate` | 3 |  |
| `.gobject setlootstate` | 3 |  |
| `.gobject spawnanim` | 3 |  |
| `.gobject target` | 3 |  |
| `.gobject tmpadd` | 3 |  |
| `.gobject toggle` | 3 |  |
| `.gobject turn` | 3 |  |
| `.gobject ufinfo` | 5 |  |
| `.gobject use` | 3 |  |

### .gold (1)

| Commande | Niveau | Console |
|---|---|---|
| `.gold remove` | 4 |  |

### .group (4)

| Commande | Niveau | Console |
|---|---|---|
| `.group additem` | 3 |  |
| `.group replenish` | 3 |  |
| `.group revive` | 3 |  |
| `.group summon` | 3 |  |

### .groupspell (2)

| Commande | Niveau | Console |
|---|---|---|
| `.groupspell add` | 6 | oui |
| `.groupspell rule` | 6 | oui |

### .guild (7)

| Commande | Niveau | Console |
|---|---|---|
| `.guild create` | 3 | oui |
| `.guild delete` | 4 | oui |
| `.guild invite` | 3 | oui |
| `.guild rank` | 3 | oui |
| `.guild rename` | 4 | oui |
| `.guild showlog` | 3 | oui |
| `.guild uninvite` | 3 | oui |

### .honor (5)

| Commande | Niveau | Console |
|---|---|---|
| `.honor add` | 4 |  |
| `.honor addkill` | 4 |  |
| `.honor reset` | 4 |  |
| `.honor setrp` | 4 |  |
| `.honor show` | 2 |  |

### .instance (11)

| Commande | Niveau | Console |
|---|---|---|
| `.instance continents` | 4 | oui |
| `.instance getdata` | 3 |  |
| `.instance groupunbind` | 4 |  |
| `.instance listbinds` | 3 |  |
| `.instance perfinfos` | 4 |  |
| `.instance savedata` | 4 |  |
| `.instance setdata` | 3 |  |
| `.instance smartrebind` | 2 |  |
| `.instance stats` | 4 | oui |
| `.instance switch` | 4 |  |
| `.instance unbind` | 3 |  |

### .learn (13)

| Commande | Niveau | Console |
|---|---|---|
| `.learn` | 5 |  |
| `.learn all` | 6 |  |
| `.learn all_crafts` | 3 |  |
| `.learn all_default` | 2 |  |
| `.learn all_gm` | 3 |  |
| `.learn all_items` | 3 |  |
| `.learn all_lang` | 1 |  |
| `.learn all_myclass` | 5 |  |
| `.learn all_myspells` | 5 |  |
| `.learn all_mytalents` | 5 |  |
| `.learn all_mytaxis` | 2 |  |
| `.learn all_recipes` | 3 |  |
| `.learn all_trainer` | 3 |  |

### .list (12)

| Commande | Niveau | Console |
|---|---|---|
| `.list auras` | 2 |  |
| `.list clicktomove` | 2 | oui |
| `.list creature` | 2 | oui |
| `.list exploredareas` | 2 |  |
| `.list hostilerefs` | 2 |  |
| `.list item` | 2 | oui |
| `.list maps` | 2 | oui |
| `.list movegens` | 2 |  |
| `.list object` | 2 | oui |
| `.list talents` | 2 |  |
| `.list threat` | 2 |  |
| `.list visibleguids` | 2 |  |

### .lookup (25)

| Commande | Niveau | Console |
|---|---|---|
| `.lookup account email` | 4 |  |
| `.lookup account ip` | 4 |  |
| `.lookup account iponline` | 4 |  |
| `.lookup account name` | 3 |  |
| `.lookup area` | 2 | oui |
| `.lookup creature` | 2 | oui |
| `.lookup creaturemodel` | 2 | oui |
| `.lookup event` | 2 | oui |
| `.lookup faction` | 2 | oui |
| `.lookup guild` | 1 | oui |
| `.lookup item` | 2 | oui |
| `.lookup itemset` | 2 | oui |
| `.lookup object` | 2 | oui |
| `.lookup player account` | 3 |  |
| `.lookup player character` | 2 |  |
| `.lookup player email` | 4 |  |
| `.lookup player ip` | 4 |  |
| `.lookup player name` | 1 |  |
| `.lookup pool` | 2 | oui |
| `.lookup quest` | 2 | oui |
| `.lookup skill` | 2 | oui |
| `.lookup sound` | 2 | oui |
| `.lookup spell` | 2 | oui |
| `.lookup taxinode` | 2 | oui |
| `.lookup tele` | 2 | oui |

### .mmap (9)

| Commande | Niveau | Console |
|---|---|---|
| `.mmap` | 6 |  |
| `.mmap connect` | 6 |  |
| `.mmap loadedtiles` | 3 |  |
| `.mmap loc` | 3 |  |
| `.mmap path` | 3 |  |
| `.mmap reload` | 6 |  |
| `.mmap stats` | 3 |  |
| `.mmap testarea` | 3 |  |
| `.mmap unload` | 6 |  |

### .modify (53)

| Commande | Niveau | Console |
|---|---|---|
| `.modify accessories` | 4 |  |
| `.modify agility` | 4 |  |
| `.modify ap` | 4 |  |
| `.modify arcane` | 4 |  |
| `.modify armor` | 4 |  |
| `.modify aspeed` | 2 |  |
| `.modify block` | 4 |  |
| `.modify boundrad` | 4 |  |
| `.modify bwalk` | 2 |  |
| `.modify castspeed` | 4 |  |
| `.modify combreach` | 4 |  |
| `.modify crit` | 4 |  |
| `.modify dodge` | 4 |  |
| `.modify drunk` | 2 |  |
| `.modify emotestate` | 3 |  |
| `.modify energy` | 3 |  |
| `.modify exhaustion` | 3 |  |
| `.modify faction` | 3 |  |
| `.modify fire` | 4 |  |
| `.modify fly` | 2 |  |
| `.modify frost` | 4 |  |
| `.modify gender` | 3 |  |
| `.modify haircolor` | 4 |  |
| `.modify hairstyle` | 4 |  |
| `.modify holy` | 4 |  |
| `.modify honor` | 4 |  |
| `.modify hp` | 3 |  |
| `.modify intellect` | 4 |  |
| `.modify mainspeed` | 4 |  |
| `.modify mana` | 3 |  |
| `.modify money` | 4 |  |
| `.modify morph` | 3 |  |
| `.modify mount` | 3 |  |
| `.modify nature` | 4 |  |
| `.modify offspeed` | 4 |  |
| `.modify parry` | 4 |  |
| `.modify rage` | 3 |  |
| `.modify rangeap` | 4 |  |
| `.modify rangecrit` | 4 |  |
| `.modify rangespeed` | 4 |  |
| `.modify rep` | 4 |  |
| `.modify scale` | 3 |  |
| `.modify shadow` | 4 |  |
| `.modify skincolor` | 4 |  |
| `.modify speed` | 2 |  |
| `.modify spellcrit` | 4 |  |
| `.modify spellpower` | 4 |  |
| `.modify spirit` | 4 |  |
| `.modify stamina` | 4 |  |
| `.modify strength` | 4 |  |
| `.modify swim` | 2 |  |
| `.modify tp` | 4 |  |
| `.modify xprate` | 0 |  |

### .npc (49)

| Commande | Niveau | Console |
|---|---|---|
| `.npc additem` | 5 |  |
| `.npc addweapon` | 3 |  |
| `.npc aiinfo` | 1 |  |
| `.npc allowattack` | 3 |  |
| `.npc allowmove` | 3 |  |
| `.npc delitem` | 5 |  |
| `.npc despawn` | 3 |  |
| `.npc evade` | 3 |  |
| `.npc follow` | 3 |  |
| `.npc group add` | 5 |  |
| `.npc group addrel` | 5 |  |
| `.npc group del` | 5 |  |
| `.npc group link` | 5 |  |
| `.npc info` | 1 |  |
| `.npc move` | 3 |  |
| `.npc playemote` | 2 |  |
| `.npc say` | 2 |  |
| `.npc set displayid` | 3 |  |
| `.npc set entry` | 3 |  |
| `.npc set faction` | 3 |  |
| `.npc set flag` | 3 |  |
| `.npc set level` | 3 |  |
| `.npc set movetype` | 3 |  |
| `.npc set reactstate` | 3 |  |
| `.npc set respawntime` | 3 |  |
| `.npc set wanderdistance` | 3 |  |
| `.npc spawn add` | 5 |  |
| `.npc spawn addentry` | 5 |  |
| `.npc spawn delete` | 5 |  |
| `.npc spawn info` | 1 |  |
| `.npc spawn load` | 3 |  |
| `.npc spawn move` | 5 |  |
| `.npc spawn set auras` | 5 |  |
| `.npc spawn set deathstate` | 5 |  |
| `.npc spawn set displayid` | 5 |  |
| `.npc spawn set emotestate` | 5 |  |
| `.npc spawn set entry` | 5 |  |
| `.npc spawn set movetype` | 5 |  |
| `.npc spawn set respawntime` | 5 |  |
| `.npc spawn set sheathstate` | 5 |  |
| `.npc spawn set standstate` | 5 |  |
| `.npc spawn set wanderdistance` | 5 |  |
| `.npc spawn unload` | 3 |  |
| `.npc summon` | 3 |  |
| `.npc tame` | 3 |  |
| `.npc textemote` | 2 |  |
| `.npc unfollow` | 3 |  |
| `.npc whisper` | 2 |  |
| `.npc yell` | 2 |  |

### .partybot (19)

| Commande | Niveau | Console |
|---|---|---|
| `.partybot add` | 6 |  |
| `.partybot aoe` | 6 |  |
| `.partybot attackstart` | 6 |  |
| `.partybot attackstop` | 6 |  |
| `.partybot caststart` | 6 |  |
| `.partybot caststop` | 6 |  |
| `.partybot ccmark` | 6 |  |
| `.partybot clearmarks` | 6 |  |
| `.partybot clone` | 6 |  |
| `.partybot cometome` | 6 |  |
| `.partybot focusmark` | 6 |  |
| `.partybot load` | 6 |  |
| `.partybot pause` | 6 |  |
| `.partybot pull` | 6 |  |
| `.partybot remove` | 6 |  |
| `.partybot setrole` | 6 |  |
| `.partybot unequip` | 6 |  |
| `.partybot unpause` | 6 |  |
| `.partybot usegobject` | 6 |  |

### .pbcast (3)

| Commande | Niveau | Console |
|---|---|---|
| `.pbcast` | 6 | oui |
| `.pbcast setthreads` | 6 | oui |
| `.pbcast stats` | 6 | oui |

### .pdump (2)

| Commande | Niveau | Console |
|---|---|---|
| `.pdump load` | 6 | oui |
| `.pdump write` | 6 | oui |

### .pet (7)

| Commande | Niveau | Console |
|---|---|---|
| `.pet delete` | 3 | oui |
| `.pet info` | 1 |  |
| `.pet learnspell` | 5 |  |
| `.pet list` | 3 | oui |
| `.pet loyalty` | 3 |  |
| `.pet rename` | 3 | oui |
| `.pet unlearnspell` | 5 |  |

### .pool (4)

| Commande | Niveau | Console |
|---|---|---|
| `.pool` | 3 | oui |
| `.pool list` | 3 |  |
| `.pool spawns` | 3 |  |
| `.pool update` | 6 |  |

### .quest (4)

| Commande | Niveau | Console |
|---|---|---|
| `.quest add` | 4 |  |
| `.quest complete` | 4 |  |
| `.quest remove` | 3 |  |
| `.quest status` | 3 |  |

### .reload (112)

| Commande | Niveau | Console |
|---|---|---|
| `.reload account_banned` | 6 | oui |
| `.reload all` | 6 | oui |
| `.reload all_area` | 6 | oui |
| `.reload all_gossips` | 6 | oui |
| `.reload all_item` | 6 | oui |
| `.reload all_locales` | 6 | oui |
| `.reload all_loot` | 6 | oui |
| `.reload all_npc` | 6 | oui |
| `.reload all_quest` | 6 | oui |
| `.reload all_scripts` | 6 | oui |
| `.reload all_spell` | 6 | oui |
| `.reload anticheat` | 6 | oui |
| `.reload areatrigger_involvedrelation` | 5 | oui |
| `.reload areatrigger_tavern` | 5 | oui |
| `.reload areatrigger_teleport` | 5 | oui |
| `.reload autobroadcast` | 6 | oui |
| `.reload character_pet` | 6 | oui |
| `.reload cinematic_waypoints` | 5 | oui |
| `.reload command` | 6 | oui |
| `.reload conditions` | 5 | oui |
| `.reload config` | 6 | oui |
| `.reload creature` | 5 | oui |
| `.reload creature_ai_events` | 5 | oui |
| `.reload creature_battleground` | 5 | oui |
| `.reload creature_display_info_addon` | 5 | oui |
| `.reload creature_groups` | 5 | oui |
| `.reload creature_involvedrelation` | 5 | oui |
| `.reload creature_loot_template` | 5 | oui |
| `.reload creature_onkill_reputation` | 5 | oui |
| `.reload creature_questrelation` | 5 | oui |
| `.reload creature_spells` | 5 | oui |
| `.reload creature_spells_scripts` | 5 | oui |
| `.reload creature_template` | 5 | oui |
| `.reload disenchant_loot_template` | 5 | oui |
| `.reload event_scripts` | 5 | oui |
| `.reload exploration_basexp` | 5 | oui |
| `.reload fishing_loot_template` | 5 | oui |
| `.reload game_graveyard_zone` | 5 | oui |
| `.reload game_tele` | 5 | oui |
| `.reload game_weather` | 5 | oui |
| `.reload gameobject` | 5 | oui |
| `.reload gameobject_battleground` | 5 | oui |
| `.reload gameobject_involvedrelation` | 5 | oui |
| `.reload gameobject_loot_template` | 5 | oui |
| `.reload gameobject_questrelation` | 5 | oui |
| `.reload gameobject_requirement` | 5 | oui |
| `.reload gameobject_scripts` | 5 | oui |
| `.reload gameobject_template` | 5 | oui |
| `.reload generic_scripts` | 5 | oui |
| `.reload gossip_menu` | 5 | oui |
| `.reload gossip_menu_option` | 5 | oui |
| `.reload gossip_scripts` | 5 | oui |
| `.reload instance_buff_removal` | 5 | oui |
| `.reload ip_banned` | 6 | oui |
| `.reload item_enchantment_template` | 5 | oui |
| `.reload item_loot_template` | 5 | oui |
| `.reload item_required_target` | 5 | oui |
| `.reload item_template` | 5 | oui |
| `.reload locales_creature` | 5 | oui |
| `.reload locales_gameobject` | 5 | oui |
| `.reload locales_gossip_menu_option` | 5 | oui |
| `.reload locales_item` | 5 | oui |
| `.reload locales_page_text` | 5 | oui |
| `.reload locales_points_of_interest` | 5 | oui |
| `.reload locales_quest` | 5 | oui |
| `.reload mail_loot_template` | 5 | oui |
| `.reload mangos_string` | 5 | oui |
| `.reload map_loot_disabled` | 5 | oui |
| `.reload map_template` | 5 | oui |
| `.reload npc_gossip` | 5 | oui |
| `.reload npc_text` | 5 | oui |
| `.reload npc_trainer` | 5 | oui |
| `.reload npc_vendor` | 5 | oui |
| `.reload page_text` | 5 | oui |
| `.reload pet_name_generation` | 5 | oui |
| `.reload petitions` | 6 | oui |
| `.reload pickpocketing_loot_template` | 5 | oui |
| `.reload player_factionchange_items` | 5 | oui |
| `.reload player_factionchange_mounts` | 5 | oui |
| `.reload player_factionchange_quests` | 5 | oui |
| `.reload player_factionchange_reputations` | 5 | oui |
| `.reload player_factionchange_spells` | 5 | oui |
| `.reload points_of_interest` | 5 | oui |
| `.reload quest_end_scripts` | 5 | oui |
| `.reload quest_greeting` | 5 | oui |
| `.reload quest_start_scripts` | 5 | oui |
| `.reload quest_template` | 5 | oui |
| `.reload reference_loot_template` | 5 | oui |
| `.reload reputation_reward_rate` | 5 | oui |
| `.reload reputation_spillover_template` | 5 | oui |
| `.reload reserved_name` | 6 | oui |
| `.reload skill_fishing_base_level` | 5 | oui |
| `.reload skinning_loot_template` | 5 | oui |
| `.reload spell_area` | 5 | oui |
| `.reload spell_chain` | 5 | oui |
| `.reload spell_disabled` | 5 | oui |
| `.reload spell_elixir` | 5 | oui |
| `.reload spell_group` | 5 | oui |
| `.reload spell_group_stack_rules` | 5 | oui |
| `.reload spell_learn_spell` | 5 | oui |
| `.reload spell_mod` | 5 | oui |
| `.reload spell_pet_auras` | 5 | oui |
| `.reload spell_proc_event` | 5 | oui |
| `.reload spell_proc_item_enchant` | 5 | oui |
| `.reload spell_script_target` | 5 | oui |
| `.reload spell_scripts` | 5 | oui |
| `.reload spell_target_position` | 5 | oui |
| `.reload spell_template` | 5 | oui |
| `.reload spell_threats` | 5 | oui |
| `.reload taxi_path_transitions` | 5 | oui |
| `.reload trainer_greeting` | 5 | oui |
| `.reload variables` | 5 | oui |

### .reset (7)

| Commande | Niveau | Console |
|---|---|---|
| `.reset all` | 6 | oui |
| `.reset honor` | 5 | oui |
| `.reset items` | 6 | oui |
| `.reset level` | 5 | oui |
| `.reset spells` | 5 | oui |
| `.reset stats` | 5 | oui |
| `.reset talents` | 3 | oui |

### .send (7)

| Commande | Niveau | Console |
|---|---|---|
| `.send items` | 6 | oui |
| `.send mail` | 1 | oui |
| `.send mass items` | 6 | oui |
| `.send mass mail` | 6 | oui |
| `.send mass money` | 6 | oui |
| `.send message` | 6 | oui |
| `.send money` | 6 | oui |

### .server (17)

| Commande | Niveau | Console |
|---|---|---|
| `.server corpses` | 6 | oui |
| `.server exit` | 7 | oui |
| `.server idlerestart` | 6 | oui |
| `.server idlerestart cancel` | 6 | oui |
| `.server idleshutdown` | 6 | oui |
| `.server idleshutdown cancel` | 6 | oui |
| `.server info` | 0 | oui |
| `.server log filter` | 7 | oui |
| `.server log level` | 7 | oui |
| `.server motd` | 0 | oui |
| `.server plimit` | 6 | oui |
| `.server resetallraids` | 6 | oui |
| `.server restart` | 6 | oui |
| `.server restart cancel` | 6 | oui |
| `.server set motd` | 6 | oui |
| `.server shutdown` | 6 | oui |
| `.server shutdown cancel` | 6 | oui |

### .service (1)

| Commande | Niveau | Console |
|---|---|---|
| `.service del_characters` | 6 | oui |

### .spamer (3)

| Commande | Niveau | Console |
|---|---|---|
| `.spamer list` | 2 | oui |
| `.spamer mute` | 1 | oui |
| `.spamer unmute` | 2 | oui |

### .spell (4)

| Commande | Niveau | Console |
|---|---|---|
| `.spell effects` | 3 | oui |
| `.spell iconfix` | 5 | oui |
| `.spell info` | 3 | oui |
| `.spell search` | 3 | oui |

### .tele (5)

| Commande | Niveau | Console |
|---|---|---|
| `.tele` | 2 |  |
| `.tele add` | 5 |  |
| `.tele del` | 5 | oui |
| `.tele group` | 2 |  |
| `.tele name` | 2 | oui |

### .ticket (24)

| Commande | Niveau | Console |
|---|---|---|
| `.ticket` | 2 | oui |
| `.ticket assign` | 2 | oui |
| `.ticket close` | 2 | oui |
| `.ticket closedlist` | 2 | oui |
| `.ticket comment` | 2 | oui |
| `.ticket complete` | 2 | oui |
| `.ticket counter` | 2 |  |
| `.ticket delete` | 3 | oui |
| `.ticket escalate` | 2 |  |
| `.ticket escalatedlist` | 3 | oui |
| `.ticket list` | 2 | oui |
| `.ticket next` | 2 |  |
| `.ticket notify` | 2 |  |
| `.ticket onlinelist` | 2 | oui |
| `.ticket previous` | 2 |  |
| `.ticket reload` | 2 | oui |
| `.ticket reset` | 6 | oui |
| `.ticket response append` | 2 | oui |
| `.ticket response appendln` | 2 | oui |
| `.ticket response reset` | 3 | oui |
| `.ticket togglesystem` | 6 | oui |
| `.ticket unassign` | 2 | oui |
| `.ticket viewid` | 2 | oui |
| `.ticket viewname` | 2 | oui |

### .trigger (3)

| Commande | Niveau | Console |
|---|---|---|
| `.trigger` | 2 | oui |
| `.trigger active` | 2 |  |
| `.trigger near` | 2 |  |

### .unban (3)

| Commande | Niveau | Console |
|---|---|---|
| `.unban account` | 6 | oui |
| `.unban character` | 6 | oui |
| `.unban ip` | 6 | oui |

### .unit (23)

| Commande | Niveau | Console |
|---|---|---|
| `.unit aiinfo` | 1 |  |
| `.unit factioninfo` | 1 |  |
| `.unit info` | 1 |  |
| `.unit moveinfo` | 1 |  |
| `.unit show class` | 1 |  |
| `.unit show combattimer` | 1 |  |
| `.unit show createspell` | 1 |  |
| `.unit show emotestate` | 1 |  |
| `.unit show form` | 1 |  |
| `.unit show gender` | 1 |  |
| `.unit show miscflags` | 1 |  |
| `.unit show moveflags` | 1 |  |
| `.unit show npcflags` | 1 |  |
| `.unit show powertype` | 1 |  |
| `.unit show race` | 1 |  |
| `.unit show sheathstate` | 1 |  |
| `.unit show standstate` | 1 |  |
| `.unit show unitflags` | 1 |  |
| `.unit show unitstate` | 1 |  |
| `.unit show visflags` | 1 |  |
| `.unit speedinfo` | 1 |  |
| `.unit statinfo` | 1 |  |
| `.unit ufinfo` | 5 |  |

### .unlearn (4)

| Commande | Niveau | Console |
|---|---|---|
| `.unlearn` | 3 |  |
| `.unlearn all_crafts` | 3 |  |
| `.unlearn all_gm` | 3 |  |
| `.unlearn all_recipes` | 3 |  |

### .video (3)

| Commande | Niveau | Console |
|---|---|---|
| `.video` | 3 |  |
| `.video expendables` | 3 |  |
| `.video turn` | 3 |  |

### .wareffort (5)

| Commande | Niveau | Console |
|---|---|---|
| `.wareffort getresource` | 5 | oui |
| `.wareffort info` | 5 | oui |
| `.wareffort setgongtime` | 5 | oui |
| `.wareffort setresource` | 5 | oui |
| `.wareffort setstage` | 5 | oui |

### .world (3)

| Commande | Niveau | Console |
|---|---|---|
| `.world cansee` | 6 |  |
| `.world detail` | 6 |  |
| `.world update` | 6 |  |

### .wp (4)

| Commande | Niveau | Console |
|---|---|---|
| `.wp add` | 5 |  |
| `.wp export` | 6 |  |
| `.wp modify` | 5 |  |
| `.wp show` | 2 |  |

