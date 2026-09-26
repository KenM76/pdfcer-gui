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

## Which space the grid is drawn in — page space, per page

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

## Rule 4

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

## Item notes

### `const MIN_GRID_PITCH_PTS`

The grid uses the same ladder as the ruler but is allowed to be far denser,
because a grid line carries no text and its whole purpose is to be fine
enough to judge alignment against. 8 points is about where a hairline grid
stops reading as a grid and starts reading as a tint.

This is also the **line-count bound**, and the bound is what makes a
page-space grid affordable on a 129,758-object A3 sheet: at 8 points'
minimum pitch a 2,000-point-wide viewport holds at most 250 vertical lines,
whatever the page's size and whatever the zoom.

### `const GRID_MINOR_ALPHA`

Low, because a grid is a reference the operator looks *through*. The
standard is the one `overlay`'s find highlight records after a screenshot
corrected it: the operator's next act is to read the drawing, and chrome
that competes with the drawing has defeated its own purpose. On a dense CAD
sheet a grid at even a quarter opacity turns black linework grey.

### `const GRID_MAJOR_ALPHA`

Two weights rather than one, and the ratio matters more than either number:
a uniform grid gives the eye nothing to count by, so judging "how far is
that" means counting hairlines one at a time. Every drafting grid in every
CAD package does this, and the heavy lines are the ones that line up with
the ruler's numbers.

### `fn lines`

Named for what it draws — a run of lines — rather than for the axis it is
given, so the parameter and the function do not share a name.

`Axis::X` produces **vertical** lines, spaced along the page's x. See
[`Axis`] on why one enum carries both readings.

### `fn every_grid_line_keeps_a_usable_pitch_at_every_zoom`

The bound applies to the **minor** step, because a minor line is a line
that gets drawn. The version that bounded the *major* chose a 1-point
grid on the benchmark A3 sheet — a line every 1.4 screen pixels, which
is a tint rather than a grid, and ten times the shape count. It passed
the old form of this test, which asserted only that the grid was finer
than the ruler.

The upper bound is asserted too. A grid whose lines drift to 80 points
apart has stopped being something to judge alignment against, and the
climb in [`Ladder::for_lines`] is exactly the code that could overshoot.

### `fn every_ruler_label_has_a_grid_line_under_it`

The coincidence is the whole reason to ship a ruler and a grid rather
than two independent ornaments: a feature sitting on a grid line can be
read off the ruler without counting.

The claim is deliberately *a grid line*, not *a heavy grid line*. Both
ladders are 1-2-5 numbers and a 1-2-5 number is not always divisible by
a smaller one — 500 over 200 is 2.5 — so the stronger statement is not
true in general, and it was written as though it were. What is true, and
what the operator actually needs, is that the ruler's labelled step is a
whole number of grid *minors*; this asserts that, and that the grid is
the finer of the two.

### `fn draw`

One call from `super::interact`'s draw step, so the "which space"
decision this module exists to enact is made in exactly one place rather
than being spread across the canvas's page loop.

`clip` is the scroll viewport: the grid is confined to the intersection of
it with each page, which is both correct (there is no paper outside a page)
and what bounds the cost — see [`MIN_GRID_PITCH_PTS`].
