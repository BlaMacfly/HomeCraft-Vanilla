<#
.SYNOPSIS
    Récupère les composants redistribuables listés dans Build/manifest.json et vérifie leur intégrité.

.DESCRIPTION
    - Télécharge chaque composant dans Build/downloads/ (sauf -VerifyOnly).
    - Calcule le SHA-256 ; si le manifeste a une empreinte, elle doit correspondre (sinon erreur).
      Si le manifeste n'en a pas encore, l'empreinte est enregistrée (premier gel) avec -Pin.
    - Vérifie la taille attendue.
    - Liste le contenu de chaque archive dans Build/downloads/<fichier>.listing.txt.
    Aucun fichier propriétaire (client WoW) n'est jamais téléchargé par ce script.

.PARAMETER VerifyOnly
    Ne télécharge rien ; vérifie seulement les fichiers déjà présents.

.PARAMETER Pin
    Autorise l'écriture des empreintes manquantes dans le manifeste.
#>
[CmdletBinding()]
param(
    [switch]$VerifyOnly,
    [switch]$Pin
)

$ErrorActionPreference = 'Stop'
$buildDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$manifestPath = Join-Path $buildDir 'manifest.json'
$downloadDir = Join-Path $buildDir 'downloads'

if (-not (Test-Path $manifestPath)) { throw "Manifeste introuvable : $manifestPath" }
$manifest = Get-Content $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
if (-not (Test-Path $downloadDir)) { New-Item -ItemType Directory -Path $downloadDir | Out-Null }

Add-Type -AssemblyName System.IO.Compression.FileSystem

$failures = 0
$manifestChanged = $false

foreach ($c in $manifest.components) {
    $target = Join-Path $downloadDir $c.file
    Write-Host "== $($c.id) ($($c.file))"

    if (-not (Test-Path $target)) {
        if ($VerifyOnly) {
            Write-Host "   ABSENT (mode VerifyOnly, pas de téléchargement)" -ForegroundColor Yellow
            $failures++
            continue
        }
        Write-Host "   Téléchargement depuis $($c.url)"
        $tmp = "$target.part"
        Invoke-WebRequest -Uri $c.url -OutFile $tmp -UseBasicParsing
        Move-Item -Force $tmp $target
    }

    $size = (Get-Item $target).Length
    if ($c.expected_size -and $size -ne $c.expected_size) {
        Write-Host "   TAILLE INATTENDUE : $size octets (attendu $($c.expected_size))" -ForegroundColor Red
        $failures++
        continue
    }

    $hash = (Get-FileHash -Algorithm SHA256 -Path $target).Hash.ToLowerInvariant()
    if ([string]::IsNullOrEmpty($c.sha256)) {
        if ($Pin) {
            $c.sha256 = $hash
            $manifestChanged = $true
            Write-Host "   SHA-256 figé : $hash" -ForegroundColor Cyan
        } else {
            Write-Host "   SHA-256 calculé (non figé, relancer avec -Pin) : $hash" -ForegroundColor Yellow
        }
    } elseif ($c.sha256 -ne $hash) {
        Write-Host "   SHA-256 DIFFÉRENT : $hash (attendu $($c.sha256))" -ForegroundColor Red
        $failures++
        continue
    } else {
        Write-Host "   SHA-256 OK"
    }

    # Inventaire de l'archive (sans extraction) ; les installateurs .exe (outils de développement) ne sont pas des zips.
    if ([IO.Path]::GetExtension($target) -ne '.zip') { Write-Host "   (pas une archive zip : inventaire ignoré)"; continue }
    $listing = "$target.listing.txt"
    $zip = [System.IO.Compression.ZipFile]::OpenRead($target)
    try {
        $zip.Entries | ForEach-Object { '{0,12}  {1}' -f $_.Length, $_.FullName } | Set-Content -Path $listing -Encoding UTF8
        Write-Host "   $($zip.Entries.Count) entrées listées dans $(Split-Path -Leaf $listing)"
    } finally { $zip.Dispose() }
}

if ($manifestChanged) {
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -Path $manifestPath -Encoding UTF8
    Write-Host "Manifeste mis à jour."
}

if ($failures -gt 0) {
    Write-Host "$failures composant(s) en échec." -ForegroundColor Red
    exit 1
}
Write-Host "Tous les composants sont vérifiés." -ForegroundColor Green
exit 0
