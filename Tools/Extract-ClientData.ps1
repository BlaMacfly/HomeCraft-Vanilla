<#
.SYNOPSIS
    Extrait dbc / maps / vmaps (et mmaps en option) depuis le client 2.4.3 fourni par l'utilisateur, avec les
    extracteurs CMaNGOS (Server/mangos-tbc/Extractors), vers Server/Data. Transposition Windows de ExtractResources.sh
    (mode « a » : basse résolution, comme le choix par défaut du script officiel). Ces données dérivent des MPQ
    Blizzard : jamais redistribuées.

    Archives officielles seulement : ad.exe et vmap_extractor chargent TOUTES les archives patch-*.MPQ et
    patch-<locale>-*.MPQ présentes. Un patch tiers (ex. patch-frFR-3.MPQ d'un serveur privé, qui modifie Spell.dbc)
    passerait dans les données du serveur et ne correspondrait plus à TBC-DB. Les extracteurs travaillent donc sur
    une « vue » du client : UserData\extract-view (ou <lecteur du client>:\HCBC-extract si le client est sur un autre
    volume), liens physiques vers les seules archives officielles 2.4.3 et WoW.exe. Le client n'est jamais modifié.
    Chemin court : avec un chemin de ~150 caractères, ad.exe ne trouve plus la locale (« No locales detected »).

.PARAMETER ClientDir   Dossier du client (contient WoW.exe et Data/).
.PARAMETER Steps       Étapes : dbcmaps, vmaps, mmaps (défaut : dbcmaps,vmaps ; mmaps est long, le launcher le fait en arrière-plan).
.PARAMETER Threads     Threads pour MoveMapGen (défaut : moitié des cœurs).
#>
[CmdletBinding()]
param(
    [string]$ClientDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'Client'),
    [string[]]$Steps = @('dbcmaps', 'vmaps'),
    [int]$Threads = [Math]::Max(1, [int]([Environment]::ProcessorCount / 2))
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$ext  = $Layout.Extractors
$data = $Layout.ServerData
$log  = Join-Path $Layout.Logs 'extract'
New-Item -ItemType Directory -Force $data, $log | Out-Null

$exe = Join-Path $ClientDir 'WoW.exe'
if (-not (Test-Path $exe)) { throw "WoW.exe introuvable dans $ClientDir" }
# Version lue dans StringFileInfo (la version « fixe » de la ressource peut mentir : piège rencontré en Vanilla).
$ver = (Get-Item $exe).VersionInfo.FileVersion -replace '[ ,]+', '.'
if ($ver -ne "2.4.3.$($Layout.ClientBuild)") { throw "Client inattendu : version $ver (attendu 2.4.3.$($Layout.ClientBuild))" }
Write-Step "Client validé : $exe ($ver)"

# ---- Vue officielle du client (liens physiques : même volume que le client, aucune copie)
$official = @('common.MPQ', 'expansion.MPQ', 'patch.MPQ', 'patch-2.MPQ')
$officialLocale = @('locale-frFR.MPQ', 'expansion-locale-frFR.MPQ', 'patch-frFR.MPQ', 'patch-frFR-2.MPQ',
    'speech-frFR.MPQ', 'expansion-speech-frFR.MPQ', 'base-frFR.MPQ', 'backup-frFR.MPQ')
$clientRoot = [IO.Path]::GetPathRoot((Resolve-Path $ClientDir).Path)
$view = if ([IO.Path]::GetPathRoot($script:ProjectRoot) -eq $clientRoot) { Get-ProjectPath 'UserData\extract-view' } else { Join-Path $clientRoot 'HCBC-extract' }
if (Test-Path $view) { Remove-Item -Recurse -Force $view }
New-Item -ItemType Directory -Force (Join-Path $view 'Data\frFR') | Out-Null
foreach ($n in $official) {
    $src = Join-Path $ClientDir "Data\$n"; if (-not (Test-Path $src)) { throw "Archive officielle absente : $src" }
    New-Item -ItemType HardLink -Path (Join-Path $view "Data\$n") -Target $src | Out-Null
}
foreach ($n in $officialLocale) {
    $src = Join-Path $ClientDir "Data\frFR\$n"
    if (Test-Path $src) { New-Item -ItemType HardLink -Path (Join-Path $view "Data\frFR\$n") -Target $src | Out-Null }
    elseif ($n -notin 'base-frFR.MPQ', 'backup-frFR.MPQ') { throw "Archive officielle absente : $src" }
}
Copy-Item $exe $view
$ignored = @(Get-ChildItem (Join-Path $ClientDir 'Data') -Recurse -Filter '*.MPQ' | Where-Object { $_.Name -notin ($official + $officialLocale) } | ForEach-Object { $_.Name })
if ($ignored.Count) { Write-Step "Archives non officielles ignorées par l'extraction : $($ignored -join ', ')" }
$ClientDir = $view

function Invoke-Extractor {
    # Sortie redirigée vers des fichiers ; sondage du processus (Start-Process -Wait attendrait aussi d'éventuels enfants).
    param([string]$Name, [string]$Exe, [string[]]$Arguments, [string]$WorkDir)
    Write-Step "$Name : $(Split-Path -Leaf $Exe) $($Arguments -join ' ')"
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $Exe -ArgumentList $Arguments -WorkingDirectory $WorkDir -NoNewWindow -PassThru `
        -RedirectStandardOutput "$log\$Name.stdout.log" -RedirectStandardError "$log\$Name.stderr.log"
    $h = $p.Handle   # garde le handle : ExitCode reste lisible après la fin
    while (-not $p.HasExited) { Start-Sleep -Seconds 2 }
    Write-Step "$Name terminé en $([int]$sw.Elapsed.TotalMinutes) min, code $($p.ExitCode)"
    if ($p.ExitCode -ne 0) { throw "$Name a échoué (voir $log\$Name.stdout.log et .stderr.log)" }
}

if ($Steps -contains 'dbcmaps') {
    # ad écrit <sortie>/dbc, <sortie>/maps (et Cameras) ; mangosd lit DataDir/dbc et DataDir/maps.
    Invoke-Extractor 'ad' (Join-Path $ext 'ad.exe') @('-i', "`"$ClientDir`"", '-o', "`"$data`"") $data
}
if ($Steps -contains 'vmaps') {
    # vmap_extractor écrit <sortie>/Buildings ; vmap_assembler lit Buildings et écrit vmaps (dossier à créer avant).
    Invoke-Extractor 'vmap_extractor' (Join-Path $ext 'vmap_extractor.exe') @('-d', "`"$(Join-Path $ClientDir 'Data')`"", '-o', "`"$data`"") $data
    New-Item -ItemType Directory -Force (Join-Path $data 'vmaps') | Out-Null
    Invoke-Extractor 'vmap_assembler' (Join-Path $ext 'vmap_assembler.exe') @("`"$(Join-Path $data 'Buildings')`"", "`"$(Join-Path $data 'vmaps')`"") $data
}
if ($Steps -contains 'mmaps') {
    New-Item -ItemType Directory -Force (Join-Path $data 'mmaps') | Out-Null
    Invoke-Extractor 'MoveMapGen' (Join-Path $ext 'MoveMapGen.exe') @('--silent', '--configInputPath', "`"$(Join-Path $ext 'config.json')`"",
        '--offMeshInput', "`"$(Join-Path $ext 'offmesh.txt')`"", '--workdir', "`"$(($data -replace '\\', '/'))/`"", '--threads', $Threads, '--buildGameObjects') $data
}

Remove-Item -Recurse -Force $view -ErrorAction SilentlyContinue   # liens seulement : le client n'est pas touché

Write-Step "Contenu de $data :"
Get-ChildItem $data -Directory | ForEach-Object {
    $n = (Get-ChildItem $_.FullName -Recurse -File | Measure-Object Length -Sum)
    Write-Step ("   {0,-12} {1,7} fichiers {2,8:N0} Mo" -f $_.Name, $n.Count, ($n.Sum / 1MB))
}
