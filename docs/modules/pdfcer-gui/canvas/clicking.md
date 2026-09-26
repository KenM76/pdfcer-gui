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
