<#
.SYNOPSIS
    Construit la version portable distribuable dans Build/package/HomeCraft-Edition-BC-v<version>/ : launcher signé,
    serveur CMaNGOS mangos-tbc, MariaDB allégée, dumps TBC-DB, compléments, HeidiSQL, documentation, licences et
    sources, manifeste de fichiers (empreintes) et SHA256SUMS.txt. Avec -Zip, produit aussi l'archive .zip.
    Jamais inclus : Client/, Server/Data/, UserData/, Logs/, Build/downloads/, Build/tmp/ (décision de l'utilisateur :
    le client WoW n'est jamais livré, quel que soit le support). Adapté de Build/New-Package.ps1 du Vanilla.
#>
[CmdletBinding()]
param([switch]$Zip)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$cargo = Get-Content (Join-Path $root 'Launcher\Cargo.toml') -Raw
$version = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
$name = "HomeCraft-Edition-BC-v$version"
$out = Join-Path $PSScriptRoot "package\$name"
$launcher = Join-Path $root 'Launcher\target\release\homecraft-bc-launcher.exe'
if (-not (Test-Path $launcher)) { throw "Launcher non compilé : $launcher" }
$sig = Get-AuthenticodeSignature $launcher
if (-not $sig.SignerCertificate) { throw "Le launcher n'est pas signé (Build\Sign-Launcher.ps1)" }

function Step { param($m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force $out | Out-Null
function CopyTree { param($src, $dst, [string[]]$Exclude = @())
    New-Item -ItemType Directory -Force $dst | Out-Null
    Get-ChildItem -LiteralPath $src -Recurse -File | ForEach-Object {
        $rel = $_.FullName.Substring($src.Length + 1)
        if ($Exclude | Where-Object { $rel -like $_ }) { return }
        $to = Join-Path $dst $rel
        New-Item -ItemType Directory -Force (Split-Path $to) | Out-Null
        Copy-Item -LiteralPath $_.FullName -Destination $to
    }
}

Step "Launcher"
Copy-Item $launcher (Join-Path $out 'HomeCraft Edition BC.exe')

Step "Serveur CMaNGOS mangos-tbc, dumps TBC-DB, compléments"
# Exclus : configurations générées (aiplayerbot.conf, lu dans le dossier courant), rapports de plantage, symboles.
CopyTree (Join-Path $root 'Server\mangos-tbc') (Join-Path $out 'Server\mangos-tbc') -Exclude @('*.conf', 'Crashes\*', '*.pdb', '*.orig', 'Extractors\*.log')
New-Item -ItemType Directory -Force (Join-Path $out 'Server\Config') | Out-Null   # modèles recopiés depuis mangos-tbc par le launcher
CopyTree (Join-Path $root 'Server\Database\sql') (Join-Path $out 'Server\Database\sql')
CopyTree (Join-Path $root 'Server\Database\custom') (Join-Path $out 'Server\Database\custom')

Step "MariaDB allégée (sans symboles de débogage, en-têtes ni bibliothèques de liaison)"
CopyTree (Join-Path $root 'Server\Database\mariadb') (Join-Path $out 'Server\Database\mariadb') -Exclude @('*.pdb', 'include\*', 'lib\*.lib', 'lib\*.pdb', 'mysql-test\*', 'sql-bench\*', 'bin\mariadb-test*.exe', 'bin\*.pdb')

Step "HeidiSQL, scripts d'exploitation, documentation"
CopyTree (Join-Path $root 'Tools\HeidiSQL') (Join-Path $out 'Tools\HeidiSQL')
New-Item -ItemType Directory -Force (Join-Path $out 'Tools\Scripts') | Out-Null
Get-ChildItem (Join-Path $root 'Tools') -Filter '*.ps1' | ForEach-Object { Copy-Item $_.FullName (Join-Path $out "Tools\Scripts\$($_.Name)") }
CopyTree (Join-Path $root 'Documentation') (Join-Path $out 'Documentation') -Exclude @('Prompt-Session-*.md')
Copy-Item (Join-Path $PSScriptRoot 'package-files\LISEZMOI.txt') (Join-Path $out 'LISEZMOI.txt')
New-Item -ItemType Directory -Force (Join-Path $out 'Build') | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'manifest.json') (Join-Path $out 'Build\manifest.json')
Copy-Item (Join-Path $PSScriptRoot 'HomeCraft-CodeSigning.cer') (Join-Path $out 'Build\HomeCraft-CodeSigning.cer') -ErrorAction SilentlyContinue

Step "Licences et sources"
$lic = Join-Path $out 'LICENCES'
New-Item -ItemType Directory -Force $lic | Out-Null
Copy-Item (Join-Path $root 'LICENCE.txt') (Join-Path $out 'LICENCE.txt')
Copy-Item (Join-Path $root 'LICENCE.txt') (Join-Path $lic 'HomeCraft-Licence.txt')
$pf = Join-Path $PSScriptRoot 'package-files\licences'
Copy-Item (Join-Path $pf 'CMaNGOS-GPL-2.0.txt') (Join-Path $lic 'CMaNGOS-mangos-tbc-et-PlayerBots-GPL-2.0.txt')
Copy-Item (Join-Path $pf 'CMaNGOS-COPYRIGHT.md') (Join-Path $lic 'CMaNGOS-COPYRIGHT.md')
Copy-Item (Join-Path $pf 'GPL-3.0.txt') (Join-Path $lic 'TBC-DB-GPL-3.0.txt')
Copy-Item (Join-Path $pf 'AzerothCore-SylvaniaCore-GPL-2.0.txt') (Join-Path $lic 'Traductions-AzerothCore-SylvaniaCore-mod-playerbots-GPL-2.0.txt')
$mdbLic = Get-ChildItem (Join-Path $root 'Server\Database\mariadb') -File | Where-Object { $_.Name -match '^(COPYING|LICENSE|THIRDPARTY|README)' }
foreach ($f in $mdbLic) { Copy-Item $f.FullName (Join-Path $lic "MariaDB-$($f.Name)") }
Copy-Item (Join-Path $root 'Tools\HeidiSQL\gpl.txt') (Join-Path $lic 'HeidiSQL-GPL-2.0.txt')
Copy-Item (Join-Path $root 'Tools\HeidiSQL\license.txt') (Join-Path $lic 'HeidiSQL-license.txt')
# Licences des crates Rust (cargo metadata)
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$metaFile = Join-Path $env:TEMP 'homecraft-bc-cargo-metadata.json'
Push-Location (Join-Path $root 'Launcher')
cmd /c "cargo metadata --format-version 1 --locked > `"$metaFile`" 2>nul"
Pop-Location
$meta = Get-Content $metaFile -Raw | ConvertFrom-Json
$crates = $meta.packages | Where-Object { $_.name -ne 'homecraft-bc-launcher' } | Sort-Object name | ForEach-Object { "| $($_.name) | $($_.version) | $($_.license) |" }
$gplCrates = @($meta.packages | Where-Object { $_.license -match 'GPL' -and $_.license -notmatch 'MIT|Apache' })
if ($gplCrates.Count) { throw "Dépendance GPL/LGPL liée au launcher : $($gplCrates.name -join ', ')" }
$manifestComponents = (Get-Content (Join-Path $PSScriptRoot 'manifest.json') -Raw | ConvertFrom-Json).components
$sources = @("# Sources et licences", "", "Package HomeCraft Edition BC v$version généré le $(Get-Date -Format 'yyyy-MM-dd HH:mm').", "",
    "## Composants", "", "| Composant | Version / commit | Source | Licence |", "|---|---|---|---|")
foreach ($c in $manifestComponents) { $sources += "| $($c.id) | $($c.commit) ($($c.released)) | $($c.source) | $($c.license) |" }
$sources += @(
    "| Compléments de base de données (Server\Database\custom) : correctif de schéma locales_gameobject (reprise de la mise à jour CMaNGOS s2485) et traductions françaises (50_homecraft_frFR.sql) reprises de VMaNGOS (db 4641790), AzerothCore / SylvaniaCore-WoTLK et mod-playerbots | 2026-10-17 | https://github.com/vmangos/core, https://github.com/BlaMacfly/SylvaniaCore-WoTLK, https://github.com/azerothcore/mod-playerbots ; outil de génération : Sources\Localisation | GPL-2.0 |",
    "| HomeCraft Edition BC (launcher, scripts, documentation, identité visuelle) | v$version | Sources\Launcher, Sources\Build, Tools\Scripts (ce package) | Propriétaire : LICENCE.txt (usage personnel ; reproduction, diffusion et modification interdites) |", "",
    "Les licences des composants tiers ci-dessus s'appliquent uniquement à ces composants ; la licence HomeCraft ne les restreint pas.", "",
    "## Modifications apportées aux composants", "",
    "- Aucun binaire tiers n'est modifié. Le build Windows officiel de mangos-tbc (variante with-all) est livré tel quel, sans les fichiers de symboles (*.pdb).",
    "- Base TBC-DB : importée telle quelle, puis complétée par les fichiers de Server\Database\custom (correctif de schéma et traductions), appliqués par le launcher après l'import.", "",
    "Le code source complet de CMaNGOS mangos-tbc (commit 15b6ddb4ec9e443d49f4e438af73782ce5c16491), du module PlayerBots (commit 81fea4cdcd0c1d3c2ae05dd39b1693cef1a75a2b), de TBC-DB (commit 87af76a0e12cbd0a29d5ccc2be56f52c2b0b199f), de MariaDB et de HeidiSQL est disponible aux adresses ci-dessus pour les versions exactes indiquées. Sur demande, l'éditeur de ce package en fournit une copie.", "",
    "## Bibliothèques Rust du launcher (licences déclarées par les auteurs)", "", "| Crate | Version | Licence |", "|---|---|---|") + $crates
[System.IO.File]::WriteAllText((Join-Path $lic 'SOURCES.md'), ($sources -join "`r`n"), (New-Object System.Text.UTF8Encoding $false))
$src = Join-Path $out 'Sources\Launcher'
CopyTree (Join-Path $root 'Launcher\src') (Join-Path $src 'src')
CopyTree (Join-Path $root 'Launcher\assets') (Join-Path $src 'assets')
foreach ($f in 'Cargo.toml', 'Cargo.lock', 'README.md', 'build.rs') { Copy-Item (Join-Path $root "Launcher\$f") (Join-Path $src $f) }
CopyTree (Join-Path $root 'Tools\Localisation') (Join-Path $out 'Sources\Localisation') -Exclude @('*\target\*', 'Cargo.lock')
CopyTree $PSScriptRoot (Join-Path $out 'Sources\Build') -Exclude @('package\*', 'downloads\*', 'tmp\*', 'files-manifest.json')

Step "Manifeste de fichiers et SHA256SUMS"
$files = New-Object System.Collections.Generic.List[object]
$sums = New-Object System.Collections.Generic.List[string]
Get-ChildItem $out -Recurse -File | Sort-Object FullName | ForEach-Object {
    $rel = $_.FullName.Substring($out.Length + 1)
    if ($rel -in 'Build\files-manifest.json', 'SHA256SUMS.txt') { return }
    $h = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
    $files.Add([pscustomobject]@{ path = $rel; size = $_.Length; sha256 = $h })
    $sums.Add("$h  $($rel -replace '\\','/')")
}
$manifest = [pscustomobject]@{ generated = (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'); version = $version; files = $files }
[System.IO.File]::WriteAllText((Join-Path $out 'Build\files-manifest.json'), ($manifest | ConvertTo-Json -Depth 4), (New-Object System.Text.UTF8Encoding $false))
[System.IO.File]::WriteAllText((Join-Path $out 'SHA256SUMS.txt'), (($sums -join "`n") + "`n"), (New-Object System.Text.UTF8Encoding $false))

# Garde-fou : rien de personnel ni de Blizzard dans le package.
$forbidden = Get-ChildItem $out -Recurse -File | Where-Object { $_.FullName -match '\\(Client|UserData|Logs)\\|\\Server\\Data\\|\.MPQ$|\\Crashes\\|db-credentials|settings\.json$|\.dbc$|\.mmtile$|\.vmtile$' }
if ($forbidden) { throw "Fichiers interdits dans le package : $($forbidden.FullName -join ', ')" }

$total = (Get-ChildItem $out -Recurse -File | Measure-Object Length -Sum).Sum
Step ("Package : {0} fichiers, {1:N0} Mo -> {2}" -f $files.Count, ($total / 1MB), $out)
if ($Zip) {
    $zipPath = Join-Path $PSScriptRoot "package\$name.zip"
    if (Test-Path $zipPath) { Remove-Item $zipPath }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory($out, $zipPath, [System.IO.Compression.CompressionLevel]::Optimal, $true)
    Step ("Archive : {0:N0} Mo -> {1}" -f ((Get-Item $zipPath).Length / 1MB), $zipPath)
    Step ("SHA-256 de l'archive : {0}" -f (Get-FileHash -Algorithm SHA256 $zipPath).Hash.ToLowerInvariant())
}
