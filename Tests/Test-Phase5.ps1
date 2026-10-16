<#
.SYNOPSIS
    Tests d'intégration phase 5 : sauvegarde, restauration, vérification, réparation, diagnostic, via le launcher
    en mode automatique (--do <action> --quit-after N). Le serveur de jeu doit être arrêté.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
$exe = Get-ProjectPath 'Launcher\target\release\vanilla-solo-launcher.exe'
$log = Get-ProjectPath 'Logs\launcher.log'
$bkDir = Get-ProjectPath 'UserData\Sauvegardes'
function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }
foreach ($p in $Layout.RealmdPort, $Layout.WorldPort) { if (Test-Port $p) { throw "Port $p occupé : arrêter le serveur de jeu d'abord" } }

$results = [ordered]@{}
function Run-Launcher { param([string[]]$Do, [int]$Quit = 40)
    $before = if (Test-Path $log) { (Get-Content $log | Measure-Object).Count } else { 0 }
    $p = Start-Process -FilePath $exe -ArgumentList (@('--do') + $Do + @('--quit-after', $Quit)) -WorkingDirectory (Split-Path $exe) -PassThru -Wait
    Start-Sleep -Seconds 2
    $lines = @(Get-Content $log -Encoding UTF8 | Select-Object -Skip $before)
    return ($lines -join "`n")
}

# Témoin : un compte supplémentaire créé directement en base, qui doit survivre aux opérations puis revenir après restauration.
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
Invoke-Sql -Database realmd -Query "DELETE FROM account WHERE username='TEMOIN'; INSERT INTO account (username, v, s) VALUES ('TEMOIN','00','00')"
$accountsBefore = [int](Invoke-Sql -Query "SELECT COUNT(*) FROM realmd.account")
Write-Step "comptes avant sauvegarde : $accountsBefore"

# 1. Sauvegarde
$t = Run-Launcher -Do @('backup')
$zip = Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip' -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$results['sauvegarde créée'] = ($t -match 'sauvegarde créée') -and $zip
if ($zip) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $z = [System.IO.Compression.ZipFile]::OpenRead($zip.FullName)
    $names = $z.Entries | ForEach-Object { $_.FullName }
    $json = (New-Object IO.StreamReader($z.GetEntry('backup.json').Open())).ReadToEnd() | ConvertFrom-Json
    $z.Dispose()
    $results['archive contient characters.sql, realmd.sql, backup.json'] = ($names -contains 'characters.sql') -and ($names -contains 'realmd.sql') -and ($names -contains 'backup.json')
    $results["manifeste : $($json.accounts) comptes = $accountsBefore"] = ([int]$json.accounts -eq $accountsBefore)
}
# La base avait été démarrée par ce script : le launcher doit l'utiliser telle quelle et NE PAS l'arrêter
# (correctif du 2026-10-15 : une base déjà en service n'est jamais considérée comme temporaire).
$results['MariaDB déjà en service : utilisée et laissée active'] = ($t -match 'MariaDB déjà en service') -and (Test-Port $Layout.MariaDbPort)
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null
$t = Run-Launcher -Do @('backup')
$results['sauvegarde avec base arrêtée : MariaDB temporaire arrêtée ensuite'] = ($t -match 'MariaDB temporaire arrêtée') -and -not (Test-Port $Layout.MariaDbPort)

# 2. Vérification des fichiers gérés (manifeste du dossier de développement régénéré d'abord : il change à chaque build)
& (Get-ProjectPath 'Build\New-FilesManifest.ps1') | Out-Null
$t = Run-Launcher -Do @('verify') -Quit 120
$results['vérification : 0 manquant, 0 altéré'] = ($t -match 'fichiers gérés : \d+ vérifiés, 0 manquants, 0 altérés')

# 3. Réparation : base logs supprimée + configuration générée effacée -> doivent être recréées, characters intact
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
Invoke-Sql -AsRoot -Query "DROP DATABASE IF EXISTS logs"
$charsBefore = [int](Invoke-Sql -Query "SELECT COUNT(*) FROM characters.characters")
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null
Remove-Item (Get-ProjectPath 'Server\Config\mangosd.conf') -ErrorAction SilentlyContinue
$t = Run-Launcher -Do @('repair') -Quit 150
$results['réparation : base logs réimportée'] = ($t -match 'bases réimportées : logs')
$results['réparation : mangosd.conf régénéré'] = (Test-Path (Get-ProjectPath 'Server\Config\mangosd.conf'))
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
$results['réparation : personnages intacts'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM characters.characters") -eq $charsBefore)
$results['réparation : base logs présente'] = ([int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name='logs'") -eq 1)

# 3b. Réimport de la base monde : une modification de la base mangos doit disparaître, les compléments HomeCraft revenir,
#     personnages et comptes rester intacts (base démarrée temporairement par le launcher).
Invoke-Sql -Database mangos -Query "UPDATE locales_quest SET Title_loc2='TEMOIN MONDE' WHERE entry=8856"
$temoinAccounts = [int](Invoke-Sql -Query "SELECT COUNT(*) FROM realmd.account")
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null
$t = Run-Launcher -Do @('reimportworld') -Quit 300
$results['réimport monde : terminé'] = ($t -match 'base monde réimportée')
$results['réimport monde : compléments journalisés'] = ($t -match 'import : complément auctionhousebot_solo.sql') -and ($t -match 'import : complément locales_quest_homecraft.sql')
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
$results['réimport monde : modification annulée, traduction HomeCraft revenue'] = ((Invoke-Sql -Query "SELECT Title_loc2 FROM mangos.locales_quest WHERE entry=8856") -eq 'Kits de survie dans le désert')
$results['réimport monde : personnages et comptes intacts'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM characters.characters") -eq $charsBefore) -and ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM realmd.account") -eq $temoinAccounts)

# 4. Restauration : on supprime le témoin, puis on restaure la sauvegarde -> il revient ; une sauvegarde de sécurité est créée
Invoke-Sql -Database realmd -Query "DELETE FROM account WHERE username='TEMOIN'"
$results['témoin supprimé avant restauration'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM realmd.account WHERE username='TEMOIN'") -eq 0)
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null
$nbBefore = (Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip').Count
$t = Run-Launcher -Do @('restore', $zip.FullName) -Quit 60
$results['restauration terminée'] = ($t -match 'restauration terminée')
$results['sauvegarde de sécurité créée avant restauration'] = ((Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip').Count -eq $nbBefore + 1)
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
$results['témoin revenu après restauration'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM realmd.account WHERE username='TEMOIN'") -eq 1)
Invoke-Sql -Database realmd -Query "DELETE FROM account WHERE username='TEMOIN'"
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null

# 5. Diagnostic
$t = Run-Launcher -Do @('diagnostic') -Quit 15
$diag = Get-ChildItem (Get-ProjectPath 'UserData') -Filter 'diagnostic-*.zip' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$results['diagnostic exporté'] = ($t -match 'diagnostic exporté') -and $diag
if ($diag) { $z=[System.IO.Compression.ZipFile]::OpenRead($diag.FullName); $conf=(New-Object IO.StreamReader($z.GetEntry('mangosd.conf').Open())).ReadToEnd(); $z.Dispose(); $results['diagnostic : mots de passe masqués'] = ($conf -match 'WorldDatabase.Info = "<masqué>"') -and -not ($conf -match ';3315;mangos;') }

$ok = $true
foreach ($k in $results.Keys) { Write-Step ("{0,-56} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
