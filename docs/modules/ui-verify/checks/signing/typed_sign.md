# `ui-verify/checks/signing/typed_sign`

`a_typed_signature_lands_in_its_box` — O269's typed signature, end to end.

## What it drives

Off the desktop, scripted pointer only, on a copy of
`fixtures/esign-three-boxes.pdf`.

1. Read the first tagged box (`form.sign-box`) and photograph the window.
2. Click the box; `hand-sign-opened` must follow. Click `handsign.tab-type`.
3. Click `handsign.name`, type the made-up name *Pat Example*, press
   `handsign.place`.
4. Photograph; Ctrl+Z; photograph; Ctrl+Shift+Z.
5. File ▸ Save a copy to `PDFCER_DIAG_SAVE_PATH`.

## Oracles

- **Offered.** `handsign.tab-type` is declared. It is drawn only when a
  handwriting face is installed and embeddable; on the machines this suite
  runs on one is, so its absence is a finding, not a skip.
- **Layout.** The tab, `handsign.name`, `handsign.preview` and
  `handsign.place` lie inside `handsign.body` (the dialog is an immediate
  viewport, which eframe never screenshots).
- **Placement.** `hand-sign-placed via=type chars=11 signed=1`, with a face.
- **Pixels.** The same ink oracle as the drawn check: zero before, at least 20
  after, zero after Ctrl+Z.
- **Tag.** After placing, `form.sign-box` names the next box down; after
  Ctrl+Z, the first again.
- **File.** The copy embeds a TrueType program (`/FontFile2`), so another
  reader shows the same face, and carries no `/ByteRange`.

The trace carries the character count, never the name, so a run on the
operator's own file records nothing personal.
