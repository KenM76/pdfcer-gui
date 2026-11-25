# `each_tab_keeps_its_own_position`

**Defect it catches:** scrolling one open document moves the others, so
coming back to a tab shows a different place from the one the operator left
(O462).

## What it drives

The app is launched off-screen (`-4200,-4200,1400,900`, scripted pointer,
`--no-input` safe) on `fixtures/four-pages.pdf`, with
`PDFCER_DIAG_OPEN_PATH` answering the Open dialog with
`fixtures/drawing-sheets.pdf`.

1. Wheel six lines in the first document's `canvas-viewport`.
2. Ctrl+O opens the second document. Wheel thirty lines in it.
3. Click `doc-tab.0`.

## Oracle

The last `canvas-pos at=` after the switch is within 1 px (screen pixels from the page corner) of where the first
document was left. A `document-activate slot=0` line must follow the click.

SKIP when the wheel did not move the first document, or when the switch was
not traced.

## Not covered

Zoom and the current page across a switch, and the reordering of tabs.
`tab_reorder` covers the reordering.
