# Driving a private, hidden pdfcer-gui

Use this when no CLI subcommand does the job and the user's window should not be
touched. The copy runs off every monitor, never takes focus, and is scripted
through environment variables (diag-env.md) and a pointer step file
(pointer-script.md).

## 1. Make a private copy

- Copy `pdfcer-gui.exe` (and `models\` only if OCR is needed) into a temporary
  folder. Copy the PDF there too.
- **Never run the user's own exe this way.** A process keeps its settings,
  layout and recent list in `userdata\` beside its exe, so it would overwrite
  the user's. The file-picker overrides (`PDFCER_DIAG_*_PATH` and others) act whenever they are
  set, even without `PDFCER_DIAG`.
- Optional: to drive the copy with `pdfcer-remote` instead of the pointer, write
  `remote_control = always` into `<copy>\userdata\preferences.txt` before launch.
  Then always pass `--pid`: the copy becomes the newest window, and calls
  without `--pid` would go to it instead of the user's, or the reverse.

## 2. Launch (PowerShell)

```powershell
$w = "<temp folder>"
$env:PDFCER_DIAG = '1'
$env:PDFCER_DIAG_VIEWPORT = '-4200,-4200,1400,900'   # off-screen, unfocused
$env:PDFCER_DIAG_POINTER = "$w\steps.txt"
$p = Start-Process "$w\pdfcer-gui.exe" -ArgumentList "`"$w\doc.pdf`"" `
     -RedirectStandardError "$w\trace.txt" -PassThru
# ... drive it ...
Stop-Process -Id $p.Id
```

- Variables are read at start-up. Clear them from the shell afterwards.
- The release exe is a GUI-subsystem program: the trace exists only when stderr
  is redirected to a file. `2>&1` into a pipe shows nothing.
- In Git Bash, `$!` is not the Windows PID. Use `Start-Process -PassThru`, or
  read the PID from `pdfcer-remote list`.
- Create the step file empty before launch, then append lines to it.

## 3. Find coordinates in the trace

Every line starts with `pdfcer-diag `.

- `start argv1=Some("<path>")` (Debug-quoted) and `window-handle present=true` appear at start-up.
- `ui-rect name=<region> rect=[[x0 y0] - [x1 y1]][ viewport=<id>]` is printed
  when a region appears or moves. The coordinates are logical points of that viewport. Click the centre.
  `ui-rect-gone name=` means it was retired. `ui-rect-clipped name= rect= clip= shown= floor=0.60`
  means less than 60% of it is visible.
- Region names: `ribbon.item.<command id>`, `ribbon.tab.<tab>`,
  `ribbon.mode.<read|review|edit>`, `ribbon.group.<...>`, `ribbon.tabs.overflow`,
  `dock.tab.<panel>`, `rail.navigate.<id>`, and others. Grep the trace.
- Rects change when the layout changes (window size, dock width). Re-read them
  after anything that could move the layout.

## 4. Act

- Commands: `PDFCER_DIAG_INVOKE=mode.edit,file.save_copy` runs ids in order,
  one per frame, after start-up. Put the mode switch first.
- Dialog pickers: set the matching `PDFCER_DIAG_*_PATH` so no OS dialog opens.
- Clicks, drags, keys, text and screenshots: append to the step file.
- With `EGUI_SHELL_DIAG=1`, the trace also gets `egui-shell-diag ribbon-command-invoked ...` lines.

## 5. Verify

- Read the trace: features record what they did (`...-applied`, `...-refused reason=`).
- `N shot` saves the window as a PPM. Look at it.
- Re-read the saved file with `pdfcer.exe`, or render it.
- A pointer acknowledgement means the events were delivered, not that they did
  what you intended.

## 6. Finish

- End only your process, by its PID. Never by image name.
- Hand over the result as a new file (file-safety.md).
