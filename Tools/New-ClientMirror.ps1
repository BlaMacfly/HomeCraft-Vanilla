<#
.SYNOPSIS
    Crée (ou complète) le dossier Client/ du projet à partir d'un client 3.3.5a existant, sans dupliquer les données :
    les archives MPQ de Data/ sont des LIENS PHYSIQUES (même volume obligatoire, aucun espace disque consommé),
    l'exécutable, ses DLL, realmlist.wtf, WTF/ et Interface/ sont COPIÉS.

    But : le launcher réécrit Data/<locale>/realmlist.wtf et WTF/Config.wtf (127.0.0.1). Travailler sur un miroir
    laisse intact le client source (par exemple celui qui sert à jouer sur les royaumes Sylvania en ligne).

    Clients acceptés : Wow.exe d'origine ou client décompilé wowdecomp (WowClient64.exe / Wow64.exe, x64).
    Client/ n'est jamais redistribué ni archivé (règle du projet).
.PARAMETER Source
    Dossier du client source (contenant Data/ et l'exécutable), par ex. le dossier bin\Win de wowdecomp.
.PARAMETER Force
    Recopie aussi l'exécutable, les DLL et WTF/ même s'ils existent déjà dans Client/ (les liens MPQ sont toujours
    recréés s'ils manquent ; un MPQ déjà présent n'est jamais écrasé).
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Source,
    [switch]$Force
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$src = (Resolve-Path $Source).Path
$dst = Get-ProjectPath 'Client'
$exeNames = @('Wow64.exe', 'WowClient64.exe', 'Wow.exe', 'Wow32.exe', 'WowClient32.exe')
$exes = @($exeNames | Where-Object { Test-Path (Join-Path $src $_) })
if ($exes.Count -eq 0) { throw "Aucun exécutable de client 3.3.5a dans $src ($($exeNames -join ', '))" }
if (-not (Test-Path (Join-Path $src 'Data\common.MPQ'))) { throw "$src\Data\common.MPQ introuvable : ce n'est pas un client 3.3.5a complet" }
if ([IO.Path]::GetPathRoot($src) -ne [IO.Path]::GetPathRoot($ProjectRoot)) { throw "Le client source doit être sur le même volume que le projet (liens physiques)" }

New-Item -ItemType Directory -Force $dst | Out-Null
Write-Step "Miroir de $src -> $dst"

# 1. Archives MPQ : liens physiques (Data/ et Data/<locale>/).
$links = 0; $kept = 0
Get-ChildItem (Join-Path $src 'Data') -Recurse -File | Where-Object { $_.Extension -ieq '.mpq' } | ForEach-Object {
    $rel = $_.FullName.Substring($src.Length + 1)
    $target = Join-Path $dst $rel
    if (Test-Path $target) { $kept++; return }
    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
    New-Item -ItemType HardLink -Path $target -Target $_.FullName | Out-Null
    $links++
}
Write-Step "MPQ : $links lien(s) physique(s) créé(s), $kept déjà présent(s)"

# 2. Petits fichiers copiés : exécutables, DLL, realmlist.wtf (le launcher le réécrit ; l'original reste dans la source).
$copied = 0
$small = @(Get-ChildItem $src -File | Where-Object { $_.Extension -in '.exe', '.dll' }) + @(Get-ChildItem (Join-Path $src 'Data') -Recurse -File -Filter 'realmlist.wtf')
foreach ($f in $small) {
    $rel = $f.FullName.Substring($src.Length + 1)
    $target = Join-Path $dst $rel
    if ((Test-Path $target) -and -not $Force) { continue }
    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
    Copy-Item $f.FullName $target -Force
    $copied++
}
# 3. Réglages et AddOns du joueur (copiés une fois ; -Force pour recopier).
foreach ($d in 'WTF', 'Interface') {
    $from = Join-Path $src $d
    $to = Join-Path $dst $d
    if (-not (Test-Path $from)) { continue }
    if ((Test-Path $to) -and -not $Force) { Write-Step "$d/ déjà présent : conservé"; continue }
    Copy-Item $from $dst -Recurse -Force
    $copied++
}
Write-Step "fichiers copiés : $copied ; exécutable(s) : $($exes -join ', ')"
Write-Step "Client/ prêt. Dans le launcher : PARAMÈTRES > dossier du client vide (= Client/), type « Automatique » ou « 3.3.5a »."
