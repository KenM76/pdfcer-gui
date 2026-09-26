# `pdfcer-gui/canvas/measure/draw`

## Item notes

### `const PICKED_RING_SCALE`

Big enough to read as a ring around the glyph rather than as a fatter glyph,
small enough that four picks round a small hole do not merge into a blob.
See the ring's own comment in [`preview`] for why the distinction exists at
all.

### `fn the_preview_projects_page_space_all_the_way_to_the_screen`

The regression test for the defect `page_to_screen`'s own docs describe:
`viewer::pdf_space_to_canvas` lands in **canvas** space — page top-left
origin, no zoom — and the painter speaks screen, so a preview that
stopped after the first hop drew every mark offset by wherever the page
sat in the window and at 100 % whatever the magnification.

Asserted as a **magnitude**, not a relation: at zoom 2 with the page's
corner at (37, 11), the page-space point that is 50 canvas units in from
the page corner must land 100 screen points in from (37, 11). A test
that merely checked "the two differ" would be satisfied by any wrong
answer in the right direction.
