# `egui-shell/dock/report`

## Item notes

### `fn a_report_carries_the_clip_in_force_not_the_region_itself`

Worth a test of its own because the failure mode is invisible: a
reporter that handed back `rect` as its own clip, or the screen
rectangle as everybody's clip, would make every consumer's
visibility fraction come out at exactly 1.0 and every check built
on it green forever.

### `struct RectReport`

# Why it carries a second rectangle

A name and a rectangle is **a report about layout**, and it is
routinely read as a report about **visibility**. The two are not the
same claim: a surface can hold a perfectly ordinary rectangle, publish
it, satisfy every check built on it, and still be unreachable on
screen. `D:/dev/rag/egui/` records that costing this project three
panels at once.

A consumer that wants the stronger claim — *the operator can see
this* — needs to know what the region was **clipped to**, because
"laid out at these coordinates" and "at least three fifths of it
survived the clip" are answerable only together. So a report carries
the clip rectangle in force at the moment the region was published,
and what the consumer does with it is the consumer's business (see
[`Reporter::report`] and, on the application side,
`pdfcer_gui::diag::ui_rect_visible`).

# Why a struct rather than two positional rectangles

`FnMut(&str, Rect, Rect)` is the dangerous shape: **two adjacent
parameters of the same type, whose meanings are not symmetric.** A
consumer that swaps them compiles, runs, and produces plausible
numbers — `clip.intersect(rect)` is commutative, so the *intersection*
is unchanged, but the denominator a visibility fraction divides by is
not. A region 20 % inside a huge clip and a region containing a tiny
clip would then be told apart only by which way round the caller
happened to write them, and that failure is silent in exactly the way
the clip field exists to stop.

Named fields cannot be swapped by accident. They are also the
extension point: a fourth thing to report (a z-order, an "is this
enabled") adds a field rather than a breaking change to every call
site in three crates.

Not `Copy`, not stored: it borrows the freshly formatted name and
lives only for the duration of one sink call.

### `fn tab_bar`

Published separately from the stack because failure mode #8 is a
statement about what fits *inside the bar*, and asserting it against
the whole compartment's rect would pass trivially.

### `fn overflow`

**This is the rect failure mode #8 is asserted against.** At a width
where tabs are hidden, this must exist, have a positive area, and lie
within the tab bar published by [`tab_bar`].

### `fn tab_caret`

Published by the strip the caret is drawn on, and only while a drag is in
flight — so a harness reads it as a *change*: the region appears on the
first frame the caret is painted and goes when the drag ends.

It is named rather than left to the pixels because it is the half of the
gesture no screenshot taken after the fact can see, and because *"clear
markers of where it is going to move to"* is the operator's own wording for
what this feature is. A drag that reorders correctly and shows nothing has
answered the wrong half of that.

Positioned by stack rather than by panel: the caret marks a boundary in a
compartment, and a boundary belongs to the strip, not to the tab that would
land at it.

### `fn drop_zone`

Only the armed zone is published. The other four are painted and not named:
a consumer asking *"where would this land"* wants the answer, and four
rectangles that are merely offered would have to be filtered back out of
every assertion.

### `fn drop_outcome`

Carries no address on purpose: the point of the replay is that the outcome
is often a compartment that does not exist yet, and naming it by an address
from the current layout would be the naive preview wearing a region name.
See [`super::preview`].

### `fn tear_outline`

Carries no address for [`drop_outcome`]'s reason and one of its own: a torn
panel is going somewhere the dock has no addresses for, and the rectangle is
the whole of what there is to say about it.

### `fn banner`

Published only on a side that actually reserved one, so its **absence**
is the evidence that no banner was drawn. That asymmetry is deliberate: a
name published unconditionally, with a constant height, would go on
reporting a healthy rectangle for a strip the caller had stopped drawing
into — which is the shape of defect this whole stream was widened to
close.

### `fn tool_rail`

Deliberately **not** [`rail`], which names a different surface: the sliver
a *collapsed* side leaves behind as the way back. Two surfaces sharing one
trace name is how a driven check reads the wrong one — recorded in
`D:/dev/rag/egui/two_trace_lines_sharing_an_event_name_make_a_check_read_the_wrong_one.md`.

Published only on a side that actually reserved one, so its **absence** is
the evidence that no rail was drawn — [`banner`]'s asymmetry, for
[`banner`]'s reason.

### `fn rail_trigger`

Published in **both** settings, and that is what makes it the reachability
oracle. A rail that is hiding publishes `toolrail` only on the frames it is
revealed, so a check reading `toolrail` alone cannot distinguish *"the rail
is hidden and one pointer-move away"* from *"the rail is gone and the panels
it switches between are unreachable"*. This region is present on every frame
the side is drawn, at [`super::rail::PEEK_WIDTH_PTS`] wide when hidden and
[`super::rail::WIDTH_PTS`] when not, so **its absence is the defect** and its
width is the answer to "is it hittable".

### `struct Reporter`

A struct rather than a bare `Option` so the "do not format the name
unless someone is listening" rule lives in one place. Every call site
in the dock goes through it, and the rule matters here more than in
the ribbon: a dock draws a name per tab per stack per column per side
per frame, and `format!` on a hot path with nobody reading it is pure
waste.

### `fn report`

# Why this takes the `Ui` rather than a `clip: Rect`

The clip is not a parameter a call site should be *choosing*; it
is a fact about the `Ui` the region was drawn into, and the only
correct value is `ui.clip_rect()`. Passing the `Ui` makes that
the only value obtainable, which closes two holes at once:

1. **The swap.** `report(rect, clip, name)` puts two `Rect`s side
   by side with no type to tell them apart. Every one of this
   module's dozen call sites would have been one transposition
   away from a consumer computing a visibility fraction against
   the wrong denominator — silently, with plausible output. See
   [`RectReport`]'s note on the same hazard at the sink end.
2. **The stale clip.** A call site that captured a clip early and
   reported late would report a rectangle from one `Ui` against a
   clip from another. Asking the `Ui` at the moment of
   publication cannot go stale.

The cost is that this module knows about [`egui::Ui`] and not only
[`Rect`]. That names nothing outside `egui`, so R7
(`tools/gates/check-shell-purity.sh`) is untouched: a clip
rectangle is as domain-free as the rectangle beside it.

# Which `Ui` to pass

**The one whose clip the region is actually subject to**, which is
not always the one that drew the region's contents. A panel body
is drawn in a child `Ui` clipped tighter still
([`super::Dock::show`]'s `draw_stack`), but the question a
consumer asks of `dock.body.…` is *is this compartment on
screen*, so the compartment's own `Ui` is the right one to ask.
Reporting against the child's clip would compare the body
rectangle to a clip derived from itself, which is the
tautology `visible == 1.0` dressed up as a measurement.
