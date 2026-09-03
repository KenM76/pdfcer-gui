<#
.SYNOPSIS
    Photograph what a COLLAPSED Word ribbon group looks like when opened.

.DESCRIPTION
    `OPERATOR_REQUESTS.md` O31, second half. `word-ribbon-study.ps1` captures
    the collapse ladder; this captures the other side of it — when a group has
    been reduced to a single button with a chevron, what does pressing that
    button give you?

    The answer is the whole justification for the collapse being acceptable at
    all: if a collapsed group's contents are *unreachable*, collapsing is data
    loss and the honest response to a narrow window would be a scroll bar
    instead. If they are one click away in their full layout, collapsing is
    free.

    Driven by KEYBOARD, not by a click at a guessed coordinate: Word publishes
    KeyTips for every ribbon control, and `Alt` then the group's letter opens
    exactly the collapsed group without this script having to know where the
    button ended up. A click would need the coordinate, and the coordinate is
    the thing the collapse just changed.
#>
[CmdletBinding()]
param(
    [string]$OutDir = "D:\Dev\pdfcer-gui\evidence\word-ribbon",
    [int]$Width = 900,
    [int]$Height = 900,
    # Home tab, then the KeyTip for one collapsed group. `L` is Styles on the
    # Home tab; it is a collapsed single button at this width.
    [string]$Keys = "%h",
    [string]$GroupKey = "l",
    [string]$Name = "popup-styles"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

$sig = @"
using System;
using System.Runtime.InteropServices;
public static class WinP {
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
"@
if (-not ("WinP" -as [type])) { Add-Type -TypeDefinition $sig }

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$word = New-Object -ComObject Word.Application
$word.Visible = $true
$doc = $word.Documents.Add()
Start-Sleep -Milliseconds 1400

$hwnd = [IntPtr]$word.ActiveWindow.Hwnd
[void][WinP]::ShowWindow($hwnd, 9)
[void][WinP]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 500
[void][WinP]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $Width, $Height, 0x0040)
Start-Sleep -Milliseconds 1000

$wshell = New-Object -ComObject WScript.Shell
$wshell.SendKeys($Keys)
Start-Sleep -Milliseconds 700
$wshell.SendKeys($GroupKey)
Start-Sleep -Milliseconds 1200

# Capture the WHOLE SCREEN, not the window: a ribbon popup is an owned
# top-level window that may extend past the application's own frame, and a
# window-rect capture would cut off exactly the part being studied.
$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen(0, 0, 0, 0, $bmp.Size)
$g.Dispose()
$path = Join-Path $OutDir "$Name.png"
$bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output "wrote $path"

$wshell.SendKeys("{ESC}")
Start-Sleep -Milliseconds 300
$wshell.SendKeys("{ESC}")
Start-Sleep -Milliseconds 300
$doc.Close(0)
$word.Quit(0)
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($word) | Out-Null
