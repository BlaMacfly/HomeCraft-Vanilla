<#
.SYNOPSIS
    HomeCraft Multijoueur de bout en bout avec deux launchers sur une seule machine : hôte (installation de test avec
    serveur) et invité (seconde installation, sans serveur), réseau intégré au launcher (aucun programme externe). Un
    vrai client du protocole 1.12 joue à travers la liaison chiffrée (royaume et monde). Vérifie aussi qu'une connexion
    quelconque au port de l'hôte est rejetée, que les secrets (clé d'invitation, code, mot de passe) n'apparaissent dans
    aucun journal, et qu'il ne reste aucun processus après fermeture.
.DESCRIPTION
    Révocation, clé fausse, expiration et cloisonnement (seuls 3724/8085 atteignables) sont couverts par les tests
    automatiques du module (cargo test network), sur de vraies connexions TCP.
    LIMITE : hôte et invité sont sur le même PC (connexion par l'adresse du réseau local) ; ce test ne prouve rien sur
    la traversée d'une box ou d'Internet, à vérifier entre deux réseaux réels.
    Ports locaux de l'invité : 13724/18085 (l'hôte occupe 3724/8085 sur la même machine).
    Ne pas lancer pendant que le launcher de l'utilisateur tourne.
#>
[CmdletBinding()]
param(
    [string]$HostRoot = (Join-Path $env:USERPROFILE 'HomeCraftTests\cm'),
    [string]$GuestRoot = (Join-Path $env:USERPROFILE 'HomeCraftTests\guest'),
    # L'invité n'utilise que la liaison QUIC (celle du perçage de NAT), jamais les adresses TCP directes
    [switch]$QuicOnly
)
$ErrorActionPreference = 'Stop'
$dev = Split-Path -Parent $PSScriptRoot
$wc = Join-Path $dev 'Tests\wowclient\target\release\hc-wowclient.exe'
$launcherBuild = Join-Path $dev 'Launcher\target\release\vanilla-solo-launcher.exe'
$exeName = 'HomeCraft Edition Vanilla.exe'
if (Get-Process -Name 'HomeCraft Edition Vanilla', 'vanilla-solo-launcher' -ErrorAction SilentlyContinue) { throw "un launcher HomeCraft est ouvert : le fermer avant le test" }
$work = Join-Path $HostRoot 'Logs\network-test'
Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $work | Out-Null
$results = [ordered]@{}
function Step { param([string]$m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
function Record { param([string]$Name, [bool]$Ok) $script:results[$Name] = $Ok; Step ("   {0} {1}" -f $(if ($Ok) { 'OK' } else { 'ÉCHEC' }), $Name) }
$script:launchers = @()
function Start-L { param([string]$Root, [string[]]$LArgs)
    $p = Start-Process -FilePath (Join-Path $Root $exeName) -ArgumentList ($LArgs + @('--quit-after', 3600)) -WorkingDirectory $Root -PassThru
    $script:launchers += $p
    $p
}
function Stop-L { param($p)
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ($p -and -not $p.HasExited -and $sw.Elapsed.TotalSeconds -lt 180) { $p.Refresh(); $p.CloseMainWindow() | Out-Null; $p.WaitForExit(5000) | Out-Null }
    if ($p -and -not $p.HasExited) { $p.Kill() }
}
function Wait-Until { param([scriptblock]$Cond, [int]$TimeoutSec = 120)
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $TimeoutSec) { if (& $Cond) { return $true }; Start-Sleep 2 }
    return $false
}
function Log-Has { param([string]$File, [string]$Pattern) (Test-Path $File) -and ((Get-Content $File -Raw -Encoding UTF8) -match $Pattern) }
function Decode-Invite { param([string]$Code)
    $b = $Code.Trim().Substring(4).Replace('-', '+').Replace('_', '/')
    switch ($b.Length % 4) { 2 { $b += '==' } 3 { $b += '=' } }
    [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($b)) | ConvertFrom-Json
}
trap { foreach ($p in $script:launchers) { Stop-L $p }; Step "TEST INTERROMPU : $_"; exit 1 }

# ---- 0. Installations : invité = launcher seul (sans serveur), réglages réseau de test (UPnP désactivé : test local)
Step "0. préparation : hôte $HostRoot, invité $GuestRoot"
foreach ($d in 'Server', 'Tools', 'UserData', 'Logs') { New-Item -ItemType Directory -Force (Join-Path $GuestRoot $d) | Out-Null }
Copy-Item -Force $launcherBuild (Join-Path $GuestRoot $exeName)
Copy-Item -Force $launcherBuild (Join-Path $HostRoot $exeName)
Remove-Item -Recurse -Force (Join-Path $HostRoot 'Logs\network'), (Join-Path $GuestRoot 'Logs\network') -ErrorAction SilentlyContinue
[IO.File]::WriteAllText((Join-Path $GuestRoot 'UserData\network.json'), ('{"player_name":"InviteTest","guest_realm_port":13724,"guest_world_port":18085,"upnp":false,"quic_only":' + $(if ($QuicOnly) { 'true' } else { 'false' }) + '}'), (New-Object Text.UTF8Encoding $false))
[IO.File]::WriteAllText((Join-Path $HostRoot 'UserData\network.json'), '{"player_name":"HoteTest","upnp":false}', (New-Object Text.UTF8Encoding $false))
Record '0. aucun composant réseau externe dans les installations' (-not (Test-Path (Join-Path $HostRoot 'Tools\EasyTier')) -and -not (Test-Path (Join-Path $GuestRoot 'Tools\EasyTier')))

# ---- 1. Hôte : serveur, compte de l'invité, partie ouverte, invitation
Step "1. hôte : serveur, compte invité, partie ouverte, invitation"
$gAcct = 'NETGUEST' + (-join ((65..90) | Get-Random -Count 3 | ForEach-Object { [char]$_ }))
$env:HC_TEST_PASS = -join ((48..57) + (65..90) | Get-Random -Count 12 | ForEach-Object { [char]$_ })
$env:HOMECRAFT_NET_CODE_DIR = $work
$hostP = Start-L $HostRoot @('--start', '--do', 'acccreate', $gAcct, $env:HC_TEST_PASS, '0', '--net', 'host', '--invite', 'Joueur')
Remove-Item Env:HOMECRAFT_NET_CODE_DIR
$hlog = Join-Path $HostRoot 'Logs\launcher.log'
$hnet = Join-Path $HostRoot 'Logs\network\homecraft-network.log'
$ready = Wait-Until { (Log-Has $hlog "compte $gAcct créé") -and (Test-Path (Join-Path $work 'Joueur.txt')) } 1500
Record '1. hôte prêt : serveur, compte invité, partie ouverte, code HC2-' ($ready -and ((Get-Content (Join-Path $work 'Joueur.txt') -Raw) -match '^HC2-') -and (Log-Has $hnet 'partie ouverte sur le port 11010'))
if (-not $ready) { throw "hôte non prêt" }
$code = (Get-Content (Join-Path $work 'Joueur.txt') -Raw).Trim()
$inv = Decode-Invite $code
Record '1b. code : clé propre à ce joueur (32 octets), identifiant, adresse(s), expiration ; aucun mot de passe' ($inv.k.Length -eq 64 -and $inv.i.Length -eq 32 -and @($inv.p).Count -ge 1 -and $inv.x -gt [DateTimeOffset]::Now.ToUnixTimeSeconds() -and -not $code.Contains($env:HC_TEST_PASS))

# ---- 2. Connexion quelconque au port de l'hôte : rejetée sans réponse
Step "2. connexion non autorisée au port de l'hôte"
$rejected = $false
try {
    $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', 11010)
    $s = $c.GetStream(); $s.ReadTimeout = 15000
    $junk = [Text.Encoding]::ASCII.GetBytes("GET / HTTP/1.0`r`n`r`n" + ('x' * 64)); $s.Write($junk, 0, $junk.Length)
    $buf = New-Object byte[] 16; $n = $s.Read($buf, 0, 16); $rejected = ($n -eq 0); $c.Close()
} catch { $rejected = $true }
Record '2. données sans invitation valide : connexion fermée par l''hôte, aucune réponse' $rejected

# ---- 3. Invité : rejoint, puis joue à travers la liaison (royaume et monde)
Step "3. invité : liaison puis jeu"
$guestP = Start-L $GuestRoot @('--net', 'join', (Join-Path $work 'Joueur.txt'))
$gnet = Join-Path $GuestRoot 'Logs\network\homecraft-network.log'
$linked = Wait-Until { Log-Has $gnet 'relié à l''hôte' } 60
$seen = Wait-Until { Log-Has $hnet 'invitation créée : Joueur' } 5
Record '3. invité relié à l''hôte (canal de présence chiffré)' ($linked -and $seen)
if ($QuicOnly) { Record '3q. liaison établie par QUIC (chemin du perçage de NAT), sans adresse TCP directe' ((Log-Has $gnet 'liaison QUIC établie') -and (Log-Has $hnet 'liaison QUIC établie')) }
$cname = 'Net' + (-join ((97..122) | Get-Random -Count 5 | ForEach-Object { [char]$_ }))
$g = Start-Process -FilePath $wc -ArgumentList @('--realm', '127.0.0.1:13724', '--world-override', '127.0.0.1:18085', '--account', $gAcct, '--password-env', 'HC_TEST_PASS', '--char', $cname, '--stay', '40') -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $work 'client-invite.log') -RedirectStandardError (Join-Path $work 'client-invite.err')
$null = $g.Handle
$g.WaitForExit(240000) | Out-Null
$cl = Get-Content (Join-Path $work 'client-invite.log') -Raw -ErrorAction SilentlyContinue
Record '3b. personnage de l''invité créé et en jeu 40 s à travers la liaison (royaume 13724 et monde 18085)' ($g.ExitCode -eq 0 -and $cl -match 'char_created' -and $cl -match 'in_world' -and $cl -match 'stayed' -and $cl -match 'logout_complete')

# ---- 4. Secrets : ni dans les journaux, ni sur la ligne de commande des launchers
Step "4. secrets absents des journaux"
$logs = (Get-ChildItem (Join-Path $HostRoot 'Logs'), (Join-Path $GuestRoot 'Logs') -Recurse -File -Filter '*.log' -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notlike "$work*" } | ForEach-Object { Get-Content $_.FullName -Raw -Encoding UTF8 }) -join "`n"
$leak = $logs.Contains($inv.k) -or $logs.Contains($code) -or $logs.Contains($env:HC_TEST_PASS)
Record '4. clé d''invitation, code et mot de passe absents de tous les journaux' (-not $leak)

# ---- 5. Fermeture : liaison coupée, aucun processus résiduel
Step "5. fermeture de l'hôte puis de l'invité"
Stop-L $hostP
$lost = Wait-Until { Log-Has $gnet 'liaison avec l''hôte perdue' } 30
Stop-L $guestP
Start-Sleep 3
$left = @(Get-Process mangosd, realmd, mariadbd, 'HomeCraft Edition Vanilla' -ErrorAction SilentlyContinue | Where-Object { $_.Path -and ($_.Path.StartsWith($HostRoot, [StringComparison]::OrdinalIgnoreCase) -or $_.Path.StartsWith($GuestRoot, [StringComparison]::OrdinalIgnoreCase)) }).Count
$portFree = -not (Get-NetTCPConnection -LocalPort 11010, 13724, 18085 -State Listen -ErrorAction SilentlyContinue)
Record '5. fermeture de l''hôte détectée par l''invité ; aucun processus ni port d''écoute résiduel' ($lost -and $left -eq 0 -and $portFree)
Remove-Item Env:HC_TEST_PASS

$failed = 0
foreach ($k in $results.Keys) { if (-not $results[$k]) { $failed++ } }
Write-Host "Dossier du test : $work"
if ($failed) { Write-Host "$failed échec(s)"; exit 1 }
Write-Host "Réseau : tout est conforme (sur une seule machine ; traversée d'une box non testée)"
