<#
.SYNOPSIS
    Test d'intégration du launcher Rust : « HomeCraft Edition WoTLK.exe --start --quit-after N ».
    Pendant l'exécution : ouverture des ports 3306 (MySQL), 3724 (authserver), 8085 (worldserver), écoute sur
    127.0.0.1 seulement. Dès que le journal annonce « monde opérationnel », la fenêtre est fermée (WM_CLOSE, comme un
    clic sur la croix) après -WaitAfterReady secondes pour éprouver l'arrêt propre ; --quit-after reste la sécurité.
    Ensuite : journal (démarrage des trois services, arrêt propre, aucune ERREUR), ports libérés, aucun orphelin.
    Exige que rien ne tourne (ports libres, aucun worldserver/authserver/mysqld/launcher).
#>
[CmdletBinding()]
param(
    [int]$QuitAfter = 600,
    [int]$WaitAfterReady = 20,
    # Ne pas fermer la fenêtre : la sortie ne repose que sur --quit-after (contrôle du chemin de fermeture automatique).
    [switch]$NoClose
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common-Tests.ps1"

Assert-LauncherPresent
Assert-ServicesStopped
Set-TestSettings

$seen = @{}
$readyAt = $null
$closeSent = $false
$logStart = Get-LogLineCount
$tick = {
    param($p, $sw)
    foreach ($port in $Layout.DbPort, $Layout.AuthPort, $Layout.WorldPort) {
        if (-not $seen[$port] -and (Test-Port $port)) { $seen[$port] = [int]$sw.Elapsed.TotalSeconds; Write-Step "port $port ouvert à t+$($seen[$port]) s" }
    }
    if (-not $readyAt -and ((Get-LogSince $logStart) -join "`n") -match 'monde opérationnel') {
        $script:readyAt = [int]$sw.Elapsed.TotalSeconds
        Write-Step "monde opérationnel à t+$readyAt s"
        # Hors ligne : les sockets en écoute des processus du launcher ne sont que sur 127.0.0.1.
        $pids = @(Get-ServerProcesses | Select-Object -ExpandProperty Id)
        $listen = @(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object { $pids -contains $_.OwningProcess })
        $listen | ForEach-Object { Write-Step ("   écoute {0}:{1} (pid {2})" -f $_.LocalAddress, $_.LocalPort, $_.OwningProcess) }
        $script:listenOk = ($listen.Count -gt 0) -and -not ($listen | Where-Object { $_.LocalAddress -notin '127.0.0.1', '::1' })
        $script:listenPorts = @($listen | Select-Object -ExpandProperty LocalPort)
    }
    if ($readyAt -and -not $closeSent -and -not $NoClose -and $sw.Elapsed.TotalSeconds -ge ($readyAt + $WaitAfterReady)) {
        $script:closeSent = $true
        if (Close-LauncherWindow -ProcessId $p.Id) { Write-Step "fermeture de la fenêtre demandée (WM_CLOSE) à t+$([int]$sw.Elapsed.TotalSeconds) s" }
        else { Write-Step "fenêtre principale introuvable : on attend --quit-after" }
    }
}
$run = Invoke-Launcher -Arguments @('--start', '--quit-after', $QuitAfter) -TimeoutSeconds ($QuitAfter + 240) -OnTick $tick
$t = $run.Text

$checks = [ordered]@{
    'MySQL opérationnelle'                 = ($t -match 'MySQL opérationnelle')
    'authserver opérationnel'              = ($t -match 'authserver opérationnel')
    'monde opérationnel (World Initialized)' = ($t -match 'monde opérationnel') -and ($t -match 'World Initialized')
    'port MySQL ouvert pendant le test'     = [bool]$seen[$Layout.DbPort]
    'port 3724 ouvert pendant le test'     = [bool]$seen[$Layout.AuthPort]
    'port 8085 ouvert pendant le test'     = [bool]$seen[$Layout.WorldPort]
    'écoute uniquement sur 127.0.0.1'      = [bool]$listenOk
    'ports MySQL/3724/8085 en écoute'       = (($Layout.DbPort, $Layout.AuthPort, $Layout.WorldPort) | ForEach-Object { $listenPorts -contains $_ }) -notcontains $false
    'worldserver arrêté proprement'        = ($t -match 'worldserver arrêté proprement')
    'authserver arrêté'                    = ($t -match 'authserver arrêté')
    'MySQL arrêtée'                        = ($t -match 'MySQL arrêtée')
    'aucune ERREUR dans le journal'        = -not ($t -match 'ERREUR')
    'launcher terminé (code 0)'            = ($run.ExitCode -eq 0)
    'port MySQL libéré'                     = -not (Test-Port $Layout.DbPort)
    'port 3724 libéré'                     = -not (Test-Port $Layout.AuthPort)
    'port 8085 libéré'                     = -not (Test-Port $Layout.WorldPort)
    'aucun processus orphelin'             = ((Get-ServerProcesses).Count -eq 0)
    'playerbots.conf : CommandServerPort = 0' = [bool](Select-String -Path (Get-ProjectPath 'Server\acore\configs\modules\playerbots.conf') -Pattern '^\s*AiPlayerbot\.CommandServerPort\s*=\s*0\s*$' -Quiet)
}
$ok = Write-Results $checks
Write-Step "Journal du launcher ($($run.Lines.Count) lignes, hors console worldserver) :"
$run.Lines | Where-Object { $_ -notmatch '\[worldserver' } | Select-Object -Last 40 | ForEach-Object { "    $_" }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
