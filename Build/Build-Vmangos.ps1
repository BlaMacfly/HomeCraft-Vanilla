<#
.SYNOPSIS
    Compile mangosd et realmd depuis les sources VMaNGOS du commit figé, avec les modifications HomeCraft, et les
    installe dans Server\vmangos.
.DESCRIPTION
    Modifications (Build\vmangos-patches\homecraft.patch, GPL-2.0 comme VMaNGOS) :
      - build client 5880 accepté (client 1.12.1 frFR), à la place du binaire modifié par Patch-MangosdBuilds.ps1 ;
      - bots du monde : les bots de peuplement acceptent l'invitation de tout joueur, se mettent à son niveau, le
        rejoignent et combattent avec lui ; hors groupe, ils restent en ville ou chassent dans une zone de leur niveau.
    Sources : clone de https://github.com/vmangos/core dans Build\vmangos-src (ignoré par git), compilation dans
    Build\vmangos-build avec CMake et MSBuild des Build Tools 2022 (aucune autre dépendance : celles de VMaNGOS sont
    dans son dépôt). Les binaires officiels remplacés sont conservés en .orig (exclus du package). Les extracteurs
    restent ceux du composant officiel. Après l'installation : Build\Set-VmangosRuntime.ps1 n'est pas à relancer
    (les DLL ne changent pas), Tests\Test-Dependencies.ps1 et Tests\Test-Integration.ps1 oui.
#>
[CmdletBinding()]
param([switch]$NoInstall)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
$commit = '53d24d5c748bef5e0cde7b1155bcb3d979af614c'
$src = Join-Path $PSScriptRoot 'vmangos-src'
$build = Join-Path $PSScriptRoot 'vmangos-build'
$patch = Join-Path $PSScriptRoot 'vmangos-patches\homecraft.patch'
$cmake = 'C:\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe'
$msbuild = 'C:\BuildTools\MSBuild\Current\Bin\MSBuild.exe'
foreach ($t in $cmake, $msbuild) { if (-not (Test-Path $t)) { throw "introuvable : $t (Build Tools 2022 avec les outils CMake)" } }

# 1. Sources au commit figé + modifications HomeCraft
if (-not (Test-Path (Join-Path $src '.git'))) {
    Write-Step "clone de vmangos/core"
    cmd /c "git clone -q --filter=blob:none https://github.com/vmangos/core.git `"$src`" >nul 2>&1"
    if ($LASTEXITCODE -ne 0) { throw "clone de vmangos/core impossible" }
}
cmd /c "git -C `"$src`" checkout -q -f $commit >nul 2>&1"
if ($LASTEXITCODE -ne 0) { throw "commit $commit introuvable" }
cmd /c "git -C `"$src`" clean -q -fd -e bin >nul 2>&1"
cmd /c "git -C `"$src`" apply --whitespace=nowarn `"$patch`""
if ($LASTEXITCODE -ne 0) { throw "le patch HomeCraft ne s'applique pas sur $commit" }
# Contrôle : chaque fichier du patch doit être modifié (une modification perdue est passée inaperçue une fois).
$expected = @(Select-String -Path $patch -Pattern '^diff --git a/(\S+)' | ForEach-Object { $_.Matches[0].Groups[1].Value } | Sort-Object -Unique)
$modified = @(cmd /c "git -C `"$src`" diff --name-only 2>nul" | Sort-Object -Unique)
$missing = @($expected | Where-Object { $modified -notcontains $_ })
if ($missing.Count) { throw "fichiers du patch non modifiés après application : $($missing -join ', ')" }
# src/shared/Progression.h est régénéré par CMake depuis ce modèle : c'est lui qui porte le build 5880.
if (-not (Select-String -Path (Join-Path $src 'cmake\generators\Progression.h.in') -Pattern '5875, 5880' -Quiet)) { throw "build client 5880 absent de cmake\generators\Progression.h.in" }
Write-Step "sources $($commit.Substring(0, 7)) + homecraft.patch"

# 2. Compilation (configuration Release x64, comme la publication officielle de VMaNGOS)
New-Item -ItemType Directory -Force $build | Out-Null
cmd /c "`"$cmake`" -S `"$src`" -B `"$build`" -DBUILD_EXTRACTORS=0 -G `"Visual Studio 17 2022`" -A x64 > `"$build\cmake.log`" 2>&1"
if ($LASTEXITCODE -ne 0) { throw "configuration CMake échouée (voir $build\cmake.log)" }
Write-Step "compilation (plusieurs minutes la première fois)"
cmd /c "`"$msbuild`" `"$build\MaNGOS.sln`" /p:Platform=x64 /p:Configuration=Release /m /v:minimal /nologo > `"$build\msbuild.log`" 2>&1"
if ($LASTEXITCODE -ne 0) { Get-Content "$build\msbuild.log" | Select-String 'error' | Select-Object -First 10 | ForEach-Object { Write-Host $_ }; throw "compilation échouée (voir $build\msbuild.log)" }
$bin = Join-Path $src 'bin\Release'
if (-not (Select-String -Path (Join-Path $src 'src\shared\Progression.h') -Pattern '5875, 5880' -Quiet)) { throw "Progression.h généré sans le build 5880" }
Write-Step "mangosd.exe et realmd.exe compilés"
if ($NoInstall) { return }

# 3. Installation (binaires officiels conservés en .orig)
$vm = Get-ProjectPath 'Server\vmangos'
foreach ($exe in 'mangosd.exe', 'realmd.exe') {
    $dst = Join-Path $vm $exe
    if ((Test-Path $dst) -and -not (Test-Path "$dst.orig")) { Copy-Item $dst "$dst.orig" }
    Copy-Item (Join-Path $bin $exe) $dst -Force
}
Write-Step "installés dans Server\vmangos (SHA-256 mangosd : $((Get-FileHash -Algorithm SHA256 (Join-Path $vm 'mangosd.exe')).Hash.ToLowerInvariant()))"
