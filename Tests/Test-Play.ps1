<#
.SYNOPSIS
    Test de bout en bout du bouton JOUER : « HomeCraft Edition WoTLK.exe --play » démarre les trois services, configure
    le client de Client/ (realmlist 127.0.0.1, Config.wtf) et lance le jeu (client 3.3.5a : wowdecomp WowClient64.exe en
    priorité, sinon Wow.exe). Le test capture la fenêtre du jeu (Logs/tests/jeu-<horodatage>.png), ferme le jeu
    (le launcher doit le détecter et faire sa sauvegarde automatique), puis ferme le launcher (arrêt propre).
    Ne se connecte pas au royaume : l'identification reste à faire à la main.
    Exige que rien ne tourne (ports libres, aucun worldserver/authserver/mysqld/launcher) et un client dans Client/
    (Tools/New-ClientMirror.ps1).
#>
[CmdletBinding()]
param(
    [int]$GameSeconds = 40,
    [int]$QuitAfter = 900
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common-Tests.ps1"

Assert-LauncherPresent
Assert-ServicesStopped
Set-TestSettings
$client = Get-ProjectPath 'Client'
$gameNames = @('Wow64', 'WowClient64', 'Wow32', 'WowClient32', 'Wow')
if (Get-Process -Name $gameNames -ErrorAction SilentlyContinue) { throw "Un client WoW tourne déjà : le fermer d'abord" }
$exe = @('Wow64.exe', 'WowClient64.exe', 'Wow32.exe', 'WowClient32.exe', 'Wow.exe') | Where-Object { Test-Path (Join-Path $client $_) } | Select-Object -First 1
if (-not $exe) { throw "Aucun client dans $client (Tools/New-ClientMirror.ps1 -Source <client 3.3.5a>)" }
$locale = (Get-ChildItem (Join-Path $client 'Data') -Directory | Where-Object { Test-Path (Join-Path $_.FullName "locale-$($_.Name).MPQ") } | Select-Object -First 1).Name
$shot = Get-ProjectPath ("Logs\tests\jeu-{0}.png" -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Force (Split-Path $shot) | Out-Null
$bkDir = Get-ProjectPath 'UserData\Sauvegardes'
$nbBk = @(Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip' -ErrorAction SilentlyContinue).Count

$st = @{ Game = $null; GameAt = $null; Title = $null; Killed = $false; KilledAt = $null; CloseSent = $false; LogStart = (Get-LogLineCount) }
$tick = {
    param($p, $sw)
    if (-not $st.Game) {
        $g = Get-Process -Name $gameNames -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($client, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
        # Chemin mémorisé tout de suite : Process.Path n'est plus lisible une fois le processus terminé.
        if ($g) { $st.Game = $g; $st.GamePath = $g.Path; $st.GameAt = [int]$sw.Elapsed.TotalSeconds; Write-Step "jeu démarré à t+$($st.GameAt) s : $($g.Path) (PID $($g.Id))" }
    } elseif (-not $st.Killed -and $sw.Elapsed.TotalSeconds -ge ($st.GameAt + $GameSeconds)) {
        $st.Title = Save-WindowShot -ProcessId $st.Game.Id -Path $shot
        Write-Step "capture de la fenêtre « $($st.Title) » : $shot"
        Stop-Process -Id $st.Game.Id -Force
        $st.Killed = $true; $st.KilledAt = [int]$sw.Elapsed.TotalSeconds
        Write-Step "jeu fermé par le test à t+$($st.KilledAt) s"
    } elseif ($st.Killed -and -not $st.CloseSent) {
        $t = (Get-LogSince $st.LogStart) -join "`n"
        if (($t -match 'sauvegarde créée') -or $sw.Elapsed.TotalSeconds -ge ($st.KilledAt + 60)) {
            $st.CloseSent = $true
            [void](Close-LauncherWindow -ProcessId $p.Id)
            Write-Step "fermeture du launcher demandée à t+$([int]$sw.Elapsed.TotalSeconds) s"
        }
    }
}
$run = Invoke-Launcher -Arguments @('--play', '--quit-after', $QuitAfter) -TimeoutSeconds ($QuitAfter + 240) -OnTick $tick
$t = $run.Text
$rl = Get-Content (Join-Path $client "Data\$locale\realmlist.wtf") -Raw
$cfg = Get-Content (Join-Path $client 'WTF\Config.wtf') -Raw
$checks = [ordered]@{
    "client détecté ($exe, 3.3.5.12340, aucun MPQ manquant)" = ($t -match 'client 3\.3\.5\.12340.*manques : 0')
    'monde opérationnel'                     = ($t -match 'monde opérationnel')
    'jeu lancé par le launcher'              = ($t -match 'jeu lancé') -and [bool]$st.Game
    "processus du jeu = Client\$exe"         = ($st.GamePath -ieq (Join-Path $client $exe))
    'fenêtre du jeu ouverte (capture)'       = [bool]$st.Title -and (Test-Path $shot)
    'realmlist.wtf -> 127.0.0.1'             = ($rl -match 'set realmlist 127\.0\.0\.1')
    'Config.wtf : realmList 127.0.0.1'       = ($cfg -match 'SET realmList "127\.0\.0\.1"')
    'fin du jeu détectée'                    = ($t -match 'jeu terminé')
    'sauvegarde automatique à la fin du jeu' = ($t -match 'sauvegarde créée') -and (@(Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip').Count -eq $nbBk + 1)
    'worldserver arrêté proprement'          = ($t -match 'worldserver arrêté proprement')
    'MySQL arrêtée'                          = ($t -match 'MySQL arrêtée')
    'aucune ERREUR dans le journal'          = -not ($t -match 'ERREUR')
    'launcher terminé (code 0)'              = ($run.ExitCode -eq 0)
    'aucun processus orphelin (serveurs, jeu)' = ((Get-ServerProcesses).Count -eq 0) -and -not (Get-Process -Name $gameNames -ErrorAction SilentlyContinue)
}
$ok = Write-Results $checks
Write-Step "Journal du launcher (hors console worldserver) :"
$run.Lines | Where-Object { $_ -notmatch '\[worldserver' } | Select-Object -Last 25 | ForEach-Object { "    $_" }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
