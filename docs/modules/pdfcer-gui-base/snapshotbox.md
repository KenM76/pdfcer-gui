# `pdfcer-gui-base::snapshotbox`

The box View ▸ Snapshot leaves on a page (O272): a zero-based page index and a
rectangle in PDF user space, in points, normalised so `llx ≤ urx` and
`lly ≤ ury`.

## Contract

- **Page space, not screen space.** The box is the page area the operator
  chose. Zoom, scroll, a window resize and a rotation of the view all change
  where it is drawn and none changes what it holds; the canvas re-maps it every
  frame (`canvas::snapshot::screen_rect`).
- **One per document.** It lives on `OpenDoc::snapshot`, so each tab has its
  own and closing the tab drops it. Laying a new box replaces the old.
- **View state.** Nothing is written to the document; a saved file never
  contains it.
- `from_corners(page, a, b)` takes two corners in either order and returns
  `None` when either side is under `MIN_SIDE_PT` (2 pt): a release that close to
  its press is a slip, and a box too thin to see would be a box the operator
  cannot find to clear.
- `trace_fields()` is the `snapshot-box` trace payload,
  `page= llx= lly= urx= ury=` to 0.1 pt. The ui-verify check
  `a_snapshot_box_stays_on_the_page_through_a_zoom` parses those keys.
