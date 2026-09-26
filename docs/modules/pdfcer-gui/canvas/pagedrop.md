# `canvas::pagedrop` — dropping pages onto the page view


> *"…drag and drop pages from one thumbnail image sidebar to another **or
> onto the canvas** to add pages and insert them **in between the pages
> we've dragged to** on the canvas or the thumbnail preview area."*

The Pages panel's grid already knew how to resolve a gap and draw a caret;
this is the same gesture with the same vocabulary, aimed at the page view.
[`crate::pagedrag`] holds the drag itself, which is what lets a gesture that
*began* in a panel — possibly in another document — end here.

---

## 1. Where a gap is, on a page view

The strip lays pages out in **rows**, top to bottom, in every display mode
([`crate::viewer::strip`]). So a boundary between two pages is a horizontal
line, and the question *"which boundary is the operator aiming at?"* is
answered by the **vertical half** of the page under the pointer: the top
half means *before this page*, the bottom half means *after it*.

That is deliberately the same shape as the Pages panel's rule, rotated
ninety degrees to match the flow of the surface it is on — the grid flows
left to right within a row, so it splits on the horizontal half and draws a
vertical caret; the page view flows top to bottom, so it splits on the
vertical half and draws a horizontal one. An operator who has used one
knows the other without being told, which is the property worth having.

### The facing modes get the same rule, and it is right there too

Under `PageDisplay::Facing` a row holds two pages side by side, so the
left/right split would also carry meaning. It is **not** used, and the
reason is not laziness: *before the right-hand page of a spread* and *after
the left-hand page of a spread* are the same boundary, so a horizontal
split would offer the operator two ways to name one gap and no way to name
the gap between rows. The vertical rule names every boundary exactly once.

## 2. Single-page mode is not a special case

It is a one-row strip ([`crate::viewer::strip`]'s header says why that is
load-bearing), so the same rule applies unchanged: the top half of the sheet
on screen means *before this sheet*, the bottom half means *after it*. An
operator who never turns continuous scroll on can still drag a sheet out of
another drawing and drop it in front of the one they are reading.

## 3. Rule 4 — the caret is the cursor, and nothing here marks content

`panels::pages::paint_caret`'s argument, verbatim and for the same surface:
*"snap indicators, hover highlights, rubber-bands and selection handles are
the cursor and are welcome"*. This draws one line, over the gap, while a
button is held, and it is gone the instant it is released. It tints no page,
badges nothing, and adds no second rendering path — the one-line test is
that a screenshot of this canvas with a drag in flight differs from one of
the same document saved and reopened only by where the pointer is.

The words half of the disclosure lives off-canvas, in the status row and in
the Pages panel's header, exactly as rule 4 requires.

## 4. What it does NOT do

**It does not accept files dropped from Explorer.** That is
`crate::app::dropped`, which reads `egui`'s window-level `dropped_files` and
is a different mechanism with a different operand. Both can be in flight at
once and they do not interact; this one is only ever about pages already
open in pdfcer.

## Item notes

### `const CARET_PTS`

`panels::pages`' `CARET_PTS`, restated rather than imported, because the two
are the same *number* and not the same *decision*: this one is measured
against a rendered page at the operator's zoom and that one against a
thumbnail tile. If a future zoom-aware caret makes one of them change, the
other must not follow by accident.

### `const CARET_INSET_PTS`

Enough to read as *"in the gap"* rather than as a border the sheet has
grown, which is the same reason the grid's caret sits half the inter-tile
spacing beyond the tile.

### `const CARET_DIMMED`

`panels::pages::CARET_DIMMED`, and the argument travels with it: **dimmed,
not hidden**, because drawing nothing over a boundary that would not land
cannot be told apart from the canvas having stopped tracking the pointer —
and the no-op boundary is where every same-document drag begins.

### `fn resolve`

`None` when the pointer is outside the page view, or over no page — the
margin around a page that is smaller than the viewport is not a gap, and
treating it as one would let a drop land somewhere the operator was not
pointing.

### `fn offer`

Called once per frame from [`crate::canvas::show_in`], after the scroll area
has closed and therefore after every visible page's screen rectangle is
known. Does nothing at all — not one branch past the first — when no page
drag is in flight, which is every frame but the handful the operator is
carrying something.

`drawn` is the visible pages with the rectangles they were actually drawn
in, which is what makes the gap resolution exact rather than reconstructed.
`D:\dev\rag\egui` records the rule this obeys: **do not compute a coordinate
the application could publish** — a harness, or a second piece of the
application, that derives a widget position by arithmetic can be wrong in
the same direction as the code under test.
