# Cœur SylvaniaCore-WoTLK + Playerbots : analyse du dépôt et construction

Dernière mise à jour : 2026-10-16 (nuit). État : **patch appliqué, configuration CMake réussie, compilation en cours**.

## 1. Le dépôt `BlaMacfly/SylvaniaCore-WoTLK` (vérifié sur le clone du 2026-10-16)

- **Privé** : l'API GitHub répond 404 sans authentification et Git demande un identifiant ; le clone a été fait
  depuis le terminal de l'utilisateur, authentifié par Git Credential Manager (jeton dans le magasin Windows,
  jamais dans le projet). Clone : `Build/src/sylvaniacore-wotlk` (1 Go de `.git`, 18 105 commits, **historique
  complet, pas shallow** : `cherry-pick`, `merge`, `apply -3` redeviennent possibles ici, contrairement au VPS).
- Branche unique `master`, HEAD `d39f71045` du 2026-10-15 « [Sylvania] Nouveau logo, README : une release de BDD
  par royaume ». Tags `royaume1-db-2026-10-15` et `royaume2-db-2026-10-15` : **les dumps des bases sont des
  releases GitHub** du dépôt (privé → à télécharger par l'utilisateur, ou dépôt rendu public).
- 19 commits maison (`[Sylvania] …`, « Patchs maison Sylvania (etat 2026-07-16) » `c11f4ddb8`) : ICC/Ulduar/Naxx
  jouables en solo, Deuillelombre, Épines, ACSoap, gardes anti stun-lock, etc. Empreinte : **85 fichiers sources**
  (+3 451/−815), 25 fichiers `data/` (+500), 580 fichiers `modules/` (+138 113 : modules embarqués en dur, sans
  `.gitmodules`).
- **23 modules embarqués** (`fa2b77425` « Modules embarques (etat de production) ») : mod-1v1-arena, mod-ah-bot,
  mod-ale (Eluna), mod-anticheat, mod-arac, mod-autobalance, mod-cfbg, mod-challenge-modes, mod-character-services,
  mod-combat-lock-rescue, mod-dk-acherus-rescue, mod-junk-to-gold, mod-morphsummon, mod-mythic-plus,
  mod-npc-free-professions, mod-npc-morph, mod-pet-loot, mod-phase-aura-rescue, mod-skip-dk-starting-area,
  mod-solo-lfg, mod-solocraft, mod-sylvania-events, mod-transmog. **Pas de playerbots.**
- SQL : base `creature` en `id1/id2/id3`, pas de `creature_multispawn` (ni en base ni dans `ObjectMgr.cpp`),
  `data/sql/updates/db_world` **plafonne à `2026_03_31_00.sql`** (1 144 fichiers), 28 fichiers dans
  `pending_db_world` (correctifs maison appliqués par l'updater au démarrage), 2 dans `pending_db_auth` et
  `pending_db_characters`.

### La vraie base du fork n'est pas celle que l'historique affiche

Le commit parent du premier commit maison, `7b0d90ca` « fix(Scripts/Ulduar): Algalon… (#26612) », porte le
message d'un commit upstream du **2026-07-15**, mais son arbre n'est pas celui d'upstream à cette date : upstream
avait déjà migré vers `creature_multispawn` (2026-06-16) et comptait 1 374 fichiers d'updates. Les commits
upstream postérieurs à mars ont été **rejoués par `git am`** (cf. `claude-memory/fork-azerothcore-sylvania.md`),
d'où des SHA différents et un contenu partiel. Mesure (comparaison de l'arbre `7b0d90ca` avec 60 commits upstream
de fin mars / début avril) : l'arbre le plus proche est **`e144326f0` du 2026-03-31** (26 fichiers `src/` d'écart,
1 fichier `data/`), cohérent avec la date du clone sur le VPS (2026-04-02) et le plafond des updates.

Conséquences : upstream master compte aujourd'hui **1 242 commits de plus** que le 15 juillet (680 fichiers
`src/`, 1 303 fichiers `data/sql/`), bien davantage depuis le 31 mars ; 74 fichiers sources sont modifiés des deux
côtés. Fusionner upstream dans le fork n'est pas envisageable pour cette édition.

## 2. Playerbots : la voie retenue, « figée à la date du fork »

Le patch cœur Playerbot d'aujourd'hui (`Build/playerbot-core.patch`, 33 fichiers) **ne s'applique pas** au fork :
il repose sur `ModuleDatabasePool` (upstream #27544 du 2026-09-14) et sur l'état récent de `PathGenerator` et du
`mmaps_generator`. Plutôt que de porter six mois d'upstream, on prend la branche Playerbot et le module **tels
qu'ils étaient quand le fork a été cloné** :

| Élément | Révision | Date | Fichier |
|---|---|---|---|
| Patch cœur = diff entre le commit master fusionné (`89b810b2f^2` = upstream `71928da8d`) et la branche Playerbot à la synchronisation `89b810b2f` | `mod-playerbots/azerothcore-wotlk` `89b810b2f` | 2026-04-03 | `Build/playerbot-core-89b810b2f.patch` (77 fichiers, +1 958/−734) |
| Module | `mod-playerbots/mod-playerbots` `f76c2863` (dernier commit avant la synchronisation cœur du même jour, inclut la fusion `#2253`) | 2026-04-03 | `Build/downloads/mod-playerbots-f76c2863.zip` → `modules/mod-playerbots/` |
| Candidat écarté | synchronisation `f0d3e7b7d` du 2026-04-10 | | `Build/playerbot-core-f0d3e7b7d.patch` (3 fichiers en échec) |

Résultat : `git apply --check` sur le fork → **1 seul fichier en échec, `Group.h`** (Sylvania y ajoute
`SendPendingRollsToPlayer` juste à côté). Appliqué avec `--exclude` puis les deux méthodes
(`GetTargetIcon`, `GetRolls`) ajoutées à la main. Le patch d'avril contient aussi la base `acore_playerbots`
(`PlayerbotsDatabase.cpp/.h`, `DBUpdater`) : pas besoin de `ModuleDatabasePool`. Il ne touche ni `MapDefines.h`
ni le `mmaps_generator` : les mmaps standard conviennent (pas de `NAV_GROUND_STEEP` à cette date).

Piège rencontré : un patch écrit avec `Out-File` (BOM + CRLF) échoue sur tous les fichiers ; toujours produire les
patchs avec `git diff --output=`.

Branche de travail : `homecraft-playerbots`, commit `b2dcd324f` « [HomeCraft] Playerbots : patch coeur … +
module … » (77 fichiers côté cœur + 1 334 fichiers de module). La branche `master` du clone reste intacte.

Limites assumées : le module d'avril 2026 n'a pas les six mois d'améliorations suivants (stratégies, performances,
corrections) ; une montée de version demandera de porter aussi le cœur. Le module publie ses SQL dans
`modules/mod-playerbots/data/sql/{playerbots,characters,world}` : à importer dans la phase base de données.

## 3. Construction

Configuration (`Logs/cmake-configure-sylvania.log`, 2026-10-16 00:47, 19,7 s) :

```bash
cmake -S Build/src/sylvaniacore-wotlk -B Build/src/sylvaniacore-wotlk/build -G "Visual Studio 17 2022" -A x64 -DBOOST_ROOT=C:/local/boost_1_87_0 -DOPENSSL_ROOT_DIR="C:/Program Files/OpenSSL-Win64" -DMYSQL_INCLUDE_DIR=C:/tools/mysql/current/include -DMYSQL_LIBRARY=C:/tools/mysql/current/lib/libmysql.lib -DTOOLS_BUILD=all -DSCRIPTS=static -DMODULES=static -DCMAKE_INSTALL_PREFIX="Build/install"
```

Résultat : révision `b2dcd324f25d (homecraft-playerbots branch)`, applications `all`, outils `all` (extracteurs
map/vmap/mmap), scripts et modules statiques, **24 modules** reconnus avec leurs `.conf` (dont `playerbots.conf`),
Boost 1.87.0, OpenSSL 3.5.9, MySQL 8.4.9.

Compilation : `Build/Build-Core.ps1 [-Configure] [-Target x] [-Install]` (journal `Logs/build-sylvania.log`), à
lancer depuis un terminal interactif (pas de limite de durée). Deux obstacles rencontrés et réglés le 2026-10-16 :

1. **`__builtin_unreachable` (GCC) dans `mod-character-services`** → erreur C3861 sous MSVC. Corrigé par une macro
   `CS_UNREACHABLE()` (`__assume(0)` sous MSVC), commit `[HomeCraft] mod-character-services …` de la branche.
2. **`error MSB6003 … rc.exe … Nom de fichier ou extension trop long`** à l'édition de `worldserver` : avec 24
   modules, la ligne de commande (chemins d'inclusion sous `C:\Users\Marty\Desktop\HomeCraft WoTLK\Build\src\…`)
   dépasse la limite Windows. Remède : **jonction NTFS `C:\hc` → le clone** (créée par le script, sans droits
   administrateur) et répertoire de build **`C:\hc\b`**. La première passe (20 min, chemin long) a été jetée.

3. **`LNK1104 : libboost_thread-vc143-mt-x64-1_87.lib introuvable`** à l'édition de `worldserver` : le
   `deps/boost/CMakeLists.txt` du fork (état mars 2026) ne demande pas le composant `thread`, que
   `mod-playerbots` utilise (auto-link MSVC sans chemin de bibliothèque). Corrigé comme upstream : `thread` ajouté
   aux composants (commit `[HomeCraft] deps/boost : composant thread`).
4. Script : `$ErrorActionPreference = "Stop"` transformait les avertissements CMake (stderr, CMP0167) en erreur
   bloquante ; passé à `Continue`, les échecs sont détectés par le code de retour.

**Résultat (2026-10-16 01:35)** : compilation complète réussie (première passe 20 min, puis relances
incrémentales), installation dans `Build/install/` (73 Mo) :

| Fichier | Rôle |
|---|---|
| `worldserver.exe` (57 Mo), `authserver.exe` | `--version` → `AzerothCore rev. a7161cd0adb7 2026-10-16 (homecraft-playerbots branch) (Win64, RelWithDebInfo, Static)` |
| `dbimport.exe` | import/mise à jour des bases en ligne de commande |
| `map_extractor.exe`, `vmap4_extractor.exe`, `vmap4_assembler.exe`, `mmaps_generator.exe` (+ `mmaps-config.yaml`) | extraction des données client 3.3.5a |
| `lua52_*` | outils Eluna (mod-ale) |
| `configs/*.conf.dist` + `configs/modules/*.conf.dist` (24 modules dont `playerbots.conf.dist`) | modèles de configuration à générer par le launcher |
| `libmysql.dll`, `libcrypto-3-x64.dll`, `libssl-3-x64.dll` | copiées à la main depuis `C:\tools\mysql\current\lib` et `C:\Program Files\OpenSSL-Win64\bin` (l'installation CMake ne les fournit pas ; à intégrer au packaging) |

Les `.pdb` restent dans `C:\hc\b\bin\RelWithDebInfo` (156 Mo pour worldserver) : utiles au débogage, exclus du
package. Branche `homecraft-playerbots` : 4 commits HomeCraft au-dessus de `d39f71045` (`b2dcd324f` patch +
`e3fde338e` module + `9ce9bca6d` MSVC + `a7161cd0a` Boost.Thread). **Non poussée** sur GitHub : à faire quand
l'utilisateur le souhaite (`git push origin homecraft-playerbots`).

Ce qui n'est pas encore vérifié : le démarrage réel contre une base (nécessite les dumps), le chargement de
`mod-playerbots` à l'exécution, l'absence de régression des 19 commits Sylvania sous MSVC (le VPS compile avec GCC).

## 4. Reste à faire pour la Phase 2

1. Télécharger les releases `royaume1-db-2026-10-15` / `royaume2-db-2026-10-15` (dépôt privé : par
   l'utilisateur) et décider du royaume repacké.
2. MariaDB portable, import des bases (`acore_auth`, `acore_characters`, `acore_world`, `acore_playerbots`) +
   `pending_*` du fork + SQL du module ; vérifier que l'updater accepte MariaDB (le VPS tourne sous MySQL ou
   MariaDB ? à relever).
3. Extraction 3.3.5a depuis `Desktop\wotlk moderna\ref-client` avec les outils compilés.
4. HermesProxy (zip Windows figé dans le manifeste) + Arctium ; premier login du client 3.4.3.
