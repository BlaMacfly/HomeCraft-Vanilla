<#
.SYNOPSIS
    Importe dans la base monde les traductions saisies dans les fichiers TSV de Localization/frFR/ :
      quetes-sans-traduction.tsv        (entry, niveau, titre_anglais, titre_francais_a_saisir)         -> locales_quest.Title_loc2
      dialogues-pnj-sans-traduction.tsv (broadcast_text_entry, texte_anglais, texte_francais_a_saisir) -> locales_broadcast_text.male_text_loc2 / female_text_loc2
      creatures-sans-traduction.tsv     (entry, nom_anglais, sous_titre, npc_flags, nom_francais_a_saisir) -> locales_creature.name_loc2
    Seules les lignes dont la dernière colonne est renseignée sont importées. Les lignes existantes sont mises à jour,
    les autres créées. Redémarrer le serveur ensuite (les textes sont chargés au démarrage).
#>
[CmdletBinding()]
param([switch]$WhatIf)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"
if (-not (Test-MariaDbAlive)) { & "$PSScriptRoot\Start-MariaDB.ps1" | Out-Null }
$dir = Get-ProjectPath 'Localization\frFR'
function Esc([string]$s) { return $s.Replace('\', '\\').Replace("'", "\'") }
function Rows([string]$file) {
    $p = Join-Path $dir $file
    if (-not (Test-Path $p)) { return @() }
    return @(Get-Content $p -Encoding UTF8 | Select-Object -Skip 1 | ForEach-Object { $_ -split "`t" } | Where-Object { $_.Count -ge 2 })
}
$done = 0
foreach ($r in Rows 'quetes-sans-traduction.tsv') {
    $fr = $r[-1].Trim(); if (-not $fr) { continue }
    $sql = "INSERT INTO locales_quest (entry, Title_loc2) VALUES ($($r[0]), '$(Esc $fr)') ON DUPLICATE KEY UPDATE Title_loc2=VALUES(Title_loc2)"
    if ($WhatIf) { Write-Step $sql } else { Invoke-Sql -Database mangos -Query $sql }; $done++
}
foreach ($r in Rows 'dialogues-pnj-sans-traduction.tsv') {
    $fr = $r[-1].Trim(); if (-not $fr) { continue }
    $sql = "INSERT INTO locales_broadcast_text (entry, male_text_loc2, female_text_loc2) VALUES ($($r[0]), '$(Esc $fr)', '$(Esc $fr)') ON DUPLICATE KEY UPDATE male_text_loc2=VALUES(male_text_loc2), female_text_loc2=VALUES(female_text_loc2)"
    if ($WhatIf) { Write-Step $sql } else { Invoke-Sql -Database mangos -Query $sql }; $done++
}
foreach ($r in Rows 'creatures-sans-traduction.tsv') {
    $fr = $r[-1].Trim(); if (-not $fr) { continue }
    $sql = "INSERT INTO locales_creature (entry, name_loc2) VALUES ($($r[0]), '$(Esc $fr)') ON DUPLICATE KEY UPDATE name_loc2=VALUES(name_loc2)"
    if ($WhatIf) { Write-Step $sql } else { Invoke-Sql -Database mangos -Query $sql }; $done++
}
Write-Step "$done traduction(s) $(if ($WhatIf) {'à importer (simulation)'} else {'importée(s)'}). Redémarrer le serveur pour les charger."
