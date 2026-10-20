<#
.SYNOPSIS
    Configure le client 1.12.1 pour le serveur local : realmlist.wtf -> 127.0.0.1, WTF/Config.wtf avec locale frFR.
    L'original de realmlist.wtf est conservé une fois (realmlist.wtf.orig). Les autres réglages de Config.wtf
    appartenant à l'utilisateur sont préservés : seules les clés gérées sont (ré)écrites.
#>
[CmdletBinding()]
param(
    [string]$ClientDir = '',
    [switch]$Windowed
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"
if (-not $ClientDir) { $ClientDir = Get-ProjectPath 'Client' }

if (-not (Test-Path (Join-Path $ClientDir 'WoW.exe'))) { throw "WoW.exe introuvable dans $ClientDir" }

# realmlist.wtf
$rl = Join-Path $ClientDir 'realmlist.wtf'
if ((Test-Path $rl) -and -not (Test-Path "$rl.orig")) { Copy-Item $rl "$rl.orig" }
Set-Content -Path $rl -Value "set realmlist 127.0.0.1`r`nset patchlist 127.0.0.1" -Encoding ASCII -NoNewline
Write-Step "realmlist.wtf -> 127.0.0.1"

# WTF/Config.wtf : fusion clé par clé
$wtfDir = Join-Path $ClientDir 'WTF'
New-Item -ItemType Directory -Force $wtfDir | Out-Null
$cfgPath = Join-Path $wtfDir 'Config.wtf'
$managed = [ordered]@{
    locale    = 'frFR'
    realmList = '127.0.0.1'
    realmName = 'Vanilla Solo'
    patchlist = '127.0.0.1'
    gxWindow  = $(if ($Windowed) { '1' } else { '0' })
}
$lines = @()
if (Test-Path $cfgPath) { $lines = @(Get-Content $cfgPath -Encoding ASCII) }
$result = New-Object System.Collections.Generic.List[string]
$done = @{}
foreach ($l in $lines) {
    if ($l -match '^SET\s+(\S+)\s+"(.*)"\s*$' -and $managed.Contains($Matches[1])) {
        $k = $Matches[1]; $result.Add("SET $k `"$($managed[$k])`""); $done[$k] = $true
    } else { $result.Add($l) }
}
foreach ($k in $managed.Keys) { if (-not $done[$k]) { $result.Add("SET $k `"$($managed[$k])`"") } }
Set-Content -Path $cfgPath -Value ($result -join "`r`n") -Encoding ASCII
Write-Step "Config.wtf : $($managed.Keys -join ', ') écrits ($($result.Count) lignes au total)"
