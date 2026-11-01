# Commandes MJ et administrateur — HomeCraft Edition Vanilla

Liste des commandes du serveur **CMaNGOS Classic** livré avec HomeCraft (commit `ff53dfc`, module playerbots `39d4700`), extraite de son code source (`src/game/Chat/Chat.cpp`, script `Build/doc-tools/cmdtree.pl`) : **517 commandes**. La première partie décrit en français celles qui servent vraiment ; l'annexe donne la liste complète avec le niveau requis.

## Comment les utiliser

- **Dans le jeu** : tapez la commande dans la fenêtre de discussion, précédée d'un point, par exemple `.gm on`. Il faut un compte dont le niveau MJ est suffisant (onglet **Comptes** du launcher).
- **Dans la console du serveur** : onglet **Journaux** du launcher, champ « Commande console mangosd », **sans** le point (`account onlinelist`). Seules les commandes marquées « Console : oui » y fonctionnent ; la console a tous les droits.
- `.commands` liste les commandes permises à votre compte ; `.help <commande>` affiche son aide (en anglais).

## Niveaux d'accès

| Niveau | Nom dans CMaNGOS | Nombre de commandes de ce niveau |
|---|---|---|
| 0 | Joueur | 13 (`.save`, `.dismount`, `.start`, `.server info`, `.commands`, `.bot`…) |
| 1 | Modérateur | 61 (téléportations, `.modify` de base, annonces) |
| 2 | Maître de jeu | 120 (créatures, événements, apparences, réputation, `.rndbot`, `.ahbot`) |
| 3 | Administrateur | 309 (objets, sorts, quêtes, niveaux, mode MJ, réinitialisations) |
| 4 | Console | 14 (création et gestion des comptes, `server exit`) : console du serveur uniquement |

Un compte de **niveau 3** (celui que le launcher propose pour jouer seul) a accès à toutes les commandes du jeu. Seules celles du niveau 4 restent réservées à la console, et le launcher les utilise lui-même pour l'onglet Comptes. En multijoueur, donnez le niveau **0** à vos amis.

> **Attention** : ces commandes modifient votre partie, parfois définitivement (objets, niveaux, quêtes, monde). Faites une sauvegarde (onglet Sauvegardes) avant de vous en servir sérieusement. Les commandes `.npc add`, `.npc delete`, `.gobject add/delete` et `.tele add` écrivent dans la base monde : « Réimporter la base monde » (Paramètres) les annule.

---

## Les plus utiles

### Mode maître de jeu

| Commande | Niveau | Effet |
|---|---|---|
| `.gm on` / `.gm off` | 3 | Active ou désactive le mode MJ (invulnérable, ignoré par les monstres, balise <MJ>). Le réglage « Mode MJ à la connexion » de l'onglet Monde décide de l'état à la connexion. |
| `.gm visible on` / `off` | 3 | Rend le MJ visible ou invisible des autres. |
| `.gm fly on` / `off` | 3 | Vol libre. |
| `.gm chat on` / `off` | 3 | Balise <MJ> dans la discussion. |
| `.gm ingame` | 3 | Liste les MJ connectés. |

### Se déplacer

| Commande | Niveau | Effet |
|---|---|---|
| `.tele <lieu>` | 1 | Téléportation vers un lieu nommé (noms en anglais : `.tele Stormwind`, `.tele Orgrimmar`, `.tele Ironforge`). |
| `.lookup tele <texte>` | 1 | Cherche les lieux dont le nom contient ce texte. |
| `.tele name <joueur> <lieu>` | 1 | Téléporte un autre personnage. |
| `.tele group <lieu>` | 1 | Téléporte tout votre groupe, bots compris. |
| `.tele add <nom>` | 3 | Enregistre votre position comme lieu de téléportation (écrit dans la base monde). |
| `.go xyz <x> <y> [z] [carte]` | 1 | Téléportation à des coordonnées. |
| `.go creature <guid>` / `.go object <guid>` | 1 | Va jusqu'à une créature ou un objet du monde. |
| `.appear <joueur>` | 1 | Vous rend auprès d'un personnage (bot compris). |
| `.summon <joueur>` | 1 | Fait venir un personnage auprès de vous. |
| `.groupgo <joueur>` | 1 | Fait venir le groupe d'un personnage. |
| `.recall` | 1 | Revient à l'endroit d'avant la dernière téléportation. |
| `.gps` | 1 | Affiche votre position (carte, zone, coordonnées). |
| `.start` | 0 | Retour au point de départ de votre race. |
| `.neargrave` | 3 | Va au cimetière le plus proche. |
| `.taxicheat on` / `off` | 1 | Tous les trajets aériens connus. |
| `.explorecheat 1` / `0` | 3 | Révèle (1) ou masque (0) toute la carte du monde. |

### Personnage

| Commande | Niveau | Effet |
|---|---|---|
| `.levelup [n]` | 3 | Gagne `n` niveaux (1 par défaut ; négatif pour descendre). |
| `.modify money <cuivre>` | 1 | Ajoute de l'argent en pièces de cuivre : `10000` = 1 or. Négatif pour retirer. |
| `.modify speed <valeur>` / `.modify aspeed <valeur>` | 1 | Vitesse de course / toutes les vitesses (1 = normale). |
| `.modify hp <n>` / `.modify mana <n>` | 1 | Points de vie / de mana actuels. |
| `.modify scale <valeur>` | 1 | Taille du personnage (1 = normale). |
| `.modify morph <apparence>` / `.demorph` | 2 | Change d'apparence / reprend la sienne. |
| `.modify rep <faction> <valeur>` | 2 | Réputation auprès d'une faction. |
| `.modify gender` | 2 | Change le sexe du personnage. |
| `.revive` | 3 | Ressuscite (la cible ou vous). |
| `.die` | 3 | Tue la cible. |
| `.unaura <id\|all>` / `.aura <id>` | 3 | Retire / applique une aura. |
| `.cast <id de sort>` | 3 | Lance un sort sur la cible. |
| `.cooldown clear` | 3 | Réinitialise les temps de recharge. |
| `.repairitems` | 2 | Répare tout l'équipement. |
| `.dismount` | 0 | Descend de monture. |
| `.reset talents` / `spells` / `stats` / `level` | 3 | Réinitialise talents, sorts, caractéristiques ou niveau. |
| `.honor add <points>` | 2 | Ajoute des points d'honneur. |
| `.pinfo [nom]` | 2 | Informations sur un personnage (compte, niveau, argent, temps de jeu). |
| `.save` | 0 | Enregistre votre personnage maintenant. |

### Objets, sorts, métiers, quêtes

| Commande | Niveau | Effet |
|---|---|---|
| `.lookup item <texte>` | 3 | Cherche un objet par nom et donne son identifiant (cliquable). |
| `.additem <id ou lien> [nombre]` | 3 | Donne un objet. |
| `.additemset <id d'ensemble>` | 3 | Donne toutes les pièces d'un ensemble. |
| `.lookup spell <texte>` | 3 | Cherche un sort et donne son identifiant. |
| `.learn <id de sort>` / `.unlearn <id>` | 3 | Apprend / oublie un sort. |
| `.learn all_myclass` | 3 | Tous les sorts de votre classe. |
| `.learn all_recipes <métier>` | 2 | Toutes les recettes d'un métier. |
| `.maxskill` | 3 | Toutes vos compétences au maximum pour votre niveau. |
| `.setskill <id> <niveau> [max]` | 3 | Fixe une compétence (métier, arme). |
| `.lookup quest <texte>` | 3 | Cherche une quête et donne son identifiant. |
| `.quest add <id>` / `complete <id>` / `remove <id>` | 3 | Ajoute, termine ou retire une quête du journal. |
| `.lookup creature` / `object` / `area <texte>` | 3 / 3 / 1 | Cherche une créature, un objet du monde, une zone. |

### Créatures et monde

| Commande | Niveau | Effet |
|---|---|---|
| `.npc info` | 3 | Informations sur la créature ciblée (identifiant, guid, faction…). |
| `.npc add <id>` / `.npc delete` | 2 | Ajoute / supprime une créature **permanente** (base monde). |
| `.npc say` / `yell` / `whisper <texte>` | 1 | Fait parler la créature ciblée. |
| `.npc follow` / `unfollow` | 2 | La créature ciblée vous suit. |
| `.npc tame` | 2 | Apprivoise la bête ciblée (chasseur). |
| `.respawn` | 3 | Fait réapparaître les créatures et objets proches. |
| `.damage <n>` | 3 | Inflige des dégâts à la cible. |
| `.instance listbinds` / `.instance unbind <carte\|all>` | 3 | Liste / supprime vos sauvegardes d'instances (donjons et raids). |
| `.event list` / `start <id>` / `stop <id>` | 2 | Événements du monde (fêtes, invasions…). |
| `.showarea <id>` | 3 | Révèle une zone de la carte. |
| `.wchange <type> <intensité>` | 3 | Change la météo. |

### Bots et hôtel des ventes

| Commande | Niveau | Effet |
|---|---|---|
| `.bot add` / `remove` / `list` / `group` / `prepare` | 0 | Compagnons bots (manuel, §10). |
| `.rndbot stats` (console : `rndbot stats`) | 2 | État des bots aléatoires du monde. |
| `.rndbot reload` | 2 | Relit la configuration des bots (le launcher le fait quand vous enregistrez les réglages). |
| `.ahbot status` / `rebuild` / `reload` | 3 | État, regarnissage, rechargement de l'hôtel des ventes automatique. |

### Serveur (jeu ou console)

| Commande | Niveau | Effet |
|---|---|---|
| `.server info` | 0 | Version, joueurs connectés, temps de fonctionnement. |
| `.saveall` | 1 | Enregistre tous les personnages connectés. |
| `.announce <texte>` / `.notify <texte>` | 1 | Message à tous (discussion / fenêtre). |
| `.kick <nom>` | 2 | Déconnecte un personnage. |
| `.mute <nom> <minutes>` / `.unmute <nom>` | 1 | Rend muet / rend la parole. |
| `.server shutdown <secondes>` / `.server restart <secondes>` | 3 | Arrêt / redémarrage différé. Préférez le bouton ARRÊTER LE SERVEUR du launcher. |

### Console du serveur uniquement (niveau 4, onglet Journaux, sans point)

| Commande | Effet |
|---|---|
| `account create <nom> <mot de passe>` | Crée un compte (le launcher le fait depuis l'onglet Comptes). |
| `account set gmlevel <nom> <niveau>` | Change le niveau MJ d'un compte. |
| `account set password <nom> <mdp> <mdp>` | Change le mot de passe. |
| `account delete <nom>` | Supprime le compte et ses personnages. |
| `account onlinelist` | Comptes connectés. |
| `server exit` | Arrête le serveur (préférez ARRÊTER LE SERVEUR). |

---

## Annexe : les 517 commandes du serveur

Niveau requis et disponibilité dans la console, par famille. Générée depuis le code source ; l'aide de chaque commande est donnée en jeu par `.help <commande>`.

### Commandes simples

| Commande | Niveau | Console |
|---|---|---|
| `.additem` | 3 |  |
| `.additemset` | 3 |  |
| `.announce` | 1 | oui |
| `.appear` | 1 |  |
| `.aura` | 3 |  |
| `.bank` | 3 |  |
| `.bot` | 0 |  |
| `.commands` | 0 | oui |
| `.damage` | 3 |  |
| `.demorph` | 2 |  |
| `.die` | 3 |  |
| `.dismount` | 0 |  |
| `.distance` | 3 |  |
| `.explorecheat` | 3 |  |
| `.goname` | 1 |  |
| `.gps` | 1 |  |
| `.groupgo` | 1 |  |
| `.guid` | 2 |  |
| `.help` | 0 | oui |
| `.hidearea` | 3 |  |
| `.itemmove` | 2 |  |
| `.kick` | 2 | oui |
| `.levelup` | 3 |  |
| `.linkgrave` | 3 |  |
| `.loadscripts` | 3 | oui |
| `.los` | 3 |  |
| `.maxskill` | 3 |  |
| `.mute` | 1 | oui |
| `.namego` | 1 |  |
| `.neargrave` | 3 |  |
| `.notify` | 1 | oui |
| `.pinfo` | 2 | oui |
| `.quit` | 4 | oui |
| `.recall` | 1 |  |
| `.repairitems` | 2 | oui |
| `.respawn` | 3 |  |
| `.revive` | 3 | oui |
| `.rndbot` | 2 | oui |
| `.save` | 0 |  |
| `.saveall` | 1 | oui |
| `.setskill` | 3 |  |
| `.showarea` | 3 |  |
| `.stable` | 3 |  |
| `.start` | 0 |  |
| `.summon` | 1 |  |
| `.taxicheat` | 1 |  |
| `.unaura` | 3 |  |
| `.unlearn` | 3 |  |
| `.unmute` | 1 | oui |
| `.waterwalk` | 2 |  |
| `.wchange` | 3 |  |
| `.whispers` | 1 |  |
| `.wr` | 0 |  |

### .account (10)

| Commande | Niveau | Console |
|---|---|---|
| `.account` | 0 | oui |
| `.account characters` | 3 | oui |
| `.account create` | 4 | oui |
| `.account delete` | 4 | oui |
| `.account lock` | 0 | oui |
| `.account onlinelist` | 4 | oui |
| `.account password` | 0 | oui |
| `.account set addon` | 3 | oui |
| `.account set gmlevel` | 4 | oui |
| `.account set password` | 4 | oui |

### .ahbot (5)

| Commande | Niveau | Console |
|---|---|---|
| `.ahbot` | 2 | oui |
| `.ahbot item` | 3 | oui |
| `.ahbot rebuild` | 3 | oui |
| `.ahbot reload` | 3 | oui |
| `.ahbot status` | 3 | oui |

### .auction (5)

| Commande | Niveau | Console |
|---|---|---|
| `.auction` | 3 |  |
| `.auction alliance` | 3 |  |
| `.auction goblin` | 3 |  |
| `.auction horde` | 3 |  |
| `.auction item` | 3 | oui |

### .ban (5)

| Commande | Niveau | Console |
|---|---|---|
| `.ban account` | 3 | oui |
| `.ban character` | 3 | oui |
| `.ban ip` | 3 | oui |
| `.ban note` | 3 | oui |
| `.ban warn` | 3 | oui |

### .baninfo (3)

| Commande | Niveau | Console |
|---|---|---|
| `.baninfo account` | 3 | oui |
| `.baninfo character` | 3 | oui |
| `.baninfo ip` | 3 | oui |

### .banlist (3)

| Commande | Niveau | Console |
|---|---|---|
| `.banlist account` | 3 | oui |
| `.banlist character` | 3 | oui |
| `.banlist ip` | 3 | oui |

### .bg (2)

| Commande | Niveau | Console |
|---|---|---|
| `.bg start` | 2 |  |
| `.bg stop` | 2 |  |

### .cast (5)

| Commande | Niveau | Console |
|---|---|---|
| `.cast` | 3 |  |
| `.cast back` | 3 |  |
| `.cast dist` | 3 |  |
| `.cast self` | 3 |  |
| `.cast target` | 3 |  |

### .channel (2)

| Commande | Niveau | Console |
|---|---|---|
| `.channel list` | 1 |  |
| `.channel static` | 1 |  |

### .character (8)

| Commande | Niveau | Console |
|---|---|---|
| `.character deleted delete` | 4 | oui |
| `.character deleted list` | 4 | oui |
| `.character deleted old` | 4 | oui |
| `.character deleted restore` | 3 | oui |
| `.character erase` | 4 | oui |
| `.character level` | 3 | oui |
| `.character rename` | 2 | oui |
| `.character reputation` | 2 | oui |

### .combat (2)

| Commande | Niveau | Console |
|---|---|---|
| `.combat list` | 0 |  |
| `.combat stop` | 2 |  |

### .cooldown (3)

| Commande | Niveau | Console |
|---|---|---|
| `.cooldown clear` | 3 |  |
| `.cooldown clearclientside` | 3 |  |
| `.cooldown list` | 3 |  |

### .debug (59)

| Commande | Niveau | Console |
|---|---|---|
| `.debug anim` | 2 |  |
| `.debug areatriggers` | 1 |  |
| `.debug bg` | 3 |  |
| `.debug byte` | 3 |  |
| `.debug chatfreeze` | 3 | oui |
| `.debug dbscript` | 3 | oui |
| `.debug dbscriptguided` | 3 | oui |
| `.debug dbscriptsourced` | 3 | oui |
| `.debug dbscripttargeted` | 3 | oui |
| `.debug debugflags` | 3 | oui |
| `.debug getitemstate` | 3 |  |
| `.debug getitemvalue` | 3 |  |
| `.debug getvaluebyindex` | 3 |  |
| `.debug getvaluebyname` | 3 |  |
| `.debug listupdatefields` | 3 |  |
| `.debug lootrecipient` | 2 |  |
| `.debug moditemvalue` | 3 |  |
| `.debug modvalue` | 3 |  |
| `.debug moveflag` | 3 |  |
| `.debug opcodeinchistory` | 3 | oui |
| `.debug opcodeouthistory` | 3 | oui |
| `.debug packetlog` | 3 | oui |
| `.debug perf gridsloaded` | 3 |  |
| `.debug perf tempspawn` | 3 |  |
| `.debug play cinematic` | 1 |  |
| `.debug play music` | 1 |  |
| `.debug play petdismiss` | 1 |  |
| `.debug play sound` | 1 |  |
| `.debug script command` | 2 |  |
| `.debug script help` | 2 |  |
| `.debug send buyerror` | 3 |  |
| `.debug send channelnotify` | 3 |  |
| `.debug send chatmmessage` | 3 |  |
| `.debug send equiperror` | 3 |  |
| `.debug send opcode` | 3 |  |
| `.debug send poi` | 3 |  |
| `.debug send qfailedresult` | 3 |  |
| `.debug send qinvalidmsg` | 3 |  |
| `.debug send qpartymsg` | 3 |  |
| `.debug send sellerror` | 3 |  |
| `.debug send spellVisual` | 3 |  |
| `.debug send spellfail` | 3 |  |
| `.debug send worldstate` | 3 |  |
| `.debug setaurastate` | 3 |  |
| `.debug setitemvalue` | 3 |  |
| `.debug setvaluebyindex` | 3 |  |
| `.debug setvaluebyname` | 3 |  |
| `.debug spawn list` | 2 |  |
| `.debug spawn respawn` | 2 |  |
| `.debug spellcheck` | 4 | oui |
| `.debug spellcoefs` | 3 | oui |
| `.debug spellmods` | 3 |  |
| `.debug taxi` | 3 |  |
| `.debug transports` | 3 | oui |
| `.debug utf8overflow` | 3 | oui |
| `.debug uws` | 3 |  |
| `.debug visibility atclient` | 1 |  |
| `.debug visibility isvisible` | 1 |  |
| `.debug waypoint` | 3 |  |

### .event (4)

| Commande | Niveau | Console |
|---|---|---|
| `.event` | 2 | oui |
| `.event list` | 2 | oui |
| `.event start` | 2 | oui |
| `.event stop` | 2 | oui |

### .gm (9)

| Commande | Niveau | Console |
|---|---|---|
| `.gm` | 3 |  |
| `.gm chat` | 3 |  |
| `.gm fly` | 3 |  |
| `.gm ingame` | 3 | oui |
| `.gm list` | 3 | oui |
| `.gm mountup` | 3 |  |
| `.gm setview` | 3 |  |
| `.gm unkillable` | 3 |  |
| `.gm visible` | 3 |  |

### .go (12)

| Commande | Niveau | Console |
|---|---|---|
| `.go` | 1 |  |
| `.go creature` | 1 |  |
| `.go graveyard` | 1 |  |
| `.go grid` | 1 |  |
| `.go next` | 3 |  |
| `.go object` | 1 |  |
| `.go taxinode` | 1 |  |
| `.go trigger` | 1 |  |
| `.go warp` | 1 |  |
| `.go xy` | 1 |  |
| `.go xyz` | 1 |  |
| `.go zonexy` | 1 |  |

### .gobject (10)

| Commande | Niveau | Console |
|---|---|---|
| `.gobject activate` | 2 |  |
| `.gobject add` | 2 |  |
| `.gobject delete` | 2 |  |
| `.gobject despawn` | 2 |  |
| `.gobject move` | 2 |  |
| `.gobject near` | 2 |  |
| `.gobject nearspawned` | 2 |  |
| `.gobject respawn` | 2 |  |
| `.gobject target` | 2 |  |
| `.gobject turn` | 2 |  |

### .guild (5)

| Commande | Niveau | Console |
|---|---|---|
| `.guild create` | 2 | oui |
| `.guild delete` | 2 | oui |
| `.guild invite` | 2 | oui |
| `.guild rank` | 2 | oui |
| `.guild uninvite` | 2 | oui |

### .honor (4)

| Commande | Niveau | Console |
|---|---|---|
| `.honor add` | 2 |  |
| `.honor addkill` | 2 |  |
| `.honor show` | 2 |  |
| `.honor update` | 2 |  |

### .instance (4)

| Commande | Niveau | Console |
|---|---|---|
| `.instance listbinds` | 3 |  |
| `.instance savedata` | 3 |  |
| `.instance stats` | 3 | oui |
| `.instance unbind` | 3 |  |

### .learn (10)

| Commande | Niveau | Console |
|---|---|---|
| `.learn` | 3 |  |
| `.learn all` | 3 |  |
| `.learn all_crafts` | 2 |  |
| `.learn all_default` | 1 |  |
| `.learn all_gm` | 2 |  |
| `.learn all_lang` | 1 |  |
| `.learn all_myclass` | 3 |  |
| `.learn all_myspells` | 3 |  |
| `.learn all_mytalents` | 3 |  |
| `.learn all_recipes` | 2 |  |

### .link (5)

| Commande | Niveau | Console |
|---|---|---|
| `.link add` | 3 |  |
| `.link check` | 3 |  |
| `.link edit` | 3 |  |
| `.link remove` | 3 |  |
| `.link toggle` | 3 |  |

### .list (6)

| Commande | Niveau | Console |
|---|---|---|
| `.list areatriggers` | 3 |  |
| `.list auras` | 3 |  |
| `.list creature` | 3 | oui |
| `.list item` | 3 | oui |
| `.list object` | 3 | oui |
| `.list talents` | 3 |  |

### .lookup (20)

| Commande | Niveau | Console |
|---|---|---|
| `.lookup account all` | 2 | oui |
| `.lookup account email` | 2 | oui |
| `.lookup account ip` | 2 | oui |
| `.lookup account name` | 2 | oui |
| `.lookup area` | 1 | oui |
| `.lookup creature` | 3 | oui |
| `.lookup event` | 2 | oui |
| `.lookup faction` | 3 | oui |
| `.lookup item` | 3 | oui |
| `.lookup itemset` | 3 | oui |
| `.lookup object` | 3 | oui |
| `.lookup player account` | 2 | oui |
| `.lookup player email` | 2 | oui |
| `.lookup player ip` | 2 | oui |
| `.lookup pool` | 2 | oui |
| `.lookup quest` | 3 | oui |
| `.lookup skill` | 3 | oui |
| `.lookup spell` | 3 | oui |
| `.lookup taxinode` | 3 | oui |
| `.lookup tele` | 1 | oui |

### .loot (2)

| Commande | Niveau | Console |
|---|---|---|
| `.loot fullstats` | 2 | oui |
| `.loot stats` | 2 | oui |

### .mmap (7)

| Commande | Niveau | Console |
|---|---|---|
| `.mmap` | 3 |  |
| `.mmap loadedtiles` | 2 |  |
| `.mmap loc` | 2 |  |
| `.mmap path` | 2 |  |
| `.mmap stats` | 2 |  |
| `.mmap testarea` | 2 |  |
| `.mmap testheight` | 2 |  |

### .modify (42)

| Commande | Niveau | Console |
|---|---|---|
| `.modify agility` | 3 |  |
| `.modify ap` | 3 |  |
| `.modify arcaneresist` | 3 |  |
| `.modify armor` | 3 |  |
| `.modify aspeed` | 1 |  |
| `.modify block` | 3 |  |
| `.modify bwalk` | 1 |  |
| `.modify crit` | 3 |  |
| `.modify dodge` | 3 |  |
| `.modify drunk` | 1 |  |
| `.modify energy` | 1 |  |
| `.modify faction` | 1 |  |
| `.modify fireresist` | 3 |  |
| `.modify frostresist` | 3 |  |
| `.modify gender` | 2 |  |
| `.modify holyresist` | 3 |  |
| `.modify honor` | 1 |  |
| `.modify hp` | 1 |  |
| `.modify intellect` | 3 |  |
| `.modify mana` | 1 |  |
| `.modify meleehaste` | 3 |  |
| `.modify money` | 1 |  |
| `.modify morph` | 2 |  |
| `.modify mount` | 1 |  |
| `.modify natureresist` | 3 |  |
| `.modify parry` | 3 |  |
| `.modify rage` | 1 |  |
| `.modify rangeap` | 3 |  |
| `.modify rangehaste` | 3 |  |
| `.modify rep` | 2 |  |
| `.modify scale` | 1 |  |
| `.modify shadowresist` | 3 |  |
| `.modify speed` | 1 |  |
| `.modify spellcrit` | 3 |  |
| `.modify spellhaste` | 3 |  |
| `.modify spellpower` | 3 |  |
| `.modify spirit` | 3 |  |
| `.modify stamina` | 3 |  |
| `.modify standstate` | 2 |  |
| `.modify strength` | 3 |  |
| `.modify swim` | 1 |  |
| `.modify tp` | 1 |  |

### .movement (5)

| Commande | Niveau | Console |
|---|---|---|
| `.movement cometome` | 3 |  |
| `.movement debug` | 3 |  |
| `.movement movegens` | 2 |  |
| `.movement movespeed` | 2 |  |
| `.movement print` | 3 |  |

### .npc (37)

| Commande | Niveau | Console |
|---|---|---|
| `.npc add` | 2 |  |
| `.npc additem` | 2 |  |
| `.npc addweapon` | 3 |  |
| `.npc aiinfo` | 2 |  |
| `.npc allowmove` | 3 |  |
| `.npc changeentry` | 3 |  |
| `.npc changelevel` | 2 |  |
| `.npc delete` | 2 |  |
| `.npc delitem` | 2 |  |
| `.npc despawn` | 2 |  |
| `.npc evade` | 2 |  |
| `.npc factionid` | 2 |  |
| `.npc flag` | 2 |  |
| `.npc follow` | 2 |  |
| `.npc formation info` | 2 |  |
| `.npc formation reset` | 2 |  |
| `.npc formation switch` | 2 |  |
| `.npc group info` | 2 |  |
| `.npc info` | 3 |  |
| `.npc move` | 2 |  |
| `.npc name` | 2 |  |
| `.npc playemote` | 3 |  |
| `.npc say` | 1 |  |
| `.npc setmodel` | 2 |  |
| `.npc setmovetype` | 2 |  |
| `.npc showloot` | 2 |  |
| `.npc spawndist` | 2 |  |
| `.npc spawntime` | 2 |  |
| `.npc spelllist` | 2 |  |
| `.npc subname` | 2 |  |
| `.npc tame` | 2 |  |
| `.npc tempspawn` | 2 |  |
| `.npc textemote` | 1 |  |
| `.npc threat` | 3 |  |
| `.npc unfollow` | 2 |  |
| `.npc whisper` | 1 |  |
| `.npc yell` | 1 |  |

### .pdump (2)

| Commande | Niveau | Console |
|---|---|---|
| `.pdump load` | 3 | oui |
| `.pdump write` | 3 | oui |

### .pet (1)

| Commande | Niveau | Console |
|---|---|---|
| `.pet levelloyalty` | 2 | oui |

### .pool (3)

| Commande | Niveau | Console |
|---|---|---|
| `.pool` | 2 | oui |
| `.pool list` | 2 |  |
| `.pool spawns` | 2 |  |

### .quest (3)

| Commande | Niveau | Console |
|---|---|---|
| `.quest add` | 3 |  |
| `.quest complete` | 3 |  |
| `.quest remove` | 3 |  |

### .reload (91)

| Commande | Niveau | Console |
|---|---|---|
| `.reload all` | 3 | oui |
| `.reload all_area` | 3 | oui |
| `.reload all_eventai` | 3 | oui |
| `.reload all_gossips` | 3 | oui |
| `.reload all_item` | 3 | oui |
| `.reload all_locales` | 3 | oui |
| `.reload all_loot` | 3 | oui |
| `.reload all_npc` | 3 | oui |
| `.reload all_quest` | 3 | oui |
| `.reload all_scripts` | 3 | oui |
| `.reload all_spell` | 3 | oui |
| `.reload areatrigger_involvedrelation` | 3 | oui |
| `.reload areatrigger_tavern` | 3 | oui |
| `.reload areatrigger_teleport` | 3 | oui |
| `.reload command` | 3 | oui |
| `.reload conditions` | 3 | oui |
| `.reload config` | 3 | oui |
| `.reload creature_ai_scripts` | 3 | oui |
| `.reload creature_ai_summons` | 3 | oui |
| `.reload creature_battleground` | 3 | oui |
| `.reload creature_cooldowns` | 3 | oui |
| `.reload creature_involvedrelation` | 3 | oui |
| `.reload creature_loot_template` | 3 | oui |
| `.reload creature_questrelation` | 3 | oui |
| `.reload creature_spell_list` | 3 | oui |
| `.reload creature_template_classlevelstats` | 3 | oui |
| `.reload dbscripts_on_creature_death` | 3 | oui |
| `.reload dbscripts_on_event` | 3 | oui |
| `.reload dbscripts_on_go_use` | 3 | oui |
| `.reload dbscripts_on_gossip` | 3 | oui |
| `.reload dbscripts_on_quest_end` | 3 | oui |
| `.reload dbscripts_on_quest_start` | 3 | oui |
| `.reload dbscripts_on_relay` | 3 | oui |
| `.reload dbscripts_on_spell` | 3 | oui |
| `.reload disenchant_loot_template` | 3 | oui |
| `.reload fishing_loot_template` | 3 | oui |
| `.reload game_graveyard_zone` | 3 | oui |
| `.reload game_tele` | 3 | oui |
| `.reload gameobject_battleground` | 3 | oui |
| `.reload gameobject_involvedrelation` | 3 | oui |
| `.reload gameobject_loot_template` | 3 | oui |
| `.reload gameobject_questrelation` | 3 | oui |
| `.reload gossip_menu` | 3 | oui |
| `.reload gossip_menu_option` | 3 | oui |
| `.reload item_enchantment_template` | 3 | oui |
| `.reload item_loot_template` | 3 | oui |
| `.reload item_required_target` | 3 | oui |
| `.reload locales_areatrigger_teleport` | 3 | oui |
| `.reload locales_creature` | 3 | oui |
| `.reload locales_gameobject` | 3 | oui |
| `.reload locales_gossip_menu_option` | 3 | oui |
| `.reload locales_item` | 3 | oui |
| `.reload locales_npc_text` | 3 | oui |
| `.reload locales_page_text` | 3 | oui |
| `.reload locales_points_of_interest` | 3 | oui |
| `.reload locales_quest` | 3 | oui |
| `.reload locales_questgiver_greeting` | 3 | oui |
| `.reload locales_trainer_greeting` | 3 | oui |
| `.reload mail_loot_template` | 3 | oui |
| `.reload mangos_string` | 3 | oui |
| `.reload npc_gossip` | 3 | oui |
| `.reload npc_text` | 3 | oui |
| `.reload npc_trainer` | 3 | oui |
| `.reload npc_vendor` | 3 | oui |
| `.reload page_text` | 3 | oui |
| `.reload pickpocketing_loot_template` | 3 | oui |
| `.reload points_of_interest` | 3 | oui |
| `.reload quest_template` | 3 | oui |
| `.reload questgiver_greeting` | 3 | oui |
| `.reload reference_loot_template` | 3 | oui |
| `.reload reputation_reward_rate` | 3 | oui |
| `.reload reputation_spillover_template` | 3 | oui |
| `.reload reserved_name` | 3 | oui |
| `.reload skill_fishing_base_level` | 3 | oui |
| `.reload skinning_loot_template` | 3 | oui |
| `.reload spam_records` | 3 | oui |
| `.reload spawn_group` | 3 | oui |
| `.reload spell_affect` | 3 | oui |
| `.reload spell_area` | 3 | oui |
| `.reload spell_chain` | 3 | oui |
| `.reload spell_elixir` | 3 | oui |
| `.reload spell_learn_spell` | 3 | oui |
| `.reload spell_pet_auras` | 3 | oui |
| `.reload spell_proc_event` | 3 | oui |
| `.reload spell_proc_item_enchant` | 3 | oui |
| `.reload spell_script_target` | 3 | oui |
| `.reload spell_target_position` | 3 | oui |
| `.reload spell_threats` | 3 | oui |
| `.reload string_id` | 3 | oui |
| `.reload taxi_shortcuts` | 3 | oui |
| `.reload trainer_greeting` | 3 | oui |

### .reset (8)

| Commande | Niveau | Console |
|---|---|---|
| `.reset all` | 3 | oui |
| `.reset honor` | 3 | oui |
| `.reset level` | 3 | oui |
| `.reset mods` | 3 | oui |
| `.reset spells` | 3 | oui |
| `.reset stats` | 3 | oui |
| `.reset talents` | 3 | oui |
| `.reset taxinodes` | 3 | oui |

### .send (7)

| Commande | Niveau | Console |
|---|---|---|
| `.send items` | 3 | oui |
| `.send mail` | 1 | oui |
| `.send mass items` | 3 | oui |
| `.send mass mail` | 3 | oui |
| `.send mass money` | 3 | oui |
| `.send message` | 3 | oui |
| `.send money` | 3 | oui |

### .server (17)

| Commande | Niveau | Console |
|---|---|---|
| `.server corpses` | 2 | oui |
| `.server exit` | 4 | oui |
| `.server idlerestart` | 3 | oui |
| `.server idlerestart cancel` | 3 | oui |
| `.server idleshutdown` | 3 | oui |
| `.server idleshutdown cancel` | 3 | oui |
| `.server info` | 0 | oui |
| `.server log filter` | 4 | oui |
| `.server log level` | 4 | oui |
| `.server motd` | 0 | oui |
| `.server plimit` | 3 | oui |
| `.server resetallraid` | 3 | oui |
| `.server restart` | 3 | oui |
| `.server restart cancel` | 3 | oui |
| `.server set motd` | 3 | oui |
| `.server shutdown` | 3 | oui |
| `.server shutdown cancel` | 3 | oui |

### .tele (5)

| Commande | Niveau | Console |
|---|---|---|
| `.tele` | 1 |  |
| `.tele add` | 3 |  |
| `.tele del` | 3 | oui |
| `.tele group` | 1 |  |
| `.tele name` | 1 | oui |

### .testing (1)

| Commande | Niveau | Console |
|---|---|---|
| `.testing bags` | 2 |  |

### .ticket (10)

| Commande | Niveau | Console |
|---|---|---|
| `.ticket` | 2 |  |
| `.ticket discard` | 2 |  |
| `.ticket escalate` | 2 |  |
| `.ticket go` | 2 |  |
| `.ticket goname` | 2 |  |
| `.ticket note` | 2 |  |
| `.ticket read` | 2 |  |
| `.ticket resolve` | 2 |  |
| `.ticket sort` | 2 |  |
| `.ticket whisper` | 2 |  |

### .tickets (3)

| Commande | Niveau | Console |
|---|---|---|
| `.tickets` | 2 |  |
| `.tickets list` | 2 |  |
| `.tickets queue` | 3 |  |

### .trigger (3)

| Commande | Niveau | Console |
|---|---|---|
| `.trigger` | 2 | oui |
| `.trigger active` | 2 |  |
| `.trigger near` | 2 |  |

### .unban (3)

| Commande | Niveau | Console |
|---|---|---|
| `.unban account` | 3 | oui |
| `.unban character` | 3 | oui |
| `.unban ip` | 3 | oui |

### .worldstate (9)

| Commande | Niveau | Console |
|---|---|---|
| `.worldstate scourgeinvasion battleswon` | 3 |  |
| `.worldstate scourgeinvasion show` | 3 |  |
| `.worldstate scourgeinvasion startzone` | 3 |  |
| `.worldstate scourgeinvasion state` | 3 |  |
| `.worldstate set` | 3 |  |
| `.worldstate variables` | 3 |  |
| `.worldstate wareffort counter` | 3 |  |
| `.worldstate wareffort phase` | 3 |  |
| `.worldstate wareffort show` | 3 |  |

### .wp (4)

| Commande | Niveau | Console |
|---|---|---|
| `.wp add` | 2 |  |
| `.wp export` | 3 |  |
| `.wp modify` | 2 |  |
| `.wp show` | 2 |  |

