<#
.SYNOPSIS
    Envoie une commande à la console de mangosd lancé par Run-World.ps1, et affiche la réponse (tail du journal).
.EXAMPLE
    .\Send-Console.ps1 "account create joueur motdepasse"
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory, Position = 0)][string]$Command,
    [int]$WaitSeconds = 3
)
. "$PSScriptRoot\Common.ps1"
$srvLog = Join-Path $Layout.Logs 'server'
$consoleIn = Join-Path $srvLog 'console.in'
$consoleOut = Join-Path $srvLog 'mangosd.console.log'
if (-not (Test-Path $consoleIn)) { throw "Run-World.ps1 ne semble pas actif ($consoleIn absent)" }
$before = (Get-Content $consoleOut -Encoding UTF8 -ErrorAction SilentlyContinue | Measure-Object).Count
Add-Content -Path $consoleIn -Value $Command -Encoding UTF8
Start-Sleep -Seconds $WaitSeconds
$after = @(Get-Content $consoleOut -Encoding UTF8 -ErrorAction SilentlyContinue)
if ($after.Count -gt $before) { $after[$before..($after.Count - 1)] }
