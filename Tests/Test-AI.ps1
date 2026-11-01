<#
.SYNOPSIS
    HomeCraft AI Bridge de bout en bout : module playerbots -> pont local du launcher -> fournisseur (LM Studio local),
    avec un vrai client du protocole 1.12 qui chuchote à un bot. Vérifie la configuration générée (le module ne voit que
    le pont), la réponse du bot, les intentions filtrées, l'absence de la clé d'API (factice, chiffrée DPAPI) dans les
    fichiers et journaux, puis le repli vers les réponses classiques quand le fournisseur est injoignable.
.DESCRIPTION
    Prérequis : LM Studio lancé sur 127.0.0.1:1234 avec le modèle -Model chargé (rien n'est téléchargé), installation de
    test déjà démarrée une fois. Ne pas lancer pendant que le launcher de l'utilisateur tourne.
#>
[CmdletBinding()]
param(
    [string]$Root = (Join-Path $env:USERPROFILE 'HomeCraftTests\cm'),
    [string]$Model = 'openai/gpt-oss-20b'
)
$ErrorActionPreference = 'Stop'
$dev = Split-Path -Parent $PSScriptRoot
$wc = Join-Path $dev 'Tests\wowclient\target\release\hc-wowclient.exe'
$exe = Join-Path $Root 'HomeCraft Edition Vanilla.exe'
$log = Join-Path $Root 'Logs\launcher.log'
$work = Join-Path $Root 'Logs\ai-test'
if (Get-Process -Name 'HomeCraft Edition Vanilla', 'vanilla-solo-launcher' -ErrorAction SilentlyContinue) { throw "un launcher HomeCraft est ouvert : le fermer avant le test" }
Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $work | Out-Null
. (Join-Path $Root 'Tools\Scripts\Common.ps1')
Add-Type -AssemblyName System.Security
$results = [ordered]@{}
function Step { param([string]$m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
function Record { param([string]$Name, [bool]$Ok) $script:results[$Name] = $Ok; Step ("   {0} {1}" -f $(if ($Ok) { 'OK' } else { 'ÉCHEC' }), $Name) }
function Sql { param($q) "$(Invoke-Sql -AsRoot -Query $q)".Trim() }
function Start-Launcher { param([string[]]$Extra = @(), [string]$Until, [int]$TimeoutSec = 900)
    $mark = if (Test-Path $log) { @(Get-Content $log).Count } else { 0 }
    $script:launcher = Start-Process -FilePath $exe -ArgumentList (@('--start') + $Extra + @('--quit-after', 7200)) -WorkingDirectory $Root -PassThru
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
# Même chiffrement que le launcher (secret.rs) : DPAPI, portée utilisateur, entropie HomeCraft, hexadécimal.
function Protect { param([string]$Plain)
    $e = [Text.Encoding]::UTF8.GetBytes('HomeCraft Edition Vanilla / secret local v1')
    $b = [Security.Cryptography.ProtectedData]::Protect([Text.Encoding]::UTF8.GetBytes($Plain), $e, 'CurrentUser')
    -join ($b | ForEach-Object { $_.ToString('x2') })
}
function Write-AiSettings { param([string]$Url)
    $s = [ordered]@{ enabled = $true; provider = 'lmstudio'; model = $Model; base_url = $Url; api_key = (Protect $fakeKey); temperature = 0.7; max_tokens = 400; timeout_secs = 120; concurrency = 2; spontaneous = 0; personality = 'Tu es un aventurier chaleureux.'; intents = $true; language = 'fr' }
    [IO.File]::WriteAllText((Join-Path $Root 'UserData\ai-settings.json'), ($s | ConvertTo-Json), (New-Object Text.UTF8Encoding $false))
}
$saved = if (Test-Path (Join-Path $Root 'UserData\ai-settings.json')) { [IO.File]::ReadAllText((Join-Path $Root 'UserData\ai-settings.json')) } else { $null }
trap { Stop-Launcher; Step "TEST INTERROMPU : $_"; exit 1 }

$fakeKey = 'sk-homecraft-test-' + (-join ((48..57) + (97..122) | Get-Random -Count 24 | ForEach-Object { [char]$_ }))
$env:HC_TEST_PASS = -join ((48..57) + (65..90) | Get-Random -Count 12 | ForEach-Object { [char]$_ })
$acct = 'AITEST' + (-join ((65..90) | Get-Random -Count 3 | ForEach-Object { [char]$_ }))
$char = 'Aia' + (-join ((97..122) | Get-Random -Count 5 | ForEach-Object { [char]$_ }))

# ---- 0. Fournisseur local disponible (aucun téléchargement)
Step "0. LM Studio et modèle $Model"
$models = (Invoke-RestMethod -Uri 'http://127.0.0.1:1234/v1/models' -TimeoutSec 10).data.id
Record "0. LM Studio répond et propose $Model" ($models -contains $Model)

# ---- 1. Configuration générée et pont démarré
Step "1. démarrage avec l'IA activée"
Write-AiSettings ''
$ok = Start-Launcher -Extra @('--do', 'acccreate', $acct, $env:HC_TEST_PASS, '3') -Until "compte $acct créé"
$conf = Get-Content (Join-Path $Root 'Server\Config\aiplayerbot.conf') -Raw
Record '1. pont IA démarré avant mangosd' ($ok -and ((Get-Content $log -Raw -Encoding UTF8) -match 'pont IA opérationnel sur 127\.0\.0\.1:15910'))
Record '1b. aiplayerbot.conf : LLM activé vers le pont local seulement, jamais la clé ni l''adresse du fournisseur' (($conf -match '(?m)^AiPlayerbot\.LLMEnabled = 2') -and ($conf -match '(?m)^AiPlayerbot\.LLMApiEndpoint = http://127\.0\.0\.1:15910/homecraft/generate') -and -not $conf.Contains($fakeKey) -and -not $conf.Contains('1234'))
$health = try { (Invoke-RestMethod -Uri 'http://127.0.0.1:15910/homecraft/health' -TimeoutSec 5).ok } catch { $false }
$unauth = try { Invoke-WebRequest -Uri 'http://127.0.0.1:15910/homecraft/generate' -Method Post -Body '{}' -UseBasicParsing -TimeoutSec 5 | Out-Null; 'accepté' } catch { [int]$_.Exception.Response.StatusCode }
Record '1c. pont : santé OK, demande sans jeton local refusée (401)' ($health -eq $true -and $unauth -eq 401)

# ---- 2. Un joueur chuchote à un bot connecté ; réponse générée par le modèle
$bot = Sql "SELECT c.name FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE a.username LIKE 'RNDBOT%' AND c.online=1 ORDER BY c.level DESC LIMIT 1"
Step "2. $char chuchote à $bot (après .appear pour être à côté)"
$c = Start-Process -FilePath $wc -ArgumentList @('--account', $acct, '--password-env', 'HC_TEST_PASS', '--char', $char, '--say', ".appear $bot", '--whisper', $bot, 'Salut ! Tu veux bien me suivre, s''il te plaît ?', '--expect-reply-from', $bot, '--stay', '180') -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $work 'client-ia.log') -RedirectStandardError (Join-Path $work 'client-ia.err')
$null = $c.Handle
$c.WaitForExit(400000) | Out-Null
$cl = Get-Content (Join-Path $work 'client-ia.log') -Raw -ErrorAction SilentlyContinue
$bridgeLog = Get-Content (Join-Path $Root 'Logs\ai-bridge.log') -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
$answered = $bridgeLog -match "$bot : réponse en \d+ ms"
Record "2. le pont a appelé le modèle pour $bot et le bot a répondu en chuchotant" ($c.ExitCode -eq 0 -and $answered -and $cl -match 'chat type=6 ')
$intent = [regex]::Match($bridgeLog, "$bot : réponse en \d+ ms \(\d+ caractères\), intention \[hc:(\w+)\]").Groups[1].Value
$mlog = Get-Content $log -Raw -Encoding UTF8
$executed = $mlog -match "BotLLM: intention « $intent » de $char exécutée par $bot"
Step ("   intention proposée par le modèle : " + $(if ($intent) { "[hc:$intent], exécutée : $executed" } else { 'aucune' }))
Copy-Item (Join-Path $Root 'Logs\ai-bridge.log') (Join-Path $work 'ai-bridge.log') -ErrorAction SilentlyContinue
Stop-Launcher

# ---- 3. Repli : fournisseur injoignable -> réponses classiques des bots
Step "3. repli : fournisseur injoignable (port fermé)"
Write-AiSettings 'http://127.0.0.1:9/v1'
$ok = Start-Launcher -Until 'monde opérationnel'
Start-Sleep 5
$bot = Sql "SELECT c.name FROM characters.characters c JOIN realmd.account a ON a.id=c.account WHERE a.username LIKE 'RNDBOT%' AND c.online=1 ORDER BY c.level DESC LIMIT 1"
# Premier chuchotement : la génération échoue (le pont répond 502), le module bascule ; second : réponse classique.
$c1 = Start-Process -FilePath $wc -ArgumentList @('--account', $acct, '--password-env', 'HC_TEST_PASS', '--char', $char, '--whisper', $bot, 'Bonjour !', '--stay', '25') -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $work 'client-repli1.log') -RedirectStandardError (Join-Path $work 'client-repli1.err')
$null = $c1.Handle
$c1.WaitForExit(120000) | Out-Null
Start-Sleep 20
$c2 = Start-Process -FilePath $wc -ArgumentList @('--account', $acct, '--password-env', 'HC_TEST_PASS', '--char', $char, '--whisper', $bot, 'Salut, ça va ?', '--expect-reply-from', $bot, '--stay', '30') -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $work 'client-repli2.log') -RedirectStandardError (Join-Path $work 'client-repli2.err')
$null = $c2.Handle
$c2.WaitForExit(120000) | Out-Null
$mlog = Get-Content $log -Raw -Encoding UTF8
$bl = Get-Content (Join-Path $Root 'Logs\ai-bridge.log') -Raw -Encoding UTF8
Record '3. fournisseur injoignable : échec journalisé par le pont, le module bascule' (($bl -match 'échec en \d+ ms — fournisseur injoignable') -and ($mlog -match 'BotLLM: génération en échec, réponses classiques'))
Record '3b. réponse classique du bot reçue pendant le repli' ($c2.ExitCode -eq 0)
Stop-Launcher

# ---- 4. Clé d'API factice : jamais en clair
Step "4. recherche de la clé d'API dans les fichiers et journaux"
$files = @(Get-ChildItem (Join-Path $Root 'Logs'), (Join-Path $Root 'Server\Config'), (Join-Path $Root 'UserData') -Recurse -File -ErrorAction SilentlyContinue | Where-Object { $_.Length -lt 50MB -and $_.FullName -notmatch '\\Database\\' })
$hits = @($files | Where-Object { (Get-Content $_.FullName -Raw -ErrorAction SilentlyContinue) -match [regex]::Escape($fakeKey) } | ForEach-Object { $_.FullName })
Record "4. clé d'API introuvable en clair dans $($files.Count) fichiers (journaux, configuration, UserData)" ($hits.Count -eq 0)
if ($hits) { $hits | ForEach-Object { Step "   trouvée dans $_" } }

if ($null -ne $saved) { [IO.File]::WriteAllText((Join-Path $Root 'UserData\ai-settings.json'), $saved, (New-Object Text.UTF8Encoding $false)) } else { Remove-Item (Join-Path $Root 'UserData\ai-settings.json') }
Remove-Item Env:HC_TEST_PASS
$failed = 0
foreach ($k in $results.Keys) { if (-not $results[$k]) { $failed++ } }
Write-Host "Dossier du test : $work"
if ($failed) { Write-Host "$failed échec(s)"; exit 1 }
Write-Host "IA : tout est conforme"
