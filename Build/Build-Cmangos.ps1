<#
.SYNOPSIS
    Compile CMaNGOS Classic avec le module playerbots (et l'AHBot) depuis les commits figés, avec les modifications
    HomeCraft, et installe serveur, extracteurs et DLL dans Server\cmangos.
.DESCRIPTION
    Sources (clones ignorés par git) :
      - https://github.com/cmangos/mangos-classic (GPL-2.0) dans Build\cmangos-src ;
      - https://github.com/cmangos/playerbots (aucun fichier de licence publié : usage personnel seulement tant que
        les auteurs n'ont pas accordé de licence de redistribution) dans Build\cmangos-src\src\modules\PlayerBots ;
      - Boost 1.87 précompilé statique (https://github.com/MarkusJx/prebuilt-boost, Boost Software License) dans
        Build\deps\boost, nécessaire à la compilation uniquement.
    Modifications HomeCraft (Build\cmangos-patches) :
      - homecraft.patch (GPL-2.0 comme CMaNGOS) : build client 5880 accepté (client 1.12.1 frFR) par realmd et mangosd ;
      - playerbots.patch : commande console « rndbot reload » enregistrée (la fonction existe dans le module), pour
        que le launcher applique les réglages de l'onglet PlayerBots sans redémarrer le serveur.
    Compilation dans Build\cmangos-build avec CMake et MSBuild des Build Tools 2022, configuration Release x64.
#>
[CmdletBinding()]
param([switch]$NoInstall)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
$coreCommit = 'ff53dfc4679cab3521726384e24692fef35d2718'
$botsCommit = '39d47006e4aa1658c7e17af15e4f63d47ec5a3eb'
$boostUrl = 'https://github.com/MarkusJx/prebuilt-boost/releases/download/1.87.0/boost-1.87.0-windows-2022-msvc-static-x86.tar.gz'
$boostSha = '8df50de4ea741789705ac35a74867f493ef73a0cfb46be4f5a02c12bea4445e8'
$src = Join-Path $PSScriptRoot 'cmangos-src'
$bots = Join-Path $src 'src\modules\PlayerBots'
$build = Join-Path $PSScriptRoot 'cmangos-build'
$boost = Join-Path $PSScriptRoot 'deps\boost'
$patch = Join-Path $PSScriptRoot 'cmangos-patches\homecraft.patch'
$botsPatch = Join-Path $PSScriptRoot 'cmangos-patches\playerbots.patch'
$cmake = 'C:\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe'
$msbuild = 'C:\BuildTools\MSBuild\Current\Bin\MSBuild.exe'
foreach ($t in $cmake, $msbuild) { if (-not (Test-Path $t)) { throw "introuvable : $t (Build Tools 2022 avec les outils CMake)" } }

function Invoke-Git([string]$repo, [string]$arguments, [string]$what) {
    cmd /c "git -C `"$repo`" $arguments >nul 2>&1"
    if ($LASTEXITCODE -ne 0) { throw $what }
}

# 1. Boost (archive figée par SHA-256)
if (-not (Test-Path (Join-Path $boost 'lib\cmake\Boost-1.87.0\BoostConfig.cmake'))) {
    $deps = Join-Path $PSScriptRoot 'deps'
    $archive = Join-Path $deps 'boost-1.87.0-windows-2022-msvc-static.tar.gz'
    New-Item -ItemType Directory -Force $deps | Out-Null
    if (-not (Test-Path $archive)) {
        Write-Step "téléchargement de Boost 1.87 (700 Mo)"
        cmd /c "curl.exe -sSL -o `"$archive`" `"$boostUrl`""
        if ($LASTEXITCODE -ne 0) { throw "téléchargement de Boost impossible" }
    }
    $sha = (Get-FileHash -Algorithm SHA256 $archive).Hash.ToLowerInvariant()
    if ($sha -ne $boostSha) { throw "Boost : SHA-256 $sha, attendu $boostSha" }
    New-Item -ItemType Directory -Force $boost | Out-Null
    # tar signale deux entrées sans nom de l'archive (code 1) : seul le fichier de configuration fait foi.
    cmd /c "tar -xzf `"$archive`" -C `"$boost`" >nul 2>&1"
    if (-not (Test-Path (Join-Path $boost 'lib\cmake\Boost-1.87.0\BoostConfig.cmake'))) { throw "extraction de Boost incomplète" }
}

# 2. Sources aux commits figés + modifications HomeCraft
if (-not (Test-Path (Join-Path $src '.git'))) {
    Write-Step "clone de cmangos/mangos-classic"
    cmd /c "git clone -q --filter=blob:none https://github.com/cmangos/mangos-classic.git `"$src`" >nul 2>&1"
    if ($LASTEXITCODE -ne 0) { throw "clone de cmangos/mangos-classic impossible" }
}
Invoke-Git $src "checkout -q -f $coreCommit" "commit $coreCommit introuvable"
Invoke-Git $src "clean -q -fd -e src/modules/PlayerBots" "nettoyage des sources impossible"
if (-not (Test-Path (Join-Path $bots '.git'))) {
    Write-Step "clone de cmangos/playerbots"
    cmd /c "git clone -q --filter=blob:none https://github.com/cmangos/playerbots.git `"$bots`" >nul 2>&1"
    if ($LASTEXITCODE -ne 0) { throw "clone de cmangos/playerbots impossible" }
}
Invoke-Git $bots "checkout -q -f $botsCommit" "commit $botsCommit introuvable"
Invoke-Git $bots "clean -q -fd" "nettoyage du module playerbots impossible"
function Add-Patch([string]$repo, [string]$patchFile) {
    cmd /c "git -C `"$repo`" apply --whitespace=nowarn `"$patchFile`""
    if ($LASTEXITCODE -ne 0) { throw "$(Split-Path -Leaf $patchFile) ne s'applique pas sur $repo" }
    # Chaque fichier du patch doit être modifié (une modification perdue est passée inaperçue une fois, avec VMaNGOS).
    $expected = @(Select-String -Path $patchFile -Pattern '^diff --git a/(\S+)' | ForEach-Object { $_.Matches[0].Groups[1].Value } | Sort-Object -Unique)
    $modified = @(cmd /c "git -C `"$repo`" diff --name-only 2>nul" | Sort-Object -Unique)
    $missing = @($expected | Where-Object { $modified -notcontains $_ })
    if ($missing.Count) { throw "fichiers de $(Split-Path -Leaf $patchFile) non modifiés après application : $($missing -join ', ')" }
}
Add-Patch $src $patch
Add-Patch $bots $botsPatch
if (-not (Select-String -Path (Join-Path $src 'src\game\Globals\SharedDefines.h') -Pattern '5875, 5880' -Quiet)) { throw "build client 5880 absent de SharedDefines.h" }
Write-Step "sources $($coreCommit.Substring(0, 7)) + playerbots $($botsCommit.Substring(0, 7)) + homecraft.patch + playerbots.patch"

# 3. Compilation. FetchContent pointé sur le clone local : CMaNGOS ne télécharge pas une autre version du module.
New-Item -ItemType Directory -Force $build | Out-Null
$botsCMake = $bots -replace '\\', '/'
$boostCMake = $boost -replace '\\', '/'
cmd /c "`"$cmake`" -S `"$src`" -B `"$build`" -G `"Visual Studio 17 2022`" -A x64 -DBUILD_PLAYERBOTS=ON -DBUILD_AHBOT=ON -DBUILD_EXTRACTORS=ON -DFETCHCONTENT_UPDATES_DISCONNECTED=ON -DFETCHCONTENT_SOURCE_DIR_PLAYERBOTS=`"$botsCMake`" -DBOOST_ROOT=`"$boostCMake`" -DBoost_ROOT=`"$boostCMake`" > `"$build\cmake.log`" 2>&1"
if ($LASTEXITCODE -ne 0) { throw "configuration CMake échouée (voir $build\cmake.log)" }
$head = (cmd /c "git -C `"$bots`" rev-parse HEAD 2>nul")
if ($head -ne $botsCommit) { throw "CMake a changé le commit du module playerbots ($head)" }
Write-Step "compilation (5 à 10 minutes)"
cmd /c "`"$msbuild`" `"$build\CMaNGOS_Classic.sln`" /p:Platform=x64 /p:Configuration=Release /m /v:minimal /nologo > `"$build\msbuild.log`" 2>&1"
if ($LASTEXITCODE -ne 0) { Get-Content "$build\msbuild.log" | Select-String 'error' | Select-Object -First 10 | ForEach-Object { Write-Host $_ }; throw "compilation échouée (voir $build\msbuild.log)" }
Write-Step "compilation terminée"
if ($NoInstall) { return }

# 4. Installation
$out = Get-ProjectPath 'Server\cmangos'
New-Item -ItemType Directory -Force $out | Out-Null
$bin = Join-Path $build 'bin\x64_Release'
# Modèles de configuration (.conf.dist) : New-ServerConfig.ps1 en dérive les .conf. Le runtime Visual C++ (VC 14)
# est copié à côté des exécutables par Build\Set-VmangosRuntime.ps1, comme pour VMaNGOS.
$files = @('mangosd.exe', 'realmd.exe', 'libmysql.dll', 'libcrypto-3-x64.dll', 'libssl-3-x64.dll', 'legacy.dll',
    'mangosd.conf.dist', 'realmd.conf.dist', 'aiplayerbot.conf.dist', 'ahbot.conf.dist', 'anticheat.conf.dist',
    'Extractors\ad.exe', 'Extractors\vmap_extractor.exe', 'Extractors\vmap_assembler.exe', 'Extractors\MoveMapGen.exe',
    'Extractors\offmesh.txt', 'Extractors\config.json')
New-Item -ItemType Directory -Force (Join-Path $out 'Extractors') | Out-Null
foreach ($f in $files) {
    $s = Join-Path $bin $f
    if (-not (Test-Path $s)) { throw "fichier compilé absent : $s" }
    Copy-Item $s (Join-Path $out $f) -Force
}
# Runtime Visual C++ 2015-2022 à côté des programmes (serveur et extracteurs) : celui livré avec MariaDB (composant
# figé du manifeste, version 14.44 comme l'éditeur de liens des Build Tools ; Microsoft autorise ce déploiement local).
# Le serveur démarre ainsi sur un Windows sans « Redistributable » installé (Tests\Test-Dependencies.ps1).
$mdb = Get-ProjectPath 'Server\Database\mariadb\bin'
foreach ($d in $out, (Join-Path $out 'Extractors')) {
    foreach ($f in 'vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'concrt140.dll') {
        Copy-Item (Join-Path $mdb $f) (Join-Path $d $f) -Force
    }
}
Write-Step "runtime Visual C++ $((Get-Item (Join-Path $out 'vcruntime140.dll')).VersionInfo.FileVersion) copié dans Server\cmangos et Extractors"
Write-Step "installés dans Server\cmangos (SHA-256 mangosd :$((Get-FileHash -Algorithm SHA256 (Join-Path $out 'mangosd.exe')).Hash.ToLowerInvariant()))"
