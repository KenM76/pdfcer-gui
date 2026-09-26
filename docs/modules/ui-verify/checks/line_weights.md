# `ui-verify/checks/line_weights`

`line_weights` — **the CAD "line weights off" display mode, driven: the
button exists, the drawing actually gets thinner, and the screen says it is
not what will print.**


> *"awhile ago you told me you removed the button to show all lines without
> their thickness — thin lines or something like cad has. The button never
> worked but I do want that display option!"*

Both halves of that were correct. `view.thin_lines` was registered, drawn on
the View tab, and **inert**; it was unregistered on 2026-08-17 because
`RenderOptions` had no field behind it. The engine shipped
`stroke_display: StrokeDisplay { Actual, Hairline }` (`Pass 254.0`,
`8f9fb3e`) the day this shell asked, so the control is back — and *"the
button never worked"* is precisely the sentence this check exists to prevent
being true a second time.

# Why a driven check when five unit tests already pass

Because every one of them is upstream of the screen.

| test | proves | cannot see |
|---|---|---|
| `settings::only_the_canvas_worker_sets_stroke_display` | no export can carry it | whether anything carries it |
| `settings::every_export_path_renders_real_widths_…` | the request carries it, the funnel does not | whether a button raises the request |
| `worker::the_render_key_moves_when_line_weights_are_turned_off` | the cache cannot serve a stale raster | whether a raster is asked for |
| `render::hairline::line_weights_off_puts_less_ink_…` | the ENGINE really thins the drawing | whether the shell ever asks it to |
| `commands::mapping::every_chrome_toggle_has_a_registered_command` | the id exists | whether the button is reachable |

The gap they leave is exactly the shape of the original defect: a registered
command, a correct handler, an invalidated cache — and **a press that never
arrives**, because the item is not on the band, or is in an overflow nothing
opens, or is drawn in a mode this document is not in. Only driving the real
window answers that.

# The pixel assertion is SIGNED, and that is the whole point

The two display conventions routinely confused with each other are
**opposites**:

| | | precedent |
|---|---|---|
| **line weights OFF ← what he asked for** | every stroke capped at one device pixel | AutoCAD `LWDISPLAY` off |
| enhance thin lines | sub-pixel strokes bumped **up** to one pixel | Acrobat's preference of that name |

*One makes thick things thin; the other makes thin things thick.* A check
asserting only *"the canvas changed"* would pass on a build that shipped the
wrong one, and shipping the wrong one is worse than shipping nothing —
it looks like the feature working while doing the reverse. So the assertion
is **strictly less ink after the press**, counted as dark pixels inside the
canvas rect.

# The zoom is a PRECONDITION, not decoration

At page-fit a CAD sheet's strokes are already at or under one device pixel,
the engine's §8.4.3.2 floor has them there, and a *ceiling* at one device
pixel has nothing to cap — so the two pictures are identical and the check
would fail against a perfectly good build. That is measured, not assumed:
`render::hairline::the_two_modes_are_identical_where_there_is_nothing_to_cap`
pins it at scale 1.0 in a unit test.

⇒ So the run **zooms in first**, to the 200–400 % band he named as where he
actually reads a title block, and asserts it got there before pressing
anything. A run that could not zoom has not built its own precondition and
is a SKIP, not a pass.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | open, and read the View tab | the `line_weights` item exists, and the disclosure has **never** appeared |
| B | Ctrl+wheel in to ≥ [`MIN_ZOOM`] | the zoom is reached and a raster catches up |
| C | capture the canvas | some ink to lose |
| D | press the item | `view-chrome LineWeights on=false` |
| E | capture again | **strictly less** ink; the `status-group:line-weights` line appears |
| F | press it again | the disclosure goes away and the ink comes back |

Phase F is not symmetry for its own sake. A mode that cannot be left is a
mode that follows the operator into the next thing he does, and the
disclosure that says *"this is not what will print"* becoming permanent
would be the worst version of that.

## Item notes

### `const MIN_ZOOM`

250 %, inside the 200–400 % band the operator named. Below roughly 150 % a
default-width (1.0 unit) stroke is under two device pixels and capping it at
one moves too little ink to distinguish from antialiasing noise.

### `const MIN_INK_DROP`

**3 %**, where the unit measurement on `a1-titleblock.pdf` at scale 4 is
**17.7 %** (484,078 → 398,578 dark pixels). Deliberately far under it: most
of a title block's ink is text and fills, which this mode correctly does not
touch, so the share that *can* move depends on the document. What is being
asserted is the direction and the reality of the change, not a rendering
constant.

### `fn ever_declared`

`ui_rect` is a **change log**, not a per-frame census — a widget that
stops being drawn publishes nothing. So "has it ever appeared" is the
question a change log can answer honestly, and "is it on screen now" is not.
Phase A therefore asks *never*, and phase F asks *not since the second
press*, which is the same question scoped to a suffix of the trace.

### `fn densest_ink`

# Why the climb has to be aimed, and what NOT aiming it cost

`Ctrl+wheel` zooms **about the pointer**, so whatever is under the cursor
stays under it for the whole climb. Aiming at the canvas's geometric centre
therefore does not zoom into the drawing — it zooms into whatever happens to
be in the middle of the sheet, and on a CAD sheet that is **blank paper**:
the linework is a border and a title block around the edges.

This check SKIPPED on `fixtures/a1-titleblock.pdf` for exactly that reason,
and its skip message named the repair without performing it — *"aim the view
at content before climbing"*. **A SKIP is not red**, so it had been telling
nobody anything for as long as it had been running.

# Why a patch and not the ink's bounding box

Measured on that fixture at scale 0.25: the ink spans **24–2356 pt
horizontally and 28–1660 vertically** — very nearly the whole A1 sheet — and
the centre of that box is the blank middle. A bounding box says where the
drawing *is*; it does not say where the drawing is *dense*, and those are
opposite answers on a sheet whose content is a frame.

# The method, and why it is deliberately coarse

Divide the canvas into a `GRID × GRID` lattice, count dark pixels per cell,
return the fullest cell's centre as fractions. Eight cells per axis is
enough to separate a title block from a border on any sheet size, and coarse
on purpose: a fine grid finds a single thick line and aims at a spot that
leaves the viewport as soon as the zoom climbs, while a coarse one finds a
*region* that stays populated all the way up.

It answers the cell's **centre**, not the darkest pixel, for the same
reason — the exact pixel of a stroke is a knife edge at 400 %, and one
rounding in the pointer's position falls off it.

Fractions rather than a screen point, so the caller converts through
`Frame::declared_at` like every other aim in this harness. `coords`' rule is
that a coordinate is **produced by a conversion and never assembled**, and
returning pixels here would be assembling one two conversions away from the
rect it belongs to.
