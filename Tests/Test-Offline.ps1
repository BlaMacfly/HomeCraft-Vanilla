<#
.SYNOPSIS
    Vérifie le fonctionnement local/hors ligne : pendant un démarrage complet, les trois services n'écoutent
    que sur 127.0.0.1 et aucun processus du jeu n'ouvre de connexion vers une adresse non locale.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
$exe = Get-ProjectPath 'Launcher\target\release\vanilla-solo-launcher.exe'
function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }
if ((Test-Port $Layout.RealmdPort) -or (Test-Port $Layout.WorldPort)) { throw "Serveur déjà en service : arrêter d'abord" }
if (Test-MariaDbAlive) { & (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null }

$p = Start-Process -FilePath $exe -ArgumentList @('--start', '--quit-after', '70') -WorkingDirectory (Split-Path $exe) -PassThru
$d = (Get-Date).AddSeconds(60)
while (-not (Test-Port $Layout.WorldPort) -and (Get-Date) -lt $d) { Start-Sleep -Seconds 1 }
Start-Sleep -Seconds 8
$pids = @(Get-Process -Name 'vanilla-solo-launcher', 'mangosd', 'realmd', 'mariadbd' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
$conns = Get-NetTCPConnection -ErrorAction SilentlyContinue | Where-Object { $pids -contains $_.OwningProcess }
$listen = $conns | Where-Object { $_.State -eq 'Listen' }
$established = $conns | Where-Object { $_.State -ne 'Listen' -and $_.State -ne 'Bound' }
$results = [ordered]@{}
Write-Step "Ports en écoute :"
$listen | ForEach-Object { Write-Step ("   {0}:{1} (pid {2})" -f $_.LocalAddress, $_.LocalPort, $_.OwningProcess) }
$results['écoute uniquement sur 127.0.0.1'] = ($listen.Count -gt 0) -and -not ($listen | Where-Object { $_.LocalAddress -notin '127.0.0.1', '::1' })
$results['ports attendus 3315, 3724, 8085 présents'] = ((3315, 3724, 8085) | ForEach-Object { $_ -in ($listen.LocalPort) }) -notcontains $false
$remote = $established | Where-Object { $_.RemoteAddress -notin '127.0.0.1', '::1', '0.0.0.0' }
Write-Step "Connexions établies non locales : $($remote.Count)"
$remote | ForEach-Object { Write-Step ("   {0}:{1} -> {2}:{3} (pid {4})" -f $_.LocalAddress, $_.LocalPort, $_.RemoteAddress, $_.RemotePort, $_.OwningProcess) }
$results['aucune connexion sortante non locale'] = ($remote.Count -eq 0)
$p.WaitForExit()
$ok = $true
foreach ($k in $results.Keys) { Write-Step ("{0,-44} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
