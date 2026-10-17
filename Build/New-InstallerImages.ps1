<#
.SYNOPSIS
    Génère l'icône Windows du launcher HomeCraft Edition BC :
    Launcher\assets\homecraft.ico, multi-tailles (16 à 256 px, entrées PNG) depuis Launcher\assets\logo-512.png,
    intégrée à l'exécutable par Launcher\build.rs (affichée dans l'Explorateur, la barre des tâches, les raccourcis).
    Reproductible : à relancer si le logo change. (Pas d'images d'installateur : distribution portable uniquement.)
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$root = Split-Path -Parent $PSScriptRoot
$logo = Join-Path $root 'Launcher\assets\logo-512.png'

function Resize-Image { param([System.Drawing.Image]$Src, [int]$W, [int]$H, [switch]$CoverCrop)
    $bmp = New-Object System.Drawing.Bitmap $W, $H
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)
    if ($CoverCrop) {
        # Remplit la cible en conservant les proportions, recadrage centrÃ© sur le haut (le titre est en haut du visuel).
        $scale = [Math]::Max($W / $Src.Width, $H / $Src.Height)
        $sw = [int][Math]::Round($W / $scale); $sh = [int][Math]::Round($H / $scale)
        $sx = [int](($Src.Width - $sw) / 2); $sy = 0
        $g.DrawImage($Src, (New-Object System.Drawing.Rectangle 0, 0, $W, $H), (New-Object System.Drawing.Rectangle $sx, $sy, $sw, $sh), [System.Drawing.GraphicsUnit]::Pixel)
    } else {
        $g.DrawImage($Src, 0, 0, $W, $H)
    }
    $g.Dispose()
    return $bmp
}

# ---- IcÃ´ne multi-tailles (format ICO avec entrÃ©es PNG, pris en charge depuis Windows Vista)
$src = [System.Drawing.Image]::FromFile($logo)
$sizes = 16, 24, 32, 48, 64, 128, 256
$pngs = foreach ($s in $sizes) {
    $b = Resize-Image $src $s $s
    $ms = New-Object System.IO.MemoryStream
    $b.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $b.Dispose()
    ,($ms.ToArray())
}
$ico = New-Object System.IO.MemoryStream
$w = New-Object System.IO.BinaryWriter $ico
$w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $s = $sizes[$i]; $len = $pngs[$i].Length
    $w.Write([byte]$(if ($s -ge 256) { 0 } else { $s })); $w.Write([byte]$(if ($s -ge 256) { 0 } else { $s }))
    $w.Write([byte]0); $w.Write([byte]0); $w.Write([uint16]1); $w.Write([uint16]32); $w.Write([uint32]$len); $w.Write([uint32]$offset)
    $offset += $len
}
foreach ($p in $pngs) { $w.Write($p) }
$w.Flush()
$icoPath = Join-Path $root 'Launcher\assets\homecraft.ico'
[System.IO.File]::WriteAllBytes($icoPath, $ico.ToArray())
Write-Host "IcÃ´ne : $icoPath ($($ico.Length) octets, tailles $($sizes -join ', '))"

$src.Dispose()
