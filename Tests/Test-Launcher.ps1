<#
.SYNOPSIS
    Test d'intégration du launcher Rust : démarrage automatique des services (--start), fermeture après N secondes
    (--quit-after), puis vérification du journal et de la libération des ports. Exige que le harnais Run-World soit arrêté.
#>
[CmdletBinding()]
param([int]$QuitAfter = 90)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"

$exe = Get-ProjectPath 'Launcher\target\release\vanilla-solo-launcher.exe'
if (-not (Test-Path $exe)) { throw "Launcher non compilé : $exe" }
function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }

# Le harnais de développement ne doit pas tenir les ports.
$stopFlag = Join-Path $Layout.Logs 'server\stop'
if ((Test-Port $Layout.RealmdPort) -or (Test-Port $Layout.WorldPort)) {
    Write-Step "Arrêt du harnais Run-World (fichier stop)"
    New-Item -ItemType File -Force $stopFlag | Out-Null
    $d = (Get-Date).AddSeconds(90)
    while (((Test-Port $Layout.RealmdPort) -or (Test-Port $Layout.WorldPort)) -and (Get-Date) -lt $d) { Start-Sleep -Milliseconds 500 }
}
if (Test-MariaDbAlive) { & (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') }
foreach ($p in $Layout.MariaDbPort, $Layout.RealmdPort, $Layout.WorldPort) { if (Test-Port $p) { throw "Port $p encore occupé" } }

$log = Get-ProjectPath 'Logs\launcher.log'
$before = if (Test-Path $log) { (Get-Content $log | Measure-Object).Count } else { 0 }
Write-Step "Lancement du launcher : --start --quit-after $QuitAfter"
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath $exe -ArgumentList @('--start', '--quit-after', $QuitAfter) -WorkingDirectory (Split-Path $exe) -PassThru

# Pendant l'exécution : les trois ports doivent s'ouvrir.
$seen = @{}
while (-not $p.HasExited) {
    foreach ($port in $Layout.MariaDbPort, $Layout.RealmdPort, $Layout.WorldPort) { if (-not $seen[$port] -and (Test-Port $port)) { $seen[$port] = [int]$sw.Elapsed.TotalSeconds; Write-Step "port $port ouvert à t+$($seen[$port]) s" } }
    Start-Sleep -Milliseconds 500
    if ($sw.Elapsed.TotalSeconds -gt ($QuitAfter + 180)) { Write-Step "délai dépassé : arrêt forcé du launcher"; $p.Kill(); break }
}
Write-Step "launcher terminé (code $($p.ExitCode)) après $([int]$sw.Elapsed.TotalSeconds) s"
Start-Sleep -Seconds 2

$lines = @(Get-Content $log -Encoding UTF8 | Select-Object -Skip $before)
$text = $lines -join "`n"
$ok = $true
$checks = [ordered]@{
    'MariaDB démarrée'            = ($text -match 'MariaDB opérationnelle')
    'realmd démarré'              = ($text -match 'realmd opérationnel')
    'monde opérationnel'          = ($text -match 'monde opérationnel')
    'mangosd arrêté proprement'   = ($text -match 'mangosd arrêté proprement')
    'MariaDB arrêtée'             = ($text -match 'MariaDB arrêtée')
    'aucune ERREUR'               = -not ($text -match 'ERREUR')
    'port 3315 libéré'            = -not (Test-Port $Layout.MariaDbPort)
    'port 3724 libéré'            = -not (Test-Port $Layout.RealmdPort)
    'port 8085 libéré'            = -not (Test-Port $Layout.WorldPort)
    'aucun processus orphelin'    = -not (Get-Process mangosd, realmd, mariadbd -ErrorAction SilentlyContinue)
}
foreach ($k in $checks.Keys) { Write-Step ("{0,-28} {1}" -f $k, $(if ($checks[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $checks[$k]) { $ok = $false } }
Write-Step "Journal du launcher ($($lines.Count) lignes) :"
$lines | Where-Object { $_ -notmatch '\[mangosd\]' } | Select-Object -Last 30 | ForEach-Object { "    $_" }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
