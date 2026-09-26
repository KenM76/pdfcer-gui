# `canvas::clicking` — what a click MEANS

## The seam

Split from [`crate::canvas::interact`] under R2. It is the seam the file was
always going to split along rather than the cheapest one to reach:
`interact`'s subject is *one frame of canvas interaction* — read the pointer,
advance the gesture machine, decompose if a hit test needs it, route the
outcome, re-resolve, draw — and this is one of the outcomes it routes,
carrying a third of its lines.

[`crate::canvas::pressing`] already owns the companion question, *what would
a press land on*. This owns *what does a completed click do about it*, and
the pair is easier to reason about than either was inside `interact`.

## The whole subject is a LADDER, and its order is the design

A click is exactly one thing. Never two. The arms of [`click`] are tried in
order and the first that answers consumes it. **Twelve rungs**, and the
table is the arms of one `if` / `else if` chain — if you add one, add a row:

| # | arm | why it is here and not one rung later |
|---|---|---|
| 1 | **the Node tool** | the most specific: the operator armed a tool whose entire subject is anchors |
| 2 | **the text caret** | an armed tool owns the press — this codebase's rule everywhere |
| 3 | **a link under the pointer** | a `/Link` **is** an annotation, so following it has to outrank selecting it, or a clickable contents page is dead in Review, where `caps.author_markup` is true |
| 4 | **an annotation under the pointer** | below every armed tool, above the text fall-through. Additive: the hit is computed ahead of the ladder and the arm is an `if let`, so a miss is not a branch |
| 5 | **an image, in a mode that cannot edit** | Read and Review only, and **above** the text arm because `takes_the_press` is true for the whole canvas there — a rung below it never runs. Narrowed to images alone so a CAD sheet's paths do not swallow every click, and text under the pointer still wins inside it, because a scanned page is one edge-to-edge image with an invisible OCR layer sitting on top |
| 6 | **a text sweep** | the fall-through for Read and Review |
| 7 | **a vertex markup** (PolyLine, Polygon) | a click-built shape; mutually exclusive with 6 by armed tool, and it needs no decomposition — a vertex lands where the pointer was and hit-tests nothing |
| 8 | **a placing tool** | the click places the lower-left corner and leaves the size to the window that asked, which is what the drag does too, so the two gestures agree about what the pointer meant |
| 9 | **a form control** | the click places one at its conventional size; the drag sizes it |
| 10 | **a sticky note** | the one text annotation placed by a click rather than a drag — the dragged kinds reach the dialog through `GestureOutcome::TextAnnot` instead |
| 11 | **a measure pick** | the dimension tools |
| 12 | **content selection** | what a click meant before any of the above existed |

Two rungs carry the same warning and it is the one to read first: **a
rung placed below an arm that answers for the whole canvas never runs.**
Rung 4's position was got wrong that way once and cost the operator four
reports; rung 5's comment records the same trap being avoided deliberately,
for the second feature in a row.

A click on a comment pop-up is resolved **before** the ladder and returns,
rather than being a rung: it is a click on a floating surface the shell drew
over the page, not a click on the page.

## Item notes

### `struct CycleCursor`

# What this closes

The operator, 2026-08-26: *"when I click on one of the objects all I get is
the page selected."* The engine already computed the whole front-to-back
list of what a click is over; this shell took the first entry and discarded
the rest, so anything underneath anything was unreachable at every point,
for ever.

`Alt`+click at the same place now steps one deeper each time and wraps —
which is Illustrator's *Select Behind* (`Ctrl`+click there) and Figma's
deep-select, the two conventions for exactly this.

# Why it resets on pointer travel, and why the threshold is generous

A depth is only meaningful *at a point*: three clicks in three different
places are three first clicks, not a walk into a stack. So the cursor
remembers where it was established and resets when the pointer has moved
away from there.

[`CYCLE_RESET_PTS`] is the radius. It is deliberately larger than a pixel:
an operator holding `Alt` and clicking repeatedly does not hold the mouse
perfectly still, and a one-pixel threshold would silently restart the cycle
on the second click and make the feature look broken in the most confusing
possible way — it would work, sometimes, depending on how steady their hand
was.

### `fn cycle_depth`

`alt` is the operator asking to go deeper. Without it the cursor is reset,
so an ordinary click always lands on the front-most candidate — which is
what makes this feature invisible to anyone not using it.

### `fn text_under`

# The one question that keeps the Read-mode image arm off a scan

Added 2026-09-01 on the operator's report — *"I can't seem to copy and paste
text we have OCRed"* — hours after the image arm shipped. That arm was
narrowed to images because a CAD sheet has a path under the pointer almost
everywhere and allowing paths would have made text unreachable. The case it
did not anticipate is the one where the narrowing does not help at all: **a
scanned page IS one image**, edge to edge, so every click hits it, and an OCR
layer is invisible text lying exactly on top of it.

⇒ The document class where selecting text matters most was the one where the
arm swallowed it.

## Asked of the SAME `PageText` the sweep would use

Not of a second extraction and not of a cached copy taken elsewhere. Two
extractions under two configurations segment differently, so a second
opinion here would produce a click that takes the image and a drag that
takes text — from one pixel, with nothing on screen to explain it.

## Absent page text answers `false`, which yields to the image

That is the honest direction. `page_text` is `None` before the extraction has
run for this page; treating "I do not know yet" as "there is text here" would
make the image unclickable for the first frames after a page turn, which is
the flicker an operator reports as *"sometimes it does not work"*.
