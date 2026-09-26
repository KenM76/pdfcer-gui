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
