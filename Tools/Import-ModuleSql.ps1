<#
.SYNOPSIS
    Importe les fichiers SQL des modules embarqués que l'updater d'AzerothCore n'applique pas tout seul
    (fichiers placés directement dans data\sql\db-world|db-characters|db-auth sans sous-dossier base/ ou updates/,
    disposition héritée de l'ancien format de modules). Idempotent : un fichier déjà appliqué (nom présent dans la
    table `updates` de la base cible, ou dans UserData\module-sql-applied.json) est ignoré.
    Les dumps du royaume contiennent normalement déjà ces tables : ce script sert surtout à l'amorçage depuis le dépôt.
.PARAMETER SourceDir  Racine du dépôt du cœur (défaut : C:\hc).
.PARAMETER WhatIf     Liste seulement.
#>
[CmdletBinding()]
param(
    [string]$SourceDir = "C:\hc",
    [switch]$DryRun
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$dbOf = @{ 'db-world' = 'acore_world'; 'world' = 'acore_world'; 'db-characters' = 'acore_characters'; 'characters' = 'acore_characters'; 'db-auth' = 'acore_auth'; 'auth' = 'acore_auth' }
$skipDirs = '\\delete\\|\(Optional\)|mod-ah-bot-master|sylvania_market|\\playerbots\\|\\create\\'   # playerbots : géré par le cœur patché ; optionnels/doublons : à la main
$ledgerPath = Get-ProjectPath 'UserData\module-sql-applied.json'
$ledger = @{}
if (Test-Path $ledgerPath) { (Get-Content $ledgerPath -Raw -Encoding UTF8 | ConvertFrom-Json).PSObject.Properties | ForEach-Object { $ledger[$_.Name] = $_.Value } }

$files = Get-ChildItem (Join-Path $SourceDir 'modules') -Recurse -Filter *.sql -File | Where-Object { $_.FullName -match '\\data\\sql\\' -and $_.FullName -notmatch $skipDirs } | Sort-Object FullName
$applied = 0; $skipped = 0; $failed = @()
foreach ($f in $files) {
    $rel = $f.FullName.Substring((Join-Path $SourceDir 'modules').Length + 1)
    $parts = $rel -split '\\'
    $kind = ($parts | Where-Object { $dbOf.ContainsKey($_) } | Select-Object -First 1)
    if (-not $kind) { Write-Step "ignoré (base inconnue) : $rel"; $skipped++; continue }
    $db = $dbOf[$kind]
    if ($ledger.ContainsKey($rel)) { $skipped++; continue }
    $known = Invoke-Sql -Database $db -Query "SELECT COUNT(*) FROM ``$db``.updates WHERE name='$($f.Name)'"
    if ([int]$known -gt 0) { $ledger[$rel] = 'updater'; $skipped++; continue }
    if ($DryRun) { Write-Step "à importer dans $db : $rel"; continue }
    Write-Step "import $db <- $rel"
    try {
        Invoke-Sql -AsRoot -Database $db -File $f.FullName
        $ledger[$rel] = (Get-Date -Format s); $applied++
    } catch {
        Write-Step "   ÉCHEC : $($_.Exception.Message)"; $failed += $rel
    }
}
if (-not $DryRun) { $ledger | ConvertTo-Json | Set-Content -Path $ledgerPath -Encoding UTF8 }
Write-Step "Terminé : $applied importé(s), $skipped ignoré(s), $($failed.Count) en échec."
$failed | ForEach-Object { Write-Step "   échec : $_" }
if ($failed.Count -gt 0) { exit 1 }
