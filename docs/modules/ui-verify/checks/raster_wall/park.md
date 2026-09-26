# `ui-verify/checks/raster_wall/park`

Getting the pointer onto the seam between two pages, and knowing how much
room is left below it.

===========================================================================
WHY THIS IS ITS OWN FILE
===========================================================================


  * [`seam_y`] says where the gap is this frame.
  * [`window_closes_at`] says how long the neighbour will survive there.
  * [`aim_at`] turns a window-logical point into something the OS can be
    told to click.
  * [`park_on_the_seam`] scrolls until the gap is in the band, reports the
    window it found, and REFUSES if that window is empty.

===========================================================================
THE LESSON THIS FILE IS THE RECORD OF
===========================================================================

**A check's measuring window can be bounded below by the application
and above by the harness's own geometry, and then it is flaky until somebody
does the arithmetic.** Part A's window was bounded below at zoom 20.7 (where
the neighbour becomes unorderable) and above at 18.9 (where the growing gap
pushed it off a 770-pt canvas). The window was EMPTY — yet the check did not
fail. It climbed its whole budget, observed no refusal, and SKIPPED with a
message saying the state had never been entered. One earlier run had
squeezed inside the window and passed, which is precisely the shape of a
check that has stopped running without anyone noticing.

The cure is in two parts, and both live in the parent's constants: a window
tall enough to leave room (`VIEWPORT`), and a seam band narrow enough to put
the seam where that room is (`SEAM_BAND`). What lives HERE is the third
part — [`park_on_the_seam`] computes the window before it spends a notch,
prints it into the report either way, and returns a SKIP carrying the
numbers when it is empty. A gate that reports *"the window was empty"* is a
finding. A gate that reports *"the state was never entered"* is a lie with
the grammar of a measurement.

===========================================================================
WHAT IS DELIBERATELY NOT HERE
===========================================================================

No verdict. [`park_on_the_seam`] returns `Err` for every refusal, which the
parent turns into a SKIP, never a FAIL — nothing in this file has measured
anything about O186, so nothing in it is entitled to fail a build. And no
trace parsing beyond calling `raster_wall::trace`: the geometry reads the
canvas's own published rect, and a check that assembles screen coordinates
from a window origin instead stops hitting anything the first time a dock
width changes.

## Item notes

### `fn aim_at`

Expressed as fractions of the published canvas rect rather than assembled
from a window origin, because that is the crate's coordinate contract: a
check that builds its own screen coordinates stops hitting anything the
first time a panel width changes, and a stale coordinate is
symptom-identical to a broken conversion.

### `fn window_closes_at`

The neighbour's top edge is one whole gap below the acting page's bottom edge
and the gap is `ROW_GAP_PT × zoom`, so the edge descends past the pointer at
roughly that rate and runs out of room at
`(usable bottom - seam) / ROW_GAP_PT`. [`BOTTOM_DEAD_BAND_PT`] is what makes
"usable" different from "published".

Deliberately an UNDER-estimate. The real rate measured nearer `10.7 ×
zoom`, because the pointer's document point slides down the screen a little as
the zoom rises, so dividing by 12 predicts the window closing earlier than it
does. An optimistic prediction here would let a run start a climb it cannot
finish and then report an unmeasured absence, which is the exact failure this
function exists to prevent.

### `fn park_on_the_seam`

Fails — as a SKIP — rather than guessing. A run that climbed from a seam
outside the band would push one of the two pages off screen early and would
then report "the neighbour was never visible", which is a statement about
the harness wearing the clothes of a statement about the application.
