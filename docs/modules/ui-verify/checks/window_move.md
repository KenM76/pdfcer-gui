# `ui-verify/checks/window_move`

`a_document_moves_between_windows` — four pdfcer-gui processes, all off the
desktop and driven only through `ScriptedPointer`, so it runs under
`--no-input`. They share one sandboxed executable, so they share one discovery
directory and see each other and nothing of the operator's windows.

## Steps

1. A opens a copy of `fixtures/four-pages.pdf`, B a copy of
   `fixtures/pure-k-square.pdf`. A must trace `window-peers n=1`.
2. A: View tab, `ribbon.item.view.move_to_window`. With one other window there
   is no picker. B must trace `window-move-received` naming A's file and then
   `doc-tabs open=2`; A must trace `window-move-sent` and `window-emptied`, and
   its process must end.
3. B: `ribbon.item.view.move_to_new_window` on its active tab (A's file). B
   must trace `window-torn-off pid=C`. The check owns C from here and kills it
   by that pid on every exit path.
4. D opens another copy of `four-pages.pdf`. B must trace `window-peers n=2`,
   and C's discovery file must list A's file name: the torn-off window opened
   the document it was started on.
5. B: `move_to_window` again. With two windows to choose from it must draw
   `window-pick.<D>`; clicking it, D must trace `window-move-received` naming
   B's own file, and B, now empty, must trace `window-emptied` and exit.

## Falsification

- The receiver dropping the path instead of opening it fails step 2 on the
  missing second tab.
- The sender skipping its tab's close after `ok` fails step 2: A neither
  empties nor exits.

## What it does not cover

Dragging a tab between windows, which is not built. The refusal of a document
with unsaved edits, and the picker's Cancel, are not driven.
