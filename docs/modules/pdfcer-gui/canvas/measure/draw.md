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

### `struct Preview`

Rule 4's pre-commit affordance: *"a snap indicator, a hover highlight, a
rubber-band … these are the cursor; they describe what is about to
happen."* It is only honest if it is derived from the values the commit will
use, which is why the placing preview goes through
[`pick::dimension_preview_segments`] — the *same* function a committed
dimension is drawn from — rather than drawing a line of its own.

### `fn draw_label`

The held dimension's value text, drawn at the hover point while it waits
for its placing click. Text and box come from `author_dimension` under the
group's resolved style, so the preview shows the value and size the commit
bakes. `label_quad` corners run baseline-left (at the descender),
baseline-right, cap-right, cap-left; the font size is the quad height over
1.3, the descender-to-cap span the baker sizes from, and the text is rotated
to the baseline. Arrowheads and the ANSI break are not drawn here: request
G063 asks the engine for an id-less preview to replace this path.
# The circular tool's preview is the whole of its feedback

The other two tools draw something that follows the pointer, so an operator
can see the gesture working. The circular tool's pick lands on geometry that
is *already drawn* — toggling an arc into the fit changes nothing on screen
unless this function says so — and its answer is a circle nobody has drawn
at all. Without both halves the operator cannot tell a set of three arcs
that fits their hole from one that has accidentally caught the leader line
beside it, and the residual is invisible until the dimension is already on
the page.

So two things are drawn, and the second goes through
[`pick::dimension_preview_segments`]:

1. **A marker on every picked point**, straight out of
   [`pick::CircularPick::points`]. A rectangle round every picked *object*
   is the wrong picture: on a real drawing one click outlines a 550 × 500 pt
   region — see `pick::CircularPick`'s header and `OPERATOR_REQUESTS.md`
   O105. A marker per point is both the honest
   picture of what is in the fit and the thing an operator aims at to take a
   point back out.
2. **The fitted circle**, derived by handing the *same*
   [`pick::CircularPick::author`] value the commit would use to the *same*
   segment function a committed dimension is drawn from. That is this
   module's standing rule and it matters more here than anywhere else: the
   fit is an inference, and an inference previewed by a second derivation is
   an inference the operator cannot actually check.

### `const SNAP_MARKER_PT`

Screen-space rather than page-space on purpose: the marker is an
affordance, not content, so it must stay the same apparent size whether the
operator is zoomed to a whole A1 sheet or to one dimension line. Carried
from the old shell's own indicator sizing.

`pub(in crate::canvas)` because the perimeter's vertex drag snaps too and
draws the SAME marker at the SAME size. A second constant would be two
sizes for one affordance, free to
diverge — and an operator who has learned that a small square means
*endpoint* while placing a perimeter must read the identical square while
correcting one.

### `fn page_to_screen`

# This function is the fix for a defect, and the defect had shipped

It replaced a `page_to_canvas` that stopped after the first hop and handed
the result straight to `ui.painter()`. Three frames are in play — screen,
canvas and PDF user (`crate::canvas::mapping`'s header carries the table) —
and `viewer::pdf_space_to_canvas` lands in the **middle** one: y-down, but
with its origin at the page's top-left corner and **no zoom applied**,
because `page_device_geometry` is asked for scale `1.0`. The painter speaks
screen. So every mark this module drew — the snap indicator and the linear
preview alike — was offset by wherever the page happened to sit in the
window and drawn at 100 % regardless of the actual magnification.

It is a whole class — a mark authored in the right place and drawn in the
wrong one — and it is invisible to every test in this file, because the
tests that exist are about *which point is picked* and the picking is
right. Only the drawing is wrong, and only in a window.

The second hop is [`PageMapping::to_screen`] — the canvas's own outward
boundary crossing, the one `canvas::overlay` has always used for selection
outlines. There is no third conversion here and no arithmetic; if there
were, it would be the second place in `canvas/` that divides by zoom, which
`crate::canvas::mapping`'s header forbids.
