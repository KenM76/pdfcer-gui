# `canvas::minreveal` — bring a rectangle into view with the least movement

`OPERATOR_REQUESTS.md` O204. A Tab press must be able to reach a form field
or a canvas object that is below the fold, and it must do so without
throwing the view around: the operator is reading a page, and the cost of
each Tab should be the smallest scroll that makes the next stop visible.

## Contract

A caller parks a [`MinReveal`] on the open document — one shot, exactly as
`find::reveal` and [`crate::canvas::destscroll`] do — naming a page and a
rectangle expressed as **fractions of that page's canvas extent**.
`crate::canvas::offset`'s ranked chain spends it on the first frame that is
drawing the named page, and [`solve_axis`] decides, per axis, between
*stay exactly where you are* and *move by the minimum*.

**Nothing here reads or writes the zoom**, and that is the difference
between this and `zoom::zoom_to_rect`: a Tab is a change of focus, not a
change of framing. A ring that re-framed the page on every press would be
unusable on a form whose fields differ in size.

## Why fractions, and why they span frames

Both for `destscroll`'s reasons. A fraction of the page extent is
independent of the zoom, so it can be recorded on the frame the ring
advances and spent on a later one; and the page a cross-page Tab lands on is
not laid out until after `Action::GoToPage` has been applied, so the
earliest frame that can solve the scroll is not the frame that asked for it.

## Why a separate solver from `destscroll`

`destscroll` moves its named axis **unconditionally** — a link is a request
for a vertical position, and refusing to move because the target happened to
be on screen reads as a broken link. Its own header carries that argument.
A Tab is the opposite: the overwhelmingly common case is the next field on
the same screen, and moving at all would be the defect. The two policies
cannot live in one function without a flag, and a flag would be a caller
choosing between two meanings of the same call.

What *is* shared is the arithmetic: the position of a page point along the
scroll content comes from [`geometry::offset_holding_anchor_at`], the same
function the zoom anchor, the find reveal and the destination scroll all
use, so no second opinion about where a page point sits can exist.

## Item notes

### `const REVEAL_GRACE_FRAMES`

Deliberately [`crate::canvas::destscroll::DEST_GRACE_FRAMES`]: both are
"wait for the page turn the action beside me raised", and a second number
would be two answers to one question. A reveal held indefinitely would be
spent minutes later, on an unrelated page change, as a view that lurches on
its own.

### `const REVEAL_MARGIN`

The same constant a fit is framed with, so "clear of the edge" means one
thing across the canvas. It is applied on both the near and the far side,
which is what stops a field that is technically visible but sitting under
the scroll bar from counting as reached.

### `fn fracs_for_canvas_rect`

Measured against the page extent rather than its drawn size, which is what
makes the value independent of the zoom and so recordable now, spendable
later.

### `fn park`

Replacing rather than queueing is the correct rule for a key the operator
holds down: three fast Tab presses are a request to be at the third stop,
and a queue would walk the view through the first two.

### `fn solve_axis`

`lo` and `hi` are content-space positions — the offsets at which the
rectangle's near and far edges would sit exactly at the start of the view.
The margin is applied outside both, so the answer is the offset at which the
rectangle *and its paper* are inside the view.

Returns `None` for *"this axis already shows it — do not move"*, which is
the answer the whole module exists to be able to give.

# The rectangle larger than the viewport

When `hi - lo + 2 * margin` exceeds `viewport` the two constraints conflict
and no offset satisfies both. The near edge wins: a field taller than the
screen is read from its start, and aligning to the far edge would put the
caret off the top of the view on the frame the operator began typing.

### `fn take_reveal_offset`

`display` is the page's drawn size and `viewport` the visible extent, both
in the units `geometry` works in; `current` is the offset the view is
sitting at, in strip space; `to_strip` is `canvas::offset`'s own page-local
→ strip conversion, handed in rather than reimplemented so the visibility
test is made in the same space as the answer.

Returns the offset the `ScrollArea` should be forced to, or `None` for
*"nothing to do"* — which covers both "no reveal parked" and "parked, and
both axes already show it". In the second case the reveal is still consumed:
it has been satisfied.
