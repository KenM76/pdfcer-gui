# `canvas::dimpreview`

A ce dimension being dragged is drawn as the engine will write it.

## Contract

- `bake(doc, id, &moved)` calls `EditSession::dimension_preview` for the
  geometry the drag would commit. On refusal it traces
  `dim-preview id=… baked=0 refused=…` and returns `None`.
- `paint(painter, doc, page, map, clip, preview)` rasterises the preview's
  `/Rect` — expanded 2 pt on screen and clipped to the visible canvas — with
  `pdfcer_render::edit_preview::paint_dimension_preview`, uploads it as one
  texture and draws it. It traces
  `dim-preview baked=1 px=W×H faults=N label=x0,y0,…,x3,y3`, where `label`
  is the label's quad in page space. Returns `false` when nothing was drawn:
  the box is off screen or wider than 8192 px.
- The painter (`canvas::painting`) draws the baked preview when there is
  one and `paint` succeeds, and falls back to the segment outline
  (`measure::pick::dimension_preview_segments`) otherwise. Never both.

## Why the engine's bake and not the shell's outline

The outline was the shell's own drawing of the dimension, and a second
rendering path for the same content drifts from the first. The bake is the
appearance stream the commit writes, so the label, arrowheads, extension
gap and text size seen mid-drag are the ones the release produces. The
committing frame draws no preview: the annotation is regenerated and drawn
for real that frame.

## Coordinate spaces

`page_to_screen` reads the page-to-screen affine off three points mapped by
`canvas::measure::page_to_screen`, so rotation and crop come from the one
mapping the rest of the canvas uses. The raster transform is that affine
scaled by pixels-per-point and translated to the clipped box's origin.

## Cost

One raster and one texture upload per dragged frame, sized to the visible
part of the dimension's box. Recorded under
`pressure::Surface::DimensionDrag`.

Driven by `ui-verify/checks/dimension_label_drag`.
