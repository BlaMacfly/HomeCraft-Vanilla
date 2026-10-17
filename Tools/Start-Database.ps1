<#
.SYNOPSIS
    Démarre le serveur MySQL 8.4 portable du projet sur 127.0.0.1:3306 (initialise le datadir au premier lancement).
    Remplace Start-MariaDB.ps1 de l'édition Vanilla : AzerothCore exige MySQL 8 (MariaDB rejetée au démarrage).
#>
[CmdletBinding()]
param([int]$TimeoutSeconds = 90)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$bin = $Layout.DbBin
$mysqld = Join-Path $bin 'mysqld.exe'
if (-not (Test-Path $mysqld)) { throw "MySQL absent : $mysqld (extraire mysql-8.4.9-winx64.zip dans Server\Database\mysql)" }

New-Item -ItemType Directory -Force $Layout.DbData, $Layout.Logs, $Layout.MyIniDir | Out-Null
$cred = Get-DbCredentials
function Ini-Path([string]$p) { return ($p -replace '\\', '/') }

# 1. Fichier de configuration géré (régénéré à chaque démarrage : il ne contient rien de personnel).
#    utf8mb4 / unicode_ci = réglages recommandés par AzerothCore ; sql_mode allégé comme son my.cnf d'exemple.
$ini = @"
[mysqld]
basedir=$(Ini-Path $Layout.DbDir)
datadir=$(Ini-Path $Layout.DbData)
port=$($Layout.DbPort)
bind-address=127.0.0.1
mysqlx=OFF
skip-name-resolve
max_allowed_packet=1G
character-set-server=utf8mb4
collation-server=utf8mb4_unicode_ci
innodb_buffer_pool_size=1G
innodb_redo_log_capacity=512M
innodb_flush_log_at_trx_commit=2
sql_mode=NO_ENGINE_SUBSTITUTION
log_error=$(Ini-Path (Join-Path $Layout.Logs 'mysql-error.log'))
pid-file=$(Ini-Path $Layout.DbPid)
log_bin=OFF

[client]
port=$($Layout.DbPort)
default-character-set=utf8mb4
"@
Set-Content -Path $Layout.DbIni -Value $ini -Encoding ASCII

# 2. Initialisation du datadir au premier lancement (root sans mot de passe, puis mot de passe posé après démarrage).
$fresh = -not (Test-Path (Join-Path $Layout.DbData 'mysql'))
if ($fresh) {
    Write-Step "Initialisation du datadir $($Layout.DbData)"
    & $mysqld --defaults-file="$($Layout.DbIni)" --initialize-insecure | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "mysqld --initialize-insecure a échoué (code $LASTEXITCODE), voir $($Layout.Logs)\mysql-error.log" }
}

# 3. Démarrage si pas déjà vivant.
if (-not $fresh -and (Test-DbAlive)) { Write-Step "MySQL déjà en service sur 127.0.0.1:$($Layout.DbPort)"; exit 0 }

Write-Step "Démarrage de mysqld"
$extra = @()
$initFile = Join-Path $Layout.DbData 'init-root.sql'
if ($fresh) {
    # Premier démarrage : root est sans mot de passe et, avec skip-name-resolve, « root@localhost » n'accepte pas
    # les connexions TCP sur 127.0.0.1. Un --init-file (exécuté par le serveur, sans connexion) pose le mot de passe
    # et crée root@127.0.0.1. Le fichier est supprimé juste après.
    @"
ALTER USER 'root'@'localhost' IDENTIFIED BY '$($cred.rootPassword)';
CREATE USER IF NOT EXISTS 'root'@'127.0.0.1' IDENTIFIED BY '$($cred.rootPassword)';
GRANT ALL PRIVILEGES ON *.* TO 'root'@'127.0.0.1' WITH GRANT OPTION;
FLUSH PRIVILEGES;
"@ | Set-Content -Path $initFile -Encoding ASCII
    $extra = @("--init-file=`"$initFile`"")
}
$p = Start-Process -FilePath $mysqld -ArgumentList (@("--defaults-file=`"$($Layout.DbIni)`"") + $extra) -WindowStyle Hidden -PassThru
$alive = Test-DbAlive -TimeoutSeconds $TimeoutSeconds
if ($fresh) { Remove-Item $initFile -Force -ErrorAction SilentlyContinue }
if (-not $alive) {
    if ($p.HasExited) { throw "mysqld s'est arrêté (code $($p.ExitCode)). Voir $($Layout.Logs)\mysql-error.log" }
    throw "MySQL ne répond pas après $TimeoutSeconds s"
}
Write-Step "MySQL opérationnel (PID $($p.Id))"
