# `render::strategy` — whole page, or just the window?

`OPERATOR_REQUESTS.md` **O24**. One decision, made in one place, from
numbers rather than from a mode flag: **at this zoom, on this page, do we
rasterize the whole sheet or only what is on screen?**

## The constraint that shaped this, in the operator's words

> *"I don't want to lose our capability to pan around a page and still see
> high detail as we pan. I don't want the affect that other readers have
> where you always have to wait for detail to render after panning to a new
> area."*

He is describing the cost of region rendering, and he is right to refuse it
as a general answer:

| | whole page | region |
|---|---|---|
| rasterized once per | **zoom** | **position** |
| what a pan costs | nothing — the texture exists and the view moves over it | a new raster, every time |
| what he sees while panning | full detail, immediately | blur or blank until it lands |

Panning at full detail is a *property of rasterizing the whole page*, and it
is free precisely because the raster does not depend on where he is looking.
So region rendering may not simply replace it.

## The tiers, and why nothing is taken away to pay for anything

| tier | when | panning |
|---|---|---|
| [`Strategy::WholePage`] | while the page's raster fits `MAX_PIXMAP_EDGE` — **and, on a page blended in ink, while it still composites in ink** ([`Ink`]) | **free, full detail** |
| [`Strategy::Region`] | only above that | free within the overscan; a re-raster on leaving it |

**The tier he works in does not change at all.** On an A1 sheet the
whole-page raster survives to about 1,034 %, and today `MAX_ZOOM` stops him
at 800 % first — so every zoom he has ever used keeps exactly the behaviour
he has, *by construction rather than by tuning*. There is no low-zoom
performance question to answer here, because at low zoom this module returns
[`Strategy::WholePage`] and nothing downstream is different.

And the region tier only ever engages where the zoom is currently
**unavailable**. It cannot regress anything, because there is nothing there
to regress.

## What this module is NOT

It does not render, does not touch a cache, and does not know what a texture
is. It is arithmetic over four numbers, which is what lets the interesting
question — *where exactly does the switch happen, and does it move when the
window resizes?* — be answered by a unit test rather than by watching a
canvas.
