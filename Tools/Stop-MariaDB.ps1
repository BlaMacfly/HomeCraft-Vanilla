<#
.SYNOPSIS
    Arrête proprement le serveur MariaDB portable du projet (shutdown SQL, jamais de kill).
#>
[CmdletBinding()]
param([int]$TimeoutSeconds = 60)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

if (-not (Test-MariaDbAlive)) { Write-Step "MariaDB n'est pas en service"; exit 0 }
$cred = Get-DbCredentials
& (Join-Path $Layout.MariaDbBin 'mariadb-admin.exe') --host=127.0.0.1 --port=$($Layout.MariaDbPort) --protocol=TCP --user=root --password="$($cred.rootPassword)" shutdown
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
while ((Test-MariaDbAlive) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 500 }
if (Test-MariaDbAlive) { throw "MariaDB répond toujours après $TimeoutSeconds s" }
Write-Step "MariaDB arrêté"
