# `ui-verify/checks/fit_places_the_view`

`a_fit_command_puts_the_page_on_screen` — the three fit modes, driven, with
the view deliberately thrown away first.

# The reports


> *"If I press the Fit width or fit page button the view should center to
> the width as well or center the page."*

> *"Adobe has fit height, so add that too."*

# Why the run must pan into the pasteboard FIRST


So the run scrolls hard into the pasteboard, **asserts that it got there**
(a run that failed to displace the view has not set up its own precondition
and is a SKIP, not a pass), and only then presses the button.

# What is asserted, per mode

Read from the `canvas` trace line's `rect=`, which is the page's true drawn
rect on screen, against `canvas-viewport`'s:

| mode | the claim |
|---|---|
| **Fit page** | every edge of the page is inside the viewport, **and the margins are equal on both axes** |
| **Fit width** | both vertical edges are flush with the viewport's, so the full width is what is on screen |
| **Fit height** | the mirror: both horizontal edges flush |

Fit page's claim is *contained and centred*, not "fills both axes". A
landscape sheet in a tall window fills the width and floats in the middle
vertically, and a check that demanded both axes fill would fail on every
page whose aspect differs from the window's. **Equal margins is the direct
statement of what the operator asked for**, and it is also the one thing a
fit that set only the scale cannot produce: a page can be entirely inside
the viewport and still jammed against one edge with a viewport of
pasteboard on the other.

"Flush" within a tolerance, not exactly: the fit divides in `f32` and the
page rect is rounded to the pixel grid, so demanding exact equality would
be a check that fails on arithmetic rather than on behaviour. A few points
— far below the *hundreds* the defect moves the page by, and far above the
rounding.

## Item notes

### `const PAN_NOTCHES`

O23 gives a whole viewport of slack on each side, so this has to be enough
to cross it. Overshooting is free — the scroll area clamps — and
undershooting would leave the page still on screen, which the precondition
below catches rather than silently accepting.

### `const EDGE_TOLERANCE`

Not a tolerance on the DEFECT, which moves the page by a whole viewport
or more. This absorbs the `f32` fit division and the pixel-grid rounding of
the page's drawn rect, and nothing else: a build that placed the page one
tenth of a viewport out would still fail by a wide margin.

### `const CANVAS_MARGIN`

Read once, from the application's own reason for existing rather than
from an observation. `canvas`'s comment: the margin is subtracted from the
viewport BEFORE the fit divides, *"so 'fit page' really does fit with the
gap visible instead of fitting exactly and then being clipped by the
gap"*. A check that demanded the page sit flush against the viewport would
therefore fail a correct build by exactly this number — which is what the
first driven run of this check did, and it is worth keeping the reason
rather than the constant: **an extreme-end mismatch is usually the
instrument, and here the instrument was asserting a promise the
application had never made.**

### `const RESIZE_BY_PX`

Large enough that the change dwarfs [`EDGE_TOLERANCE`] — a resize the
check cannot distinguish from noise would make the phase vacuous — and
small enough that the ribbon still lays out, since a window too narrow to
draw the ribbon fails for a reason that is not the subject.

### `const BORDER_PX`

Approximate on purpose, and the check does not depend on the number
being right: it resizes by a delta and asserts the CANVAS changed, then
restores by the same arithmetic. An error here makes the window a few
pixels different from where it started and is invisible to every claim —
whereas assuming `client_size` IS the window size would shrink it by the
chrome on every iteration, which compounds.

### `enum Claim`

Three claims rather than a pair of `fill` booleans, because the three
modes do not differ by a flag — they differ by **what they promise**, and
fit-page's promise is the odd one: it does not fill either axis in general
(a landscape sheet in a tall window fills the width and floats in the
middle vertically), it *contains and centres*. Writing that as two
booleans is what produced a first draft that asserted fit-page filled both
axes, which is false for every page whose aspect differs from the window's.
