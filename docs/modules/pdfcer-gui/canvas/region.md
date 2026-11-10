# `canvas::region` — what the snapshot region takes from its page

The region is the snapshot box (`OpenDoc::snapshot`): one rect, on one page,
in PDF user space. It is the shared subject of Copy, Delete in region, Redact,
Redact outside and Crop to region.

## What the region takes

The same hit test the Select tool's marquee runs (`marquee::select_with`),
run twice on fresh selections: once enclosing (`crossing = false`) and once
touching (`crossing = true`). The difference between the two is the
*crossing* count, the objects that straddle the edge.

The standing choice **Delete takes: Inside only | Touching** decides which
band is the answer. Inside only is the default because it is the narrower
answer and cannot surprise. The Select tool's marquee picks by drag direction;
the region does not, because the box is moved and resized after it is drawn
and the direction it was dragged in stops meaning anything.

## The selection follows the box

When a box is drawn, moved or resized, `fill_selection` makes what it takes
the canvas selection, so the properties panel, Delete and Copy all see it.
The region commands do NOT read the selection back: they recompute from the
box, so Shift/Ctrl adjusting the selection afterwards changes the selection
alone.

## Limits

Only the page on screen has its object model loaded
(`OpenDoc::page_objects`). A region on another page takes nothing, and Delete
in region says so on the status line rather than doing nothing silently.
