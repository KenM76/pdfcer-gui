<#
.SYNOPSIS
    Photograph pdfcer-gui's own ribbon at the same widths as Word's.

.DESCRIPTION
    `OPERATOR_REQUESTS.md` O31. `word-ribbon-study.ps1` photographs the thing
    we are learning from; this photographs the thing we are changing, at the
    **same widths**, so the two series can be laid side by side and the gap
    between them read off rather than argued about.

    ## ★ Why not `ui-verify`

    Because this is not a check. It asserts nothing, it can neither pass nor
    fail, and giving it a verdict would make it a check that always passes —
    the shape this project has twice had to delete. It is a camera. What it
    produces is evidence for a design decision, and the decision is made by a
    person looking at the pictures.

    ## ★★ It drives `target/release`, never the published build

    Filed 2026-08-24 after the suite was pointed at `OneDrive\pdfcer-gui1` and
    left the operator's own copy with a feature switched on that he had not
    asked for. A portable build keeps its state beside the exe; the
    development build keeps it in `%APPDATA%`, which is this harness's to
    disturb.
#>
[CmdletBinding()]
param(
    [string]$Exe = "D:\Dev\pdfcer-gui\target\release\pdfcer-gui.exe",
    [string]$Pdf = "D:\Dev\temp\pdfcer\SW41177.pdf",
    [string]$OutDir = "D:\Dev\pdfcer-gui\evidence\our-ribbon",
    [int[]]$Widths = @(1900, 1500, 1300, 900, 620, 460),
    [int]$Height = 900,
    # A ribbon.tab.* region to click before capturing, as a fraction of the
    # window width -- the tab strip is the top row, so a y of 45 px lands on
    # it at every width this study uses. Empty means "whatever tab opens".
    [double]$TabX = 0.0,
    [string]$TabName = ""
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$sig = @"
using System;
using System.Runtime.InteropServices;
public static class WinO {
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr lparam);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, IntPtr e);
    public static void Click() { mouse_event(0x0002, 0, 0, 0, IntPtr.Zero); mouse_event(0x0004, 0, 0, 0, IntPtr.Zero); }
    public delegate bool EnumProc(IntPtr h, IntPtr lparam);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    // ** The first VISIBLE top-level window of a process.
    //
    // `Process.MainWindowHandle` is not that, and the difference is not
    // academic: a winit application creates helper windows, and the one
    // MainWindowHandle picks can be invisible with a nonsensical rect. Sizing
    // it succeeds, reports the size back, and moves nothing on screen -- which
    // is a resize study that photographs the same width six times and says
    // nothing. `ui-verify`'s own win32 layer carries the same comment.
    public static IntPtr FirstVisible(uint want) {
        IntPtr found = IntPtr.Zero;
        EnumWindows(delegate(IntPtr h, IntPtr l) {
            uint pid; GetWindowThreadProcessId(h, out pid);
            if (pid == want && IsWindowVisible(h)) { found = h; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@
if (-not ("WinO" -as [type])) { Add-Type -TypeDefinition $sig }

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Output "our-ribbon-study: launching $Exe"
$proc = Start-Process -FilePath $Exe -ArgumentList $Pdf -PassThru
# Wait for a real main window rather than sleeping a guessed interval: a cold
# start that has to rasterise a 36-page CAD set takes appreciably longer than a
# warm one, and a photograph taken before the ribbon has laid out is a picture
# of nothing.
$deadline = (Get-Date).AddSeconds(40)
$hwnd = [IntPtr]::Zero
while ($hwnd -eq [IntPtr]::Zero) {
    if ((Get-Date) -gt $deadline) { throw "no visible window after 40 s" }
    Start-Sleep -Milliseconds 300
    $hwnd = [WinO]::FirstVisible([uint32]$proc.Id)
}
Start-Sleep -Milliseconds 2500
[void][WinO]::ShowWindow($hwnd, 9)
[void][WinO]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 700

if ($TabX -gt 0.0) {
    # Click the tab once, at the widest size, before the series begins: the
    # active tab persists across resizes, so clicking per width would be five
    # extra chances for a click to land somewhere the layout had moved.
    [void][WinO]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $Widths[0], $Height, 0x0040)
    Start-Sleep -Milliseconds 1200
    $r = New-Object WinO+RECT
    [void][WinO]::GetWindowRect($hwnd, [ref]$r)
    $cx = [int]($r.Left + ($r.Right - $r.Left) * $TabX)
    $cy = $r.Top + 45
    [void][WinO]::SetCursorPos($cx, $cy)
    Start-Sleep -Milliseconds 250
    [WinO]::Click()
    Start-Sleep -Milliseconds 900
    Write-Output "  clicked tab '$TabName' at ($cx, $cy)"
}

foreach ($w in $Widths) {
    [void][WinO]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $w, $Height, 0x0040)
    Start-Sleep -Milliseconds 1400
    $r = New-Object WinO+RECT
    [void][WinO]::GetWindowRect($hwnd, [ref]$r)
    $ww = $r.Right - $r.Left; $wh = $r.Bottom - $r.Top
    if ($ww -le 0 -or $wh -le 0) { throw "degenerate window rect at width $w" }
    $bmp = New-Object System.Drawing.Bitmap $ww, $wh
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
    $g.Dispose()
    $path = Join-Path $OutDir ("ribbon-{0:d4}.png" -f $w)
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    $c = New-Object WinO+RECT
    [void][WinO]::GetClientRect($hwnd, [ref]$c)
    Write-Output ("  asked {0,5}  window {1,4}x{2,-4}  client {3,5}  -> {4}" -f $w, $ww, $wh, ($c.Right - $c.Left), (Split-Path $path -Leaf))
}

Write-Output "our-ribbon-study: closing"
$proc.CloseMainWindow() | Out-Null
Start-Sleep -Milliseconds 1200
if (-not $proc.HasExited) { $proc.Kill() }
Write-Output "done"
