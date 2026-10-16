<#
.SYNOPSIS
    Crée le compte SQL local « mangos » et importe les quatre bases VMaNGOS (realmd, characters, mangos, logs)
    depuis Server/Database/sql. Reproduit la procédure du CI VMaNGOS (prepare-sql-test-environment.sh).

.PARAMETER Force
    Ré-importe même si les bases existent déjà. REFUSÉ si la base characters contient des personnages,
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

$imports = [ordered]@{
    realmd     = 'logon.sql'
    characters = 'characters.sql'
    mangos     = 'mangos.sql'
    logs       = 'logs.sql'
}
foreach ($f in $imports.Values) {
    $p = Join-Path $Layout.SqlDumps $f
    if (-not (Test-Path $p)) { throw "Dump manquant : $p" }
}

# Garde-fou : ne jamais écraser des personnages sans accord explicite.
$existing = Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name='characters'"
if ([int]$existing -gt 0) {
    $nbChars = [int](Invoke-Sql -AsRoot -Query "SELECT COUNT(*) FROM characters.characters")
    if (-not $Force) { Write-Step "Bases déjà présentes ($nbChars personnage(s)). Rien à faire (utiliser -Force pour ré-importer)."; exit 0 }
    if ($nbChars -gt 0 -and -not $IKnowThisDeletesCharacters) {
        throw "La base characters contient $nbChars personnage(s). Ré-import refusé sans -IKnowThisDeletesCharacters (faire une sauvegarde d'abord)."
    }
}

Write-Step "Création du compte SQL dédié '$($cred.user)' (loopback uniquement)"
Invoke-Sql -AsRoot -Query @"
CREATE USER IF NOT EXISTS '$($cred.user)'@'127.0.0.1' IDENTIFIED BY '$($cred.password)';
ALTER USER '$($cred.user)'@'127.0.0.1' IDENTIFIED BY '$($cred.password)';
CREATE USER IF NOT EXISTS '$($cred.user)'@'localhost' IDENTIFIED BY '$($cred.password)';
ALTER USER '$($cred.user)'@'localhost' IDENTIFIED BY '$($cred.password)';
"@

foreach ($db in $imports.Keys) {
    Write-Step "Base '$db' : création et import de $($imports[$db])"
    Invoke-Sql -AsRoot -Query "DROP DATABASE IF EXISTS ``$db``; CREATE DATABASE ``$db`` DEFAULT CHARSET utf8mb3 COLLATE utf8mb3_general_ci; GRANT ALL PRIVILEGES ON ``$db``.* TO '$($cred.user)'@'127.0.0.1'; GRANT ALL PRIVILEGES ON ``$db``.* TO '$($cred.user)'@'localhost'; FLUSH PRIVILEGES;"
    $sw = [Diagnostics.Stopwatch]::StartNew()
    Invoke-Sql -AsRoot -Database $db -File (Join-Path $Layout.SqlDumps $imports[$db])
    Write-Step "   importé en $([int]$sw.Elapsed.TotalSeconds) s"
}

Write-Step "Vérification"
foreach ($db in $imports.Keys) {
    $n = Invoke-Sql -Database $db -Query "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='$db'"
    $m = Invoke-Sql -Database $db -Query "SELECT COUNT(*) FROM ``$db``.migrations"
    Write-Step "   $db : $n tables, $m migrations enregistrées"
}
Write-Step "Import terminé."
