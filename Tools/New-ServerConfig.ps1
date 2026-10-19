<#
.SYNOPSIS
    Génère Server/Config/realmd.conf et mangosd.conf à partir des modèles *.conf.dist.in de VMaNGOS.
    Les fichiers générés sont « gérés » : ils sont régénérés à chaque démarrage et ne doivent pas être édités à la main.
    Les réglages modifiables par l'utilisateur (rates, etc.) viendront de UserData/settings.json (Phase 4).
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$cred = Get-DbCredentials
$cfgDir   = Get-ProjectPath 'Server\Config'
$dataDir  = Get-ProjectPath 'Server\Data'       # dbc/maps/vmaps/mmaps extraits du client (non redistribuables)
$logDir   = Join-Path $Layout.Logs 'server'
New-Item -ItemType Directory -Force $dataDir, $logDir | Out-Null

function Set-ConfValue {
    # Remplace « Clé = valeur » (première occurrence en début de ligne). Erreur si la clé n'existe pas dans le modèle :
    # cela signale un changement de format VMaNGOS plutôt que de l'ignorer en silence.
    param([string[]]$Lines, [string]$Key, [string]$Value)
    $pattern = '^' + [regex]::Escape($Key) + '\s*='
    $idx = -1
    for ($i = 0; $i -lt $Lines.Count; $i++) { if ($Lines[$i] -match $pattern) { $idx = $i; break } }
    if ($idx -lt 0) { throw "Clé absente du modèle : $Key" }
    $Lines[$idx] = "$Key = $Value"
    return $Lines
}

function Convert-PathForConf { param([string]$p) return '"' + ($p -replace '\\', '/') + '"' }
$dbInfo = { param($db) '"127.0.0.1;' + $Layout.MariaDbPort + ';' + $cred.user + ';' + $cred.password + ';' + $db + '"' }

# ---- realmd.conf
$r = Get-Content (Join-Path $cfgDir 'realmd.conf.dist.in') -Encoding UTF8
$r = Set-ConfValue $r 'LoginDatabaseInfo'  (& $dbInfo 'realmd')   # (sans point dans realmd, avec point dans mangosd)
$r = Set-ConfValue $r 'LogsDir'            (Convert-PathForConf $logDir)
$r = Set-ConfValue $r 'BindIP'             '"127.0.0.1"'
$r = Set-ConfValue $r 'RealmServerPort'    $Layout.RealmdPort
$r = Set-ConfValue $r 'PidFile'            (Convert-PathForConf (Join-Path $logDir 'realmd.pid'))
$r = Set-ConfValue $r 'WaitAtStartupError' '0'
$r = Set-ConfValue $r 'StrictVersionCheck' '0'   # pas d'empreinte d'intégrité pour le build 5880 ; serveur local
Set-Content -Path (Join-Path $cfgDir 'realmd.conf') -Value $r -Encoding UTF8

# ---- mangosd.conf
$m = Get-Content (Join-Path $cfgDir 'mangosd.conf.dist.in') -Encoding UTF8
$m = Set-ConfValue $m 'DataDir'                (Convert-PathForConf $dataDir)
$m = Set-ConfValue $m 'LogsDir'                (Convert-PathForConf $logDir)
$m = Set-ConfValue $m 'LoginDatabase.Info'     (& $dbInfo 'realmd')
$m = Set-ConfValue $m 'WorldDatabase.Info'     (& $dbInfo 'mangos')
$m = Set-ConfValue $m 'CharacterDatabase.Info' (& $dbInfo 'characters')
$m = Set-ConfValue $m 'LogsDatabase.Info'      (& $dbInfo 'logs')
$m = Set-ConfValue $m 'BindIP'                 '"127.0.0.1"'
$m = Set-ConfValue $m 'WorldServerPort'        $Layout.WorldPort
$m = Set-ConfValue $m 'PidFile'                (Convert-PathForConf (Join-Path $logDir 'mangosd.pid'))
$m = Set-ConfValue $m 'WaitAtStartupError'     '0'
$m = Set-ConfValue $m 'Motd'                   '"Bienvenue dans HomeCraft Edition Vanilla !"'
$m = Set-ConfValue $m 'DBC.Locale'             '255'      # auto-détection : frFR si les DBC viennent du client français
$m = Set-ConfValue $m 'Warden.WinEnabled'      '0'
$m = Set-ConfValue $m 'Anticheat.Enable'       '0'
$m = Set-ConfValue $m 'PartyBot.SkipChecks'    '1'        # un joueur normal peut invoquer des compagnons partout
$m = Set-ConfValue $m 'PartyBot.MaxBots'       '0'
$m = Set-ConfValue $m 'PlayerBot.ShowInWhoList' '1'
$m = Set-ConfValue $m 'PlayerLimit'            '0'        # chaque bot de peuplement compte comme un joueur : pas de file d'attente
$m = Set-ConfValue $m 'Console.Enable'         '1'        # la console sert au launcher (entrée standard) pour les commandes d'administration

# ---- AHBot : hôtel des ventes approvisionné (exigence utilisateur). L'AHBot VMaNGOS ne sert qu'un seul hôtel :
# on choisit celui de la faction du dernier personnage joué (Alliance 55 / Horde 29), Alliance par défaut.
$ahFid = 55; $ahCount = 500
if (Test-MariaDbAlive) {
    try {
        $race = Invoke-Sql -Query "SELECT race FROM characters.characters ORDER BY logout_time DESC LIMIT 1"
        if ($race -and ([int]$race -in 2, 5, 6, 8)) { $ahFid = 29 }
        $n = [int](Invoke-Sql -Query "SELECT COUNT(*) FROM mangos.auctionhousebot")
        if ($n -gt 0) { $ahCount = 3 * $n }   # tirage avec remise : 3N enchères couvrent ~95 % des objets
    } catch { Write-Step "AHBot : valeurs par défaut (base injoignable : $($_.Exception.Message))" }
}
$m = Set-ConfValue $m 'AHBot.Enable'    '1'
$m = Set-ConfValue $m 'AHBot.ah.fid'    $ahFid
$m = Set-ConfValue $m 'AHBot.itemcount' $ahCount
Set-Content -Path (Join-Path $cfgDir 'mangosd.conf') -Value $m -Encoding UTF8

Write-Step "Configurations générées dans $cfgDir (realmd.conf, mangosd.conf)"
