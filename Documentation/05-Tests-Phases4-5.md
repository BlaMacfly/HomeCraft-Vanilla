# Phases 4 et 5 — journal de tests du launcher

## Phase 4 — launcher minimal (2026-10-14, soir)

`Tests/Test-Launcher.ps1` : lance l'exécutable release avec `--start --quit-after N`, puis vérifie le journal et les ports.

| Critère | Résultat |
|---|---|
| MariaDB opérationnelle (port + `SELECT 1`) | t+2 s |
| realmd en écoute 3724 | t+2 s |
| « World initialized » lu et 8085 en écoute | t+6 s |
| Fermeture : mangosd arrêté proprement (`server exit` + fermeture de stdin), MariaDB par `SHUTDOWN` | OK, < 2 s |
| Ports libérés, aucun processus orphelin, aucune ERREUR | OK |

Exécuté 3 fois (dont après l'habillage graphique). Exécutable signé (auto-signé) après chaque build.

## Phase 5 — fiabilité (2026-10-14, 23:24)

`Tests/Test-Phase5.ps1` (15 contrôles, tous OK) :

| Fonction | Contrôles |
|---|---|
| Sauvegarde | archive créée, contient `characters.sql` / `realmd.sql` / `backup.json`, manifeste cohérent avec la base (2 comptes), MariaDB temporaire arrêtée ensuite |
| Vérification | 690 fichiers gérés (527 Mo) : 0 manquant, 0 altéré (durée ≈ 2 min 30, SHA-256 intégral) |
| Réparation | base `logs` supprimée volontairement → réimportée ; `mangosd.conf` effacé → régénéré ; nombre de personnages inchangé |
| Restauration | compte témoin supprimé → revenu après restauration ; sauvegarde de sécurité automatique créée avant |
| Diagnostic | archive exportée, chaînes de connexion masquées dans les `.conf` |

Effets de bord du test conservés : 2 archives dans `UserData/Sauvegardes` (test automatique, sécurité avant restauration) et un `diagnostic-*.zip` dans `UserData/`. Le compte témoin a été supprimé.

Complément du 2026-10-15 (soir) : étape « réimport de la base monde » ajoutée (`--do reimportworld`) : un titre de quête modifié en base disparaît, la traduction HomeCraft revient, les compléments sont journalisés (« import : complément … »), personnages et comptes inchangés. Le script compte désormais 20 contrôles.

## Phase 6 — package portable (2026-10-14, 23:53)

`Tests/Test-Package.ps1` (12 contrôles, tous OK sur le package final v0.1.0 avec onglet PlayerBots et logo transparent) :

| Étape | Résultat |
|---|---|
| Copie vierge du package dans `C:\HomeCraftTest`, client indiqué par `settings.json` | OK |
| Premier démarrage : datadir MariaDB initialisé, 4 bases importées + compléments SQL (AHBot), dbc/maps/vmaps extraits, monde en écoute | OK, monde prêt à **t+110 s** (126 s au premier essai) |
| Arrêt brutal simulé (processus tués) puis redémarrage : monde opérationnel, arrêt propre | OK |
| Image ISO (IMAPI2, 448 Mo) montée en lecture seule : détection, copie assistée `--install-to`, fichiers rendus inscriptibles, launcher installé démarré automatiquement | OK |

Package final : 290 fichiers, 445 Mo (CD-ROM), zip 132 Mo, ISO 447 Mo. Dossiers de test laissés : `C:\HomeCraftTest` (installation complète avec données extraites, ~4 Go, à supprimer manuellement).

## Critères du cahier des charges (§11, §12) et couverture

- Sauvegarde cohérente pendant les écritures : tables verrouillées pendant `mariadb-dump` ; pendant le jeu, `saveall` préalable. **Couvert.**
- Sauvegardes hors des fichiers remplaçables : `UserData/`. **Couvert.**
- Aucune suppression de données personnelles sans avertissement : suppression d'archive et restauration passent par une confirmation explicite ; jamais de purge automatique. **Couvert.**
- Ne jamais écraser une base contenant des personnages par une base vide : `import_dumps` refuse `characters` non vide ; la restauration crée d'abord une sauvegarde de sécurité. **Couvert.**
- Manifeste de versions et empreintes vérifiables : `Build/manifest.json` (composants) + `Build/files-manifest.json` (fichiers). **Couvert.**
- Réparations impossibles présentées clairement : liste des fichiers manquants/altérés avec la mention « réinstaller ». **Couvert.**
- Non couvert : reconstitution des fichiers altérés depuis les archives téléchargées ; SRP6 hors ligne pour les comptes.
