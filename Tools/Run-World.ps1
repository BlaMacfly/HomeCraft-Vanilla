<#
.SYNOPSIS
    Harnais de développement : démarre MySQL, authserver et worldserver, puis reste en vie.
    - Les lignes ajoutées à Logs/server/console.in sont transmises à la console de worldserver (Send-Console.ps1).
    - La sortie de worldserver est copiée dans Logs/server/worldserver.console.log.
    - Créer le fichier Logs/server/stop (ou Ctrl+C) arrête proprement : « server shutdown 1 » à worldserver,
      puis authserver, puis MySQL (-StopDatabase).
    Préfigure la machine à états du launcher ; n'est pas destiné à l'utilisateur final.
    Adapté de ../HomeCraft/Tools/Run-World.ps1 (VMaNGOS → AzerothCore).
#>
[CmdletBinding()]
param(
    [int]$StartupTimeoutSeconds = 1800,
    [switch]$StopDatabase,
    [string]$RealmName = "HomeCraft",
    [int]$RandomBots = 40
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

& "$PSScriptRoot\Start-Database.ps1"
& "$PSScriptRoot\New-ServerConfig.ps1" -RandomBots $RandomBots

# AzerothCore refuse « -c <chemin> » quand le chemin contient un espace (boost::program_options + fs::path) :
# tout passe par la jonction courte C:\hcw -> racine du projet.
$short = $Layout.ShortRoot
if (-not (Test-Path $short)) { cmd /c mklink /J "$short" "$ProjectRoot" | Out-Null }
if (-not (Test-Path (Join-Path $short 'Server\acore\worldserver.exe'))) { throw "jonction $short invalide (doit pointer sur $ProjectRoot)" }
$bin = Join-Path $short 'Server\acore'
$cfg = Join-Path $short 'Server\acore\configs'
$srvLog = Join-Path $Layout.Logs 'server'
New-Item -ItemType Directory -Force $srvLog | Out-Null
$consoleIn = Join-Path $srvLog 'console.in'
$consoleOut = Join-Path $srvLog 'worldserver.console.log'
$stopFlag = Join-Path $srvLog 'stop'
if (Test-Path $stopFlag) { Remove-Item -Force $stopFlag }
New-Item -ItemType File -Force $consoleIn | Out-Null

function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }

if (Test-Port $Layout.AuthPort)  { throw "Le port $($Layout.AuthPort) est déjà utilisé : un authserver tourne déjà ?" }
if (Test-Port $Layout.WorldPort) { throw "Le port $($Layout.WorldPort) est déjà utilisé : un worldserver tourne déjà ?" }

# Royaume local AVANT authserver : il refuse de démarrer sans ligne dans realmlist (« No valid realms specified »).
# Si la base auth n'est pas encore amorcée (premier lancement depuis le dépôt), la table n'existe pas : on réessaie après.
function Set-LocalRealm {
    $rn = $RealmName -replace "'", "''"
    Invoke-Sql -Database acore_auth -Query "INSERT INTO realmlist (id, name, address, localAddress, localSubnetMask, port, icon, flag, timezone, allowedSecurityLevel, population, gamebuild) VALUES (1, '$rn', '127.0.0.1', '127.0.0.1', '255.255.255.0', $($Layout.WorldPort), 0, 0, 0, 0, 0, 12340) ON DUPLICATE KEY UPDATE name='$rn', address='127.0.0.1', localAddress='127.0.0.1', localSubnetMask='255.255.255.0', port=$($Layout.WorldPort), gamebuild=12340"
}
try { Set-LocalRealm; Write-Step "Royaume '$RealmName' inscrit (127.0.0.1:$($Layout.WorldPort))" } catch { Write-Step "realmlist indisponible avant amorçage ($($_.Exception.Message)) : nouvel essai après authserver" }

Write-Step "Démarrage authserver"
$auth = Start-Process -FilePath (Join-Path $bin 'authserver.exe') -ArgumentList '-c', (Join-Path $cfg 'authserver.conf') -WorkingDirectory $bin -WindowStyle Hidden -PassThru -RedirectStandardOutput "$srvLog\authserver.stdout.log" -RedirectStandardError "$srvLog\authserver.stderr.log"
$deadline = (Get-Date).AddSeconds(120)
while (-not (Test-Port $Layout.AuthPort) -and (Get-Date) -lt $deadline -and -not $auth.HasExited) { Start-Sleep -Milliseconds 300 }
if (-not (Test-Port $Layout.AuthPort)) { throw "authserver ne répond pas sur $($Layout.AuthPort) (voir $srvLog\Auth.log)" }
Write-Step "authserver prêt (PID $($auth.Id))"

# Royaume local (nouvel essai : la base auth existe maintenant, amorcée par authserver si nécessaire).
Set-LocalRealm

Write-Step "Démarrage worldserver"
$psi = New-Object Diagnostics.ProcessStartInfo
$psi.FileName = Join-Path $bin 'worldserver.exe'
$psi.Arguments = "-c `"$(Join-Path $cfg 'worldserver.conf')`""
$psi.WorkingDirectory = $bin
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.CreateNoWindow = $true
$psi.StandardOutputEncoding = [Text.Encoding]::UTF8
$psi.StandardErrorEncoding  = [Text.Encoding]::UTF8
$world = [Diagnostics.Process]::Start($psi)
$outWriter = [IO.StreamWriter]::new($consoleOut, $false, [Text.Encoding]::UTF8)
$outWriter.AutoFlush = $true
$outputHandler = { if ($null -ne $EventArgs.Data) { $Event.MessageData.WriteLine($EventArgs.Data) } }
$ev1 = Register-ObjectEvent -InputObject $world -EventName OutputDataReceived -Action $outputHandler -MessageData $outWriter
$ev2 = Register-ObjectEvent -InputObject $world -EventName ErrorDataReceived  -Action $outputHandler -MessageData $outWriter
$world.BeginOutputReadLine(); $world.BeginErrorReadLine()

$deadline = (Get-Date).AddSeconds($StartupTimeoutSeconds)
while (-not $world.HasExited -and (Get-Date) -lt $deadline -and -not (Test-Port $Layout.WorldPort)) { Start-Sleep -Milliseconds 500 }
if ($world.HasExited) { Write-Step "worldserver s'est arrêté pendant le démarrage (code $($world.ExitCode)) : voir $consoleOut"; Stop-Process -Id $auth.Id -Force; exit 1 }
if (-not (Test-Port $Layout.WorldPort)) { Write-Step "worldserver ne répond pas après $StartupTimeoutSeconds s"; $world.Kill(); Stop-Process -Id $auth.Id -Force; exit 1 }
Write-Step "worldserver prêt (PID $($world.Id)) sur 127.0.0.1:$($Layout.WorldPort). Console : Send-Console.ps1 ; arrêt : fichier $stopFlag"

# Boucle de service : transmet console.in, surveille stop et les processus.
$sent = 0
try {
    while (-not $world.HasExited) {
        if (Test-Path $stopFlag) { break }
        $lines = @(Get-Content $consoleIn -Encoding UTF8 -ErrorAction SilentlyContinue)
        if ($lines.Count -gt $sent) {
            foreach ($l in $lines[$sent..($lines.Count - 1)]) { if ($l.Trim()) { Write-Step "console> $l"; $world.StandardInput.WriteLine($l) } }
            $sent = $lines.Count
        }
        if ($auth.HasExited) { Write-Step "authserver s'est arrêté (code $($auth.ExitCode))" ; break }
        Start-Sleep -Milliseconds 500
    }
} finally {
    if (-not $world.HasExited) {
        Write-Step "Arrêt de worldserver (server shutdown 1)"
        try { $world.StandardInput.WriteLine('server shutdown 1'); $world.StandardInput.Flush(); $world.StandardInput.Close() } catch {}
        if (-not $world.WaitForExit(120000)) { Write-Step "worldserver ne répond pas : arrêt forcé"; $world.Kill() }
    }
    Write-Step "worldserver terminé (code $($world.ExitCode))"
    Unregister-Event -SourceIdentifier $ev1.Name; Unregister-Event -SourceIdentifier $ev2.Name
    $outWriter.Dispose()
    if (-not $auth.HasExited) { Stop-Process -Id $auth.Id -Force; Write-Step "authserver arrêté" }
    if ($StopDatabase) { & "$PSScriptRoot\Stop-Database.ps1" }
}
