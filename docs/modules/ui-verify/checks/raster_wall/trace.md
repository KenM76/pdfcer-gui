# `ui-verify/checks/raster_wall/trace`

Reading the raster-wall checks' evidence out of a `PDFCER_DIAG` trace.

===========================================================================
WHY THIS IS ITS OWN FILE
===========================================================================

Split out of `raster_wall.rs` on 2026-09-12, when that file reached 1,614
physical lines against standing rule R2's limit of 1,500. The response the
size gate asks for is to find the seam, not to shrink the prose, and the
seam here is obvious: everything in this file answers *"what did the
application say?"* and nothing in it drives, waits, or decides a verdict.

That division is worth more than the line count. A trace reader is pure —
one `&Trace` in, one value out — so every function here is readable and
arguable without knowing anything about wheel notches, seams or zoom
ceilings. The two `part_*` functions in the parent are the opposite: they
are a sequence of actions whose order is the whole content. Mixing the two
kinds in one file is what made the original hard to navigate.

===========================================================================
THE ONE PROPERTY EVERY FUNCTION HERE SHARES, AND WHY IT MATTERS
===========================================================================

**Every reader that can be fooled by a stale line takes or returns a line
number.** `crate::diag::trace_changed` de-duplicates per slot, so a line
written once stays in the file for ever and `events(X).last()` can return a
fossil from before the state changed. Two separate defects in this project
came from exactly that — a healthy `canvas` line standing over a blank
screen, and a dock tab's last caption outliving the tab.

So the contract of this module is:

  * A reader that asks *"is this true NOW?"* compares line numbers, never
    event names alone — see [`unavailable_now`].
  * A reader that asks *"did this happen since I acted?"* takes an `after`
    line number from the caller's mark.
  * A reader whose answer will be ATTRIBUTED returns the line number it
    found, so the caller can ask what preceded it — see
    [`first_bad_raster_after`] and [`went_blank_between`], which together
    tell three different defects apart that all produce one sentence.

===========================================================================
WHAT IS DELIBERATELY NOT HERE
===========================================================================

No geometry (that is `raster_wall::park`), no driving, no verdict. Nothing
in this file may call `Session`, because a reader that can act is a reader
whose answer depends on when it was called, and then a report cannot be
reproduced from the trace file alone.

## Item notes

### `struct CanvasState`

Read as one struct from one trace line on purpose. The alternative — a
helper per field — re-reads the trace per question and can answer two
questions from two different frames, which is how a check comes to report a
zoom from after a clamp against a visible count from before it.

### `fn unavailable_now`

`canvas` and `canvas-unavailable` share one trace slot, so the channel
emits whichever changed and the other's last line stays in the file for
ever. Asking `events("canvas").last()` alone would read a fossil from before
the canvas went empty and report a healthy frame over a blank screen. The
comparison is by line number, which is the same property
`driving::declared` relies on for retired regions.

### `fn first_bad_raster_after`

First rather than last: the first one is the one whose cause is still
legible, because `absorb_render` learns a ceiling from it and everything
after is a consequence of that.

The line number is returned because the attribution depends on what came
BEFORE the refusal — see [`went_blank_between`]. Three different defects can
produce this one line and the only way to tell them apart is the order of the
trace.

### `fn went_blank_between`

This is the discriminator between O186's third route and its first.
`canvas-unavailable reason=nothing-visible` says no part of any page was on
screen, which is upstream of everything: with nothing visible there is no
region, with no region the request is the whole sheet, and above the pixmap
ceiling a whole-sheet request is a refusal. A check that read only the
refusal would name `fill_strip` — the wrong function — and the next reader
would spend the investigation there.

Restricted to `reason=nothing-visible` on purpose. The other reasons the
canvas declines to draw (no document, a collapsed dock) are not this, and
matching the event name alone would make the discriminator fire on them.

### `fn last_two_visible_zoom`

Read only when part A's window turns out to be empty, to say *when* the
neighbour left rather than only that it was not there at the end. A climb
whose answer is `None` never had a neighbour at all, which is a different
fault from one that had it and lost it.

### `fn unfillable_counts`

⚠ These are **transitions, not frames.** `diag::trace_changed` writes only
when the formatted line changes, so a climb that declined for two hundred
consecutive frames counts 1. That is the right granularity for reading a
regime change and the wrong one for reading a duration, and a report that
called it a frame count would be overstating by two orders of magnitude.

Both halves are wanted. A count of declines alone cannot be distinguished
from a guard that never ran at all, which is why the guard traces both
transitions rather than only the interesting one.

### `fn last_learned_after`

`moved=false` lines are ignored deliberately: the shell traces its decision
either way, and a ceiling learned at a zoom the view was already below
changes nothing the operator can see — so it is not the event part B is
waiting for. A non-finite `to` is treated as no event at all rather than
compared against, because every comparison with `NaN` is false and a
silently-false assertion is worse than an absent one.
