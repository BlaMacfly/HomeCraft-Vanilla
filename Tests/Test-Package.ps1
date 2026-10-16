<#
.SYNOPSIS
    Test du package portable (phase 6) :
    1. première installation depuis une copie vierge du package (initialisation MariaDB, import des bases,
       extraction des données du client, démarrage complet, arrêt propre) ;
    2. image ISO montée en lecture seule : détection et copie assistée (--install-to).
    Le client est emprunté à <projet>\Client via UserData\settings.json. Le serveur de jeu doit être arrêté.
#>
[CmdletBinding()]
param([switch]$SkipIso, [switch]$OnlyIso)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
. "$root\Tools\Common.ps1"
function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }
foreach ($p in $Layout.MariaDbPort, $Layout.RealmdPort, $Layout.WorldPort) { if (Test-Port $p) { throw "Port $p occupé : arrêter les services d'abord" } }

$pkg = (Get-ChildItem "$root\Build\package" -Directory -Filter 'HomeCraft-Edition-Vanilla-v*' | Sort-Object Name -Descending | Select-Object -First 1).FullName
if (-not $pkg) { throw "Package introuvable : lancer Build\New-Package.ps1" }
$work = 'C:\HomeCraftTest'   # chemin court : les extracteurs VMaNGOS tronquent les chemins longs
$results = [ordered]@{}

# ---- 1. Installation vierge
if (-not $OnlyIso) {
if (Test-Path $work) { Remove-Item -Recurse -Force $work }
Write-Step "Copie du package vers $work"
Copy-Item -Recurse $pkg $work
$results['copie du package'] = (Test-Path "$work\HomeCraft Edition Vanilla.exe")
New-Item -ItemType Directory -Force "$work\UserData" | Out-Null
[System.IO.File]::WriteAllText("$work\UserData\settings.json", ('{"client_dir":"' + ("$root\Client" -replace '\\','\\') + '","windowed":true,"auto_mmaps":false}'), (New-Object System.Text.UTF8Encoding $false))

Write-Step "Premier démarrage (--start --quit-after 900) : base, import, extraction, serveur"
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath "$work\HomeCraft Edition Vanilla.exe" -ArgumentList @('--start', '--quit-after', '900') -WorkingDirectory $work -PassThru
$worldAt = $null
while (-not $p.HasExited) {
    if (-not $worldAt -and (Test-Port $Layout.WorldPort)) { $worldAt = [int]$sw.Elapsed.TotalSeconds; Write-Step "monde en écoute à t+$worldAt s : fermeture demandée"; New-Item -ItemType File -Force "$work\Logs\server\stop" | Out-Null; Stop-Process -Id $p.Id -ErrorAction SilentlyContinue; break }
    Start-Sleep -Seconds 2
    if ($sw.Elapsed.TotalSeconds -gt 1000) { $p.Kill(); break }
}
# Le kill ci-dessus simule une fermeture brutale ; on vérifie ensuite que le launcher redémarre proprement après ça.
Start-Sleep -Seconds 3
Get-Process mangosd, realmd, mariadbd -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
$log = Get-Content "$work\Logs\launcher.log" -Encoding UTF8 -Raw
$results['datadir MariaDB initialisé'] = ($log -match 'première initialisation du datadir')
$results['bases importées (4)'] = ($log -match 'bases importées : realmd, characters, mangos, logs')
$results['données du client extraites'] = ($log -match 'données du client extraites') -and ((Get-ChildItem "$work\Server\Data\5875\dbc" -File).Count -ge 150) -and ((Get-ChildItem "$work\Server\Data\vmaps" -File).Count -ge 1000)
$results['monde opérationnel au premier démarrage'] = [bool]$worldAt
& "$root\Tools\Start-MariaDB.ps1" | Out-Null   # (non utilisé : la base du test est dans C:\HomeCraftTest) — garde-fou désactivé
& "$root\Tools\Stop-MariaDB.ps1" | Out-Null

Write-Step "Second démarrage après arrêt brutal (--start --quit-after 60)"
$p = Start-Process -FilePath "$work\HomeCraft Edition Vanilla.exe" -ArgumentList @('--start', '--quit-after', '60') -WorkingDirectory $work -PassThru -Wait
Start-Sleep -Seconds 2
$log2 = (Get-Content "$work\Logs\launcher.log" -Encoding UTF8 -Raw).Substring($log.Length)
$results['redémarrage après arrêt brutal : monde opérationnel'] = ($log2 -match 'monde opérationnel')
$results['redémarrage : arrêt propre'] = ($log2 -match 'mangosd arrêté proprement') -and ($log2 -match 'MariaDB arrêtée')
$results['AHBot : compléments SQL appliqués'] = ($log -match 'import : complément auctionhousebot_solo.sql') -and ($log2 -match 'auctionhousebot')
$results['traductions HomeCraft : compléments SQL appliqués'] = ($log -match 'import : complément locales_quest_homecraft.sql') -and ($log -match 'import : complément locales_creature_homecraft.sql')
$results['aucun processus orphelin'] = -not (Get-Process mangosd, realmd, mariadbd -ErrorAction SilentlyContinue)
}

# ---- 2. ISO en lecture seule
if (-not $SkipIso) {
    Write-Step "Création de l'image ISO"
    $isoOut = & "$root\Build\New-Iso.ps1" -Source $pkg
    $isoOut | ForEach-Object { Write-Step "   $_" }
    $iso = "$pkg.iso"
    $results['ISO créée'] = (Test-Path $iso)
    $disk = Mount-DiskImage -ImagePath $iso -PassThru
    Start-Sleep -Seconds 3
    $letter = ($disk | Get-Volume).DriveLetter
    Write-Step "ISO montée sur ${letter}:"
    $dest = 'C:\HomeCraftTest-Install'
    if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
    # Pas de -Wait : sous PowerShell 5.1 il attendrait aussi le launcher installé (processus enfant) lancé à la fin de la copie.
    $p = Start-Process -FilePath "${letter}:\HomeCraft Edition Vanilla.exe" -ArgumentList @('--install-to', $dest, '--quit-after', '240') -WorkingDirectory "${letter}:\" -PassThru
    $d = (Get-Date).AddSeconds(300)
    while (-not $p.HasExited -and (Get-Date) -lt $d) { Start-Sleep -Seconds 2 }
    Start-Sleep -Seconds 3
    $copiedExe = "$dest\HomeCraft Edition Vanilla.exe"
    $expected = (Get-ChildItem $pkg -Recurse -File).Count
    # >= : le launcher installé crée Logs/launcher.log (et UserData) dès son premier démarrage.
    $results["ISO : copie assistée vers le disque ($expected fichiers)"] = (Test-Path $copiedExe) -and ((Get-ChildItem $dest -Recurse -File).Count -ge $expected)
    $results['ISO : fichiers copiés inscriptibles'] = -not (Get-Item $copiedExe).IsReadOnly
    $spawned = Get-Process | Where-Object { $_.Path -eq $copiedExe }
    $results['ISO : launcher installé démarré automatiquement'] = [bool]$spawned
    $spawned | Stop-Process -Force -ErrorAction SilentlyContinue
    Dismount-DiskImage -ImagePath $iso | Out-Null
    Remove-Item -Recurse -Force $dest -ErrorAction SilentlyContinue
}

$ok = $true
foreach ($k in $results.Keys) { Write-Step ("{0,-52} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
Write-Step "Dossier de test conservé pour inspection : $work"
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
