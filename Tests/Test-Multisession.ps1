<#
.SYNOPSIS
    Multiboxing et anti-AFK (édition CMaNGOS) avec de vrais clients du protocole 1.12 (Tests\wowclient) : plusieurs
    personnages d'un même compte connectés en même temps, limite de sessions, coupure brutale et reconnexion,
    sauvegarde, et (option -Afk) absence d'expulsion après 15 minutes d'AFK avec HomeCraft.DisableAfkKick = 1, puis
    contrôle inverse avec 0.
.DESCRIPTION
    Travaille sur une installation de test déjà démarrée une fois (mmaps générées), jamais sur le dossier de
    développement. Le client de test doit être compilé (cargo build --release dans Tests\wowclient).
    Ne pas lancer pendant que le launcher de l'utilisateur tourne (mêmes ports).
#>
[CmdletBinding()]
param(
    [string]$Root = (Join-Path $env:USERPROFILE 'HomeCraftTests\cm'),
    [switch]$Afk,
    # Seulement le contrôle inverse (DisableAfkKick = 0 : expulsion attendue vers 15 min)
    [switch]$AfkControlOnly
)
$ErrorActionPreference = 'Stop'
$dev = Split-Path -Parent $PSScriptRoot
$wc = Join-Path $dev 'Tests\wowclient\target\release\hc-wowclient.exe'
$exe = Join-Path $Root 'HomeCraft Edition Vanilla.exe'
$log = Join-Path $Root 'Logs\launcher.log'
$out = Join-Path $Root 'Logs\multisession'
foreach ($p in $wc, $exe) { if (-not (Test-Path $p)) { throw "absent : $p" } }
if (Get-Process -Name 'HomeCraft Edition Vanilla', 'vanilla-solo-launcher' -ErrorAction SilentlyContinue) { throw "un launcher HomeCraft est ouvert : le fermer avant le test" }
New-Item -ItemType Directory -Force $out | Out-Null
. (Join-Path $Root 'Tools\Scripts\Common.ps1')
$results = [ordered]@{}
function Record { param([string]$Name, [bool]$Ok) $script:results[$Name] = $Ok; Step ("   {0} {1}" -f $(if ($Ok) { 'OK' } else { 'ÉCHEC' }), $Name) }
function Step { param([string]$m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
function Sql { param($q) "$(Invoke-Sql -AsRoot -Query $q)".Trim() }

# Mot de passe du compte de test : aléatoire, transmis aux clients par l'environnement, jamais affiché.
# Compte et personnages propres à ce passage (noms de personnages uniques sur le royaume, lettres seulement).
$tag = -join ((97..122) | Get-Random -Count 3 | ForEach-Object { [char]$_ })
$acct = "MBOX$($tag.ToUpper())"
$CN = @{}; foreach ($l in 'A', 'B', 'C', 'D', 'E') { $CN[$l] = "Mb$tag$($l.ToLower())" }
$env:HC_TEST_PASS = -join ((48..57) + (65..90) + (97..122) | Get-Random -Count 14 | ForEach-Object { [char]$_ })

function Start-Launcher { param([string[]]$Extra = @(), [string]$Until, [int]$TimeoutSec = 1800)
    $mark = if (Test-Path $log) { @(Get-Content $log).Count } else { 0 }
    $script:launcher = Start-Process -FilePath $exe -ArgumentList (@('--start') + $Extra + @('--quit-after', 14400)) -WorkingDirectory $Root -PassThru
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $TimeoutSec -and -not $script:launcher.HasExited) {
        Start-Sleep 2
        $text = (@(Get-Content $log -Encoding UTF8 -ErrorAction SilentlyContinue | Select-Object -Skip $mark) -join "`n")
        if ($text -match $Until) { return $true }
        if ($text -match 'ERREUR') { return $false }
    }
    return $false
}
function Stop-Launcher {
    $p = $script:launcher
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ($p -and -not $p.HasExited -and $sw.Elapsed.TotalSeconds -lt 180) { $p.Refresh(); $p.CloseMainWindow() | Out-Null; $p.WaitForExit(5000) | Out-Null }
    if ($p -and -not $p.HasExited) { $p.Kill() }
    Start-Sleep 2
}
trap { Stop-Launcher; Step "TEST INTERROMPU : $_"; exit 1 }

# Client de test en arrière-plan ; sa sortie (lignes EVENT) va dans Logs\multisession\<nom>.log
function Start-Client { param([string]$Name, [int]$Race = 1, [int]$Class = 1, [int]$Stay = 60, [string[]]$More = @())
    # Journal d'un passage précédent du même personnage conservé sous un autre nom
    $f = Join-Path $out "$Name.log"
    if (Test-Path $f) { Move-Item -Force $f (Join-Path $out ("$Name-" + (Get-Date -Format HHmmss) + '.log')) }
    $cargs = @('--account', $acct, '--password-env', 'HC_TEST_PASS', '--char', $Name, '--race', $Race, '--class', $Class, '--stay', $Stay) + $More
    # Handle lu tout de suite : sinon, sous PowerShell 5.1, ExitCode reste vide une fois le processus terminé.
    $p =     Start-Process -FilePath $wc -ArgumentList $cargs -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $out "$Name.log") -RedirectStandardError (Join-Path $out "$Name.err")
    $null = $p.Handle
    $p
}
function Wait-Event { param([string]$Name, [string]$Event, [int]$TimeoutSec = 90)
    $f = Join-Path $out "$Name.log"
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $TimeoutSec) {
        if ((Test-Path $f) -and ((Get-Content $f -Raw -ErrorAction SilentlyContinue) -match "EVENT \S+ $Event")) { return $true }
        Start-Sleep 1
    }
    return $false
}
function Client-Ok { param($Proc, [string]$Name, [int]$TimeoutSec = 1500)
    # Délai d'attente supérieur à la durée de présence demandée au client (--stay) : sinon le test se poursuit
    # (arrêt du serveur) pendant que le client joue encore, et la déconnexion qui en résulte serait mal attribuée.
    if (-not $Proc.WaitForExit($TimeoutSec * 1000)) { Step "   $Name toujours en jeu après $TimeoutSec s"; return $false }
    $t = Get-Content (Join-Path $out "$Name.log") -Raw -ErrorAction SilentlyContinue
    return ($Proc.ExitCode -eq 0 -and $t -match 'EVENT \S+ in_world' -and $t -match 'EVENT \S+ stayed')
}
function Online { param([string[]]$Names) [int](Sql "SELECT COUNT(*) FROM characters.characters WHERE online=1 AND name IN ('$($Names -join "','")')") }

# ---- 0. Serveur et compte
Step "0. démarrage du serveur de test et création du compte $acct"
$ok = Start-Launcher -Extra @('--do', 'acccreate', $acct, $env:HC_TEST_PASS, '0') -Until "compte $acct créé|création du compte impossible|existe déjà"
if (-not $ok) { throw "serveur ou compte : échec (voir $log)" }
Start-Sleep 5
$conf = Get-Content (Join-Path $Root 'Server\Config\mangosd.conf')
Record '0. options écrites (multi-sessions 1, max 4, anti-AFK désactivé 1)' ((($conf -match '^HomeCraft.AllowMultipleSessionsPerAccount = 1$').Count -eq 1) -and (($conf -match '^HomeCraft.MaxSessionsPerAccount = 4$').Count -eq 1) -and (($conf -match '^HomeCraft.DisableAfkKick = 1$').Count -eq 1))

if ($AfkControlOnly) { $Afk = $true }
if (-not $AfkControlOnly) {
# ---- 1. Deux personnages du même compte en même temps
Step "1. deux sessions simultanées (création des personnages $($CN.A), $($CN.B))"
$a = Start-Client $CN.A 1 1 45; Start-Sleep 3; $b = Start-Client $CN.B 3 1 45
$both = (Wait-Event $CN.A in_world) -and (Wait-Event $CN.B in_world)
Start-Sleep 5
$on2 = Online @($CN.A, $CN.B)
$okA = Client-Ok $a $CN.A; $okB = Client-Ok $b $CN.B
Record '1. deux personnages du même compte en jeu ensemble (online=2)' ($both -and $on2 -eq 2 -and $okA -and $okB)

# ---- 2. Quatre personnages
Step "2. quatre sessions simultanées"
$names = $CN.A, $CN.B, $CN.C, $CN.D
$procs = @{}
$races = @{ $CN.A = 1; $CN.B = 3; $CN.C = 4; $CN.D = 7 }
foreach ($n in $names) { $procs[$n] = Start-Client $n $races[$n] 1 75; Start-Sleep 2 }
$all = $true; foreach ($n in $names) { if (-not (Wait-Event $n in_world)) { $all = $false } }
Start-Sleep 5
$on4 = Online $names
# ---- 3. Limite : cinquième session refusée, les quatre autres restent
Step "3. cinquième session (au-delà de MaxSessionsPerAccount = 4)"
$e = Start-Client $CN.E 1 1 20
$e.WaitForExit(90000) | Out-Null
$eLog = Get-Content (Join-Path $out "$($CN.E).log") -Raw -ErrorAction SilentlyContinue
$refused = ($e.ExitCode -ne 0) -and ($eLog -notmatch 'EVENT \S+ in_world')
$stillOn = Online $names
$ok4 = $true; foreach ($n in $names) { if (-not (Client-Ok $procs[$n] $n)) { $ok4 = $false } }
Record '2. quatre personnages du même compte en jeu ensemble (online=4)' ($all -and $on4 -eq 4 -and $ok4)
Record '3. cinquième session refusée, les quatre autres non déconnectées' ($refused -and $stillOn -eq 4)

# ---- 4. Coupure brutale d'un client puis reconnexion, l'autre session continue
Step "4. coupure brutale (processus tué) et reconnexion de $($CN.A) pendant que $($CN.B) joue"
$b = Start-Client $CN.B 3 1 120; $a = Start-Client $CN.A 1 1 300
$in = (Wait-Event $CN.A in_world) -and (Wait-Event $CN.B in_world)
Start-Sleep 5
$a.Kill(); $a.WaitForExit()
Start-Sleep 3
Move-Item -Force (Join-Path $out "$($CN.A).log") (Join-Path $out "$($CN.A)-coupe.log")
$a2 = Start-Client $CN.A 1 1 20
$back = Wait-Event $CN.A in_world 120
$okA = Client-Ok $a2 $CN.A; $okB = Client-Ok $b $CN.B
Record '4. reconnexion après coupure brutale, autre session non affectée' ($in -and $back -and $okA -and $okB)

# ---- 5. Sauvegarde : tous hors ligne, temps de jeu enregistré
Step "5. sauvegarde à la déconnexion"
Start-Sleep 5
$off = [int](Sql "SELECT COUNT(*) FROM characters.characters WHERE online=1 AND name IN ('$($names -join "','")')")
$played = [int](Sql "SELECT COUNT(*) FROM characters.characters WHERE totaltime > 0 AND name IN ('$($names -join "','")')")
Record '5. personnages hors ligne et temps de jeu enregistré pour les 4' (($off -eq 0) -and ($played -eq 4))
}

if ($Afk) {
    if (-not $AfkControlOnly) {
    # ---- 6. AFK > 15 min sans expulsion (DisableAfkKick = 1)
    Step "6. AFK 17 minutes avec HomeCraft.DisableAfkKick = 1 (attente)"
    $k = Start-Client $CN.A 1 1 1020 @('--afk')
    $afkSet = Wait-Event $CN.A afk_set 60; $okK = Client-Ok $k $CN.A
    Record '6. AFK 17 min : pas d''expulsion (DisableAfkKick = 1)' ($afkSet -and $okK)
    }
    Stop-Launcher
    # ---- 7. Contrôle : DisableAfkKick = 0 → expulsion après 15 min (preuve que le test détecte bien l'AFK)
    Step "7. contrôle : HomeCraft.DisableAfkKick = 0, l'expulsion doit avoir lieu vers 15 min"
    $sf = Join-Path $Root 'UserData\settings.json'
    $saved = [IO.File]::ReadAllText($sf)
    $s = $saved | ConvertFrom-Json
    if (-not $s.conf) { $s | Add-Member -NotePropertyName conf -NotePropertyValue (New-Object PSObject) }
    $s.conf | Add-Member -Force -NotePropertyName 'HomeCraft.DisableAfkKick' -NotePropertyValue '0'
    [IO.File]::WriteAllText($sf, ($s | ConvertTo-Json -Depth 6), (New-Object Text.UTF8Encoding $false))
    try {
        if (-not (Start-Launcher -Until 'monde opérationnel|serveur prêt')) { throw "redémarrage du serveur : échec" }
        Start-Sleep 5
        $sw = [Diagnostics.Stopwatch]::StartNew()
        $k = Start-Client $CN.A 1 1 1080 @('--afk')
        $k.WaitForExit(1200000) | Out-Null
        $mins = [math]::Round($sw.Elapsed.TotalMinutes, 1)
        Record "7. contrôle DisableAfkKick = 0 : expulsé par le serveur (code 5, après $mins min)" (($k.ExitCode -eq 5) -and ($mins -ge 14.5))
    } finally {
        [IO.File]::WriteAllText($sf, $saved, (New-Object Text.UTF8Encoding $false))
    }
}

Stop-Launcher
$left = @(Get-Process mangosd, realmd, mariadbd -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase) }).Count
Record 'arrêt sans processus résiduel' ($left -eq 0)
Remove-Item Env:HC_TEST_PASS
$failed = 0
foreach ($k in $results.Keys) { $v = $results[$k]; if (-not $v) { $failed++ }; Write-Host ("{0,-8} {1}" -f $(if ($v) { 'OK' } else { 'ÉCHEC' }), $k) }
Write-Host "Journaux des clients : $out"
if ($failed) { Write-Host "$failed échec(s)"; exit 1 }
Write-Host "Multisession : tout est conforme"
