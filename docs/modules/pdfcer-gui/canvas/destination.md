# `canvas::destination` — **arriving where a bookmark points**

Operator report, 2026-09-01: *"in Acrobat clicking on the nested bookmarks
in the drawing package takes you to a zoomed in area of the page … when we
click on ours it just jumps us to the correct page, but doesn't send us to
the spot on the page the bookmark actually points to."*

Two halves, in two places, and this is the second:

| | where |
|---|---|
| what a destination MEANS — the five `/XYZ`-family views | `app::actions::destination` |
| where the view actually LANDS — viewport, margin, zoom ceiling | here |

## ★★★ Why the landing cannot happen in the apply phase

Arriving needs the canvas rectangle, the page's drawn extent and the scroll
offset. None of those exists where actions are applied, so the action parks
a [`crate::app::state::PendingDestination`] and this drains it on the next
frame — `OpenDoc::fit_placement`'s own pattern, for its own reason.

## ★★ It frames through `zoom::zoom_to_rect`, which is the zoom marquee's
## own code

Deliberately, and it is what makes the fix trustworthy rather than merely
close: a bookmark and a rubber band drawn over the same region arrive
**identically**. Two framings that agreed approximately would drift, and the
drift would present as a bookmark that lands *nearly* right — harder to
diagnose than one that does not move at all.

## ★★★ And it must not frame until the canvas is drawing the right page


[`arrive_step`] is the gate, and it carries the whole argument — including
why the repair is a bounded wait on the *destination* path rather than a
change to the framing every zoom in the product shares.

## ★ A rectangle is framed; a point is scrolled to

`/FitR` asks for a magnification and gets one, through `zoom_to_rect`.
`/XYZ` and the two one-axis fits ask for a *position*, and go to
[`crate::canvas::destscroll`] instead — this module parks the scroll, it
does not solve it. O200 / D47 is the report that separated the two, and
`destscroll`'s header carries the argument.
