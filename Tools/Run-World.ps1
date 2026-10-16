<#
.SYNOPSIS
    Harnais de développement : démarre MariaDB, realmd et mangosd, puis reste en vie.
    - Les lignes ajoutées à Logs/server/console.in sont transmises à la console de mangosd (Send-Console.ps1).
    - La sortie de mangosd est copiée dans Logs/server/mangosd.console.log.
    - Créer le fichier Logs/server/stop (ou Ctrl+C) arrête proprement : « server exit » à mangosd, puis realmd, puis MariaDB (-StopDatabase).
    Préfigure la machine à états du launcher (Phase 4) ; n'est pas destiné à l'utilisateur final.
#>
[CmdletBinding()]
param(
    [int]$StartupTimeoutSeconds = 600,
    [switch]$StopDatabase
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

& "$PSScriptRoot\Start-MariaDB.ps1"
& "$PSScriptRoot\New-ServerConfig.ps1"
# Builds acceptés : 5875 et 5880 (le client frFR fourni annonce 5880). realmbuilds sinon royaume « hors ligne » ; allowed_clients sinon « version invalide ».
Invoke-Sql -Database realmd -Query "INSERT INTO realmlist (id,name,address,localAddress,localSubnetMask,port,icon,realmflags,timezone,allowedSecurityLevel,population,gamebuild_min,gamebuild_max,flag,realmbuilds) VALUES (1,'Vanilla Solo','127.0.0.1','127.0.0.1','255.255.255.0',$($Layout.WorldPort),1,0,0,0,0,5875,5880,0,'5875 5880') ON DUPLICATE KEY UPDATE address='127.0.0.1', port=$($Layout.WorldPort), gamebuild_min=5875, gamebuild_max=5880, realmbuilds='5875 5880'"
Invoke-Sql -Database realmd -Query "INSERT INTO allowed_clients (major_version,minor_version,bugfix_version,hotfix_version,build,os,platform,integrity_hash) SELECT 1,12,1,'',5880,'Win','x86','' FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM allowed_clients WHERE build=5880 AND os='Win' AND platform='x86')"

$bin = Get-ProjectPath 'Server\vmangos'
$cfg = Get-ProjectPath 'Server\Config'
$srvLog = Join-Path $Layout.Logs 'server'
New-Item -ItemType Directory -Force $srvLog | Out-Null
$consoleIn = Join-Path $srvLog 'console.in'
$consoleOut = Join-Path $srvLog 'mangosd.console.log'
$stopFlag = Join-Path $srvLog 'stop'
Remove-Item -Force $stopFlag, $consoleIn -ErrorAction SilentlyContinue
New-Item -ItemType File -Force $consoleIn | Out-Null

function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }

if (Test-Port $Layout.RealmdPort) { throw "Le port $($Layout.RealmdPort) est déjà utilisé : un realmd tourne déjà ?" }
if (Test-Port $Layout.WorldPort)  { throw "Le port $($Layout.WorldPort) est déjà utilisé : un mangosd tourne déjà ?" }

Write-Step "Démarrage realmd"
$realmd = Start-Process -FilePath (Join-Path $bin 'realmd.exe') -ArgumentList "-c `"$cfg\realmd.conf`"" -WorkingDirectory $bin -WindowStyle Hidden -PassThru -RedirectStandardOutput "$srvLog\realmd.stdout.log" -RedirectStandardError "$srvLog\realmd.stderr.log"
$deadline = (Get-Date).AddSeconds(30)
while (-not (Test-Port $Layout.RealmdPort) -and (Get-Date) -lt $deadline -and -not $realmd.HasExited) { Start-Sleep -Milliseconds 300 }
if (-not (Test-Port $Layout.RealmdPort)) { throw "realmd ne répond pas sur $($Layout.RealmdPort)" }
Write-Step "realmd prêt (PID $($realmd.Id))"

Write-Step "Démarrage mangosd"
$psi = New-Object Diagnostics.ProcessStartInfo
$psi.FileName = Join-Path $bin 'mangosd.exe'
$psi.Arguments = "-c `"$cfg\mangosd.conf`""
$psi.WorkingDirectory = $bin
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.CreateNoWindow = $true
$psi.StandardOutputEncoding = [Text.Encoding]::UTF8   # mangos_string (français) est émis en UTF-8
$psi.StandardErrorEncoding  = [Text.Encoding]::UTF8
$mangosd = [Diagnostics.Process]::Start($psi)
$outWriter = [IO.StreamWriter]::new($consoleOut, $false, [Text.Encoding]::UTF8)
$outWriter.AutoFlush = $true
$outputHandler = { if ($null -ne $EventArgs.Data) { $Event.MessageData.WriteLine($EventArgs.Data) } }
$ev1 = Register-ObjectEvent -InputObject $mangosd -EventName OutputDataReceived -Action $outputHandler -MessageData $outWriter
$ev2 = Register-ObjectEvent -InputObject $mangosd -EventName ErrorDataReceived  -Action $outputHandler -MessageData $outWriter
$mangosd.BeginOutputReadLine(); $mangosd.BeginErrorReadLine()

$deadline = (Get-Date).AddSeconds($StartupTimeoutSeconds)
while (-not $mangosd.HasExited -and (Get-Date) -lt $deadline -and -not (Test-Port $Layout.WorldPort)) { Start-Sleep -Milliseconds 500 }
if ($mangosd.HasExited) { Write-Step "mangosd s'est arrêté pendant le démarrage (code $($mangosd.ExitCode)) : voir $consoleOut"; Stop-Process -Id $realmd.Id -Force; exit 1 }
if (-not (Test-Port $Layout.WorldPort)) { Write-Step "mangosd ne répond pas après $StartupTimeoutSeconds s"; $mangosd.Kill(); Stop-Process -Id $realmd.Id -Force; exit 1 }
Write-Step "mangosd prêt (PID $($mangosd.Id)) sur 127.0.0.1:$($Layout.WorldPort). Console : Send-Console.ps1 ; arrêt : fichier $stopFlag"

# Boucle de service : transmet console.in, surveille stop et les processus.
$sent = 0
try {
    while (-not $mangosd.HasExited) {
        if (Test-Path $stopFlag) { break }
        $lines = @(Get-Content $consoleIn -Encoding UTF8 -ErrorAction SilentlyContinue)
        if ($lines.Count -gt $sent) {
            foreach ($l in $lines[$sent..($lines.Count - 1)]) { if ($l.Trim()) { Write-Step "console> $l"; $mangosd.StandardInput.WriteLine($l) } }
            $sent = $lines.Count
        }
        if ($realmd.HasExited) { Write-Step "realmd s'est arrêté (code $($realmd.ExitCode))" ; break }
        Start-Sleep -Milliseconds 500
    }
} finally {
    if (-not $mangosd.HasExited) {
        Write-Step "Arrêt de mangosd (server exit)"
        # Fermer stdin est indispensable : après « Halting process... » mangosd attend la fin du thread console,
        # qui ne se termine qu'à la fin de l'entrée standard (sinon 60 s puis kill).
        try { $mangosd.StandardInput.WriteLine('server exit'); $mangosd.StandardInput.Flush(); $mangosd.StandardInput.Close() } catch {}
        if (-not $mangosd.WaitForExit(60000)) { Write-Step "mangosd ne répond pas : arrêt forcé"; $mangosd.Kill() }
    }
    Write-Step "mangosd terminé (code $($mangosd.ExitCode))"
    Unregister-Event -SourceIdentifier $ev1.Name; Unregister-Event -SourceIdentifier $ev2.Name
    $outWriter.Dispose()
    if (-not $realmd.HasExited) { Stop-Process -Id $realmd.Id -Force; Write-Step "realmd arrêté" }
    if ($StopDatabase) { & "$PSScriptRoot\Stop-MariaDB.ps1" }
}
