<#
.SYNOPSIS
    Photograph Microsoft Word's ribbon at a series of window widths.

.DESCRIPTION
    `OPERATOR_REQUESTS.md` O31 — *"if you can learn how word handles when to
    have text labels, organization on two rows for some commands, and how it
    handles narrowing the window."*

    ## Why a camera and not an API

    Word's ribbon layout rules are **not in its object model**. `CommandBars`
    is the legacy 2003 toolbar surface and reports nothing about the ribbon;
    the ribbon itself is declared in RibbonX XML compiled into the product, and
    its scaling behaviour (which group collapses first, when a label is
    dropped, when two rows become three) is implemented inside the Office UI
    framework and exposed nowhere.

    So the only instrument that can answer the question is the one this project
    already treats as authoritative for any layout question: **a screenshot**.
    The script sets a width, waits for the ribbon to re-lay-out, and captures
    the window. Reading the resulting series is what produces the rules.

    ## What it does NOT do

    It does not type into the document, open a file, or change any Word
    setting. It creates one blank document, resizes the window, photographs it,
    and closes without saving. The only lasting effect on the machine is the
    PNG files it writes.

.PARAMETER OutDir
    Where the PNGs go. Created if absent.

.PARAMETER Widths
    Client widths to photograph, in physical pixels, largest first. Largest
    first matters: Word re-lays-out incrementally and a series that grows would
    photograph the *recovery* path rather than the collapse path, and the two
    are not guaranteed to be symmetric.

.PARAMETER Tab
    Which ribbon tab to select before capturing. "Home" is the densest and the
    one whose groups carry the most varied item sizes.
#>
[CmdletBinding()]
param(
    [string]$OutDir = "D:\Dev\pdfcer-gui\evidence\word-ribbon",
    [int[]]$Widths = @(1900, 1700, 1500, 1300, 1150, 1000, 900, 800, 700, 620, 540, 460),
    [string]$Tab = "Home",
    [int]$Height = 900
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

# --- Win32: move/size a window, and read back what we actually got ----------
#
# SetWindowPos rather than the Word object model's Application.Width, because
# that property is in POINTS and is subject to Word's own minimum-size logic;
# we want to know what the ribbon does at a given number of physical pixels and
# we want to hear about it when the window refuses to be that small.
$sig = @"
using System;
using System.Runtime.InteropServices;
public static class Win {
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
"@
if (-not ("Win" -as [type])) { Add-Type -TypeDefinition $sig }

function Capture-Window {
    param([IntPtr]$Handle, [string]$Path)
    $r = New-Object Win+RECT
    [void][Win]::GetWindowRect($Handle, [ref]$r)
    $w = $r.Right - $r.Left
    $h = $r.Bottom - $r.Top
    if ($w -le 0 -or $h -le 0) { throw "the window reported a degenerate rect: ${w}x${h}" }
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    # CopyFromScreen, not PrintWindow: the ribbon is composited and PrintWindow
    # returns a blank or partial surface for it on several Office builds.
    # The window is raised first, so screen-copy is the honest picture.
    $g.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
    $g.Dispose()
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    return "${w}x${h}"
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Output "word-ribbon-study: starting Word"
$word = New-Object -ComObject Word.Application
$word.Visible = $true
$doc = $word.Documents.Add()
Start-Sleep -Milliseconds 1200

# Make sure the ribbon is expanded rather than collapsed-to-tabs, and that the
# tab we want is the active one. Both are Word settings that persist, so both
# are read first and restored at the end.
$wasMinimised = $word.ActiveWindow.Application.CommandBars.Item("Ribbon").Height -lt 100
try { $word.ShowWindowsInTaskbar = $true } catch {}

$hwnd = [IntPtr]$word.ActiveWindow.Hwnd
[void][Win]::ShowWindow($hwnd, 9)   # SW_RESTORE, so a maximised Word can be sized
[void][Win]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 600

# Select the requested tab through the keyboard, because there is no supported
# object-model call for it. Alt+H is Home; the KeyTip letters are stable across
# every localised build this machine will ever see, being tied to the English
# UI it is installed with.
$tabKey = switch ($Tab) {
    "Home"   { "%h" }
    "Insert" { "%n" }
    "Layout" { "%p" }
    "View"   { "%w" }
    default  { "%h" }
}
$wshell = New-Object -ComObject WScript.Shell
$wshell.SendKeys($tabKey)
Start-Sleep -Milliseconds 900
# Escape any KeyTip overlay the Alt press left on screen, so it is not in the
# photograph.
$wshell.SendKeys("{ESC}")
Start-Sleep -Milliseconds 500

$results = @()
foreach ($w in $Widths) {
    # SWP_NOMOVE=0x2 is deliberately NOT set: the window is pinned to the top
    # left so every photograph has the same origin and the series can be
    # compared by eye without registering them first.
    [void][Win]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $w, $Height, 0x0040)
    Start-Sleep -Milliseconds 900
    $path = Join-Path $OutDir ("ribbon-{0:d4}.png" -f $w)
    $got = Capture-Window -Handle $hwnd -Path $path
    $client = New-Object Win+RECT
    [void][Win]::GetClientRect($hwnd, [ref]$client)
    $cw = $client.Right - $client.Left
    Write-Output ("  asked {0,5}  window {1,-10}  client {2,5}  -> {3}" -f $w, $got, $cw, (Split-Path $path -Leaf))
    $results += [pscustomobject]@{ Asked = $w; Client = $cw; File = $path }
}

Write-Output "word-ribbon-study: closing without saving"
$doc.Close(0)
$word.Quit(0)
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($word) | Out-Null

Write-Output ""
Write-Output "Captured $($results.Count) width(s) into $OutDir"
