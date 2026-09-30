# pdfcer-gui — manual for an AI assistant

You are an AI assistant (Claude Code or similar) on Ken's Windows machine. Ken
has a PDF open in **pdfcer-gui**, his desktop PDF editor, and has asked you to
change it or read it. This file tells you what you can and cannot do, and the
safe way to do it.

## The one fact that governs everything

**You can drive the window Ken has open, but only through `pdfcer-remote.exe`,
and only with his permission.** It sits beside `pdfcer-gui.exe` in the same
folder. If it is not there, that build predates the live link: use the
hand-off procedure below instead.

A live-link edit happens *inside* the open document, with the same undo as his
own click, so there is no hand-off and nothing to reload. **Prefer it** whenever
the window is running. See *Driving the open window*.

Without the live link, **the open file is not live.** pdfcer-gui reads the
whole file into memory and closes it. It holds no lock, so you *can* write the
file, but:

- **pdfcer-gui does not notice.** It keeps showing its own copy. Nothing
  watches the file on disk, and there is no Reload. Opening the same path again
  just switches to the tab that is already open.
- **Its next Save overwrites your change.** Save builds the file from its own
  copy, plus Ken's edits, and replaces what is on disk. Your edit is lost and
  nothing reports it.

So an edit made from outside is a hand-off between you and Ken. It is never a
background change. A second `pdfcer-gui.exe` does not forward anything to the
first; it starts a separate program in its own window.

## Driving the open window: `pdfcer-remote`

```
pdfcer-remote list                          running windows: pid, active document
pdfcer-remote help                          the verb list
pdfcer-remote --purpose "<why>" <verb> [args...]
pdfcer-remote --pid <pid> <verb> ...        a particular window (default: newest)
```

Each call connects, introduces itself, sends one request, prints the reply and
disconnects. Exit code: `0` success, `1` the window refused or the verb failed
(`error <code>: <reason>` on stdout), `2` no window is running or bad usage.

**Permission.** Settings ▸ Remote control holds Ken's answer: *ask* (the
default), *always* or *never*. Under *ask*, your first call puts a bar across
the top of his window naming your `--client` and `--purpose`, and **your call
waits** (up to two minutes) until he answers. So pass a `--purpose` a person
can decide on, like `--purpose "apply the red markup on sheet 3"`. A refusal
holds for a minute; do not retry in a loop. The window's status bar shows while
a program is connected and keeps a log of every request.

**Verbs.** Every one acts on the **active** document:

| Verb | What it does |
|---|---|
| `state` | document, page `n/total`, zoom, `unsaved=`, `selected=`, `undo_depth=` |
| `commands` | every command: `id`, `enabled`/`disabled`/`hidden`, label |
| `run <id>` | run a command exactly as its ribbon button would |
| `page <n>` | go to page n (1-based) |
| `objects` | the current page's objects: index, `path`/`text`/`image`, bounds in PDF points |
| `select <i,j,…>` / `select none` | select objects on the current page by `objects` index |
| `undo [n]` / `redo [n]` | step the undo history |
| `history` | undo depth, and the kinds on top of the undo and redo stacks |
| `render [page] [dpi]` | render a page to a PNG; the reply is its path. **Read the image.** |
| `open <path>` | open a PDF in a new tab |

A `run`, `page`, `undo` or `open` replies with `state` as it stands *after* the
change, so you can check it in the same call.

**The loop that works:** `render` → look → `objects` → `select` → `run` →
`render` again to check what changed. Every change is one Undo for Ken. **Do not
save** (`run file.save`) unless he asked: leaving it unsaved lets him review and
undo your change.

**Things that bite:**

- `run` refuses a command the current mode hides (`error hidden`): editing
  commands need Edit mode, so send `run mode.edit` first. `commands` shows
  which state each command is in.
- A command that opens a dialog opens it on **his** screen. Prefer commands
  that act on the selection directly.
- `objects` indices are paint order on the current page and **change after a
  delete**. Re-read `objects` after every edit; never reuse an old index. A
  delete clears the selection.
- `page` is where the view is, and the view moves when Ken scrolls. Check
  `page=` in the reply before you `select`.

## Without the live link: changing a file Ken has open

1. **Ask Ken to Save (Ctrl+S) and close that tab** (File ▸ Close, or Ctrl+W).
   If he has unsaved edits, the tab shows a marker, and Close asks before it
   discards them. Do not write the file while it has unsaved edits in the GUI:
   one of the two sets of changes will be lost.
2. **Make the change with the `pdfcer` command-line tool**. See the next
   section. Prefer writing a new file (`--output something-edited.pdf`). Only
   overwrite the original when Ken asked for that. Writing over the original
   is safe mechanically: the tool writes a temporary file and renames it, so a
   failed write leaves the original as it was.
3. **Ask Ken to reopen it.** Use File ▸ Open or the recent-files list, or run
   `pdfcer-gui.exe "<path>"`, which opens it in a **new** window.
4. **Check what you did before you report it.** Re-read it with the same tool
   (`extract-text`, `list-annotations`, `list-fields`, `render-page`, …). Do
   not report the command's exit code as the result.

**Reading needs no hand-off.** You can inspect, list, extract text and render
pages at any time, even while the file is open. Remember that you are reading
what is **on disk**, which does not include Ken's unsaved edits. If he says
"what I'm looking at", ask him to save first.

## The command-line tool: `pdfcer`

`pdfcer` is the same engine that pdfcer-gui is built on, as a scriptable
command-line program. **It does not ship in the portable pdfcer-gui folder**:
that folder holds only the GUI, which has the engine built in. On this machine
the tool is here:

    D:\Dev\pdfcer\target\release\pdfcer.exe

It is not on PATH, so call it by that full path. `D:\Dev\pdfcer` is a source
tree that another session develops. Do not edit anything in it. If the exe is
missing, ask Ken rather than building it yourself.

**Discover before you guess.** The tool has about 190 subcommands, and every
one documents itself:

    pdfcer.exe --help               # every subcommand, one line each
    pdfcer.exe <subcommand> --help  # its options, units and refusals

Read the subcommand's `--help` before you use it. Pages are numbered from 1
unless the help says otherwise. Coordinates are PDF points (1/72 inch) with the
origin at the page's bottom-left. A few subcommands (`embed-font`,
`unembed-font`, `print`) are a dry run unless you pass the flag their help
names.

**Where to start, by task:**

| Task | Subcommands |
|---|---|
| Look at it | `inspect`, `render-page` (a PNG you can view), `extract-text`, `find-text`, `list-objects`, `object-list` |
| Pages | `extract-pages`, `insert-pages`, `delete-pages`, `reorder-pages`, `rotate`, `rotate-page`, `set-page-size`, `set-crop-box`, `merge`, `split` |
| Text on the page | `find-text` → `edit-text`, `format-text`, `add-text`, `reflow` |
| Comments and markup | `list-annotations`, `annotate`, `set-markup-note`, `add-reply`, `move-annotation`, `delete-annotation` |
| Forms | `list-fields`, `fill-field`, `reset-form`, `export-data`, `import-data`, `flatten` |
| Drawings (vectors) | `object-list` → `object-move`, `object-transform`, `object-delete`, `export-dxf` |
| ce dimensions (the ones pdfcer authors) | `dimension-list`, `dimension-add`, `dimension-label`, `group-set-scale` |
| Bookmarks | `list-outline`, `add-bookmark`, `rename-bookmark`, `move-bookmark`, `delete-bookmark` |
| Security | `encrypt`, `remove-encryption`, `redact-mark` then `redact-apply`, `sign`, `verify-signatures` |
| Export | `export-docx`, `export-xlsx`, `export-image`, `extract-tables` |

**Several edits in a row** each read and write the whole file. Chain them
through one `--output`, or through intermediate files in a temporary folder.
Do not run them in parallel on the same file.

## Worked example: "do what the markup says" on a drawing

Ken marks up a drawing (a box, an arrow and a note: *"Move Door D2 here and
replace the D2 area with wall. Door swings inward towards where D2 used to
be."*) and asks you to carry it out.

1. **Find the file.** The pdfcer-gui window title is the file name; the
   recent-files list is `userdata\recent.txt` beside the exe he runs (read
   it, never write it). Copy the PDF to a temporary folder and work there.
2. **Read the markup.** `list-annotations` gives each note's text and
   `rect`. The note is the instruction; the box and arrow say where.
3. **Mind the coordinate system.** `inspect` or the raw `/MediaBox` may not
   start at 0,0 (Revit sheets are centred: `[-1296 -864 1296 864]`). To crop
   a render around a point, pixel = (x − MediaBox.x0) × scale, and the row
   counts down from MediaBox.y1.
4. **Look before and after.** `render-page --scale 6` and crop around the
   area. View the PNG. This is the only reliable check of a drawing edit.
5. **Find the objects.** `object-list --page 1`, filtered to a bounding box
   around the area. A CAD door is typically a leaf rectangle (four lines),
   a dashed swing arc (many short lines), jamb boxes, and a tag (a closed
   path plus a `text` object).
6. **Reuse the real geometry where you can; draw where you must.**
   - To draw new lines, rectangles, polylines, circles or freehand strokes,
     add them as markup with `annotate --type line|polyline|square|circle|ink`
     and then burn just those into the page with
     `flatten-annotations --page N --index I` (repeatable; with no `--index`
     it burns every annotation on the page, Ken's own markup included).
   - Moving the drawing's own objects keeps its line weights and colours
     exactly. `object-transform --objects … --scale -1,1 --rotate 90 --pivot X,Y`
     mirrors across the 45° line through X,Y, which turns a door in a vertical
     wall into the same door in the horizontal wall at a corner, with the
     swing reversed.
   - `object-copy --clip f` then `object-paste --clip f` duplicates objects
     in place (they get new indices at the end of `object-list`); transform
     the copies.
   - `node-move` stretches a line (try `--node 0` and `1`, check the bbox).
   - `object-transform --scale SX,1 --pivot X,Y` shortens a wall fill from
     one end.
   - `object-delete` removes edges that no longer bound anything. Delete
     last, highest index first: every later index shifts down by one.
7. **Keep the drawing's conventions.** Read the legend: that sheet drew
   existing walls grey-filled and new walls as a black outline. New work must
   look like new work.
8. **Write compactly.** Every step appends a revision by default; after a
   dozen steps the file was 30× larger. Give the last step `--mode full`
   (only when `list-signatures` reports none).
9. **Prove nothing else moved.** Render the result and the original at the
   same scale and diff the images: the changed region must be only the area
   you meant to change.
10. **Deliver a new file** beside the original (`… - D2 moved.pdf`), leave the
    markup in place, and tell Ken to open it. Do not overwrite the file he
    has open.

## Things that bite

- **Signed documents.** Any change to a signed PDF can invalidate its
  signature. `list-signatures` first. If there is one, tell Ken before you
  write.
- **Redaction is two steps.** `redact-mark` only marks. `redact-apply` removes
  the content for good. Never apply unless he asked for the content to be
  removed.
- **Two kinds of dimension.** *ce dimensions* are measurement annotations
  pdfcer created; the `dimension-*` and `group-*` subcommands edit those.
  *pdf dimensions* are dimensions drawn by the CAD program that made the file:
  they are ordinary page vectors and text, and must not be changed unless Ken
  asks. Always say which kind you mean.
- **Drawings are large.** A CAD sheet can hold tens of thousands of objects.
  Filter `object-list` / `list-objects` output by page and area, and do not
  paste it whole into the conversation.
- **Undo.** pdfcer-gui's undo history is in memory and covers only edits made
  in the GUI. Your edits are not in it. Keep the original, or write to a new
  file, so there is always a way back.

## Driving a hidden pdfcer-gui (no mouse, no interruption)

When the `pdfcer` tool has no subcommand for the job, or the task needs the
app itself, run a private copy of pdfcer-gui off the desktop and script it.
Ken keeps working; nothing appears on his screen and nothing takes focus.

**1. Use a private copy of the program.** Never start Ken's own
`pdfcer-gui.exe`: a process writes settings, layout and recent files into the
`userdata` folder beside its exe, which would overwrite his. Copy the exe (and
the `models` folder, if you need OCR) into a temporary folder and run it from
there. Work on a copy of the PDF too.

**2. Start it with these environment variables** (read once, at start-up):

| Variable | Value | Effect |
|---|---|---|
| `PDFCER_DIAG` | `1` | Writes a trace to stderr. Required by everything below. Redirect stderr to a file. |
| `EGUI_SHELL_DIAG` | `1` | Adds the ribbon's lines to the trace (`ribbon-command-invoked …`). |
| `PDFCER_DIAG_VIEWPORT` | `-4200,-4200,1400,900` | Window position and size (x,y,w,h). This value puts it off every monitor, unfocused. |
| `PDFCER_DIAG_INVOKE` | `id1,id2,…` | Runs these ribbon commands, one per frame, in order. |
| `PDFCER_DIAG_POINTER` | a text-file path | A step file you append clicks and drags to (below). |
| `PDFCER_DIAG_SAVE_PATH` | a PDF path | Answers the Save As / Save a copy dialog with this path instead of showing it. |

Pass the PDF path as the only argument. Other `PDFCER_DIAG_*` variables
answer other file pickers the same way (`OPEN_PATH`, `INSERT_PATH`,
`IMAGE_PATH`, `CERTIFICATE_PATH`, `MERGE_SOURCES`, `FORM_DATA_PATH`,
`ATTACH_PATH`, …). Grep the pdfcer-gui source for the exact name before
relying on one.

**Command ids** are the ribbon's: `file.save`, `file.save_as`,
`file.save_copy`, `file.close`, `file.sign`, and so on. The full list is in
`D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\shell\ron\built_in.ron`. Many
commands need Edit mode first, so put the mode switch before them in the list.

**3. Click and drag through the step file.** Append one line per step, each
starting with a sequence number that goes up by one:

    1 click 640 44
    2 click 300 400 btn=r
    3 dclick 300 400
    4 drag 100 200 400 260 steps=12
    5 wheel 600 400 -120
    6 key A mods=ctrl
    7 type http://example.test/
    8 shot
    9 gone

Coordinates are the window's own logical points, not screen pixels. Read them
from the trace: every clickable region is announced as
`ui-rect name=ribbon.item.file.save rect=[[x y] - [x y]]`; click its centre.
The app answers each step with a `diag-pointer seq=N` line, or
`diag-pointer-refused` for a line it cannot parse. Wait for that answer before
writing the next step. `shot` writes the window's own image and names the file
in its answer, so you can see what you did although the window is off-screen.
`gone` takes the pointer off the window. `key` presses and releases one key
(egui's name: `A`, `Enter`, `Tab`, `Escape`; `mods=ctrl+shift`). `type` sends
the rest of the line as text to whichever field has focus, so click the field
first. A step in a pop-up window names it with `vp=` (the viewport the region
was announced in).

**4. Check the trace, not the exit code.** Features record what they did
(for example `evidence-applied certs=2 …` or `evidence-refused reason=…`).
Then re-read the saved file with the `pdfcer` tool.

**5. End only your own process, by its process id.** Never by name: that
closes every pdfcer-gui Ken has open.

**6. Hand the result over** as in the safe procedure above: a new file, or
Ken closes his tab before you replace his.

The developers' test harness does exactly this; its checks are working
examples: `D:\Dev\pdfcer-gui\tools\ui-verify\src\checks\evidence_scripted.rs`
and `purge_passwords_scripted.rs`.

## Things not to do

- **Never close or kill pdfcer-gui yourself** unless it is the hidden copy you
  started. Ken's window is his daily reader and may hold unsaved work. End
  processes by process id, never by name.
- **Never start Ken's own copy of the exe with `PDFCER_DIAG_*` set**, and do
  not edit anything beside it (`userdata\`, `models\`). Copy it first.
- **Never write the file Ken has open while he has unsaved edits.** One of the
  two sets of changes will be lost.

## Helping Ken do it himself

Sometimes the edit is easier for Ken to make in the GUI, for example moving
something by eye. Tell him where the command is, not how the code works:

- `MANUAL.md` beside `pdfcer-gui.exe` is the user manual. Its section *Every
  keyboard shortcut* lists the keys, and it says where each feature lives on
  the ribbon.
- In the app, **File ▸ Keyboard shortcuts** shows the same list.
- The app has three modes (Read, Review, Edit), switched from the selector at
  the top. Most editing commands appear only in **Edit**. If Ken cannot find a
  command, it is usually because he is in the wrong mode.

## Why this file exists

A new AI session was asked to edit a file Ken had open in pdfcer-gui, and it
had no way to learn any of the above. If you find something here that is
wrong, tell Ken, so the file can be corrected.
