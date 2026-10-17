<#
.SYNOPSIS
    Compile le cœur SylvaniaCore-WoTLK + Playerbots (Build/src/sylvaniacore-wotlk, branche homecraft-playerbots).
.DESCRIPTION
    Configure (si besoin) puis compile en RelWithDebInfo avec Visual Studio 2022 x64 et les dépendances
    décrites dans Build/build-tools.json. Journal : Logs/build-sylvania.log. Relancer après une correction :
    la compilation est incrémentale.
.PARAMETER Configure
    Force une nouvelle configuration CMake.
.PARAMETER Target
    Cible MSBuild/CMake (par défaut : tout).
.PARAMETER Install
    Lance aussi l'installation dans Build/install (binaires + conf.dist + SQL de base).
#>
[CmdletBinding()]
param(
    [switch]$Configure,
    [string]$Target = "",
    [switch]$Install
)
# Pas de "Stop" : CMake écrit ses avertissements sur stderr et PowerShell 5.1 en ferait des erreurs bloquantes
# (NativeCommandError). Les échecs sont détectés par $LASTEXITCODE.
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$srcReal = Join-Path $root "Build\src\sylvaniacore-wotlk"
# Chemin court obligatoire : avec 24 modules, les lignes de commande de rc.exe/cl.exe dépassent la limite Windows
# (erreur MSB6003 « Nom de fichier ou extension trop long ») si l'on compile depuis Build\src\... .
# Une jonction NTFS C:\hc -> le clone (pas besoin de droits administrateur) ; répertoire de build C:\hc\b.
$src = "C:\hc"
if (-not (Test-Path $src)) {
    cmd /c mklink /J "$src" "$srcReal" | Out-Null
    if (-not (Test-Path $src)) { throw "impossible de créer la jonction $src -> $srcReal" }
}
$bld = Join-Path $src "b"
$logDir = Join-Path $root "Logs"
New-Item -ItemType Directory -Force $logDir | Out-Null
$log = Join-Path $logDir "build-sylvania.log"

$cmake = Get-ChildItem "$env:LOCALAPPDATA\Microsoft\WinGet\Packages" -Recurse -Filter cmake.exe -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty FullName
if (-not $cmake) { $cmake = "cmake" }
$env:PATH = "C:\Program Files\Git\cmd;" + $env:PATH

$common = @(
    "-DBOOST_ROOT=C:/local/boost_1_87_0",
    "-DOPENSSL_ROOT_DIR=C:/Program Files/OpenSSL-Win64",
    "-DMYSQL_INCLUDE_DIR=C:/tools/mysql/current/include",
    "-DMYSQL_LIBRARY=C:/tools/mysql/current/lib/libmysql.lib",
    "-DTOOLS_BUILD=all", "-DSCRIPTS=static", "-DMODULES=static",
    "-DCMAKE_INSTALL_PREFIX=$($root -replace '\\', '/')/Build/install"
)

if ($Configure -or -not (Test-Path (Join-Path $bld "CMakeCache.txt"))) {
    Write-Host "== Configuration CMake"
    & $cmake -S $src -B $bld -G "Visual Studio 17 2022" -A x64 @common
    if ($LASTEXITCODE -ne 0) { throw "configuration CMake en échec ($LASTEXITCODE)" }
}

"=== build debut $(Get-Date -Format s) target='$Target'" | Out-File $log -Append -Encoding utf8
$args = @("--build", $bld, "--config", "RelWithDebInfo", "--parallel")
if ($Target) { $args += @("--target", $Target) }
$args += @("--", "/m", "/nologo", "/v:minimal", "/nodeReuse:false")
Write-Host "== Compilation (journal : $log)"
$sw = New-Object System.IO.StreamWriter($log, $true, (New-Object System.Text.UTF8Encoding($false)))
try {
    & $cmake @args 2>&1 | ForEach-Object {
        $line = "$_"
        $sw.WriteLine($line)
        if ($line -match "error C|error LNK|error MSB|fatal error") { Write-Host $line.Trim() }
    }
    $code = $LASTEXITCODE
} finally { $sw.Close() }
"=== build fin $(Get-Date -Format s) EXIT=$code" | Out-File $log -Append -Encoding utf8
if ($code -ne 0) { Write-Host "== ECHEC (code $code), voir $log"; exit $code }
Write-Host "== Compilation réussie"

if ($Install) {
    Write-Host "== Installation dans Build/install"
    & $cmake --install $bld --config RelWithDebInfo 2>&1 | ForEach-Object { "$_" } | Out-File $log -Append -Encoding utf8
    if ($LASTEXITCODE -ne 0) { Write-Host "== ECHEC installation"; exit $LASTEXITCODE }
}
exit 0
