# WoW Vanilla Solo Edition — Phase 1 : audit technique

Date : 2026-10-14
Auteur : session Claude Code (poste Windows « Marty »)
Statut : **audit terminé, architecture recommandée, Phase 2 non commencée**

Toutes les affirmations ci-dessous proviennent de vérifications faites le 2026-10-14 sur GitHub
(pages de dépôt, API GitHub, fichiers bruts, workflows CI). Ce qui n'a pas pu être vérifié est
indiqué explicitement dans la section 9.

---

## 1. Projets examinés

### 1.1 VMaNGOS — `vmangos/core` (serveur)

| Critère | Constat vérifié |
|---|---|
| Licence | GPL-2.0 (fichier LICENSE du dépôt) |
| Activité | Branche `development`, 7 364 commits, build de développement publié le **2026-10-07** (commit `53d24d5`) |
| Clients supportés | 1.5.1 → **1.12.1.5875** (liste explicite dans le README) |
| Binaires précompilés | **Oui** : release mobile `latest` avec `dev-windows-amd64-<sha>.zip` (5,8 Mo), Linux amd64 (glibc ≥ 2.39), macOS arm64. Build CI Windows : `windows-2022`, VS 2022, x64, `BUILD_EXTRACTORS=1`. Le zip contient le dossier `bin/Release` entier + `libmySQL.dll` + `libeay32.dll` (script `package-windows-release.sh`) |
| Base de données | **MySQL obligatoire**. Snapshot `db_latest` (2026-09-28) : `db-<sha>.zip` (30,5 Mo) produit avec l'image Docker `mysql:5.6`. Un second zip `db-sqlite-<sha>.zip` existe mais c'est une **conversion des dumps en SQLite**, pas un backend SQLite du serveur : le cœur ne lit pas SQLite (`CMakeLists.txt` : MySQL/OpenSSL/ZLIB requis, aucune mention de SQLite) |
| Dépendances Windows | Fournies dans `dep/windows` (libmySQL, libeay32, zlib). ACE/TBB ne sont plus mentionnés dans le `CMakeLists.txt` actuel (le wiki « Compiling on Windows » est obsolète sur ce point) |
| Bots intégrés | **Oui, dans le cœur** (`src/game/PlayerBots/`) : `PartyBotAI`, `BattleBotAI`, `PlayerBotMgr`, `CombatBotBaseAI`. Donc sous GPL-2.0 comme le reste |
| Warden | Désactivé par défaut (`Warden.WinEnabled = 0`) |
| Anticheat | `Anticheat.Enable = 0` par défaut |
| Localisation | `DBC.Locale = 255` (auto-détection depuis les DBC extraits du client). La base monde contient des tables `locales_*` ; la pull request AzerothCore #28044 confirme que VMaNGOS contient des textes de quêtes français d'origine **communautaire** (32 % de tutoiement là où le client officiel en a 0,5 %) et des textes fidèles au client là où SPP a la même quête. Le dossier `sql/translations/` ne contient que `ptBR` (langues sans client officiel) |

### 1.2 CMaNGOS Classic — `cmangos/mangos-classic` (serveur)

| Critère | Constat vérifié |
|---|---|
| Licence | GPL-2.0 (+ exception OpenSSL) |
| Activité | 11 396 commits, dernier commit **2026-08-31** |
| Binaires précompilés | **Aucun** (Releases et Packages vides). Build obligatoire : VS 2022, CMake ≥ 3.22, Boost ≥ 1.75, MySQL |
| Base de données | `cmangos/classic-db` (GPL-3.0, 7 009 commits). MySQL ou MariaDB (« any MySQL server ») |
| Bots | Module externe `cmangos/playerbots` (voir 1.3) activé par `BUILD_PLAYERBOTS` |
| Localisation | `classic-db/locales/` contient **German** et **Chinese** seulement (+ `BroadcastTextLocales.sql`). **Pas de français officiel.** L'issue cmangos/issues#2030 : « We do not support them officially » (killerwife, 2020). Un pack communautaire `esno/cmangos-vanilla-db-localized` (fork de `Chuck5ta/MangosZero_Localised`, pour MangosZero, dernier push **2020-10-14**, **sans licence**) contient `French_Quest.sql`, `French_Creature.sql`, `French_Items.sql`, etc. (source UDBFR) |

### 1.3 PlayerBots ike3 — `cmangos/playerbots`

| Critère | Constat vérifié |
|---|---|
| Licence | **Aucune** (`license: null` via l'API GitHub ; pas de fichier LICENSE). Idem pour les parents `celguar/mangosbot-bots` et `ike3/mangosbot-bots`. Le code est dérivé de MaNGOS (GPL-2), donc vraisemblablement GPL par dérivation, mais **aucune déclaration explicite** |
| Activité | 6 256 commits, dernier commit **2026-10-13** (« Test: Improved quest tests ») — très actif |
| Intégration | `BUILD_PLAYERBOTS` dans CMake ; `aiplayerbotvanilla.conf.dist` ; SQL dans `sql/vanilla/{characters,world}` ; `InstallFullDB.sh` avec `PLAYERBOTS_DB="YES"` |
| Fonctionnalités (README) | Monde peuplé de bots aléatoires, BG/arènes, **alts du compte comme bots**, PvE « any kind » avec guidage sur mécaniques complexes, stratégies configurables |
| Commandes (doc Google) | `.bot add <nom>` : le bot doit être **sur le compte du joueur ou dans sa guilde** (ou compte GM). Ordres par chuchotement : `follow`, `stay`, `attack`, `flee`, `help`. Stratégies `+tank` (guerrier/paladin/druide), `+heal` (paladin/prêtre/chaman/druide), états `nc`/`co`/`dead`/`react` |
| Repack existant | `celguar/spp-classics-cmangos` : installeur Windows + `Launcher.bat`, Vanilla/TBC/WotLK, 1 000 bots aléatoires par défaut. Dernier build Classic daté **2025-11-28**. **Sans licence** ; batchs, pas de launcher graphique |

### 1.4 Benilla — `samwhosung/benilla` (client réécrit)

| Critère | Constat vérifié |
|---|---|
| Licence | MIT ou Apache-2.0 (code propre) ; Lua 5.1 et kira vendorisés |
| Activité | 660 commits, dernier commit **2026-10-14** (le jour même), 719 étoiles, **580 issues ouvertes**, auteur principal unique |
| État | README : « Complete and fully playable », reste « the long tail of small features ». Les addons tiers (Questie, pfUI) sont annoncés comme fonctionnels. Issue #1093 ouverte : vérification d'intégrité MPQ non implémentée |
| Prérequis | **Client anglais 1.12.1 (build 5875)** pour les données. Serveur 1.12.1 avec Warden désactivé (vmangos recommandé, cmangos « devrait fonctionner »). Rust stable + compilateur C + MSVC Build Tools sur Windows |
| Binaires précompilés | **Aucun** (« There are no prebuilt downloads ») |
| Localisation | Issue #429 : Benilla lit **uniquement la colonne 0 (enUS) des DBC**, présenté comme une « différence choisie ». Un client frFR ne donnerait donc pas une interface française. Issues #621/#746 (closes) : problèmes AZERTY corrigés |

### 1.5 Autres pistes recherchées

- **WoWee** (client open source multi-version) : mentionné dans la presse, non audité en détail ; même problème (pas de binaire, projet jeune).
- Aucun autre client communautaire 1.12.1 mature n'a été identifié.

---

## 2. Comparaison des architectures

### Solution A — client original 1.12.1 + serveur compatible + bots

Deux variantes possibles :

| Critère | **A1 : VMaNGOS (PartyBots intégrés)** | **A2 : CMaNGOS Classic + ike3 PlayerBots** |
|---|---|---|
| Fidélité Vanilla | Très élevée (progressif 1.2 → 1.12, données taguées par patch) | Élevée |
| Stabilité | Builds dev quotidiens, pas de release stable étiquetée | Idem, pas de release |
| Windows 11 | Binaire x64 VS 2022 fourni | À compiler soi-même |
| Binaires redistribuables prêts | **Oui** | **Non** |
| Licence serveur | GPL-2.0 propre, bots inclus | GPL-2.0 cœur ; **bots sans licence** |
| Bots : compagnons de groupe | `.partybot add tank/healer/dps/<classe>` crée un bot instantané au niveau du joueur, rôle réglable (`setrole tank/meleedps/rangedps/healer`), ordres `attackstart/stop`, `pull`, `aoe`, `cometome`, `pause`, marques CC/focus, `usegobject`. 9 classes. Bots **temporaires** (pas de persistance/progression) | `.bot add` sur alts du compte ; bots **persistants** qui progressent, questent, apprennent sorts/métiers ; addon MangosBot pour piloter sans commandes |
| Bots : monde peuplé | `RandomBot.Enable` existe mais « no AI unless you code one » → monde vide | 1 000 bots aléatoires avec IA |
| Restrictions commandes bots | Toutes les commandes `.partybot` sont `SEC_ADMINISTRATOR` → le compte local doit être GM (acceptable : le launcher crée le compte). `PartyBot.MaxBots`, `PartyBot.SkipChecks` | `.bot add` possible sur compte joueur |
| Français côté serveur | Tables `locales_*` partiellement remplies (qualité mixte, communautaire) | Pack 2020 non officiel, sans licence, pour MangosZero (schéma proche mais à adapter) |
| Français côté client | Client frFR officiel 1.12.1 (interface, DBC, sorts, objets, infobulles) | Idem |
| Base de données | MySQL (snapshot généré sous MySQL 5.6 ; wiki : 5.5/5.6 recommandés, 5.7 « problèmes signalés » — **compatibilité MySQL 8 / MariaDB à tester**) | MySQL ou MariaDB |
| Facilité launcher | Deux exécutables (`realmd`, `mangosd`) + MySQL ; ports 3724/8085 | Identique |
| Taille package serveur | ~6 Mo binaires + ~30 Mo dumps + MySQL portable (~100-300 Mo) + données extraites du client (maps/vmaps/mmaps : 1 à 3 Go) | Identique + bots |
| Maintenance | Suivre la release mobile `latest` + `db_latest` | Recompilation à chaque mise à jour |

### Solution B — client communautaire Benilla + serveur Vanilla

| Critère | Constat |
|---|---|
| Fidélité | Bonne mais incomplète (580 issues ouvertes, long tail) |
| Windows 11 | Oui (Rust/Bevy) mais à compiler ; pas de binaire |
| **Français** | **Bloquant** : DBC lues en enUS uniquement (#429), client anglais exigé |
| Bots | Dépend du serveur (A1 ou A2) |
| Launcher | Plus simple (pas de `realmlist.wtf`, pas de Warden) |
| Taille | Client + données MPQ d'origine (toujours nécessaires) |
| Licence | MIT/Apache : redistribution du binaire possible |
| Risque | Projet à un seul auteur, API instable, pas de version stable |

**Conclusion :** la solution B est écartée pour cette édition. Elle n'apporte rien à l'objectif « français intégral » (elle le rend impossible aujourd'hui) et ajoute un composant sans binaire ni version stable. À réévaluer dans 12 mois si Benilla publie des releases et le support des locales.

---

## 3. Architecture recommandée

**Solution A1 : client officiel frFR 1.12.1 (5875) fourni par l'utilisateur + VMaNGOS (binaires Windows officiels) + PartyBots intégrés + MySQL portable + launcher Windows.**

Raisons :

1. **Seul serveur avec binaires Windows x64 officiels** régulièrement publiés → chaîne de build reproductible sans compiler le cœur (on pourra tout de même recompiler plus tard pour figer une révision).
2. **Licences propres** : GPL-2.0 pour cœur + bots. Redistribuer les binaires VMaNGOS est autorisé à condition de fournir les sources (lien vers le commit exact) et la licence. Avec A2, le module ike3 est sans licence : redistribution juridiquement fragile.
3. **PartyBots couvrent l'objectif principal** : constituer un groupe tank/heal/dps à son niveau, faire donjons et combats. Les bots ne progressent pas eux-mêmes, mais ils sont recréés au niveau du joueur à chaque session, ce qui simplifie énormément le launcher (pas d'alts à gérer).
4. **Français** : client frFR officiel pour toute l'interface + tables `locales_*` VMaNGOS pour quêtes/PNJ/objets. Qualité à mesurer (Phase 3), complément possible par le pack UDBFR si la licence est clarifiée.
5. **Warden et anticheat désactivés par défaut** : pas de faux positifs en solo.

**Limites acceptées et plan d'amélioration :**

- Monde vide (pas de bots aléatoires avec IA). Amélioration progressive possible : évaluer le portage ike3 ou un peuplement cosmétique plus tard.
- Pas de persistance des compagnons : amélioration future via `.partybot load <personnage>` (charge un personnage hors ligne du compte comme bot) — à tester en Phase 2, cela permettrait des compagnons persistants créés par le launcher.
- Difficulté non adaptée au nombre de personnages : VMaNGOS n'a pas de scaling solo. À traiter via rates (`Rate.*`) configurables dans le launcher (option « Solo facile »).

**Fallback documenté :** si les tests Phase 2 montrent que les PartyBots sont insuffisants (donjons impraticables), basculer vers A2 (CMaNGOS + ike3) en compilant nous-mêmes, en acceptant le risque de licence du module bots (ou en obtenant une clarification des auteurs).

---

## 4. Risques

### Techniques

| Risque | Gravité | Mitigation |
|---|---|---|
| VMaNGOS exige MySQL ; snapshot généré en 5.6 (EOL). MySQL 8.x / MariaDB 10-11 non garantis | Élevée | Tester en Phase 2 : MariaDB 11 portable d'abord (licence GPL, zip sans installeur), puis MySQL 8.4 LTS en repli. Figer la version validée dans le manifeste |
| Builds VMaNGOS « dev », pas de release stable | Moyenne | Figer un commit précis (sha) dans le manifeste ; ne mettre à jour qu'après tests |
| Extraction maps/vmaps/mmaps longue (mmaps : heures) et non redistribuable (dérivé des MPQ Blizzard) | Élevée | Le launcher/installateur lance les extracteurs sur le client de l'utilisateur à la première installation, avec barre de progression ; mmaps optionnelles mais recommandées pour l'IA |
| Client 1.12.1 sur Windows 11 (plein écran, DirectX 9) | Moyenne | Config.wtf générée (`gxWindow`, `gxColorBits`), tests sur machine propre |
| Commandes `.partybot` réservées GM | Faible | Compte créé avec gmlevel suffisant ; `PartyBot.SkipChecks=1` ; une interface addon/launcher masque les commandes |
| Qualité/complétude des traductions FR en base | Moyenne | Inventaire chiffré en Phase 3 (comptage par table), rapport des trous |
| Ports 3724/8085/3306 en écoute locale : pare-feu Windows peut afficher une alerte | Faible | Lier sur 127.0.0.1 uniquement (`BindIP`, `bind-address`) ; aucune règle de pare-feu nécessaire pour le loopback |

### Juridiques

| Point | Statut |
|---|---|
| Client WoW 1.12.1 frFR, MPQ, DBC, maps extraites | **Propriété Blizzard, non redistribuable.** L'utilisateur fournit son client ; l'installateur ne le télécharge jamais |
| VMaNGOS binaires + bots | GPL-2.0 : redistribution OK avec sources et licence jointes |
| Base VMaNGOS | Contenu sous GPL-2.0 (dépôt core) ; vérifier le fichier de licence de la base lors de la Phase 2 |
| MySQL Community / MariaDB | GPL-2.0 : redistribution OK avec licence jointe |
| Pack traductions UDBFR (`esno/cmangos-vanilla-db-localized`) | Sans licence → **ne pas intégrer** sans accord |
| ike3 PlayerBots, repack SPP | Sans licence → **ne pas intégrer** |
| Launcher | À placer sous GPL-3.0 (cohérent avec le Sylvania Launcher existant) |

---

## 5. Faisabilité de la traduction française intégrale

| Couche | Source | Faisabilité | Remarque |
|---|---|---|---|
| Interface client (menus, création perso, inventaire, talents, sorts, objets, infobulles, messages système, journal de combat, groupes) | Client frFR officiel (`Data/frFR/*.MPQ`, DBC localisées, FrameXML/GlobalStrings) | **Complète** | Le client frFR 1.12.1 existe (patch `wow-1.12.0-to-1.12.1-frfr.exe`, MD5 46c328c6dd4accd68fd4ee5a7b112dd2 relevé sur ModDB et Blue Tracker) |
| Noms de créatures, objets, sorts | DBC + `locales_creature`/`locales_item` | Partielle à complète | Les sorts et une partie des objets viennent des DBC (client) ; les noms de PNJ viennent de la base |
| Quêtes, dialogues PNJ, objectifs, textes de pages | `locales_quest`, `locales_npc_text`, `locales_page_text`, `broadcast_text` (loc2) | **Partielle, qualité à mesurer** | VMaNGOS a des textes FR communautaires (tutoiement fréquent) |
| Messages serveur (`.commands`, erreurs) | `mangos_string` | Faible | Le launcher cache la plupart des commandes |
| Launcher | Code | Complète | Fichiers de ressources FR, gettext-like |

Méthode retenue : auto-détection DBC (`DBC.Locale=255`) + extraction des DBC depuis le client frFR → le serveur sert les données en français. Inventaire chiffré en Phase 3 (requêtes SQL sur les tables `locales_*`, comptage des lignes vides en loc2).

---

## 6. Prérequis

### Côté développeur (poste de build)

État de cette machine le 2026-10-14 : Windows 11 Famille 64 bits, 32 Go RAM, C: 716 Go libres.

| Outil | Présent | Requis pour |
|---|---|---|
| .NET runtime 8.0.19 | Oui (runtime seulement, **pas de SDK**) | — |
| Git | **Non** | Cloner/figer les sources |
| CMake, Visual Studio 2022 | **Non** | Recompiler VMaNGOS (optionnel en Phase 2 grâce aux binaires officiels) |
| Qt 6 + MinGW ou MSVC | **Non** ici (le Sylvania Launcher C++/Qt6 existant a été compilé avec aqtinstall) | Launcher |
| 7-Zip | **Non** | Packaging |
| Inno Setup | **Non** | Installateur |
| MariaDB/MySQL portable | **Non** | Base locale |
| Client WoW 1.12.1 frFR | **Non trouvé** (seul le client retail est installé sur G:) | Données + tests |

### Côté joueur final

- Windows 10/11 64 bits.
- Son propre client WoW 1.12.1 (5875) frFR, ~5 Go.
- ~4 à 6 Go d'espace supplémentaire (base, données extraites, sauvegardes).
- Aucun droit administrateur permanent, aucun port ouvert vers Internet.

---

## 7. Choix technologique du launcher

> **Décision utilisateur du 2026-10-14 : le launcher sera écrit entièrement en Rust** (voir `04-Launcher-Exigences.md`). La recommandation ci-dessous est conservée pour mémoire.

**Recommandation initiale : C++17 / Qt 6 (Widgets), licence GPL-3.0**, en réutilisant le Sylvania Launcher existant (`Desktop/Sylvania Launcher - C++`, v3.0, Qt 6.8) qui contient déjà : écriture atomique de `Config.wtf`/`realmlist.wtf`, suivi non bloquant du processus de jeu, téléchargement avec SHA-256 bloquant, extraction d'archives durcie, i18n FR/EN, notifications natives. Alternatives écartées : C#/.NET (pas de SDK installé, pas de code existant), Rust (toolchain absente, pas de code réutilisable). Le choix pourra être reconsidéré si la machine de build change.

---

## 8. Plan de développement priorisé

| # | Tâche | Critère de validation |
|---|---|---|
| 2.1 | Récupérer `dev-windows-amd64-53d24d5.zip` + `db-4641790.zip`, vérifier les empreintes, inventorier le contenu | Liste des exécutables/DLL/conf, SHA-256 consignés dans `Build/manifest.json` |
| 2.2 | MariaDB 11 portable : init `--datadir`, `bind-address=127.0.0.1`, import `mangos.sql`/`characters.sql`/`logon.sql`/`logs.sql` + migrations | `mangosd` démarre sans « missing migration » |
| 2.3 | Extraction DBC/maps/vmaps depuis un client frFR 1.12.1 (à fournir) | Dossiers `dbc`, `maps`, `vmaps` présents ; `mangosd` charge `DBC.Locale` = frFR |
| 2.4 | Connexion client, création perso, `.partybot add tank/healer/dps`, combat, donjon bas niveau (Ragefire/Mortemines) | Compte-rendu de test avec captures |
| 2.5 | Tester `.partybot load` pour compagnons persistants | Oui/non documenté |
| 3.x | Inventaire traductions (SQL de comptage par table), tests accents/encodage | Tableau de couverture |
| 4.x | Launcher minimal : machine à états, santé réelle (ports 3306/3724/8085 + requête SQL + log « World initialized »), lancement client, arrêt propre | Démarrage/arrêt 10 fois sans fuite de processus |
| 5.x | Sauvegardes (`mariadb-dump --single-transaction` de `characters` + `realmd`), restauration, réparation par manifeste | Restauration vérifiée |
| 6.x | Packaging Inno Setup, test sur VM propre | Installation sans outil de dev |
| 7 | Validation finale (liste §16 du cahier des charges) | Rapport |

---

## 9. Vérifications impossibles dans cet environnement

- Contenu exact du zip Windows VMaNGOS et du snapshot de base : nécessite le téléchargement (autorisation demandée).
- Compatibilité MySQL 8.x / MariaDB 11 avec les dumps et migrations VMaNGOS : nécessite un test réel.
- Couverture chiffrée des traductions françaises dans la base VMaNGOS : nécessite l'import de la base.
- Comportement réel des PartyBots en donjon : nécessite un client 1.12.1 frFR, absent de cette machine.
- Licence formelle de la base VMaNGOS (fichier dans le zip) : à lire après téléchargement.

## 10. Sources consultées

- https://github.com/vmangos/core (README, releases `latest` et `db_latest` via API, `CMakeLists.txt`, `.github/workflows/*`, `.github/scripts/package-windows-release.sh`, `src/mangosd/mangosd.conf.dist.in`, `src/game/Chat/Chat.cpp`, `src/game/PlayerBots/*`, `sql/translations/`)
- https://github.com/vmangos/wiki (Compiling on Windows, Getting it working, World Database)
- https://github.com/cmangos/mangos-classic, https://github.com/cmangos/classic-db (`locales/`), https://github.com/cmangos/issues/wiki/Installation-Instructions, https://github.com/cmangos/issues/issues/2030
- https://github.com/cmangos/playerbots (README, API), https://github.com/celguar/mangosbot-bots (API), https://github.com/ike3/mangosbot-bots (API), document de commandes Google Docs lié par le README
- https://github.com/celguar/spp-classics-cmangos (README, releases via API)
- https://github.com/samwhosung/benilla (README brut, issues #429, #543, #621, #746, #1093, API commits)
- https://github.com/esno/cmangos-vanilla-db-localized (README, API)
- https://github.com/azerothcore/azerothcore-wotlk/pull/28044 (analyse des sources de textes frFR, dont VMaNGOS)
- https://www.moddb.com/games/world-of-warcraft/downloads/patch-1-12-to-1-12-1-french, https://lenain.info/fr/running-wow-like-it-is-2006-again/ (existence du client frFR 1.12.1)
