<#
.SYNOPSIS
    Génère Server/Database/custom/auctionhousebot_solo.sql : la liste des objets que l'AHBot VMaNGOS met en vente,
    à partir de item_template (objets utiles en solo : consommables, composants, recettes, réactifs, familiers,
    équipement vert/bleu/épique non lié au ramassage). Avec -Apply, remplace le contenu de mangos.auctionhousebot.

.NOTES
    Fonctionnement de l'AHBot VMaNGOS (AuctionHouseBotMgr.cpp, commit 53d24d5) : tirage uniforme avec remise dans
    la table jusqu'à AHBot.itemcount enchères ; prix bid/buyout pris tels quels ; durée 12/24/48 h ; vendeur = 0.
    Pour qu'« à peu près tout » soit en vente, AHBot.itemcount est réglé à 3 × (nombre de lignes) par New-ServerConfig.ps1.
#>
[CmdletBinding()]
param(
    [int]$WowPatch = 10,
    [switch]$Apply
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$outDir = Get-ProjectPath 'Server\Database\custom'
New-Item -ItemType Directory -Force $outDir | Out-Null
$outFile = Join-Path $outDir 'auctionhousebot_solo.sql'

# Dernière version de chaque objet pour le patch de contenu courant, puis filtre « utile en solo ».
$select = @"
SELECT i.entry,
       CASE WHEN i.stackable > 1 THEN LEAST(i.stackable, 20) ELSE 1 END AS stack,
       GREATEST(50, CASE WHEN i.buy_price > 0 THEN i.buy_price ELSE i.sell_price * 4 END) AS unit_price
FROM mangos.item_template i
JOIN (SELECT entry, MAX(patch) AS patch FROM mangos.item_template WHERE patch <= $WowPatch GROUP BY entry) last
  ON last.entry = i.entry AND last.patch = i.patch
WHERE i.bonding IN (0, 2, 3)
  AND (i.buy_price > 0 OR i.sell_price > 0)
  AND i.name NOT LIKE 'Deprecated%' AND i.name NOT LIKE 'Monster -%' AND i.name NOT LIKE '%(test)%'
  AND i.name NOT LIKE 'OLD%' AND i.name NOT LIKE '%[PH]%' AND i.name NOT LIKE 'zz%' AND i.name NOT LIKE '%DEBUG%'
  AND (
        (i.class IN (0, 5, 7, 9) AND i.quality BETWEEN 1 AND 4)              -- consommables, réactifs, composants, recettes
     OR (i.class = 15 AND i.subclass IN (1, 2, 5) AND i.quality BETWEEN 1 AND 4) -- réactifs divers, familiers, montures
     OR (i.class IN (2, 4) AND i.quality BETWEEN 2 AND 4)                    -- armes et armures vertes, bleues, épiques
     OR (i.class IN (1, 11) AND i.quality BETWEEN 1 AND 4)                   -- sacs, carquois/giberne
     OR (i.class = 6 AND i.quality BETWEEN 1 AND 3)                          -- munitions
  )
ORDER BY i.entry
"@

Write-Step "Sélection des objets (patch <= $WowPatch)"
$rows = @(Invoke-Sql -Query $select)
if ($rows.Count -eq 0) { throw "Aucun objet sélectionné" }

$sb = New-Object System.Text.StringBuilder
[void]$sb.AppendLine("-- Généré par Tools/New-AHBotTable.ps1 le $(Get-Date -Format 'yyyy-MM-dd HH:mm') — $($rows.Count) objets, patch <= $WowPatch")
[void]$sb.AppendLine("DELETE FROM auctionhousebot;")
[void]$sb.AppendLine("INSERT INTO auctionhousebot (item, stack, bid, buyout) VALUES")
$values = foreach ($r in $rows) {
    $f = $r -split "`t"
    $entry = [int]$f[0]; $stack = [int]$f[1]; $unit = [int64]$f[2]
    $buyout = $unit * $stack
    $bid = [int64][math]::Max(1, [math]::Floor($buyout * 0.75))
    "($entry,$stack,$bid,$buyout)"
}
[void]$sb.AppendLine(($values -join ",`n") + ";")
[System.IO.File]::WriteAllText($outFile, $sb.ToString(), (New-Object System.Text.UTF8Encoding $false))
Write-Step "$($rows.Count) objets écrits dans $outFile"

if ($Apply) {
    Invoke-Sql -Database mangos -File $outFile
    $n = Invoke-Sql -Query "SELECT COUNT(*) FROM mangos.auctionhousebot"
    Write-Step "Table mangos.auctionhousebot remplacée : $n lignes"
}
