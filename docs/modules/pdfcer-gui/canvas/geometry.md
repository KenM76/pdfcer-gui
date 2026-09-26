# `pdfcer-gui/canvas/geometry`

## Item notes

### `const PASTEBOARD_FRACTION`

**A whole viewport is also, exactly, the placement at which the sheet
stops being visible at all** — and that is what O186 turned out to be. The
fraction is NOT reduced to fix it: it is still one viewport, because the
operator's sentence is still the rule. [`MIN_SHEET_ON_SCREEN`] is
subtracted from it instead, so the page corner arrives at the opposite
corner and **can be seen there**, which is what he was asking for in the
first place. See that constant for the measurement.

### `fn sheet_sliver`

[`MIN_SHEET_ON_SCREEN`] normally, and **half the viewport** on a canvas
narrower than twice it. The second case is not hypothetical — a docked
panel can be dragged down to a few points, and a frame measured before
layout reports a viewport of zero — and the `min` is what keeps the
pasteboard non-negative there without a separate guard. At `viewport = 1.0`
the pasteboard becomes `0.5`: still a pasteboard, still positive, and the
interval [`visible_origin_range`] returns is still non-inverted, which is
the property the rest of this module is entitled to assume.

Deliberately a function rather than an expression inlined at its one call
site, so the tests can measure the rule directly instead of inferring it
from a pasteboard that has already had the overhang branch applied to it.

### `fn pan_offset`

# Why the clamp is not optional


# Known limitation, deliberately left

This clamps to the PAGE, so the page edges cannot be dragged inward past the
viewport edge. The operator asked to "navigate beyond the page's edges",
which needs reserved space around the page rather than a different clamp —
a change to how the canvas reserves its content area, with a visible
consequence (scrollbars present at every zoom). That is a UX call, and this
function is the one place it would need to change.

### `fn content_extent`

`overhang` is [`pasteboard`]'s, in logical points — see it for the whole
argument. Pass `0.0` for *"no page content hangs off the sheet, or nobody
has looked yet"*; that is the original behaviour exactly.

### `fn strip_margin`

Not the same function as [`margin`], and must not become it. Two
offset spaces exist and only one is padded — the scroll offset egui is
given is measured from the content's origin, the page-local offset the
view stores is measured from the page's. [`strip_offset`] and
[`page_local_offset`] therefore call **one of each**; using the same
margin for both makes the pad cancel and a stored offset of zero scrolls
to blank paper.

`anchor_screen_pos` and `offset_holding_anchor_at` look like scroll-space
functions and are **page-local** — `canvas::mod` converts before building
the `CanvasFrame` — so they keep [`margin`]. Padding them doubles the pad.

`pub` since O26g, because `canvas::show` must place the strip from it
**symbolically** rather than by subtracting two large rectangles. See
[`strip_origin_offset`].

### `fn scroll_to_strip`

Its absence was O23's whole failure, through three attempts. The
canvas builds its visible-region rect from `last_scroll_offset`, which is a
**content-space** offset, and then intersects it with the strip's own
layout. Before the pasteboard those were the same space and the omission
was invisible. With one, the rect lands a whole pasteboard past the end of
the strip, `layout.visible()` returns nothing, and the application draws
**no canvas at all** — it says so itself, as
`canvas-unavailable reason=nothing-visible`.

Every symptom chased for three attempts followed from that one line: no
pointer input, because there was no canvas to point at; a page rect that
looked correct, because it was published before the region went; and
`drawn=0`, because nothing was visible to raster.

### `fn offset_holding_anchor_at`

**Unclamped, and that is the point.** Two callers need the raw solve for
two different reasons and a clamp inside here would spoil both:

* [`zoom_anchor_offset`] applies the scrollable-range clamp *itself*, after
  composing this with [`anchor_screen_pos`], because the clamp belongs to
  the offset that is actually handed to a `ScrollArea` and not to an
  intermediate;
* [`crate::canvas::zoom`] uses it to *fabricate a before-state* — "the
  offset at which the anchor would have been sitting where we want it to
  end up" — which is a hypothetical, not a scroll position, and clamping a
  hypothetical into the current page's range would quietly change the
  framing it describes.

A non-finite axis yields `0.0` on that axis. There is no honest answer to
"where would the offset have been" when one of the inputs is NaN, and `0.0`
is the one value guaranteed to be a legal scroll offset for any page — the
same "fail to a finite, harmless value" discipline `viewer` applies to a
degenerate zoom.

### `fn fit_placement_offset`

# The report, and why a fit is now a position as well as a scale

> *"If I press the Fit width or fit page button the view should center to
> the width as well or center the page."*

Before O23's pasteboard a page no larger than the viewport had nowhere to
be except the middle, so *fit* and *centred* were the same act and the
button never had to choose. The pasteboard added a whole viewport of slack
on every side, and with it the state the operator is describing: the scale
is right and the page is not on screen.

# The rule, per axis

* **Pinned** — the fit has just decided this axis's extent, so there is one
  honest position for it and the answer is **zero**. Zero is not an
  arbitrary choice: [`anchor_screen_pos`] at `frac = 0` places the page's
  top-left at `margin - offset` from the viewport's, and [`margin`] is
  *half the slack when the page is smaller than the viewport and exactly
  zero once it is larger*. So a page-local offset of zero means **centred
  if it fits, flush if it does not** — which is fit-page's answer on both
  axes and fit-width's on the horizontal, without a special case for
  either.
* **Unpinned** — the operator is still navigating this axis, so their
  position is *kept*, clamped to the page's own range `0 ..= display -
  viewport`. Keeping it is why "Fit width" on page twelve of a drawing set
  does not throw them back to the top of the sheet; clamping it is what
  stops "kept" meaning "still looking at pasteboard".

The clamp collapses to `[0, 0]` whenever the page is no larger than the
viewport on that axis — the fit-page case, and the landscape-sheet case —
and `0` is centred there, so the two rules agree at the boundary rather
than fighting over it.

`current` is the page-local offset the view is at now. Non-finite input
yields the pinned answer, because a `NaN` position is not one worth
preserving.

### `fn page_local_offset`

`page_origin` is the current page's top-left in strip space (from
[`crate::viewer::strip::Strip::rect_of`]); `strip` is the strip's whole
drawn size; `page_display` is the current page's drawn size. Under
[`crate::viewer::PageDisplay::Single`] the origin is `(0,0)` and `strip`
equals `page_display`, so this is the identity — which is the mechanical
form of "the single-page path is untouched", asserted by
[`tests::the_strip_bridge_is_the_identity_for_a_single_page`].

Not clamped, deliberately: the result is fed to solves that do their own
clamping ([`zoom_anchor_offset`]) or that are measuring a hypothetical
(`offset_holding_anchor_at`), and a clamp here would quietly change what
they were asked.

### `fn strip_offset`

Clamped to the strip's scrollable range, because *this* is the value that
is handed to a `ScrollArea` — the same division of labour
[`offset_holding_anchor_at`] and [`zoom_anchor_offset`] already observe
between them, where the raw solve is unclamped and the offset that actually
reaches the widget is not.

### `fn pdf_rect_to_canvas`

Both corners go through [`crate::viewer::pdf_space_to_canvas`], the one
bridge between the two spaces, rather than through a local flip. PDF is
y-up and canvas is y-down, so a hand-rolled conversion mirrors the
rectangle about the page centre when it is wrong — and a mirrored
destination lands plausibly on the wrong half of the sheet, which reads as
"the bookmark is broken" rather than as an arithmetic error.

`None` on a page whose device geometry will not invert, which is the same
condition every other coordinate hop here declines on.
