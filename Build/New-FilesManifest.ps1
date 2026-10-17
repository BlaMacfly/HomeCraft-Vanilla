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
$roots = @('Server\mangos-tbc', 'Server\Database\mariadb', 'Server\Database\sql', 'Tools\HeidiSQL', 'Build\Get-Components.ps1', 'Build\manifest.json', 'Launcher\target\release\homecraft-bc-launcher.exe')
# Les modèles *.conf.dist sont dans Server\mangos-tbc (recopiés dans Server\Config par le launcher : non gérés ici)
$managedConfig = @()

$files = New-Object System.Collections.Generic.List[object]
foreach ($r in $roots + $managedConfig) {
    $p = Join-Path $root $r
    if (-not (Test-Path $p)) { Write-Warning "absent : $r"; continue }
    # Fichiers générés dans le dossier du programme : aiplayerbot.conf (lu dans le dossier courant) et Crashes\ (rapports de
    # plantage de mangosd, noms avec crochets : -LiteralPath obligatoire). Non gérés.
    $items = if ((Get-Item $p).PSIsContainer) { Get-ChildItem $p -Recurse -File | Where-Object { $_.Extension -ne '.conf' -and $_.FullName -notmatch '\\Crashes\\' } } else { @(Get-Item $p) }
    foreach ($f in $items) {
        $rel = $f.FullName.Substring($root.Length + 1)
        $files.Add([pscustomobject]@{ path = $rel; size = $f.Length; sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $f.FullName).Hash.ToLowerInvariant() })
    }
}
$manifest = [pscustomobject]@{ generated = (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'); files = $files }
$out = Join-Path $PSScriptRoot 'files-manifest.json'
[System.IO.File]::WriteAllText($out, ($manifest | ConvertTo-Json -Depth 4), (New-Object System.Text.UTF8Encoding $false))
"$($files.Count) fichiers, $([math]::Round(($files | Measure-Object size -Sum).Sum/1MB)) Mo -> $out"
