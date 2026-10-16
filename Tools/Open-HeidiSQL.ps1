<#
.SYNOPSIS
    Ouvre HeidiSQL portable (Tools/HeidiSQL) directement connecté à la base locale du projet.
    Préfigure le bouton « Ouvrir dans HeidiSQL » du launcher : même ligne de commande.
#>
[CmdletBinding()]
param([switch]$AsRoot)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$exe = Get-ProjectPath 'Tools\HeidiSQL\heidisql.exe'
if (-not (Test-Path $exe)) { throw "HeidiSQL absent : $exe (Build\Get-Components.ps1 puis extraire dans Tools\HeidiSQL)" }
if (-not (Test-MariaDbAlive)) { & "$PSScriptRoot\Start-MariaDB.ps1" }

$cred = Get-DbCredentials
$user = if ($AsRoot) { 'root' } else { $cred.user }
$pwd  = if ($AsRoot) { $cred.rootPassword } else { $cred.password }
# Syntaxe Windows v12 : -clé=valeur ; les valeurs contenant un point entre guillemets. Mode portable : portable.lock présent.
$args = @('-d=Vanilla Solo (local)', '-n=0', '-h="127.0.0.1"', "-P=$($Layout.MariaDbPort)", "-u=$user", "-p=$pwd", '-db=characters;realmd;mangos;logs')
$p = Start-Process -FilePath $exe -ArgumentList $args -WorkingDirectory (Split-Path $exe) -PassThru
Write-Step "HeidiSQL lancé (PID $($p.Id)) sur 127.0.0.1:$($Layout.MariaDbPort) en tant que $user"
