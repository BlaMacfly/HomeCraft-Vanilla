<#
.SYNOPSIS
    Crée le compte SQL local et importe les quatre bases CMaNGOS-tbc (tbcrealmd, tbccharacters, tbcmangos, tbclogs)
    depuis les dumps complets de TBC-DB (Server/Database/sql, archive tbc-all-db.zip), puis les compléments
    Server/Database/custom/*.sql (ordre alphabétique) dans la base monde. Les dumps contiennent déjà les tables
    PlayerBots (ai_playerbot_*) et AHBot (ahbot_*) : aucun SQL de module à appliquer en plus.

.PARAMETER Force
    Ré-importe même si les bases existent déjà. REFUSÉ si tbccharacters contient des personnages,
    sauf avec -IKnowThisDeletesCharacters.
#>
[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$IKnowThisDeletesCharacters
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

& "$PSScriptRoot\Start-MariaDB.ps1"
$cred = Get-DbCredentials

foreach ($db in $Databases.Values) {
    $p = Join-Path $Layout.SqlDumps "$db.sql"
    if (-not (Test-Path $p)) { throw "Dump manquant : $p" }
}

# Garde-fou : ne jamais écraser des personnages sans accord explicite.
$charsDb = $Databases.characters
$existing = Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name='$charsDb'"
if ([int]$existing -gt 0) {
    $nbChars = [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM $charsDb.characters")
    if (-not $Force) { Write-Step "Bases déjà présentes ($nbChars personnage(s)). Rien à faire (utiliser -Force pour ré-importer)."; exit 0 }
    if ($nbChars -gt 0 -and -not $IKnowThisDeletesCharacters) {
        throw "La base $charsDb contient $nbChars personnage(s). Ré-import refusé sans -IKnowThisDeletesCharacters (faire une sauvegarde d'abord)."
    }
}

Write-Step "Création du compte SQL dédié '$($cred.user)' (loopback uniquement)"
Invoke-Sql -AsRoot -Query @"
CREATE USER IF NOT EXISTS '$($cred.user)'@'127.0.0.1' IDENTIFIED BY '$($cred.password)';
ALTER USER '$($cred.user)'@'127.0.0.1' IDENTIFIED BY '$($cred.password)';
CREATE USER IF NOT EXISTS '$($cred.user)'@'localhost' IDENTIFIED BY '$($cred.password)';
ALTER USER '$($cred.user)'@'localhost' IDENTIFIED BY '$($cred.password)';
"@

foreach ($db in $Databases.Values) {
    Write-Step "Base '$db' : création et import de $db.sql"
    Invoke-Sql -AsRoot -Query "DROP DATABASE IF EXISTS ``$db``; CREATE DATABASE ``$db`` DEFAULT CHARSET utf8mb3 COLLATE utf8mb3_general_ci; GRANT ALL PRIVILEGES ON ``$db``.* TO '$($cred.user)'@'127.0.0.1'; GRANT ALL PRIVILEGES ON ``$db``.* TO '$($cred.user)'@'localhost'; FLUSH PRIVILEGES;"
    $sw = [Diagnostics.Stopwatch]::StartNew()
    Invoke-Sql -AsRoot -Database $db -File (Join-Path $Layout.SqlDumps "$db.sql")
    Write-Step "   importé en $([int]$sw.Elapsed.TotalSeconds) s"
    if ($db -eq $Databases.world -and (Test-Path $Layout.SqlCustom)) {
        foreach ($f in Get-ChildItem $Layout.SqlCustom -Filter '*.sql' | Sort-Object Name) {
            Write-Step "   import : complément $($f.Name)"
            Invoke-Sql -AsRoot -Database $db -File $f.FullName
        }
    }
}

# Les comptes d'exemple du dump (ADMINISTRATOR, GAMEMASTER, MODERATOR, PLAYER) ont des mots de passe publics :
# supprimés à l'import (aucun personnage ne leur appartient encore).
$realmDb = $Databases.realmd
Invoke-Sql -AsRoot -Database $realmDb -Query "DELETE FROM account WHERE username IN ('ADMINISTRATOR','GAMEMASTER','MODERATOR','PLAYER')"
Write-Step "Comptes d'exemple du dump supprimés"

Write-Step "Vérification"
foreach ($db in $Databases.Values) {
    $n = Invoke-Sql -Database $db -Query "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='$db'"
    Write-Step "   $db : $n tables"
}
$nCreatures = Invoke-Sql -Database $Databases.world -Query "SELECT COUNT(*) FROM creature"
$nQuests = Invoke-Sql -Database $Databases.world -Query "SELECT COUNT(*) FROM quest_template"
Write-Step "   monde : $nCreatures créatures placées, $nQuests quêtes"
Write-Step "Import terminé."
