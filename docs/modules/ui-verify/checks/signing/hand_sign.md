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
- **Placement.** `hand-sign-placed strokes>=2 tagged=1`: the page reads back
  a `/pdfc_HandSig` sequence naming the box's field.
- **Pixels.** Ink-coloured pixels (blue well above red and green) in the box
  and the band one box-height above it: zero before (the control: the
  fixture's border is grey, its captions black), at least 20 after, zero after
  Ctrl+Z.
- **Tag.** After placing, `form.sign-box` names the *next* box down; after
  Ctrl+Z, the first again; after redo, the next again.
- **File.** The copy is longer than the source, carries a `/pdfc_HandSig`
  tag and no `/ByteRange`, and still names `/FT /Sig` three times.

## Falsified

- The write left untagged (`MarkupOptions::hand_signature` `None`): the
  file and placement arms fire.
- The body region swapped for the pad's in the check: the layout arm fires.

## Settings isolation

Place with *Remember* unticked deletes `hand-signature.txt` beside the store.
`ui-verify` gives each check its own copy of the binary under a profile
directory, and the store is portable, so the operator's remembered signature is
never touched.
