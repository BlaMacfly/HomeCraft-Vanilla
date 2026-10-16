<#
.SYNOPSIS
    Extrait dbc / maps / vmaps (et mmaps en option) depuis le client 1.12.1 fourni par l'utilisateur,
    avec les extracteurs VMaNGOS, vers Server/Data. Ces données dérivent des MPQ Blizzard : jamais redistribuées.

.PARAMETER ClientDir   Dossier du client (contient WoW.exe et Data/).
.PARAMETER Steps       Étapes à exécuter : dbcmaps, vmaps, mmaps (défaut : dbcmaps,vmaps).
.PARAMETER Threads     Threads pour MoveMapGenerator.
#>
[CmdletBinding()]
param(
    [string]$ClientDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'Client'),
    [string[]]$Steps = @('dbcmaps', 'vmaps'),
    [int]$Threads = [Environment]::ProcessorCount
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$bin  = Get-ProjectPath 'Server\vmangos'
$data = Get-ProjectPath 'Server\Data'
$log  = Join-Path $Layout.Logs 'extract'
New-Item -ItemType Directory -Force $data, $log | Out-Null

$exe = Join-Path $ClientDir 'WoW.exe'
if (-not (Test-Path $exe)) { throw "WoW.exe introuvable dans $ClientDir" }
$ver = (Get-Item $exe).VersionInfo.FileVersion -replace '[ ,]+', '.'
if ($ver -ne '1.12.1.5875') { throw "Client inattendu : version $ver (attendu 1.12.1.5875)" }
Write-Step "Client validé : $exe ($ver)"

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
    Invoke-Extractor 'MapExtractor' (Join-Path $bin 'MapExtractor.exe') @('-i', "`"$ClientDir`"", '-o', "`"$data`"", '--silent') $data
    # mangosd lit les DBC dans DataDir/<build>/dbc ; MapExtractor les écrit dans <sortie>/dbc.
    $dbcTarget = Join-Path $data '5875\dbc'
    if (Test-Path (Join-Path $data 'dbc')) {
        New-Item -ItemType Directory -Force (Join-Path $data '5875') | Out-Null
        if (Test-Path $dbcTarget) { Remove-Item -Recurse -Force $dbcTarget }
        Move-Item (Join-Path $data 'dbc') $dbcTarget
        Write-Step "DBC déplacées vers $dbcTarget"
    }
}
if ($Steps -contains 'vmaps') {
    # VMapExtractor écrit « Buildings » dans le dossier courant ; VMapAssembler lit « Buildings » et écrit « vmaps »
    # (il ne crée pas le dossier de sortie lui-même : « Cannot open vmaps/000.vmtree » sinon).
    Invoke-Extractor 'VMapExtractor' (Join-Path $bin 'VMapExtractor.exe') @('-l', '-d', "`"$(Join-Path $ClientDir 'Data')`"", '--silent') $data
}
if ($Steps -contains 'vmaps' -or $Steps -contains 'vmapsassemble') {
    New-Item -ItemType Directory -Force (Join-Path $data 'vmaps') | Out-Null
    Invoke-Extractor 'VMapAssembler' (Join-Path $bin 'VMapAssembler.exe') @('Buildings', 'vmaps') $data
}
if ($Steps -contains 'mmaps') {
    Invoke-Extractor 'MoveMapGenerator' (Join-Path $bin 'MoveMapGenerator.exe') @('--threads', $Threads, '--silent') $data
}

Write-Step "Contenu de $data :"
Get-ChildItem $data -Directory | ForEach-Object {
    $n = (Get-ChildItem $_.FullName -Recurse -File | Measure-Object Length -Sum)
    Write-Step ("   {0,-12} {1,7} fichiers {2,8:N0} Mo" -f $_.Name, $n.Count, ($n.Sum / 1MB))
}
