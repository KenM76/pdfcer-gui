# `ui-verify/checks/copy_as_vector_scripted`

`copy_as_vector_without_the_mouse` — Edit ▸ Clipboard ▸ Copy as vector on a
window off the desktop, driven only through `ScriptedPointer`, so it runs
under `--no-input`. The payload goes to a folder through
`PDFCER_DIAG_CLIPBOARD_DIR`, so the operator's clipboard is untouched.

## Steps

1. The capture folder is emptied, so an earlier run cannot pass for this one.
2. `fixtures/pure-k-square.pdf`. Clicks: `ribbon.mode.edit`,
   `ribbon.tab.edit`, the collapsed `ribbon.group.edit.clipboard` when the
   band is narrow, then `ribbon.item.edit.copy_as_vector`.
3. The trace must carry `clipboard-copy-out selection=false
   formats=image/svg+xml,CF_ENHMETAFILE,PNG,CF_DIBV5` and a
   `clipboard-captured` line, and no refusal.
4. The folder must hold exactly four files, in that order, each well formed:
   the SVG names `<svg`; the EMF starts with record type 1 and carries ` EMF`
   at offset 40; the PNG has its signature; the DIB starts with
   `BITMAPV5HEADER`'s size, 124.

## Falsification

- Placing the PNG ahead of the EMF in `ORDER` fails it on the traced order.
- Writing the EMF slot with the PNG's bytes fails it on the second file.

## What it does not cover

The real clipboard: `copy_as_vector_places_the_measured_order` reads the
placement back from Windows, needs OS input, and replaces the clipboard, so
it runs only when the machine is free. Copying a selection is not driven.
