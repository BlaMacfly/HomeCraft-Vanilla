<#
.SYNOPSIS
    Test du package portable (HomeCraft Edition BC) : première installation depuis une copie vierge du package
    (initialisation MariaDB, import des 4 bases TBC-DB et des compléments, extraction des données du client sur la vue
    des archives officielles, génération des mmaps, première initialisation des PlayerBots, monde opérationnel), puis
    fermeture brutale du launcher, nettoyage des services orphelins et redémarrage (monde opérationnel, arrêt propre).
    Le client est emprunté à <projet>\Client via UserData\settings.json (jamais copié). Durée : ~30 min.
    Pas d'image ISO : distribution portable uniquement (décision de l'utilisateur).
.PARAMETER Work  Dossier de test (chemin court : extracteurs ; hors racine de C:\ pour pouvoir le supprimer).
#>
[CmdletBinding()]
param([string]$Work = (Join-Path $env:USERPROFILE 'HomeCraftTestBC'))
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
. "$root\Tools\Common.ps1"
foreach ($p in $Layout.MariaDbPort, $Layout.RealmdPort, $Layout.WorldPort) { if (Test-Port $p) { throw "Port $p occupé : arrêter les services d'abord" } }
foreach ($n in 'HomeCraft Edition Vanilla', 'HomeCraft Edition WoTLK', 'HomeCraft Edition BC') { if (Get-Process $n -ErrorAction SilentlyContinue) { throw "$n est en cours d'exécution : test refusé" } }

$pkg = (Get-ChildItem "$root\Build\package" -Directory -Filter 'HomeCraft-Edition-BC-v*' | Sort-Object Name -Descending | Select-Object -First 1).FullName
if (-not $pkg) { throw "Package introuvable : lancer Build\New-Package.ps1" }
$exe = Join-Path $Work 'HomeCraft Edition BC.exe'
$results = [ordered]@{}
function Stop-Orphans { Get-Process mangosd, realmd, mariadbd -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "$Work\*" } | Stop-Process -Force; Start-Sleep -Seconds 2 }

if (Test-Path $Work) { Remove-Item -Recurse -Force $Work }
Write-Step "Copie du package vers $Work"
Copy-Item -Recurse $pkg $Work
$results['copie du package'] = (Test-Path $exe)
$results['empreintes SHA256SUMS conformes'] = -not (Get-Content "$Work\SHA256SUMS.txt" | Where-Object { $_ } | Where-Object {
    $h, $f = $_ -split '  ', 2; (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $Work $f)).Hash.ToLowerInvariant() -ne $h })
New-Item -ItemType Directory -Force "$Work\UserData" | Out-Null
[System.IO.File]::WriteAllText("$Work\UserData\settings.json", ('{"client_dir":"' + ("$root\Client" -replace '\\', '\\') + '","windowed":true}'), (New-Object System.Text.UTF8Encoding $false))

Write-Step "Premier démarrage (--start) : base, import, extraction, mmaps, PlayerBots, serveur"
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath $exe -ArgumentList @('--start', '--quit-after', '7200') -WorkingDirectory $Work -PassThru
$worldAt = $null
while (-not $p.HasExited -and $sw.Elapsed.TotalMinutes -lt 110) {
    if ((Get-Content "$Work\Logs\launcher.log" -Encoding UTF8 -ErrorAction SilentlyContinue) -match 'monde opérationnel|ERREUR') { break }
    Start-Sleep -Seconds 10
}
$log = Get-Content "$Work\Logs\launcher.log" -Encoding UTF8 -Raw
if ($log -match 'monde opérationnel') { $worldAt = [int]$sw.Elapsed.TotalMinutes; Write-Step "monde opérationnel après $worldAt min" }
$results['datadir MariaDB initialisé'] = ($log -match 'première initialisation du datadir')
$results['bases importées (4)'] = ($log -match 'bases importées : tbcrealmd, tbccharacters, tbcmangos, tbclogs')
$results['compléments SQL appliqués (schéma + français)'] = ($log -match 'import : complément 00_homecraft_schema_locales_gameobject.sql') -and ($log -match 'import : complément 50_homecraft_frFR.sql')
$results['archive non officielle ignorée'] = ($log -match 'archives non officielles ignorées : frFR/patch-frFR-3.MPQ')
$results['données du client extraites (dbc, maps, vmaps, mmaps)'] = ($log -match 'données du client extraites') -and ((Get-ChildItem "$Work\Server\Data\dbc" -File).Count -ge 180) -and ((Get-ChildItem "$Work\Server\Data\vmaps" -File).Count -ge 5000) -and ((Get-ChildItem "$Work\Server\Data\mmaps" -File).Count -ge 1000) -and -not (Test-Path "$Work\Server\Data\mmaps\.generation-en-cours")
$results['Spell.dbc officiel (sans patch tiers)'] = ((Get-Item "$Work\Server\Data\dbc\Spell.dbc").Length -eq 25991940)
$results['vue d''extraction supprimée'] = -not (Test-Path "$Work\UserData\extract-view")
$results['PlayerBots actifs'] = ((Get-Content "$Work\Logs\server\Server.log" -Encoding UTF8 -Raw) -match 'random bot accounts with \d+ characters available')
$results['monde opérationnel au premier démarrage'] = [bool]$worldAt

Write-Step "Fermeture brutale du launcher (simulation de plantage), nettoyage des services orphelins"
if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
Stop-Orphans

Write-Step "Second démarrage après arrêt brutal (--start --quit-after 240)"
$before = (Get-Content "$Work\Logs\launcher.log" -Encoding UTF8 -Raw).Length
$sw2 = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath $exe -ArgumentList @('--start', '--quit-after', '240') -WorkingDirectory $Work -PassThru
while (-not $p.HasExited -and $sw2.Elapsed.TotalSeconds -lt 600) { Start-Sleep -Seconds 3 }
Start-Sleep -Seconds 3
$log2 = (Get-Content "$Work\Logs\launcher.log" -Encoding UTF8 -Raw).Substring($before)
$results['redémarrage : monde opérationnel'] = ($log2 -match 'monde opérationnel')
$results['redémarrage : arrêt propre'] = ($log2 -match 'mangosd arrêté proprement') -and ($log2 -match 'MariaDB arrêtée')
$results['aucun processus orphelin'] = -not (Get-Process mangosd, realmd, mariadbd -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "$Work\*" })
Stop-Orphans

$ok = $true
foreach ($k in $results.Keys) { Write-Step ("{0,-56} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
Write-Step "Dossier de test conservé pour inspection : $Work"
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
