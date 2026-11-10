# HomeCraft Edition Vanilla 1.0.0 — validation de l'implémentation

Rapport de la phase 8 (1er novembre 2026), branche `cmangos` (CMaNGOS Classic `ff53dfc` + playerbots `39d4700`,
launcher Rust 1.0.0). Chaque ligne distingue ce qui est **écrit**, **compilé**, **testé** et **réellement opérationnel** ;
aucun résultat n'est supposé. Les preuves sont dans `docs/preuves/` (journaux recopiés, adresses IP publiques masquées) et
les commandes sont rejouables depuis la racine du dépôt.

**Statuts** : VALIDÉ (testé de bout en bout, preuve jointe) · PARTIEL (une partie testée, le reste dit) · NON TESTÉ (écrit et
compilé, aucun test n'a pu l'exercer) · ÉCHEC · BLOQUÉ (test impossible pour une raison extérieure, dite).

## Environnement

| Élément | Valeur |
|---|---|
| Machine | Windows 11 Famille 10.0.26200, une seule machine (box Orange Livebox, NAT « port restricted », UPnP actif) |
| Compilateur | rustc 1.99.0, cargo 1.99.0 ; Visual Studio Build Tools 2022 (core C++) |
| Serveur | `Server/cmangos/mangosd.exe` SHA-256 `628c47524248d458…` (core + module corrigés, `Build/Build-Cmangos.ps1`) |
| Installation de test | `%USERPROFILE%\HomeCraftTests\cm` (copie isolée ; jamais le dossier de développement ni l'installation de l'utilisateur) |
| Clients réels | Benilla (copie isolée, MPQ en liens physiques) ; client d'origine 1.12.1 frFR build 5875 (copie isolée) |
| Client de test | `Tests/wowclient` (Rust) : protocole 1.12 complet (SRP6, en-têtes chiffrés, création de personnage, entrée en jeu, AFK, chuchotements, déconnexion) |
| Incidents | deux interruptions du PC pendant la campagne (écran bleu 0x1E à 10 h 23, puis redémarrage manuel à 11 h 39) ; les tests concernés ont été relancés |

## Compilation

| Composant | Commande | Résultat | Preuve |
|---|---|---|---|
| Core CMaNGOS + module playerbots (correctifs sessions, AFK, IA) | `Build/Build-Cmangos.ps1` | VALIDÉ : compilé, installé, SHA-256 ci-dessus | journal de build (session) |
| Launcher 1.0.0 | `cargo build --release` (Launcher/) | VALIDÉ : 0 erreur | — |
| Client de test | `cargo build --release` (Tests/wowclient) | VALIDÉ | — |
| Package portable | `Build/New-Package.ps1` | VALIDÉ : 380 fichiers, 476 Mo ; sans client, UserData, secret ni composant externe | `Build/package/HomeCraft-Edition-Vanilla-v1.0.0` |
| Dépendances Visual C++ | `Tests/Test-Dependencies.ps1` | VALIDÉ : 89 fichiers, aucune dépendance à un runtime installé | `docs/preuves/dependances-package-1.0.0.txt` |
| Licences des crates | `cargo metadata` | VALIDÉ : aucune GPL/LGPL obligatoire ; `attohttpc` (MPL-2.0, via `igd-next`) utilisé sans modification | — |

## Tests automatisés (unitaires)

`cargo test --release` (Launcher/) : **36 réussis, 0 échec** — `docs/preuves/tests-unitaires-launcher.txt`.

| Domaine | Tests |
|---|---|
| Secrets | aller-retour DPAPI (rien en clair), masquage |
| Réseau | code d'invitation (aller-retour, espaces, altération, expiration), base64, tunnel Noise de bout en bout (200 Ko), clé fausse refusée, révocation (connexion coupée et refus), expiration, type de connexion inconnu (tentative vers un autre port) refusé, en-tête invalide rejeté, port local occupé, code de réponse signé (autre clé, adresses modifiées refusées), **liaison complète par QUIC** |
| Perçage de NAT | réponse STUN, HMAC (vecteur RFC 4231), tuyau QUIC local (100 Ko), certificat à empreinte fausse refusé |
| IA | nettoyage de réponse et intentions (liste blanche, une seule, balises inventées supprimées), consignes |
| Profils | mot de passe jamais en clair dans profiles.json, arguments, comptes valides |
| Langue | toutes les entrées françaises ont leur traduction anglaise ; toutes les clés du catalogue de réglages sont traduites |
| Existants | client (Benilla, cvars), dépôt, réparation, journaux sans mots de passe, utilitaires |

## Tests de bout en bout

| # | Fonction | Commande | Statut | Preuve |
|---|---|---|---|---|
| A1 | 2 personnages du même compte en jeu ensemble | `Tests/Test-Multisession.ps1 -Afk` | VALIDÉ (online = 2 en base) | `docs/preuves/multisession-run6.txt` |
| A2 | 4 personnages du même compte | idem | VALIDÉ (online = 4) | idem |
| A3 | 5e session refusée sans déconnecter les 4 autres | idem | VALIDÉ | idem |
| A4 | coupure brutale (processus tué) puis reconnexion, autre session non affectée | idem | VALIDÉ | idem |
| A5 | sauvegarde à la déconnexion (hors ligne, temps de jeu enregistré) | idem | VALIDÉ | idem |
| B1 | AFK 17 minutes sans expulsion (`DisableAfkKick = 1`) | idem | VALIDÉ | idem |
| B2 | contrôle : option à 0 → expulsion par le serveur au bout de 15 min (le test détecte bien l'AFK) | `Test-Multisession.ps1 -AfkControlOnly` | VALIDÉ | `docs/preuves/multisession-controle-afk.txt` |
| B3 | vraie coupure détectée et nettoyée avec l'option active | A4 | VALIDÉ | `multisession-run6.txt` |
| C1 | multijoueur, deux launchers, liaison TCP directe : vrai client créé et en jeu 40 s à travers la liaison (royaume et monde) | `Tests/Test-Network.ps1` | VALIDÉ (sur une seule machine) | `docs/preuves/network-run1.txt` |
| C2 | multijoueur par QUIC seul (chemin du perçage de NAT), même scénario | `Test-Network.ps1 -QuicOnly` | VALIDÉ (sur une seule machine) | `docs/preuves/reseau-quic.txt`, `reseau-quic-hote.log`, `reseau-quic-invite.log` |
| C3 | connexion sans invitation valide fermée sans réponse | C1 étape 2 | VALIDÉ | idem |
| C4 | MariaDB / autres ports de l'hôte injoignables par la liaison | tests unitaires `type_inconnu_et_en_tete_invalide_refuses` | VALIDÉ (par construction : seuls les types 1 et 2 existent, testés en refus) | `tests-unitaires-launcher.txt` |
| C5 | révocation d'un invité (connexions coupées, code refusé) | test unitaire `revocation_coupe_et_refuse` | VALIDÉ (unitaire, vraies connexions TCP) ; non rejoué entre deux launchers | idem |
| C6 | secrets (clé, code, mot de passe) absents des journaux | C1/C2 étape 4 | VALIDÉ | idem |
| C7 | adresse publique trouvée par STUN, UPnP de la box | journal de l'hôte (C2) | PARTIEL : STUN VALIDÉ (adresse publique et port conservé) ; ouverture UPnP **non exercée** (test local, UPnP désactivé ; seule la présence d'une box UPnP a été constatée en lecture) | `reseau-quic-hote.log` |
| C8 | traversée réelle d'une box / d'Internet (deux réseaux distincts), perçage de NAT réel | — | NON TESTÉ : deux processus sur une même machine ne le prouvent pas ; à faire avec un second réseau | — |
| D1 | IA : configuration générée (module → pont local seulement), pont démarré, jeton exigé | `Tests/Test-AI.ps1` | BLOQUÉ : LM Studio arrêté après le redémarrage du PC (à relancer par l'utilisateur) | — |
| D2 | IA : un joueur chuchote, le bot répond par le modèle, intention filtrée | idem | BLOQUÉ (idem) ; nettoyage et intentions VALIDÉS en unitaire ; seul un appel **direct** à LM Studio (`curl`, `gpt-oss-20b`, réponse avec la balise `[hc:follow]`) a été fait le matin : le pont lui-même n'a encore appelé aucun vrai fournisseur | — |
| D3 | IA : repli vers les réponses classiques si le fournisseur est injoignable | idem | BLOQUÉ (idem) | — |
| D4 | IA : clé d'API jamais en clair | idem | BLOQUÉ (idem) ; DPAPI VALIDÉ en unitaire | — |
| E1 | profil Benilla : client lancé par le profil, authentifié sans saisie, entré en jeu avec le personnage préféré | `Tests/Test-Profiles.ps1 -Client <copie Benilla>` | VALIDÉ (preuve côté serveur : `account_logons`, `Char.log`) | `docs/preuves/profils-benilla.txt` |
| E2 | profil client d'origine : compte dans Config.wtf, mot de passe saisi dans la fenêtre, authentifié sans saisie humaine | `Test-Profiles.ps1 -Client <copie d'origine>` | VALIDÉ | `docs/preuves/profils-client-origine.txt` |
| E3 | mot de passe introuvable en clair (journaux, UserData, configuration du client) | E1/E2 étape 3 | VALIDÉ (66 fichiers) | idem |
| E4 | un seul client par profil | code (`orchestrator::play_with`) | NON TESTÉ (vérifié par lecture du code) | — |
| E5 | plusieurs profils ouverts en même temps (multiboxing par profils) | — | NON TESTÉ avec de vrais clients (le multiboxing serveur est VALIDÉ en A) | — |
| F1 | interface en anglais | capture | VALIDÉ (capture de l'onglet PlayerBots en anglais) | `docs/preuves/interface-anglaise-playerbots.png` |
| F2 | bouton « Supprimer » d'un compte actif serveur démarré, refus si un personnage est en jeu | code | NON TESTÉ en interface (vérifié par lecture du code ; la suppression hors ligne est couverte par Test-Integration) | — |
| G1 | non-régression du launcher (premier démarrage, comptes, bots, réglages à chaud, fermeture forcée, arrêt) sur le package 1.0.0 | `Tests/Test-Integration.ps1` | VALIDÉ (33/33) | `docs/preuves/integration-package-1.0.0.txt` |

## Limites connues (dites, non masquées)

- **Traversée de box non prouvée** : la liaison (TCP et QUIC) est validée entre deux launchers sur une seule machine ; le
  perçage de NAT réel et l'UPnP réel restent à vérifier entre deux réseaux. Le perçage échoue par principe derrière un NAT
  symétrique (CGNAT, certaines box 4G/5G).
- **Tous les invités apparaissent en 127.0.0.1** au serveur : les règles par adresse IP (bannissement) ne distinguent pas les
  joueurs ; utiliser les comptes.
- **Client d'origine** : le mot de passe est saisi par messages clavier adressés à la fenêtre du jeu (dernier recours, le
  client 1.12.1 n'ayant aucun moyen officiel) ; impossible via VanillaFixes ; pas de présélection du personnage.
- **IA** : la qualité et la durée des réponses dépendent du modèle ; le premier appel à un modèle local peut prendre une minute.
- **Licence playerbots** : le module ne publie pas de licence ; l'utilisateur a choisi de publier la branche complète.

## Sécurité des secrets (vérifiée)

- Mots de passe des profils et clé d'API : DPAPI (session Windows), jamais écrits en clair (tests E3, unitaires).
- Codes d'invitation : clé propre au joueur, temporaire, révocable ; secret de session jamais persistant.
- Dépôt : contrôle avant commit (aucune clé, aucun mot de passe, aucun certificat privé ; adresse IP publique masquée dans les
  preuves et remplacée par une adresse de documentation dans les tests).

## Après la 1.0.0

| # | Fonction | Commande | Statut | Preuve |
|---|---|---|---|---|
| H1 | Mode Low Computer : mangosd.conf allégé, PlayerBots coupés, base réduite ; serveur démarré sans bot | script de vérification (installation de test, serveur réel) | VALIDÉ : MapUpdateInterval 200, GridCleanUpDelay 60000, visibilité 60/100, AiPlayerbot.Enabled 0, innodb 128M ; mangosd 498 Mo et MariaDB 173 Mo après 2 min, 0 session de bot, démarrage en 36 s | `docs/preuves/low-computer-serveur.txt` |
| H2 | Low Computer : bouton à bascule sur l'accueil, onglet PlayerBots grisé avec infobulle, retour à l'accueil si l'onglet était ouvert | capture avec la souris sur l'onglet | VALIDÉ | `docs/preuves/low-computer-interface.png` |
| H3 | Low Computer sur un vrai PC de 4 Go (Celeron N4120) | — | NON TESTÉ : le PC concerné n'est pas disponible ici | — |
| H4 | Client d'origine : mot de passe saisi quand l'écran de connexion est prêt ; entrée en jeu et 61 s en jeu sans fermeture | `Tests/Test-Profiles.ps1` et reproduction d'entrée en jeu | VALIDÉ (le crash rapporté par un joueur n'a pas été reproduit ici : ses journaux montrent un manque de mémoire, voir H1) | — |
