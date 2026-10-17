<#
.SYNOPSIS
    Tests d'intégration de fiabilité (HomeCraft Edition BC) : sauvegarde, vérification, réparation, réimport de la base
    monde, restauration avec compte témoin, diagnostic, via le launcher en mode automatique (--do <action> --quit-after N).
    Adapté de Tests/Test-Phase5.ps1 du Vanilla : bases tbc*, dumps tbccharacters.sql / tbcrealmd.sql, compléments
    00_homecraft_schema_locales_gameobject.sql et 50_homecraft_frFR.sql.
    Le serveur de jeu doit être arrêté ; ne jamais lancer pendant qu'un launcher de l'utilisateur tourne.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
$exe = Get-ProjectPath 'HomeCraft Edition BC.exe'
$log = Get-ProjectPath 'Logs\launcher.log'
$bkDir = Get-ProjectPath 'UserData\Sauvegardes'
foreach ($p in $Layout.RealmdPort, $Layout.WorldPort) { if (Test-Port $p) { throw "Port $p occupé : arrêter le serveur de jeu d'abord" } }
foreach ($n in 'HomeCraft Edition Vanilla', 'HomeCraft Edition WoTLK', 'HomeCraft Edition BC') { if (Get-Process $n -ErrorAction SilentlyContinue) { throw "$n est en cours d'exécution : test refusé" } }

$results = [ordered]@{}
function Run-Launcher { param([string[]]$Do, [int]$Quit = 40)
    $before = if (Test-Path $log) { (Get-Content $log | Measure-Object).Count } else { 0 }
    # Sondage plutôt que -Wait (qui attendrait aussi d'éventuels processus enfants).
    $p = Start-Process -FilePath $exe -ArgumentList (@('--do') + $Do + @('--quit-after', $Quit)) -PassThru
    while (-not $p.HasExited) { Start-Sleep -Milliseconds 500 }
    Start-Sleep -Seconds 2
    $lines = @(Get-Content $log -Encoding UTF8 | Select-Object -Skip $before)
    return ($lines -join "`n")
}
$R = $Databases.realmd; $C = $Databases.characters; $W = $Databases.world; $L = $Databases.logs

# Témoin : un compte créé directement en base, qui doit survivre aux opérations puis revenir après restauration.
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
Invoke-Sql -Database $R -Query "DELETE FROM account WHERE username='TEMOIN'; INSERT INTO account (username, v, s, expansion) VALUES ('TEMOIN','00','00',1)"
$accountsBefore = [int](Invoke-Sql -Query "SELECT COUNT(*) FROM $R.account")
Write-Step "comptes avant sauvegarde : $accountsBefore"

# 1. Sauvegarde (base déjà en service : utilisée, jamais arrêtée par le launcher)
$t = Run-Launcher -Do @('backup')
$zip = Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip' -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$results['sauvegarde créée'] = ($t -match 'sauvegarde créée') -and $zip
if ($zip) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $z = [System.IO.Compression.ZipFile]::OpenRead($zip.FullName)
    $names = $z.Entries | ForEach-Object { $_.FullName }
    $json = (New-Object IO.StreamReader($z.GetEntry('backup.json').Open())).ReadToEnd() | ConvertFrom-Json
    $z.Dispose()
    $results['archive : tbccharacters.sql, tbcrealmd.sql, backup.json'] = ($names -contains 'tbccharacters.sql') -and ($names -contains 'tbcrealmd.sql') -and ($names -contains 'backup.json')
    $results["manifeste : $($json.accounts) comptes = $accountsBefore"] = ([int]$json.accounts -eq $accountsBefore)
}
$results['MariaDB déjà en service : utilisée et laissée active'] = ($t -match 'MariaDB déjà en service') -and (Test-Port $Layout.MariaDbPort)
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null
$t = Run-Launcher -Do @('backup')
$results['sauvegarde base arrêtée : MariaDB temporaire arrêtée ensuite'] = ($t -match 'MariaDB temporaire arrêtée') -and -not (Test-Port $Layout.MariaDbPort)

# 2. Vérification des fichiers gérés (manifeste régénéré d'abord : il change à chaque build)
& (Get-ProjectPath 'Build\New-FilesManifest.ps1') | Out-Null
$t = Run-Launcher -Do @('verify') -Quit 180
$results['vérification : 0 manquant, 0 altéré'] = ($t -match 'fichiers gérés : \d+ vérifiés, 0 manquants, 0 altérés')

# 3. Réparation : base logs supprimée + configuration générée effacée -> recréées, personnages intacts
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
Invoke-Sql -AsRoot -Query "DROP DATABASE IF EXISTS $L"
$charsBefore = [int](Invoke-Sql -Query "SELECT COUNT(*) FROM $C.characters")
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null
Remove-Item (Get-ProjectPath 'Server\Config\mangosd.conf') -ErrorAction SilentlyContinue
$t = Run-Launcher -Do @('repair') -Quit 180
$results["réparation : base $L réimportée"] = ($t -match "bases réimportées : $L")
$results['réparation : mangosd.conf régénéré'] = (Test-Path (Get-ProjectPath 'Server\Config\mangosd.conf'))
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
$results['réparation : personnages intacts'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM $C.characters") -eq $charsBefore)
$results["réparation : base $L présente"] = ([int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name='$L'") -eq 1)

# 3b. Réimport de la base monde : modification annulée, compléments HomeCraft rejoués (correctif de schéma + français),
#     personnages et comptes intacts. Quête 11551 : titre français venu du complément (absent de TBC-DB).
Invoke-Sql -Database $W -Query "UPDATE locales_quest SET Title_loc2='TEMOIN MONDE' WHERE entry=11551"
$temoinAccounts = [int](Invoke-Sql -Query "SELECT COUNT(*) FROM $R.account")
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null
$t = Run-Launcher -Do @('reimportworld') -Quit 400
$results['réimport monde : terminé'] = ($t -match 'base monde réimportée')
$results['réimport monde : compléments journalisés'] = ($t -match 'import : complément 00_homecraft_schema_locales_gameobject.sql') -and ($t -match 'import : complément 50_homecraft_frFR.sql')
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
$results['réimport monde : traduction HomeCraft revenue'] = ((Invoke-Sql -Query "SELECT Title_loc2 FROM $W.locales_quest WHERE entry=11551") -eq 'Agamath, la première porte')
$results['réimport monde : schéma locales_gameobject corrigé'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM information_schema.columns WHERE table_schema='$W' AND table_name='locales_gameobject' AND column_name IN ('opening_text_loc2','closing_text_loc2')") -eq 2)
$results['réimport monde : messages serveur en français'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM $W.mangos_string WHERE COALESCE(content_loc2,'')<>''") -gt 600)
$results['réimport monde : personnages et comptes intacts'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM $C.characters") -eq $charsBefore) -and ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM $R.account") -eq $temoinAccounts)

# 4. Restauration : témoin supprimé, puis sauvegarde restaurée -> il revient ; sauvegarde de sécurité créée avant
Invoke-Sql -Database $R -Query "DELETE FROM account WHERE username='TEMOIN'"
$results['témoin supprimé avant restauration'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM $R.account WHERE username='TEMOIN'") -eq 0)
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null
$nbBefore = (Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip').Count
# Guillemets obligatoires : Start-Process ne cite pas les arguments, et le chemin du projet contient un espace.
$t = Run-Launcher -Do @('restore', "`"$($zip.FullName)`"") -Quit 120
$results['restauration terminée'] = ($t -match 'restauration terminée')
$results['sauvegarde de sécurité créée avant restauration'] = ((Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip').Count -eq $nbBefore + 1)
& (Get-ProjectPath 'Tools\Start-MariaDB.ps1') | Out-Null
$results['témoin revenu après restauration'] = ([int](Invoke-Sql -Query "SELECT COUNT(*) FROM $R.account WHERE username='TEMOIN'") -eq 1)
Invoke-Sql -Database $R -Query "DELETE FROM account WHERE username='TEMOIN'"
& (Get-ProjectPath 'Tools\Stop-MariaDB.ps1') | Out-Null

# 5. Diagnostic
$t = Run-Launcher -Do @('diagnostic') -Quit 20
$diag = Get-ChildItem (Get-ProjectPath 'UserData') -Filter 'diagnostic-*.zip' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$results['diagnostic exporté'] = ($t -match 'diagnostic exporté') -and $diag
if ($diag) {
    $z = [System.IO.Compression.ZipFile]::OpenRead($diag.FullName)
    $conf = (New-Object IO.StreamReader($z.GetEntry('mangosd.conf').Open())).ReadToEnd()
    $entries = $z.Entries | ForEach-Object { $_.FullName }
    $z.Dispose()
    $results['diagnostic : mots de passe masqués'] = ($conf -match 'WorldDatabaseInfo = "<masqué>"') -and -not ($conf -match ";$($Layout.MariaDbPort);")
    $results['diagnostic : aiplayerbot.conf et rapports de plantage'] = ($entries -contains 'aiplayerbot.conf') -and (@($entries | Where-Object { $_ -like 'Crashes/*' }).Count -gt 0)
}
$results['aucun processus serveur restant'] = -not (Get-Process mangosd, realmd, mariadbd -ErrorAction SilentlyContinue)

$ok = $true
foreach ($k in $results.Keys) { Write-Step ("{0,-60} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
