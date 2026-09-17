<#
.SYNOPSIS
    Photograph Acrobat's form-field widgets, open and closed.

.DESCRIPTION
    O209 — *"You need to compare how your form fields look and act vs how they
    look and act when opened in Acrobat."*

    Acrobat's field-editing surface is drawn by `AcroForm.api` and is exposed
    by no API: how a closed combo box is decorated, what the open list looks
    like, whether a list box shows its rows before it is clicked, and where the
    scroll bar appears are all things only a photograph can answer. The
    companion `tools/acrobat-form-strings.py` lifts Acrobat's *labels*; this
    lifts its *pixels*.

    The script opens one document, positions the window at a fixed origin so a
    series can be compared without registration, and photographs it at each
    requested click point. It never types and never saves.

.PARAMETER Pdf
    The document to open. Copied to a scratch path first, so an accidental
    Acrobat save cannot touch a committed fixture.

.PARAMETER OutDir
    Where the PNGs go.

.PARAMETER Clicks
    Screen points to click before each capture, as "x,y" strings. Each click is
    followed by a capture, so the series reads as gesture, picture, gesture,
    picture. An entry of "-" captures without clicking, an entry of "esc"
    presses Escape, and an entry of "key:<SendKeys>" presses whatever follows
    the colon -- "key:%{DOWN}" for Alt+Down, "key:{F4}", "key:abc".

    Escape is in the vocabulary because Acrobat regularly draws a blank white
    rectangle over the page — the operator's own instruction, verbatim: *"When
    acrobat puts up that blank white rectangle you have to press escape."* A
    series that does not clear it photographs the rectangle rather than the
    form, and every field below the first is hidden.

.PARAMETER Width, Height
    The window size, in physical pixels.

.NOTES
    Acrobat is single-instance. If one is already open the launched process
    exits immediately after handing its argument to the running copy, so the
    script adopts that window instead -- and then never closes it, because it
    may be holding a document the operator opened.
#>
[CmdletBinding()]
param(
    [string]$Pdf = "D:\Dev\pdfcer-gui\fixtures\all-field-kinds.pdf",
    [string]$OutDir = "D:\Dev\pdfcer-gui\target\scratch\formcmp\acrobat",
    [string[]]$Clicks = @("-"),
    [int]$Width = 1100,
    [int]$Height = 1300,
    [switch]$KeepOpen
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$sig = @"
using System;
using System.Runtime.InteropServices;
public static class AWin {
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, IntPtr e);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
"@
if (-not ("AWin" -as [type])) { Add-Type -TypeDefinition $sig }

function Capture-Window {
    param([IntPtr]$Handle, [string]$Path)
    $r = New-Object AWin+RECT
    [void][AWin]::GetWindowRect($Handle, [ref]$r)
    $w = $r.Right - $r.Left
    $h = $r.Bottom - $r.Top
    if ($w -le 0 -or $h -le 0) { throw "degenerate window rect ${w}x${h}" }
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    # CopyFromScreen, not PrintWindow: an open drop-down is a SEPARATE
    # top-level window layered over the document, and PrintWindow on the
    # document window would photograph the surface without it -- which is
    # precisely the thing under study.
    $g.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
    $g.Dispose()
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    return @{ W = $w; H = $h; X = $r.Left; Y = $r.Top }
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# Work on a copy. Acrobat writes to a form document on close without always
# asking, and the fixture is committed.
$scratch = Join-Path $OutDir ("open-" + (Split-Path $Pdf -Leaf))
Copy-Item -Force $Pdf $scratch

$acro = "C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe"
if (-not (Test-Path $acro)) { throw "Acrobat not found at $acro" }

Write-Output "acrobat-form-study: opening $scratch"
$adopted = $false
$proc = Start-Process -FilePath $acro -ArgumentList "`"$scratch`"" -PassThru
# Acrobat's first window is a splash; poll for a main window rather than
# guessing a sleep long enough for a cold start.
$deadline = 40
$hwnd = [IntPtr]::Zero
while ($deadline -gt 0) {
    Start-Sleep -Milliseconds 500
    $proc.Refresh()
    if ($proc.MainWindowHandle -ne [IntPtr]::Zero) { $hwnd = $proc.MainWindowHandle; break }
    $deadline--
}
if ($hwnd -eq [IntPtr]::Zero) {
    # Acrobat is single-instance. When one is already running, the process
    # this script started hands the file over and exits, so its own
    # MainWindowHandle never becomes non-zero and the document is on screen in
    # a process we did not launch. Adopt that window rather than failing --
    # and remember we do not own it, so the close step leaves it alone.
    $owner = Get-Process -Name Acrobat -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
        Select-Object -First 1
    if ($null -ne $owner) {
        $hwnd = $owner.MainWindowHandle
        $proc = $owner
        $adopted = $true
        Write-Output ("acrobat-form-study: adopted existing Acrobat pid {0}" -f $owner.Id)
    }
}
if ($hwnd -eq [IntPtr]::Zero) { throw "Acrobat never produced a main window" }
Start-Sleep -Seconds 4

[void][AWin]::ShowWindow($hwnd, 9)
[void][AWin]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $Width, $Height, 0x0040)
Start-Sleep -Milliseconds 1200
[void][AWin]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 800

$i = 0
foreach ($c in $Clicks) {
    $i++
    if ($c -eq "esc" -or $c.StartsWith("key:")) {
        # A synthesized press-and-release on a combo box's arrow cannot
        # photograph the open list: the popup takes the mouse capture and
        # Acrobat stops painting, so the frame is the white slab the operator
        # already reported. The keyboard opens the same list without the
        # capture, so "key:%{DOWN}" is the route that produces a picture.
        [void][AWin]::SetForegroundWindow($hwnd)
        Start-Sleep -Milliseconds 200
        $wsh = New-Object -ComObject WScript.Shell
        $wsh.SendKeys($(if ($c -eq "esc") { "{ESC}" } else { $c.Substring(4) }))
        Start-Sleep -Milliseconds 900
    }
    elseif ($c -ne "-") {
        # Foreground before every click, not only at the start. Acrobat leaves
        # the page half-painted -- a black or white slab over the form -- when a
        # click arrives while another window owns the foreground, and a series
        # that does not re-assert it photographs the slab instead of the field.
        [void][AWin]::SetForegroundWindow($hwnd)
        Start-Sleep -Milliseconds 200
        $parts = $c.Split(",")
        $x = [int]$parts[0]; $y = [int]$parts[1]
        [void][AWin]::SetCursorPos($x, $y)
        Start-Sleep -Milliseconds 250
        [AWin]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)  # LEFTDOWN
        Start-Sleep -Milliseconds 80
        [AWin]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)  # LEFTUP
        Start-Sleep -Milliseconds 1200
    }
    $path = Join-Path $OutDir ("acro-{0:d2}.png" -f $i)
    $got = Capture-Window -Handle $hwnd -Path $path
    Write-Output ("  click {0,-12} -> {1}  (window {2}x{3} at {4},{5})" -f $c, (Split-Path $path -Leaf), $got.W, $got.H, $got.X, $got.Y)
}

if (-not $KeepOpen -and -not $adopted) {
    Write-Output "acrobat-form-study: closing"
    try { $proc.CloseMainWindow() | Out-Null } catch {}
    Start-Sleep -Seconds 3
    if (-not $proc.HasExited) { try { $proc.Kill() } catch {} }
}
Write-Output "done -> $OutDir"
