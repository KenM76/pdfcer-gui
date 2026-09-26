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
