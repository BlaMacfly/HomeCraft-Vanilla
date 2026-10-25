<#
.SYNOPSIS
    Test d'intégration de bout en bout sur une copie du package (dossier temporaire), édition CMaNGOS + playerbots :
    premier démarrage (import des bases, création des bots aléatoires), création de compte, suppression de compte hors
    ligne sans toucher aux données des autres personnages, vie des bots (déplacements, expérience), suppression des
    bots serveur en marche (arrêt et relance automatiques), réglages appliqués à chaud, arrêt sans processus résiduel.
.DESCRIPTION
    Le package doit avoir été construit (Build\New-Package.ps1). Les données du serveur (format CMaNGOS : dbc, maps,
    vmaps, mmaps, avec les marqueurs .cmangos-*) et les archives du client sont des liens physiques : rien n'y est
    écrit. Le dossier de développement contient les données de l'édition VMaNGOS : celles de CMaNGOS sont prises dans
    -DataDir (par défaut l'extraction de test %TEMP%\hc-cm\Server\Data).
    Ne pas lancer pendant que le launcher de l'utilisateur tourne (mêmes ports).
#>
[CmdletBinding()]
param(
    [string]$Root = (Join-Path $env:TEMP 'hc-integration'),
    [string]$DataDir = (Join-Path $env:TEMP 'hc-cm\Server\Data'),
    [switch]$Keep
)
$ErrorActionPreference = 'Stop'
$dev = Split-Path -Parent $PSScriptRoot
$version = ((Get-Content (Join-Path $dev 'Launcher\Cargo.toml')) -match '^version' | Select-Object -First 1) -replace '.*"(.*)".*', '$1'
$pkg = Join-Path $dev "Build\package\HomeCraft-Edition-Vanilla-v$version"
if (-not (Test-Path $pkg)) { throw "package v$version absent : lancer Build\New-Package.ps1 d'abord" }
foreach ($m in '.cmangos-dbc-maps', '.cmangos-vmaps', 'mmaps\.cmangos-mmaps') { if (-not (Test-Path (Join-Path $DataDir $m))) { throw "données CMaNGOS incomplètes dans $DataDir ($m absent)" } }
if (Get-Process -Name 'HomeCraft Edition Vanilla', 'vanilla-solo-launcher' -ErrorAction SilentlyContinue) { throw "un launcher HomeCraft est ouvert : le fermer avant le test" }
function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }
foreach ($p in 3315, 3724, 8085) { if (Test-Port $p) { throw "port $p occupé : arrêter le serveur HomeCraft d'abord" } }
function Step { param([string]$m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }

Add-Type -Namespace HcTest -Name Native -MemberDefinition '[DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] public static extern bool CreateHardLink(string lpFileName, string lpExistingFileName, IntPtr lpSecurityAttributes);'
function Link-Tree { param([string]$Src, [string]$Dst, [string[]]$SkipDirs = @(), [switch]$CopySmall)
    Get-ChildItem $Src -Recurse -File -Force | Where-Object { $rel = $_.FullName.Substring($Src.Length).TrimStart('\'); -not ($SkipDirs | Where-Object { $rel -like "$_\*" }) } | ForEach-Object {
        $to = Join-Path $Dst $_.FullName.Substring($Src.Length).TrimStart('\')
        $dir = Split-Path $to
        if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force $dir | Out-Null }
        # Fichiers du client autres que les archives : copiés (le launcher peut les réécrire, ex. Config.wtf).
        if ($CopySmall -and $_.Extension -ne '.MPQ') { Copy-Item $_.FullName $to }
        elseif (-not [HcTest.Native]::CreateHardLink($to, $_.FullName, [IntPtr]::Zero)) { Copy-Item $_.FullName $to }
    }
}
function Write-Settings { param([string]$Json) [IO.File]::WriteAllText((Join-Path $Root 'UserData\settings.json'), $Json, (New-Object Text.UTF8Encoding $false)) }

# ---- Installation de test
if (Test-Path $Root) {
    if (-not (Test-Path (Join-Path $Root '.hc-integration'))) { throw "$Root existe et n'est pas un dossier de test : refus de le supprimer" }
    Remove-Item $Root -Recurse -Force
}
Step "copie du package v$version vers $Root"
robocopy $pkg $Root /E /NFL /NDL /NJH /NJS /NP | Out-Null
if ($LASTEXITCODE -ge 8) { throw "robocopy : code $LASTEXITCODE" }
Set-Content (Join-Path $Root '.hc-integration') 'dossier de test' -Encoding UTF8
Step "liens physiques vers les données CMaNGOS ($DataDir) et le client du dossier de développement"
# Sans mmaps d'abord, comme une installation neuve (elles ne sont générées qu'après le premier démarrage) : en 0.10.0,
# mmap.enabled = 1 sans fichier .mmap arrêtait mangosd au chargement des cartes. Ajoutées avant la phase 3.
Link-Tree $DataDir (Join-Path $Root 'Server\Data') -SkipDirs @('Buildings', 'mmaps-generation', 'extraction-client', 'mmaps')
Link-Tree (Join-Path $dev 'Client') (Join-Path $Root 'Client') -SkipDirs @('WTF', 'Cache', 'Logs', 'Screenshots') -CopySmall
New-Item -ItemType Directory -Force (Join-Path $Root 'UserData') | Out-Null
Write-Settings '{"windowed":true,"auto_mmaps":true,"auto_backup":false,"conf":{"AiPlayerbot.MinRandomBots":"20","AiPlayerbot.MaxRandomBots":"30"}}'

$exe = Join-Path $Root 'HomeCraft Edition Vanilla.exe'
$log = Join-Path $Root 'Logs\launcher.log'
$scripts = Join-Path $Root 'Tools\Scripts'
. (Join-Path $scripts 'Common.ps1')
$results = [ordered]@{}
$q = { param($sql) [int]("$(Invoke-Sql -AsRoot -Query $sql)".Trim()) }
$botAccounts = "realmd.account a WHERE a.username REGEXP '^RNDBOT[0-9]+`$'"
$botChars = "FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE a.username REGEXP '^RNDBOT[0-9]+`$'"

function Remaining-Processes {
    @(Get-Process mangosd, realmd, mariadbd, MoveMapGen -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase) }).Count
}
# Lance le launcher, attend que le journal contienne $Until (ou ERREUR), puis le ferme comme l'utilisateur.
function Run-Launcher { param([string[]]$LauncherArgs, [string]$Until, [int]$TimeoutSec = 900, [int]$ExtraSec = 0)
    $mark = if (Test-Path $log) { @(Get-Content $log).Count } else { 0 }
    $p = Start-Process -FilePath $exe -ArgumentList ($LauncherArgs + @('--quit-after', ($TimeoutSec + $ExtraSec + 60))) -WorkingDirectory $Root -PassThru
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $text = ''
    while ($sw.Elapsed.TotalSeconds -lt $TimeoutSec -and -not $p.HasExited) {
        Start-Sleep 2
        $text = (@(Get-Content $log -Encoding UTF8 -ErrorAction SilentlyContinue | Select-Object -Skip $mark) -join "`n")
        if ($text -match $Until -or $text -match 'ERREUR') { break }
    }
    if ($ExtraSec -gt 0 -and $text -notmatch 'ERREUR') { Start-Sleep $ExtraSec }
    $script:lastProcess = $p
    return (@(Get-Content $log -Encoding UTF8 -ErrorAction SilentlyContinue | Select-Object -Skip $mark) -join "`n")
}
function Close-Launcher {
    $p = $script:lastProcess
    # Demande de fermeture répétée toutes les 5 s : un seul CloseMainWindow est parfois perdu (fenêtre pas encore
    # reconnue comme fenêtre principale), le launcher restait alors ouvert jusqu'à l'arrêt forcé.
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while (-not $p.HasExited -and $sw.Elapsed.TotalSeconds -lt 180) {
        $p.Refresh()
        $p.CloseMainWindow() | Out-Null
        $p.WaitForExit(5000) | Out-Null
    }
    if (-not $p.HasExited) { $p.Kill(); return $false }
    Start-Sleep 2
    return $true
}

# Erreur imprévue (base injoignable après un échec du serveur…) : le launcher de test est fermé avant de sortir,
# sinon il reste ouvert et le test suivant refuse de démarrer.
trap {
    if ($script:lastProcess -and -not $script:lastProcess.HasExited) { $script:lastProcess.CloseMainWindow() | Out-Null; $script:lastProcess.WaitForExit(180000) | Out-Null }
    Step "TEST INTERROMPU : $_ (dossier conservé : $Root)"
    exit 1
}

# ---- 1. Premier démarrage : import, bots aléatoires créés par le serveur, compte joueur MJ
Step "1. premier démarrage (création des bots aléatoires) et création du compte AUDITA"
$t = Run-Launcher -LauncherArgs @('--start', '--do', 'acccreate', 'AUDITA', 'Motdepasse1', '3') -Until 'compte AUDITA créé|création du compte impossible' -ExtraSec 90
$results['1. monde opérationnel au premier démarrage (sans mmaps)'] = $t -match 'monde opérationnel'
$results['1. mmaps désactivées tant qu''elles manquent'] = [bool](Select-String -Path (Join-Path $Root 'Server\Config\mangosd.conf') -Pattern '^mmap\.enabled = 0' -Quiet)
# En 0.10.1, une connexion d'essai trop lente pendant la génération des mmaps faisait tout arrêter au bout de 20 s.
$results['1. génération des mmaps lancée en arrière-plan'] = $t -match 'mmaps absentes : génération en arrière-plan'
$results['1. serveur resté en service 90 s pendant la génération'] = $t -notmatch 'service interrompu'
$results['1. MoveMapGen en priorité réduite'] = [bool](Get-Process MoveMapGen -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase) -and $_.PriorityClass -eq 'BelowNormal' })
$results['1. compte créé avec le niveau 3'] = $t -match 'compte AUDITA créé \(niveau 3\)'
$results['1. niveau 3 en base'] = (& $q "SELECT gmlevel FROM realmd.account WHERE username='AUDITA'") -eq 3
$builds = "$(Invoke-Sql -AsRoot -Query "SELECT realmbuilds FROM realmd.realmlist WHERE id=1")".Trim()
$results["1. build client 5880 dans le royaume ($builds)"] = ($builds -split '\s+') -contains '5880'
$results['1. aucun compte de démonstration'] = (& $q "SELECT COUNT(*) FROM realmd.account WHERE username IN ('ADMINISTRATOR','GAMEMASTER','MODERATOR','PLAYER')") -eq 0
$accounts = & $q "SELECT COUNT(*) FROM $botAccounts"
$results["1. comptes de bots créés par le serveur ($accounts, attendu 4 pour 30 bots)"] = $accounts -eq 4
$online = & $q "SELECT COUNT(*) $botChars AND c.online=1"
$results["1. bots aléatoires connectés ($online)"] = $online -ge 15
$results['1. aiplayerbot.conf lu (option -p)'] = -not (Select-String -Path (Join-Path $Root 'Logs\server\mangosd.console.log') -Pattern 'Unable to open configuration file aiplayerbot.conf' -SimpleMatch -Quiet)
$results['1. arrêt propre à la fermeture'] = Close-Launcher
$results['1. aucun processus serveur restant'] = (Remaining-Processes) -eq 0
$results['1. mot de passe absent du journal'] = -not (Select-String -Path $log -Pattern 'Motdepasse1' -SimpleMatch -Quiet)

# ---- 2. Suppression de compte serveur arrêté : seules les données du compte disparaissent
Step "2. suppression hors ligne du compte AUDITA, données d'un autre compte intactes"
& (Join-Path $scripts 'Start-MariaDB.ps1') | Out-Null
$cols = 'guid, account, name, race, class, gender, level, position_x, position_y, position_z, map, orientation, taximask, exploredZones, equipmentCache, taxi_path, zone'
$row = { param($g, $acc, $n) "($g,(SELECT id FROM realmd.account WHERE username='$acc'),'$n',1,1,0,10,-8949,-132,83,0,0,'','','','',12)" }
Invoke-Sql -AsRoot -Query "INSERT INTO realmd.account (username, v, s) VALUES ('AUDITB','00','00')"
Invoke-Sql -AsRoot -Query "INSERT INTO characters.characters ($cols) VALUES $(& $row 90001 'AUDITA' 'Auditun'), $(& $row 90002 'AUDITB' 'Auditdeux')"
$item = { param($g, $owner) "($g,$owner,25,0,0,1,0,'0 0 0 0 0 ',0,'$('0 ' * 21)',0,20,0)" }
# L'objet 90001 appartient à AUDITB mais porte le numéro du personnage de AUDITA : il ne doit pas être effacé.
Invoke-Sql -AsRoot -Query "INSERT INTO characters.item_instance (guid, owner_guid, itemEntry, creatorGuid, giftCreatorGuid, count, duration, charges, flags, enchantments, randomPropertyId, durability, itemTextId) VALUES $(& $item 90001 90002), $(& $item 90500 90001)"
Invoke-Sql -AsRoot -Query "INSERT INTO characters.character_inventory (guid, bag, slot, item, item_template) VALUES (90002,0,23,90001,25),(90001,0,23,90500,25)"
Invoke-Sql -AsRoot -Query "INSERT INTO characters.creature_respawn (guid, respawntime, instance) VALUES (90001, 0, 0)"
& (Join-Path $scripts 'Stop-MariaDB.ps1') | Out-Null
$t = Run-Launcher -LauncherArgs @('--do', 'accdel', 'AUDITA') -Until 'compte AUDITA supprimé|Suppression impossible' -TimeoutSec 120
Close-Launcher | Out-Null
$results['2. compte supprimé (1 personnage)'] = $t -match 'compte AUDITA supprimé \(1 personnage'
& (Join-Path $scripts 'Start-MariaDB.ps1') | Out-Null
$results['2. personnage et objet de AUDITA supprimés'] = ((& $q "SELECT COUNT(*) FROM characters.characters WHERE guid=90001") -eq 0) -and ((& $q "SELECT COUNT(*) FROM characters.item_instance WHERE guid=90500") -eq 0)
$results['2. personnage de AUDITB intact'] = (& $q "SELECT COUNT(*) FROM characters.characters WHERE guid=90002") -eq 1
$results['2. objet de AUDITB (numéro 90001) intact'] = ((& $q "SELECT COUNT(*) FROM characters.item_instance WHERE guid=90001 AND owner_guid=90002") -eq 1) -and ((& $q "SELECT COUNT(*) FROM characters.character_inventory WHERE guid=90002") -eq 1)
$results['2. réapparition de créature 90001 intacte'] = (& $q "SELECT COUNT(*) FROM characters.creature_respawn WHERE guid=90001") -eq 1
# Le personnage de test AUDITB n'a pas de données complètes pour le serveur : retiré avant de le relancer.
Invoke-Sql -AsRoot -Query "DELETE FROM characters.character_inventory WHERE guid=90002; DELETE FROM characters.item_instance WHERE guid=90001; DELETE FROM characters.characters WHERE guid=90002; DELETE FROM characters.creature_respawn WHERE guid=90001"
$startXp = & $q "SELECT COALESCE(SUM(c.xp),0) $botChars"
& (Join-Path $scripts 'Stop-MariaDB.ps1') | Out-Null

# ---- 3. Vie des bots : 5 minutes de jeu, puis expérience et déplacements lus en base (sauvegardés à l'arrêt)
Step "3. mmaps ajoutées, bots aléatoires pendant 5 minutes"
Link-Tree (Join-Path $DataDir 'mmaps') (Join-Path $Root 'Server\Data\mmaps')
$t = Run-Launcher -LauncherArgs @('--start') -Until 'monde opérationnel' -ExtraSec 300
$results['3. mmaps activées une fois présentes'] = [bool](Select-String -Path (Join-Path $Root 'Server\Config\mangosd.conf') -Pattern '^mmap\.enabled = 1' -Quiet)
$mangosd = Get-Process mangosd -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
$cpu1 = if ($mangosd) { $mangosd.TotalProcessorTime.TotalSeconds } else { 0 }
Start-Sleep 20
$cpu2 = if ($mangosd) { (Get-Process -Id $mangosd.Id).TotalProcessorTime.TotalSeconds } else { 0 }
$cores = [math]::Round(($cpu2 - $cpu1) / 20, 2)
$mem = if ($mangosd) { [math]::Round((Get-Process -Id $mangosd.Id).WorkingSet64 / 1MB) } else { 0 }
$results["3. serveur resté en service (charge : $cores cœur(s), $mem Mo)"] = ($mangosd -ne $null) -and -not $mangosd.HasExited -and ($t -notmatch 'service interrompu')
$results['3. arrêt propre'] = Close-Launcher
$results['3. aucun processus serveur restant'] = (Remaining-Processes) -eq 0
& (Join-Path $scripts 'Start-MariaDB.ps1') | Out-Null
$endXp = & $q "SELECT COALESCE(SUM(c.xp),0) $botChars"
$withXp = & $q "SELECT COUNT(*) $botChars AND c.xp > 0"
$guilds = & $q "SELECT COUNT(*) FROM characters.guild"
Step "   expérience totale des bots : $startXp -> $endXp ; bots avec de l'expérience : $withXp ; guildes : $guilds"
$results["3. les bots gagnent de l'expérience ($withXp bots)"] = ($endXp -gt $startXp) -and ($withXp -ge 5)
& (Join-Path $scripts 'Stop-MariaDB.ps1') | Out-Null

# ---- 3b. Launcher fermé de force serveur en marche : realmd et mangosd restent ; le démarrage suivant doit les arrêter
Step "3b. fermeture forcée du launcher serveur en marche"
$t = Run-Launcher -LauncherArgs @('--start') -Until 'monde opérationnel' -ExtraSec 5
$script:lastProcess.Kill()
$script:lastProcess.WaitForExit(30000) | Out-Null
Start-Sleep 2
$results['3b. serveur resté en marche après la fermeture forcée'] = (Remaining-Processes) -gt 0

# ---- 4. Réglages appliqués à chaud, puis suppression des bots serveur en marche (arrêt et relance automatiques)
Step "4. réglages à chaud puis suppression des bots serveur en marche"
$hot = Join-Path $Root 'UserData\reglages-test.json'
[IO.File]::WriteAllText($hot, '{"windowed":true,"auto_mmaps":false,"auto_backup":false,"conf":{"AiPlayerbot.MinRandomBots":"10","AiPlayerbot.MaxRandomBots":"30"}}', (New-Object Text.UTF8Encoding $false))
$t = Run-Launcher -LauncherArgs @('--start', '--do', 'applysettings', $hot) -Until 'réglages appliqués au serveur en marche' -ExtraSec 5
$results['4. restes de la fermeture forcée arrêtés au démarrage'] = $t -match 'serveur resté en marche après une fermeture forcée, arrêté'
$results['4. réglages appliqués à chaud (rndbot reload)'] = ($t -match 'réglages appliqués au serveur en marche') -and ((Get-Content (Join-Path $Root 'Logs\server\mangosd.console.log') -Raw) -match 'Playerbot config reloaded')
Close-Launcher | Out-Null
& (Join-Path $scripts 'Start-MariaDB.ps1') | Out-Null
$botsBefore = & $q "SELECT COUNT(*) $botChars"
& (Join-Path $scripts 'Stop-MariaDB.ps1') | Out-Null
$t = Run-Launcher -LauncherArgs @('--start', '--do', 'botdel') -Until '(?s)bot\(s\) supprimé\(s\).*monde opérationnel' -ExtraSec 10
$results["4. $botsBefore personnages bots supprimés"] = $t -match "$botsBefore personnage\(s\) bot\(s\) supprimé\(s\)"
$results['4. serveur relancé après la suppression'] = ([regex]::Matches($t, 'monde opérationnel')).Count -ge 2
$results['4. bots recréés au redémarrage'] = (& $q "SELECT COUNT(*) $botChars") -gt 0
$results['4. aucun personnage joueur touché'] = (& $q "SELECT COUNT(*) FROM characters.characters c LEFT JOIN realmd.account a ON a.id=c.account WHERE a.id IS NULL") -eq 0
$results['4. arrêt propre'] = Close-Launcher
$results['4. aucun processus serveur restant'] = (Remaining-Processes) -eq 0

# ---- Bilan
$all = Get-Content $log -Encoding UTF8
$errors = @($all | Where-Object { $_ -match '\bERREUR\b' })
$results["journal sans ERREUR ($($errors.Count))"] = $errors.Count -eq 0
$ok = $true
foreach ($k in $results.Keys) { Step ("{0,-70} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
$errors | Select-Object -First 10 | ForEach-Object { Step "  $_" }
if ($ok -and -not $Keep) { Remove-Item $Root -Recurse -Force; Step "dossier de test supprimé" }
if ($ok) { Step "TEST RÉUSSI"; exit 0 } else { Step "TEST EN ÉCHEC (dossier conservé : $Root)"; exit 1 }
