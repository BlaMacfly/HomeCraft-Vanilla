<#
.SYNOPSIS
    Tests de fiabilité du launcher en mode automatique (--do <action> --quit-after N) : sauvegarde (base déjà en service
    puis base arrêtée), vérification des fichiers gérés, réparation (conf régénérée, personnages intacts), restauration
    avec compte témoin (sauvegarde de sécurité), diagnostic (mots de passe masqués), ouverture directe d'un onglet (--tab).
    Porté de ../HomeCraft/Tests/Test-Phase5.ps1 (realmd/characters -> acore_auth/acore_characters).
    Exige que rien ne tourne (ports libres, aucun worldserver/authserver/mysqld/launcher).
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common-Tests.ps1"

Assert-LauncherPresent
Assert-ServicesStopped
Set-TestSettings
$bkDir = Get-ProjectPath 'UserData\Sauvegardes'
$results = [ordered]@{}
function Run-Do { param([string[]]$Do, [int]$Quit = 40) return (Invoke-Launcher -Arguments (@('--do') + $Do + @('--quit-after', $Quit)) -TimeoutSeconds ($Quit + 120)).Text }
function Count-Accounts { return [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM acore_auth.account WHERE username NOT LIKE 'RNDBOT%'") }
function Count-Witness { return [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM acore_auth.account WHERE username='TEMOIN'") }
function Count-Characters { return [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM acore_characters.characters") }

# Témoin : un compte créé directement en base (salt/verifier nuls : jamais connectable), qui doit survivre aux
# opérations puis revenir après restauration.
& (Get-ProjectPath 'Tools\Start-Database.ps1') | Out-Null
Invoke-Sql -AsRoot -Query "DELETE FROM acore_auth.account WHERE username='TEMOIN'; INSERT INTO acore_auth.account (username, salt, verifier) VALUES ('TEMOIN', UNHEX(REPEAT('00',32)), UNHEX(REPEAT('00',32)))"
$accountsBefore = Count-Accounts
$charsBefore = Count-Characters
Write-Step "avant sauvegarde : $accountsBefore comptes (hors bots), $charsBefore personnages"

# 1. Sauvegarde, base déjà en service (démarrée par ce script) : utilisée telle quelle et laissée active.
$t = Run-Do @('backup')
$zip = Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip' -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$results['sauvegarde créée'] = ($t -match 'sauvegarde créée') -and $zip
if ($zip) {
    $names = Get-ZipEntryNames $zip.FullName
    $json = (Read-ZipEntry $zip.FullName 'backup.json') | ConvertFrom-Json
    $results['archive : acore_characters.sql, acore_auth.sql, backup.json'] = ($names -contains 'acore_characters.sql') -and ($names -contains 'acore_auth.sql') -and ($names -contains 'backup.json')
    $results["manifeste : $($json.accounts) comptes = $accountsBefore (hors bots)"] = ([int]$json.accounts -eq $accountsBefore)
    $results['manifeste : settings.json joint'] = ($names -contains 'settings.json')
}
$results['MySQL déjà en service : utilisée et laissée active'] = ($t -match 'MySQL déjà en service') -and (Test-Port $Layout.DbPort) -and -not ($t -match 'MySQL temporaire arrêtée')

# 1b. Sauvegarde, base arrêtée : MySQL démarrée temporairement puis arrêtée.
& (Get-ProjectPath 'Tools\Stop-Database.ps1') | Out-Null
$t = Run-Do @('backup')
$results['base arrêtée : MySQL temporaire démarrée puis arrêtée'] = ($t -match 'MySQL démarrée temporairement') -and ($t -match 'MySQL temporaire arrêtée') -and -not (Test-Port $Layout.DbPort)
$results['base arrêtée : seconde sauvegarde créée'] = ($t -match 'sauvegarde créée')

# 2. Vérification des fichiers gérés (manifeste régénéré d'abord : il change à chaque build du launcher).
& (Get-ProjectPath 'Build\New-FilesManifest.ps1') | Out-Null
$t = Run-Do @('verify') -Quit 240
$results['vérification : 0 manquant, 0 altéré'] = ($t -match 'fichiers gérés : \d+ vérifiés, 0 manquants, 0 altérés')
$results['vérification : base arrêtée, laissée arrêtée'] = -not (Test-Port $Layout.DbPort)

# 3. Réparation : worldserver.conf effacé -> régénéré ; bases complètes (rien à réimporter) ; personnages intacts.
$worldConf = Get-ProjectPath 'Server\acore\configs\worldserver.conf'
Remove-Item $worldConf -ErrorAction SilentlyContinue
$t = Run-Do @('repair') -Quit 180
$results['réparation terminée'] = ($t -match 'réparation terminée')
$results['réparation : worldserver.conf régénéré'] = (Test-Path $worldConf)
$results['réparation : aucune base réimportée'] = -not ($t -match 'bases à réimporter')
$results['réparation : MySQL temporaire arrêtée ensuite'] = ($t -match 'MySQL temporaire arrêtée') -and -not (Test-Port $Layout.DbPort)
& (Get-ProjectPath 'Tools\Start-Database.ps1') | Out-Null
$results['réparation : personnages intacts'] = ((Count-Characters) -eq $charsBefore)
$results['réparation : témoin intact'] = ((Count-Witness) -eq 1)

# 4. Restauration : témoin supprimé, puis restauration de la première sauvegarde -> il revient ; sauvegarde de sécurité créée.
Invoke-Sql -AsRoot -Query "DELETE FROM acore_auth.account WHERE username='TEMOIN'"
$results['témoin supprimé avant restauration'] = ((Count-Witness) -eq 0)
& (Get-ProjectPath 'Tools\Stop-Database.ps1') | Out-Null
$nbBefore = (Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip').Count
$t = Run-Do @('restore', $zip.FullName) -Quit 120
$results['restauration terminée'] = ($t -match 'restauration terminée')
$results['sauvegarde de sécurité créée avant restauration'] = ((Get-ChildItem $bkDir -Filter 'sauvegarde-*.zip').Count -eq $nbBefore + 1)
$results['restauration : MySQL temporaire arrêtée ensuite'] = -not (Test-Port $Layout.DbPort)
& (Get-ProjectPath 'Tools\Start-Database.ps1') | Out-Null
$results['témoin revenu après restauration'] = ((Count-Witness) -eq 1)
$results['restauration : personnages intacts'] = ((Count-Characters) -eq $charsBefore)
Invoke-Sql -AsRoot -Query "DELETE FROM acore_auth.account WHERE username='TEMOIN'"
& (Get-ProjectPath 'Tools\Stop-Database.ps1') | Out-Null

# 5. Diagnostic : archive exportée, chaînes de connexion masquées dans les .conf joints.
$t = Run-Do @('diagnostic') -Quit 15
$diag = Get-ChildItem (Get-ProjectPath 'UserData') -Filter 'diagnostic-*.zip' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$results['diagnostic exporté'] = ($t -match 'diagnostic exporté') -and $diag
if ($diag) {
    $conf = Read-ZipEntry $diag.FullName 'worldserver.conf'
    $pb = Read-ZipEntry $diag.FullName 'playerbots.conf'
    $results['diagnostic : mots de passe masqués (worldserver.conf, playerbots.conf)'] = ($conf -match 'WorldDatabaseInfo = "<masqué>"') -and ($conf -match 'PlayerbotsDatabaseInfo = "<masqué>"') -and -not ($conf -match ';3315;acore;') -and -not ($pb -match ';3315;acore;')
    $entries = Get-ZipEntryNames $diag.FullName
    $results['diagnostic : launcher.log et etat.txt joints'] = ($entries -contains 'launcher.log') -and ($entries -contains 'etat.txt')
}

# 6. Ouverture directe d'un onglet (--tab), sans démarrer de service.
foreach ($tab in 'backups', 'bots', 'settings') {
    $run = Invoke-Launcher -Arguments @('--tab', $tab, '--quit-after', '5') -TimeoutSeconds 60
    $results["--tab $tab : fermeture normale, aucune ERREUR"] = ($run.ExitCode -eq 0) -and -not ($run.Text -match 'ERREUR') -and -not (Test-Port $Layout.DbPort)
}

# 7. Modules (onglet Modules, 05-Launcher-Rust.md §6) : bascule d'un module par --do modoff/modon, persistance de
#    l'état à travers la régénération des conf (--do repair repart du .conf existant), retour au modèle (--do modreset).
function Get-ConfValue { param([string]$File, [string]$Key) $m = Select-String -Path $File -Pattern ('^\s*' + [regex]::Escape($Key) + '\s*=\s*(\S+)') | Select-Object -First 1; if ($m) { return $m.Matches[0].Groups[1].Value } }
$junk = Get-ProjectPath 'Server\acore\configs\modules\JunkToGold.conf'
$ale = Get-ProjectPath 'Server\acore\configs\modules\mod_ale.conf'
$pb = Get-ProjectPath 'Server\acore\configs\modules\playerbots.conf'
$t = Run-Do @('modoff', 'mod-junk-to-gold') -Quit 8
$results['modoff : JunkToGold.Enable = 0 et journalisé'] = ($t -match 'module mod-junk-to-gold désactivé') -and ((Get-ConfValue $junk 'JunkToGold.Enable') -eq '0')
$t = Run-Do @('repair') -Quit 60
$results['repair : état du module conservé (JunkToGold.Enable = 0)'] = ($t -match 'réparation terminée') -and ((Get-ConfValue $junk 'JunkToGold.Enable') -eq '0')
$results['repair : CommandServerPort = 0 réappliqué dans playerbots.conf'] = ((Get-ConfValue $pb 'AiPlayerbot.CommandServerPort') -eq '0')
$t = Run-Do @('modon', 'mod-junk-to-gold') -Quit 8
$results['modon : JunkToGold.Enable = 1'] = ($t -match 'module mod-junk-to-gold activé') -and ((Get-ConfValue $junk 'JunkToGold.Enable') -eq '1')
$t = Run-Do @('modoff', 'mod-ale') -Quit 8
$results['modoff : ALE.Enabled = false'] = ((Get-ConfValue $ale 'ALE.Enabled') -eq 'false')
$t = Run-Do @('modreset', 'mod-ale') -Quit 8
$results['modreset : mod_ale.conf identique au modèle'] = ($t -match 'configuration rétablie depuis le modèle') -and ((Get-FileHash $ale).Hash -eq (Get-FileHash "$ale.dist").Hash)
$run = Invoke-Launcher -Arguments @('--tab', 'modules', '--quit-after', '5') -TimeoutSeconds 60
$results['--tab modules : fermeture normale, aucune ERREUR'] = ($run.ExitCode -eq 0) -and -not ($run.Text -match 'ERREUR')

$results['aucun processus orphelin'] = ((Get-ServerProcesses).Count -eq 0)
$ok = Write-Results $results
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
