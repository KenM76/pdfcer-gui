# `ui-verify/checks/off_page_zoom`

`an_object_off_the_page_survives_being_zoomed_in_on` — **O23's "edit"
half.**

# The report


> *"how do I view and edit objects that are off of the page? we added this
> feature but I didn't see how to enable it."*

Three verbs, and each one broke on its own:

| verb | check | what it proves |
|---|---|---|
| **reach** | `off_page_press` | a press in the grey becomes a gesture |
| **see** | `off_page_visible` | the object is painted at all |
| **edit** | **this file** | it is still there once you zoom in on it |

# Why "edit" is a separate property, and why the first two were green


The pasteboard O23 shipped was `viewport × 1.0`: a fixed count of **screen
pixels**. The slice of the **drawing** that slack covers is therefore
`viewport / zoom`, and it shrinks with every notch. An object `k` points off
the sheet can be brought to the viewport's **edge** only while
`viewport ≥ k × zoom`, and to its **centre** only while
`viewport ≥ k × zoom + viewport / 2`. Past that, the offset solved by the
zoom anchor is thrown away by `canvas::geometry::strip_offset`'s clamp —
which clamps to `content_extent − viewport`, and `content_extent` is built
from that same fixed pasteboard.

⇒ **The operator zooms toward the object and it walks off the screen.** Both
sibling checks stay green throughout: they run at 100 %, where the fixed
pasteboard is enormous relative to the 100 pt of overhang. That is the shape
of this whole request — each verb green, the operator still stuck.

The fix folds the *content's* overhang into the pasteboard.
`render::halo::overhang` measures it in page points, `canvas::present`
multiplies it by the zoom and publishes it once per frame onto
`OpenDoc::pasteboard_overhang`, and `canvas::geometry::pasteboard` takes
`max(viewport × FRACTION, overhang + viewport / 2)`. The `+ viewport / 2` is
the difference between *reaching* a point and *looking at* it.

# The climb is calibrated against THIS window, not against a constant

The old ceiling is a function of the viewport, so a check that zoomed to a
hard-coded 1000 % would have been a real test on a laptop and a vacuous one
on a wide monitor — the same silently-inert control this suite keeps
catching itself building. So the check **measures** the viewport, computes
the zoom at which the old pasteboard stopped reaching
([`OFF_PTS`] × zoom > viewport), and climbs [`PAST_THE_OLD_CEILING`] beyond
it before it asserts anything.

It reports that number. A reader of a *passing* run can see how far past
the old limit it actually got, which is the difference between "green" and
"measured".

# The oracle: ink above the edge, paper below it

The anchor is the **midpoint of the off-page square's bottom edge** —
`(−100, 100)` on the fixture. After the climb the trace is re-read and that
doc point is converted afresh, so this check does **not** assert that the
zoom anchor held it under the pointer: `zoom_keeps_place` owns that
property, and one check covering both would go red for one reason while the
other was still broken.

Two patches, at a fixed **logical-point** offset either side of where the
application says that edge now is:

* **above** it on screen — inside the square, because page y grows up while
  screen y grows down — must be ink;
* **below** it on screen — off the sheet, inside the widened raster — must
  be paper.

A fixed *screen* offset rather than a *page-point* one is what makes the
pair work at any magnification: the clearance from the edge stays constant
in the units antialiasing happens in, and shrinks to nothing in page points
exactly as fast as the zoom makes that irrelevant.
[`the_patches_straddle_the_edge_at_every_zoom_this_check_reaches`] pins the
arithmetic against the fixture's own content stream.

The **pair** is the point. If the halo is not painted, both patches read
canvas grey: the ink patch fails and the control passes, which is the
feature failing. If the probe is aimed at a panel or at the desktop, both
read dark: the control fires first and the check reports a **harness**
finding rather than an application one. A single-patch check cannot tell
those apart, and this suite has been wrong that way before.

# ⚠ One deliberate non-skip: the conversion refusing

`CanvasMapping::doc_to_window_off_page` returns an error when the point has
left the viewport. Everywhere else in this suite that is a SKIP. **Here it
is the defect itself** — "the object walked off the screen while I zoomed
toward it" is the literal complaint — so it is caught and reported as a
FAILURE carrying the arithmetic that predicts it.

The same call is made *before* the climb as well, and there it **is** a
SKIP, because a fixture that is already out of view at 100 % says nothing
about zooming. Same error, opposite verdict, decided by which side of the
climb it happened on.

# Every way this reports SKIP

No binary, no diagnostic channel, input disabled, no `canvas-viewport`
region, the anchor already off screen at 100 %, the canvas never settling,
or the zoom saturating before it reached the calibrated target. **Not** "the
object was not painted" and **not** "the point left the viewport after the
climb" — those are failures, and they are the two this check exists for.

## Item notes

### `const INVOKE`

`mode.edit` is named FIRST, and it is not decoration. Since
2026-09-11 the display of off-sheet content is a per-mode preference and
**Read ships with it OFF** — the operator's request: *"by default, read
doesn't show off page items, review and edit do show off page items."*
This check's whole subject is off the sheet, so without an explicit mode it
would run in whatever mode the shell opens in, find nothing, and report a
defect that is a correctly-implemented setting.

Edit rather than Review because that is the mode this check's gestures
belong in anyway, and because a mode named explicitly cannot drift when a
later session changes which mode the shell opens in.

### `const EDGE_AT`

The fixture draws `-160 100 120 40 re f`, so the square spans x −160…−40 and
y 100…140, and this is `(−100, 100)`. An *edge* rather than a centre, so
that ink and paper are a few pixels apart at any magnification — see the
module header.

### `const OFF_PTS`

This is the `k` in the ceiling arithmetic, and it is `|EDGE_AT.0|`;
[`the_off_page_distance_matches_the_anchor`] pins the two together so the
numbers this check prints cannot drift from the point it actually drives.

### `const PAST_THE_OLD_CEILING`

The old pasteboard stopped **reaching** `OFF_PTS` at `viewport / OFF_PTS`
and stopped **centring** it at half that. Climbing to 1.5× the looser of the
two clears both with margin, and the margin is what keeps a passing run from
depending on which rung of the zoom ladder the application happens to land
on.

### `const MAX_NOTCHES`

A cap, not a count: the loop climbs until the calibrated target is
reached. Hitting the cap is a SKIP, because a run that never got past the
old ceiling has not tested anything — it is neither a pass nor evidence of a
defect. One notch is about 1.223×, so this reaches roughly 4 × 10⁵ %.

### `const INK_FRACTION`

Not 1.0 — the patch is rounded through the window frame's scale and its
outermost row can land a pixel outside the square on a fractional-DPI
display.

### `const UNAVAILABLE_EVENT`

`canvas-unavailable reason=nothing-visible` is the one line that separates
*"the shell declined to zoom any further"* from *"the shell zoomed, and the
document surface went blank."*

### `fn stall_verdict`

# Why this function exists at all

The first draft of this check treated *any* stall as a precondition failure
and skipped, with the helpful-sounding suffix *"Raise `max_zoom_percent`, or
run in a narrower window."* Both halves of that sentence were wrong:

* `max_zoom_percent` defaults to `1e12` and its floor is `10.0`, so the
  operator's zoom cap was never what stopped the climb. **That suffix was an
  excuse the check had not measured**, and an unevidenced excuse is worse
  than silence — it reads as an answered question, so nobody investigates.
* The stall on 2026-09-11 was **the defect this check exists to find**. The
  strip culled pages on the sheet's rectangle rather than on the rectangle
  its content actually reaches, so once the magnification carried the sheet
  off the viewport the canvas laid out nothing, dropped its `canvas-viewport`
  and `page` rects, and stopped publishing a zoom to climb with. Reported as
  SKIP, that read as *"the harness could not run"*, which is the exact
  failure mode this project has written down three times: **a SKIP is not
  red, so a check can stop running unnoticed.**

# What it measures

A fresh trace, read at the moment of the stall, and only the
`canvas-unavailable` lines written **after** `mark` — the mark being taken
immediately before the first notch, so a line written while the document was
still opening cannot be mistaken for one the climb provoked.

* A `reason=nothing-visible` after the mark ⇒ **FAIL**, naming the cull.
* Anything else ⇒ **SKIP**, stating what was and was not measured and
  offering no cause it did not observe.

A trace that cannot be re-read is itself a SKIP: the stall is real but the
evidence is not available, and guessing between the two verdicts is how a
harness invents defects that do not exist.

### `fn the_patches_straddle_the_edge_at_every_zoom_this_check_reaches`

In page points the patches reach `(OFFSET + HALF) / zoom` from the edge,
so the ink patch stays inside the square while that is under the
square's height, and the paper patch stays inside the widened raster
while it is under the distance down to the raster's bottom. Both get
*easier* as the zoom rises, so the binding case is the LOWEST zoom the
check ever asserts at — which is not a constant, it is
`PAST_THE_OLD_CEILING × viewport / OFF_PTS`. Evaluated here at a
viewport narrower than any dock layout this shell produces.
