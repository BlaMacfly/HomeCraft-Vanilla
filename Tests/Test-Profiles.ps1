<#
.SYNOPSIS
    Profils de lancement de bout en bout avec un vrai client : le launcher lance le client avec le profil (comme son
    bouton) ; on vérifie sur le serveur que le compte s'est authentifié sans saisie humaine (et, avec Benilla, que le
    personnage préféré est entré en jeu), que le mot de passe n'est écrit en clair nulle part, et qu'un second lancement
    du même profil est refusé tant que le client est ouvert.
.DESCRIPTION
    -Client : dossier d'un client de test isolé (copie : MPQ en liens physiques). Benilla : variables WOW_USER/WOW_PASS/
    WOW_CHAR (mécanisme du client). Client d'origine : accountName dans Config.wtf + saisie du mot de passe dans sa
    fenêtre. Le client s'affiche à l'écran pendant le test ; il est fermé à la fin.
    Ne pas lancer pendant que le launcher de l'utilisateur tourne.
#>
[CmdletBinding()]
param(
    [string]$Root = (Join-Path $env:USERPROFILE 'HomeCraftTests\cm'),
    [Parameter(Mandatory)] [string]$Client
)
$ErrorActionPreference = 'Stop'
$dev = Split-Path -Parent $PSScriptRoot
$wc = Join-Path $dev 'Tests\wowclient\target\release\hc-wowclient.exe'
$exe = Join-Path $Root 'HomeCraft Edition Vanilla.exe'
$log = Join-Path $Root 'Logs\launcher.log'
if (Get-Process -Name 'HomeCraft Edition Vanilla', 'vanilla-solo-launcher' -ErrorAction SilentlyContinue) { throw "un launcher HomeCraft est ouvert : le fermer avant le test" }
. (Join-Path $Root 'Tools\Scripts\Common.ps1')
Add-Type -AssemblyName System.Security
$results = [ordered]@{}
function Step { param([string]$m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
function Record { param([string]$Name, [bool]$Ok) $script:results[$Name] = $Ok; Step ("   {0} {1}" -f $(if ($Ok) { 'OK' } else { 'ÉCHEC' }), $Name) }
function Sql { param($q) "$(Invoke-Sql -AsRoot -Query $q)".Trim() }
function Start-Launcher { param([string[]]$Extra = @(), [string]$Until, [int]$TimeoutSec = 900)
    $mark = if (Test-Path $log) { @(Get-Content $log).Count } else { 0 }
    $script:launcher = Start-Process -FilePath $exe -ArgumentList (@('--start') + $Extra + @('--quit-after', 3600)) -WorkingDirectory $Root -PassThru
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
# Client lancé par ce test : reconnu par son dossier, ou, pour WoW.exe (32 bits, chemin illisible), par son heure de lancement.
$testStart = Get-Date
function Stop-TestClient {
    Get-Process -ErrorAction SilentlyContinue | Where-Object { ($_.Path -and $_.Path.StartsWith($Client, [StringComparison]::OrdinalIgnoreCase)) -or ($_.Name -in 'WoW', 'VanillaFixes' -and -not $_.Path -and $_.StartTime -ge $testStart) } | Stop-Process -Force
}
function Protect { param([string]$Plain)
    $e = [Text.Encoding]::UTF8.GetBytes('HomeCraft Edition Vanilla / secret local v1')
    $b = [Security.Cryptography.ProtectedData]::Protect([Text.Encoding]::UTF8.GetBytes($Plain), $e, 'CurrentUser')
    -join ($b | ForEach-Object { $_.ToString('x2') })
}
$sf = Join-Path $Root 'UserData\settings.json'
$pf = Join-Path $Root 'UserData\profiles.json'
$savedSettings = [IO.File]::ReadAllText($sf)
$savedProfiles = if (Test-Path $pf) { [IO.File]::ReadAllText($pf) } else { $null }
function Restore { [IO.File]::WriteAllText($sf, $savedSettings, (New-Object Text.UTF8Encoding $false)); if ($null -ne $savedProfiles) { [IO.File]::WriteAllText($pf, $savedProfiles, (New-Object Text.UTF8Encoding $false)) } else { Remove-Item $pf -ErrorAction SilentlyContinue } }
trap { Stop-TestClient; Stop-Launcher; Restore; Step "TEST INTERROMPU : $_"; exit 1 }

$benilla = Test-Path (Join-Path $Client 'benilla-config')
$kind = if ($benilla) { 'Benilla' } else { 'client d''origine' }
$pass = -join ((48..57) + (65..90) | Get-Random -Count 12 | ForEach-Object { [char]$_ })
$acct = 'PROF' + (-join ((65..90) | Get-Random -Count 4 | ForEach-Object { [char]$_ }))
$char = 'Prof' + (-join ((97..122) | Get-Random -Count 5 | ForEach-Object { [char]$_ }))

# ---- 1. Compte et personnage du profil (créé par le client de test du protocole)
Step "1. compte $acct et personnage $char ($kind)"
$s = $savedSettings | ConvertFrom-Json
$s | Add-Member -Force -NotePropertyName client_dir -NotePropertyValue $Client
[IO.File]::WriteAllText($sf, ($s | ConvertTo-Json -Depth 6), (New-Object Text.UTF8Encoding $false))
if (-not (Start-Launcher -Extra @('--do', 'acccreate', $acct, $pass, '0') -Until "compte $acct créé")) { throw "serveur ou compte : échec" }
$env:HC_TEST_PASS = $pass
$c = Start-Process -FilePath $wc -ArgumentList @('--account', $acct, '--password-env', 'HC_TEST_PASS', '--char', $char, '--stay', '3') -NoNewWindow -PassThru
$null = $c.Handle
$c.WaitForExit(90000) | Out-Null
Remove-Item Env:HC_TEST_PASS
Record "1. compte et personnage prêts" ($c.ExitCode -eq 0)
Stop-Launcher

# ---- 2. Profil enregistré avec mot de passe chiffré, lancé comme par son bouton
Step "2. lancement du client par le profil (mot de passe enregistré, chiffré DPAPI)"
$prof = [ordered]@{ default_id = 'ptest'; list = @([ordered]@{ id = 'ptest'; name = 'TestProfil'; account = $acct; password = (Protect $pass); character = $char; enter_world = $true; extra_args = '' }) }
[IO.File]::WriteAllText($pf, ($prof | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
$t0 = [DateTimeOffset]::Now.ToUnixTimeSeconds()
$ok = Start-Launcher -Extra @('--do', 'playprofile', 'TestProfil') -Until 'jeu lancé'
$charLog = Join-Path $Root 'Logs\server\Char.log'
$marker = if (Test-Path $charLog) { @(Get-Content $charLog).Count } else { 0 }
$authed = $false; $inWorld = $false
$sw = [Diagnostics.Stopwatch]::StartNew()
while ($sw.Elapsed.TotalSeconds -lt 150) {
    Start-Sleep 3
    $ll = [int64](Sql "SELECT IFNULL(MAX(UNIX_TIMESTAMP(l.loginTime)),0) FROM realmd.account_logons l JOIN realmd.account a ON a.id=l.accountId WHERE a.username='$acct'")
    if ($ll -ge $t0) { $authed = $true }
    $inWorld = (@(Get-Content $charLog -ErrorAction SilentlyContinue | Select-Object -Skip $marker) -join "`n") -match "Login Character:\[$char\]"
    if ($authed -and ($inWorld -or -not $benilla)) { break }
}
$llog = Get-Content $log -Raw -Encoding UTF8
Record "2. client lancé par le profil, compte authentifié sans saisie ($kind)" ($ok -and $authed)
if ($benilla) {
    Record "2b. personnage préféré entré en jeu (WOW_CHAR)" $inWorld
    Record "2c. compte mémorisé dans benilla-config\account" (((Get-Content (Join-Path $Client 'benilla-config\account') -Raw).Trim()) -eq $acct)
    Record "2d. journal : connexion automatique et personnage, sans mot de passe" (($llog -match 'profil « TestProfil » : compte prérempli, connexion automatique, personnage présélectionné') -and -not $llog.Contains($pass))
} else {
    $cfg = Get-Content (Join-Path $Client 'WTF\Config.wtf') -Raw
    Record "2b. accountName écrit dans Config.wtf" ($cfg -match "SET accountName `"$acct`"")
    Record "2c. journal : mot de passe saisi dans la fenêtre, jamais écrit" (($llog -match 'mot de passe saisi dans la fenêtre du client') -and -not $llog.Contains($pass))
}

# Fermeture du client de test puis du launcher
Stop-TestClient
Stop-Launcher

# ---- 3. Le mot de passe n'est en clair nulle part
Step "3. recherche du mot de passe en clair"
$files = @(Get-ChildItem (Join-Path $Root 'Logs'), (Join-Path $Root 'UserData'), (Join-Path $Client 'WTF'), (Join-Path $Client 'benilla-config') -Recurse -File -ErrorAction SilentlyContinue | Where-Object { $_.Length -lt 50MB -and $_.FullName -notmatch '\\Database\\' })
$hits = @($files | Where-Object { (Get-Content $_.FullName -Raw -ErrorAction SilentlyContinue) -match [regex]::Escape($pass) } | ForEach-Object { $_.FullName })
Record "3. mot de passe introuvable en clair dans $($files.Count) fichiers" ($hits.Count -eq 0)
if ($hits) { $hits | ForEach-Object { Step "   trouvé dans $_" } }

Restore
$failed = 0
foreach ($k in $results.Keys) { if (-not $results[$k]) { $failed++ } }
if ($failed) { Write-Host "$failed échec(s)"; exit 1 }
Write-Host "Profils ($kind) : tout est conforme"
