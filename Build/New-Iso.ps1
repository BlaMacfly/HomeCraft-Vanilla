<#
.SYNOPSIS
    Crée une image ISO (UDF + Joliet, bootable non requis) du dossier package, avec l'API IMAPI2 de Windows,
    sans outil externe. L'image peut être gravée avec l'outil de gravure intégré à Windows (clic droit > Graver).
.PARAMETER Source   Dossier package (défaut : le dernier Build/package/HomeCraft-Edition-Vanilla-v*).
#>
[CmdletBinding()]
param(
    [string]$Source,
    [string]$Label = 'HOMECRAFT_VANILLA'
)
$ErrorActionPreference = 'Stop'
if (-not $Source) { $Source = (Get-ChildItem (Join-Path $PSScriptRoot 'package') -Directory -Filter 'HomeCraft-Edition-Vanilla-v*' | Sort-Object Name -Descending | Select-Object -First 1).FullName }
if (-not $Source -or -not (Test-Path $Source)) { throw "Dossier package introuvable (lancer New-Package.ps1)" }
$iso = "$Source.iso"

# Pont COM -> fichier pour IStream (IMAPI2FS fournit un IStream, pas un fichier).
if (-not ([System.Management.Automation.PSTypeName]'IsoWriter').Type) {
Add-Type -TypeDefinition @"
using System; using System.IO; using System.Runtime.InteropServices.ComTypes;
public static class IsoWriter {
    public static void Write(string path, object stream, int blockSize, int blockCount) {
        var s = (IStream)stream;
        using (var fs = new FileStream(path, FileMode.Create, FileAccess.Write)) {
            var buf = new byte[blockSize];
            var ptr = System.Runtime.InteropServices.Marshal.AllocHGlobal(4);
            try {
                while (blockCount-- > 0) {
                    s.Read(buf, blockSize, ptr);
                    int read = System.Runtime.InteropServices.Marshal.ReadInt32(ptr);
                    if (read <= 0) break;
                    fs.Write(buf, 0, read);
                }
            } finally { System.Runtime.InteropServices.Marshal.FreeHGlobal(ptr); }
        }
    }
}
"@
}

$fsi = New-Object -ComObject IMAPI2FS.MsftFileSystemImage
$fsi.FileSystemsToCreate = 7   # ISO9660 + Joliet + UDF
$fsi.VolumeName = $Label
$fsi.Root.AddTree($Source, $false)
$result = $fsi.CreateResultImage()
[IsoWriter]::Write($iso, $result.ImageStream, $result.BlockSize, $result.TotalBlocks)
$size = (Get-Item $iso).Length
"ISO : {0} ({1:N0} Mo){2}" -f $iso, ($size / 1MB), $(if ($size -gt 700MB) { ' — DVD requis' } else { ' — compatible CD-ROM' })
"SHA-256 : $((Get-FileHash -Algorithm SHA256 $iso).Hash.ToLowerInvariant())"
