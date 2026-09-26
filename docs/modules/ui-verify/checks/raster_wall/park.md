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

★★★ **A check's measuring window can be bounded below by the application
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
