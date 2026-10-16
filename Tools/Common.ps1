# Fonctions partagées par les scripts Tools/. À « dot-sourcer » : . "$PSScriptRoot\Common.ps1"
# Tous les chemins sont relatifs à la racine du projet (dossier parent de Tools/).

$script:ProjectRoot = Split-Path -Parent $PSScriptRoot

function Get-ProjectPath {
    param([Parameter(Mandatory)][string]$Relative)
    return Join-Path $script:ProjectRoot $Relative
}

# Dossiers et ports documentés (voir Documentation/02-Construction.md)
$script:Layout = [ordered]@{
    MariaDbBin   = Get-ProjectPath 'Server\Database\mariadb\bin'
    MariaDbData  = Get-ProjectPath 'UserData\Database'        # données personnelles : hors Server/
    MariaDbIni   = Get-ProjectPath 'Server\Config\my.ini'
    MariaDbPid   = Get-ProjectPath 'UserData\Database\mariadb.pid'
    SqlDumps     = Get-ProjectPath 'Server\Database\sql'
    Credentials  = Get-ProjectPath 'UserData\db-credentials.json'
    Logs         = Get-ProjectPath 'Logs'
    MariaDbPort  = 3315
    RealmdPort   = 3724
    WorldPort    = 8085
}

function Write-Step { param([string]$Message) Write-Host "[$(Get-Date -Format HH:mm:ss)] $Message" }

function Get-DbCredentials {
    # Identifiants du compte SQL local dédié (générés une fois, stockés hors de Server/).
    $path = $script:Layout.Credentials
    if (Test-Path $path) { return Get-Content $path -Raw -Encoding UTF8 | ConvertFrom-Json }
    $bytes = New-Object byte[] 24
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    $cred = [pscustomobject]@{
        user     = 'mangos'
        password = ([Convert]::ToBase64String($bytes) -replace '[^A-Za-z0-9]', 'x')
        rootPassword = ([Convert]::ToBase64String($bytes[0..15]) -replace '[^A-Za-z0-9]', 'y')
    }
    New-Item -ItemType Directory -Force (Split-Path $path) | Out-Null
    $cred | ConvertTo-Json | Set-Content -Path $path -Encoding UTF8
    return $cred
}

function Test-MariaDbAlive {
    param([int]$TimeoutSeconds = 0)
    $admin = Join-Path $script:Layout.MariaDbBin 'mariadb-admin.exe'
    $rootPwd = (Get-DbCredentials).rootPassword
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        # cmd /c pour absorber la sortie d'erreur : sous PowerShell 5.1, « 2>$null » sur un exe natif
        # transforme chaque ligne stderr en erreur (NativeCommandError) et casse le script avec ErrorActionPreference=Stop.
        cmd /c "`"$admin`" --host=127.0.0.1 --port=$($script:Layout.MariaDbPort) --protocol=TCP --user=root --password=$rootPwd ping >nul 2>&1"
        if ($LASTEXITCODE -eq 0) { return $true }
        if ($TimeoutSeconds -gt 0) { Start-Sleep -Milliseconds 500 }
    } while ((Get-Date) -lt $deadline)
    return $false
}

function Invoke-Sql {
    # Exécute une requête ou un fichier SQL via le client mariadb.exe (TCP, loopback).
    param(
        [string]$Database = '',
        [string]$Query = '',
        [string]$File = '',
        [switch]$AsRoot
    )
    $cred = Get-DbCredentials
    $client = Join-Path $script:Layout.MariaDbBin 'mariadb.exe'
    $user = if ($AsRoot) { 'root' } else { $cred.user }
    $pwd  = if ($AsRoot) { $cred.rootPassword } else { $cred.password }
    $args = @('--host=127.0.0.1', "--port=$($script:Layout.MariaDbPort)", '--protocol=TCP', "--user=$user", "--password=$pwd", '--max_allowed_packet=134217728', '--default-character-set=utf8', '--batch', '--skip-column-names')
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
