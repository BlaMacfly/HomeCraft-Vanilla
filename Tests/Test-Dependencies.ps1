<#
.SYNOPSIS
    Vérifie qu'aucun programme du package ne dépend d'un « Visual C++ Redistributable » installé dans Windows.
.DESCRIPTION
    Pour chaque .exe/.dll des dossiers exécutables du package (racine, Server\vmangos, MariaDB) : chaque DLL importée
    (dumpbin /dependents) doit être soit une DLL de Windows 10/11 (présente dans System32 et hors runtimes Visual C++),
    soit présente dans le même dossier ; aucune dépendance à un assemblage côte à côte Visual C++ 2005/2008
    (Microsoft.VC80/VC90.CRT) n'est admise. Les redirections d'exportation (couche libmySQL.dll) sont vérifiées aussi.
    Ce PC ayant les runtimes installés, c'est le seul moyen de garantir le fonctionnement sur un Windows neuf.
#>
[CmdletBinding()]
param([string]$Root = '')
$ErrorActionPreference = 'Stop'
$dev = Split-Path -Parent $PSScriptRoot
if (-not $Root) {
    $version = ((Get-Content (Join-Path $dev 'Launcher\Cargo.toml')) -match '^version' | Select-Object -First 1) -replace '.*"(.*)".*', '$1'
    $Root = Join-Path $dev "Build\package\HomeCraft-Edition-Vanilla-v$version"
}
if (-not (Test-Path $Root)) { throw "dossier introuvable : $Root" }
$dumpbin = Get-ChildItem 'C:\BuildTools', 'C:\Program Files*\Microsoft Visual Studio' -Recurse -Filter dumpbin.exe -ErrorAction SilentlyContinue | Where-Object { $_.FullName -match 'Hostx64\\x64' } | Select-Object -First 1
if (-not $dumpbin) { throw "dumpbin.exe introuvable (Build Tools 2022)" }
$vcRuntime = '^(msvc[pr]\d+|vcruntime\d+|concrt\d+|vcomp\d+|mfc\d+|vccorlib\d+|ucrtbased)'
$sys32 = Join-Path $env:WINDIR 'System32'
function Step { param([string]$m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }

$dirs = @($Root, (Join-Path $Root 'Server\vmangos'), (Join-Path $Root 'Server\Database\mariadb\bin'))
$failures = New-Object System.Collections.Generic.List[string]
$checked = 0
foreach ($d in $dirs) {
    foreach ($f in Get-ChildItem $d -File | Where-Object { $_.Extension -in '.exe', '.dll' }) {
        $checked++
        $deps = & $dumpbin.FullName /nologo /dependents $f.FullName | Where-Object { $_ -match '^\s+(\S+\.dll)\s*$' } | ForEach-Object { $Matches[1] }
        foreach ($dep in $deps) {
            $local = Test-Path (Join-Path $d $dep)
            $os = ($dep -match '^api-ms-win-|^ext-ms-win-') -or ((Test-Path (Join-Path $sys32 $dep)) -and ($dep -notmatch $vcRuntime))
            if (-not $local -and -not $os) { $failures.Add("$($f.FullName.Substring($Root.Length)) : $dep introuvable hors runtime Visual C++ installé") }
        }
        # Redirections d'exportation : la DLL cible doit être à côté.
        $fwd = & $dumpbin.FullName /nologo /exports $f.FullName | Where-Object { $_ -match 'forwarded to ([^.]+)\.' } | ForEach-Object { $Matches[1] } | Sort-Object -Unique
        foreach ($t in $fwd) { if (-not (Test-Path (Join-Path $d "$t.dll")) -and -not (Test-Path (Join-Path $sys32 "$t.dll"))) { $failures.Add("$($f.FullName.Substring($Root.Length)) : redirection vers $t.dll absente") } }
        $text = [Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes($f.FullName))
        if ($text -match 'Microsoft\.VC[89]0\.(CRT|MFC)') { $failures.Add("$($f.FullName.Substring($Root.Length)) : dépend d'un assemblage Visual C++ 2005/2008 ($($Matches[0]))") }
    }
}
Step "$checked fichiers vérifiés dans $Root"
if ($failures.Count -eq 0) { Step "TEST RÉUSSI : aucune dépendance à un runtime Visual C++ installé"; exit 0 }
$failures | ForEach-Object { Step "ÉCHEC  $_" }
Step "TEST EN ÉCHEC ($($failures.Count))"
exit 1
