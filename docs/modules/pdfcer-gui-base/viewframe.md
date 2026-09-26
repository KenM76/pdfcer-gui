# `viewframe` — what one canvas remembers between frames

[`ViewFrame`] is the per-**view** bookkeeping a canvas writes at the end of
a frame and reads at the start of the next: where the scroll area settled,
what the zoom was, where the zoom is anchored, and the deep tier's position
when the scroll offset can no longer carry it.

## Why it is not part of [`ViewState`]

[`ViewState`] is a record of **choices** — a zoom that was *set*. Its
header turns that into a licence to derive `PartialEq` over an `f32`, on
the ground that two states which arrived at 1.0 by different routes are
genuinely the same state. Nothing here is a choice. `observed_zoom` is a
measurement, `zoom_commit_at` is a clock reading, and two views showing the
identical thing will hold different values for both. Folding these in would
make that equality quietly mean "and was arrived at during the same
millisecond", which is not what any caller of it wants.

So a view is a *pair*: the stance it was put into, and what its canvas
observed while presenting that stance.

## Why it is per view and not per document

Every field here is frame bookkeeping about **a** canvas. With one canvas
on screen, storing it on the document is indistinguishable from storing it
on the view. With two canvases showing
one document — `OPERATOR_REQUESTS.md` O226 — a single copy is not a
limitation but a defect: the second pane's scroll overwrites the first
pane's settled offset, and the first pane then pans from a position it was
never at. The same goes for the zoom anchor, which would put pane A's
zoom-to-cursor over pane B's cursor.

## Item notes

### `struct ViewFrame`

Written by [`crate::canvas`] at the end of a frame, read by it at the
start of the next. Nothing outside the canvas and the render settle logic
has a reason to write any of it.

### `fn new`

`observed_zoom` takes that zoom rather than a sentinel so the first
frame does not read as "the zoom just changed" and schedule a
rasterization the opening render is already doing.
