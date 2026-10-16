<#
.SYNOPSIS
    GÃ©nÃ¨re les images de l'installateur et l'icÃ´ne Windows du launcher Ã  partir des sources du projet :
    - Launcher\assets\homecraft.ico : icÃ´ne multi-tailles (16 Ã  256 px, entrÃ©es PNG) depuis Launcher\assets\logo-512.png,
      intÃ©grÃ©e Ã  l'exÃ©cutable par Launcher\build.rs (affichÃ©e dans l'Explorateur, la barre des tÃ¢ches, les raccourcis) ;
    - Build\installer-assets\wizard-*.bmp : bandeau gauche de l'assistant Inno Setup (164x314 et variantes haute densitÃ©)
      depuis Build\installer-assets\visuel-source.png (visuel fourni par l'utilisateur, recadrÃ© en hauteur) ;
    - Build\installer-assets\wizard-small-*.bmp : petit logo en haut Ã  droite de l'assistant (55x55 et variantes).
    Reproductible : Ã  relancer si une source change.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$root = Split-Path -Parent $PSScriptRoot
$assets = Join-Path $PSScriptRoot 'installer-assets'
$logo = Join-Path $root 'Launcher\assets\logo-512.png'
$visual = Join-Path $assets 'visuel-source.png'

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

# ---- Petit logo de l'assistant (fond blanc : le BMP n'a pas de transparence)
foreach ($s in 55, 64, 83, 92, 110, 119, 138) {
    $b = Resize-Image $src $s $s
    $flat = New-Object System.Drawing.Bitmap $s, $s
    $g = [System.Drawing.Graphics]::FromImage($flat); $g.Clear([System.Drawing.Color]::White); $g.DrawImage($b, 0, 0); $g.Dispose()
    $flat.Save((Join-Path $assets "wizard-small-$s.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
    $b.Dispose(); $flat.Dispose()
}
$src.Dispose()

# ---- Bandeau gauche de l'assistant depuis le visuel (recadrage Â« cover Â»)
$vis = [System.Drawing.Image]::FromFile($visual)
foreach ($wh in @(@(164, 314), @(192, 386), @(246, 459), @(273, 556), @(328, 604), @(355, 700), @(410, 797))) {
    $b = Resize-Image $vis $wh[0] $wh[1] -CoverCrop
    $b.Save((Join-Path $assets "wizard-$($wh[0])x$($wh[1]).bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
    $b.Dispose()
}
$vis.Dispose()
Get-ChildItem $assets -Filter '*.bmp' | ForEach-Object { Write-Host ("{0,-24} {1,8} octets" -f $_.Name, $_.Length) }
