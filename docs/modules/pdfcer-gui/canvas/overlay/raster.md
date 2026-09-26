# `pdfcer-gui/canvas/overlay/raster`

## Item notes

### `const RASTER_GHOST_ALPHA`

High enough that the lettering reads as lettering rather than as a smudge,
low enough that whatever it is passing over stays visible — which is the
whole reason the copy is translucent: the operator is choosing where to put
the chunk by looking at what is already there.
