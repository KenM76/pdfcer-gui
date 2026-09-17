<#
.SYNOPSIS
    Photograph a screen rectangle, or a named window, right now.

.DESCRIPTION
    A camera with no opinions: the layout and clipping oracle this project
    holds itself to (`D:/dev/rag/egui/`) is a rendered screenshot, and a
    comparison against another application needs one that can be taken between
    two gestures driven by hand.

    With `-Process` it photographs that process's main window. With `-Full` it
    photographs the whole virtual desktop, which is what an open drop-down
    needs when the drop-down is its own top-level window positioned outside its
    owner's rectangle.

.PARAMETER Out
    Destination PNG.

.PARAMETER Process
    Process name whose main window to photograph.

.PARAMETER Full
    Photograph the primary screen instead of a window.

.PARAMETER Click
    "x,y" to click before capturing.

.PARAMETER DelayMs
    Wait between the click and the shutter.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Process = "",
    [switch]$Full,
    [string]$Click = "",
    [string]$Keys = "",
    [string]$Raise = "",
    [string]$Rect = "",
    [int]$DelayMs = 1200
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

$sig = @"
using System;
using System.Runtime.InteropServices;
public static class Shot {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, IntPtr e);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
"@
if (-not ("Shot" -as [type])) { Add-Type -TypeDefinition $sig }

if ($Raise -ne "") {
    $rp = Get-Process -Name $Raise -ErrorAction Stop | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
    # A background process cannot always take the foreground; the alt-key
    # nudge is the documented workaround and is what every driver here uses.
    [System.Windows.Forms.SendKeys]::SendWait("%")
    [void][Shot]::SetForegroundWindow($rp.MainWindowHandle)
    Start-Sleep -Milliseconds 500
}

if ($Click -ne "") {
    $p = $Click.Split(",")
    [void][Shot]::SetCursorPos([int]$p[0], [int]$p[1])
    Start-Sleep -Milliseconds 200
    [Shot]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 80
    [Shot]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
    Start-Sleep -Milliseconds $DelayMs
}
if ($Keys -ne "") {
    [System.Windows.Forms.SendKeys]::SendWait($Keys)
    Start-Sleep -Milliseconds $DelayMs
}

New-Item -ItemType Directory -Force -Path (Split-Path $Out -Parent) | Out-Null

if ($Rect -ne "") {
    # An explicit virtual-desktop rectangle, "x,y,w,h". Needed when the
    # subject is on a secondary monitor -- `PrimaryScreen.Bounds` is the wrong
    # answer there, and a process's "main" window is not always the one on
    # screen: a winit application owns several, and the first with a non-zero
    # handle can be an off-screen helper at (-32000,-32000).
    $p = $Rect.Split(",")
    $x = [int]$p[0]; $y = [int]$p[1]; $w = [int]$p[2]; $h = [int]$p[3]
} elseif ($Full -or $Process -eq "") {
    $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $x = $b.X; $y = $b.Y; $w = $b.Width; $h = $b.Height
} else {
    $proc = Get-Process -Name $Process -ErrorAction Stop | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
    $r = New-Object Shot+RECT
    [void][Shot]::GetWindowRect($proc.MainWindowHandle, [ref]$r)
    $x = $r.Left; $y = $r.Top; $w = $r.Right - $r.Left; $h = $r.Bottom - $r.Top
}

$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($x, $y, 0, 0, $bmp.Size)
$g.Dispose()
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output ("{0}  {1}x{2} at {3},{4}" -f $Out, $w, $h, $x, $y)
