# Fonctions de HomeCraft Edition Vanilla 1.0 à porter dans les éditions BC et WotLK

Document de référence (1er novembre 2026) pour les sessions qui portent dans **HomeCraft Edition BC** (`..\HomeCraft BC`,
CMaNGOS `mangos-tbc`, client 2.4.3) et **HomeCraft Edition WoTLK** (`..\HomeCraft WoTLK`, fork AzerothCore + `mod-playerbots`,
client 3.4.3 par HermesProxy) les fonctions ajoutées à l'édition Vanilla pour sa version 1.0.0 (branche `cmangos` de ce dépôt,
CMaNGOS Classic + playerbots). Il décrit **ce qui existe, où est le code, ce qui se reprend tel quel et ce qui dépend du core**.

Règle commune : ne jamais confondre code écrit, code compilé, code testé et fonctionnalité réellement opérationnelle ; ne
jamais inventer un résultat ; aucun secret (clé d'API, mot de passe, certificat privé) dans un commit ni un journal. Le
rapport de validation de chaque édition suit le modèle de `docs/IMPLEMENTATION_VALIDATION.md` (statuts VALIDÉ / ÉCHEC /
PARTIEL / NON TESTÉ / BLOQUÉ, avec commande, journal et chemin de preuve).

## Vue d'ensemble

| Fonction | Code Vanilla (launcher Rust) | Reprise | Dépend du core / du client |
|---|---|---|---|
| Secrets chiffrés DPAPI | `Launcher/src/secret.rs` | telle quelle (changer l'entropie : « HomeCraft Edition BC / … ») | non |
| Multijoueur intégré (Noise sur TCP, invitations révocables) | `Launcher/src/network.rs`, `Launcher/src/ui/multiplayer.rs` | quasi telle quelle (ports) | non : le serveur ne voit que des connexions locales |
| Perçage de NAT (STUN, QUIC, code de réponse) | `Launcher/src/nat.rs` | telle quelle | non |
| Profils de lancement | `Launcher/src/profiles.rs`, `Launcher/src/ui/profiles.rs`, `client.rs` (`launch_with`, `Prefill`) | modèle et UI tels quels ; **préremplissage à refaire** | client : `accountName` du Config.wtf en 2.4.3 ; 3.4.3 + Arctium + HermesProxy à étudier |
| Plusieurs clients suivis (un par profil) | `orchestrator.rs` (`GameProc`, `play_with`, `poll_games`) | telle quelle | non |
| Multiboxing (plusieurs sessions par compte) | correctif du core (`Build/cmangos-patches/homecraft.patch`) | **à refaire dans chaque core** | oui |
| Pas d'expulsion AFK | correctif du core (même patch) | **à refaire** | oui |
| Pont IA des bots (7 fournisseurs, clé chiffrée, intentions en liste blanche) | `Launcher/src/ai_bridge.rs`, `Launcher/src/ui/ai.rs` ; correctif du module (`Build/cmangos-patches/playerbots.patch`) | pont tel quel ; **intégration au module à refaire** | oui : interface LLM du module playerbots |
| Sélecteur de langue FR/EN | `Launcher/src/i18n.rs` (`tr!`, `t()`), `i18n_en.rs`, `conf_catalog_en.rs` | mécanisme tel quel ; textes à traduire | non |
| Client de test du protocole | `Tests/wowclient` (Rust) | **à adapter** (chiffrement d'en-tête et opcodes diffèrent) | oui |
| Tests de bout en bout | `Tests/Test-Multisession.ps1`, `Test-Network.ps1`, `Test-AI.ps1`, `Test-Profiles.ps1` | à adapter (ports, comptes, client de test) | — |
| Manuel avec schémas | `Documentation/Manuel-Utilisateur.md`, `Documentation/images/manuel/*.svg` | structure et schémas à adapter | — |

## 1. Secrets (`secret.rs`)

`protect()` / `unprotect()` : `CryptProtectData` / `CryptUnprotectData` (windows-sys, feature `Win32_Security_Cryptography`),
portée utilisateur, entropie propre à l'édition, stockage hexadécimal. `random_token()`, `masked()`. Tests unitaires inclus.
Sert aux mots de passe des profils, à la clé d'API de l'IA. Jamais de secret en clair sur disque ni dans un journal.

## 2. Multijoueur intégré (`network.rs`, `nat.rs`, `ui/multiplayer.rs`)

**Principe** : aucun programme externe (EasyTier a été essayé puis écarté : LGPL incompatible avec la licence propriétaire du
launcher en liaison statique, et sa publication Windows embarque Packet.dll de Npcap, non redistribuable).

- Hôte : `Host::start(port, (port_auth, port_monde))` écoute en TCP (11010 par défaut). Chaque connexion commence par un en-tête
  en clair `HCL1` + identifiant d'invitation (16 octets) + type (0 présence, 1 authentification, 2 monde), lié au chiffrement
  (prologue Noise), puis échange `Noise_NNpsk0_25519_ChaChaPoly_BLAKE2s` avec la clé de l'invitation (32 octets). Le type ne
  peut mener qu'à 127.0.0.1:auth ou 127.0.0.1:monde : MariaDB et le reste sont injoignables par construction.
- Invitation `HC2-…` (JSON en base64url) : identifiant, clé, adresses de l'hôte, expiration, nom de l'hôte, adresses QUIC et
  empreinte du certificat. Une clé par joueur, révocable (`Host::revoke` coupe les connexions et efface la clé), rien de
  persistant (fermer la partie invalide tout).
- Invité : `Guest::start` écoute sur 127.0.0.1:auth et :monde ; le client du jeu se connecte « en local ». Le serveur voit les
  invités en 127.0.0.1 : rien à régler dans le core (sauf si le core refuse plusieurs connexions de la même IP : à vérifier).
- Accessibilité de l'hôte sans redirection manuelle : 1) **UPnP** (`igd-next`) ; 2) **perçage de NAT** (`nat.rs`) : STUN
  public (Google, Cloudflare, Nextcloud) pour l'adresse publique, QUIC (`quinn` + `rustls` ring) dont le certificat est épinglé
  par empreinte dans l'invitation, code de réponse `HCR-…` signé HMAC-SHA256 avec la clé de l'invitation ; chaque flux QUIC est
  raccordé à une connexion TCP locale qui passe par le même Noise ; 3) redirection manuelle en dernier recours.
- Mode invité : `network::guest_active()` empêche le démarrage du serveur local (ports pris) ; `Command::PlayGuest` lance le
  client sans serveur.
- Tests : `cargo test network` et `cargo test nat` (tunnel, clé fausse, révocation, expiration, type inconnu, en-tête invalide,
  port occupé, chemin QUIC complet, empreinte fausse, code de réponse) ; `Tests/Test-Network.ps1 [-QuicOnly]` (deux launchers,
  vrai client, secrets absents des journaux). **Non testé** : traversée réelle de deux box (deux réseaux distincts).
- **À adapter** : ports de l'édition (BC 3726/8087 ; WotLK 3725/8086, plus le proxy HermesProxy côté client : le client 3.4.3
  parle à HermesProxy, qui parle au core en 3.3.5a — vérifier où se placent les redirections de l'invité : il faut que
  HermesProxy de l'invité joigne 127.0.0.1:auth/monde de l'invité, ce qui revient au même mécanisme ; à tester).
- Dépendances ajoutées : `snow`, `igd-next` (tire `attohttpc`, MPL-2.0, utilisé sans modification), `quinn`, `tokio`, `rcgen`,
  `rustls` ; licences vérifiées par `cargo metadata` (aucune GPL/LGPL obligatoire).

## 3. Profils de lancement (`profiles.rs`, `ui/profiles.rs`, `client.rs`)

- `Profiles` dans `UserData/profiles.json` : nom, compte, mot de passe chiffré (DPAPI), personnage préféré, entrée directe en
  jeu, arguments du client ; profil par défaut ; boutons sur l'accueil ; fenêtre de gestion (créer, modifier, renommer,
  supprimer, défaut). Un client par profil (`orchestrator::play_with` refuse un second lancement du même profil).
- Préremplissage, par ordre de préférence : mécanisme officiel du client > adaptation du client (si sources) > intégration
  dédiée > automatisation locale (dernier recours, documentée).
  - Vanilla Benilla : variables `WOW_USER`/`WOW_PASS`/`WOW_CHAR`/`WOW_CHARSELECT_PICK` passées au seul processus du jeu,
    compte dans `benilla-config\account`.
  - Vanilla client d'origine : `SET accountName` dans `WTF\Config.wtf` ; mot de passe saisi par `PostMessageW(WM_CHAR)` dans
    la fenêtre du processus lancé, 8 s après son apparition (`client::type_into_login`).
  - **BC 2.4.3** : `accountName` existe aussi dans Config.wtf (à vérifier) ; même automatisation possible pour le mot de passe.
  - **WotLK 3.4.3 (Arctium + HermesProxy)** : le client Classic se connecte avec un portail Battle.net émulé ; vérifier ce que
    HermesProxy et Arctium permettent (identifiants préremplis, `SET accountName`, `-config`), sinon automatisation locale.
- Tests : `cargo test profiles` (mot de passe jamais en clair, arguments) ; `Tests/Test-Profiles.ps1 -Client <copie isolée du
  client>` (le launcher lance le client avec le profil ; preuve côté serveur : `realmd.account_logons` après le lancement,
  `Char.log` pour l'entrée en jeu ; recherche du mot de passe en clair dans tous les fichiers). Utiliser une copie isolée du
  client (MPQ en liens physiques), jamais le client de l'utilisateur.

## 4. Multiboxing et AFK (correctifs du core)

CMaNGOS Classic (fichiers modifiés, voir `Build/cmangos-patches/homecraft.patch`) :

- `World.h/.cpp` : `SessionMap` en `std::unordered_multimap<uint32, WorldSession*>` ; options `HomeCraft.AllowMultipleSessionsPerAccount`,
  `HomeCraft.MaxSessionsPerAccount` (1-10), `HomeCraft.DisableAfkKick` ; `AddSession_` refuse au-delà du maximum **sans
  déconnecter les autres** ; `FindSessionForReconnect` (reprise d'une session hors ligne du compte) ; `FindSessionForCallback`
  (les requêtes asynchrones retrouvent la bonne session par une clé propre à la session, plus par l'identifiant de compte).
- `WorldSession.h/.cpp` : `m_callbackKey`, `GetCallbackId()` ; l'expulsion AFK (`ShouldAfkDisconnect`, 15 min) est ignorée si
  `DisableAfkKick`, la détection d'une vraie coupure (socket fermé) est inchangée.
- `WorldSocket.cpp`, `CharacterHandler.cpp`, `MiscHandler.cpp`, `QueryHandler.cpp` : callbacks par `GetCallbackId()`.
- Launcher : trois options du groupe « Sessions » (`conf_catalog.rs`), écrites en fin de `mangosd.conf`.
- **BC (mangos-tbc)** : code très proche de Classic : le même correctif devrait s'appliquer avec peu d'ajustements (à vérifier).
- **WotLK (AzerothCore)** : structure différente (`WorldSessionMgr`, `sWorldSessionMgr`, `World::AddSession_`, `CharacterHandler`
  asynchrone par `QueryCallback`) ; AzerothCore a déjà des options proches (vérifier `AllowTwoSide`, `MaxPlayerLevel`… et
  surtout l'expulsion AFK : chercher `ShouldAfkDisconnect` / `AfkTime`). Refaire le même comportement, mêmes noms d'options.
- Tests : `Tests/Test-Multisession.ps1 -Afk` (2 puis 4 sessions, 5e refusée, coupure brutale et reconnexion, sauvegarde, AFK
  17 min sans expulsion) puis `-AfkControlOnly` (option désactivée : expulsion vers 15 min, preuve que le test détecte l'AFK).
  Pièges PowerShell relevés : variables insensibles à la casse (`$n` écrase `$N`), `ExitCode` vide sans lecture de `.Handle`,
  court-circuit de `-and` qui saute une attente, délai d'attente inférieur à la durée de présence demandée.

## 5. Pont IA des bots (`ai_bridge.rs`, `ui/ai.rs`, correctif du module)

- Pont HTTP local (`tiny_http`, 127.0.0.1:15910 en Vanilla, jeton local `UserData/ai-bridge-token`), fournisseurs OpenAI,
  Anthropic, Mistral, OpenRouter, Ollama, LM Studio, compatible OpenAI (`ureq`), clé chiffrée DPAPI, liste des modèles
  (jamais de téléchargement), test, nettoyage de la réponse, au plus une intention de la liste blanche (`follow`, `stay`,
  `attack`, `wave`, `bow`, `cheer`, `laugh`, `salute`). En multijoueur, seul l'hôte appelle le fournisseur.
- Module cmangos/playerbots (Classic, et **probablement TBC**, même module) : `PlayerbotLLMInterface` (disjoncteur 60 s vers
  les réponses classiques, `ExtractIntents`/`TakeIntents`), `SayAction.cpp` (identifiants du bot et du joueur passés à la
  génération), `PlayerbotAI::UpdateAI` (intentions exécutées dans le fil du monde, joueur à moins de 60 m, avec les droits d'un
  ordre chuchoté). Configuration écrite par `ai_bridge::conf_values` dans `aiplayerbot.conf`.
- **WotLK (`mod-playerbots` de liyunfan1223)** : ce module n'a pas forcément d'interface LLM ; vérifier. S'il n'en a pas, il
  faut l'ajouter (point d'entrée des messages chuchotés / dits aux bots, appel asynchrone au pont, réponse et intentions dans
  le fil du monde), avec le même pont.
- Tests : `cargo test ai_bridge` ; `Tests/Test-AI.ps1` (LM Studio local, un vrai client chuchote à un bot, réponse, intention,
  repli quand le fournisseur est injoignable, clé factice introuvable en clair).

## 6. Langue FR/EN

`i18n.rs` : module `fr` (constantes et fonctions), module `en` dans `i18n_en.rs` avec les **mêmes noms** (test
`memes_entrees`), macro `tr!(NOM)` / `tr!(fonction)(args)`, `t("fr", "en")` pour les textes écrits sur place ;
`conf_catalog_en.rs` traduit libellés, aides, groupes et choix du catalogue (test `toutes_les_cles_traduites`). Sélecteur en
haut de Paramètres, enregistré dans `settings.json` (`language`). Les journaux techniques restent en français.

## 7. Documentation

Manuel réécrit (`Documentation/Manuel-Utilisateur.md`, 29 sections) avec 8 schémas SVG (`Documentation/images/manuel/`) :
architecture, premier démarrage, multijoueur, perçage de NAT, pont IA, profils, sauvegardes, dossiers. Liste des commandes MJ
régénérée depuis les sources du core (`Build/doc-tools/cmdtree.pl <Chat.cpp> cmangos`, puis `cmdmd.pl`) : à refaire pour
chaque core (AzerothCore stocke ses commandes en base, table `command`, et dans le code `*_commandscript.cpp`).

## 8. Ordre conseillé pour chaque édition

1. Lire ce document et le code Vanilla cité ; vérifier l'état réel de l'édition (git, launcher, core compilé).
2. Reprendre `secret.rs`, `i18n` (mécanisme), `profiles.rs` + UI, suivi multi-clients de l'orchestrateur.
3. Correctif du core : multisession + AFK ; adapter le client de test du protocole ; `Test-Multisession`.
4. Multijoueur (`network.rs`, `nat.rs`) ; `Test-Network` et `-QuicOnly`.
5. Pont IA + intégration au module de bots ; `Test-AI`.
6. Préremplissage des profils pour le client de l'édition ; `Test-Profiles`.
7. Traductions EN, manuel et schémas, rapport de validation, version 1.0.0.
