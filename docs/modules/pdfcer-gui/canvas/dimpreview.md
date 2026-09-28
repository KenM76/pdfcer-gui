# `canvas::dimpreview`

A ce dimension being dragged is drawn as the engine will write it.

## Contract

- `bake(doc, id, &moved)` returns a `Baked { id, preview }` from `EditSession::dimension_preview` for the
  geometry the drag would commit. On refusal it traces
  `dim-preview id=… baked=0 refused=…` and returns `None`.
- `paint(painter, doc, page, map, clip, &baked)` first draws the underlay
  (below), then rasterises the preview's
  `/Rect` — expanded 2 pt on screen and clipped to the visible canvas — with
  `pdfcer_render::edit_preview::paint_dimension_preview`, uploads it as one
  texture and draws it. It traces
  `dim-preview baked=1 px=W×H faults=N label=x0,y0,…,x3,y3`, where `label`
  is the label's quad in page space. Returns `false` when nothing was drawn:
  the box is off screen or wider than 8192 px.
- **The underlay.** The committed ce dimension is still in the page tiles,
  and a shortened extension line is a sub-segment of the committed one, so
  the bake alone would show no change. `underlay` covers the committed
  dimension's `/Rect` (its current geometry baked, 2 pt margin, clipped to the
  canvas and snapped to whole pixels) with `pdfcer_render::render_page_region`
  of that rectangle, rendered with `RenderOptions::with_omit_annotations([its
  annotation])` at the canvas's own scale. The texture is cached in egui
  memory, keyed on the annotation, the snapped screen box and
  pixels-per-point, so it is rendered once per drag and again only on scroll
  or zoom. It traces `dim-preview underlay=1 omit=N px=W×H` when it renders,
  `underlay=0 refused=…` when the engine refuses. Drawn only when the
  page-to-screen affine is axis-aligned with y flipped, that is an unrotated
  page; on a rotated page the bake is drawn without it and a shortening shows
  only through the grip.
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

The underlay is one region render per drag. A region render still
interprets the whole content stream (`render_page_region`'s own cost note), so
on a dense sheet the press frame pays it once. After that, one raster and one texture upload per dragged frame, sized to the visible
part of the dimension's box. Recorded under
`pressure::Surface::DimensionDrag`.

Driven by `ui-verify/checks/dimension_label_drag`.
