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
