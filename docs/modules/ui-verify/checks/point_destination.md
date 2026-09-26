# `ui-verify/checks/point_destination`

**A destination that names a point scrolls, and never sets the
magnification** — the driven half of `DEFECTS.md` D47, for
`OPERATOR_REQUESTS.md` O200.

# The operator's report

> *"after we made it so bookmarks in drawing files exported from solidworks
> zoom to the correct spot on the page, this behaviour carried over onto
> links in word documents saved as pdfs such as table of contents where they
> should just jump the position on the page pointed to without changing the
> zoom. If the position jumped to is visible on the page with the current
> horizontal position of the page, the horizontal position shouldn't be
> changed."*

Two clauses, and they need different instruments. The first is an
**identity**: the magnification after the jump is the magnification before
it, to the last digit the canvas prints. The second is a **conditional**, and
a conditional cannot be measured by one drive — "the horizontal did not move"
is also what a shell that ignored the destination entirely would produce.

# Why there are two checks over one fixture

| check | link | destination `left` | what it asserts |
|---|---|---|---|
| [`APointDestinationLeavesTheMagnificationAlone`] | near | 168 — under the pointer | the zoom is unchanged **and** the horizontal is held where the operator left it |
| [`APointDestinationOffScreenMovesTheHorizontal`] | far | 1150 — across the sheet | the zoom is unchanged **and** the horizontal moved |

`fixtures/xyz-null-zoom.pdf` is built so the two links differ in exactly one
property: same target page, same `top`, same null zoom, same rectangle size
and the same rectangle `x`, so both drives zoom about the same content point
and arrive at the click with the same horizontal geometry. Only the
destination's `left` differs. A difference in outcome therefore has one
candidate cause. Its `PROVENANCE.py` carries the argument for every number.

The pair is what makes either one evidence. Without the far link, a shell
that never touched the horizontal at all would be green; without the near
link, a shell that always recentred would be.

# Why it zooms in first

For the reason `link_follow` records from a falsification that failed to
falsify: at a fitted view there is nothing for the check to measure. A
1,224 pt page fitted into the window has no horizontal scroll range, so the
offset is pinned at the same number whatever the shell decides, and the
"held" assertion is satisfied by a build with no horizontal logic in it.
Eight Ctrl+wheel notches about the link's own centre put the page several
times the width of the viewport; the range is then **measured** before the
click, and a run that did not get enough of it reports SKIP with the numbers
rather than passing on a degenerate view.

Zoom-to-cursor also does the check a second favour: it holds the point under
the pointer fixed, so the near destination is on screen **by construction**
and the check never has to predict a window size, a fit zoom or a notch step.

# What it reads

`canvas … zoom= page= off=` for the view the operator ends up with, and
`dest-scroll-solved … origin_x= keep_x= off_x=` for what the solver decided.
Both, deliberately: the trace of a stage records what that stage decided, not
what the frame settled on, and the offset chain has seven other ranks that
could overwrite it. `origin_x` is also asserted against the offset measured
before the click, which is the only way to catch the plumbing defect this
feature actually had — `canvas::strip::page_scroll_offset` centres the named
page horizontally on the frame the page turns, so an origin read after that
measures the centring rather than the operator.

# Every way these report SKIP

No binary, `--no-input`, no diagnostic channel, no canvas rect to aim
against, the fixture missing, or a window so wide that eight notches of zoom
still leave no horizontal scroll range. None of those is a pass, and each
says which it was.

## Item notes

### `const ZOOM_NOTCHES`

Enough that a 1,224 pt page is several times the width of any window this
runs in. The range is measured afterwards regardless — this constant is the
thing to raise if that measurement starts reporting SKIP.

### `const MIN_MOVE`

The far destination is 982 pt to the right of the near one and the view is
magnified, so the real figure is several hundred points. This is a floor
well clear of a rounding difference, not an expected value.

### `struct View`

The zoom is carried as the **string** the application printed rather than a
parsed float, because the first clause of the request is an identity and the
honest test of an identity is byte equality. A parsed `f32` compared with a
tolerance would accept a shell that re-derived the same magnification by a
different route, which is the failure D47 was.

### `fn both_aim_points_are_inside_their_link_rectangles`

Pinned because a click on empty paper is symptom-identical to a broken
hit test, and these numbers are transcribed by hand from the fixture's
`PROVENANCE.py`.

### `fn the_two_drives_differ_only_in_the_destination`

The whole argument for the pair rests on this: same x, so both zoom about
the same content column and reach the click with the same horizontal
geometry; different rows, so they are different links; different
destination `left`, which is the one input the request's second clause
turns on. If a future edit moves one aim sideways, the two outcomes stop
being attributable to the destination.

### `fn the_near_destination_is_the_point_the_pointer_is_on`

That equality is what makes it visible by construction — zoom-to-cursor
holds the content under the pointer fixed — so the check never has to
predict a window size or a notch step. Break it and the control drive
starts depending on the machine it runs on.

### `fn the_movement_floor_fits_inside_the_range_required`

If `MIN_MOVE` ever exceeded `MIN_RANGE` the witness drive could not pass
on any view the precondition admits, and the failure would read as a
product defect.
