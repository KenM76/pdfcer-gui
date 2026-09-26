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
