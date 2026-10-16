<#
.SYNOPSIS
    Modifie Server/vmangos/mangosd.exe pour accepter le build client 5880 (1.12.1) à la place de 6141 (1.12.3).

.DESCRIPTION
    mangosd (commit 53d24d5) n'accepte à la connexion monde que les builds d'une liste fixe {5875, 6005, 6141, 0}
    (comparaison exacte, réponse AUTH_VERSION_MISMATCH sinon) et écrit cette liste dans realmlist.realmbuilds à chaque
    démarrage. Le client frFR 1.12.1 fourni annonce le build 5880. Le binaire est un logiciel GPL-2.0 : cette
    modification (6141 -> 5880 dans la liste) est documentée ici et dans LICENCES/SOURCES.md du package ; l'original
    est conservé en mangosd.exe.orig. Idempotent. À remplacer, à terme, par une recompilation du cœur avec le patch source.
#>
[CmdletBinding()]
param([string]$File = (Join-Path (Split-Path -Parent $PSScriptRoot) 'Server\vmangos\mangosd.exe'))
$ErrorActionPreference = 'Stop'
if (-not (Test-Path $File)) { throw "Introuvable : $File" }
$b = [System.IO.File]::ReadAllBytes($File)
$orig    = [byte[]](0xF3,0x16,0,0, 0x75,0x17,0,0, 0xFD,0x17,0,0, 0,0,0,0)   # 5875, 6005, 6141, 0
$patched = [byte[]](0xF3,0x16,0,0, 0x75,0x17,0,0, 0xF8,0x16,0,0, 0,0,0,0)   # 5875, 6005, 5880, 0
function Find([byte[]]$hay,[byte[]]$needle){ for($i=0;$i -le $hay.Length-$needle.Length;$i++){ $ok=$true; for($j=0;$j -lt $needle.Length;$j++){ if($hay[$i+$j] -ne $needle[$j]){$ok=$false;break} }; if($ok){ return $i } }; return -1 }
if ((Find $b $patched) -ge 0) { "Déjà modifié : liste {5875, 6005, 5880}."; exit 0 }
$off = Find $b $orig
if ($off -lt 0) { throw "Liste de builds {5875, 6005, 6141, 0} introuvable : binaire différent de celui attendu (commit 53d24d5) ; ne rien modifier." }
if (-not (Test-Path "$File.orig")) { Copy-Item $File "$File.orig" }
[Array]::Copy($patched, 0, $b, $off, $patched.Length)
[System.IO.File]::WriteAllBytes($File, $b)
"Modifié à l'offset $off : 6141 -> 5880. Original conservé : $File.orig"
"SHA-256 : $((Get-FileHash -Algorithm SHA256 $File).Hash.ToLowerInvariant())"
