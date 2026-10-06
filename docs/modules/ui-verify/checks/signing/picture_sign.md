# `ui-verify/checks/signing/picture_sign`

`a_picture_signature_lands_in_its_box` — O287's picture signature, end to end.

## What it drives

Off the desktop, scripted pointer only, on a copy of
`fixtures/esign-three-boxes.pdf`.

1. Write a 120 x 40 RGB PNG to the run folder: white paper with one dark
   blue-black bar, so the clear-white key has paper to remove.
2. Read the first tagged box (`form.sign-box`) and photograph the window.
3. Click the box, then `handsign.tab-picture`, then `handsign.choose-picture`;
   `PDFCER_DIAG_IMAGE_PATH` answers the picker with the PNG.
4. Press `handsign.place`; photograph; Ctrl+Z; photograph.
5. File ▸ Save a copy to `PDFCER_DIAG_SAVE_PATH`.

`open` and `undo_and_save` are shared with `placement`.

## Oracles

- **Chosen.** `hand-sign-picture-chosen read=1 clear_white=1 offered=1`:
  the file imported, and clearing white was offered and ticked for an RGB
  picture with no transparency of its own.
- **Placement.** `hand-sign-placed via=picture tagged=1 clear_white=1`.
- **Pixels.** Zero ink before, at least 20 after, zero after Ctrl+Z.
- **File.** The copy carries a `/pdfc_HandSig` tag, a `/Mask` (the colour
  key), and no `/ByteRange`.

## Falsification

Never keeping the operator's *make white clear* (the picture tab storing
`clear_white: false`) fails **Chosen**, **Placement** and **File**: the trace
reads `clear_white=0` and the saved copy holds no `/Mask`.
