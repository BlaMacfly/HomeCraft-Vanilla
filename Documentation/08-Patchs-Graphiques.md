# Patchs graphiques HD (VanillaHD) — analyse

Question de l'utilisateur (2026-10-15) : intégrer https://github.com/VanillaHD pour moderniser le rendu.

## Ce que c'est (vérifié sur GitHub le 2026-10-15)

- Organisation « VanillaHD », projet **« led by and created for Turtle WoW »** : trois dépôts contenant l'arborescence brute de patchs MPQ (pas d'archive MPQ publiée, aucune release) : Patch-A « personnages et créatures » (1,76 Go dans git, dossiers Character, Creature, Item, spells, Interface, **DBFilesClient**), Patch-B « monde et environnement » (430 Mo), Patch-C « sons et musiques ». Dernière activité : août 2025. Avertissement en tête : « NOT UP TO DATE FOR 1.18 YET » (1.18 = client modifié de Turtle WoW).
- **Aucune licence**, aucune mention de provenance. Les dossiers de races incluent BloodElf, Goblin, Worgen (races ajoutées par Turtle WoW) : ce sont des modèles et textures rétroportés depuis des versions ultérieures de World of Warcraft, donc des ressources Blizzard.
- Patch-A modifie **13 DBC** côté client : `ChrRaces`, `CharSections`, `CharHairGeosets`, `CharVariations`, `CreatureDisplayInfo`, `CreatureDisplayInfoExtra`, `CreatureModelData`, `CreatureSoundData`, `ItemDisplayInfo`, `NPCSounds`, `SoundEntries`, etc.

## Conséquences pour HomeCraft

1. **Jamais dans le package** : ressources Blizzard sans licence, 2,2 Go. Même logique que pour le client : l'utilisateur les installe lui-même.
2. **Compatibilité serveur à tester** : VMaNGOS extrait ses DBC depuis les MPQ du client, patchs compris. Un patch qui modifie `ChrRaces` ou `CreatureDisplayInfo` pour Turtle WoW (races supplémentaires, nouveaux identifiants d'affichage) peut faire échouer le chargement des DBC par mangosd ou décaler des apparences. À vérifier avec les fichiers réels avant toute recommandation ; le risque est réel puisque le projet cible un client 1.17/1.18 modifié, pas le 1.12.1 d'origine.
3. **Sans effet sur la localisation** : les textes restent ceux du client frFR, sauf si un patch écrase des DBC contenant des chaînes (aucun des 13 listés ne porte de texte visible, à l'exception de noms internes).
4. **Obtention** : il faut construire les MPQ depuis l'arborescence (outil StormLib, MIT) ou récupérer les `patch-*.MPQ` distribués avec le client Turtle WoW.

## Correction importante (2026-10-15) : les patchs n'atteignent jamais le serveur

Lecture de `contrib/extractor/System.cpp` (commit 53d24d5) : `CONF_mpq_list = { model.MPQ, dbc.MPQ, terrain.MPQ, patch.MPQ, patch-2.MPQ }`. Les extracteurs VMaNGOS **ignorent** `patch-3.MPQ`, `patch-A.MPQ`, etc. Conséquences :
- la comparaison « avec / sans patch-3 » du 2026-10-14 ne prouvait rien (le fichier n'était pas lu) ; l'effet de `patch-3.MPQ` sur le client reste inconnu ;
- un patch client ne peut pas désynchroniser les données du serveur : le point 2 ci-dessus (« compatibilité serveur à tester ») devient : un patch qui modifie des DBC **ne sera pas reconnu par le serveur** (races, objets, sorts ajoutés côté client = inutilisables), tandis que les changements purement visuels ou sonores fonctionnent.

## Gestionnaire de patchs client (réalisé le 2026-10-15)

Onglet « Patchs » du launcher (`Launcher/src/patches.rs`) : liste des archives de `Client/Data` (officielles verrouillées ; `patch-X.MPQ` avec X chiffre 3-9 ou lettre A-Z = chargé par le client ; autre nom = signalé « non chargé »), activer/désactiver par renommage `.off`, ajouter (sélecteur de fichier, copie dans Data, renommage automatique vers le prochain `patch-X` libre si le nom n'est pas chargeable), supprimer avec confirmation, ouverture du dossier. Refus pendant que le jeu tourne. Aucun patch n'est fourni ni téléchargé. Mode test : `--do patchoff <nom>` / `patchon <nom>`.

## Construction et test des patchs VanillaHD (2026-10-15)

L'utilisateur a fourni `Patch-A-main.zip` (1,4 Go, 21 396 fichiers extraits, 2,48 Go) et `Patch-B-main.zip` (361 Mo, 3 750 fichiers, 666 Mo). Python n'étant pas installé, le constructeur MPQ a été intégré au launcher (`mpqbuild.rs`, bibliothèque `wow-mpq`). Résultats :

| Patch | Archive créée | Fichiers | Taille | Durée | Client 1.12.1 frFR |
|---|---|---|---|---|---|
| B (monde) | `patch-4.MPQ` | 3 748 | 392 Mo | ≈ 1 min | démarre, écran de connexion OK (capture `Logs/wow-patchB.png`) |
| A (personnages, créatures, **13 DBC**) | `patch-5.MPQ` | 21 393 | 1,54 Go | 3 min | démarre, écran de connexion OK avec A + B actifs |

Reste à vérifier **en jeu** par l'utilisateur : création de personnage (DBC `ChrRaces`/`CharSections` modifiées), apparence des créatures et objets, stabilité. En cas de problème : désactiver le patch depuis l'onglet Patchs. Rappel : ces archives ne sont jamais incluses dans le package.

## Ce que le launcher peut offrir légalement (proposition initiale, remplacée par la section ci-dessus)

Un gestionnaire de **patchs client** : l'utilisateur dépose ses fichiers `patch-X.MPQ` dans `Client/Data`, le launcher les liste, les active ou désactive (renommage en `.MPQ.off`), et **ré-extrait les DBC du serveur** quand la liste active change, pour garder client et serveur cohérents, avec un test de chargement (démarrage de mangosd) avant de valider. Aucune ressource n'est fournie ni téléchargée par le launcher.

## Verdict

Pas d'intégration dans le package. Si l'utilisateur fournit les MPQ VanillaHD, test de compatibilité avec VMaNGOS à mener (extraction des DBC, démarrage de mangosd, création de personnage, affichage des créatures) ; si concluant, prise en charge via le gestionnaire de patchs ci-dessus. Alternatives sans risque juridique ni technique : DXVK (déjà en option), résolution et champ de vision.
