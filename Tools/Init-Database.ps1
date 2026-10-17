<#
.SYNOPSIS
    Crée le compte SQL local « acore » et les bases acore_auth / acore_characters / acore_world / acore_playerbots,
    puis importe les dumps du royaume présents dans Server/Database/sql (releases GitHub du dépôt SylvaniaCore-WoTLK :
    fichiers .sql ou .sql.gz dont le nom contient auth, characters ou world). Les tables manquantes de
    acore_playerbots et les mises à jour en attente sont créées par l'updater de worldserver (Updates.AutoSetup).

.PARAMETER Force
    Ré-importe même si les bases existent déjà. REFUSÉ si acore_characters contient des personnages,
    sauf avec -IKnowThisDeletesCharacters.
#>
[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$IKnowThisDeletesCharacters
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

& "$PSScriptRoot\Start-Database.ps1"
$cred = Get-DbCredentials
New-Item -ItemType Directory -Force $Layout.SqlDumps | Out-Null

# Dumps disponibles (par base), .sql ou .sql.gz
$dumps = [ordered]@{}
foreach ($db in 'acore_auth', 'acore_characters', 'acore_world') {
    $key = $db -replace '^acore_', ''
    $f = Get-ChildItem $Layout.SqlDumps -File | Where-Object { $_.Name -match "(^|[_\-.])$key([_\-.]|$)" -and $_.Name -match '\.sql(\.gz)?$' } | Sort-Object Length -Descending | Select-Object -First 1
    if ($f) { $dumps[$db] = $f.FullName }
}
if ($dumps.Count -eq 0) { Write-Step "Aucun dump dans $($Layout.SqlDumps) : les bases seront créées vides (l'updater de worldserver y importera la base du dépôt si SourceDirectory est configuré)." }
else { $dumps.GetEnumerator() | ForEach-Object { Write-Step "dump $($_.Key) : $($_.Value)" } }

# Garde-fou : ne jamais écraser des personnages sans accord explicite.
$existing = Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name='acore_characters'"
if ([int]$existing -gt 0) {
    $hasTable = [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='acore_characters' AND table_name='characters'")
    $nbChars = if ($hasTable) { [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM acore_characters.characters") } else { 0 }
    if (-not $Force) { Write-Step "Bases déjà présentes ($nbChars personnage(s)). Rien à faire (utiliser -Force pour ré-importer)."; exit 0 }
    if ($nbChars -gt 0 -and -not $IKnowThisDeletesCharacters) {
        throw "acore_characters contient $nbChars personnage(s). Ré-import refusé sans -IKnowThisDeletesCharacters (faire une sauvegarde d'abord)."
    }
}

Write-Step "Compte SQL dédié '$($cred.user)' (loopback uniquement)"
Invoke-Sql -AsRoot -Query @"
CREATE USER IF NOT EXISTS '$($cred.user)'@'127.0.0.1' IDENTIFIED BY '$($cred.password)';
ALTER USER '$($cred.user)'@'127.0.0.1' IDENTIFIED BY '$($cred.password)';
CREATE USER IF NOT EXISTS '$($cred.user)'@'localhost' IDENTIFIED BY '$($cred.password)';
ALTER USER '$($cred.user)'@'localhost' IDENTIFIED BY '$($cred.password)';
"@

function Expand-Gz([string]$Path) {
    # Les .sql.gz sont décompressés par .NET (aucun gzip.exe fourni).
    $out = [System.IO.Path]::ChangeExtension($Path, $null).TrimEnd('.')
    if ((Test-Path $out) -and (Get-Item $out).Length -gt 0) { return $out }
    Write-Step "décompression $Path"
    $in = [System.IO.File]::OpenRead($Path); $gz = New-Object System.IO.Compression.GZipStream($in, [System.IO.Compression.CompressionMode]::Decompress); $o = [System.IO.File]::Create($out)
    try { $gz.CopyTo($o) } finally { $o.Close(); $gz.Close(); $in.Close() }
    return $out
}

foreach ($db in $Layout.Databases) {
    Write-Step "Base '$db' : création"
    Invoke-Sql -AsRoot -Query "DROP DATABASE IF EXISTS ``$db``; CREATE DATABASE ``$db`` DEFAULT CHARSET utf8mb4 COLLATE utf8mb4_unicode_ci; GRANT ALL PRIVILEGES ON ``$db``.* TO '$($cred.user)'@'127.0.0.1'; GRANT ALL PRIVILEGES ON ``$db``.* TO '$($cred.user)'@'localhost'; FLUSH PRIVILEGES;"
    if ($dumps.Contains($db)) {
        $file = $dumps[$db]
        if ($file -like '*.gz') { $file = Expand-Gz $file }
        Write-Step "   import de $file"
        $sw = [Diagnostics.Stopwatch]::StartNew()
        Invoke-Sql -AsRoot -Database $db -File $file
        Write-Step "   importé en $([int]$sw.Elapsed.TotalSeconds) s"
    }
}

# Nettoyage après import des dumps du royaume (objets propres au VPS, inutiles ou bloquants en local) :
# 1. triggers/routines/vues/événements dont le DEFINER n'existe pas ici (ex. 'blamacfly'@'%') : MySQL refuse alors
#    TOUTE écriture sur la table (erreur 1449) → aucun compte ne peut être créé. Les triggers Sylvania sur `account`
#    (forçage d'IP d'un compte précis, notification de points de vote) n'ont pas de sens dans l'édition solo.
$users = @(Invoke-Sql -AsRoot -Query "SELECT CONCAT(user,'@',host) FROM mysql.user")
$objs = Invoke-Sql -AsRoot -Query "SELECT CONCAT('TRIGGER;',trigger_schema,';',trigger_name,';',definer) FROM information_schema.triggers WHERE trigger_schema LIKE 'acore_%' UNION ALL SELECT CONCAT(routine_type,';',routine_schema,';',routine_name,';',definer) FROM information_schema.routines WHERE routine_schema LIKE 'acore_%' UNION ALL SELECT CONCAT('VIEW;',table_schema,';',table_name,';',definer) FROM information_schema.views WHERE table_schema LIKE 'acore_%' UNION ALL SELECT CONCAT('EVENT;',event_schema,';',event_name,';',definer) FROM information_schema.events WHERE event_schema LIKE 'acore_%'"
foreach ($o in @($objs)) {
    if (-not $o) { continue }
    $kind, $sch, $name, $definer = $o -split ';'
    if ($users -contains $definer) { continue }
    Write-Step "suppression $kind $sch.$name (DEFINER '$definer' inexistant ici)"
    Invoke-Sql -AsRoot -Query "DROP $kind IF EXISTS ``$sch``.``$name``"
}
# 2. dumps « schéma seul » (auth/characters) : tables de données du module playerbots vides alors que la table `updates`
#    du dump les déclare appliquées → remplissage depuis le module.
$pbBase = Join-Path $script:ProjectRoot 'Build\src\sylvaniacore-wotlk\modules\mod-playerbots\data\sql\characters\base'
if (-not (Test-Path $pbBase)) { $pbBase = 'C:\hc\modules\mod-playerbots\data\sql\characters\base' }
foreach ($t in 'playerbots_names', 'playerbots_guild_names', 'playerbots_arena_team_names') {
    $exists = [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='acore_characters' AND table_name='$t'")
    if ($exists -eq 0) { continue }
    $n = [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM acore_characters.``$t``")
    $f = Join-Path $pbBase "$t.sql"
    if ($n -eq 0 -and (Test-Path $f)) { Write-Step "remplissage de $t depuis le module"; Invoke-Sql -AsRoot -Database acore_characters -File $f }
}
# 3. lignes indispensables au démarrage absentes d'un schéma seul.
$hasSeason = Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='acore_characters' AND table_name='active_arena_season'"
if ([int]$hasSeason -gt 0 -and [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM acore_characters.active_arena_season") -eq 0) {
    Write-Step "active_arena_season vide : saison 8 active (valeur des royaumes Sylvania)"
    Invoke-Sql -AsRoot -Query "INSERT INTO acore_characters.active_arena_season (season_id, season_state) VALUES (8, 1)"
}

Write-Step "Vérification"
foreach ($db in $Layout.Databases) {
    $n = Invoke-Sql -Database $db -Query "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='$db'"
    Write-Step "   $db : $n tables"
}
Write-Step "Import terminé."
