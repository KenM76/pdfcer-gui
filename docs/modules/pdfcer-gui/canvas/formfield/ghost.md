# `pdfcer-gui/canvas/formfield/ghost`

## Item notes

### `const GHOST_PX`

Flat rather than scaled with zoom: this is a cursor affordance, and a cursor
that grows with magnification stops reading as one. The same argument
[`crate::canvas::measure::SNAP_MARKER_PT`] carries for the snap glyph.

### `fn the_ghost_is_the_rect_the_click_places_at_every_rotation`

The two halves a preview can get wrong independently. The first is the
`/Rect` itself: the click point must be its lower-left corner in **PDF**
space and its extent must be the kind's default size, which is what a
build that added the size in canvas units would fail on a quarter-turned
page. The second is where that rect lands back on the canvas: the click
point must still be one of the drawn box's corners — a different corner
on each quarter turn, which is why the assertion is "some corner" rather
than a named one — and the drawn box must be the kind's size with its
axes swapped on 90 and 270.

A build that skipped the projection and hung a screen-space box off the
pointer passes the first half and fails the second on 90 and 270 only,
which is precisely the sheet the operator notices and the test author
does not.

### `fn click_rect`

`at` is **canvas** space, as the click handler receives it; the answer is
PDF user space, because that is what goes into the file.

The click point is the **lower-left** corner rather than the centre, which
matches what the drag does — the press is one corner and the control grows
from it — so the two gestures agree about what the pointer meant.

`None` for a page whose device transform cannot be inverted, which is the
same refusal [`crate::canvas::markup::band::endpoints`] makes and for the
same reason.

### `fn preview`

`pointer` is canvas space and `None` when the pointer has left the widget,
in which case nothing is drawn — honestly, because there is no click to
describe.

# Why the projection goes the long way round

The rect is computed in **PDF** space and projected back out through
[`mapping::annot_canvas_rect`], which is the same function that places an
existing widget's outline on the canvas. A ghost drawn as a screen-space box
hung off the pointer would need width and height in pixels, which means
deriving the page scale and the page rotation here — a second copy of the
projection, agreeing with the first on an unrotated page and disagreeing on
every `/Rotate 90` sheet. Going through PDF space costs one inversion per
frame and makes the ghost and the placed box the same geometry by
construction.
