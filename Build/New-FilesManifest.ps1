<#
.SYNOPSIS
    Génère Build/files-manifest.json : liste (chemin relatif, taille, SHA-256) des fichiers gérés par le launcher
    (binaires VMaNGOS, MariaDB, modèles de configuration, dumps SQL, HeidiSQL, launcher signé).
    Exécuté par le développeur à chaque packaging ; le launcher s'en sert pour VÉRIFIER / RÉPARER.
    Jamais : Client/, Server/Data/ (données Blizzard dérivées), UserData/, Logs/.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$roots = @('Server\vmangos', 'Server\Database\mariadb', 'Server\Database\sql', 'Tools\HeidiSQL', 'Build\Get-Components.ps1', 'Build\manifest.json', 'Launcher\target\release\vanilla-solo-launcher.exe')
$managedConfig = @('Server\Config\mangosd.conf.dist.in', 'Server\Config\realmd.conf.dist.in', 'Server\Config\LICENSE')

$files = New-Object System.Collections.Generic.List[object]
foreach ($r in $roots + $managedConfig) {
    $p = Join-Path $root $r
    if (-not (Test-Path $p)) { Write-Warning "absent : $r"; continue }
    $items = if ((Get-Item $p).PSIsContainer) { Get-ChildItem $p -Recurse -File } else { @(Get-Item $p) }
    foreach ($f in $items) {
        $rel = $f.FullName.Substring($root.Length + 1)
        $files.Add([pscustomobject]@{ path = $rel; size = $f.Length; sha256 = (Get-FileHash -Algorithm SHA256 $f.FullName).Hash.ToLowerInvariant() })
    }
}
$manifest = [pscustomobject]@{ generated = (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'); files = $files }
$out = Join-Path $PSScriptRoot 'files-manifest.json'
[System.IO.File]::WriteAllText($out, ($manifest | ConvertTo-Json -Depth 4), (New-Object System.Text.UTF8Encoding $false))
"$($files.Count) fichiers, $([math]::Round(($files | Measure-Object size -Sum).Sum/1MB)) Mo -> $out"
