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

`render::settle::absorb`'s `absorb_render` now has a `BeyondRaster` arm that learns
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
much ink the page holds. `render::settle::absorb`'s `learn_raster_ceiling` turns it
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
