<#
.SYNOPSIS
    Démarre le serveur MariaDB portable du projet sur 127.0.0.1:3317 (initialise le datadir au premier lancement).
    Si une base répond déjà sur ce port, elle est utilisée telle quelle (et n'est pas « possédée » : ne pas l'arrêter).
#>
[CmdletBinding()]
param([int]$TimeoutSeconds = 60)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$bin = $Layout.MariaDbBin
$mariadbd = Join-Path $bin 'mariadbd.exe'
if (-not (Test-Path $mariadbd)) { throw "MariaDB absent : $mariadbd (lancer Build\Get-Components.ps1 puis extraire dans Server\Database\mariadb)" }

New-Item -ItemType Directory -Force $Layout.MariaDbData, $Layout.Logs, $Layout.ServerConfig | Out-Null
$cred = Get-DbCredentials

# 1. Fichier de configuration géré (régénéré à chaque démarrage : il ne contient rien de personnel).
# Le dump tbcmangos fait 283 Mo : buffer pool à 512 Mo et paquets de 256 Mo pour l'import.
$ini = @"
[mysqld]
basedir=$(Split-Path $bin)
datadir=$($Layout.MariaDbData)
port=$($Layout.MariaDbPort)
bind-address=127.0.0.1
skip-name-resolve
skip-networking=0
lower_case_table_names=2
max_allowed_packet=256M
character-set-server=utf8mb3
collation-server=utf8mb3_general_ci
innodb_buffer_pool_size=512M
innodb_flush_log_at_trx_commit=2
log_error=$($Layout.Logs)\mariadb-error.log
pid-file=$($Layout.MariaDbPid)

[client]
port=$($Layout.MariaDbPort)
default-character-set=utf8mb4
"@
Set-Content -Path $Layout.MariaDbIni -Value $ini -Encoding ASCII

# 2. Initialisation du datadir au premier lancement.
if (-not (Test-Path (Join-Path $Layout.MariaDbData 'mysql'))) {
    Write-Step "Initialisation du datadir $($Layout.MariaDbData)"
    & (Join-Path $bin 'mariadb-install-db.exe') --datadir="$($Layout.MariaDbData)" --password="$($cred.rootPassword)" --port=$($Layout.MariaDbPort) | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "mariadb-install-db a échoué (code $LASTEXITCODE)" }
    # mariadb-install-db écrit son propre my.ini dans le datadir ; le nôtre (--defaults-file) prime.
}

# 3. Démarrage si pas déjà vivant.
if (Test-MariaDbAlive) { Write-Step "MariaDB déjà en service sur 127.0.0.1:$($Layout.MariaDbPort)"; exit 0 }

Write-Step "Démarrage de mariadbd"
$p = Start-Process -FilePath $mariadbd -ArgumentList "--defaults-file=`"$($Layout.MariaDbIni)`"" -WindowStyle Hidden -PassThru
if (-not (Test-MariaDbAlive -TimeoutSeconds $TimeoutSeconds)) {
    if ($p.HasExited) { throw "mariadbd s'est arrêté (code $($p.ExitCode)). Voir $($Layout.Logs)\mariadb-error.log" }
    throw "MariaDB ne répond pas après $TimeoutSeconds s"
}
Write-Step "MariaDB opérationnel (PID $($p.Id)) sur 127.0.0.1:$($Layout.MariaDbPort)"
