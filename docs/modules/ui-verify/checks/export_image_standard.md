# `ui-verify/checks/export_image_standard`

`export_image_draws_the_chosen_background_and_standard` — Export image with a
typed background colour and a rendering standard writes a file drawn on that
colour, under that standard (O284).

## Steps

1. Launch on `fixtures/pure-k-square.pdf` off the desktop, with
   `PDFCER_DIAG_SAVE_PATH` naming `image-standard.svg`, deleted first.
2. **Control.** File ▸ Export image; choose SVG; clear *Keep transparency* if
   the window opened with it on; Export. The file must draw the pure-K square
   as `rgb(35,31,31)`, the calibrated CMYK intent. If it does not, the
   profile's settings already draw pure K as black, the standard cannot be
   told apart, and the check SKIPs.
3. Open the window again; SVG, transparency off. Click
   `export-image.background`, press End and Backspace once per character of
   the colour the window opened with, and type `#3366cc`.
4. Open `export-image.standard` and click `export-image.standard.pdf-x4`.
5. Export. The SVG must carry `fill="#3366cc"` (the background rectangle) and
   draw the square as `rgb(0,0,0)` with no `rgb(35,31,31)`.

Every click but Export first wheels the window's scrolling body (`dialog:export-image`)
until the target lies between the body's top and the pinned Export row: a
region is declared even while it is clipped, and a click on the clip lands on
nothing.

Pure K is the oracle for the standard because every PDF/X and PDF/A preset
sets `CmykIntent::NeutralBlack`, which renders it `#000000`, where the default
calibrated table renders it near `#231F20`. SVG is the format because its
colours are text; the harness carries no PNG decoder.

## Falsification

- With `export::image` applying the preset to a copy it then does not render
  with, step 5 fails on `rgb(35,31,31)`.
- With `svg_bytes` passing a white background whatever the plan says, step 5
  fails on the missing `fill="#3366cc"`.

## What it does not cover

The PNG and JPEG flatten onto a non-white colour (`flatten_over` and
`JpegOptions::background`), which `ImagePlan`'s unit tests and the action's
`background_note` reach only as far as the plan; the swatch picker; the
remembered colour across a restart.
