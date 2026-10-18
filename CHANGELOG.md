# Journal des versions

## 0.9.4 — 18 octobre 2026

- Surveillance des services : les ports du serveur ne sont plus sondés qu'une fois toutes les 10 secondes (l'état des processus reste contrôlé chaque seconde). Le launcher ouvrait une connexion au serveur de jeu à chaque seconde, journalisée par mangosd (« Connection accepted/closed », 9 434 lignes sur une journée de test).

## 0.9.3 — 18 octobre 2026

- Bots de peuplement : après une génération, le launcher règle au plus 300 bots connectés (minimum la moitié) au lieu du total généré, et active « Bots visibles dans /qui ». Mesure : avec 2 600 personnages bots, l'ancien réglage (minimum 1 300) faisait connecter plus de 1 000 bots d'un coup (1,1 Go pour le serveur, un cœur de processeur), invisibles dans `/qui`.
- Réglages « Nombre minimal / maximal de bots connectés » : limite relevée de 500 à 5 000, aide détaillée (connexion au démarrage, coût mesuré).
- Manuel : où se trouvent les bots (quartier commerçant de Hurlevent, Forgefer, Orgrimmar), combien en connecter, dépannage pas à pas.
- Nouveau document `Documentation/Commandes-GM.md` : commandes MJ et administrateur utiles en solo (en français) et liste complète des 756 commandes du serveur avec leur niveau, extraite du code de VMaNGOS (`Build/doc-tools/`).
## 0.9.2 — 17 octobre 2026

- Correctif : premier démarrage avec un client Benilla sans `base.MPQ`. VMapExtractor (VMaNGOS) ouvre cette archive et plantait (code 0xC0000005) sans elle. Le launcher vérifie désormais les archives d'extraction ; s'il en manque, il lit celles d'un client d'origine complet trouvé près de l'installation (sans rien copier), sinon il indique clairement l'archive à ajouter. `base.MPQ` peut rester dans le `Data\` de Benilla, qui ne la monte jamais.

## 0.9.1 — 17 octobre 2026

- **Client Benilla 64 bits pris en charge** en plus du client d'origine : reconnaissance d'un exécutable Benilla (`benilla.exe` ou renommé) à côté de `benilla-config\` et des douze archives 1.12.1 (`base.MPQ` non exigé), configuration de `benilla-config\config.toml` (`realmList`, `realmName`, `gxWindow`, `gxResolution`, le reste du fichier conservé), lancement direct, AddOns dans `benilla-config\AddOns`, VanillaFixes et DXVK désactivés pour ce client.
- Paramètres : type du client détecté sous le dossier, bouton « Rechercher les clients » (`Client` et dossiers voisins de l'installation) avec « Utiliser ».
- Accueil et journal : type du client affiché.
- Tests unitaires du module client (fusion du `config.toml`, détection, dossier réel si présent).

## 0.9.0 — 16 octobre 2026 (version candidate)

Première publication. Tout ce qui suit est couvert par les tests automatisés de `Tests/` (tous réussis).

### Jeu
- Serveur VMaNGOS 1.12.1 (commit `53d24d5`, build de développement du 2026-10-07) avec les PartyBots intégrés ; base de données VMaNGOS (snapshot `4641790`) sur MariaDB 11.8.9 portable, tout sur 127.0.0.1.
- Compagnons `.partybot` (tank, soigneur, DPS, classes au choix), bots de peuplement générés par le launcher (compte dédié), champs de bataille complétés par des bots.
- Hôtel des ventes approvisionné : 6 392 objets utiles en solo (`Server/Database/custom/auctionhousebot_solo.sql`).
- Compatibilité du client frFR build 5880 (realm, clients autorisés, correctif reproductible du serveur monde `Build/Patch-MangosdBuilds.ps1`).
- Traductions françaises complétées : 0 quête réelle, 0 dialogue de PNJ, 0 créature visible sans texte français (`locales_quest_homecraft.sql`, `locales_creature_homecraft.sql`, non officielles, jamais écrasantes).

### Launcher (Rust, egui)
- Un bouton JOUER : base → authentification → monde → configuration du client (realmlist, fenêtré, résolution bornée) → jeu. Premier démarrage automatique (import, extraction dbc/maps/vmaps), mmaps en arrière-plan.
- Onglets Accueil, Comptes (création, mot de passe, niveau GM, suppression avec personnages), PlayerBots (réglages + génération de bots), Monde (70 réglages avec défauts Vanilla), Patchs (activer, désactiver, ajouter, construire un MPQ depuis un dossier), Base de données (explorateur natif façon HeidiSQL + bouton HeidiSQL), Sauvegardes (manuelles, automatiques à la fermeture du jeu avec rotation, de sécurité, restauration vérifiée, diagnostic), Paramètres (client, affichage, VanillaFixes 1.5.3, DXVK 2.6.1, AHBot, mmaps, sauvegarde auto, réimport de la base monde, AddOns), Journaux (console serveur).
- Vérification des fichiers par manifeste SHA-256, réparation, bouton FERMER LE JEU, détection de support en lecture seule.
- Exécutable signé (certificat auto-signé, horodatage), icône et informations de version intégrées.

### Distribution
- Package portable `Build/New-Package.ps1` (dossier + zip ≈ 135 Mo, compatible CD-ROM) et image ISO `Build/New-Iso.ps1`. Le client n'est jamais inclus.
- Licence propriétaire `LICENCE.txt` (usage personnel ; consultation et fork sur GitHub autorisés) ; composants tiers sous leurs licences, sources référencées dans `LICENCES/SOURCES.md`.
- Manuel d'utilisation complet (`Documentation/Manuel-Utilisateur.md`) et documentation technique en dix chapitres.

### Connu, non corrigé
- Certificat auto-signé : avertissement SmartScreen « Éditeur inconnu » au premier lancement.
- Les bots de peuplement sont cachés de `/qui` par défaut (réglage dans PlayerBots).
- Le correctif du build 5880 est un patch binaire ; une recompilation de VMaNGOS avec le correctif source reste à faire.
