# Phase 7 — validation finale (version candidate 0.9.0)

Date : 2026-10-15. Poste : Windows 11 Famille, client 1.12.1 frFR (build 5880) fourni par l'utilisateur.
Tous les tests sont des scripts rejouables (`Tests/`), exécutés sur l'exécutable release signé et sur le package portable construit par `Build/New-Package.ps1`.

## Critères du cahier des charges (§16, Phase 7)

| Critère | Preuve | Verdict |
|---|---|---|
| Démarrage complet | `Test-Launcher` : MariaDB t+1 s, realmd t+3 s, « World initialized » et port 8085 t+6 s ; `Test-Package` : première installation vierge (datadir, import des 4 bases + compléments, extraction dbc/maps/vmaps) → monde prêt en 110 à 126 s | **Validé** |
| Fermeture propre | `Test-Launcher` : `server exit` + fermeture de stdin, `SHUTDOWN` MariaDB, ports 3315/3724/8085 libérés, aucun processus orphelin, aucune ERREUR | **Validé** |
| Redémarrage après arrêt brutal | `Test-Package` : processus tués pendant le fonctionnement, redémarrage → monde opérationnel, arrêt propre (récupération InnoDB/MyISAM automatique) | **Validé** |
| Sauvegardes | `Test-Phase5` : archive avec `characters.sql`, `realmd.sql`, `settings.json`, manifeste SHA-256 ; export cohérent (tables verrouillées) ; `saveall` préalable si le jeu tourne | **Validé** (voir note) |
| Restauration | `Test-Phase5` : empreintes vérifiées, sauvegarde de sécurité automatique, compte témoin supprimé puis revenu | **Validé** |
| PlayerBots | Compagnons : connexion réelle de l'utilisateur et création de personnage confirmées le 2026-10-15 ; commandes `.partybot` vérifiées dans le code VMaNGOS, réglages exposés. Bots de peuplement : 20 personnages générés, 15 connectés en ville en 2 min (`botgen`) | **Validé côté serveur** ; comportement en donjon à confirmer par l'utilisateur en jeu |
| Localisation | `Test-Localisation` : après les compléments HomeCraft du 2026-10-15, **0 quête réelle** sans titre ou texte français (97 % des 4 433 entrées, le reste étant technique), **0 dialogue de PNJ** manquant, **0 créature visible** sans nom français (91 % des 10 381 entrées ; les 19 entrées du monde restantes sont des déclencheurs invisibles, le reste n'est jamais placé dans le monde) ; objets 99 %, zones 100 %, textes diffusés 86 %, chaînes serveur 95 % ; accents et UTF-8 vérifiés ; interface, sorts et DBC par le client frFR (locale détectée automatiquement) ; launcher intégralement en français | **Validé** (`09-Localisation-Inventaire.md`) |
| Absence de configuration manuelle | `Test-Package` : dossier vierge + `settings.json` ne contenant que l'emplacement du client (absent de `Client/` dans ce test) → tout le reste est généré (identifiants SQL, `my.ini`, `.conf`, royaume, comptes autorisés, `realmlist.wtf`, `Config.wtf`) | **Validé** |
| Fonctionnement hors ligne | `Test-Offline` : les trois services n'écoutent que sur 127.0.0.1, aucune connexion sortante ; arbre des dépendances du launcher sans aucune bibliothèque réseau (231 crates) ; aucun téléchargement à l'exécution | **Validé** |
| Support physique | ISO créée (IMAPI2), montée en lecture seule : détection, copie assistée, launcher installé démarré | **Validé** |
| Signature | Exécutable signé (certificat auto-signé, horodatage DigiCert) ; `SHA256SUMS.txt` et manifeste de fichiers dans le package | **Validé**, avertissement « éditeur inconnu » attendu |

## Résultats de la chaîne du 2026-10-15 (10:01 → 10:17)

| Test | Résultat |
|---|---|
| `Test-Localisation` | RÉUSSI |
| `Test-Offline` | RÉUSSI |
| `Test-Launcher` | RÉUSSI (10/10) |
| `Test-Phase5` | première passe : 2 attentes obsolètes (base déjà en service désormais laissée active ; manifeste de développement à régénérer) → script corrigé ; une fuite restait (arrêt à la fermeture d'une base non démarrée par le launcher) → propriété de la base rendue explicite ; **troisième passe : RÉUSSI (16/16)** |
| `New-Package` 0.9.0 | 306 fichiers, 450 Mo (CD-ROM), zip 133 Mo |
| `Test-Package` | RÉUSSI (12/12), ISO 452 Mo |

## Réserves et éléments non validés par un test automatique

- **Jeu en donjon avec compagnons** (tank/soigneur/DPS, suivi avec mmaps) : à confirmer par l'utilisateur en jeu ; aucun moyen d'automatiser le client.
- **Machine propre** : la « copie vierge » est un dossier vierge sur le poste de développement, pas une autre machine. Un test sur un PC sans outil de développement reste à faire (le package ne dépend d'aucun runtime : vérifié par l'absence de dépendances dynamiques hors système dans l'exécutable Rust et les binaires VMaNGOS/MariaDB fournis avec leurs DLL).
- **mmaps** : depuis le 2026-10-15 (soir), générées automatiquement en arrière-plan après le premier démarrage du serveur (état affiché sur l'accueil, reprise automatique si le launcher est fermé avant la fin) ; elles sont actives au démarrage suivant du serveur. Désactivable dans Paramètres.
- **Patchs graphiques HD** : non testés faute d'archives ; le gestionnaire est en place.
- **Certificat** : auto-signé ; un certificat reconnu reste nécessaire pour une distribution publique sans avertissement.
- **Recompilation de VMaNGOS** : non faite ; le correctif du build 5880 est un patch binaire documenté (`Build/Patch-MangosdBuilds.ps1`).

## Fonctionnalités livrées en plus du cahier des charges initial (demandes de l'utilisateur)

AHBot approvisionné (6 392 objets), onglets Comptes (création, mot de passe, niveau GM, **suppression définitive avec confirmation**, personnages compris), PlayerBots (réglages + génération de personnages bots de peuplement), Monde (70 réglages), Patchs (archives MPQ : activer, ajouter, **construire depuis un dossier**), Base de données (**explorateur natif façon HeidiSQL** : grille éditable, structure, SQL ; bouton HeidiSQL corrigé), section AddOns dans Paramètres (installation depuis zip), VanillaFixes et DXVK en option, bouton FERMER LE JEU, résolution bornée à l'écran. Tests automatisés correspondants : suppression de compte hors ligne (compte + personnage + objets + rappels + bot supprimés), AddOn installé/désactivé/réactivé/supprimé depuis une archive GitHub, patchs VanillaHD A et B construits et client démarré avec les deux.

## Compléments de robustesse (revue « le launcher est-il complet ? », 2026-10-15 soir)

Livrés et testés : mmaps en arrière-plan avec indicateur sur l'accueil, sauvegarde automatique à chaque fermeture du jeu avec rotation (10 conservées par défaut), réimport de la base monde sans perte de personnages (`--do reimportworld`, couvert par `Test-Phase5`), journalisation des imports de compléments SQL (vérifiée par `Test-Package`). Détail : `04-Launcher-Exigences.md`, section « Compléments de robustesse ». Restent à la charge de l'utilisateur : validation en jeu, certificat reconnu, test sur machine propre.

## Installateur Windows et distribution (2026-10-16)

L'utilisateur distribuera HomeCraft à sa famille et à ses amis : installateur Inno Setup ajouté (`06-Packaging-Portable.md`), licence propriétaire `LICENCE.txt` présentée et acceptée dans l'assistant, manuel d'utilisation installé et accessible depuis le menu Démarrer, icône et informations de version intégrées à l'exécutable. `Tests/Test-Installer.ps1` (00:22, 2026-10-16) : **RÉUSSI (14/14)** : installation silencieuse (68 Mo, signée), raccourcis, dossier du client écrit dans `settings.json`, premier démarrage complet du launcher installé (monde opérationnel à t+180 s), arrêt propre par fermeture de fenêtre, désinstallation silencieuse avec programme retiré et `UserData` conservé. Réserve inchangée : certificat auto-signé, donc avertissement SmartScreen chez chaque destinataire (expliqué dans l'assistant et le manuel).

## Localisation : quêtes, dialogues et créatures (demande de l'utilisateur, 2026-10-15)

Constat initial : 138 quêtes réelles sur 4 433 gardaient un titre anglais (et une cinquantaine un texte), 6 dialogues de PNJ sur 3 536 restaient en anglais, et 2 906 créatures présentes dans le monde n'avaient pas de nom français en base.

Compléments livrés le même jour (`Server/Database/custom/`, importés automatiquement par le launcher, rejouables : seuls les champs vides sont remplis, les textes officiels ne sont jamais écrasés) :

- `locales_quest_homecraft.sql` : les 138 titres, tous les textes manquants des quêtes réelles (dont les variantes « patch » de VMaNGOS, par exemple les quêtes de talismans du goulet des Chanteguerres) et les 6 dialogues de PNJ. **Traductions HomeCraft, non officielles**, signalées comme telles dans le fichier, le `LISEZMOI.txt` du package et `Localization/frFR/LISEZMOI.md` ; balises du jeu conservées.
- `locales_creature_homecraft.sql` : 2 887 noms, selon la règle vérifiée sur les traductions officielles de la base (« Maréchal Dughan », « Garde Thomas », « Adjoint Willem ») : le titre est traduit, le nom propre conservé. Résultat : 3 titres traduits (« Chef Galgosh », « Chef Tho'grun », « M. Bigglesworth »), 2 884 noms propres identiques à l'anglais, 20 entrées techniques ignorées.

Résultat mesuré (`Test-Localisation`, 2026-10-15) : 0 quête réelle incomplète, 0 dialogue de PNJ manquant, 0 créature visible sans nom français (19 déclencheurs invisibles « Trigger », « Doodad », « Debug Target » restent nommés en anglais, volontairement). Les 128 quêtes encore sans titre français sont toutes techniques (`<UNUSED>`, `<nyi>`, `REUSE`, `DEPRECATED`, `<TXT>`, quêtes de test) et jamais proposées au joueur. Détail : `09-Localisation-Inventaire.md`.
