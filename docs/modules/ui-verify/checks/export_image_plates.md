# `ui-verify/checks/export_image_plates`

`export_image_writes_one_metafile_per_ink`: File ▸ Export image ▸ EMF ▸
*One file per ink (plates)* writes a metafile for every separation plate,
each named for its ink.

## What it drives

Off the desktop, scripted pointer only.

1. Write a one-page PDF painting two `/Separation` inks, `Ink0` and `Ink1`
   (`spots_flattened::spots_pdf(2)`), under a CMYK page group.
2. Open the window through `PDFCER_DIAG_INVOKE=file.export_image`. The save
   picker answers from `PDFCER_DIAG_SAVE_PATH`, a fresh folder in the run
   folder cleared of `.emf` files first.
3. Click `export-image.format.emf`, then `export-image.plates`, then
   `export-image.export`.

## Oracles

- **Offered.** `export-image.plates` is declared once EMF is selected.
- **One per ink.** At least six `export-image-plate` lines (four process inks
  and two spots), and as many `.emf` files on disk as lines.
- **Named and real.** A file whose name holds `Ink0`, and one holding `Ink1`,
  each with `ENHMETA_SIGNATURE` at offset 40.

## Falsified by

Ignoring `plan.plates` in `app::actions::export::image` writes one composite
metafile and no `export-image-plate` line.
