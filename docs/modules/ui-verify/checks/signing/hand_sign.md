# `ui-verify/checks/signing/hand_sign`

`a_drawn_signature_lands_in_its_box` — O269's drawn signature, end to end.

## What it drives

Off the desktop, scripted pointer only, on a copy of
`fixtures/esign-three-boxes.pdf` (three empty `/FT /Sig` boxes; its
`PROVENANCE.py` says why each exists).

1. Read the first tagged box (`form.sign-box`) and photograph the window.
2. Click the box; `hand-sign-opened` must follow.
3. Drag two strokes in `handsign.pad`, press `handsign.place`.
4. Photograph; Ctrl+Z; photograph; Ctrl+Shift+Z.
5. File ▸ Save a copy to `PDFCER_DIAG_SAVE_PATH`.

## Oracles

- **Layout.** `handsign.pad`, `handsign.place` and `handsign.digital-id` lie
  inside `handsign.body`. The dialog is an immediate viewport, which eframe
  never screenshots, so regions are the only layout witness for it.
- **Placement.** `hand-sign-placed strokes>=2 signed=1`.
- **Pixels.** Ink-coloured pixels (blue well above red and green) in the box
  and the band one box-height above it: zero before (the control: the
  fixture's border is grey, its captions black), at least 20 after, zero after
  Ctrl+Z.
- **Tag.** After placing, `form.sign-box` names the *next* box down; after
  Ctrl+Z, the first again; after redo, the next again.
- **File.** The copy is longer than the source, carries no `/ByteRange`, and
  still names `/FT /Sig` three times.

## Falsified

- Ledger never told of the placement and the ink black: four arms fire
  (placement, pixels, tag, redo).
- Undo not reported to the ledger: only the after-undo tag arm fires.
- The body region swapped for the pad's in the check: the layout arm fires.

## Settings isolation

Place with *Remember* unticked deletes `hand-signature.txt` beside the store.
`ui-verify` gives each check its own copy of the binary under a profile
directory, and the store is portable, so the operator's remembered signature is
never touched.
