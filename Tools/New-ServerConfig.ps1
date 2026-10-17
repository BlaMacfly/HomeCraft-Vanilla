<#
.SYNOPSIS
    Génère Server/Config/{realmd,mangosd,aiplayerbot,ahbot}.conf à partir des modèles *.conf.dist du build
    CMaNGOS-tbc (copiés une fois depuis Server/mangos-tbc dans Server/Config). Les fichiers générés sont « gérés » :
    régénérés à chaque démarrage, ne pas les éditer à la main. Le launcher (server.rs) reproduit ce script.

    mangosd et realmd sont lancés avec Server/mangos-tbc comme dossier courant : le core y cherche legacy.dll
    (OSSL_PROVIDER_set_default_search_path(current_path)), sinon arrêt immédiat « Failed to load Legacy provider ».
    mangosd reçoit « -c mangosd.conf -a ahbot.conf -p aiplayerbot.conf ». anticheat.conf (lu dans le dossier courant)
    n'est pas écrit dans le dossier du programme : le module est désactivé par variables d'environnement
    (Anticheat_Enable=0, Anticheat_Warden_Enable=0, Anticheat_Antispam_Enable=0) et ses défauts sont déjà à 0.
#>
[CmdletBinding()]
param(
    [int]$RandomBots = 50   # PlayerBots aléatoires (défaut du module : 1000, trop lourd pour un PC solo)
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$cred = Get-DbCredentials
$cfgDir  = $Layout.ServerConfig
$dataDir = $Layout.ServerData        # dbc/maps/vmaps/mmaps extraits du client (non redistribuables)
$logDir  = Join-Path $Layout.Logs 'server'
New-Item -ItemType Directory -Force $cfgDir, $dataDir, $logDir | Out-Null

# Modèles : copiés depuis le build s'ils manquent (ou s'ils ont changé après une mise à jour du core).
foreach ($n in 'realmd', 'mangosd', 'aiplayerbot', 'ahbot') {
    $src = Join-Path $Layout.ServerBin "$n.conf.dist"
    $dst = Join-Path $cfgDir "$n.conf.dist"
    if (-not (Test-Path $src)) { throw "Modèle absent du build : $src" }
    if (-not (Test-Path $dst) -or (Get-FileHash $src).Hash -ne (Get-FileHash $dst).Hash) { Copy-Item $src $dst -Force }
}

function Set-ConfValue {
    # Remplace « Clé = valeur » (première occurrence en début de ligne). Erreur si la clé n'existe pas dans le modèle :
    # cela signale un changement de format CMaNGOS plutôt que de l'ignorer en silence.
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
function Read-Dist { param($n) return ,[string[]](Get-Content (Join-Path $cfgDir "$n.conf.dist") -Encoding UTF8) }
function Write-Conf { param($n, [string[]]$lines) [IO.File]::WriteAllLines((Join-Path $cfgDir "$n.conf"), $lines, (New-Object Text.UTF8Encoding $false)) }

# ---- realmd.conf
$r = Read-Dist 'realmd'
$r = Set-ConfValue $r 'LoginDatabaseInfo'  (& $dbInfo $Databases.realmd)
$r = Set-ConfValue $r 'LogsDir'            (Convert-PathForConf $logDir)
$r = Set-ConfValue $r 'BindIP'             '"127.0.0.1"'
$r = Set-ConfValue $r 'RealmServerPort'    $Layout.RealmdPort
$r = Set-ConfValue $r 'PidFile'            (Convert-PathForConf (Join-Path $logDir 'realmd.pid'))
$r = Set-ConfValue $r 'WaitAtStartupError' '0'
$r = Set-ConfValue $r 'StrictVersionCheck' '0'
Write-Conf 'realmd' $r

# ---- mangosd.conf
$m = Read-Dist 'mangosd'
$m = Set-ConfValue $m 'DataDir'               (Convert-PathForConf $dataDir)
$m = Set-ConfValue $m 'LogsDir'               (Convert-PathForConf $logDir)
$m = Set-ConfValue $m 'LoginDatabaseInfo'     (& $dbInfo $Databases.realmd)
$m = Set-ConfValue $m 'WorldDatabaseInfo'     (& $dbInfo $Databases.world)
$m = Set-ConfValue $m 'CharacterDatabaseInfo' (& $dbInfo $Databases.characters)
$m = Set-ConfValue $m 'LogsDatabaseInfo'      (& $dbInfo $Databases.logs)
$m = Set-ConfValue $m 'BindIP'                '"127.0.0.1"'
$m = Set-ConfValue $m 'WorldServerPort'       $Layout.WorldPort
$m = Set-ConfValue $m 'PidFile'               (Convert-PathForConf (Join-Path $logDir 'mangosd.pid'))
$m = Set-ConfValue $m 'WaitAtStartupError'    '0'
$m = Set-ConfValue $m 'Motd'                  '"Bienvenue dans HomeCraft Edition BC !"'
$m = Set-ConfValue $m 'DBC.Locale'            '255'     # auto-détection : frFR si les DBC viennent du client français
$m = Set-ConfValue $m 'GameType'              '0'       # royaume Normal (JcE) : serveur solo / famille
$m = Set-ConfValue $m 'Expansion'             '1'       # The Burning Crusade
$m = Set-ConfValue $m 'Console.Enable'        '1'       # console sur l'entrée standard : utilisée par le launcher
$m = Set-ConfValue $m 'Ra.Enable'             '0'
$m = Set-ConfValue $m 'SOAP.Enabled'          '0'
Write-Conf 'mangosd' $m

# ---- aiplayerbot.conf (module cmangos/playerbots, compilé dans le build « with-all »)
$b = Read-Dist 'aiplayerbot'
$b = Set-ConfValue $b 'AiPlayerbot.Enabled'              '1'
$b = Set-ConfValue $b 'AiPlayerbot.MinRandomBots'        $RandomBots
$b = Set-ConfValue $b 'AiPlayerbot.MaxRandomBots'        $RandomBots
$b = Set-ConfValue $b 'AiPlayerbot.RandomBotAccountCount' ([Math]::Max(1, [Math]::Ceiling($RandomBots / 9)))
Write-Conf 'aiplayerbot' $b
# Le module lit aiplayerbot.conf dans le dossier courant (Server/mangos-tbc) et ignore l'option -p de mangosd : copie.
[IO.File]::WriteAllLines((Join-Path $Layout.ServerBin 'aiplayerbot.conf'), $b, (New-Object Text.UTF8Encoding $false))

# ---- ahbot.conf (AHBot intégré CMaNGOS : réglages par qualité/source d'objet, valeurs du modèle conservées)
Write-Conf 'ahbot' (Read-Dist 'ahbot')


Write-Step "Configurations générées dans $cfgDir (realmd, mangosd, aiplayerbot, ahbot ; $RandomBots bots aléatoires)"
