# `ui-verify/checks/scale_sweep`

`mouse_work_survives_every_render_tier` — **the ordinary mouse gestures,
driven at every scale where the renderer changes strategy.**

# The report this exists for


> *"you should also try zooming in on the atoms of the banana pdf file and
> see what happens when you try to draw a box around a molecule and move it,
> or select the ion and move it, or edit the nodes at that scale. you should
> check all the mouse actions and capabilities out at each scale where our
> scaling algorithm changes."*

He is naming a place he suspects the program breaks — **deep zoom, in the
region tier, doing ordinary mouse work** — rather than a feature. So this
check is a sweep rather than an assertion about one gesture: it walks the
zoom up through the two tier boundaries and, at each rung, runs the same
battery and says what happened.

# The three tiers, and the boundaries they are found at

Both boundaries are *derived from the page*, never written down as a zoom:

| tier | begins when | on US Letter (792 pt long edge) |
|---|---|---|
| [`render::strategy::Strategy::WholePage`] | always | up to 2,068 % |
| [`render::strategy::Strategy::Region`] | `longest_pt × raster_scale > MAX_PIXMAP_EDGE − 1` | 2,068 % and up |
| + the `f64` position anchor | `longest_pt × zoom > SUB_PIXEL_CONTENT_EXTENT` | 132,396 % and up |

The rungs this check walks are chosen **just below, at, and just above**
each of those, plus one far inside the region tier. This project's own
`strategy.rs` records why: *"two samples either side of a transition look
exactly like no transition at all if the transition is not where it was
assumed to be."*

# What is measured, and why the pointer probe is the sharpest of them

The battery is: **pointer conversion**, **click-select**, **drag the
selection**, **marquee on empty paper**, **anchors and node drag**, and
**resize handles**.

The first is the one that can fail silently and take the rest with it.
`canvas::trace::pointer` publishes, for every pointer position, the
**canvas-space point the application thinks the pointer is on**, computed
by `viewer::screen_to_page`:

```text
page.x = (screen.x − image_rect.min.x) / zoom
```

— entirely in `f32`. At a deep zoom `image_rect.min.x` is an enormous
negative number (the page's own left edge, scaled), and `f32`'s
representable spacing there can exceed the size of the whole viewport. If
that subtraction has lost its low bits then **every hit test, every grip
placement and every drag delta on the canvas is derived from a number that
no longer distinguishes one part of the window from another** — and nothing
else in the trace says so, because a wrong-but-plausible coordinate still
selects *something* and still moves it *somewhere*.

So the probe moves the pointer a known number of screen points and asserts
the reported canvas point moved by exactly that over the zoom. It is a
linearity test, and it needs no fixture knowledge at all.

# Why the zoom is driven by Ctrl+wheel and not by the status bar's `+`

Zoom-to-cursor keeps the point under the pointer fixed, so the content this
check aims at stays under the aim point all the way down. The `+` button
zooms about the viewport centre, which on a page whose interesting detail is
off-centre magnifies blank paper — the operator's own complaint of
2026-08-22, *"Right now you are just zooming into a blank area on the
canvas."*


The 2026-09-05 sweep filed it as application defect **A4**: *"mouse work
degrades with the render tier — no traced drag outcome between 104 % and
6,957 %, and no anchor marks published above 942 %."* Four separate probes
were wrong, and none of them was the application:

| probe | what it reported | what was true |
|---|---|---|
| the **drag** | *"nothing at all — no move, no decline, no resize"* at every rung | it pressed the CENTRE OF THE BOUNDING BOX of an open polyline, twelve points off the stroke; O72 makes that a marquee, and there was no arm reading `marquee-mode`. Pressed on the ink, the object moves at every rung |
| the **anchors** | *"6 anchors and the overlay published no mark for any of them"* above 942 % | the same line carried `on_screen=0`. `canvas.anchor.N` is published for the culled set by design (O69), and the field that says so was ignored |
| the **click** | *"clicking directly on the content selected nothing"* above 6,957 % | the closed-loop aim had lost the target — 312 screen px away — because the pan probe scrolled the view and never scrolled it back. See `scale_aim::re_aim` |
| the **pan** | *"the canvas published no coverage line after the wheel"* at every rung including 104 % | `canvas-coverage` is a CHANGE LOG. No new line means the coverage did not move, which is the healthy answer |

⇒ **A uniform failure at every rung of a scale sweep is evidence about
the probe, not about scale.** The baseline rung is the control, and a
control that fails is the finding. The repaired check now drives every
gesture successfully from 104 % to **2,298,019 %**, with an aim residual of
0.0000 canvas points at the deepest rungs — so the `f32` `screen_to_page`
hypothesis above is **falsified by measurement**, twice, and this header's
description of the risk is kept because the risk is real and the outcome is
not.

# Configuration

`--doc-point PAGE,X,Y` names the content to zoom into (**0-based page**).
`UI_VERIFY_SWEEP_ZOOMS`, if set, replaces [`DEFAULT_RUNGS`] with a
comma-separated list of zoom multipliers — the sweep is meant to be re-aimed
from the command line while a boundary is being narrowed down.

## Item notes

### `const OUTLINE_REGION`

The measurement this whole sweep turns on. `overlay::visible_outline_rect`
widens it to [`MIN_OUTLINE_EXTENT`] on each axis, and `handles::grip_at`
then covers `GRIP_SIZE_PX / 2 + GRIP_GRAB_SLACK_PX` = 6 pt inward from each
corner — so a box narrower than **12 pt has no body left to drag**, and
every press on it is a grip.

### `const RESIZE_COMMIT_EVENT`

The worst of the three outcomes and the one this sweep was not
watching for on its first two runs. A drag meant as a move that lands on a
grip and is *refused* costs the operator a gesture; one that lands on a grip
and **succeeds** costs them their artwork, silently. Measured on the
banana's cells at 15,808 %: `resize-commit grip=SouthEast sx=3.9770
sy=4.1891` — a 0.09 pt cell quadrupled on both axes by a drag that was aimed
at the middle of it.

### `const MARQUEE_EVENT`

It is also **the drag outcome that had no arm** until 2026-09-05, and
its absence produced this check's whole headline. A press on blank paper
inside a selection's bounding box draws a band rather than moving the
selection (`OPERATOR_REQUESTS.md` O72); with nothing reading this line
during the drag probe, that registered as *"nothing at all — no move, no
decline, no resize"*, and the sweep filed *"mouse work dies at every render
tier"* against a build in which the same drag moves the object every time it
is pressed on the ink. See [`drag_selection`].

### `const COVERAGE_EVENT`

The operator's own report of 2026-09-04 — *"the canvas does a fading
around the edges on stuff shown at the edges of the view. I don't want this.
it should render true."* — is a claim about exactly this line: `sharp` is
the fraction of the viewport the SHARP raster covers, and anything below
1.000 is the low-resolution backdrop showing through.

### `const DEFAULT_RUNGS`

Chosen against the **US Letter** boundaries in the module header — 20.69×
for the pixmap ceiling, 1,324× for the `f64` position anchor — and bracketed
on both sides of each rather than merely stepped past. `strategy.rs`'s own
test header states the rule these follow: a transition sampled only at its
endpoints is a transition that cannot be seen.

### `const PROBE_TOLERANCE`

The reported canvas point is printed to two decimals, so at a deep zoom the
quantum of the *printout* is already a large fraction of the movement being
measured — `100 px / 20000×` is `0.005` canvas units, which prints as `0.01`
or `0.00`. So the probe is only asserted where the movement is legible in
the printed precision, and this tolerance covers the rounding that remains.

### `const PROBE_FLOOR`

Below this the `{:.2}` printout, not the arithmetic, is the limit — see
[`PROBE_TOLERANCE`]. Reported as "not measurable at this zoom" rather than
as a pass or a failure, because either would be a claim the evidence cannot
support.

### `const AIM_TOLERANCE_PX`

Screen pixels, not canvas points: what every probe below needs is that
the press lands on the same ink, and "the same ink" is a screen distance.
The canvas's own pick tolerance is of this order, so a residual under it
cannot change what a click hits; in canvas points the same tolerance would
be meaninglessly tight at 100 % and meaninglessly loose at 200,000 %.

Above this the rung's pointer probes are **not run**, and the rung says
so in its own words. The 2026-09-05 sweep ran them anyway and filed
*"clicking directly on the content the zoom is anchored to selected
nothing"* at five rungs — measured with the pointer **312 px** away from
that content.

### `fn undo`

The sweep holds ONE document across every rung and steers one aim point
through it, so a rung that leaves the page changed hands the next rung a
different document. Measured before this existed: the 107 % marquee move
translated all 212 objects by 37.6 pt, and the three rungs above it then
reported *"clicking directly on the content selected nothing"* — correctly,
because the content had been moved out from under the aim by the check
itself.

Undo rather than "do not test the move": the move IS the subject. What has
to be true between rungs is that the document is the one the sweep started
with, and the application's own undo is the only thing that can promise
that.

### `fn probe_pointer`

Moves the pointer a known distance and asks the application where it thinks
the pointer went. A conversion that has lost its low bits answers with a
movement that is too small, zero, or quantised.

### `fn selection_count`

`canvas-selection` is **de-duplicated** — `trace_changed` suppresses a
line identical to the last one in its slot — so a second click that selects
the same object writes nothing, and a check counting those events reads
"the click did nothing" about a click that worked. That produced three false
findings on this sweep's first run. `canvas … sel=N` carries the count on a
line whose other fields move, and `selection-set` is written
unconditionally; between them there is no silence to misread.

### `fn drag_selection`

# The FOUR outcomes, and why counting only `canvas-move` hid the real one

A drag on a selected object can become a **move**, a **resize** (the press
landed on a grip), a **marquee** (the press landed on empty paper), or
nothing. The first version of this counted `canvas-move` alone, so a drag
that was routed to the resize machinery and then refused by it reported as
*"the gesture was thrown away"* — true, and silent about the mechanism.
`resize-declined reason=Degenerate` is the line that says what happened.




> *"Grab it in the middle and move it" is the gesture the operator
> described, and the middle of the object is a fact only the application
> knows. It publishes it as `canvas.selection-outline`; aiming anywhere else
> is the harness inventing a coordinate.*

**The middle of a bounding box is not the middle of an object.** The sweep
fixture `polyline-nodes.pdf` is one open path — a zigzag and two Béziers
from (100, 200) to (580, 320) — whose bounding box is 480 × 120 and whose
centre, (340, 260), is **twelve points of blank paper above the stroke**.

And a press on blank paper inside a selection's bounding box is a
**marquee**, deliberately, since `OPERATOR_REQUESTS.md` **O72**:

> *"Click and hold shouldn't select an object - it should allow me to draw a
> box around objects to select."*

`canvas::pressing` downgrades `Grip::Move` to `None` unless `body_under`
finds ink at the press point, and `(None, None)` is `DragKind::Marquee`. So
this probe was measuring the operator's own feature and reporting it as
*"dragging a selected object produced no traced outcome of any kind"* — at
**every** rung including 104 %, which is what made it read as a
zoom-dependent defect. Driven with the press moved to the aim point, the
same build MOVES the object at 104 %, 942 %, 2,096 % and 2,559 %.

⇒ **A uniform failure at every rung of a scale sweep is evidence about
the probe, not about scale.** The one rung that is not the subject — the
baseline — is the control, and a control that fails is the finding.

The aim point is the document coordinate the caller supplied and the one
the click immediately before this selected the object from, so it is on the
object by the same evidence that produced the selection. The old comment's
worry — that the aim can sit on a **grip** — is answered rather than
ignored: the resize arms below report which grip, and a resize at the aim
point is a statement about where the aim is, not about the mouse.

### `fn marquee`

# ⚠ A press on ink is not a band, and finding that out cost a moved object

`canvas::presspick`'s rule is that a press on an object **selects it**, and
a drag from there **moves it**. The first version of this started the band
at a fixed fraction of the viewport; at 107 % that fraction landed on the
banana's own outline, and the "marquee" dragged a 250-point object across
the sheet — silently changing the document every later rung was measured
against. The trace said so plainly (`selection-set … object=2 via=press`
followed by `canvas-move … dx=254`) and the check did not look.

⇒ So the origin is **probed** rather than assumed: candidates are clicked
until one selects nothing, and only then is a band dragged from it. The
probe is a click, which is reversible; a drag is not.

### `fn pan_and_watch_the_edges`

The operator, 2026-09-04: *"the canvas does a fading around the edges on
stuff shown at the edges of the view. I don't want this. it should render
true."* `render::strategy::region_for`'s header records the fix that landed
for it — the snap now centres the window on the grid instead of flooring its
origin, so the guaranteed margin is a quarter of a viewport on **every** side
instead of half a screen on two sides and nothing on the other two.

This is that claim, driven rather than computed: scroll, then read the
**worst** `sharp=` the canvas reported over the frames that followed.
`sharp=1.000` means the sharp raster covered the whole viewport; anything
less is the backdrop showing through somewhere.

A **wheel** rather than a drag, because a drag on the canvas is a
selection gesture and would be measuring something else. The status line
says `wheel=scroll`, so a plain wheel here is a pan.
