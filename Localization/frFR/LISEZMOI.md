# Localisation française — compléments HomeCraft

Les listes `*-sans-traduction.tsv` exportées le 2026-10-15 ont été traduites le jour même et intégrées
sous forme de SQL dans `Server/Database/custom/` (appliqué automatiquement par le launcher après l'import
de la base monde, et rejouable sans risque : seuls les champs vides sont remplis) :

| Fichier | Contenu | Nature |
|---|---|---|
| `locales_quest_homecraft.sql` | 138 titres de quête + textes (description, objectifs, remise, progression) manquants, 6 dialogues de PNJ (`locales_broadcast_text`) | **Traductions HomeCraft, non officielles** (balises `$N`, `$B`, `$C`, `$R`, `$G…;` conservées) |
| `locales_creature_homecraft.sql` | 2 887 noms de créatures | Règle de la localisation officielle : titre traduit, nom propre conservé (3 titres traduits, le reste identique à l'anglais) |

Les entrées techniques (quêtes `<UNUSED>`, `<nyi>`, `REUSE`, `DEPRECATED`, `<TXT>`, quêtes de test ; créatures
« Trigger », « Doodad », « Debug Target »…) restent volontairement en anglais : le joueur ne les voit jamais.

Pour de futures traductions au format TSV (dernière colonne = texte français), `Tools/Import-Translations.ps1`
reste disponible. État courant mesuré par `Tests/Test-Localisation.ps1` → `Documentation/09-Localisation-Inventaire.md`.
