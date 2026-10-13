# `ui-verify/checks/window_carry`

`a_document_moves_with_its_edits_and_password` — two pdfcer-gui processes and
one torn-off child, all off the desktop and driven only through
`ScriptedPointer`, so it runs under `--no-input`. It shares `window_move`'s
launcher, so every window runs the same sandboxed executable and sees only the
others.

## Steps

1. A opens a copy of `fixtures/encrypted-aes-128.pdf`, answers the prompt with
   `userpw`, and rotates the page from the left rail (`rotate-pages`). The
   rotation is unsaved. The copy's length is noted.
2. B opens a copy of `fixtures/four-pages.pdf`; A must see one peer.
3. A: View tab, `ribbon.item.view.move_to_window`. B must trace
   `window-move-received` with `carried` non-zero and `key=true`, then
   `window-move-opened ok=true unsaved=true`; it must declare no
   `password.field` and must show `doc-tabs open=2`. A must trace
   `window-move-sent` and `window-emptied`, and exit. The original file must
   not have changed.
4. B: Ctrl+S. `save-in-place outcome=ok`; the bytes appended past the original
   length must hold `/Rotate`, and the file must still hold `/Encrypt`.
5. B rotates again (unsaved) and presses `ribbon.item.view.move_to_new_window`.
   `window-torn-off` must say `carried=true` and name the child, which the
   check owns from here and kills by pid on every path. B must trace
   `window-move-sent to=<child>` and `doc-tabs open=1`, and the child's
   discovery file must list the document.

## Falsification

- Opening the arrival without its key (`PdfcerApp::open_arrival` passing
  `None`) fails step 3: `window-move-opened ok=false unsaved=false why="the document
  did not open there"`.
- Opening the arrival from its file instead of the carry copy (`source` as
  `None` in `parcel`'s `PdfcerApp::take`) fails step 4: 1872 -> 1872 bytes,
  appended `/Rotate` false.

## What it does not reach

The undo history's absence in the receiver (G184) is a disclosure, read from
the code rather than asserted here.
