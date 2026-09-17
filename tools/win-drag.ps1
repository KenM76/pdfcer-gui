<#
.SYNOPSIS
    Press, drag, photograph mid-gesture, release, photograph again.

.DESCRIPTION
    `tools/win-shot.ps1` can click. It cannot hold a button down, and a
    *live preview* is by definition only on screen between the press and the
    release — so a resize ghost, a rubber band or a drag outline is invisible
    to every driver this project had.

    This holds the button, walks the pointer to the destination in steps (one
    jump produces a single large delta that some hit tests treat as a click
    with noise rather than as a drag), photographs while still held, releases,
    and photographs again. The pair is the evidence: the first frame says what
    the operator sees while dragging, the second says what was committed.

.PARAMETER From, To
    "x,y" in virtual-desktop coordinates.

.PARAMETER OutDrag, OutDrop
    The two PNGs.

.PARAMETER Rect
    "x,y,w,h" to photograph. Same meaning as win-shot.ps1's.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$From,
    [Parameter(Mandatory = $true)][string]$To,
    [Parameter(Mandatory = $true)][string]$OutDrag,
    [Parameter(Mandatory = $true)][string]$OutDrop,
    [Parameter(Mandatory = $true)][string]$Rect,
    [int]$Steps = 10,
    [int]$SettleMs = 700
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$sig = @"
using System;
using System.Runtime.InteropServices;
public static class Drag {
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, IntPtr e);
}
"@
if (-not ("Drag" -as [type])) { Add-Type -TypeDefinition $sig }

function Shoot {
    param([string]$Path, [string]$R)
    $p = $R.Split(",")
    $x = [int]$p[0]; $y = [int]$p[1]; $w = [int]$p[2]; $h = [int]$p[3]
    New-Item -ItemType Directory -Force -Path (Split-Path $Path -Parent) | Out-Null
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($x, $y, 0, 0, $bmp.Size)
    $g.Dispose()
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

$a = $From.Split(","); $ax = [int]$a[0]; $ay = [int]$a[1]
$b = $To.Split(",");   $bx = [int]$b[0]; $by = [int]$b[1]

[void][Drag]::SetCursorPos($ax, $ay)
Start-Sleep -Milliseconds 250
[Drag]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)   # LEFTDOWN
Start-Sleep -Milliseconds 150

for ($i = 1; $i -le $Steps; $i++) {
    $x = [int]($ax + ($bx - $ax) * $i / $Steps)
    $y = [int]($ay + ($by - $ay) * $i / $Steps)
    [void][Drag]::SetCursorPos($x, $y)
    Start-Sleep -Milliseconds 40
}
Start-Sleep -Milliseconds $SettleMs
Shoot -Path $OutDrag -R $Rect

[Drag]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)   # LEFTUP
Start-Sleep -Milliseconds $SettleMs
Shoot -Path $OutDrop -R $Rect

Write-Output ("drag {0} -> {1}; {2} and {3}" -f $From, $To, (Split-Path $OutDrag -Leaf), (Split-Path $OutDrop -Leaf))
