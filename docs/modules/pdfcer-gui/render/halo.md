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
