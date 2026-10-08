# `ui-verify/checks/cross_window_paste`

`a_selection_copied_in_one_window_pastes_as_objects_in_another` — two
pdfcer-gui processes at once, both off the desktop and driven only through
`ScriptedPointer`, so it runs under `--no-input`. Both share one
`PDFCER_DIAG_CLIPBOARD_DIR`, so the clipboard they exchange through is a folder
and the operator's own clipboard is untouched.

## Steps

1. The capture folder is emptied, so an earlier run cannot pass for this one.
2. The source window opens a copy of `fixtures/pure-k-square.pdf`, the target
   a copy of `fixtures/four-pages.pdf`; both stay running.
3. Source: `ribbon.mode.edit`, `ribbon.tab.edit`, `ribbon.item.edit.select_all`,
   `ribbon.item.edit.copy`. Its trace must carry `clipboard-copy` and
   `clipboard-objectclip bytes=B`, and the folder must hold a
   `*-pdfcer_gui_ObjectClip.bin` of exactly `B` bytes.
4. Target: `ribbon.mode.edit`, `ribbon.tab.edit`, `ribbon.item.edit.paste`. Its
   trace must carry `clip-adopted` and `paste-objects-applied`, and the
   `objects=` of the copy, the `objects=` of the adoption and the `pasted=` of
   the apply must agree.

The agreement is what a picture paste cannot produce: an image pasted from the
OS clipboard writes no `paste-objects-applied` line at all.

## Falsification

- `clipshared::adopt` returning `None` at once fails it on the missing
  `clip-adopted` line; the target pastes nothing of the source's.
- `clipimage::publish` leaving out the `OBJECT_CLIP_FORMAT` entry fails it on
  the missing capture file before the target is driven.

## What it does not cover

The real clipboard across two processes: the private format is registered and
read through `native_clipboard`, whose round trip the capture folder replaces.
Dragging a selection between windows is not a clipboard act and is not here.
