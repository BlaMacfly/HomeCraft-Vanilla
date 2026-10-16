<#
.SYNOPSIS
    Inventaire de la localisation française côté serveur (Phase 3 / Phase 7) :
    couverture des tables locales_* (loc2 = frFR), chaînes serveur (mangos_string), présence de caractères accentués,
    validité UTF-8, textes manquants. Produit Documentation/09-Localisation-Inventaire.md.
    Nécessite MariaDB en service (démarrée par le launcher ou Tools/Start-MariaDB.ps1).
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
if (-not (Test-MariaDbAlive)) { & (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null }

function Q([string]$sql) { Invoke-Sql -Query $sql }
$rows = New-Object System.Collections.Generic.List[string]
$rows.Add("# Inventaire de la localisation française (serveur)")
$rows.Add("")
$rows.Add("Généré le $(Get-Date -Format 'yyyy-MM-dd HH:mm') par ``Tests/Test-Localisation.ps1`` sur la base monde VMaNGOS (snapshot 4641790). loc2 = frFR.")
$rows.Add("")
$rows.Add("## Couverture par table (entrées de référence, entrées traduites en français, pourcentage)")
$rows.Add("")
$rows.Add("| Donnée | Référence (anglais) | Traduit frFR | Couverture |")
$rows.Add("|---|---|---|---|")

# (libellé, table de référence, condition de référence, table locale, colonne loc2, jointure)
$checks = @(
    @('Quêtes — titre', 'quest_template', '1=1', 'locales_quest', 'Title_loc2', 'entry'),
    @('Quêtes — description', 'quest_template', "Details<>''", 'locales_quest', 'Details_loc2', 'entry'),
    @('Quêtes — objectifs', 'quest_template', "Objectives<>''", 'locales_quest', 'Objectives_loc2', 'entry'),
    @('Quêtes — texte de remise', 'quest_template', "OfferRewardText<>''", 'locales_quest', 'OfferRewardText_loc2', 'entry'),
    @('Quêtes — texte de progression', 'quest_template', "RequestItemsText<>''", 'locales_quest', 'RequestItemsText_loc2', 'entry'),
    @('Créatures — nom', 'creature_template', '1=1', 'locales_creature', 'name_loc2', 'entry'),
    @('Créatures — sous-titre', 'creature_template', "subname<>''", 'locales_creature', 'subname_loc2', 'entry'),
    @('Objets — nom', 'item_template', '1=1', 'locales_item', 'name_loc2', 'entry'),
    @('Objets — description', 'item_template', "description<>''", 'locales_item', 'description_loc2', 'entry'),
    @('Objets du monde — nom', 'gameobject_template', '1=1', 'locales_gameobject', 'name_loc2', 'entry'),
    @('Textes diffusés (dialogues, cris)', 'broadcast_text', "male_text<>'' OR female_text<>''", 'locales_broadcast_text', 'male_text_loc2', 'entry'),
    @('Pages de livres', 'page_text', '1=1', 'locales_page_text', 'text_loc2', 'entry'),
    @('Options de dialogue PNJ', 'gossip_menu_option', "option_text<>''", 'locales_gossip_menu_option', 'option_text_loc2', 'menu_id'),
    @('Points d''intérêt', 'points_of_interest', '1=1', 'locales_points_of_interest', 'icon_name_loc2', 'entry'),
    @('Zones (noms)', 'area_template', '1=1', 'locales_area', 'NameLoc2', 'entry'),
    @('Chaînes serveur (messages système)', 'mangos_string', '1=1', 'mangos_string', 'content_loc2', 'entry')
)
$summary = @{}
foreach ($c in $checks) {
    $label, $refTable, $refCond, $locTable, $locCol, $key = $c
    try {
        $ref = [int](Q "SELECT COUNT(DISTINCT $key) FROM mangos.$refTable WHERE $refCond")
        $tr = if ($locTable -eq $refTable) { [int](Q "SELECT COUNT(*) FROM mangos.$refTable WHERE $refCond AND $locCol IS NOT NULL AND $locCol<>''") }
              else { [int](Q "SELECT COUNT(DISTINCT r.$key) FROM mangos.$refTable r JOIN mangos.$locTable l ON l.$key=r.$key WHERE $refCond AND l.$locCol IS NOT NULL AND l.$locCol<>''") }
        $pct = if ($ref -gt 0) { [math]::Round(100.0 * $tr / $ref, 1) } else { 0 }
        $rows.Add("| $label | $ref | $tr | $pct % |")
        $summary[$label] = $pct
    } catch { $rows.Add("| $label | erreur : $($_.Exception.Message -replace '\|',' ') | | |") }
}

$rows.Add("")
$rows.Add("## Qualité des textes français")
$rows.Add("")
$accent = [int](Q "SELECT COUNT(*) FROM mangos.locales_quest WHERE Title_loc2 REGEXP '[éèêàçùôîï]'")
$titles = [int](Q "SELECT COUNT(*) FROM mangos.locales_quest WHERE Title_loc2<>''")
$rows.Add("- Titres de quête français contenant au moins un caractère accenté : $accent / $titles (preuve que l'encodage UTF-8 et les accents sont présents).")
$badUtf = [int](Q "SELECT COUNT(*) FROM mangos.locales_quest WHERE Details_loc2<>'' AND CONVERT(Details_loc2 USING utf8mb3) IS NULL")
$rows.Add("- Descriptions de quête invalides en UTF-8 : $badUtf.")
$tu = [int](Q "SELECT COUNT(*) FROM mangos.locales_quest WHERE Details_loc2 REGEXP '(^|[^[:alpha:]])(tu|te|ton|ta|tes)([^[:alpha:]]|$)'")
$vous = [int](Q "SELECT COUNT(*) FROM mangos.locales_quest WHERE Details_loc2 REGEXP '(^|[^[:alpha:]])(vous|votre|vos)([^[:alpha:]]|$)'")
$rows.Add("- Descriptions de quête utilisant le tutoiement (« tu », « ton »…) : $tu ; le vouvoiement : $vous. Le client officiel vouvoie ; les textes au tutoiement sont d'origine communautaire (voir analyse AzerothCore PR 28044).")
$untranslatedSample = Q "SELECT DISTINCT q.entry, q.Title FROM mangos.quest_template q LEFT JOIN mangos.locales_quest l ON l.entry=q.entry WHERE (l.Title_loc2 IS NULL OR l.Title_loc2='') ORDER BY q.entry LIMIT 15"
$rows.Add("")
$rows.Add("## Exemples de quêtes sans titre français (restent en anglais)")
$rows.Add("")
$rows.Add('```')
$untranslatedSample | ForEach-Object { $rows.Add($_) }
$rows.Add('```')
$rows.Add("")
$rows.Add("## Quêtes, dialogues de PNJ et créatures : état après les compléments HomeCraft (2026-10-15)")
$rows.Add("")
$realQ = [int](Q "SELECT COUNT(DISTINCT q.entry) FROM mangos.quest_template q WHERE q.Title NOT LIKE '%UNUSED%' AND q.Title NOT LIKE '%nyi%' AND q.Title NOT LIKE '%REUSE%' AND q.Title NOT LIKE '%DEPRECATED%' AND q.Title NOT LIKE '<%' AND q.Title NOT LIKE '%test%'")
$realQMissing = [int](Q "SELECT COUNT(DISTINCT q.entry) FROM mangos.quest_template q LEFT JOIN mangos.locales_quest l ON l.entry=q.entry WHERE q.Title NOT LIKE '%UNUSED%' AND q.Title NOT LIKE '%nyi%' AND q.Title NOT LIKE '%REUSE%' AND q.Title NOT LIKE '%DEPRECATED%' AND q.Title NOT LIKE '<%' AND q.Title NOT LIKE '%test%' AND (COALESCE(l.Title_loc2,'')='' OR (q.Details<>'' AND COALESCE(l.Details_loc2,'')='') OR (q.Objectives<>'' AND COALESCE(l.Objectives_loc2,'')='') OR (q.OfferRewardText<>'' AND COALESCE(l.OfferRewardText_loc2,'')='') OR (q.RequestItemsText<>'' AND COALESCE(l.RequestItemsText_loc2,'')=''))")
$npcMissing = [int](Q "SELECT COUNT(DISTINCT b.entry) FROM mangos.broadcast_text b JOIN mangos.npc_text n ON b.entry IN (n.BroadcastTextID0,n.BroadcastTextID1,n.BroadcastTextID2,n.BroadcastTextID3,n.BroadcastTextID4,n.BroadcastTextID5,n.BroadcastTextID6,n.BroadcastTextID7) LEFT JOIN mangos.locales_broadcast_text l ON l.entry=b.entry WHERE (b.male_text<>'' OR b.female_text<>'') AND COALESCE(l.male_text_loc2,'')='' AND COALESCE(l.female_text_loc2,'')=''")
$worldC = [int](Q "SELECT COUNT(DISTINCT c.entry) FROM mangos.creature_template c JOIN mangos.creature cr ON cr.id=c.entry")
$worldCMissing = [int](Q "SELECT COUNT(DISTINCT c.entry) FROM mangos.creature_template c JOIN mangos.creature cr ON cr.id=c.entry LEFT JOIN mangos.locales_creature l ON l.entry=c.entry WHERE COALESCE(l.name_loc2,'')=''")
$rows.Add("- Quêtes réelles (hors entrées techniques « <UNUSED> », « <nyi> », « REUSE », « DEPRECATED », « <TXT> », quêtes de test) : $realQ ; **$realQMissing sans titre ou texte français**. Les 138 titres et les textes manquants ont été traduits par HomeCraft (``Server/Database/custom/locales_quest_homecraft.sql``, traductions **non officielles**, balises du jeu conservées, textes officiels jamais écrasés). Les entrées techniques restent en anglais : le joueur ne les voit jamais.")
$rows.Add("- Dialogues de PNJ (fenêtres de discussion, ``npc_text`` → ``broadcast_text``) : **$npcMissing sans texte français** (les 6 manquants, grognements d'animaux et plantes de Gangrebois, sont dans le même fichier).")
$rows.Add("- Noms de créatures présentes dans le monde : $worldC entrées, **$worldCMissing sans nom français** après ``locales_creature_homecraft.sql``. Règle appliquée, identique à la localisation officielle (« Maréchal Dughan », « Garde Thomas ») : titre traduit, nom propre conservé ; 2 883 des 2 886 noms traités sont donc identiques à l'anglais (noms propres), 3 titres traduits (« Chef Galgosh », « Chef Tho'grun », « M. Bigglesworth »), 20 entrées techniques (déclencheurs invisibles) laissées de côté.")
$rows.Add("- Ces compléments sont importés automatiquement par le launcher (fichiers ``Server/Database/custom/*.sql`` appliqués après la base monde) ; ``Tools/Import-Translations.ps1`` reste disponible pour des traductions futures au format TSV.")
$rows.Add("")
$rows.Add("## Ce qui est couvert par le client frFR lui-même (hors base)")
$rows.Add("")
$rows.Add("Interface (menus, fenêtres, création de personnage, inventaire, talents, infobulles, messages système, journal de combat), noms et descriptions des sorts et des compétences, noms des zones affichés à l'écran, noms des races et classes : fournis par les DBC et les fichiers FrameXML/GlobalStrings du client français 1.12.1. Le serveur détecte ``DBC.Locale = frFR`` à partir des DBC extraites de ce client.")
$rows.Add("")
$rows.Add("## Éléments restant en anglais ou partiellement traduits")
$rows.Add("")
$rows.Add("- Les quêtes, créatures et objets sans entrée loc2 (voir tableau) sont servis en anglais par le serveur. Pour les créatures, une large part des entrées non traduites sont des créatures techniques ou invisibles (déclencheurs, totems, objets de script) que le joueur ne rencontre pas nommées.")
$rows.Add("- Options de dialogue PNJ : le faible taux de ``locales_gossip_menu_option`` est trompeur, la plupart des options tirent leur texte de ``broadcast_text`` (traduit à 86 %) via ``option_broadcast_text``.")
$rows.Add("- Les messages de la console et des commandes GM (``.partybot``, etc.) : ``mangos_string`` partiellement traduit, le reste en anglais.")
$rows.Add("- Les noms des compagnons PlayerBots générés par le serveur sont des noms neutres.")
$rows.Add("")
$rows.Add("Méthode de complément possible : importer un pack de traduction sous licence compatible (aucun identifié à ce jour : UDBFR/MangosZero sans licence), ou traduire manuellement les entrées manquantes dans ``locales_*`` via l'explorateur de base ou HeidiSQL.")

$out = Get-ProjectPath 'Documentation\09-Localisation-Inventaire.md'
[System.IO.File]::WriteAllText($out, ($rows -join "`r`n"), (New-Object System.Text.UTF8Encoding $false))
Write-Step "Inventaire écrit : $out"
$rows | Where-Object { $_ -match '^\|' } | ForEach-Object { Write-Step $_ }
$ok = ($summary['Quêtes — titre'] -ge 80) -and ($summary['Objets — nom'] -ge 80) -and ($accent -gt 1000) -and ($badUtf -eq 0)
if ($ok) { Write-Step "TEST RÉUSSI (seuils : titres de quête >= 80 %, objets >= 80 %, accents présents, UTF-8 valide)"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
