# `canvas::grid` — the drawing grid, in the page's own space

The middle of `RIBBON_IA.md` §5.2's *"Rulers · Grid · Guides"*, split out of
[`super::rulers`] when that file reached R2's 1,500-line ceiling. The seam
is the one that module's header already implied: a ruler is chrome **beside**
the canvas that reserves layout space and answers to R128, while a grid is
chrome **over the page** that reserves nothing and answers to a different
question entirely.

What stays behind in [`super::rulers`] is everything the two share — the
unit ([`Scale`]), the 1-2-5 [`Ladder`] and its exact tick walk — because a
grid drawn on a different ladder from the ruler beside it would be two
ornaments rather than one reading.

## ★ Which space the grid is drawn in — page space, per page

[`super::rulers`]' header §2 carries the argument in full and it is the
decision this module exists to enact, so the short form is here:

Under a continuous mode several pages are on screen at once. **A
viewport-space grid** is anchored to the window, so scrolling slides it
across the paper: a line that sat on an intersection comes off it, and the
same feature on two sheets falls at two different places in the grid. It is
wallpaper, not a reference — and it is the cheaper and easier one to write,
which is why it is the one to be careful about.

**A page-space grid** is drawn per page, anchored to that page's own
top-left corner and clipped to that page's rectangle. It scrolls *with* the
sheet, so an intersection is a fixed place on the drawing; every sheet in a
set gets the same grid in the same place relative to its own border; and
the row gaps between pages carry no grid, which is truthful — there is no
paper there to be ruled.

pdfcer draws the second, because of what a grid is *for*. A drafter uses one
to judge alignment and spacing **on the drawing**, and every answer read off
it is a statement about the sheet. A grid not attached to the sheet cannot
make such a statement. The same argument settles the guides, which is why
[`super::guides`] stores a guide against a **page**.

## ★ Rule 4

A grid the operator switched on is chrome they asked for, and `panels`'
one-line test — *would a screenshot of the editing canvas differ from a
screenshot of the same document saved and reopened?* — answers **yes,
because they asked**. Nothing here is keyed on any property of the content:
the spacing comes from the zoom and the document's stated scale, never from
what is on the sheet, and it vanishes the instant the toggle goes off.

The version that would fail the test is a grid that **snapped to something
pdfcer found** — a detected drawing frame, an inferred module size. That is
an inference, an inference owes an off-canvas report, and there is no such
code path here.
