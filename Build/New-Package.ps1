<#
.SYNOPSIS
    Construit la version portable distribuable dans Build/package/HomeCraft-Edition-Vanilla-v<version>/ :
    launcher signé, serveur, MariaDB allégée, dumps, HeidiSQL, documentation, licences et sources GPL,
    manifeste de fichiers (empreintes) et SHA256SUMS.txt. Avec -Zip, produit aussi l'archive .zip.
    Jamais inclus : Client/, Server/Data/, UserData/, Logs/, Build/downloads/.
#>
[CmdletBinding()]
param([switch]$Zip)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$cargo = Get-Content (Join-Path $root 'Launcher\Cargo.toml') -Raw
$version = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
$name = "HomeCraft-Edition-Vanilla-v$version"
$out = Join-Path $PSScriptRoot "package\$name"
$launcher = Join-Path $root 'Launcher\target\release\vanilla-solo-launcher.exe'
if (-not (Test-Path $launcher)) { throw "Launcher non compilé : $launcher" }
$sig = Get-AuthenticodeSignature $launcher
if (-not $sig.SignerCertificate) { throw "Le launcher n'est pas signé (Build\Sign-Launcher.ps1)" }

function Step { param($m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force $out | Out-Null
function CopyTree { param($src, $dst, [string[]]$Exclude = @())
    New-Item -ItemType Directory -Force $dst | Out-Null
    Get-ChildItem $src -Recurse -File | ForEach-Object {
        $rel = $_.FullName.Substring($src.Length + 1)
        if ($Exclude | Where-Object { $rel -like $_ }) { return }
        $to = Join-Path $dst $rel
        New-Item -ItemType Directory -Force (Split-Path $to) | Out-Null
        Copy-Item $_.FullName $to
    }
}

Step "Launcher"
Copy-Item $launcher (Join-Path $out 'HomeCraft Edition Vanilla.exe')

Step "Serveur CMaNGOS (avec ses modèles de configuration et extracteurs), dumps"
foreach ($f in 'mangosd.exe', 'realmd.exe', 'aiplayerbot.conf.dist', 'Extractors\MoveMapGen.exe') {
    if (-not (Test-Path (Join-Path $root "Server\cmangos\$f"))) { throw "Server\cmangos\$f absent : lancer Build\Build-Cmangos.ps1" }
}
# anticheat.conf.dist exclu : sans anticheat.conf, le module anticheat reste inactif (serveur solo).
CopyTree (Join-Path $root 'Server\cmangos') (Join-Path $out 'Server\cmangos') -Exclude @('anticheat.conf.dist')
New-Item -ItemType Directory -Force (Join-Path $out 'Server\Config') | Out-Null
foreach ($f in 'realmd.sql', 'characters.sql', 'mangos.sql', 'logs.sql') {
    if (-not (Test-Path (Join-Path $root "Server\Database\cmangos-sql\$f"))) { throw "Server\Database\cmangos-sql\$f absent : lancer Build\Build-CmangosDb.ps1" }
}
CopyTree (Join-Path $root 'Server\Database\cmangos-sql') (Join-Path $out 'Server\Database\cmangos-sql')

Step "MariaDB allégée (sans symboles de débogage, en-têtes ni bibliothèques de liaison)"
CopyTree (Join-Path $root 'Server\Database\mariadb') (Join-Path $out 'Server\Database\mariadb') -Exclude @('*.pdb', 'include\*', 'lib\*.lib', 'lib\*.pdb', 'mysql-test\*', 'sql-bench\*', 'bin\mariadb-test*.exe', 'bin\*.pdb')

Step "HeidiSQL, scripts d'exploitation, documentation"
CopyTree (Join-Path $root 'Tools\HeidiSQL') (Join-Path $out 'Tools\HeidiSQL')
if (Test-Path (Join-Path $root 'Tools\VanillaFixes')) { CopyTree (Join-Path $root 'Tools\VanillaFixes') (Join-Path $out 'Tools\VanillaFixes') }
New-Item -ItemType Directory -Force (Join-Path $out 'Tools\Scripts') | Out-Null
Get-ChildItem (Join-Path $root 'Tools') -Filter '*.ps1' | ForEach-Object { Copy-Item $_.FullName (Join-Path $out "Tools\Scripts\$($_.Name)") }
CopyTree (Join-Path $root 'Documentation') (Join-Path $out 'Documentation')
Copy-Item (Join-Path $root 'README.md') (Join-Path $out 'Documentation\00-README-projet.md')
Copy-Item (Join-Path $PSScriptRoot 'package-files\LISEZMOI.txt') (Join-Path $out 'LISEZMOI.txt')
New-Item -ItemType Directory -Force (Join-Path $out 'Build') | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'manifest.json') (Join-Path $out 'Build\manifest.json')
Copy-Item (Join-Path $PSScriptRoot 'HomeCraft-CodeSigning.cer') (Join-Path $out 'Build\HomeCraft-CodeSigning.cer') -ErrorAction SilentlyContinue

Step "Licences et sources"
$lic = Join-Path $out 'LICENCES'
New-Item -ItemType Directory -Force $lic | Out-Null
# Licence propriétaire du Logiciel (launcher, scripts, documentation, compléments, identité visuelle) : à la racine et dans LICENCES.
Copy-Item (Join-Path $root 'LICENCE.txt') (Join-Path $out 'LICENCE.txt')
Copy-Item (Join-Path $root 'LICENCE.txt') (Join-Path $lic 'HomeCraft-Licence.txt')
foreach ($f in 'CMaNGOS-GPL-2.0.txt', 'classic-db-GPL-3.0.txt', 'OpenSSL-Apache-2.0.txt') { Copy-Item (Join-Path $PSScriptRoot "package-files\$f") (Join-Path $lic $f) }
$mdbLic = Get-ChildItem (Join-Path $root 'Server\Database\mariadb') -File | Where-Object { $_.Name -match '^(COPYING|LICENSE|THIRDPARTY|README)' }
foreach ($f in $mdbLic) { Copy-Item $f.FullName (Join-Path $lic "MariaDB-$($f.Name)") }
Copy-Item (Join-Path $root 'Tools\HeidiSQL\gpl.txt') (Join-Path $lic 'HeidiSQL-GPL-2.0.txt')
Copy-Item (Join-Path $root 'Tools\HeidiSQL\license.txt') (Join-Path $lic 'HeidiSQL-license.txt')
foreach ($f in 'LICENSE-VanillaFixes.txt', 'LICENSE-DXVK.txt') { $p = Join-Path $root "Tools\VanillaFixes\$f"; if (Test-Path $p) { Copy-Item $p (Join-Path $lic ($f -replace '^LICENSE-', '' -replace '\.txt$', '-licence.txt')) } }
# Licences des crates Rust (cargo metadata)
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$metaFile = Join-Path $env:TEMP 'homecraft-cargo-metadata.json'
Push-Location (Join-Path $root 'Launcher')
cmd /c "cargo metadata --format-version 1 --locked > `"$metaFile`" 2>nul"
Pop-Location
$meta = Get-Content $metaFile -Raw | ConvertFrom-Json
$crates = $meta.packages | Where-Object { $_.name -ne 'vanilla-solo-launcher' } | Sort-Object name | ForEach-Object { "| $($_.name) | $($_.version) | $($_.license) |" }
$manifestComponents = (Get-Content (Join-Path $PSScriptRoot 'manifest.json') -Raw | ConvertFrom-Json).components
$sources = @("# Sources et licences", "", "Package HomeCraft Edition Vanilla v$version généré le $(Get-Date -Format 'yyyy-MM-dd HH:mm').", "",
    "## Composants", "", "| Composant | Version / commit | Source | Licence |", "|---|---|---|---|")
# vmangos-db-mysql et innosetup ne servent qu'à la construction : seuls les composants livrés sont listés.
foreach ($c in $manifestComponents | Where-Object { $_.id -notin 'vmangos-db-mysql', 'innosetup' }) { $sources += "| $($c.id) | $($c.commit) ($($c.released)) | $($c.source) | $($c.license) |" }
foreach ($c in (Get-Content (Join-Path $PSScriptRoot 'manifest.json') -Raw | ConvertFrom-Json).sources | Where-Object { $_.id -ne 'boost-prebuilt' }) { $sources += "| $($c.id) | $($c.commit) ($($c.released)) | $($c.source) | $($c.license) |" }
$sources += "| OpenSSL (libcrypto-3-x64.dll, libssl-3-x64.dll, legacy.dll, fournis par CMaNGOS) | 3.0.14 | https://www.openssl.org | Apache-2.0 |"
$sources += "| Client MySQL (libmysql.dll, fourni par CMaNGOS) | 8.4.0 | https://dev.mysql.com/downloads/c-api/ | GPL-2.0 avec exception FOSS universelle |"
$sources += @("| HomeCraft Edition Vanilla (launcher, scripts, documentation, compléments Server\Database\custom, identité visuelle) | v$version | Sources\Launcher, Sources\Build, Tools\Scripts (ce package) | Propriétaire : LICENCE.txt (usage personnel ; reproduction, diffusion et modification interdites) |", "",
    "Les licences des composants tiers ci-dessus s'appliquent uniquement à ces composants ; la licence HomeCraft ne les restreint pas.", "",
    "## Modifications apportées aux composants", "",
    "- **Serveur CMaNGOS Classic (Server\cmangos)** : mangosd.exe, realmd.exe et les extracteurs sont compilés par HomeCraft depuis les sources des commits indiqués (mangos-classic et module playerbots, compilation reproductible : Sources\Build\Build-Cmangos.ps1, Visual Studio Build Tools 2022, Boost 1.87). Modifications : **Sources\Build\cmangos-patches\homecraft.patch** (GPL-2.0, comme CMaNGOS : build client 5880 accepté pour le client 1.12.1 frFR ; option -p de mangosd prise en compte par le module playerbots) et **playerbots.patch** (commande console « rndbot reload » enregistrée).",
    "- **Module playerbots** : son dépôt ne publie aucune licence. HomeCraft l'utilise avec l'accord demandé à ses auteurs (voir Documentation\11-Licence-PlayerBots.md) ; sans cet accord, ce package est réservé à l'usage personnel de son éditeur.",
    "- **Bases de données (Server\Database\cmangos-sql)** : construites par Sources\Build\Build-CmangosDb.ps1 avec l'installateur officiel de classic-db (GPL-3.0, avec les tables du module playerbots) ; les traductions françaises (colonnes loc2 des quêtes, créatures, objets, objets du monde, pages et options de dialogue) sont reprises de la base VMaNGOS (GPL-2.0, commit indiqué dans Build\manifest.json) complétée de traductions HomeCraft ; les comptes de démonstration de realmd.sql (mots de passe publics) sont supprimés.",
    "- **Runtime Visual C++ 2015-2022** (vcruntime140*.dll, msvcp140*.dll, concrt140.dll) : copié de MariaDB à côté des programmes CMaNGOS (déploiement local autorisé par Microsoft pour ces fichiers redistribuables), pour qu'aucun « Visual C++ Redistributable » ne soit requis.", "")
$sources += @(
    "Le code source complet de CMaNGOS, du module playerbots, de classic-db, de MariaDB et de HeidiSQL est disponible aux adresses ci-dessus pour les versions exactes indiquées (commit ou numéro de version). Sur demande, l'éditeur de ce package en fournit une copie.", "",
    "## Bibliothèques Rust du launcher (licences déclarées par les auteurs)", "", "| Crate | Version | Licence |", "|---|---|---|") + $crates
[System.IO.File]::WriteAllText((Join-Path $lic 'SOURCES.md'), ($sources -join "`r`n"), (New-Object System.Text.UTF8Encoding $false))
$src = Join-Path $out 'Sources\Launcher'
CopyTree (Join-Path $root 'Launcher\src') (Join-Path $src 'src')
CopyTree (Join-Path $root 'Launcher\assets') (Join-Path $src 'assets')
foreach ($f in 'Cargo.toml', 'Cargo.lock', 'README.md', 'build.rs') { Copy-Item (Join-Path $root "Launcher\$f") (Join-Path $src $f) }
CopyTree $PSScriptRoot (Join-Path $out 'Sources\Build') -Exclude @('package\*', 'downloads\*', 'tmp\*', 'files-manifest.json', 'installer-assets\*', 'vmangos-src\*', 'vmangos-build\*', 'cmangos-src\*', 'cmangos-build\*', 'cmangos-build-pdb\*','cmangos-db\*', 'classic-db\*', 'deps\*', '*.log')

Step "autorun.inf"
Set-Content -Path (Join-Path $out 'autorun.inf') -Encoding ASCII -Value "[autorun]`r`nopen=HomeCraft Edition Vanilla.exe`r`nicon=HomeCraft Edition Vanilla.exe,0`r`nlabel=HomeCraft Edition Vanilla`r`n"

Step "Manifeste de fichiers et SHA256SUMS"
$files = New-Object System.Collections.Generic.List[object]
$sums = New-Object System.Collections.Generic.List[string]
Get-ChildItem $out -Recurse -File | Sort-Object FullName | ForEach-Object {
    $rel = $_.FullName.Substring($out.Length + 1)
    if ($rel -in 'Build\files-manifest.json', 'SHA256SUMS.txt') { return }
    $h = (Get-FileHash -Algorithm SHA256 $_.FullName).Hash.ToLowerInvariant()
    $files.Add([pscustomobject]@{ path = $rel; size = $_.Length; sha256 = $h })
    $sums.Add("$h  $($rel -replace '\\','/')")
}
$manifest = [pscustomobject]@{ generated = (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'); version = $version; files = $files }
[System.IO.File]::WriteAllText((Join-Path $out 'Build\files-manifest.json'), ($manifest | ConvertTo-Json -Depth 4), (New-Object System.Text.UTF8Encoding $false))
[System.IO.File]::WriteAllText((Join-Path $out 'SHA256SUMS.txt'), (($sums -join "`n") + "`n"), (New-Object System.Text.UTF8Encoding $false))

$total = (Get-ChildItem $out -Recurse -File | Measure-Object Length -Sum).Sum
Step ("Package : {0} fichiers, {1:N0} Mo -> {2}" -f $files.Count, ($total / 1MB), $out)
if ($total -gt 700MB) { Write-Warning "Dépasse un CD-ROM (700 Mo) : prévoir un DVD." } else { Step "Tient sur un CD-ROM (700 Mo)." }
if ($Zip) {
    $zipPath = Join-Path $PSScriptRoot "package\$name.zip"
    if (Test-Path $zipPath) { Remove-Item $zipPath }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory($out, $zipPath, [System.IO.Compression.CompressionLevel]::Optimal, $true)
    Step ("Archive : {0:N0} Mo -> {1}" -f ((Get-Item $zipPath).Length / 1MB), $zipPath)
}
