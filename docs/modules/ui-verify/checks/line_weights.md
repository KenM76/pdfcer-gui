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

# ★★★ Why a driven check when five unit tests already pass

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

# ★★★ The pixel assertion is SIGNED, and that is the whole point

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

# ★★ The zoom is a PRECONDITION, not decoration

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

★ Phase F is not symmetry for its own sake. A mode that cannot be left is a
mode that follows the operator into the next thing he does, and the
disclosure that says *"this is not what will print"* becoming permanent
would be the worst version of that.
