# `pdfcer-gui/app/actions/export/plates`

One page of an image export written as one EMF per separation plate, through
`pdfcer_emf::export_emf_plates_view`. The recording scale and patch
resolution are the plan's resolution (`EmfOptions::raster_dpi`).

- **Separated.** Each plate goes to `imageexport::plate_path`: the page's own
  name with the ink after a hyphen. The receipt gives the plate count and, per
  plate, its vector operations and how many parts had to be pictures
  (`PlatePatches::rasters`). Each plate traces `export-image-plate` with its
  ink, file, ops and per-cause patch counts.
- **Composite.** A page the engine cannot composite in ink is written as its
  ordinary metafile at the page's name, and the receipt says it was not
  separated (`export-image-plates composite=1`).

A write failure stops the run with `text::export_image::write_failed`, as the
one-file path does.
