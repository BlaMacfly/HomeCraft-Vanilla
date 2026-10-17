<p align="center">
  <img src="Documentation/images/logo.png" alt="HomeCraft Edition BC" width="200">
</p>

<h1 align="center">HomeCraft Edition BC</h1>

<p align="center">
  <strong>World of Warcraft: The Burning Crusade 2.4.3 en solo, hors ligne, en français.</strong><br>
  Un serveur local automatisé, des compagnons et des bots contrôlés par l'ordinateur, un launcher à un seul bouton.
</p>

Édition personnelle de **World of Warcraft: The Burning Crusade 2.4.3** (client build 8606, frFR) jouable en solo,
avec un launcher Windows écrit en Rust qui démarre et surveille tout le serveur local : MariaDB portable, serveur
d'authentification et serveur monde **CMaNGOS mangos-tbc**, compagnons et bots du module **PlayerBots**, hôtel des
ventes approvisionné (AHBot), interface et textes en français. Tout fonctionne sur 127.0.0.1, sans compilation ni
installation côté joueur.

Branche sœur de [HomeCraft Edition Vanilla](https://github.com/BlaMacfly/HomeCraft-Vanilla) (branche `main`, 1.12.1
sur VMaNGOS) : même launcher, même disposition, adapté à TBC.

![Accueil du launcher](Documentation/images/launcher-home.png)

## Contenu de cette branche

| Dossier | Contenu |
|---|---|
| `Launcher/` | launcher Rust (eframe/egui 0.32, client MySQL pur Rust), un seul exécutable |
| `Tools/` | scripts PowerShell d'exploitation (MariaDB, import des bases, configurations, extraction des données du client, harnais serveur) ; `Tools/Localisation/` : outils de mesure et de complément des traductions françaises |
| `Build/` | manifeste des composants figés (URL, commit, SHA-256), téléchargement, signature, manifeste de fichiers, packaging portable |
| `Server/Database/custom/` | compléments appliqués après l'import de TBC-DB : correctif de schéma `locales_gameobject`, traductions françaises |
| `Tests/` | tests d'intégration rejouables (fiabilité, package) |
| `Documentation/` | audit, construction, adaptation du launcher, inventaire de la localisation |

Ne sont **jamais** versionnés : le client du jeu, les données extraites du client, les données personnelles
(`UserData/`), les journaux, ni les binaires tiers (récupérés par `Build/Get-Components.ps1` d'après
`Build/manifest.json`).

## Composants (figés dans `Build/manifest.json`)

CMaNGOS mangos-tbc, build Windows officiel « with-all » (commit 15b6ddb4, GPL-2) ; module PlayerBots (GPL-2) ;
TBC-DB (GPL-3) ; MariaDB 11.8.9 (GPL-2) ; HeidiSQL 12.21 (GPL-2).

## Licence

Launcher, scripts et documentation : licence propriétaire HomeCraft (`LICENCE.txt` : usage personnel, lecture du
code autorisée). Les composants tiers et les compléments de base de données dérivés de projets GPL restent sous leurs
licences. World of Warcraft et The Burning Crusade sont des marques de Blizzard Entertainment, Inc. ; HomeCraft est
une création indépendante, sans lien avec Blizzard.
