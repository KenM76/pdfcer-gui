# `ui-verify/checks/deep_pan`

`panning_at_deep_zoom_stays_where_it_was_put` — the operator's "it jumps
back" report, made falsifiable.

# The report


> *"is that the challenge I was running into trying to pan over a little bit
> at high zoom, but it would jump back to it's original location I panned
> from because I couldn't pan to the next point?"*

Two claims in one sentence and they need separating, because they have
different causes and only one of them is a bug:

| claim | would be |
|---|---|
| *"I couldn't pan to the next point"* | a **quantised** pan — the view refuses small movements and only moves in steps |
| *"it would jump back to its original location"* | a **reverting** pan — the view moves and is then put back |

A quantised pan is what an `f32` scroll offset does when its
representable spacing exceeds the drag: `last - pan` rounds straight back to
`last`, so the view does not move at all. It looks like the drag was
ignored. A reverting pan is something actively re-setting the offset after
the drag — a different fault with a different fix.

This check tells them apart by measuring the offset at three moments: before
the drag, immediately after, and several frames later.

# Why it rolls the wheel rather than dragging

The first version drag-panned with the primary button and reported the view
as stuck. That was a **harness** defect: `canvas::input::pan_delta` pans on
the middle button always and on the primary button only under the hand
tool, so a primary drag with the default Select tool correctly rubber-band
selected and correctly moved nothing. The check had measured a gesture the
application never offered, and blamed the application for not honouring it.

It is recorded here rather than quietly fixed because it is the same shape
as the false layout report in `checks::driving::declared`'s header: a
measurement of the wrong thing is indistinguishable, from the verdict line,
from a real defect. **Ask what the check sampled before asking what is
broken.**

The wheel is unconditional — no tool, no modifier, no button — so a view
that does not move under it is unambiguously the application's fault.
