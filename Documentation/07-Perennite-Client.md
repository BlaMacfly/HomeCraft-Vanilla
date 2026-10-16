# Pérennité du client 1.12.1 (32 bits) — analyse

Question posée le 2026-10-14 (échange de l'utilisateur avec ChatGPT) : existe-t-il un « traducteur 32 bits → 64 bits » pour protéger le client Vanilla des incompatibilités futures ?

## Réponse courte

Non. Il n'existe pas d'outil qui convertisse un exécutable 32 bits en vrai programme 64 bits ; le code source de `WoW.exe` n'est pas disponible, et un tel portage serait de toute façon illégal pour un package distribué. Les trois options listées par ChatGPT sont exactes dans leur principe ; voici ce qu'elles valent concrètement pour ce projet, après vérification des dépôts cités.

## 1. Couche de compatibilité (WOW64) — ce que nous utilisons

- Windows 11 exécute les programmes 32 bits via WOW64 ; Microsoft n'a annoncé aucune fin de ce support sur x64. Le risque réel à moyen terme n'est pas le 32 bits mais **Direct3D 9 et le plein écran exclusif** sur des pilotes récents, et les décalages de synchronisation (saccades) du moteur de 2006.
- Parade déjà disponible et éprouvée : **DXVK** (traduction D3D9 → Vulkan, licence zlib) fonctionne aussi sous Windows ; le client fourni par l'utilisateur contient d'ailleurs un `WoW.dxvk-cache`, signe qu'il a tourné avec DXVK. Option à proposer dans le launcher (copie de `d3d9.dll` DXVK à côté de `WoW.exe`, désactivable) si des problèmes d'affichage apparaissent sur Windows 11 ou dans Wine.

## 2. Correctifs communautaires — vérifiés le 2026-10-14

| Projet | Vérifié | Verdict pour HomeCraft |
|---|---|---|
| **VanillaFixes** (hannesmann) | MIT ; corrige saccades et animations (timer haute précision) pour 1.6.1 → 1.12.1 ; ne modifie pas `WoW.exe` (charge `VfPatcher.dll` au lancement, se décharge à l'écran de connexion) ; testé par l'auteur avec VMaNGOS et Warden ; peut charger d'autres DLL listées dans `dlls.txt` | **Candidat sérieux**, redistribuable (MIT). À tester avec le client frFR, puis à intégrer comme option du launcher (« Lancer via VanillaFixes »), activée par défaut si le test est concluant. |
| **SuperWoW** (balakethelock) | **Source fermé**, licence « usage personnel uniquement », redistribution et modification interdites sans accord écrit ; injecte une DLL ; étend l'API Lua, correctifs divers | **Exclu du package** (licence). L'utilisateur peut l'ajouter lui-même ; sans intérêt pour le solo. |

## 3. Portage 64 bits réel — la seule voie est une réimplémentation

Le seul chemin vers un client 64 bits est un client réécrit lisant les données du jeu : **Benilla** (Rust, MIT/Apache, audité en Phase 1). Verdict inchangé : actif et prometteur, mais pas de binaire, API instable, et surtout lecture des DBC en anglais seulement (issue #429), donc incompatible aujourd'hui avec l'objectif « tout en français ». À réévaluer périodiquement ; le launcher pourrait à terme proposer « client original » ou « Benilla » au choix.

Mise à jour du 2026-10-16 (vérification du dépôt) : README « Complete and fully playable », 733 étoiles, 605 issues ouvertes, licence MIT ou Apache-2.0, aucun binaire (tags Git seulement), Warden « not planned », interface Lua/FrameXML compatible Questie, pfUI, Bagnon. L'issue #429 (DBC lus en enUS seulement) a été fermée le 2026-10-15 et remplacée par le défaut **#1338** : la prise en charge des locales est désormais considérée comme un bug à corriger, plus comme un choix. Verdict inchangé pour HomeCraft : pas de binaire, pas de release stable, français non garanti ; à réévaluer quand #1338 sera clos et qu'un binaire Windows sera publié.

## Intégration réalisée (2026-10-14, 23:58, décision utilisateur)

- Composant figé `vanillafixes` dans `Build/manifest.json` : `vanillafixes-1.5.3-dxvk.zip` (1,7 Mo, SHA-256 figé), extrait dans `Tools/VanillaFixes/` avec les textes de licence MIT et zlib récupérés depuis les dépôts (v1.5.3, DXVK v2.6.1).
- Launcher, onglet Paramètres : « Lancer le jeu via VanillaFixes » et « Rendu DXVK », **désactivés par défaut**. Si l'option est active, `VanillaFixes.exe` et `VfPatcher.dll` sont copiés dans le dossier du client (mis à jour par empreinte), `dlls.txt` seulement s'il n'existe pas (l'utilisateur peut y ajouter ses DLL), puis le jeu est lancé par `VanillaFixes.exe`. DXVK : `d3d9.dll` et `dxvk.conf` copiés à côté de `WoW.exe` ; décocher retire `d3d9.dll` uniquement si c'est notre copie (même empreinte).
- Si `Tools/VanillaFixes` est absent, le launcher le signale et lance `WoW.exe` directement.

## Décision proposée

1. Rester sur le client officiel 1.12.1 frFR (fidélité, français intégral).
2. Ajouter VanillaFixes comme composant optionnel figé dans `Build/manifest.json` après test frFR (saccades, Warden off).
3. Prévoir DXVK en option de secours pour l'affichage.
4. Suivre Benilla (locales, releases) sans en dépendre.
