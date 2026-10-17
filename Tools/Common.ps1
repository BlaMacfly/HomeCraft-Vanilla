# Fonctions partagées par les scripts Tools/. À « dot-sourcer » : . "$PSScriptRoot\Common.ps1"
# Tous les chemins sont relatifs à la racine du projet (dossier parent de Tools/). Adapté de ../HomeCraft/Tools/Common.ps1.
# Base de données : MySQL 8.4 Community (zip portable). AzerothCore rejette MariaDB (contrôle de version naïf,
# identique upstream) : voir Documentation/03-Base-de-donnees.md.

$script:ProjectRoot = Split-Path -Parent $PSScriptRoot

function Get-ProjectPath {
    param([Parameter(Mandatory)][string]$Relative)
    return Join-Path $script:ProjectRoot $Relative
}

# Dossiers et ports documentés (voir Documentation/00-Analyse-HomeCraft-et-cadrage.md §7)
$script:Layout = [ordered]@{
    ServerBin    = Get-ProjectPath 'Server\acore'              # authserver.exe, worldserver.exe, extracteurs, DLL
    ServerConfig = Get-ProjectPath 'Server\acore\configs'      # *.conf.dist (modèles) et *.conf générés ; sous Windows AzerothCore
                                                               # lit les conf de modules dans <dossier courant>/configs/modules/
    MyIniDir     = Get-ProjectPath 'Server\Config'             # my.ini de MySQL (hors du dossier des serveurs)
    ServerData   = Get-ProjectPath 'Server\Data'               # dbc, maps, vmaps, mmaps, Cameras (extraits du client)
    DbDir        = Get-ProjectPath 'Server\Database\mysql'     # MySQL 8.4 portable (bin, share, lib)
    DbBin        = Get-ProjectPath 'Server\Database\mysql\bin'
    DbData       = Get-ProjectPath 'UserData\Database'         # données personnelles : hors Server/
    DbIni        = Get-ProjectPath 'Server\Config\my.ini'
    DbPid        = Get-ProjectPath 'UserData\Database\mysqld.pid'
    SqlDumps     = Get-ProjectPath 'Server\Database\sql'       # dumps du royaume (releases GitHub) : auth, characters, world
    SqlCustom    = Get-ProjectPath 'Server\Database\custom'
    Credentials  = Get-ProjectPath 'UserData\db-credentials.json'
    Logs         = Get-ProjectPath 'Logs'
    DbPort       = 3306   # port pré-rempli par Keira3 (2026-10-17 ; avant 3315, partagé avec l'édition Vanilla)
    AuthPort     = 3724
    WorldPort    = 8085
    Databases    = @('acore_auth', 'acore_characters', 'acore_world', 'acore_playerbots')
    ClientBuild  = '3.3.5.12340'
    ShortRoot    = 'C:\hcw'                                    # jonction vers la racine : AzerothCore refuse « -c » avec un espace dans le chemin
}

function Write-Step { param([string]$Message) Write-Host "[$(Get-Date -Format HH:mm:ss)] $Message" }

function Get-DbCredentials {
    # Identifiants du compte SQL local dédié (générés une fois, stockés hors de Server/).
    $path = $script:Layout.Credentials
    if (Test-Path $path) { return Get-Content $path -Raw -Encoding UTF8 | ConvertFrom-Json }
    $bytes = New-Object byte[] 24
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    $cred = [pscustomobject]@{
        user     = 'acore'
        password = ([Convert]::ToBase64String($bytes) -replace '[^A-Za-z0-9]', 'x')
        rootPassword = 'root'   # pré-rempli par Keira3 ; base limitée à 127.0.0.1 (décision du 2026-10-17)
    }
    New-Item -ItemType Directory -Force (Split-Path $path) | Out-Null
    $cred | ConvertTo-Json | Set-Content -Path $path -Encoding UTF8
    return $cred
}

function Test-DbAlive {
    param([int]$TimeoutSeconds = 0)
    $admin = Join-Path $script:Layout.DbBin 'mysqladmin.exe'
    $rootPwd = (Get-DbCredentials).rootPassword
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        # cmd /c pour absorber la sortie d'erreur : sous PowerShell 5.1, « 2>$null » sur un exe natif
        # transforme chaque ligne stderr en erreur (NativeCommandError) et casse le script avec ErrorActionPreference=Stop.
        cmd /c "`"$admin`" --host=127.0.0.1 --port=$($script:Layout.DbPort) --protocol=TCP --user=root --password=$rootPwd ping >nul 2>&1"
        if ($LASTEXITCODE -eq 0) { return $true }
        if ($TimeoutSeconds -gt 0) { Start-Sleep -Milliseconds 500 }
    } while ((Get-Date) -lt $deadline)
    return $false
}

function Invoke-Sql {
    # Exécute une requête ou un fichier SQL via le client mysql.exe (TCP, loopback).
    param(
        [string]$Database = '',
        [string]$Query = '',
        [string]$File = '',
        [switch]$AsRoot
    )
    $cred = Get-DbCredentials
    $client = Join-Path $script:Layout.DbBin 'mysql.exe'
    $user = if ($AsRoot) { 'root' } else { $cred.user }
    $pwd  = if ($AsRoot) { $cred.rootPassword } else { $cred.password }
    $args = @('--host=127.0.0.1', "--port=$($script:Layout.DbPort)", '--protocol=TCP', "--user=$user", "--password=$pwd", '--max_allowed_packet=1073741824', '--default-character-set=utf8mb4', '--batch', '--skip-column-names')
    if ($Database) { $args += $Database }
    if ($File) {
        # Import de fichier : on passe par le flux d'entrée standard de cmd pour éviter de charger le dump en mémoire.
        $quoted = ($args | ForEach-Object { '"' + $_ + '"' }) -join ' '
        cmd /c "`"$client`" $quoted < `"$File`""
    } else {
        $args += @('--execute', $Query)
        & $client @args
    }
    if ($LASTEXITCODE -ne 0) { throw "Échec SQL (code $LASTEXITCODE)" }
}
