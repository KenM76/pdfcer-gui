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
