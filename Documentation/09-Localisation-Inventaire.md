# HomeCraft Edition BC — localisation française du serveur

Date : 2026-10-17. Demande de l'utilisateur : « vérifie aussi que le serveur soit bien entièrement en FR » ;
piste donnée : « le projet SylvaniaCore-WoTLK sur github aura toutes les traductions si besoin ».
Mesure faite **sans démarrer de service**, par analyse des dumps SQL (outil `Tools/Localisation/sqlcov-main.rs`),
loc2 = frFR.

## 1. Ce qui est en français sans rien faire

- **Tout ce qui vient du client** (client frFR de l'utilisateur) : interface, sorts et talents, zones, noms et textes
  stockés dans les DBC (extraits des archives frFR : `DBC.Locale = 255` détecte frFR).
- **Base monde TBC-DB** (colonnes `_loc2`) :

| Donnée | Référence (anglais) | frFR dans TBC-DB | Couverture |
|---|---|---|---|
| Quêtes — titre | 6 599 | 6 515 | 98,7 % |
| Quêtes — description | 5 670 | 5 556 | 98,0 % |
| Quêtes — objectifs | 5 643 | 5 606 | 99,3 % |
| Quêtes — texte de remise | 6 167 | 2 078 | **33,7 %** |
| Quêtes — texte de progression | 4 554 | 1 255 | **27,6 %** |
| Créatures — nom | 18 797 | 18 529 | 98,6 % |
| Objets — nom | 30 392 | 23 802 | 78,3 % (dont ~3 900 objets de test / obsolètes) |
| Objets du monde — nom | 14 108 | 12 773 | 90,5 % |
| Textes diffusés (dialogues, cris, PNJ) | 73 040 | 72 980 | 99,9 % |
| Pages de livres | 1 483 | 1 427 | 96,2 % |
| Messages serveur (`mangos_string`) | 952 | **0** | **0 %** |
| Phrases des bots (`ai_playerbot_texts`) | 1 943 | 500 | 25,7 % |

Les textes de scripts (`script_texts`, 2 190 / 2 298 reliés à un texte diffusé) et les options de dialogue
(`gossip_menu_option`, 3 495 / 3 594 reliées) passent par `broadcast_text_locale` : en français.

## 2. Complément HomeCraft : `Server/Database/custom/50_homecraft_frFR.sql`

Généré par l'outil `Tools/Localisation/frmerge` (Rust) à partir de sources **déjà présentes sur ce poste** (aucun
téléchargement) :

| Source | Fichier local | Données utilisées |
|---|---|---|
| VMaNGOS (Vanilla HomeCraft) | `HomeCraft\Build\downloads\db-4641790.zip` → `mangos.sql` | `mangos_string` (format printf identique à CMaNGOS) |
| SylvaniaCore-WoTLK / AzerothCore | `HomeCraft WoTLK\Server\Database\sql\acore_world.sql` | `acore_string` ; quêtes, objets, créatures, objets du monde, pages + `*_locale` frFR |
| mod-playerbots (clone SylvaniaCore) | `…\modules\mod-playerbots\data\sql\playerbots\base\ai_playerbot_texts.sql` + `updates\2025_09_18_00_ai_playerbot_french_texts.sql` | phrases des bots |

Règles (aucun risque d'écraser ou de mal traduire) :
- seuls les textes français **absents** sont remplis (`UPDATE … WHERE loc2 vide`, `INSERT … ON DUPLICATE KEY UPDATE … IF(vide)`) ;
- seulement si le texte anglais de la source est **identique** à celui de TBC-DB pour la même entrée (comparaison
  tolérante : variables du client `$N/$n`, `$C/$c`, `$B/$b` sans casse, espaces multiples réduits) ;
- messages serveur : mêmes spécificateurs printf dans le même ordre (sinon rejet, risque de plantage) ;
- **garde-fou de langue** : certaines entrées « frFR » d'AzerothCore sont de l'anglais (ex. quête 16 :
  « Thanks, &lt;name&gt;! &nbsp;… ») : rejet si entité HTML, texte identique à l'anglais, ou plus de mots-outils
  anglais que français (845 textes de remise écartés ainsi) ;
- caractères hors plan de base exclus (bases en utf8mb3).

Résultat : **5 059 compléments**.

| Donnée | Avant | Ajoutés | Après | Reste en anglais |
|---|---|---|---|---|
| Messages serveur | 0 / 952 | 623 (VMaNGOS) | 65,4 % | 329 (surtout commandes MJ propres à TBC) |
| Phrases des bots | 500 / 1 943 | 271 | 39,7 % | 1 172 |
| Quêtes — titre | 6 515 / 6 599 | 84 | 100 % | 0 |
| Quêtes — texte de remise | 2 078 / 6 167 | 2 112 | 67,9 % | 1 977 |
| Quêtes — texte de progression | 1 255 / 4 554 | 629 | 41,4 % | 2 670 |
| Objets — nom | 23 802 / 30 392 | 720 | 80,7 % | 5 870 (≈ 3 875 objets de test / obsolètes) |
| Objets du monde — nom | 12 773 / 14 108 | 493 | 94,0 % | 842 |
| Créatures — nom | 18 529 / 18 797 | 65 | 98,9 % | 203 |
| Pages de livres | 1 427 / 1 483 | 9 | 96,8 % | 47 |

Le fichier est appliqué automatiquement après l'import de la base monde (launcher `import_dumps`, script
`Tools/Init-Database.ps1`), ou par « Réimporter la base monde… ». **Il n'a pas encore été appliqué à la base
installée ni testé à l'import** (services arrêtés pendant que le Vanilla tournait) : à faire au prochain démarrage.

## 3. Ce qui reste en anglais (aucune source française locale)

- Textes de remise et de progression des quêtes (≈ 4 650) : absents de TBC-DB comme d'AzerothCore, ou texte anglais
  différent entre les deux bases.
- Accueils des donneurs de quêtes (`questgiver_greeting`, 204) et des maîtres (`trainer_greeting`, 664) : aucune
  traduction ni dans TBC-DB ni dans AzerothCore.
- 329 messages serveur (commandes MJ), 1 172 phrases des bots aléatoires.
- Messages des téléporteurs de zone (`areatrigger_teleport`, 72), `gossip_texts` (78), ~100 options de dialogue sans
  texte diffusé.

Pistes : le dépôt GitHub SylvaniaCore-WoTLK (proposé par l'utilisateur) contient peut-être des traductions plus
récentes que le clone local du 2026-10-16 ; traduction HomeCraft des accueils (~870 phrases courtes, souvent
répétées) ; dumps d'autres projets frFR. À décider par l'utilisateur.

## 4. Licences (à trancher avant diffusion)

Le complément est **dérivé** de VMaNGOS (GPL-2), d'AzerothCore / SylvaniaCore (GPL-2, vérifié dans le fichier LICENSE du clone le 2026-10-17) et de mod-playerbots (GPL-2). Il ne peut pas
être couvert par la licence propriétaire HomeCraft (LICENCE.txt, article 1.3 d, qui vise « les traductions
françaises HomeCraft ») : il doit être livré comme composant tiers sous ces licences, avec mention des sources.

## 5. Régénérer

```powershell
cd Tools\Localisation\frmerge
cargo build --release
.\target\release\frmerge.exe <tbcmangos.sql> <vmangos mangos.sql> <acore_world.sql> <bots base.sql> <bots fr.sql> ..\..\..\Server\Database\custom\50_homecraft_frFR.sql
```

Variables de diagnostic : `FRDBG=1` (écarts anglais entre sources), `FRMISS=1` (liste des textes restés en anglais).
