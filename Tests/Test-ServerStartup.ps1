<#
.SYNOPSIS
    Test d'intégration : MariaDB -> realmd -> mangosd. Vérifie que les serveurs se connectent à la base,
    que le contrôle des migrations passe, et où s'arrête mangosd (DBC absents attendus tant que le client n'est pas extrait).
    Arrête tout ce qu'il a démarré. Sortie : 0 si les critères sont remplis.
#>
[CmdletBinding()]
param([int]$MangosdSeconds = 90)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"

& (Get-ProjectPath 'Tools\Start-MariaDB.ps1')
& (Get-ProjectPath 'Tools\New-ServerConfig.ps1')

# Royaume local (idempotent)
Invoke-Sql -Database realmd -Query "INSERT INTO realmlist (id,name,address,localAddress,localSubnetMask,port,icon,realmflags,timezone,allowedSecurityLevel,population,gamebuild_min,gamebuild_max,flag,realmbuilds) VALUES (1,'HomeCraft','127.0.0.1','127.0.0.1','255.255.255.0',$($Layout.WorldPort),1,0,0,0,0,5875,5880,0,'5875 5880') ON DUPLICATE KEY UPDATE address='127.0.0.1', port=$($Layout.WorldPort), gamebuild_min=5875, gamebuild_max=5880, realmbuilds='5875 5880'"

$bin = Get-ProjectPath 'Server\vmangos'
$cfg = Get-ProjectPath 'Server\Config'
$out = Join-Path $Layout.Logs 'tests'
New-Item -ItemType Directory -Force $out | Out-Null
$ok = $true

function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }

# ---- realmd
Write-Step "Démarrage realmd"
$realmd = Start-Process -FilePath (Join-Path $bin 'realmd.exe') -ArgumentList "-c `"$cfg\realmd.conf`"" -WorkingDirectory $bin -WindowStyle Hidden -PassThru -RedirectStandardOutput "$out\realmd.stdout.log" -RedirectStandardError "$out\realmd.stderr.log"
$deadline = (Get-Date).AddSeconds(30)
while (-not (Test-Port $Layout.RealmdPort) -and (Get-Date) -lt $deadline -and -not $realmd.HasExited) { Start-Sleep -Milliseconds 300 }
if (Test-Port $Layout.RealmdPort) { Write-Step "realmd écoute sur 127.0.0.1:$($Layout.RealmdPort) : OK" } else { Write-Step "realmd n'écoute pas (exited=$($realmd.HasExited))" ; $ok = $false }

# ---- mangosd
Write-Step "Démarrage mangosd (observation $MangosdSeconds s)"
$psi = New-Object Diagnostics.ProcessStartInfo
$psi.FileName = Join-Path $bin 'mangosd.exe'
$psi.Arguments = "-c `"$cfg\mangosd.conf`""
$psi.WorkingDirectory = $bin
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.CreateNoWindow = $true
$mangosd = [Diagnostics.Process]::Start($psi)
$stdoutTask = $mangosd.StandardOutput.ReadToEndAsync()
$stderrTask = $mangosd.StandardError.ReadToEndAsync()
$deadline = (Get-Date).AddSeconds($MangosdSeconds)
while (-not $mangosd.HasExited -and (Get-Date) -lt $deadline -and -not (Test-Port $Layout.WorldPort)) { Start-Sleep -Milliseconds 500 }
$worldUp = Test-Port $Layout.WorldPort
if ($worldUp) { Write-Step "mangosd écoute sur 127.0.0.1:$($Layout.WorldPort)" }
if (-not $mangosd.HasExited) {
    Write-Step "Arrêt de mangosd via la console (quit)"
    try { $mangosd.StandardInput.WriteLine('quit'); $mangosd.StandardInput.Close() } catch {}
    if (-not $mangosd.WaitForExit(30000)) { Write-Step "mangosd ne s'arrête pas : kill"; $mangosd.Kill(); $ok = $false }
}
$stdout = $stdoutTask.Result; $stderr = $stderrTask.Result
Set-Content "$out\mangosd.stdout.log" $stdout -Encoding UTF8
Set-Content "$out\mangosd.stderr.log" $stderr -Encoding UTF8
Write-Step "mangosd terminé, code $($mangosd.ExitCode)"

# ---- realmd stop (pas de console : arrêt par signal)
if (-not $realmd.HasExited) { Stop-Process -Id $realmd.Id -Force; Write-Step "realmd arrêté" }

# ---- critères
$all = $stdout + "`n" + $stderr
$checks = [ordered]@{
    'Connexion base monde'        = ($all -match '(?i)connected to mysql|MySQL client library|Database: .*mangos')
    'Aucune migration manquante'  = -not ($all -match '(?i)missing migration|requires? .*migration|Please apply')
    'Pas de refus de connexion DB'= -not ($all -match '(?i)Cannot connect to|Access denied|Can.t connect')
}
foreach ($k in $checks.Keys) { Write-Step ("{0,-32} {1}" -f $k, $(if ($checks[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $checks[$k]) { $ok = $false } }
Write-Step "Dernières lignes mangosd :"
($all -split "`r?`n" | Where-Object { $_ -ne '' } | Select-Object -Last 25) | ForEach-Object { "    $_" }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
