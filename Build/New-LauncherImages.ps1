<#
.SYNOPSIS
    Génère les ressources graphiques du launcher depuis les visuels fournis par l'utilisateur (Launcher\assets) :
    - logo-512.png / logo-256.png : logo « HomeCraft WoTLK » détouré (le PNG d'origine a un fond noir plein : les pixels
      noirs reliés aux bords deviennent transparents ; les zones sombres internes au médaillon sont conservées) ;
    - homecraft.ico : icône multi-tailles (16 à 256 px, entrées PNG) intégrée à l'exécutable par build.rs ;
    - fond.jpg : taverne (1680x944) en JPEG qualité 85, affichée en couverture et assombrie par l'interface.
    Reproductible : à relancer si une source change.
#>
[CmdletBinding()]
param([int]$BlackThreshold = 34)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$assets = Join-Path (Split-Path -Parent $PSScriptRoot) 'Launcher\assets'
$logoSrc = Join-Path $assets 'HomeCraft-WoTLK-original.png'
$fondSrc = Join-Path $assets 'fond-original.png'

function Resize-Image { param([System.Drawing.Image]$Src, [int]$W, [int]$H)
    $bmp = New-Object System.Drawing.Bitmap $W, $H, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.DrawImage($Src, 0, 0, $W, $H)
    $g.Dispose()
    return $bmp
}

# ---- 1. Détourage du logo : remplissage par diffusion depuis les quatre coins sur les pixels presque noirs.
$src = New-Object System.Drawing.Bitmap $logoSrc
$w = $src.Width; $h = $src.Height
$rect = New-Object System.Drawing.Rectangle 0, 0, $w, $h
$data = $src.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadWrite, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$stride = $data.Stride
$bytes = New-Object byte[] ($stride * $h)
[System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)
$visited = New-Object bool[] ($w * $h)
$queue = New-Object System.Collections.Generic.Queue[int]
foreach ($start in @(0, ($w - 1), ($w * ($h - 1)), ($w * $h - 1))) { $queue.Enqueue($start); $visited[$start] = $true }
$cleared = 0
while ($queue.Count -gt 0) {
    $i = $queue.Dequeue()
    $x = $i % $w; $y = [int][Math]::Floor($i / $w)
    $o = $y * $stride + $x * 4
    $b = $bytes[$o]; $g = $bytes[$o + 1]; $r = $bytes[$o + 2]
    $max = [Math]::Max($r, [Math]::Max($g, $b))
    if ($max -gt $BlackThreshold) { continue }
    # Transparence totale pour le noir, dégradé sur les derniers niveaux pour adoucir le contour.
    $bytes[$o + 3] = [byte][Math]::Min(255, [int]($max * 255 / $BlackThreshold) * 0)
    $cleared++
    foreach ($n in @(($i - 1), ($i + 1), ($i - $w), ($i + $w))) {
        if ($n -lt 0 -or $n -ge $w * $h) { continue }
        if (($n -eq $i - 1 -and $x -eq 0) -or ($n -eq $i + 1 -and $x -eq $w - 1)) { continue }
        if (-not $visited[$n]) { $visited[$n] = $true; $queue.Enqueue($n) }
    }
}
# Lissage du bord : les pixels opaques voisins d'un pixel transparent reçoivent une alpha proportionnelle à leur luminosité.
for ($y = 1; $y -lt $h - 1; $y++) {
    for ($x = 1; $x -lt $w - 1; $x++) {
        $o = $y * $stride + $x * 4
        if ($bytes[$o + 3] -eq 0) { continue }
        $edge = $false
        foreach ($d in @(-4, 4, -$stride, $stride)) { if ($bytes[$o + $d + 3] -eq 0) { $edge = $true; break } }
        if ($edge) {
            $max = [Math]::Max($bytes[$o + 2], [Math]::Max($bytes[$o + 1], $bytes[$o]))
            $bytes[$o + 3] = [byte][Math]::Max(40, [Math]::Min(255, $max * 3))
        }
    }
}
[System.Runtime.InteropServices.Marshal]::Copy($bytes, 0, $data.Scan0, $bytes.Length)
$src.UnlockBits($data)
Write-Host "Logo : $w x $h, $cleared pixels de fond rendus transparents"
$logo512 = Resize-Image $src 512 512
$logo512.Save((Join-Path $assets 'logo-512.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$logo256 = Resize-Image $src 256 256
$logo256.Save((Join-Path $assets 'logo-256.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$logo256.Dispose()

# ---- 2. Icône multi-tailles (ICO avec entrées PNG)
$sizes = 16, 24, 32, 48, 64, 128, 256
$pngs = foreach ($s in $sizes) {
    $b = Resize-Image $logo512 $s $s
    $ms = New-Object System.IO.MemoryStream
    $b.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $b.Dispose()
    ,($ms.ToArray())
}
$ico = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter $ico
$bw.Write([uint16]0); $bw.Write([uint16]1); $bw.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $s = $sizes[$i]; $len = $pngs[$i].Length
    $bw.Write([byte]$(if ($s -ge 256) { 0 } else { $s })); $bw.Write([byte]$(if ($s -ge 256) { 0 } else { $s }))
    $bw.Write([byte]0); $bw.Write([byte]0); $bw.Write([uint16]1); $bw.Write([uint16]32); $bw.Write([uint32]$len); $bw.Write([uint32]$offset)
    $offset += $len
}
foreach ($p in $pngs) { $bw.Write($p) }
$bw.Flush()
[System.IO.File]::WriteAllBytes((Join-Path $assets 'homecraft.ico'), $ico.ToArray())
Write-Host "Icône : homecraft.ico ($($ico.Length) octets)"
$logo512.Dispose(); $src.Dispose()

# ---- 3. Fond (JPEG 85, taille d'origine)
$fond = New-Object System.Drawing.Bitmap $fondSrc
$codec = [System.Drawing.Imaging.ImageCodecInfo]::GetImageEncoders() | Where-Object { $_.MimeType -eq 'image/jpeg' }
$params = New-Object System.Drawing.Imaging.EncoderParameters 1
$params.Param[0] = New-Object System.Drawing.Imaging.EncoderParameter ([System.Drawing.Imaging.Encoder]::Quality), 85L
$flat = New-Object System.Drawing.Bitmap $fond.Width, $fond.Height
$g = [System.Drawing.Graphics]::FromImage($flat); $g.DrawImage($fond, 0, 0, $fond.Width, $fond.Height); $g.Dispose()
$flat.Save((Join-Path $assets 'fond.jpg'), $codec, $params)
Write-Host "Fond : fond.jpg $($fond.Width)x$($fond.Height)"
$flat.Dispose(); $fond.Dispose()
Get-ChildItem $assets | ForEach-Object { "  $($_.Name)  $($_.Length) o" }
