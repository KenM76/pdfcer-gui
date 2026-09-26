# `find::reveal` — bringing a hit onto the screen

The second half of what "go to this hit" means. The first half is a page
change, which [`super::apply`] performs directly through
[`crate::viewer::ViewState::go_to_page`]; this module is everything after
that — the pending intent, the gate that decides when to spend it, the
scroll solve, and the projection from PDF geometry into the space the
canvas paints in.

## Why it takes two frames, and why that is not avoidable

The request is made during the **apply phase**, which runs *after* the
canvas has drawn ([`crate::app`]'s frame order, step 3). The page change
it carries is applied in the same phase. So the earliest frame on which
the target page's real drawn size is known — and the drawn size is what a
scroll offset is solved against — is the **next** one.

That is exactly the situation [`crate::app::state::ZoomAnchor`] documents
for a zoom, and the shape of the answer is the same: *record the inputs
now, solve later*. [`Reveal`] is therefore a fraction of the page rather
than a canvas point, because a fraction is independent of the zoom and
survives the operator zooming in between the two frames.

## Why it cannot simply ride the zoom anchor's handshake

Because that handshake is gated on the page's **drawn size changing** —
`zoom::anchor_step` compares `display_now` against the size recorded when
the anchor was armed, and treats "unchanged" as *the zoom has not landed
yet*. A find reveal changes the scroll offset and nothing else, so under
that gate it would `Hold` for one frame and then `Drop`, every time,
having moved nothing. A search that navigated to a hit and then did not
scroll to it is precisely how a plausible implementation of this feature
ships doing nothing.

So the gate is different — **the page index**, not the drawn size — and
the *solve* is shared: [`take_reveal_offset`] asks
[`crate::canvas::geometry::offset_holding_anchor_at`], which is the single
owner of *"put this page point at this screen position"* and is the same
function `canvas::zoom::place_centred` uses to centre a framing zoom. One
arithmetic, two callers, two gates.

## Why this is a module rather than a section of [`super`]

Rule R2's 1,500-line ceiling requires the split, and the seam it forces is
a real one: [`super`] answers *what is a search, and what does its answer
mean* — the query, the options, the wildcard trap, staleness, the readout
— while this file answers *how does one hit get in front of the operator*.
The two change for different reasons and are read at different times, and
the coordinate-space reasoning below has nothing to do with the search
semantics above.
