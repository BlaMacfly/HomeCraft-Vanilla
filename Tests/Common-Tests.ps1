<#
.SYNOPSIS
    Fonctions partagées par les tests du launcher (Tests/*.ps1). À « dot-sourcer » : . "$PSScriptRoot\Common-Tests.ps1"
    Porté de ../HomeCraft/Tests (édition Vanilla) : mêmes principes, binaires AzerothCore / MySQL 8.4.
    - Le launcher testé est « HomeCraft Edition WoTLK.exe » à la racine (lancé depuis la racine).
    - Aucun test ne démarre ni n'arrête un service tant que le serveur ou le launcher de l'utilisateur tourne :
      Assert-ServicesStopped s'arrête net si un port est occupé ou si un processus serveur existe.
#>
. "$PSScriptRoot\..\Tools\Common.ps1"

$script:LauncherExe = Get-ProjectPath 'HomeCraft Edition WoTLK.exe'
$script:LauncherLog = Get-ProjectPath 'Logs\launcher.log'
$script:ServerProcessNames = @('worldserver', 'authserver', 'mysqld', 'HermesProxy', 'HomeCraft Edition WoTLK', 'homecraft-wotlk-launcher')

function Test-Port {
    param([int]$Port)
    try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false }
}

function Get-ServerProcesses {
    # Processus serveur ou launcher encore vivants (vide si tout est arrêté).
    return @(Get-Process | Where-Object { $script:ServerProcessNames -contains $_.ProcessName })
}

function Assert-ServicesStopped {
    # Règle du projet : ne jamais toucher aux services pendant que le serveur/launcher de l'utilisateur tourne.
    $busy = @()
    foreach ($p in $Layout.DbPort, $Layout.AuthPort, $Layout.WorldPort) { if (Test-Port $p) { $busy += "port $p occupé" } }
    foreach ($p in Get-ServerProcesses) { $busy += "processus $($p.ProcessName) (PID $($p.Id))" }
    if ($busy.Count -gt 0) { throw "Serveur ou launcher en cours d'exécution : $($busy -join ', '). Arrêter d'abord (aucun test lancé)." }
}

function Assert-LauncherPresent {
    if (-not (Test-Path $script:LauncherExe)) { throw "Launcher introuvable : $($script:LauncherExe) (cargo build --release puis copie à la racine)" }
}

function Get-LogLineCount {
    if (Test-Path $script:LauncherLog) { return @(Get-Content $script:LauncherLog -Encoding UTF8).Count }
    return 0
}

function Get-LogSince {
    # Lignes du journal du launcher ajoutées depuis un relevé Get-LogLineCount.
    param([int]$Skip)
    if (-not (Test-Path $script:LauncherLog)) { return @() }
    return @(Get-Content $script:LauncherLog -Encoding UTF8 | Select-Object -Skip $Skip)
}

function Set-TestSettings {
    # Réglages imposés pendant les tests : pas de génération de mmaps en arrière-plan (20 min).
    # Le reste de settings.json est conservé (fichier créé avec les valeurs par défaut du launcher s'il manque).
    $path = Get-ProjectPath 'UserData\settings.json'
    $obj = if (Test-Path $path) { (Get-Content $path -Raw -Encoding UTF8) -replace '^﻿', '' | ConvertFrom-Json } else { [pscustomobject]@{} }
    if ($obj.PSObject.Properties['auto_mmaps']) { $obj.auto_mmaps = $false } else { $obj | Add-Member -NotePropertyName auto_mmaps -NotePropertyValue $false }
    New-Item -ItemType Directory -Force (Split-Path $path) | Out-Null
    [System.IO.File]::WriteAllText($path, ($obj | ConvertTo-Json -Depth 5), (New-Object System.Text.UTF8Encoding($false)))
}

function Invoke-Launcher {
    <#
    Lance le launcher avec des arguments, attend sa fin (délai maximal, puis arrêt forcé) et renvoie le texte du journal
    ajouté pendant l'exécution. $OnTick (facultatif) est appelé toutes les 500 ms avec le processus et le chronomètre.
    #>
    param(
        [string[]]$Arguments,
        [int]$TimeoutSeconds = 300,
        [scriptblock]$OnTick
    )
    Assert-LauncherPresent
    $before = Get-LogLineCount
    Write-Step "launcher : $($Arguments -join ' ')"
    $sw = [Diagnostics.Stopwatch]::StartNew()
    # Start-Process joint les arguments par des espaces sans les protéger : un chemin contenant un espace (la racine
    # « HomeCraft WoTLK ») serait coupé en deux (constaté sur --do restore <zip> : « Accès refusé » sur un dossier).
    $quoted = $Arguments | ForEach-Object { if ("$_" -match '\s') { '"' + $_ + '"' } else { "$_" } }
    $p = Start-Process -FilePath $script:LauncherExe -ArgumentList $quoted -WorkingDirectory $ProjectRoot -PassThru
    while (-not $p.HasExited) {
        if ($OnTick) { & $OnTick $p $sw }
        if ($sw.Elapsed.TotalSeconds -gt $TimeoutSeconds) { Write-Step "délai dépassé ($TimeoutSeconds s) : arrêt forcé du launcher"; $p.Kill(); break }
        Start-Sleep -Milliseconds 500
    }
    $p.WaitForExit()
    Write-Step "launcher terminé (code $($p.ExitCode)) après $([int]$sw.Elapsed.TotalSeconds) s"
    Start-Sleep -Seconds 2
    $lines = Get-LogSince $before
    return [pscustomobject]@{ ExitCode = $p.ExitCode; Seconds = [int]$sw.Elapsed.TotalSeconds; Lines = $lines; Text = ($lines -join "`n") }
}

function Close-LauncherWindow {
    <#
    Envoie WM_CLOSE à la fenêtre titrée « HomeCraft Edition WoTLK » du processus (même effet qu'un clic sur la croix).
    Ne pas utiliser Process.CloseMainWindow() : il peut viser la fenêtre interne sans titre de winit (16×16), ce qui
    gèle la boucle d'événements du launcher (constaté le 2026-10-16 : ni l'arrêt ni --quit-after ne s'exécutaient plus).
    #>
    param([int]$ProcessId, [string]$Title = 'HomeCraft Edition WoTLK')
    if (-not ('HcWin' -as [type])) {
        Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text;
public static class HcWin {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
}
"@
    }
    # Table modifiée en place par le rappel (une affectation « $target = … » dans le rappel créerait une variable locale).
    $state = @{ Target = [IntPtr]::Zero; Pid = $ProcessId; Title = $Title }
    $cb = [HcWin+EnumProc]{
        param($h, $l)
        $owner = 0
        [void][HcWin]::GetWindowThreadProcessId($h, [ref]$owner)
        if ($owner -eq $state.Pid -and [HcWin]::IsWindowVisible($h)) {
            $sb = New-Object Text.StringBuilder 256
            [void][HcWin]::GetWindowText($h, $sb, 256)
            if ($sb.ToString() -eq $state.Title) { $state.Target = $h; return $false }
        }
        return $true
    }
    [void][HcWin]::EnumWindows($cb, [IntPtr]::Zero)
    if ($state.Target -eq [IntPtr]::Zero) { return $false }
    return [HcWin]::PostMessage($state.Target, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)   # WM_CLOSE
}

function Save-WindowShot {
    # Capture d'écran de la plus grande fenêtre visible et titrée d'un processus (PNG). Renvoie le titre ou $null.
    param([int]$ProcessId, [string]$Path)
    if (-not ('HcShot' -as [type])) {
        Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text;
public static class HcShot {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
}
"@
    }
    $state = @{ Pid = $ProcessId; Best = $null; Area = 0 }
    $cb = [HcShot+EnumProc]{
        param($h, $l)
        $owner = 0
        [void][HcShot]::GetWindowThreadProcessId($h, [ref]$owner)
        if ($owner -eq $state.Pid -and [HcShot]::IsWindowVisible($h)) {
            $sb = New-Object Text.StringBuilder 256
            [void][HcShot]::GetWindowText($h, $sb, 256)
            $r = New-Object HcShot+R
            [void][HcShot]::GetWindowRect($h, [ref]$r)
            $area = ($r.Rt - $r.L) * ($r.B - $r.T)
            if ($sb.Length -gt 0 -and $area -gt $state.Area) { $state.Area = $area; $state.Best = [pscustomobject]@{ Title = $sb.ToString(); R = $r } }
        }
        return $true
    }
    [void][HcShot]::EnumWindows($cb, [IntPtr]::Zero)
    if (-not $state.Best) { return $null }
    $r = $state.Best.R
    Add-Type -AssemblyName System.Drawing
    $bmp = New-Object Drawing.Bitmap ($r.Rt - $r.L), ($r.B - $r.T)
    $g = [Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
    $g.Dispose()
    $bmp.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    return $state.Best.Title
}

function Write-Results {
    # Affiche un tableau de contrôles (dictionnaire ordonné nom -> booléen) et renvoie $true si tout est OK.
    param([System.Collections.Specialized.OrderedDictionary]$Results)
    $ok = $true
    $w = ($Results.Keys | ForEach-Object { $_.Length } | Measure-Object -Maximum).Maximum
    foreach ($k in $Results.Keys) {
        $pass = [bool]$Results[$k]
        Write-Step ("{0,-$w} {1}" -f $k, $(if ($pass) { 'OK' } else { 'ÉCHEC' }))
        if (-not $pass) { $ok = $false }
    }
    return $ok
}

function Read-ZipEntry {
    # Contenu texte d'une entrée d'archive zip (ou $null si absente).
    param([string]$Zip, [string]$Entry)
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $z = [System.IO.Compression.ZipFile]::OpenRead($Zip)
    try {
        $e = $z.GetEntry($Entry)
        if (-not $e) { return $null }
        $r = New-Object IO.StreamReader($e.Open())
        try { return $r.ReadToEnd() } finally { $r.Dispose() }
    } finally { $z.Dispose() }
}

function Get-ZipEntryNames {
    param([string]$Zip)
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $z = [System.IO.Compression.ZipFile]::OpenRead($Zip)
    try { return @($z.Entries | ForEach-Object { $_.FullName }) } finally { $z.Dispose() }
}
