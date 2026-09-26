# `canvas::destscroll` — a destination that named a POINT scrolls, it does not zoom

`OPERATOR_REQUESTS.md` O200, `DEFECTS.md` D47:

> *"…links in word documents saved as pdfs such as table of contents …
> should just jump the position on the page pointed to without changing the
> zoom. If the position jumped to is visible on the page with the current
> horizontal position of the page, the horizontal position shouldn't be
> changed."*

## Contract

[`crate::canvas::destination::arrive`] parks a [`DestScroll`] on the document
when a `/XYZ`-family destination resolves; `crate::canvas::offset`'s ranked
chain spends it, once, on the first frame that is drawing the named page.
**Nothing here reads or writes the zoom.** A `/XYZ` that carried an explicit
magnification still gets one, because `app::actions::destination` raises a
separate `Action::ZoomTo` that lands a frame earlier — so the scroll is
solved against the size that zoom produced.

Rectangle destinations (`/FitR`, and every SolidWorks drawing bookmark
measured in the operator's own packages) do not come here at all: they keep
travelling through `zoom::zoom_to_rect`, so O154's framing is untouched.

## Why a second solver, when `destination` argues against one

Framing and scrolling answer two different questions — *"what magnification
shows this region?"* and *"where must the view sit for this point to be at
the top-left?"*. The risk the old argument named, two positions that drift,
is answered by construction instead: the position itself comes from
[`geometry::offset_holding_anchor_at`], shared with the zoom anchor and with
`find::reveal`. All this module decides is **which screen position to ask
for, per axis**.

The parked-fraction shape is `find::reveal`'s, for its reason: a scroll that
changes no magnification cannot ride `zoom::AnchorStep`'s handshake, because
that handshake is gated on the page's drawn size changing.

## Item notes

### `const VISIBLE_CLEARANCE`

A point exactly on the boundary is technically visible and practically is
not — it is under the scroll bar, or half of it is. Deliberately the same
number as [`DEST_EDGE_MARGIN`]: "clear of the edge" and "inset from the
edge" are one idea, and a second constant would let them drift.

### `fn a_point_destination_can_only_answer_with_a_position`

The mechanism this replaced grew the point into a 150 pt square and
handed it to the framing solver, which answered with a zoom. That
cannot be expressed here, which is the point: the type is the enforcer.

### `fn a_null_horizontal_defers_to_whatever_placed_the_view_this_frame`

`/FitH` names no left edge and raises `Action::Fit(Width)` beside the
scroll; that fit outranks this in `canvas::offset` and has already
placed the horizontal. Holding at `origin_x` here would undo it one
frame later, which is D47's own failure shape wearing a different hat.

### `struct DestScroll`

Spans frames for the same reason `find_reveal` and `zoom_anchor` do: the
page change is applied after the canvas has drawn, so the earliest frame on
which the target page's drawn size is known is a later one.

### `fn fracs_for`

# Why the axes are probed rather than assumed

`left` and `top` are PDF user space; the canvas is the page as drawn, with
`/Rotate` applied. The probe transforms a second point displaced along PDF
*x*: whether that displacement comes out mostly along canvas *x* or mostly
along canvas *y* is the answer, and it is right for every rotation without
this module reading `/Rotate` at all.

The value substituted for a null axis never reaches the result — that axis's
fraction is dropped by the `map` below.

### `fn axis_stays_where_it_is`

`point` and `current` are both content-space — the position of the
destination along the scroll content, and the offset the view sits at.

# Why it is applied to the horizontal and not to both

A link is overwhelmingly a request for a vertical position: a heading, a
sheet, a paragraph. Applying this test vertically would make a link to a
heading half a screen below the current one do nothing at all, which reads
as a broken link. Horizontally there is no such expectation, and moving
sideways for a point already on screen is the lurch being reported.

### `fn solve`

Separated from [`take_dest_scroll_offset`] so that the rule has an enforcer
that does not need a window: everything above this line is geometry shared
with the zoom anchor and the find reveal, and everything the operator
actually asked for is the choice made here.

# Arguments

* `point` — where the destination sits along the content: the offset that
  would put it at screen position zero.
* `want` — the offset that puts it [`DEST_EDGE_MARGIN`] in from the
  top-left corner. Used only on an axis that moves.
* `current` — the offset the view is sitting at this frame, after whatever
  outranked this in `canvas::offset` has had its say.

# The two answers to "hold this axis still"

* A **null** axis is §12.3.2.2's *"leave this one as it is"*, and "as it
  is" means whatever won this frame's ranking — a fit-width's placement,
  say. Held at `current`, so this cannot undo it.
* A **specified** horizontal axis already on screen is the operator's own
  position, and that is [`DestScroll::origin_x`]: the offset before the page
  turn, not the strip's horizontal centring of the page it turned to.

### `fn take_dest_scroll_offset`

# Arguments that are not obvious

* `current` — the offset the view is sitting at this frame, in content
  space. What a NULL axis is held at; see [`solve`] for why a specified
  horizontal axis is held at [`DestScroll::origin_x`] instead.
* `to_strip` — the caller's page-local ⇒ content-space conversion. Passed in
  because a continuous strip's conversion needs the strip layout, the
  pasteboard overhang and the row rect, none of which belong here.

# The probe for "where is the point?", and what its clamp costs

`to_strip` clamps to the scrollable range, so near either end of the content
the probe reports the clamp rather than the point. That degrades safely: in
every clamped case the probe lands at the limit of the range, the visibility
test therefore answers *not visible*, and the offset the axis is then moved
to is that same limit — so the view does not move.
