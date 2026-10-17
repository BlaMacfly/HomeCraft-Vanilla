<#
.SYNOPSIS
    Lance le launcher sur un onglet donné et photographie sa fenêtre (rectangle DWM, sans l'ombre)
    dans Documentation\images\launcher-<onglet>.png. Sert aux captures du README et du manuel.
.PARAMETER Tab        Onglet : home, accounts, bots, world, patches, database, backups, settings, logs.
.PARAMETER Delay      Secondes d'attente avant la capture (chargement des textures).
.PARAMETER QuitAfter  Secondes avant la fermeture automatique du launcher (--quit-after).
.NOTES
    Ne démarre aucun service (pas de --start). Ne pas lancer pendant qu'un launcher de l'utilisateur tourne.
#>
[CmdletBinding()]
param(
    [ValidateSet('home', 'accounts', 'bots', 'world', 'patches', 'database', 'backups', 'settings', 'logs')]
    [string]$Tab = 'home',
    [int]$Delay = 6,
    [int]$QuitAfter = 14,
    [string]$Exe = (Join-Path (Split-Path -Parent $PSScriptRoot) 'HomeCraft Edition BC.exe'),
    [string]$OutDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'Documentation\images')
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class Dwm {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
    [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int attr, out RECT r, int size);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    public delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr l);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
    // Plus grande fenêtre visible du processus (MainWindowHandle peut désigner une fenêtre utilitaire minuscule).
    public static IntPtr FindMain(uint pid) {
        IntPtr best = IntPtr.Zero; long bestArea = 0;
        EnumWindows((h, l) => {
            uint p; GetWindowThreadProcessId(h, out p);
            if (p == pid && IsWindowVisible(h)) { RECT r; GetWindowRect(h, out r); long a = (long)(r.R - r.L) * (r.B - r.T); if (a > bestArea) { bestArea = a; best = h; } }
            return true; }, IntPtr.Zero);
        return best;
    }
}
"@
if (-not (Test-Path $Exe)) { throw "Launcher introuvable : $Exe" }
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir | Out-Null }
$p = Start-Process -FilePath $Exe -ArgumentList "--tab $Tab --quit-after $QuitAfter" -PassThru
Start-Sleep -Seconds $Delay
$h = [Dwm]::FindMain([uint32]$p.Id)
if ($h -eq [IntPtr]::Zero) { throw "Fenêtre introuvable (processus $($p.Id))" }
# Rendu de la fenêtre par PrintWindow (PW_RENDERFULLCONTENT) : aucune prise de focus, capture correcte même si une
# autre fenêtre (le jeu de l'utilisateur) est au premier plan. Puis recadrage sur le rectangle DWM (sans l'ombre).
$wr = New-Object Dwm+RECT; [void][Dwm]::GetWindowRect($h, [ref]$wr)
$r = New-Object Dwm+RECT
[void][Dwm]::DwmGetWindowAttribute($h, 9, [ref]$r, [System.Runtime.InteropServices.Marshal]::SizeOf($r))  # DWMWA_EXTENDED_FRAME_BOUNDS
$full = New-Object System.Drawing.Bitmap ($wr.R - $wr.L), ($wr.B - $wr.T)
$g = [System.Drawing.Graphics]::FromImage($full); $hdc = $g.GetHdc()
[void][Dwm]::PrintWindow($h, $hdc, 2)
$g.ReleaseHdc($hdc); $g.Dispose()
$w = $r.R - $r.L; $hh = $r.B - $r.T
$bmp = $full.Clone((New-Object System.Drawing.Rectangle ($r.L - $wr.L), ($r.T - $wr.T), $w, $hh), $full.PixelFormat)
$full.Dispose()
$out = Join-Path $OutDir "launcher-$Tab.png"
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
Write-Host "Capture : $out (${w}x${hh})"
# Attend la fermeture automatique (boucle de sondage : Start-Process -Wait attendrait aussi d'éventuels enfants)
while (-not $p.HasExited) { Start-Sleep -Milliseconds 500 }
