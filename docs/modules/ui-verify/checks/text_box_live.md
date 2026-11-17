# `ui-verify/checks/text_box_live`

`a_text_box_is_typed_on_the_page`, on `fixtures/annots-align.pdf`, in Review
with Markup ▸ Text box armed. Scripted pointer, off the desktop.

PASS needs, in order:

1. A drag from (230, 140) to (380, 80) traces `text-box-draft-begun` and no
   `text-annot-open`.
2. Typing `QKMYW` traces `text-box-preview baked=1 … lines=1 chars=5` with
   `px` not `0x0`: the engine baked the appearance and it was painted.
3. Enter and `ZJ` trace `lines=2`, and no `add-text-annot` yet.
4. Ctrl+Enter traces `add-text-annot`.
5. A second box is dragged and `XQKEPT` typed; dragging a third traces
   `add-text-annot` and a new `text-box-draft-begun`.
6. `XQESCD` is typed in the third and Escape traces `add-text-annot`.
7. After Ctrl+S the file holds three more `/Subtype /FreeText` than the
   fixture and all three words.

## Falsified

- The build before the feature opens the dialog → FAIL at step 1.

## What it does not prove

- That the pixels painted are the pixels saved; `text-box-live.png` is kept
  for a reading by eye.
- Caret placement by click, selection by sweep, the grown rectangle, the
  clipped-width sentence, or the commit raised by putting the tool down.
- A rotated page, which keeps the dialog.
