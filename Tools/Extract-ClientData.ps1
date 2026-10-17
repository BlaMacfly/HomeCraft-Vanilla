<#
.SYNOPSIS
    Extrait dbc / maps / Cameras / vmaps (et mmaps en option) depuis un client 3.3.5a (12340) avec les extracteurs
    AzerothCore compilés (Server/acore), vers Server/Data. Ces données dérivent des MPQ Blizzard : jamais redistribuées.

.PARAMETER ClientDir   Dossier du client (contient Wow.exe et Data/).
.PARAMETER Steps       Étapes : dbcmaps, vmaps, mmaps (défaut : dbcmaps,vmaps).
.PARAMETER Threads     Threads pour mmaps_generator.
#>
[CmdletBinding()]
param(
    [string]$ClientDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'Client'),
    [string[]]$Steps = @('dbcmaps', 'vmaps'),
    [int]$Threads = [Math]::Max(1, [Environment]::ProcessorCount - 2)
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"
# Appelé avec « powershell -File … -Steps dbcmaps,vmaps », la liste arrive en une seule chaîne : on la découpe.
$Steps = @($Steps | ForEach-Object { $_ -split '[,; ]+' } | Where-Object { $_ })

$bin  = $Layout.ServerBin
$data = $Layout.ServerData
$log  = Join-Path $Layout.Logs 'extract'
New-Item -ItemType Directory -Force $data, $log | Out-Null

$exe = Join-Path $ClientDir 'Wow.exe'
if (-not (Test-Path $exe)) { throw "Wow.exe introuvable dans $ClientDir" }
$ver = (Get-Item $exe).VersionInfo.FileVersion -replace '[ ,]+', '.'
if ($ver -ne $Layout.ClientBuild) { throw "Client inattendu : version $ver (attendu $($Layout.ClientBuild))" }
$locales = Get-ChildItem (Join-Path $ClientDir 'Data') -Directory | Where-Object { $_.Name -match '^[a-z]{2}[A-Z]{2}$' } | ForEach-Object { $_.Name }
Write-Step "Client validé : $exe ($ver), locales : $($locales -join ', ')"

function Invoke-Extractor {
    param([string]$Name, [string]$Exe, [string[]]$Arguments, [string]$WorkDir)
    Write-Step "$Name : $Exe $($Arguments -join ' ')"
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $Exe -ArgumentList $Arguments -WorkingDirectory $WorkDir -NoNewWindow -Wait -PassThru `
        -RedirectStandardOutput "$log\$Name.stdout.log" -RedirectStandardError "$log\$Name.stderr.log"
    Write-Step "$Name terminé en $([int]$sw.Elapsed.TotalMinutes) min, code $($p.ExitCode)"
    if ($p.ExitCode -ne 0) { throw "$Name a échoué (voir $log\$Name.stderr.log)" }
}

if ($Steps -contains 'dbcmaps') {
    # map_extractor : -i dossier du client, -o dossier de sortie (dbc/, maps/, Cameras/), -e 7 = dbc + maps + caméras.
    Invoke-Extractor 'map_extractor' (Join-Path $bin 'map_extractor.exe') @('-i', "`"$ClientDir`"", '-o', "`"$data`"", '-e', '7') $data
}
if ($Steps -contains 'vmaps') {
    # vmap4_extractor écrit « Buildings » dans le dossier courant ; vmap4_assembler lit « Buildings » et écrit « vmaps ».
    Invoke-Extractor 'vmap4_extractor' (Join-Path $bin 'vmap4_extractor.exe') @('-l', '-d', "`"$(Join-Path $ClientDir 'Data')`"") $data
    New-Item -ItemType Directory -Force (Join-Path $data 'vmaps') | Out-Null
    Invoke-Extractor 'vmap4_assembler' (Join-Path $bin 'vmap4_assembler.exe') @('Buildings', 'vmaps') $data
}
if ($Steps -contains 'mmaps') {
    # mmaps_generator lit maps/ et vmaps/ dans le dossier courant et exige mmaps-config.yaml à côté de l'exécutable.
    New-Item -ItemType Directory -Force (Join-Path $data 'mmaps') | Out-Null
    Invoke-Extractor 'mmaps_generator' (Join-Path $bin 'mmaps_generator.exe') @('--threads', $Threads, '--silent') $data
}

Write-Step "Contenu de $data :"
Get-ChildItem $data -Directory | ForEach-Object {
    $n = (Get-ChildItem $_.FullName -Recurse -File | Measure-Object Length -Sum)
    Write-Step ("   {0,-12} {1,7} fichiers {2,8:N0} Mo" -f $_.Name, $n.Count, ($n.Sum / 1MB))
}
