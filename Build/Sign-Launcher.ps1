<#
.SYNOPSIS
    Signe numériquement l'exécutable du launcher (Authenticode, SHA-256, horodatage RFC 3161) avec signtool du SDK Windows.

.PARAMETER PfxPath      Certificat de signature de code au format PFX (avec -PfxPassword), OU
.PARAMETER Thumbprint   Empreinte d'un certificat de signature de code présent dans Cert:\CurrentUser\My.
.PARAMETER File         Fichier à signer (défaut : Launcher\target\release\vanilla-solo-launcher.exe).

.NOTES
    Un certificat auto-signé permet de vérifier l'intégrité mais n'évite pas l'avertissement SmartScreen :
    pour une distribution publique sans avertissement, il faut un certificat délivré par une autorité reconnue
    (OV/EV, ou Azure Trusted Signing). Voir Documentation/04-Launcher-Exigences.md.
#>
[CmdletBinding()]
param(
    [string]$PfxPath,
    [string]$PfxPassword,
    [string]$Thumbprint,
    [string]$File = (Join-Path (Split-Path -Parent $PSScriptRoot) 'Launcher\target\release\vanilla-solo-launcher.exe'),
    [string]$TimestampUrl = 'http://timestamp.digicert.com',
    [string]$Description = 'HomeCraft Edition Vanilla'
)
$ErrorActionPreference = 'Stop'

$signtool = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe' -ErrorAction SilentlyContinue | Sort-Object FullName -Descending | Select-Object -First 1
if (-not $signtool) { throw "signtool.exe introuvable (SDK Windows absent)" }
if (-not (Test-Path $File)) { throw "Fichier à signer introuvable : $File" }

$args = @('sign', '/fd', 'SHA256', '/td', 'SHA256', '/tr', $TimestampUrl, '/d', $Description)
if ($PfxPath) {
    $args += @('/f', $PfxPath)
    if ($PfxPassword) { $args += @('/p', $PfxPassword) }
} elseif ($Thumbprint) {
    $args += @('/sha1', $Thumbprint, '/s', 'My')
} else {
    throw "Indiquer -PfxPath (et -PfxPassword) ou -Thumbprint"
}
$args += $File

& $signtool.FullName @args
if ($LASTEXITCODE -ne 0) { throw "signtool a échoué (code $LASTEXITCODE)" }
# Vérification de la chaîne : échoue (sans gravité) avec un certificat auto-signé non installé dans « Autorités racines ».
cmd /c "`"$($signtool.FullName)`" verify /pa /q `"$File`" >nul 2>&1"
$chainOk = ($LASTEXITCODE -eq 0)
$sig = Get-AuthenticodeSignature $File
"Signature : $($sig.Status) - $($sig.SignerCertificate.Subject)"
"Horodatage : $(if ($sig.TimeStamperCertificate) { $sig.TimeStamperCertificate.Subject } else { 'absent' })"
if (-not $chainOk) { "Chaîne non approuvée par Windows (certificat auto-signé ou autorité inconnue) : Windows affichera « éditeur inconnu »." }
# Copie à la racine du projet sous le nom définitif (le launcher détecte la racine par la présence de Server/ et Tools/).
$deploy = Join-Path (Split-Path -Parent $PSScriptRoot) 'HomeCraft Edition Vanilla.exe'
try { Copy-Item $File $deploy -Force -ErrorAction Stop; "Déployé : $deploy" } catch { "Déploiement impossible (launcher en cours d'exécution ?) : $($_.Exception.Message) — fermer le launcher puis relancer ce script." }
exit 0
