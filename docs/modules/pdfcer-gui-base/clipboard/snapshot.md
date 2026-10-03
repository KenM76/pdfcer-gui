# `pdfcer-gui-base::clipboard::snapshot`

Copies the snapshot box (O272) to the operating system's clipboard: the region
under the box and nothing else, in the formats and order a page copy uses (SVG,
EMF, PNG, DIBV5), then the cut page itself as `application/pdf`, placed by
[`place`](place.md).

## Contract

- `copy_snapshot(doc)` reads `OpenDoc::snapshot`, the page it names, and the
  operator's resolution `prefs.snapshot.dpi()`. `Refusal::NoPage` when there is
  no box or it does not overlap its page's crop box; the other refusals are the
  page copy's, with the same sentences.
- **The engine cuts the region out.** `pageops::extract_region` takes the
  document's view (so unsaved edits are copied), the page, the box intersected
  with the page's crop box (`cropped`), and a `RegionExport` carrying the
  viewer's state: `with_annotations` from `place::render_options`, and
  `with_hidden_layers` from `OpenDoc::layers.hidden` when the operator has
  overridden the document's default layer state. Its result is a one-page PDF
  whose page is the box: annotations, fields and ce dimensions flattened (or
  gone when the review layer is hidden), hidden-layer content deleted, and the
  content outside the box removed by redaction rather than clipped. Every format
  is made from that page, so nothing outside the box reaches any of them.
- **What is copied is what is shown.** The cut page is rendered and exported
  with the viewer's render options, its layer overrides dropped: the cut has no
  optional content left, and the source's layer ids name nothing in it.
  Selection outlines, grips, guides, grids and the box itself are GUI furniture
  the canvas paints over the page, and never reach the clipboard.
- **`Vectors` says what became of the vector forms.**
  - `Cut { glyphs_removed, notes }` — SVG, EMF and the picture placed through
    `place`. `glyphs_removed` counts characters crossing the box's edge, which
    the engine leaves out whole; `notes` are its sentences about anything else
    it removed or could not flatten.
  - `Withheld { notes }` — `RegionReport::has_residuals` is true: a path, a
    shading or a form straddling the edge could not be cut, so a vector form
    would carry drawing from outside the box. The picture, rendered from the
    cut page, is placed alone through `place_withheld`.
  - `Refused(why)` — the engine declined the region (a certified or encrypted
    document, an image it could not cut, among others; `RegionError`'s
    sentence). The picture is rendered from the original page cropped to the
    box with `render_page_with_view`, and placed alone.
- **The cut page goes on the clipboard too**, as `application/pdf`, last in
  `ORDER`, and only when the vectors are `Cut`: a withheld or refused cut would
  carry drawing from outside the box. Its reader is pdfcer itself, whose paste
  takes it as a drawing (`clippaste::read`), so a snapshot pasted back into a
  document stays vector. It costs a copy of bytes the cut already made.
- **Resolution.** `fitted_dpi(rect, asked)` is the highest whole dpi at or
  below the operator's whose picture fits both the renderer's edge limit
  (`MAX_PIXMAP_EDGE` less one pixel, for the renderer's rounding up) and
  `MAX_PIXELS` (50 million); never below 1. The vectors' internal rasters use
  the same dpi, and the PNG and DIB carry it as their physical resolution.
- `SnapshotCopy` reports the formats placed, the dpi used and the dpi asked,
  the picture's pixel size and the `Vectors` outcome; the dispatcher traces them
  and states them on the status row (`text::snapshot::copied`).
- `snapshot_pdf(doc)` is the same cut, `region_state` and all, returned as
  bytes with the engine's `RegionReport` instead of placed on the clipboard;
  *Save as PDF…* on the box writes it to a file (`app::dispatch::snapshotsave`).
  `Refusal::NoPage` and `Refusal::Render` mean what they mean for the copy.
- **Cost.** The cut runs on the UI thread. It has not been measured on a dense
  drawing.
