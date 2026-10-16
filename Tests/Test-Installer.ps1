<#
.SYNOPSIS
    Test de l'installateur Windows (Inno Setup) : installation silencieuse dans C:\HomeCraftTest-Setup avec le client
    du projet passé en paramètre (/ClientDir), raccourcis, réglage du client écrit, premier démarrage complet du
    launcher installé, puis désinstallation silencieuse : programme retiré, UserData et Client conservés.
    Le serveur de jeu doit être arrêté (ports libres).
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
. "$root\Tools\Common.ps1"
function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }
foreach ($p in $Layout.MariaDbPort, $Layout.RealmdPort, $Layout.WorldPort) { if (Test-Port $p) { throw "Port $p occupé : arrêter les services d'abord" } }

$setup = Get-ChildItem "$root\Build\package" -Filter 'HomeCraft-Edition-Vanilla-v*-setup.exe' | Sort-Object Name -Descending | Select-Object -First 1
if (-not $setup) { throw "Installateur introuvable : lancer Build\New-Installer.ps1" }
$dest = 'C:\HomeCraftTest-Setup'
$results = [ordered]@{}
if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
$setupLog = Join-Path $env:TEMP 'homecraft-setup.log'

Write-Step "Installation silencieuse de $($setup.Name) vers $dest"
$p = Start-Process -FilePath $setup.FullName -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/CURRENTUSER', "/DIR=`"$dest`"", "/ClientDir=`"$root\Client`"", "/LOG=`"$setupLog`"") -PassThru -Wait
$results["installateur terminé (code $($p.ExitCode))"] = ($p.ExitCode -eq 0)
$exe = "$dest\HomeCraft Edition Vanilla.exe"
$results['launcher installé'] = (Test-Path $exe)
$results['serveur, base, documentation, licences installés'] = (Test-Path "$dest\Server\vmangos\mangosd.exe") -and (Test-Path "$dest\Server\Database\mariadb\bin\mariadbd.exe") -and (Test-Path "$dest\Documentation\Manuel-Utilisateur.md") -and (Test-Path "$dest\LICENCE.txt") -and (Test-Path "$dest\LICENCES\SOURCES.md")
$results['autorun.inf exclu'] = -not (Test-Path "$dest\autorun.inf")
$results['désinstallateur présent'] = [bool](Get-ChildItem $dest -Filter 'unins*.exe')
$group = Join-Path ([Environment]::GetFolderPath('Programs')) 'HomeCraft Edition Vanilla'
$results['raccourcis du menu Démarrer'] = (Test-Path "$group\HomeCraft Edition Vanilla.lnk") -and (Test-Path "$group\Manuel d'utilisation.lnk")
$settings = "$dest\UserData\settings.json"
# Dans le JSON, chaque antislash du chemin est doublé.
$expectedJson = '"client_dir":"' + (("$root\Client") -replace '\\', '\\') + '"'
$results['dossier du client écrit dans settings.json'] = (Test-Path $settings) -and ((Get-Content $settings -Raw -Encoding UTF8).Contains($expectedJson))
$sig = Get-AuthenticodeSignature $setup.FullName
$results['installateur signé'] = [bool]$sig.SignerCertificate

# Premier démarrage du launcher installé (mmaps désactivées pour le test : génération longue en arrière-plan).
[System.IO.File]::WriteAllText($settings, ('{"client_dir":"' + ("$root\Client" -replace '\\', '\\') + '","windowed":true,"auto_mmaps":false}'), (New-Object System.Text.UTF8Encoding $false))
Write-Step "Premier démarrage du launcher installé (--start --quit-after 900)"
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath $exe -ArgumentList @('--start', '--quit-after', '900') -WorkingDirectory $dest -PassThru
$worldAt = $null
while (-not $p.HasExited) {
    if (-not $worldAt -and (Test-Port $Layout.WorldPort)) { $worldAt = [int]$sw.Elapsed.TotalSeconds; Write-Step "monde en écoute à t+$worldAt s"; break }
    Start-Sleep -Seconds 2
    if ($sw.Elapsed.TotalSeconds -gt 1000) { $p.Kill(); break }
}
$results['monde opérationnel depuis l''installation'] = [bool]$worldAt
# Arrêt propre : fermeture de la fenêtre (le launcher arrête ses services), sinon arrêt forcé.
if (-not $p.HasExited) { $p.CloseMainWindow() | Out-Null; $d = (Get-Date).AddSeconds(90); while (-not $p.HasExited -and (Get-Date) -lt $d) { Start-Sleep -Seconds 2 } }
if (-not $p.HasExited) { $p.Kill() }
Start-Sleep -Seconds 3
Get-Process mangosd, realmd, mariadbd -ErrorAction SilentlyContinue | Stop-Process -Force
$results['arrêt propre après fermeture de la fenêtre'] = ((Get-Content "$dest\Logs\launcher.log" -Raw -Encoding UTF8) -match 'MariaDB arrêtée')

Write-Step "Désinstallation silencieuse"
$unins = Get-ChildItem $dest -Filter 'unins*.exe' | Select-Object -First 1
$p = Start-Process -FilePath $unins.FullName -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -PassThru -Wait
Start-Sleep -Seconds 3
$results["désinstallateur terminé (code $($p.ExitCode))"] = ($p.ExitCode -eq 0)
$results['programme retiré (exe, Server, Logs)'] = -not (Test-Path $exe) -and -not (Test-Path "$dest\Server") -and -not (Test-Path "$dest\Logs")
$results['UserData conservé (base, réglages)'] = (Test-Path "$dest\UserData\Database") -and (Test-Path $settings)
$results['raccourcis retirés'] = -not (Test-Path "$group\HomeCraft Edition Vanilla.lnk")

$ok = $true
foreach ($k in $results.Keys) { Write-Step ("{0,-52} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
Write-Step "Journal de l'installateur : $setupLog ; reste du dossier de test (UserData) : $dest"
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
