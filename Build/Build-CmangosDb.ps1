<#
.SYNOPSIS
    Construit les bases CMaNGOS Classic (mangos, characters, realmd, logs) avec les données classic-db, le module
    playerbots et les traductions françaises, et les exporte dans Server\Database\cmangos-sql.
.DESCRIPTION
    Travaille sur une instance MariaDB temporaire (Build\cmangos-db, port 3316, loopback) : la base de l'utilisateur
    (UserData\Database, port 3315) n'est jamais touchée.
      1. Installation officielle : InstallFullDB.sh de classic-db (Git Bash), avec AHBot, locales et playerbots.
      2. Traductions françaises : classic-db ne fournit en français que les textes diffusés (broadcast_text). Les
         colonnes loc2 (frFR) des quêtes, créatures, objets, objets du monde, pages, options de dialogue et points
         d'intérêt sont reprises de la base monde VMaNGOS (Server\Database\sql\mangos.sql, mêmes identifiants
         Blizzard) complétée par les traductions HomeCraft (Server\Database\custom\locales_*.sql). Seules les entrées
         présentes dans la base CMaNGOS sont copiées.
      3. Export avec mariadb-dump, sans comptes SQL.
    Prérequis : Build\Build-Cmangos.ps1 (sources), Build\classic-db au commit figé, MariaDB dans Server\Database\mariadb.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
$classicDbCommit = 'f460448'
$port = 3316
$work = Join-Path $PSScriptRoot 'cmangos-db'
$data = Join-Path $work 'data'
$ini = Join-Path $work 'my.ini'
$classicDb = Join-Path $PSScriptRoot 'classic-db'
$coreSrc = Join-Path $PSScriptRoot 'cmangos-src'
$out = Get-ProjectPath 'Server\Database\cmangos-sql'
$bin = $Layout.MariaDbBin
$bash = 'C:\Program Files\Git\bin\bash.exe'
foreach ($t in (Join-Path $bin 'mariadbd.exe'), $bash, (Join-Path $coreSrc 'sql\base\mangos.sql')) { if (-not (Test-Path $t)) { throw "introuvable : $t" } }

function ConvertTo-BashPath([string]$p) { '/' + $p.Substring(0, 1).ToLowerInvariant() + ($p.Substring(2) -replace '\\', '/') }

# Mots de passe jetables : l'instance ne vit que le temps de la construction.
$rootPwd = -join ((48..57) + (97..122) | Get-Random -Count 24 | ForEach-Object { [char]$_ })
$userPwd = -join ((48..57) + (97..122) | Get-Random -Count 24 | ForEach-Object { [char]$_ })

function Invoke-BuildSql([string]$Query, [string]$Database = '', [string]$File = '') {
    $a = "--host=127.0.0.1 --port=$port --protocol=TCP --user=root --password=$rootPwd --max_allowed_packet=1073741824 --default-character-set=utf8 --batch --skip-column-names"
    if ($Database) { $a += " $Database" }
    if ($File) { $r = cmd /c "`"$(Join-Path $bin 'mariadb.exe')`" $a < `"$File`" 2>&1" }
    else { $r = cmd /c "`"$(Join-Path $bin 'mariadb.exe')`" $a --execute=`"$Query`" 2>&1" }
    if ($LASTEXITCODE -ne 0) { throw "Échec SQL : $r" }
    $r
}

# 0. classic-db au commit figé
cmd /c "git -C `"$classicDb`" checkout -q -f $classicDbCommit >nul 2>&1"
if ($LASTEXITCODE -ne 0) { throw "classic-db : commit $classicDbCommit introuvable (cloner https://github.com/cmangos/classic-db dans $classicDb)" }
cmd /c "git -C `"$classicDb`" clean -q -fdx >nul 2>&1"

# 1. Instance MariaDB temporaire, repartie de zéro
$admin = Join-Path $bin 'mariadb-admin.exe'
cmd /c "`"$admin`" --host=127.0.0.1 --port=$port --protocol=TCP --user=root ping >nul 2>&1"
if ($LASTEXITCODE -eq 0) { throw "le port $port est déjà utilisé : arrêter l'instance précédente" }
if (Test-Path $work) { Remove-Item -Recurse -Force $work }
New-Item -ItemType Directory -Force $data | Out-Null
& (Join-Path $bin 'mariadb-install-db.exe') --datadir="$data" --password="$rootPwd" --port=$port | Out-Null
if ($LASTEXITCODE -ne 0) { throw "mariadb-install-db a échoué" }
@"
[mysqld]
basedir=$(Split-Path $bin)
datadir=$data
port=$port
bind-address=127.0.0.1
skip-name-resolve
max_allowed_packet=1G
character-set-server=utf8mb3
collation-server=utf8mb3_general_ci
innodb_buffer_pool_size=1G
innodb_flush_log_at_trx_commit=0
log_error=$work\error.log
"@ | Set-Content -Path $ini -Encoding ASCII
$proc = Start-Process -FilePath (Join-Path $bin 'mariadbd.exe') -ArgumentList "--defaults-file=`"$ini`"" -WindowStyle Hidden -PassThru
try {
    $deadline = (Get-Date).AddSeconds(60)
    do {
        Start-Sleep -Milliseconds 500
        cmd /c "`"$admin`" --host=127.0.0.1 --port=$port --protocol=TCP --user=root --password=$rootPwd ping >nul 2>&1"
    } while ($LASTEXITCODE -ne 0 -and (Get-Date) -lt $deadline)
    if ($LASTEXITCODE -ne 0) { throw "l'instance MariaDB temporaire ne répond pas (voir $work\error.log)" }
    Write-Step "instance MariaDB temporaire sur 127.0.0.1:$port"

    # 2. Installation officielle classic-db (mêmes noms de bases que l'édition VMaNGOS)
    @"
MYSQL_HOST="127.0.0.1"
MYSQL_PORT="$port"
MYSQL_USERNAME="mangos"
MYSQL_PASSWORD="$userPwd"
MYSQL_USERIP="127.0.0.1"
WORLD_DB_NAME="mangos"
REALM_DB_NAME="realmd"
CHAR_DB_NAME="characters"
LOGS_DB_NAME="logs"
MYSQL_PATH="$(ConvertTo-BashPath (Join-Path $bin 'mariadb.exe'))"
MYSQL_DUMP_PATH="$(ConvertTo-BashPath (Join-Path $bin 'mariadb-dump.exe'))"
CORE_PATH="$(ConvertTo-BashPath $coreSrc)"
LOCALES="YES"
FORCE_WAIT="NO"
DEV_UPDATES="NO"
AHBOT="YES"
PLAYERBOTS_DB="YES"
"@ | Set-Content -Path (Join-Path $classicDb 'InstallFullDB.config') -Encoding ASCII
    Write-Step "installation classic-db (InstallFullDB.sh -InstallAll, 10 à 20 minutes)"
    $log = Join-Path $work 'InstallFullDB.log'
    Push-Location $classicDb
    try { cmd /c "`"$bash`" InstallFullDB.sh -InstallAll root $rootPwd DeleteAll > `"$log`" 2>&1" } finally { Pop-Location }
    Remove-Item (Join-Path $classicDb 'InstallFullDB.config') -Force
    if ($LASTEXITCODE -ne 0 -or (Select-String -Path $log -Pattern 'FAILED!|>>> ERROR' -CaseSensitive -Quiet)) {
        Get-Content $log | Select-String 'FAILED!|>>> ERROR' -CaseSensitive | Select-Object -First 10 | ForEach-Object { Write-Host $_ }
        throw "InstallFullDB.sh a échoué (voir $log)"
    }

    # 3. Traductions françaises depuis la base monde VMaNGOS + compléments HomeCraft
    Write-Step "import de la base monde VMaNGOS (source des traductions françaises)"
    Invoke-BuildSql "DROP DATABASE IF EXISTS vmangos; CREATE DATABASE vmangos DEFAULT CHARSET utf8mb3;" | Out-Null
    Invoke-BuildSql '' 'vmangos' (Get-ProjectPath 'Server\Database\sql\mangos.sql') | Out-Null
    foreach ($f in Get-ChildItem (Get-ProjectPath 'Server\Database\custom') -Filter 'locales_*.sql') {
        Invoke-BuildSql '' 'vmangos' $f.FullName | Out-Null
    }
    # table CMaNGOS, table de référence (l'entrée doit exister), clé, colonnes loc2
    $tables = @(
        @('locales_quest', 'quest_template', 'entry', 'Title_loc2,Details_loc2,Objectives_loc2,OfferRewardText_loc2,RequestItemsText_loc2,EndText_loc2'),
        @('locales_creature', 'creature_template', 'entry', 'name_loc2,subname_loc2'),
        @('locales_item', 'item_template', 'entry', 'name_loc2,description_loc2'),
        @('locales_gameobject', 'gameobject_template', 'entry', 'name_loc2'),
        @('locales_page_text', 'page_text', 'entry', 'Text_loc2'),
        @('locales_points_of_interest', 'points_of_interest', 'entry', 'icon_name_loc2')
    )
    foreach ($t in $tables) {
        $name, $ref, $key, $cols = $t
        $colList = $cols -split ','
        $set = ($colList | ForEach-Object { "l.$_ = IF(COALESCE(l.$_,'')='', v.$_, l.$_)" }) -join ', '
        $notEmpty = ($colList | ForEach-Object { "COALESCE(v.$_,'')<>''" }) -join ' OR '
        # Entrées sans ligne de traduction côté CMaNGOS : on crée la ligne, puis on remplit les loc2 vides.
        Invoke-BuildSql "INSERT IGNORE INTO mangos.$name ($key) SELECT v.$key FROM vmangos.$name v JOIN mangos.$ref r ON r.$key = v.$key WHERE $notEmpty;" | Out-Null
        Invoke-BuildSql "UPDATE mangos.$name l JOIN vmangos.$name v ON v.$key = l.$key SET $set;" | Out-Null
    }
    $set = "l.option_text_loc2 = IF(COALESCE(l.option_text_loc2,'')='', v.option_text_loc2, l.option_text_loc2), l.box_text_loc2 = IF(COALESCE(l.box_text_loc2,'')='', v.box_text_loc2, l.box_text_loc2)"
    Invoke-BuildSql "INSERT IGNORE INTO mangos.locales_gossip_menu_option (menu_id, id) SELECT v.menu_id, v.id FROM vmangos.locales_gossip_menu_option v JOIN mangos.gossip_menu_option r ON r.menu_id = v.menu_id AND r.id = v.id WHERE COALESCE(v.option_text_loc2,'')<>'' OR COALESCE(v.box_text_loc2,'')<>'';" | Out-Null
    Invoke-BuildSql "UPDATE mangos.locales_gossip_menu_option l JOIN vmangos.locales_gossip_menu_option v ON v.menu_id = l.menu_id AND v.id = l.id SET $set;" | Out-Null
    Invoke-BuildSql "DROP DATABASE vmangos;" | Out-Null
    foreach ($q in @(
            @('quêtes', "SELECT COUNT(*) FROM mangos.quest_template q JOIN mangos.locales_quest l USING(entry) WHERE COALESCE(l.Title_loc2,'')<>''", "SELECT COUNT(*) FROM mangos.quest_template"),
            @('créatures', "SELECT COUNT(*) FROM mangos.creature_template q JOIN mangos.locales_creature l USING(entry) WHERE COALESCE(l.name_loc2,'')<>''", "SELECT COUNT(*) FROM mangos.creature_template"),
            @('objets', "SELECT COUNT(*) FROM mangos.item_template q JOIN mangos.locales_item l USING(entry) WHERE COALESCE(l.name_loc2,'')<>''", "SELECT COUNT(*) FROM mangos.item_template"),
            @('textes diffusés', "SELECT COUNT(*) FROM mangos.broadcast_text_locale WHERE locale='frFR'", "SELECT COUNT(*) FROM mangos.broadcast_text"))) {
        Write-Step ("   français : {0} {1} / {2}" -f $q[0], (Invoke-BuildSql $q[1]), (Invoke-BuildSql $q[2]))
    }

    # Comptes de démonstration de realmd.sql (ADMINISTRATOR, GAMEMASTER, MODERATOR, PLAYER) : leurs mots de passe
    # sont publics. Le joueur crée ses comptes dans le launcher.
    Invoke-BuildSql "DELETE FROM realmd.account; DELETE FROM realmd.account_banned; DELETE FROM realmd.realmcharacters; ALTER TABLE realmd.account AUTO_INCREMENT = 1; UPDATE realmd.realmlist SET name = 'HomeCraft', realmbuilds = '5875 5880' WHERE id = 1;" | Out-Null

    # 4. Export (structure + données, sans comptes SQL)
    New-Item -ItemType Directory -Force $out | Out-Null
    foreach ($db in 'realmd', 'characters', 'mangos', 'logs') {
        $file = Join-Path $out "$db.sql"
        cmd /c "`"$(Join-Path $bin 'mariadb-dump.exe')`" --host=127.0.0.1 --port=$port --protocol=TCP --user=root --password=$rootPwd --default-character-set=utf8 --single-transaction --routines --hex-blob --skip-comments --skip-dump-date $db > `"$file`""
        if ($LASTEXITCODE -ne 0) { throw "export de $db impossible" }
        Write-Step ("   {0}.sql : {1:N1} Mo" -f $db, ((Get-Item $file).Length / 1MB))
    }
    Write-Step "bases exportées dans Server\Database\cmangos-sql"
}
finally {
    cmd /c "`"$admin`" --host=127.0.0.1 --port=$port --protocol=TCP --user=root --password=$rootPwd shutdown >nul 2>&1"
    if (-not $proc.WaitForExit(60000)) { $proc.Kill() }
}
