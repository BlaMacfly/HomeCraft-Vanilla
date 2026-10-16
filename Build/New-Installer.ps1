<#
.SYNOPSIS
    Construit l'installateur Windows HomeCraft-Edition-Vanilla-v<version>-setup.exe (Inno Setup 6, Tools\InnoSetup)
    à partir du package portable de Build\package (construit par New-Package.ps1 si absent ou avec -Rebuild),
    et le signe (installateur et désinstallateur) avec le certificat indiqué.

.PARAMETER Thumbprint  Empreinte du certificat de signature (Cert:\CurrentUser\My). Sans elle : installateur non signé.
.PARAMETER Rebuild     Reconstruit d'abord le package portable (New-Package.ps1 -Zip).
#>
[CmdletBinding()]
param([string]$Thumbprint, [switch]$Rebuild, [string]$TimestampUrl = 'http://timestamp.digicert.com')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
function Step { param($m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }

$cargo = Get-Content (Join-Path $root 'Launcher\Cargo.toml') -Raw
$version = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
$pkg = Join-Path $PSScriptRoot "package\HomeCraft-Edition-Vanilla-v$version"
if ($Rebuild -or -not (Test-Path (Join-Path $pkg 'HomeCraft Edition Vanilla.exe'))) {
    Step "Package portable"
    & (Join-Path $PSScriptRoot 'New-Package.ps1') -Zip
}
if (-not (Test-Path (Join-Path $PSScriptRoot 'installer-assets\wizard-164x314.bmp')) -or -not (Test-Path (Join-Path $root 'Launcher\assets\homecraft.ico'))) {
    Step "Images de l'installateur"
    & (Join-Path $PSScriptRoot 'New-InstallerImages.ps1') | Out-Null
}
$iscc = Join-Path $root 'Tools\InnoSetup\ISCC.exe'
if (-not (Test-Path $iscc)) { throw "Inno Setup absent : $iscc (composant innosetup du manifeste, installé avec /DIR=Tools\InnoSetup)" }

$args = @("/DAppVersion=$version", "/DSourceDir=$pkg", '/Qp')
if ($Thumbprint) {
    $signtool = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe' -ErrorAction SilentlyContinue | Sort-Object FullName -Descending | Select-Object -First 1
    if (-not $signtool) { throw "signtool.exe introuvable (SDK Windows absent)" }
    # Syntaxe Inno Setup : $q = guillemet, $f = fichier à signer.
    $cmd = '$q' + $signtool.FullName + '$q sign /fd SHA256 /td SHA256 /tr ' + $TimestampUrl + ' /sha1 ' + $Thumbprint + ' /s My /d $qHomeCraft Edition Vanilla$q $f'
    $args += @('/DSignSetup', ('/Shc=' + $cmd))
}
Step "Compilation Inno Setup (v$version, source $pkg)"
& $iscc @args (Join-Path $PSScriptRoot 'HomeCraft.iss')
if ($LASTEXITCODE -ne 0) { throw "ISCC a échoué (code $LASTEXITCODE)" }
$setup = Join-Path $PSScriptRoot "package\HomeCraft-Edition-Vanilla-v$version-setup.exe"
if (-not (Test-Path $setup)) { throw "Installateur non produit : $setup" }
$sig = Get-AuthenticodeSignature $setup
Step ("Installateur : {0:N0} Mo -> {1}" -f ((Get-Item $setup).Length / 1MB), $setup)
Step ("Signature : {0} ({1})" -f $sig.Status, $(if ($sig.SignerCertificate) { $sig.SignerCertificate.Subject } else { 'non signé' }))
$setup
