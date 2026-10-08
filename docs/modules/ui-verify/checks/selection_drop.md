# `ui-verify/checks/selection_drop`

`a_selection_dropped_on_another_window_is_copied_there` — two pdfcer-gui
processes off the desktop, driven only through `ScriptedPointer`, so it runs
under `--no-input`. A sits at `-4200,-4200` and B 1,400 pixels below it
(`tab_drag::BELOW`). Both write the clipboard to a capture folder
(`PDFCER_DIAG_CLIPBOARD_DIR`), so the operator's clipboard is never touched.

## Steps

1. A opens a copy of `fixtures/pure-k-square.pdf`, B a copy of
   `fixtures/four-pages.pdf`; each must see the other (`window-peers n=1`).
2. B switches to Edit mode. A switches to Edit mode and the Edit tab and
   selects all.
3. A's selection is dragged from the square's centre, page point (100, 100),
   to B's centre in A's points (`tab_drag::other_centre`). A must trace
   `window-selection-dropped onto=<B>` and `clipboard-copy objects=N`; B must
   trace `window-paste-received`, `clip-adopted objects=N` and
   `paste-objects-applied pasted=N`. A must trace no `move-*` event and no
   `clipboard-cut`.
4. The same drag with Shift held: the same lines in both windows except
   `clip-adopted`, no `move-*` in A, and A must trace `clipboard-cut`. The
   capture folder leaves the system clipboard's sequence number unchanged, so
   B adopts A's clip on the first drop only and pastes its held copy after
   that; the object counts must still agree.

Every line is looked for after a mark taken before its drag, so step 4 cannot
pass on step 3's lines.

## Falsification

- `selection_released` answering `false` always fails step 3 on the missing
  `window-selection-dropped` line.
- The receiver ignoring a `paste` request fails step 3 on the missing
  `window-paste-received` line.

## What it does not cover

Whether the pasted objects land under the release point (the counts are
asserted, not the position); a drop on a window in a mode that may not paste
content; a drop refused by the receiver.
