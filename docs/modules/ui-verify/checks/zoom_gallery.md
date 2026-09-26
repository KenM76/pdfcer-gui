# `ui-verify/checks/zoom_gallery`

`the_page_still_renders_at_every_decade_of_zoom` — pixels on the screen,
not numbers in a trace.

# The request


> *"Can you confirm that rendering on screen is actually happening at
> maximum zoom? zoom in on one of the michocondria structures and post
> screenshots here to confirm. start with the full page first to confirm it
> renders."*

# Why the existing checks do not answer this

`zooming_does_not_throw_away_where_the_operator_panned` proves the view
stays where it was put to a trillion percent, and
`zooming_past_the_pixmap_ceiling_still_renders` proves no raster is refused.
**Neither looks at the screen.** A canvas could satisfy both while drawing a
blank sheet: the position arithmetic would be perfect and the rasters would
complete, and the operator would see nothing.

That is not a hypothetical gap. `D:/dev/rag/egui/` records panels that
shipped unreachable in real builds with every gate green, and the rule it
draws from them is that layout and clipping defects have exactly one oracle:
a rendered screenshot. This check is that oracle for deep zoom.

# What it does

Opens the document, captures the window, then climbs by Ctrl+wheel with the
pointer parked on a target that is given in **document coordinates**, so the
same run works whatever the window size. It captures again at each decade.

At every step it asserts three things, and each rules out a different way of
being wrong:

| assertion | rules out |
|---|---|
| the capture is not near-uniform | a blank or white canvas |
| the canvas traced `drawn ≥ 1` | a page reserving space with no raster in it |
| no `outcome=failed` render | a refused rasterization the shell swallowed |

All three, because any two can hold while the third fails. A page that
draws its *state message* is not near-uniform; a page whose raster completed
can still be drawn off-screen; a shell that stopped asking cannot report a
failure.


The uniformity assertion above is correct, and it is what caught O174. It is
also unfalsifiable in one direction, and that cost most of an afternoon.

A near-uniform canvas is consistent with **two** worlds: the shell lost a
raster that had ink in it (a defect), or the engine faithfully drew a blank
rectangle (not a defect). At five thousand percent a viewport is about a
fifth of a point across, and most fifth-of-a-point squares of a real drawing
contain nothing at all — so on any fixture other than the one this ladder was
calibrated against, world two is the *common* one at the deep rungs.

This check reported world two as world one, three rungs running, on the
operator's `A-591.pdf`. Settling it took a hand-written test that scraped the
region rectangles out of the trace and called `pdfcer_render` on them
directly; the engine returned one tone for the two deepest.

`ink=` on `render-async-done` (see `crates/pdfcer-gui-base/src/renderworker/ink.rs`)
makes that hand-written test permanent and free. The rule is now:

| canvas | raster | verdict |
|---|---|---|
| uniform | `ink=1` | the DOCUMENT is blank here — SKIP, so the run is INCOMPLETE, not FAILED |
| uniform | `ink>1` | **the defect** — the engine drew a picture and the shell did not show it |
| uniform | no field | the old, weaker verdict, and the message says the field was missing |
| not uniform | — | the rung passes, as before |

The blank-document case is deliberately not a pass. A rung that measured
nothing is a third state, and collapsing it into either verdict is how a
check comes to look green while seeing nothing.

# The captures are evidence, kept on pass as well as fail

Written to the output directory and named by zoom, because the question
being answered is *"show me"* and the answer is a file the operator can
open. On a later failure they are also the only thing to compare against.

## Item notes

### `const TIERS`

# Chosen against what the fixture actually contains

`banana.pdf`'s own generator prints the scale chain, and these are its
tiers rather than round numbers picked for the look of them:

| zoom | what becomes visible |
|---|---|
| 1 × | the banana, at life size |
| 20 × | the two cell outlines |
| 120 × | cell labels, starch grains |
| 450 × | organelle labels |
| 4,000 × | chloroplast grana, plasmodesmata |
| 26,000 × | mitochondrial cristae |
| 350,000 × | ATP synthase heads — the 10 nm features |
| 10,000,000,000 × | the configured ceiling |

The last rung is not a feature tier. It is there because the operator
asked whether rendering *still happens* at the maximum, and a gallery that
stopped where the detail stops would not have answered him.

### `const BATCH`

A small batch rather than a computed count. A Ctrl+wheel notch multiplies
by about 1.22, but the ladder's rungs are not a pure geometric series near
the bottom, so the number of notches to reach a given zoom is not something
a check should predict. It rolls, reads what the application says, and stops
when it is there — which is also what makes it survive a change to the
ladder.

### `fn last_ink`

Reads `ink=` off the last `render-async-done` whose `outcome=done`. Renders
that were cancelled or failed carry `ink=-1` and are filtered out rather than
read as a count: a cancelled render says nothing about what the page
contains, and letting `-1` through would turn every mid-zoom cancellation
into a spurious *"the engine drew nothing"*.
