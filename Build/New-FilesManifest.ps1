<#
.SYNOPSIS
    Génère Build/files-manifest.json : liste (chemin relatif, taille, SHA-256) des fichiers gérés par le launcher
    (binaires AzerothCore et DLL, modèles .conf.dist, MySQL portable, dumps SQL et compléments, scripts Tools,
    launcher à la racine). Exécuté par le développeur à chaque build/packaging ; le launcher s'en sert pour
    VÉRIFIER / RÉPARER. Jamais : Client/, Server/Data/ (données Blizzard dérivées), UserData/, Logs/, ni les .conf
    générés (ils contiennent les identifiants SQL et changent à chaque démarrage). Porté de ../HomeCraft/Build.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$roots = @('Server\acore', 'Server\Database\mysql', 'Server\Database\sql', 'Server\Database\custom', 'Tools', 'HomeCraft Edition WoTLK.exe', 'LICENCE.txt')
# Exclusions (chemin relatif, regex) : conf générées, fichiers d'exécution du serveur, outils tiers non livrés.
$exclude = '^Server\\acore\\configs\\(modules\\)?[^\\]+\.conf$', '^Server\\acore\\Crashes\\', '\.pid$', '^Tools\\HeidiSQL\\', '^Tools\\HermesProxy\\'

$files = New-Object System.Collections.Generic.List[object]
foreach ($r in $roots) {
    $p = Join-Path $root $r
    if (-not (Test-Path $p)) { Write-Warning "absent : $r"; continue }
    $items = if ((Get-Item $p).PSIsContainer) { Get-ChildItem $p -Recurse -File } else { @(Get-Item $p) }
    foreach ($f in $items) {
        $rel = $f.FullName.Substring($root.Length + 1)
        if (@($exclude | Where-Object { $rel -match $_ }).Count -gt 0) { continue }
        $files.Add([pscustomobject]@{ path = $rel; size = $f.Length; sha256 = (Get-FileHash -Algorithm SHA256 $f.FullName).Hash.ToLowerInvariant() })
    }
}
$manifest = [pscustomobject]@{ generated = (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'); files = $files }
$out = Join-Path $PSScriptRoot 'files-manifest.json'
[System.IO.File]::WriteAllText($out, ($manifest | ConvertTo-Json -Depth 4), (New-Object System.Text.UTF8Encoding $false))
"$($files.Count) fichiers, $([math]::Round(($files | Measure-Object size -Sum).Sum/1MB)) Mo -> $out"
