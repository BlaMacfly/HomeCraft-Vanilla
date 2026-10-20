<#
.SYNOPSIS
    Rend Server\vmangos autonome vis-à-vis des runtimes Visual C++ installés dans Windows.
.DESCRIPTION
    Les programmes VMaNGOS (mangosd, realmd, extracteurs) demandent MSVCP140 / VCRUNTIME140 (Visual C++ 2015-2022) et
    leur libmySQL.dll (MySQL 5.5.62) le runtime Visual C++ 2008 : sur un Windows sans ces « Redistributables », le
    serveur ne démarre pas. Ce script :
      1. copie à côté des programmes les DLL Visual C++ 2015-2022 livrées avec MariaDB (composant figé du manifeste,
         même version 14.44 que l'éditeur de liens de VMaNGOS ; Microsoft autorise ce déploiement local) ;
      2. remplace libmySQL.dll par la couche de compatibilité Build\libmysql-shim (redirections vers libmariadb.dll,
         MariaDB Connector/C, LGPL-2.1), compilée ici avec les Build Tools ; l'original est conservé en libmySQL.dll.orig.
    Idempotent. À relancer après une réinstallation du composant VMaNGOS ou de MariaDB.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
$vm = Get-ProjectPath 'Server\vmangos'
$mdb = Get-ProjectPath 'Server\Database\mariadb'
$runtime = @('vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'concrt140.dll')

# 1. Runtime Visual C++ 2015-2022 et client MariaDB
foreach ($f in $runtime) { Copy-Item (Join-Path $mdb "bin\$f") (Join-Path $vm $f) -Force }
Copy-Item (Join-Path $mdb 'lib\libmariadb.dll') (Join-Path $vm 'libmariadb.dll') -Force
Write-Step "runtime Visual C++ $((Get-Item (Join-Path $vm 'vcruntime140.dll')).VersionInfo.FileVersion) et libmariadb.dll $((Get-Item (Join-Path $vm 'libmariadb.dll')).VersionInfo.FileVersion) copiés dans Server\vmangos"

# 2. Couche de compatibilité libmySQL.dll
$src = Join-Path $PSScriptRoot 'libmysql-shim'
$work = Join-Path $PSScriptRoot 'tmp\libmysql-shim'
New-Item -ItemType Directory -Force $work | Out-Null
$vcvars = @('C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat') + @(Get-ChildItem 'C:\Program Files*\Microsoft Visual Studio\*\*\VC\Auxiliary\Build\vcvars64.bat' -ErrorAction SilentlyContinue | ForEach-Object FullName) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $vcvars) { throw "vcvars64.bat introuvable : Build Tools 2022 requis (voir Launcher\README.md)" }
# Exportations de libmariadb.dll (dumpbin) -> directives de redirection pour l'éditeur de liens, sauf les trois
# fonctions écrites dans le .c (une redirection dans un .def n'est pas reconnue par link.exe sans bibliothèque d'import).
$exports = cmd /c "`"$vcvars`" >nul 2>&1 && dumpbin /nologo /exports `"$(Join-Path $vm 'libmariadb.dll')`"" | Where-Object { $_ -match '^\s+\d+\s+[0-9A-F]+\s+[0-9A-F]+\s+(\S+)' } | ForEach-Object { $Matches[1] }
if ($exports.Count -lt 100) { throw "exportations de libmariadb.dll illisibles ($($exports.Count))" }
$own = @('mysql_thread_init', 'mysql_thread_end', 'mysql_thread_safe')
$forwards = $exports | Where-Object { $own -notcontains $_ } | ForEach-Object { "#pragma comment(linker, `"/export:$_=libmariadb.$_`")" }
[IO.File]::WriteAllLines((Join-Path $work 'forwards.c'), [string[]](@('/* Généré par Build\Set-VmangosRuntime.ps1 : redirections vers libmariadb.dll. */') + $forwards))
$out = Join-Path $work 'libmySQL.dll'
Remove-Item $out -ErrorAction SilentlyContinue
$log = cmd /c "`"$vcvars`" >nul 2>&1 && cd /d `"$work`" && cl /nologo /c /O1 /GS- /Zl `"$(Join-Path $src 'libmysql_shim.c')`" forwards.c && link /nologo /DLL /NOENTRY /NODEFAULTLIB /OUT:libmySQL.dll libmysql_shim.obj forwards.obj"
if ($LASTEXITCODE -ne 0 -or -not (Test-Path $out) -or (Get-Item $out).Length -eq 0) { $log | Select-Object -Last 15 | Write-Host; throw "compilation de la couche libmySQL.dll échouée (voir $work)" }
$dll = Join-Path $vm 'libmySQL.dll'
if (-not (Test-Path "$dll.orig")) { Copy-Item $dll "$dll.orig" }
Copy-Item $out $dll -Force
Write-Step "libmySQL.dll remplacé par la couche de compatibilité ($($exports.Count) fonctions redirigées vers libmariadb.dll ; original : libmySQL.dll.orig)"
