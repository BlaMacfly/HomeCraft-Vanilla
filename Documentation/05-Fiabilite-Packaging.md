# HomeCraft Edition BC — Fiabilité et packaging portable

Date : 2026-10-17. Décision de l'utilisateur : « Continue sans test en jeu : fiabilité (sauvegarde, restauration,
réparation), puis packaging portable. »

## 1. Fiabilité : `Tests/Test-Phase5.ps1` — **24 / 24 OK** (13:35)

Adapté du Vanilla (bases `tbc*`, dumps `tbccharacters.sql` / `tbcrealmd.sql`, compléments `00_` et `50_`). Le test
refuse de démarrer si un launcher Vanilla, WotLK ou BC tourne (vérifié : refus propre pendant que le Vanilla tournait).

| Domaine | Contrôles |
|---|---|
| Sauvegarde | archive créée (tbccharacters.sql, tbcrealmd.sql, backup.json), 7 comptes au manifeste ; base déjà en service utilisée et laissée active ; base arrêtée : MariaDB temporaire arrêtée ensuite |
| Vérification | 0 fichier manquant, 0 altéré (manifeste régénéré) |
| Réparation | base `tbclogs` supprimée → réimportée ; `mangosd.conf` effacé → régénéré ; personnages intacts |
| Réimport du monde | modification annulée ; compléments `00_homecraft_schema_locales_gameobject.sql` et `50_homecraft_frFR.sql` journalisés ; traduction HomeCraft revenue (quête 11551) ; schéma `locales_gameobject` corrigé ; > 600 messages serveur en français ; personnages et comptes intacts |
| Restauration | compte témoin supprimé puis revenu ; sauvegarde de sécurité créée avant |
| Diagnostic | zip exporté, mots de passe masqués (`WorldDatabaseInfo = "<masqué>"`), `aiplayerbot.conf` et rapports de plantage inclus |
| Fin | aucun processus serveur restant |

Défaut trouvé au premier passage (3 échecs de restauration, « Accès refusé (os error 5) ») : **défaut du test, pas
du launcher**. `Start-Process -ArgumentList` ne cite pas les arguments, et le chemin du projet contient un espace
(« HomeCraft BC ») : le launcher recevait `C:\Users\Marty\Desktop\HomeCraft`, un dossier, que Windows refuse d'ouvrir
comme un fichier. Corrigé dans le test (guillemets) ; le launcher indique désormais le chemin dans ses erreurs de
restauration (« archive introuvable ou invalide : … »).

## 2. Package portable : `Build/New-Package.ps1 -Zip`

`Build/package/HomeCraft-Edition-BC-v0.1.0/` : **534 fichiers, 639 Mo** ; archive **194 Mo** (35 s).

- Contenu : launcher signé, `Server/mangos-tbc` (sans `.pdb`, sans `aiplayerbot.conf` généré, sans `Crashes`),
  `Server/Config` vide (modèles recopiés par le launcher), dumps TBC-DB, compléments, MariaDB allégée, HeidiSQL,
  scripts (`Tools/Scripts`), documentation, `LISEZMOI.txt`, `LICENCE.txt`, `LICENCES/` (CMaNGOS et PlayerBots GPL-2 +
  COPYRIGHT, TBC-DB GPL-3, traductions AzerothCore / SylvaniaCore / mod-playerbots GPL-2, MariaDB, HeidiSQL,
  `SOURCES.md` avec commits exacts et crates Rust), `Sources/` (launcher, outils de localisation, scripts Build),
  `Build/files-manifest.json`, `SHA256SUMS.txt`.
- Garde-fous : le script échoue si une dépendance GPL/LGPL est liée au launcher, ou si un fichier interdit est présent
  (client, MPQ, `UserData`, `Logs`, `Server/Data`, DBC, tuiles mmaps/vmaps, identifiants, réglages, rapports de plantage).
- Licence HomeCraft (`LICENCE.txt`) corrigée : les compléments de `Server/Database/custom` (correctif de schéma,
  traductions reprises de projets GPL-2) sont des composants tiers ; AzerothCore, SylvaniaCore et mod-playerbots sont
  sous **GPL-2** (et non AGPL-3 comme écrit d'abord : vérifié dans leurs fichiers LICENSE).

## 3. Reste à faire

- `Tests/Test-Package.ps1` (copie vierge dans `%USERPROFILE%\HomeCraftTestBC`, premier démarrage complet : import,
  extraction, mmaps, PlayerBots ; arrêt brutal puis redémarrage ; ~30 min) : **non lancé**, un launcher HomeCraft
  (Vanilla puis WotLK) tournait. À lancer quand aucun launcher n'est ouvert.
- Manuel utilisateur BC (`Documentation/Manuel-Utilisateur.md`) : à rédiger ; `LISEZMOI.txt` couvre l'essentiel.
- Test en jeu par l'utilisateur.
