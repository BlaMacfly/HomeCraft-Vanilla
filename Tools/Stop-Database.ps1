<#
.SYNOPSIS
    Arrête proprement le serveur MySQL portable du projet (mysqladmin shutdown, jamais de kill).
#>
[CmdletBinding()]
param([int]$TimeoutSeconds = 90)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

if (-not (Test-DbAlive)) { Write-Step "MySQL n'est pas en service"; exit 0 }
$cred = Get-DbCredentials
& (Join-Path $Layout.DbBin 'mysqladmin.exe') --host=127.0.0.1 --port=$($Layout.DbPort) --protocol=TCP --user=root --password="$($cred.rootPassword)" shutdown
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
while ((Test-DbAlive) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 500 }
if (Test-DbAlive) { throw "MySQL répond toujours après $TimeoutSeconds s" }
Write-Step "MySQL arrêté"
