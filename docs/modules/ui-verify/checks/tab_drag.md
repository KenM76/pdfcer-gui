# `ui-verify/checks/tab_drag`

`a_tab_dragged_out_moves_its_document` — four pdfcer-gui processes off the
desktop, driven only through `ScriptedPointer`, so it runs under `--no-input`.
A sits at `-4200,-4200` and B 1,400 pixels below it, so neither covers the
other and a drop on B's centre is unambiguous. Window launching and the
await/exit helpers are `window_move`'s.

## Steps

1. A opens a copy of `fixtures/four-pages.pdf`, B a copy of
   `fixtures/pure-k-square.pdf`; each must see the other (`window-peers n=1`).
2. `End` in A. A's canvas must reach page index 3 and still show it after
   settling.
3. A's `doc-tab.0` is dragged to B's centre, computed from both windows'
   `window-rect` lines and A's `window-inner ppp=`. A must trace
   `window-tab-dropped … onto=<B>`; B must trace `window-move-received` naming
   A's file with `page=3`, its canvas must reach page 3 and stay there, and it
   must show two tabs; A must empty and exit.
4. B's `doc-tab.1` is dragged 400 points straight down, onto B's own canvas. B
   must trace `window-torn-off pid=T … page=3`; the check owns T from here and
   kills it by pid on every exit path. T's discovery file must list A's file
   within fifteen seconds.
5. C starts on `four-pages.pdf` with `--page 4`; its canvas must reach page
   index 3 and stay there.

"Stay there" is asserted on the last canvas line after settling, not on any
line: a page reached and then lost to the open's own placement passes an
any-line check.

## Falsification

- The receiver ignoring the page it is sent fails step 3 on B's canvas.
- `DragOut` handled as a no-op fails step 3 on the missing
  `window-tab-dropped` line.

## What it does not cover

A drop on a window that a third window partly covers; a drop refused because
the document has unsaved edits; the lifted label drawn at the pointer, which
only a screenshot shows.
