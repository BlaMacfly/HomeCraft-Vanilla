# Inventaire de la localisation française (serveur)

Généré le 2026-10-15 11:45 par `Tests/Test-Localisation.ps1` sur la base monde VMaNGOS (snapshot 4641790). loc2 = frFR.

## Couverture par table (entrées de référence, entrées traduites en français, pourcentage)

| Donnée | Référence (anglais) | Traduit frFR | Couverture |
|---|---|---|---|
| Quêtes — titre | 4433 | 4305 | 97.1 % |
| Quêtes — description | 3672 | 3626 | 98.7 % |
| Quêtes — objectifs | 3670 | 3629 | 98.9 % |
| Quêtes — texte de remise | 4221 | 4221 | 100 % |
| Quêtes — texte de progression | 3267 | 3267 | 100 % |
| Créatures — nom | 10381 | 9452 | 91.1 % |
| Créatures — sous-titre | 2492 | 2409 | 96.7 % |
| Objets — nom | 17707 | 17556 | 99.1 % |
| Objets — description | 719 | 697 | 96.9 % |
| Objets du monde — nom | 9368 | 7991 | 85.3 % |
| Textes diffusés (dialogues, cris) | 11076 | 9533 | 86.1 % |
| Pages de livres | 1257 | 1212 | 96.4 % |
| Options de dialogue PNJ | 1339 | 132 | 9.9 % |
| Points d'intérêt | 250 | 195 | 78 % |
| Zones (noms) | 1081 | 1081 | 100 % |
| Chaînes serveur (messages système) | 976 | 924 | 94.7 % |

## Qualité des textes français

- Titres de quête français contenant au moins un caractère accenté : 1708 / 4305 (preuve que l'encodage UTF-8 et les accents sont présents).
- Descriptions de quête invalides en UTF-8 : 0.
- Descriptions de quête utilisant le tutoiement (« tu », « ton »…) : 8 ; le vouvoiement : 3064. Le client officiel vouvoie ; les textes au tutoiement sont d'origine communautaire (voir analyse AzerothCore PR 28044).

## Exemples de quêtes sans titre français (restent en anglais)

```
73	<TXT> No Reward
108	<TXT> Mystery Reward
137	<NYI> <TXT> Stormwind Commendation
241	<TEST> HEY MISTER WILSON!
242	<UNUSED>
259	<UNUSED>
260	<UNUSED>
316	<UNUSED>
352	<UNUSED>
390	<UNUSED>
406	<UNUSED>
462	<UNUSED>
490	<UNUSED>
497	<UNUSED>
534	<UNUSED>
```

## Quêtes, dialogues de PNJ et créatures : état après les compléments HomeCraft (2026-10-15)

- Quêtes réelles (hors entrées techniques « <UNUSED> », « <nyi> », « REUSE », « DEPRECATED », « <TXT> », quêtes de test) : 4277 ; **0 sans titre ou texte français**. Les 138 titres et les textes manquants ont été traduits par HomeCraft (`Server/Database/custom/locales_quest_homecraft.sql`, traductions **non officielles**, balises du jeu conservées, textes officiels jamais écrasés). Les entrées techniques restent en anglais : le joueur ne les voit jamais.
- Dialogues de PNJ (fenêtres de discussion, `npc_text` → `broadcast_text`) : **0 sans texte français** (les 6 manquants, grognements d'animaux et plantes de Gangrebois, sont dans le même fichier).
- Noms de créatures présentes dans le monde : 7433 entrées, **19 sans nom français** après `locales_creature_homecraft.sql`. Règle appliquée, identique à la localisation officielle (« Maréchal Dughan », « Garde Thomas ») : titre traduit, nom propre conservé ; 2 883 des 2 886 noms traités sont donc identiques à l'anglais (noms propres), 3 titres traduits (« Chef Galgosh », « Chef Tho'grun », « M. Bigglesworth »), 20 entrées techniques (déclencheurs invisibles) laissées de côté.
- Ces compléments sont importés automatiquement par le launcher (fichiers `Server/Database/custom/*.sql` appliqués après la base monde) ; `Tools/Import-Translations.ps1` reste disponible pour des traductions futures au format TSV.

## Ce qui est couvert par le client frFR lui-même (hors base)

Interface (menus, fenêtres, création de personnage, inventaire, talents, infobulles, messages système, journal de combat), noms et descriptions des sorts et des compétences, noms des zones affichés à l'écran, noms des races et classes : fournis par les DBC et les fichiers FrameXML/GlobalStrings du client français 1.12.1. Le serveur détecte `DBC.Locale = frFR` à partir des DBC extraites de ce client.

## Éléments restant en anglais ou partiellement traduits

- Les quêtes, créatures et objets sans entrée loc2 (voir tableau) sont servis en anglais par le serveur. Pour les créatures, une large part des entrées non traduites sont des créatures techniques ou invisibles (déclencheurs, totems, objets de script) que le joueur ne rencontre pas nommées.
- Options de dialogue PNJ : le faible taux de `locales_gossip_menu_option` est trompeur, la plupart des options tirent leur texte de `broadcast_text` (traduit à 86 %) via `option_broadcast_text`.
- Les messages de la console et des commandes GM (`.partybot`, etc.) : `mangos_string` partiellement traduit, le reste en anglais.
- Les noms des compagnons PlayerBots générés par le serveur sont des noms neutres.

Méthode de complément possible : importer un pack de traduction sous licence compatible (aucun identifié à ce jour : UDBFR/MangosZero sans licence), ou traduire manuellement les entrées manquantes dans `locales_*` via l'explorateur de base ou HeidiSQL.