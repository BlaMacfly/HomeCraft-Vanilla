<#
.SYNOPSIS
    Test d'intégration de bout en bout sur une copie du package (dossier temporaire) : premier démarrage, création
    de compte, suppression de compte hors ligne sans toucher aux données des autres personnages, génération et
    suppression de bots serveur en marche (arrêt et relance automatiques), arrêt sans processus résiduel.
.DESCRIPTION
    Le package doit avoir été construit (Build\New-Package.ps1). Les données du serveur (Server\Data) et les
    archives du client sont des liens physiques vers celles du dossier de développement : rien n'y est écrit.
    Ne pas lancer pendant que le launcher de l'utilisateur tourne (mêmes ports).
#>
[CmdletBinding()]
param(
    [string]$Root = (Join-Path $env:TEMP 'hc-integration'),
    [switch]$Keep
)
$ErrorActionPreference = 'Stop'
$dev = Split-Path -Parent $PSScriptRoot
$version = ((Get-Content (Join-Path $dev 'Launcher\Cargo.toml')) -match '^version' | Select-Object -First 1) -replace '.*"(.*)".*', '$1'
$pkg = Join-Path $dev "Build\package\HomeCraft-Edition-Vanilla-v$version"
if (-not (Test-Path $pkg)) { throw "package v$version absent : lancer Build\New-Package.ps1 d'abord" }
if (Get-Process -Name 'HomeCraft Edition Vanilla', 'vanilla-solo-launcher' -ErrorAction SilentlyContinue) { throw "un launcher HomeCraft est ouvert : le fermer avant le test" }
function Test-Port { param([int]$Port) try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false } }
foreach ($p in 3315, 3724, 8085) { if (Test-Port $p) { throw "port $p occupé : arrêter le serveur HomeCraft d'abord" } }
function Step { param([string]$m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }

Add-Type -Namespace HcTest -Name Native -MemberDefinition '[DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] public static extern bool CreateHardLink(string lpFileName, string lpExistingFileName, IntPtr lpSecurityAttributes);'
function Link-Tree { param([string]$Src, [string]$Dst, [string[]]$SkipDirs = @(), [switch]$CopySmall)
    Get-ChildItem $Src -Recurse -File | Where-Object { $rel = $_.FullName.Substring($Src.Length).TrimStart('\'); -not ($SkipDirs | Where-Object { $rel -like "$_\*" }) } | ForEach-Object {
        $to = Join-Path $Dst $_.FullName.Substring($Src.Length).TrimStart('\')
        $dir = Split-Path $to
        if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force $dir | Out-Null }
        # Fichiers du client autres que les archives : copiés (le launcher peut les réécrire, ex. Config.wtf).
        if ($CopySmall -and $_.Extension -ne '.MPQ') { Copy-Item $_.FullName $to }
        elseif (-not [HcTest.Native]::CreateHardLink($to, $_.FullName, [IntPtr]::Zero)) { Copy-Item $_.FullName $to }
    }
}

# ---- Installation de test
if (Test-Path $Root) {
    if (-not (Test-Path (Join-Path $Root '.hc-integration'))) { throw "$Root existe et n'est pas un dossier de test : refus de le supprimer" }
    Remove-Item $Root -Recurse -Force
}
Step "copie du package v$version vers $Root"
robocopy $pkg $Root /E /NFL /NDL /NJH /NJS /NP | Out-Null
if ($LASTEXITCODE -ge 8) { throw "robocopy : code $LASTEXITCODE" }
Set-Content (Join-Path $Root '.hc-integration') 'dossier de test' -Encoding UTF8
Step "liens physiques vers Server\Data et le client du dossier de développement"
Link-Tree (Join-Path $dev 'Server\Data') (Join-Path $Root 'Server\Data') -SkipDirs @('Buildings', 'mmaps-generation', 'extraction-client')
Link-Tree (Join-Path $dev 'Client') (Join-Path $Root 'Client') -SkipDirs @('WTF', 'Cache', 'Logs', 'Screenshots') -CopySmall
New-Item -ItemType Directory -Force (Join-Path $Root 'UserData') | Out-Null
[IO.File]::WriteAllText((Join-Path $Root 'UserData\settings.json'), '{"windowed":true,"auto_mmaps":false,"auto_backup":false,"conf":{"RandomBot.Enable":"1","RandomBot.MinBots":"10","RandomBot.MaxBots":"30","PlayerBot.ShowInWhoList":"1"}}', (New-Object Text.UTF8Encoding $false))

$exe = Join-Path $Root 'HomeCraft Edition Vanilla.exe'
$log = Join-Path $Root 'Logs\launcher.log'
$scripts = Join-Path $Root 'Tools\Scripts'
. (Join-Path $scripts 'Common.ps1')
$results = [ordered]@{}

function Remaining-Processes {
    @(Get-Process mangosd, realmd, mariadbd, MoveMapGenerator -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase) }).Count
}
# Lance le launcher, attend que le journal contienne $Until (ou ERREUR), puis le ferme comme l'utilisateur.
function Run-Launcher { param([string[]]$LauncherArgs, [string]$Until, [int]$TimeoutSec = 600, [int]$ExtraSec = 0)
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
    return $text
}
function Close-Launcher {
    $p = $script:lastProcess
    if (-not $p.HasExited) { $p.CloseMainWindow() | Out-Null }
    if (-not $p.WaitForExit(180000)) { $p.Kill(); return $false }
    Start-Sleep 2
    return $true
}
function Log-Since { param([int]$Mark) (@(Get-Content $log -Encoding UTF8 | Select-Object -Skip $Mark) -join "`n") }

# ---- 1. Premier démarrage, création de compte (niveau MJ attribué après la création réelle)
Step "1. premier démarrage et création du compte AUDITA"
$t = Run-Launcher -LauncherArgs @('--start', '--do', 'acccreate', 'AUDITA', 'Motdepasse1', '6') -Until 'compte AUDITA créé|création du compte impossible'
$results['1. monde opérationnel au premier démarrage'] = $t -match 'monde opérationnel'
$results['1. compte créé avec le niveau 6'] = $t -match 'compte AUDITA créé \(niveau 6\)'
$results['1. pas de « Compte inexistant »'] = $t -notmatch 'Compte inexistant'
$gm = Invoke-Sql -AsRoot -Query "SELECT COALESCE(MAX(aa.gmlevel), a.gmlevel) FROM realmd.account a LEFT JOIN realmd.account_access aa ON aa.id=a.id WHERE a.username='AUDITA' GROUP BY a.id"
$results['1. niveau 6 en base'] = ("$gm".Trim() -eq '6')
# mangosd réécrit realmlist.realmbuilds avec sa propre liste au démarrage : le client frFR (5880) doit y figurer.
$builds = "$(Invoke-Sql -AsRoot -Query "SELECT realmbuilds FROM realmd.realmlist WHERE id=1")".Trim()
$results["1. build client 5880 accepté par mangosd ($builds)"] = ($builds -split '\s+') -contains '5880'
$results['1. arrêt propre à la fermeture'] = Close-Launcher
$results['1. aucun processus serveur restant'] = (Remaining-Processes) -eq 0
$results['1. mot de passe absent du journal'] = -not (Select-String -Path $log -Pattern 'Motdepasse1' -SimpleMatch -Quiet)

# ---- 2. Suppression de compte serveur arrêté : seules les données du compte disparaissent
Step "2. suppression hors ligne du compte AUDITA, données d'un autre compte intactes"
& (Join-Path $scripts 'Start-MariaDB.ps1') | Out-Null
$cols = 'guid, account, name, race, class, gender, skin, face, hair_style, hair_color, facial_hair, level, xp, money, zone, map, position_x, position_y, position_z, orientation, known_taxi_mask, current_taxi_path, online, create_time, logout_time, health, power1, power2, power3, power4, power5, explored_zones, equipment_cache, action_bars, world_phase_mask'
$row = { param($g, $acc, $n) "($g,(SELECT id FROM realmd.account WHERE username='$acc'),'$n',1,1,0,0,0,0,0,0,10,0,0,12,0,-8949,-132,83,0,'0 0 0 0 0 0 0 0 ','',0,0,0,100,0,0,0,0,0,'$('0 ' * 64)','$('0 0 ' * 19)',0,1)" }
Invoke-Sql -AsRoot -Query "INSERT INTO realmd.account (username, v, s) VALUES ('AUDITB','00','00')"
Invoke-Sql -AsRoot -Query "INSERT INTO characters.characters ($cols) VALUES $(& $row 9001 'AUDITA' 'Auditun'), $(& $row 9002 'AUDITB' 'Auditdeux')"
$item = { param($g, $owner) "($g,25,$owner,0,0,1,0,'0 0 0 0 0 ',0,'$('0 ' * 21)',0,20,0,0)" }
# L'objet 9001 appartient à AUDITB mais porte le numéro du personnage de AUDITA : l'ancien code l'effaçait.
Invoke-Sql -AsRoot -Query "INSERT INTO characters.item_instance (guid, item_id, owner_guid, creator_guid, gift_creator_guid, count, duration, charges, flags, enchantments, random_property_id, durability, text, generated_loot) VALUES $(& $item 9001 9002), $(& $item 9500 9001)"
Invoke-Sql -AsRoot -Query "INSERT INTO characters.character_inventory (guid, bag, slot, item_guid, item_id) VALUES (9002,0,23,9001,25),(9001,0,23,9500,25)"
Invoke-Sql -AsRoot -Query "INSERT INTO characters.creature_respawn (guid, respawn_time, instance, map) VALUES (9001, 0, 0, 0)"
& (Join-Path $scripts 'Stop-MariaDB.ps1') | Out-Null
$t = Run-Launcher -LauncherArgs @('--do', 'accdel', 'AUDITA') -Until 'compte AUDITA supprimé|Suppression impossible' -TimeoutSec 120
Close-Launcher | Out-Null
$results['2. compte supprimé (1 personnage)'] = $t -match 'compte AUDITA supprimé \(1 personnage'
& (Join-Path $scripts 'Start-MariaDB.ps1') | Out-Null
$q = { param($sql) [int]("$(Invoke-Sql -AsRoot -Query $sql)".Trim()) }
$results['2. personnage et objet de AUDITA supprimés'] = ((& $q "SELECT COUNT(*) FROM characters.characters WHERE guid=9001") -eq 0) -and ((& $q "SELECT COUNT(*) FROM characters.item_instance WHERE guid=9500") -eq 0)
$results['2. personnage de AUDITB intact'] = (& $q "SELECT COUNT(*) FROM characters.characters WHERE guid=9002") -eq 1
$results['2. objet de AUDITB (numéro 9001) intact'] = ((& $q "SELECT COUNT(*) FROM characters.item_instance WHERE guid=9001 AND owner_guid=9002") -eq 1) -and ((& $q "SELECT COUNT(*) FROM characters.character_inventory WHERE guid=9002") -eq 1)
$results['2. réapparition de créature 9001 intacte'] = (& $q "SELECT COUNT(*) FROM characters.creature_respawn WHERE guid=9001") -eq 1
& (Join-Path $scripts 'Stop-MariaDB.ps1') | Out-Null

# ---- 3. Bots générés serveur en marche : arrêt, génération, relance automatiques
Step "3. génération de 30 bots serveur en marche"
$mark = @(Get-Content $log).Count
$t = Run-Launcher -LauncherArgs @('--start', '--do', 'botgen', '30') -Until '(?s)personnages bots créés.*monde opérationnel' -ExtraSec 70
$results['3. arrêt du serveur pour la génération'] = $t -match "arrêt du serveur le temps de l'opération"
$results['3. 30 bots créés'] = $t -match '30 personnages bots créés'
$results['3. serveur relancé'] = ([regex]::Matches($t, 'monde opérationnel')).Count -ge 2
$results['3. numéros des bots après ceux des joueurs'] = (& $q "SELECT COUNT(*) FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE a.username='HOMECRAFTBOTS' AND c.guid<=9002") -eq 0
$results['3. bots en phase 1 (visibles dans /qui)'] = (& $q "SELECT COUNT(*) FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE a.username='HOMECRAFTBOTS' AND c.world_phase_mask<>1") -eq 0
$results['3. 30 inscriptions de peuplement'] = (& $q "SELECT COUNT(*) FROM characters.playerbot") -eq 30
$online = & $q "SELECT COUNT(*) FROM characters.characters c JOIN characters.playerbot p ON p.char_guid=c.guid WHERE c.online=1"
$results["3. bots connectés après la relance ($online)"] = $online -gt 0
$results['3. arrêt propre'] = Close-Launcher
$results['3. aucun processus serveur restant'] = (Remaining-Processes) -eq 0

# ---- 4. Suppression des bots serveur en marche
Step "4. suppression des bots serveur en marche"
$t = Run-Launcher -LauncherArgs @('--start', '--do', 'botdel') -Until '(?s)bot\(s\) supprimé\(s\).*monde opérationnel' -ExtraSec 5
$results['4. 30 bots supprimés'] = $t -match '30 personnage\(s\) bot\(s\) supprimé\(s\)'
$results['4. aucun personnage ni inscription de bot'] = ((& $q "SELECT COUNT(*) FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE a.username='HOMECRAFTBOTS'") -eq 0) -and ((& $q "SELECT COUNT(*) FROM characters.playerbot") -eq 0)
$results['4. personnage et objet de AUDITB intacts'] = ((& $q "SELECT COUNT(*) FROM characters.characters WHERE guid=9002") -eq 1) -and ((& $q "SELECT COUNT(*) FROM characters.item_instance WHERE guid=9001") -eq 1)
$results['4. arrêt propre'] = Close-Launcher
$results['4. aucun processus serveur restant'] = (Remaining-Processes) -eq 0

# ---- 5. Bots du monde (mangosd compilé par HomeCraft) : chasse hors des capitales, expérience gagnée
Step "5. bots du monde : 60 bots pendant 5 minutes"
$t = Run-Launcher -LauncherArgs @('--do', 'botgen', '60') -Until 'personnages bots créés' -TimeoutSec 120
Close-Launcher | Out-Null
$results['5. 60 bots générés'] = $t -match '60 personnages bots créés'
# Sauvegarde des bots activée pour le test : leur position et leur expérience sont lues en base après l'arrêt.
[IO.File]::WriteAllText((Join-Path $Root 'UserData\settings.json'), '{"windowed":true,"auto_mmaps":false,"auto_backup":false,"conf":{"RandomBot.Enable":"1","RandomBot.MinBots":"60","RandomBot.MaxBots":"60","RandomBot.Refresh":"10000","PlayerBot.AllowSaving":"1","HomeCraftBot.CityPercent":"25","PlayerBot.ShowInWhoList":"1"}}', (New-Object Text.UTF8Encoding $false))
$t = Run-Launcher -LauncherArgs @('--start') -Until 'monde opérationnel' -ExtraSec 300
$mangosd = Get-Process mangosd -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
$cpu1 = if ($mangosd) { $mangosd.TotalProcessorTime.TotalSeconds } else { 0 }
Start-Sleep 20
$cpu2 = if ($mangosd) { (Get-Process -Id $mangosd.Id).TotalProcessorTime.TotalSeconds } else { 0 }
$cores = [math]::Round(($cpu2 - $cpu1) / 20, 2)
$online = & $q "SELECT COUNT(*) FROM characters.characters c JOIN characters.playerbot p ON p.char_guid=c.guid WHERE c.online=1"
$results["5. bots connectés ($online)"] = $online -ge 40
$results["5. serveur resté en service (charge : $cores cœur(s))"] = ($mangosd -ne $null) -and -not $mangosd.HasExited -and ((Log-Since 0) -notmatch 'service interrompu')
$results['5. arrêt propre'] = Close-Launcher
$results['5. aucun processus serveur restant'] = (Remaining-Processes) -eq 0
& (Join-Path $scripts 'Start-MariaDB.ps1') | Out-Null
$bots = "FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE a.username='HOMECRAFTBOTS'"
$inCapital = & $q "SELECT COUNT(*) $bots AND c.zone IN (1519, 1537, 1637)"
$outside = & $q "SELECT COUNT(*) $bots AND c.zone NOT IN (1519, 1537, 1637)"
$gainedXp = & $q "SELECT COUNT(*) $bots AND c.xp > 0"
Step "   bots en capitale : $inCapital ; hors capitale : $outside ; avec de l'expérience : $gainedXp"
$results["5. la plupart des bots partis chasser ($outside hors capitale)"] = $outside -ge 30
$results["5. expérience gagnée en chassant ($gainedXp bots)"] = $gainedXp -ge 10
& (Join-Path $scripts 'Stop-MariaDB.ps1') | Out-Null

# ---- Bilan
$all = Get-Content $log -Encoding UTF8
$errors = @($all | Where-Object { $_ -match '\bERREUR\b' })
$results["journal sans ERREUR ($($errors.Count))"] = $errors.Count -eq 0
$ok = $true
foreach ($k in $results.Keys) { Step ("{0,-58} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
$errors | Select-Object -First 10 | ForEach-Object { Step "  $_" }
if ($ok -and -not $Keep) { Remove-Item $Root -Recurse -Force; Step "dossier de test supprimé" }
if ($ok) { Step "TEST RÉUSSI"; exit 0 } else { Step "TEST EN ÉCHEC (dossier conservé : $Root)"; exit 1 }
