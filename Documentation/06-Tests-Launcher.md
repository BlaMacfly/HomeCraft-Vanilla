# Phase 4 — premier lancement réel du launcher et harnais de tests (2026-10-16)

Le harnais de l'édition Vanilla (`../HomeCraft/Tests/`) est porté dans `Tests/` et exécuté sur le launcher compilé en
phase 3 (`05-Launcher-Rust.md`), serveur de l'utilisateur arrêté. Le launcher testé est **`HomeCraft Edition WoTLK.exe`**
à la racine, lancé depuis la racine (comme le fera l'utilisateur), pas l'exécutable de `Launcher/target/`.

## 1. Harnais `Tests/`

| Script | Rôle |
|---|---|
| `Common-Tests.ps1` | Fonctions partagées (dot-sourcé par les tests) : `Assert-ServicesStopped` (**s'arrête net** si un port 3315/3724/8085 est occupé ou si un `worldserver`/`authserver`/`mysqld`/launcher tourne : règle du projet), `Set-TestSettings` (`auto_mmaps:false` dans `UserData/settings.json`, le reste conservé), `Invoke-Launcher` (lancement avec arguments, délai maximal, journal ajouté pendant l'exécution), `Close-LauncherWindow` (WM_CLOSE à la fenêtre **titrée**, voir §3), lecture de zip, tableau de résultats. |
| `Test-Launcher.ps1` | `--start --quit-after 600` : ouverture des ports, écoute **127.0.0.1 seulement** (toutes les sockets en écoute des processus lancés), « World Initialized », puis fermeture de la fenêtre 20 s après le monde prêt (`-WaitAfterReady`) ou attente de `--quit-after` (`-NoClose`) ; arrêt propre des trois services, aucune ERREUR, ports libérés, aucun orphelin, code de sortie 0. 18 contrôles (dont `CommandServerPort = 0` dans `playerbots.conf`). |
| `Test-Phase5.ps1` | Mode automatique `--do` : sauvegarde (base déjà en service, puis base arrêtée), vérification (`Build/New-FilesManifest.ps1` d'abord), réparation (`worldserver.conf` effacé), restauration avec compte témoin, diagnostic (mots de passe masqués), `--tab backups|bots|settings|modules`, modules (`modoff`/`modon`/`modreset`, persistance à travers `repair`). 36 contrôles. |
| `Test-Play.ps1` | Bouton JOUER de bout en bout (`--play`) avec le client de `Client/` : jeu lancé, capture de la fenêtre, fin du jeu détectée, sauvegarde automatique, arrêt propre. 14 contrôles (§4 ter). |
| `Build/New-FilesManifest.ps1` | Porté de Vanilla : manifeste des fichiers gérés (`Server/acore` hors `.conf` générés / `Crashes` / `.pid`, MySQL portable, dumps et compléments, `Tools` hors HeidiSQL/HermesProxy, launcher, licence). 215 fichiers, 1 341 Mo. |

Journaux des exécutions : `Logs/tests/*.log`.

## 2. Premier lancement réel (`Test-Launcher.ps1`)

Quatre exécutions le 2026-10-16 (14:53, 15:10 à la main, 15:12, 15:24), base `UserData/Database` de la phase 2
(bots `rndbot*` existants), `settings.json` par défaut (**40 bots aléatoires**, nom de royaume `HomeCraft`) sauf
`auto_mmaps:false` ; aucun client dans `Client/` (les données `Server/Data` étant complètes, l'extraction n'est pas
déclenchée : « client absent » est journalisé, le bouton JOUER est grisé, le serveur démarre quand même).

| Étape | 1er lancement (14:53) | Lancements suivants |
|---|---|---|
| `Layout::detect` depuis l'exe à la racine, `short_root()` | racine trouvée ; nom court 8.3 `C:\Users\Marty\Desktop\HOMECR~2` utilisé pour `-c` (pas de jonction créée) | idem |
| MySQL (`mysqld --defaults-file=Server/Config/my.ini`, datadir existant, pas d'`--init-file`) | opérationnelle à **t+10 s** (récupération XA du dernier arrêt) | t+5 s |
| authserver (`realmlist` assurée avant) | t+13 s | t+7 s |
| worldserver : « World Initialized » + port 8085 | **1 min 39 s** (cache disque froid) | **14 à 22 s** (cache chaud) |
| Bots | 40 bots connectés (« Active: 40 », rapport toutes les 5 min dans la console) | idem |
| Fermeture de la fenêtre (WM_CLOSE sur la fenêtre titrée) | — | `server shutdown 1` + fermeture de stdin → worldserver terminé en **1 s**, authserver tué, MySQL `SHUTDOWN` en 1 s ; code de sortie 0 |
| `--quit-after 600` seul (15:12) | — | même séquence d'arrêt à t+600 s, code 0 |
| Écoute | `0.0.0.0:8888` **en plus** des trois ports loopback (voir §3) | 127.0.0.1 seulement après correctif |

Résultat final (15:24) : **17 contrôles OK** (`Logs/tests/Test-Launcher.log`). Les points 1 à 5 et 7 du §4 de
`05-Launcher-Rust.md` sont validés : conf générées identiques en substance à celles de `Tools/New-ServerConfig.ps1`
(`SourceDirectory` pointe sur `Build/src/sylvaniacore-wotlk` au lieu de la jonction `C:\hc`, même dépôt ; l'updater
n'a rien eu à appliquer), `PlayerbotsDatabaseInfo` / `Playerbots.Updates.EnableDatabases` en fin de
`worldserver.conf`, `Warden.Enabled = 0`, `AiPlayerbot.Min/MaxRandomBots = 40`. Le point 6 (bouton JOUER, client
3.3.5a puis 3.4.3 + HermesProxy) reste à faire par l'utilisateur, aucun client n'étant présent sur ce poste de test.

Remarque : MySQL 8 sous Windows affiche **deux processus `mysqld.exe`** (un superviseur et le serveur, fils du
premier) : normal, les deux disparaissent au `SHUTDOWN`.

## 3. Pièges rencontrés

1. **Serveur de commandes TCP de mod-playerbots** : `AiPlayerbot.CommandServerPort = 8888` (valeur du modèle) ouvre
   un port **sur toutes les interfaces** (`tcp::endpoint(tcp::v4(), port)`, `PlayerbotCommandServer.cpp`), sans
   authentification, contraire à la règle « tout sur 127.0.0.1 ». Correctif dans `server.rs::generate_configs`
   (`AiPlayerbot.CommandServerPort = 0`) et dans `Tools/New-ServerConfig.ps1`. Détecté par le contrôle « écoute
   uniquement sur 127.0.0.1 » de `Test-Launcher.ps1` (toutes les sockets en écoute des processus lancés, pas
   seulement les trois ports attendus).
2. **`Process.CloseMainWindow()` vise la mauvaise fenêtre** : winit crée une fenêtre interne sans titre (16 × 16) ;
   .NET peut la prendre pour « fenêtre principale ». Le WM_CLOSE qu'elle reçoit **gèle la boucle d'événements** du
   launcher (plus de repaint : ni l'arrêt des services ni `--quit-after` ne s'exécutent ; `IsHungAppWindow` reste
   faux). Le premier essai a dû être tué après 14 min, laissant authserver et MySQL orphelins (arrêtés à la main par
   `Stop-Database.ps1`). Un utilisateur clique sur la croix de la fenêtre titrée : non reproductible par lui. Le
   harnais envoie désormais WM_CLOSE à la fenêtre **titrée** `HomeCraft Edition WoTLK` (`Close-LauncherWindow`).
3. **`Start-Process -ArgumentList` ne protège pas les espaces** : la racine « HomeCraft WoTLK » coupait le chemin du
   zip de `--do restore <zip>` en deux arguments → le launcher ouvrait un dossier → « Accès refusé (os error 5) ».
   `Invoke-Launcher` met entre guillemets tout argument contenant un espace. Au passage, les erreurs de
   `backup.rs` (ouverture d'archive, extraction, import, sauvegarde de sécurité) indiquent maintenant l'étape et le
   chemin en cause.
4. Les scripts lancés par `powershell -File` exigent `-ExecutionPolicy Bypass` sur ce poste (stratégie
   `Restricted`) ; le launcher lancé depuis un script dont la sortie est redirigée fonctionne normalement.

## 4. Fiabilité (`Test-Phase5.ps1`, 15:40 → 15:52, **28 contrôles OK**)

Compte témoin `TEMOIN` inséré directement dans `acore_auth.account` (salt/verifier nuls, jamais connectable) ;
base de départ : 2 comptes hors bots, 540 personnages (dont 540 de bots : le manifeste des sauvegardes compte
`0 personnages` de joueurs, les bots `rndbot*` étant exclus de ce compteur mais bien présents dans les dumps).

| Fonction | Contrôles | Observations |
|---|---|---|
| Sauvegarde (`--do backup`) | archive `sauvegarde-AAAAMMJJ-HHMMSS.zip` avec `acore_characters.sql`, `acore_auth.sql`, `settings.json`, `backup.json` (SHA-256, 2 comptes) ; base déjà en service → « MySQL déjà en service : utilisée telle quelle », laissée active ; base arrêtée → démarrée temporairement puis arrêtée | 850 Ko ; ~10 s avec démarrage/arrêt de MySQL |
| Vérification (`--do verify`) | « fichiers gérés : 215 vérifiés, 0 manquants, 0 altérés » (1 341 Mo, SHA-256 intégral) ; MySQL non démarrée | 4 s |
| Réparation (`--do repair`) | `worldserver.conf` effacé → régénéré ; aucune base à réimporter (les quatre présentes avec leur table `updates`) ; `realmlist` et `post_import_cleanup` rejoués ; personnages et témoin intacts ; MySQL temporaire arrêtée | 8 s |
| Restauration (`--do restore <zip>`) | témoin supprimé puis revenu ; sauvegarde de sécurité « automatique avant restauration » créée avant l'import ; personnages intacts ; MySQL temporaire arrêtée | 12 s |
| Diagnostic (`--do diagnostic`) | `UserData/diagnostic-*.zip` : `launcher.log`, `etat.txt`, `worldserver.conf` et `playerbots.conf` avec toutes les lignes `*DatabaseInfo` remplacées par `"<masqué>"` (dont `PlayerbotsDatabaseInfo` hors modèle) | 129 Ko |
| `--tab backups|bots|settings --quit-after 5` | fermeture normale, code 0, aucune ERREUR, aucun service démarré | |

Effets de bord conservés : 6 archives dans `UserData/Sauvegardes` (tests + sécurité avant restauration) et deux
`UserData/diagnostic-*.zip` ; le compte témoin a été supprimé ; `UserData/settings.json` contient `auto_mmaps:false`
(posé par les tests, à remettre à `true` dans PARAMÈTRES si l'on veut la génération automatique des mmaps : elles
sont déjà présentes ici, donc sans effet).

### 4 bis. Rejeu après l'onglet Modules (exe du 16/10 15:57, tests rejoués à 22:44 → 23:00)

Le générateur ayant changé (les `.conf` de modules repartent du fichier existant, `05-Launcher-Rust.md` §6), les
deux tests ont été rejoués sur l'exe actuel et étendus : `Test-Launcher` **18/18** (contrôle ajouté :
`AiPlayerbot.CommandServerPort = 0` dans `playerbots.conf` après le démarrage ; monde prêt à t+89 s, arrêt propre en
2 s), `Test-Phase5` **36/36** avec une section Modules : `--do modoff mod-junk-to-gold` → `JunkToGold.Enable = 0`
journalisé ; `--do repair` (régénération des conf) **conserve** cet état et réapplique `CommandServerPort = 0` ;
`--do modon` → `1` ; `--do modoff mod-ale` → `ALE.Enabled = false` ; `--do modreset mod-ale` → fichier identique
octet pour octet au `.conf.dist` ; `--tab modules` sans erreur. Les modules sont remis dans leur état initial à la fin
(`mod-junk-to-gold` actif, `mod_ale.conf` = modèle).

Non couvert : `--do reimportworld` (réimport des 304 Mo de `acore_world`, ~75 s) et `--do botdel`, `--do repair`
avec base manquante, `--install-to`, génération des mmaps par le launcher. Le mode `--do` attend toujours
`--quit-after` (le launcher ne se ferme pas de lui-même à la fin de l'action), comme dans l'édition Vanilla.

## 4 ter. Client open source wowdecomp x64 et bouton JOUER (`Test-Play.ps1`, 2026-10-16 soir)

Proposition de l'utilisateur : utiliser le client **wowdecomp** (`C:\Users\Marty\Desktop\wotlk moderna\wowdecomp\bin\Win`),
décompilation C++ du client 3.3.5a 12340 (`github.com/SatyPardus/wowdecomp`, commit `061b6e7` du 2026-10-15),
compilée en **x64** le 16/10 avec `-DWOWDECOMP_DISABLE_SIGNATURE_CHECKS=ON` (cible `WowClient` = client + journal de
plantage). Son `Data/` est le client frFR 3.3.5a complet plus les patchs du royaume Sylvania (HD, etc.), en liens
physiques partagés avec les autres copies du client. Il s'était déjà connecté au royaume Sylvania en ligne le 16/10.

- **Miroir `Client/`** : `Tools/New-ClientMirror.ps1 -Source <dossier>` crée `Client/` avec les 41 MPQ en **liens
  physiques** (aucun espace disque), l'exécutable, `XvidDecoder.dll`, `realmlist.wtf`, `WTF/` et `Interface/` copiés.
  Raison : le launcher réécrit `realmlist.wtf` et `Config.wtf` (127.0.0.1) ; le client source, qui pointe sur
  `sylvania-servergame.com`, reste intact. Renommer ou supprimer un MPQ dans l'onglet Patchs ne touche que le lien.
- **`client.rs`** : exécutables 3.3.5a reconnus par ordre de préférence `Wow64.exe`, `WowClient64.exe`, `Wow32.exe`,
  `WowClient32.exe`, `Wow.exe` (`WOTLK_EXES`). L'exe wowdecomp n'a **aucune ressource de version** : le build est lu
  dans la chaîne embarquée « 3.3.5a (12340) » (`embedded_version`). Affichage « 3.3.5.12340 (wowdecomp x64) ».
  Lancement et FERMER LE JEU utilisent l'exe retenu. Texte d'aide de PARAMÈTRES mis à jour.
- **`--play`** (ligne de commande) : équivalent du bouton JOUER, pour les tests.
- **`Test-Play.ps1`** : `--play` → services → jeu lancé depuis `Client\WowClient64.exe` → capture de la fenêtre
  (`Logs/tests/jeu-*.png`) → le test ferme le jeu → le launcher détecte la fin (« jeu terminé ») et fait sa
  **sauvegarde automatique** → fermeture du launcher → arrêt propre. Résultat : **14/14 OK** (23:46, jeu lancé 30 s après le démarrage du launcher, arrêt complet en 8 s après la fin du jeu). Écran de connexion
  en français, « Version 3.3.5 (12340) », royaume affiché « HomeCraft — 127.0.0.1 ». L'identification et l'entrée en jeu
  restent à faire à la main (compte `homecraft`).

À trancher avant le packaging : wowdecomp est du code décompilé du client Blizzard ; le livrer dans le package
(plutôt que de laisser l'utilisateur le compiler ou le fournir comme le client) pose la même question de
redistribution que le client lui-même. En l'état, il est traité comme le client : fourni par l'utilisateur, jamais
archivé.

## 5. Reste à faire

- **Test en jeu par l'utilisateur** : bouton JOUER avec le client wowdecomp de `Client/` (connexion, création de
  personnage, bots), puis éventuellement client 3.4.3 + HermesProxy (`Tools/HermesProxy`, téléchargement à autoriser
  et à figer dans `Build/manifest.json`).
- Compiler aussi la cible `Wow` de wowdecomp (`Wow64.exe`, sans console de journal de plantage) si la console gêne :
  le launcher la préfère automatiquement.
- `Build/manifest.json` (composants figés) et `Build/Get-Components.ps1` n'existent pas encore pour l'édition WoTLK :
  `New-FilesManifest.ps1` ne les référence donc pas.
- Packaging portable (`New-Package.ps1`), signature, `Test-Package.ps1`, `Test-Offline.ps1` dédié (le contrôle
  d'écoute loopback est pour l'instant intégré à `Test-Launcher.ps1`), manuel utilisateur.
- Décider si `--do <action>` doit fermer le launcher dès la fin de l'action (raccourcirait les tests de ~8 min).
