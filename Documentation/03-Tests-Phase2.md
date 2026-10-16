# Phase 2 — journal de tests

## État du prototype au 2026-10-14

| Tâche | Résultat |
|---|---|
| 2.1 Composants figés et vérifiés | OK (`Build/manifest.json`, SHA-256 figés) |
| 2.2 MariaDB 11.8.9 portable + import des 4 bases | OK (12 s, aucune migration manquante) |
| 2.3 Extraction dbc / maps / vmaps | OK (158 DBC dans `Server/Data/5875/dbc`, 2 429 maps, 6 082 fichiers vmaps). mmaps : génération lancée à 14:05 (16 threads), durée attendue : plusieurs heures |
| 2.3 Démarrage complet de mangosd | OK : « World initialized » en 10 s, locale DBC détectée **frFR**, 8085 en écoute. Messages console du serveur en français |
| Compte de test | créé via la console (`account create`, `account set gmlevel … 6`) — identifiants dans `UserData/compte-local.txt` |
| Client | `realmlist.wtf` → 127.0.0.1, `WTF/Config.wtf` : locale frFR, fenêtré |

Harnais : `Tools/Run-World.ps1` (laisse tourner MariaDB + realmd + mangosd ; console via `Tools/Send-Console.ps1` ; arrêt en créant `Logs/server/stop`).

## Blocage détecté avant le test en jeu (2026-10-15, 00:10)

Le client frFR fourni annonce le build **5880** (écran de connexion), non accepté ni par `allowed_clients` (realmd) ni par la liste interne de mangosd `{5875, 6005, 6141}`. Sans correctif, le royaume serait apparu hors ligne puis la connexion refusée. Corrigé : `allowed_clients` + `realmbuilds` (launcher et scripts), `StrictVersionCheck = 0`, et `Build/Patch-MangosdBuilds.ps1` (6141 → 5880 dans `mangosd.exe`). Voir `02-Construction.md`.

## Premier test en jeu de l'utilisateur (2026-10-15, 08:40)

- Connexion réussie avec un compte créé depuis le launcher (`BLAMACFLY`, locale frFR détectée côté serveur), personnage « Taelia » (humaine paladin, niveau 1) créé : la chaîne complète launcher → realmd → mangosd → client 5880 fonctionne.
- Incident : passage en mode fenêtré depuis les options vidéo du jeu → **écran noir** (fenêtre visible un instant à l'appui sur la touche Windows). Cause : `Config.wtf` réécrit par le jeu avec `gxResolution 1920x1080` sur un écran 1920×1080 et sans `gxWindow` ; la fenêtre avec bordures dépasse l'écran et le client 1.12.1 ne s'en remet pas. Correctifs : `Config.wtf` remis en fenêtré 1600×900 ; le launcher borne désormais la résolution à l'écran (`client::safe_resolution`), passe en fenêtré par défaut, et dispose d'un bouton **FERMER LE JEU** pendant la partie.

## Test 2.4 — à réaliser en jeu (procédure)

Préalable : `Run-World.ps1` actif (vérifier `Logs/server/mangosd.console.log` : « World initialized »).

1. Lancer `Client/WoW.exe`. Écran de connexion attendu **en français**, version 1.12.1 (5875) en bas à gauche.
2. Se connecter avec le compte de `UserData/compte-local.txt`. Royaume « Vanilla Solo » doit apparaître et être sélectionnable.
3. Créer un personnage (n'importe quelle classe), entrer dans le monde. Noter : textes de l'interface, nom de la zone, infobulles des PNJ et objets en français ?
4. Parler à un PNJ donneur de quête, accepter une quête : titre et description en français ? (couverture attendue ~88 %)
5. Compagnons : taper dans le chat
   - `.partybot add tank` puis `.partybot add healer` puis `.partybot add dps`
   - vérifier qu'ils rejoignent le groupe et suivent
   - attaquer un monstre : les bots participent ? le soigneur soigne ?
   - `.partybot cometome` (sélectionner un bot d'abord), `.partybot pause`, `.partybot unpause`, `.partybot remove` (bot sélectionné)
6. Déplacement longue distance (courir 1 minute) : les bots suivent sans se bloquer ? (sans mmaps, le pathfinding est dégradé : refaire ce test une fois les mmaps générées)
7. Déconnexion / reconnexion : le personnage est bien sauvegardé (vérifier aussi `SELECT name, level FROM characters.characters`).

Noter tout texte anglais rencontré (fenêtre, quête, PNJ) pour l'inventaire de la Phase 3.

| AHBot | Actif au redémarrage de 14:13 : 6 392 objets chargés, hôtel Alliance (fid 55), cible 19 176 enchères ; 4 185 enchères (3 099 objets distincts) 2 min après le démarrage, remplissage par le timer de 60 s |

Ajouter au test en jeu : **8.** ouvrir l'hôtel des ventes d'une capitale de l'Alliance (Forgefer, Hurlevent, Darnassus) et vérifier que des potions, composants, recettes et équipements verts sont en vente, avec des prix lisibles.

## Points ouverts

- `patch-3.MPQ` : **sans effet** sur les DBC (158 fichiers identiques avec et sans) — conservé tel quel.
- AHBot : un seul hôtel servi (faction du dernier personnage joué, Alliance par défaut) ; doublons dus au tirage aléatoire ; journal console très verbeux (« AHBot::AddItem ») à réduire.
- mmaps : **terminées** à 14:39 (1 783 fichiers, 1,96 Go, 34 min sur 16 threads). Serveur redémarré à 14:40 avec mmaps actives (arrêt propre en moins d'une seconde avec la fermeture de l'entrée standard, code 0).
