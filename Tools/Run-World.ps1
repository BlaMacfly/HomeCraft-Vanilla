<#
.SYNOPSIS
    Harnais de développement : démarre MariaDB, realmd et mangosd (CMaNGOS-tbc), puis reste en vie.
    - Les lignes ajoutées à Logs/server/console.in sont transmises à la console de mangosd (Send-Console.ps1).
    - La sortie de mangosd est copiée dans Logs/server/mangosd.console.log.
    - Créer le fichier Logs/server/stop (ou Ctrl+C) arrête proprement : « server shutdown 0 » à mangosd puis fermeture
      de stdin, puis realmd, puis MariaDB (-StopDatabase).
    Préfigure la machine à états du launcher ; n'est pas destiné à l'utilisateur final.
#>
[CmdletBinding()]
param(
    [int]$StartupTimeoutSeconds = 600,
    [int]$RandomBots = 50,
    [switch]$StopDatabase
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

& "$PSScriptRoot\Start-MariaDB.ps1"
& "$PSScriptRoot\New-ServerConfig.ps1" -RandomBots $RandomBots
# Royaume : seul le build 8606 est accepté (realmbuilds vide = royaume affiché hors ligne pour certains clients).
Invoke-Sql -Database $Databases.realmd -Query "INSERT INTO realmlist (id,name,address,port,icon,realmflags,timezone,allowedSecurityLevel,population,realmbuilds) VALUES (1,'$($Layout.RealmName)','127.0.0.1',$($Layout.WorldPort),0,0,1,0,0,'$($Layout.ClientBuild)') ON DUPLICATE KEY UPDATE name='$($Layout.RealmName)', address='127.0.0.1', port=$($Layout.WorldPort), icon=0, realmbuilds='$($Layout.ClientBuild)'"

$bin = $Layout.ServerBin
$cfg = $Layout.ServerConfig
$srvLog = Join-Path $Layout.Logs 'server'
New-Item -ItemType Directory -Force $srvLog | Out-Null
$consoleIn = Join-Path $srvLog 'console.in'
$consoleOut = Join-Path $srvLog 'mangosd.console.log'
$stopFlag = Join-Path $srvLog 'stop'
Remove-Item -Force $stopFlag, $consoleIn -ErrorAction SilentlyContinue
New-Item -ItemType File -Force $consoleIn | Out-Null

if (Test-Port $Layout.RealmdPort) { throw "Le port $($Layout.RealmdPort) est déjà utilisé : un realmd tourne déjà ?" }
if (Test-Port $Layout.WorldPort)  { throw "Le port $($Layout.WorldPort) est déjà utilisé : un mangosd tourne déjà ?" }

Write-Step "Démarrage realmd"
$realmd = Start-Process -FilePath (Join-Path $bin 'realmd.exe') -ArgumentList "-c `"$cfg\realmd.conf`"" -WorkingDirectory $bin -WindowStyle Hidden -PassThru -RedirectStandardOutput "$srvLog\realmd.stdout.log" -RedirectStandardError "$srvLog\realmd.stderr.log"
$deadline = (Get-Date).AddSeconds(30)
while (-not (Test-Port $Layout.RealmdPort) -and (Get-Date) -lt $deadline -and -not $realmd.HasExited) { Start-Sleep -Milliseconds 300 }
if (-not (Test-Port $Layout.RealmdPort)) { throw "realmd ne répond pas sur $($Layout.RealmdPort) (voir $srvLog\realmd.stdout.log)" }
Write-Step "realmd prêt (PID $($realmd.Id)) sur 127.0.0.1:$($Layout.RealmdPort)"

Write-Step "Démarrage mangosd"
$psi = New-Object Diagnostics.ProcessStartInfo
$psi.FileName = Join-Path $bin 'mangosd.exe'
# Dossier courant = Server/mangos-tbc : legacy.dll (OpenSSL) y est cherché ; anticheat désactivé par l'environnement.
$psi.Arguments = "-c `"$cfg\mangosd.conf`" -a `"$cfg\ahbot.conf`" -p `"$cfg\aiplayerbot.conf`""
$psi.WorkingDirectory = $bin
foreach ($k in 'Anticheat_Enable', 'Anticheat_Warden_Enable', 'Anticheat_Antispam_Enable') { $psi.EnvironmentVariables[$k] = '0' }
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.CreateNoWindow = $true
$psi.StandardOutputEncoding = [Text.Encoding]::UTF8
$psi.StandardErrorEncoding  = [Text.Encoding]::UTF8
# Les DLL (libmysql, OpenSSL) sont à côté de mangosd.exe : le dossier de l'exe est prioritaire dans la recherche Windows.
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
        Write-Step "Arrêt de mangosd (server shutdown 0)"
        # Fermer stdin est indispensable : le thread console de mangosd ne se termine qu'à la fin de l'entrée standard.
        try { $mangosd.StandardInput.WriteLine('server shutdown 0'); $mangosd.StandardInput.Flush(); Start-Sleep -Seconds 2; $mangosd.StandardInput.Close() } catch {}
        if (-not $mangosd.WaitForExit(90000)) { Write-Step "mangosd ne répond pas : arrêt forcé"; $mangosd.Kill() }
    }
    Write-Step "mangosd terminé (code $($mangosd.ExitCode))"
    Unregister-Event -SourceIdentifier $ev1.Name; Unregister-Event -SourceIdentifier $ev2.Name
    $outWriter.Dispose()
    if (-not $realmd.HasExited) { Stop-Process -Id $realmd.Id -Force; Write-Step "realmd arrêté" }
    if ($StopDatabase) { & "$PSScriptRoot\Stop-MariaDB.ps1" }
}
