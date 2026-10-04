# `ui-verify/checks/insert_drawing`

`insert_image_places_a_drawing` — Edit ▸ Insert image with an SVG, an EMF
and a PNG, each on its own launch, driven only through `ScriptedPointer` on a
window off the desktop, so it runs under `--no-input`. The picker is answered
by `PDFCER_DIAG_IMAGE_PATH`.

## Steps, per file

1. `fixtures/pure-k-square.pdf`; the picked file is `fixtures/vector-art.svg`,
   `fixtures/vector-art.emf` (see `fixtures/vector-art.PROVENANCE.md`) or a
   4 x 4 PNG the harness encodes.
2. Clicks: `ribbon.mode.edit`, `ribbon.tab.edit`, the collapsed group when
   the band is narrow, `ribbon.item.edit.insert_image`, then
   `insert-image.insert` inside the dialog's own viewport.
3. The trace must carry `image-imported kind=<svg|emf|image>`; the SVG's
   `notes=` must name `text`, which the engine does not carry.
4. `insert-image-requested kind=…` must follow, with a numeric `dpi=` for the
   picture and `dpi=none` for a drawing.
5. `add-<kind>` with no refusal; its disclosures carry "placed as vector
   artwork" (drawings) or " dpi" (the picture), and the SVG's repeat the
   `text` note.
6. A `selection-set … via=placed` after the placement line.

## Falsification

- Dropping the import notes from the drawing disclosures fails it at step 5.
- Not selecting the placed object fails it at step 6.
- Forcing the dialog's `dpi` to `none` fails it at step 4 on the PNG.

## What it does not cover

The rendered page: no capture is taken. A stretched drawing's `distorted`
disclosure is not exercised (the default box keeps the drawing's shape).
Dropping a drawing onto the page shares the import and apply code but is not
driven here.
