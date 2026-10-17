<#
.SYNOPSIS
    Génère Server/Config/authserver.conf, worldserver.conf et modules/*.conf depuis les modèles .conf.dist
    (jamais édités à la main : relancer ce script). Identifiants SQL, chemins, ports loopback, playerbots.
.PARAMETER RandomBots
    Nombre de bots aléatoires qui peuplent le monde (AiPlayerbot.Min/MaxRandomBots). 0 = désactivés.
.PARAMETER Updates
    Masque Updates.EnableDatabases (15 = auth+characters+world+playerbots ; 0 = updater désactivé).
#>
[CmdletBinding()]
param(
    [int]$RandomBots = 40,
    [int]$Updates = 15,
    # Dépôt source du cœur (data/sql/base, updates, modules/*/data/sql) pour l'amorçage automatique des bases par
    # l'updater (Updates.AutoSetup). Vide = pas d'amorçage (les dumps doivent avoir été importés).
    [string]$SourceDir = "C:\hc"
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$cfg = $Layout.ServerConfig
$cred = Get-DbCredentials
$logs = Join-Path $Layout.Logs 'server'
New-Item -ItemType Directory -Force $logs, (Join-Path $cfg 'modules') | Out-Null
function Conf-Path([string]$p) { return ($p -replace '\\', '/') }
function Db-Info([string]$db) { return "`"127.0.0.1;$($Layout.DbPort);$($cred.user);$($cred.password);$db`"" }

function Set-ConfValue {
    # Remplace « Clé = valeur » (clé exacte, hors commentaires). Erreur si la clé n'existe pas dans le modèle.
    param([string[]]$Lines, [string]$Key, [string]$Value)
    $rx = '^\s*' + [regex]::Escape($Key) + '\s*='
    $hit = $false
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        if ($Lines[$i] -match $rx) { $Lines[$i] = "$Key = $Value"; $hit = $true; break }
    }
    if (-not $hit) { throw "clé absente du modèle : $Key" }
    return $Lines
}

function New-ConfFromDist {
    param([string]$Dist, [string]$Out, [hashtable]$Values, [hashtable]$Extra = @{})
    if (-not (Test-Path $Dist)) { throw "modèle introuvable : $Dist" }
    $lines = [System.IO.File]::ReadAllLines($Dist)
    foreach ($k in $Values.Keys) { $lines = Set-ConfValue $lines $k $Values[$k] }
    if ($Extra.Count -gt 0) {
        # Clés absentes du modèle (lues par le cœur patché), ajoutées en fin de fichier.
        $lines += ''
        $lines += '# --- HomeCraft : clés hors modèle ---'
        foreach ($k in $Extra.Keys) { $lines += "$k = $($Extra[$k])" }
    }
    [System.IO.File]::WriteAllLines($Out, $lines, (New-Object System.Text.UTF8Encoding($false)))
    Write-Step "généré : $Out"
}

# authserver.conf
New-ConfFromDist (Join-Path $cfg 'authserver.conf.dist') (Join-Path $cfg 'authserver.conf') ([ordered]@{
    'LogsDir'                 = "`"$(Conf-Path $logs)`""
    'RealmServerPort'         = $Layout.AuthPort
    'BindIP'                  = '"127.0.0.1"'
    'PidFile'                 = "`"$(Conf-Path (Join-Path $logs 'authserver.pid'))`""
    'LoginDatabaseInfo'       = (Db-Info 'acore_auth')
    'Updates.EnableDatabases' = $(if ($Updates -band 1) { 1 } else { 0 })
    'Updates.AutoSetup'       = 1
    'SourceDirectory'         = "`"$(Conf-Path $SourceDir)`""
    'MySQLExecutable'         = "`"$(Conf-Path (Join-Path $Layout.DbBin 'mysql.exe'))`""
})

# worldserver.conf
New-ConfFromDist (Join-Path $cfg 'worldserver.conf.dist') (Join-Path $cfg 'worldserver.conf') ([ordered]@{
    'RealmID'                 = 1
    'WorldServerPort'         = $Layout.WorldPort
    'BindIP'                  = '"127.0.0.1"'
    'LoginDatabaseInfo'       = (Db-Info 'acore_auth')
    'WorldDatabaseInfo'       = (Db-Info 'acore_world')
    'CharacterDatabaseInfo'   = (Db-Info 'acore_characters')
    'DataDir'                 = "`"$(Conf-Path $Layout.ServerData)`""
    'LogsDir'                 = "`"$(Conf-Path $logs)`""
    'PidFile'                 = "`"$(Conf-Path (Join-Path $logs 'worldserver.pid'))`""
    'Console.Enable'          = 1
    'Ra.Enable'               = 0
    'SOAP.Enabled'            = 0
    'Updates.EnableDatabases' = $Updates
    'Updates.AutoSetup'       = 1
    'Warden.Enabled'          = 0          # obligatoire : HermesProxy ne relaie pas Warden (client 3.4.3)
    'FlashAtStart'            = 0
    'SourceDirectory'         = "`"$(Conf-Path $SourceDir)`""
    'MySQLExecutable'         = "`"$(Conf-Path (Join-Path $Layout.DbBin 'mysql.exe'))`""
}) ([ordered]@{
    # Le patch Playerbot d'avril 2026 (PlayerbotsDatabase dans le cœur) lit ces deux clés dans worldserver.conf,
    # avant le chargement des .conf de modules.
    'PlayerbotsDatabaseInfo'             = (Db-Info 'acore_playerbots')
    'Playerbots.Updates.EnableDatabases' = $(if ($Updates -band 8) { 1 } else { 0 })
})
# Le message du jour d'AzerothCore est en base (table acore_auth.motd), pas dans la conf.

# Modules : copie de chaque modèle en .conf, avec surcharges pour playerbots.
Get-ChildItem (Join-Path $cfg 'modules') -Filter '*.conf.dist' | ForEach-Object {
    $out = Join-Path $cfg ('modules\' + ($_.Name -replace '\.dist$', ''))
    $values = [ordered]@{}
    if ($_.Name -eq 'playerbots.conf.dist') {
        $values['PlayerbotsDatabaseInfo']      = (Db-Info 'acore_playerbots')
        $values['AiPlayerbot.MinRandomBots']   = $RandomBots
        $values['AiPlayerbot.MaxRandomBots']   = $RandomBots
        $values['AiPlayerbot.RandomBotAutologin'] = $(if ($RandomBots -gt 0) { 1 } else { 0 })
        # Serveur de commandes TCP du module (8888 par défaut, sur toutes les interfaces, sans authentification) : désactivé.
        $values['AiPlayerbot.CommandServerPort'] = 0
    }
    New-ConfFromDist $_.FullName $out $values
}
Write-Step "Configurations générées dans $cfg"
