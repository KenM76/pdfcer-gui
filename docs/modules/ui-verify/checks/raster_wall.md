# `ui-verify/checks/raster_wall`

`the_raster_wall_stops_the_zoom_instead_of_painting_an_error` — **O186**,
the two halves that can be driven.

# The report


> *"At deeper zooms I am still experiencing the cursor jumping, and at some
> point it sometimes repositions to where the object area I was zooming into
> is no longer on screen. […] I think this sometimes results in similar
> error to 'This page could not be drawn. requested raster size
> 50411508x32619210 is empty or exceeds MAX_PIXMAP_EDGE'. perhaps the zoom
> is fine, but the cursor has jumped somewhere unsuported. If this error is
> caused by some other limitation that will always happen, zoom should stop
> at the limit and not end up showing an error - the canvas will just stop
> zooming in and can still function. the error can still be shown on the
> bottom bar so the user has some idea as to why zooming stopped short of 1
> trillion percent."*

One paragraph, and it contains two findings that turned out to have nothing
to do with each other:


**The sheet that could not be drawn was never the sheet he was looking
at.** `50411508 × 32619210` is `1224 × 792` pt at scale `41185.87`, and
`SW41177.pdf` holds thirty-four sheets at `1584 × 1224` against exactly two
at `1224 × 792`. He was right that *"perhaps the zoom is fine"*: the zoom
was fine, the acting page rendered perfectly through the region tier, and
the error was a **neighbour** in the continuous strip.

# Why part A asserts on the REFUSAL and not on the sentence he saw

This is the single most important property of this check, and getting it
wrong would have produced a check that passes on a build with the defect
still in it.

`app::settle::absorb`'s `absorb_render` now has a `BeyondRaster` arm that learns
a raster ceiling from the refusal and **returns early** — without setting
`render_error` and without clearing the page texture. That arm is part of
O186's own fix (clause two), and it is upstream of the sentence. So on a
build where `fill_strip` still places the impossible order:

* the engine still refuses it — `bad-raster-size px=… page=…` still appears;
* a ceiling is still learned from a **neighbour's** geometry, which silently
  caps the zoom of a page that could have gone much further;
* and **no `canvas-message` is published at all**, because the refusal was
  absorbed.

A check asserting *"the operator is never shown the MAX_PIXMAP_EDGE
sentence"* would therefore pass on the broken build, and would have been
read as proof that the defect was fixed. The assertion is made one layer
down, at the engine's refusal, where the condition is unambiguous — and the
refusal line carries `page=`, which is the whole finding: it names the sheet
and the sheet is not the one being looked at.

Generalised, and worth carrying away: **a net that swallows a symptom
invalidates every check that asserts on that symptom.** When a fix adds a
handler upstream of a user-visible complaint, the regression test has to
move upstream with it.

# The seam trick — how part A reaches the state at all

Part A needs a state that sounds awkward to reach: a page **visible** at the
same time as the acting page, at a zoom high enough that the visible one
cannot be rastered whole. Two facts make it reachable, and a third — added
after the check turned out flaky — decides how wide the window is.

1. **Ctrl+wheel is zoom-to-cursor.** The document point under the pointer is
   held where it is, give or take the drift measured below. So a pointer
   parked in the gap *between* two pages keeps **both** of those pages either
   side of it as the climb goes up, rather than scrolling one away.
2. **`viewer::strip::ROW_GAP` is 12 points at zoom 1 and is scaled by the
   zoom**, like the rest of the strip's geometry. The gap therefore grows
   with the zoom, and the two pages separate at a known rate: aiming at the
   gap's midpoint puts each page's edge `6 × zoom` points from the pointer.
3. **And that is also what closes the window.** The neighbour's top edge
   descends towards the bottom of the canvas as the gap grows, so there is a
   zoom above which it is off screen and the state cannot be measured at all.

# The window is NARROW, and getting that wrong made the check flaky

This section used to read *"With `fixtures/four-pages.pdf` the window is
wide … both are far below the zoom at which the growing gap pushes the
neighbour off screen."* That was reasoned from the design, not measured, and
it is **false**. Two runs on 2026-09-12, minutes apart, one passed and one
SKIPPED with *"the strip never reported a visible page it could not order"* —
and the only difference was where a plain wheel notch happened to leave the
seam.

The **lower** bound of part A's window is the engine's.  [`FIXTURE`]'s pages
are `2383.937 × 1683.78`, `612 × 792`, `612 × 792` and `306 × 396`, and
`render::strategy::whole_page_raster_fits` refuses a whole-page raster once
`longest × raster_scale` passes `MAX_PIXMAP_EDGE - 1`. A letter-size
neighbour therefore becomes unorderable at a raster scale of about **20.7**
(`16384 / 792`), and the E-size sheet at about 6.9.

The **upper** bound is the one that was missed. It is not *"half the canvas"*
— it is the room between the parked seam and the bottom of the canvas, and
the seam is parked *below* the middle for a separate and unrelated reason
([`SEAM_BAND`]). From the failing run's own trace:

```text
canvas-viewport   [288 174] - [1258 944]   770 pt tall
seam parked at    y = 700                  room below it: 244 pt
neighbour last counted visible at zoom 17.9, gone by 18.9
neighbour unorderable from zoom 20.7
```

The window was **empty**: the neighbour left the screen two notches before it
became unorderable, so the state part A exists to measure was unreachable and
the run reported an absence it had never been in a position to observe. The
repair is in two constants — [`VIEWPORT`] is now tall enough that half a
canvas is 575 points rather than 385, and [`SEAM_BAND`] is narrow and sits
just below the middle — plus a **pre-climb feasibility check** that SKIPs with
the arithmetic printed when the window is empty, instead of climbing ninety
notches and calling the result an absence.

Two details the measurement turned up, both of which the naive arithmetic
gets wrong:

* the strip stops counting a page as visible about **50 points above** the
  bottom of the published canvas rect, not at it ([`BOTTOM_DEAD_BAND_PT`]);
* the pointer does **not** hold its document point exactly. Over one climb the
  parked seam slid down the screen at about 4.7 points per unit of zoom, so
  the neighbour's top edge approaches the bottom of the canvas at about
  `10.7 × zoom` rather than `12 × zoom`. The feasibility check divides by 12,
  which makes its prediction land *early* — the conservative direction for a
  gate whose job is to refuse to measure.

**None of this arithmetic is used for a verdict.** The state is still
*detected* from the application's own `strip-beyond-raster pages=` line. What
it is used for is aim, budget, and the decision not to bother.

# Why `strip-beyond-raster pages=0` is printed, and why that matters here

`fill_strip` traces the count of visible-but-unorderable pages **before** it
scans for something to order, and prints `pages=0` deliberately. Its own
comment says why: *"or the check cannot tell 'never entered it' from 'still
in it'"*. This check is the caller that needed it. The whole of part A's
subject is an absence — no order, no refusal, no sentence — and an absence
measured in a state the run never entered is not a measurement. `pages=` is
what lets the run prove it stood where the defect fires.

# ⚠ Why the beyond-raster state is a precondition and NOT a gate on the
failure

Order matters here and it is not obvious. On a build with the defect, the
neighbour's refusal *teaches a ceiling* and the zoom is pulled back — so the
climb stalls at roughly the boundary and `strip-beyond-raster pages=` may
never reach 1 at all. A check that treated `pages >= 1` as a gate would then
SKIP on exactly the build it exists to catch.

So the refusal is looked for **first**, on every notch, over the whole climb.
Only when the climb finishes with no refusal does `pages >= 1` decide
between *"the state was entered and nothing went wrong"* (pass) and *"the
state was never entered"* (skip).

# What part B asserts, and the one thing it only NOTES

Part B leaves the strip, enters Single, and climbs until the rasterizer's
own content-dependent wall is met — `RasterizerLimit`, which is a different
refusal from the pixmap-edge one and arrives at a scale that depends on how
much ink the page holds. `app::settle::absorb`'s `learn_raster_ceiling` turns it
into a learned ceiling and pulls the zoom back, tracing
`raster-ceiling-learned … moved=true`. From that point the operator's fourth
clause is the specification, and it is checked literally: the zoom does not
rise past the learned ceiling, no error sentence is painted anywhere, and
the status bar's `status-group:raster-stop` region **is** on screen, not
clipped, inside the window.

**Part B now DOES assert that the page is still drawn at saturation**,
and the history of that sentence is worth a paragraph because it is an
argument about when an exemption expires.



⚠ The failure message names O186 and points at `geometry::pasteboard`, so a
reader who finds *only* this check red is not sent hunting in the rasterizer
for a defect that lives in the layout.

# What a passing run does NOT prove

That the learned ceiling is the *highest* scale the page could have reached
(that is the rasterizer's property, not the shell's, and it varies 28× with
ink — an E-size sheet was measured giving out at 284,964 where a business
card reached 8,053,069). That the neighbour is ever drawn again on the way
back down. That anything holds on a facing-continuous layout, which lays out
two pages per row and is not exercised here.

## Item notes

### `const FIXTURE`

Part A needs **more than one page**, pages of **different sizes** (so a
neighbour reaches its pixmap ceiling at a different zoom from the acting
page, which is what the operator's own set did), and a **known page count**
so the seam search can refuse to park after the last sheet. The operator's
drawings are frequently single-sheet, and a single-sheet document cannot be
in the state part A is about at all.

### `const FIXTURE_DENSE`

# Why a repository fixture cannot serve — measured, not assumed


That is not a defect, it is the region tier working. Above
`viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT` the canvas asks for the VISIBLE
REGION rather than the whole page, and the visible region shrinks as the zoom
rises — the last line of that trace asked for a region about 2×10⁻⁷ pt
across. The raster therefore stays viewport-sized for ever and there is no
zoom at which a pixmap ceiling can bind on it.

**So the refusal is a property of INK, not of size or of zoom.** What
gives out is the rasterizer's capacity to draw the content inside the
requested region at that scale, and a test document of four empty sheets has
no content to give out on. The operator met it on a 36-sheet SOLIDWORKS
drawing set; this check meets it on a 5.7 MB dense vector site plan, which is
the same kind of document and is already on the machine as the project's
standing render benchmark.

⚠ Absent, this check is **SKIPPED and says which file is missing** — never
passed, and never failed. Committing a multi-megabyte CAD drawing to
`fixtures/` was considered and rejected: that directory is for documents
whose specific content is the point, and this one is wanted for the sheer
quantity of it.

### `const PAGE_COUNT`

Used only to refuse to look for a seam *after the last page*, where there is
no neighbour to be unorderable. A wrong value here makes the run SKIP, not
pass — the seam band is still asserted.

### `const MESSAGE_REGION`

It has exactly two publishers — `canvas::present`'s no-pages arm and its
**Single-mode** `render_error` arm — and the `nothing-visible` arm publishes
**no** region at all. So part B's dead end cannot be mistaken for the
painted error, which is the one confusion that would have made this region a
useless oracle.

### `const UNFILLABLE_EVENT`

`fillable=false` means the frame declined to place an order because the page
had no region and its whole sheet is past the pixmap ceiling. Read here for
two different jobs: it is the transition pair that proves the new guard is
live, and when part A's window turns out to be empty it distinguishes "the
neighbour left the screen" from "the acting page itself went unorderable".

### `const ROW_GAP_PT`

# Why a copy of another module's constant is tolerable here, and how a
drift would show up

This is used for **aim**, never for a verdict. The pointer is parked
`ROW_GAP_PT / 2 × zoom` below the acting page's bottom edge, i.e. at the
midpoint of the gap, because that is the position at which the two pages
separate most slowly as the zoom rises.

If the application's gap grew, this aim would land *nearer the upper page*
but still inside the gap. If it shrank below this value, the aim would land
a few points onto the **lower** page — which is still a point that holds
both pages either side of it, because zoom-to-cursor is linear in the strip.
Either way the run still works, and the two things that could actually
invalidate it are asserted rather than assumed: the seam must land inside
[`SEAM_BAND`] of the canvas, and the canvas must report `visible >= 2`.

### `const BOTTOM_DEAD_BAND_PT`

On the failing run the neighbour was last counted at `visible=2` with its top
edge at y = 889 and was gone by y = 900, against a canvas whose published
bottom was 944. Whatever accounts for the band — a scroll bar, a clip inset,
the strip's own culling margin — the arithmetic that predicts when part A's
window closes has to allow for it, or it predicts a window that is about two
notches wider than the one that exists.

Used only for the feasibility prediction, never for a verdict. An exact
value is not needed and is not claimed; what is needed is that the prediction
errs on the early side.

### `const WINDOW_MARGIN`

A Ctrl+wheel notch multiplies the zoom by about 1.22, and the climb can only
observe the state on a notch boundary, so the first notch above 20.7 can land
as high as 25.9. 1.5 covers that with room to spare and still leaves the whole
of [`SEAM_BAND`] feasible.

### `const SEAM_SEARCH_NOTCHES`

It moves one notch at a time and re-reads the canvas each time, because the
notch distance is egui's and the document's opening zoom is the
application's — neither is this check's business to know.

### `const EDGE_MARGIN_PT`

The acting page's horizontal centre is used when it is on screen, and
clamped into the canvas when it is not. A point on the very edge risks
`Driver::confirm_uncovered` finding a scroll bar, which is a harness failure
dressed as an application one.

### `const CLIMB_NOTCHES_A`

At roughly 1.22× a notch this is about eleven orders of magnitude of zoom —
vastly more than the twenty or so needed to put a letter-size neighbour past
its pixmap ceiling. Overshooting is cheap and under-shooting would SKIP, so
the budget is set generously and the run breaks as soon as the state is
reached.

### `const CLIMB_NOTCHES_B`

The wall is content-dependent — the same build gave out at raster scale
284,964 on an E-size sheet and 8,053,069 on a business card — so this cannot
be derived, only budgeted. If it is not met, part B is NOTED as unmeasured
with the zoom it reached, never quietly dropped.

### `const CLIMB_BATCH_B`

Batched, unlike part A's single notches, because part B has no state to
detect *during* the climb other than its end, and a round trip per notch
over a budget of 160 is most of a minute of wall clock for nothing.

### `const EXTRA_NOTCHES_B`

This is the operator's *"the canvas will just stop zooming in"*: the test is
not that the clamp happened once, it is that it holds against continued
pressure.

### `const CEILING_SLACK`

The ceiling is converted scale → zoom through a division by the display
density and back again, so an exact comparison would be asserting `f32`
rounding. A thousandth is four orders of magnitude below a single wheel
notch, so it cannot mask a zoom that kept climbing.

### `fn part_a`

Returns `Some(failure)` when the refusal happened, `None` when the state was
entered and nothing went wrong. A state that was never entered is an `Err`,
which the report turns into a SKIP.

### `fn part_b`

Returns `Some(failure)` for a breach of that clause. An unreachable wall is
`Ok(None)` **with a note** — never an `Err`, because part A has already
measured something real and turning the whole check into a SKIP would throw
that away.

### `fn drive_b`

⚠ Deliberately a second launch of its own rather than a second stage of
[`drive_a`]'s session. The fixtures differ, and a check that opened a second
document into the first's window would be measuring the multi-document tab
machinery as well as the raster wall — two subjects, one verdict.

### `struct TheRasterWallStopsTheZoomInsteadOfPaintingAnError`

# Why this is a SEPARATE check, decided by driving rather than by taste


The first half needs several pages of differing sizes in one strip. This half
needs the rasterizer to REFUSE, and on a repository fixture it never does —
the measured run climbed to a zoom of ten billion, a trillion percent, with
zero refusals of any kind, because above `SUB_PIXEL_CONTENT_EXTENT` the
region tier asks only for the visible region and the visible region SHRINKS
as the zoom rises. There is no page size at which that order becomes too
large. What makes the rasterizer give out is the amount of INK inside the
region, which is why the operator met it on a 36-sheet SOLIDWORKS drawing
set and this check meets it on a dense CAD site plan. See [`FIXTURE_DENSE`].

Keeping them separate buys the thing the project keeps relearning: one check
is red for one reason. A merged check would have been SKIPPED on every
machine without the operator's drawings, taking the neighbour-sheet half —
which is measurable anywhere — down with it.

### `fn panic_was_converted`

[`Session::expect_thread_panic`] silences the harness's panic detector for
the whole session, and on its own that is an assertion both outcomes satisfy:
a rasterizer that gave out and was caught and one that simply died look the
same afterwards. `pdfcer-render` catches the panic and hands back a
`RasterizerLimit`, which the canvas publishes as a `raster-limit` line
carrying the panic text in its `panic=` field — so the conversion has a
witness, and a check that declares a panic owes the reader that witness.

Measured 1:1 across three runs on 2026-09-15: one `raster-limit` per panic in
each of the two zoom climbs, two of each in this check's own part B.

Returns the failure sentence, or `None` when every panic was converted.
