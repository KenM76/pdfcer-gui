# `render::halo` — rasterizing the ground OUTSIDE the sheet

## The report, and which half of it this is


> *"also objects should still be reachable even if they are off the page."*

and, three weeks later, when the reach half had shipped and this one had
not:

> *"how do I view and edit objects that are off of the page? we added this
> feature but I didn't see how to enable it."*

O23 broke into three pieces and they landed in this order:

| part | what it gave him | where it lives |
|---|---|---|
| A — **scroll** | slack on every side of the strip, so the grey can be reached | [`crate::canvas::geometry::content_extent`] |
| B1 — **reach** | a press in that grey becomes a gesture, so an off-page object can be selected and dragged | [`crate::canvas::pasteboard`] |
| B2 — **see** | the off-page object is actually painted | **this module** |

Until B2 the operator could select an object he could not see and watch its
properties change — honest, and useless. B1's own header says so verbatim so
that nobody reading it mistakes reach for sight.

## Why the object was invisible, in one sentence

`pdfcer_render::render_page` sizes its pixmap to the page's `/CropBox`.
Nothing culls the *content* — [`crate::render::offpage`] proves that against
the pinned engine — the raster simply has no pixels out there to put it in.

So the whole of B2 is: **ask for a bigger box**. `render_page_region` takes
an arbitrary page-space rectangle and never intersects it with the crop box,
which `render::offpage`'s three tests assert directly because the engine's
own suite has never exercised a region outside the page.

## The box, and the two things it is NOT

[`region`] returns the **crop box unioned with the drawn content's bounding
box** — in PDF user space, which is the space `render_page_region` and
[`crate::render::region::PageFrame`] both already speak, so `/Rotate` needs
no special case here at all.

It is **not the visible rectangle**. That is the *other* region tier —
O24's, which engages above the pixmap ceiling and re-rasterizes whenever a
pan crosses a quantisation grid line. A halo raster is a picture of the
whole page plus its overhang, exactly as a whole-page raster is a picture of
the whole page, so **panning stays free**: the texture is cached under the
same key machinery, the operator scrolls out into the grey, and the picture
is already there.

It is **not overscanned or quantised**.
[`crate::render::strategy::region_for`] grows and snaps the visible rect
because the visible rect moves continuously; this box moves only when the
document is edited, and growing it would make the raster bigger than it has
to be for no cache benefit at all.

## The ceiling, and why exceeding it returns `None` rather than a clamp

A halo box is at least as large as the crop box and can be far larger — an
object dragged 5,000 pt off a 200 pt page makes it 26 times the sheet. Past
`pdfcer_render::MAX_PIXMAP_EDGE` the engine refuses outright, and the shell
must not ask.

The answer is [`None`], which means *"there is no halo tier for this page at
this zoom"* and lets `canvas::present` fall through to O24's visible-region
tier — which covers whatever of the grey is actually **on screen**, because
[`reach`] stopped that tier clipping its visible rect to the sheet. So the
two tiers compose: below the ceiling one cached raster covers everything;
above it, the viewport-sized raster covers what is being looked at. There is
no zoom at which off-page content disappears, which matters because the zoom
an operator uses to *edit* an off-page object is exactly the deep one.

A clamp would have been the wrong answer for the reason
[`crate::render::region::region_on_screen`]'s header already gives about a
different rectangle: shrinking the box without telling the destination is
how the right pixels end up in the wrong place.

## The tolerance, and the case it deliberately drops

Content bounding boxes poke a hair outside the crop box all the time — half
a stroke width on a border line is enough, and a CAD title block draws one
on every sheet. Flipping every such page into a bigger raster would cost
every operator memory and time to show nothing. So an overhang under
[`OVERHANG_TOLERANCE_PTS`] is not a halo.

The case that drops is an object deliberately placed less than a point
outside the sheet. It stays invisible, exactly as it is today, and it is
**already unreachable by eye** at any zoom where a point is less than a
pixel. Stated here rather than left to be discovered.

## Rule 15

Every number in this module is a **pdf dimension** — a coordinate in the
CAD-exported page's own user space. None of it is a **ce dimension**;
nothing here authors anything.

## Item notes

### `fn finite`

Its own function because `Bounds::EMPTY` is built from infinities on
purpose — see [`region`]'s case 1 — so "is this box real?" is a question
this module asks twice and must answer the same way both times.

### `const OVERHANG_TOLERANCE_PTS`

See the module header. One point is about a stroke's half-width on a heavy
border and about 1/72 inch — far below anything an operator could see, and
far below anything he would have placed on purpose.

### `fn region`

* `crop` — the page's crop box, in PDF user space.
* `content` — the drawn content's bounding box in the same space, as
  `PageObjects::page_bbox` reports it. [`None`] when the page has not been
  decomposed, which is the ordinary state of a strip neighbour.
* `raster_scale` — the device scale the whole-page tier would render at.

Returns [`None`] in three distinct situations, and they are three different
facts about the page rather than three spellings of "no":

1. **nothing is known** — `content` is `None` or empty. Not "there is no
   off-page content"; *"nobody has looked"*. The caller must not read this
   as a guarantee.
2. **nothing hangs over** — the union is the crop box, to within
   [`OVERHANG_TOLERANCE_PTS`]. The overwhelming majority of pages, and the
   reason this whole module costs a normal document nothing.
3. **it would not fit** — see the module header; O24's visible-region tier
   takes over.

### `fn reach`

O24's visible-region tier asks *"what part of this page can be seen?"* and
answered it, until this function existed, with `visible.intersect(page)` —
which is correct as a statement about the **sheet** and wrong as a statement
about the **page's content**. It is what made an off-page object invisible
at every zoom above the pixmap ceiling even after [`region`] had covered
every zoom below it.

* `place` — where the sheet is on screen.
* `extent` — the page's canvas extent in points, as
  [`crate::viewer::page_extent_pts`] reports it (rotation already resolved).
  `place` was laid out from this, so the two scales come from the pair and
  not from the crop box — a page whose extent rounded is still placed
  exactly on itself, which is
  [`crate::render::region::region_on_screen`]'s rule and must stay one rule.
* `frame` — the page's crop box and `/Rotate`.
* `content` — the drawn content's bounding box in PDF user space, or
  [`None`] when nobody has looked.

Returns `place` unchanged whenever there is nothing to add, so the caller
has no branch and the ordinary page keeps exactly today's behaviour.

## Why this takes the content box and not [`region`]'s answer

[`region`] returns [`None`] above the pixmap ceiling — which is precisely
when this function matters. Feeding it here would switch the visible-region
tier's reach off at the one zoom it is the only tier left.

### `fn overhang`

Returns `(x, y)`, each the **larger** of the two sides on that axis and
never negative. `(0.0, 0.0)` whenever nothing is known or nothing hangs
over, so a caller has no branch and the ordinary page keeps exactly today's
behaviour.

* `extent` — the page's canvas extent in points, as
  [`crate::viewer::page_extent_pts`] reports it (`/Rotate` resolved).
* `frame` — the page's crop box and `/Rotate`.
* `content` — the drawn content's bounding box in PDF user space, or
  [`None`] when nobody has decomposed the page. [`None`] is *"nobody has
  looked"*, not *"there is nothing out there"*, and the caller must not read
  the resulting zero as a guarantee — see
  [`crate::app::cache`]'s `content_bounds_if_known`, which peeks rather than
  builds precisely so the canvas can run this every frame.

# Why the canvas needs this and [`reach`] would not do

[`reach`] answers *"what rectangle on screen may I paint into"*, and it
needs the page's **placement** to answer, which is only known **after** the
scroll area has laid out. The scroll area's own content size has to be
decided **before** it lays out. Feeding it `reach` would be a frame late and
would be R128's feedback loop besides — a content size that depends on where
the content was put.

So this states the same fact one step earlier in the frame, in canvas points
rather than screen pixels, and the caller multiplies by the zoom. Both
functions take the box from [`crate::render::region::PageFrame::canvas_box_of`],
which is the single place `/Rotate` is resolved — O174's rule, and the
reason neither of them maps corners by hand.

# What it is for, stated as the defect it removes


`content_extent` puts **one viewport of slack** on every side of the strip,
and a viewport is a count of **screen pixels**. So the pasteboard is a fixed
number of pixels wide at every zoom, and the region of the *drawing* it
covers shrinks in exact proportion to the zoom. An object 100 pt off the
left edge of the sheet can be brought to the middle of a 470 px-wide canvas
only while `235 / zoom ≥ 100` — that is, **only below about 235 %**. Above
it the operator can see the object, select it and drag it, and cannot zoom
in on it: every notch walks it back toward the edge of the screen and then
off it, because the scroll offset has hit a clamp that the anchor solve
knows nothing about.

The same arithmetic is what made the driven check fail. It panned to a point
3.6 pt below the bottom of the sheet and zoomed; at 7,683 % the view was
274 px into a 578 px pasteboard, at 9,384 % it was against the clamp at
289 px, and the page point under the viewport centre slid 0.48 pt. The zoom
anchor was solving correctly and [`crate::canvas::geometry::strip_offset`]
was clamping its answer away.

With the overhang folded into the pasteboard the slack stops being a count
of pixels and becomes a region of the drawing plus half a screen, so every
point of the content can be brought to the centre of the viewport at **any**
zoom. That is [`crate::canvas::geometry::content_extent`]'s rule; this
function only supplies the number it needs.
