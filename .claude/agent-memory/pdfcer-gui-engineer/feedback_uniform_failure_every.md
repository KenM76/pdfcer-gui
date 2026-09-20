---
name: a-uniform-failure-at-every-rung-of-a-sweep-is-about-the-probe
description: A parameter sweep whose every rung fails — including the baseline — has measured its own instrument. The first rung is the control; a control that fails is the finding.
metadata:
  type: feedback
---

**Before reading a sweep's failures as a property of the parameter, ask whether
the FIRST rung failed too.** A sweep is an experiment with a built-in control:
the rung nobody suspects. If it fails, nothing above it is evidence about
anything except the probe.

## The evidence, 2026-09-05

`mouse_work_survives_every_render_tier` walks the zoom through both render-tier
boundaries and runs a gesture battery at each rung. It reported *"dragging a
selected object produced no traced outcome of any kind"* at **104 %, 942 %,
2,096 %, 2,559 % and 6,957 %** — and was filed as *"mouse work degrades with
the render tier."*

104 % is not a render tier. It is the ordinary view of an ordinary page, and
the report said the mouse did not work there either.

**What was actually happening:** the probe pressed at the **centre of the
published grip box**, deliberately, on the reasoning that *"the middle of the
object is a fact only the application knows."* The fixture is one open polyline
480 × 120 pt; the centre of its bounding box is **twelve points of blank paper
above the stroke**, and a press on blank paper inside a selection's box is a
**marquee** — which the operator asked for by name (O72). No arm read
`marquee-mode`, so a correct answer registered as *"nothing at all."*

Moved to the aim point — the coordinate whose click had just selected the
object — the same build moves it at every rung including **2,298,019 %**.

## Three more probes in the same check, all wrong the same way

| probe | reported | true |
|---|---|---|
| anchors | *"6 anchors and no mark for any of them"* above 942 % | the same line carried `on_screen=0`. The marks are published for the culled set **by design** (O69), and the field that says so was added precisely to tell these two cases apart |
| click-select | *"selected nothing"* above 6,957 % | the closed-loop aim had lost the target by 312 px, because a pan probe scrolled the view and never scrolled it back |
| pan | *"the canvas published no coverage line"* at every rung | `canvas-coverage` is a **change log**. No new line means the coverage did not move |

## The rules this earns

- **A uniform failure at every rung is evidence about the instrument.** Two
  samples either side of a boundary look like no boundary; nine samples all
  failing look like no boundary either, and mean the probe.
- **Press where the click that selected the object landed.** The point that
  produced the selection is on the object by the same evidence. A bounding-box
  centre is a coordinate the harness invented.
- **Enumerate every outcome the gesture can have**, including the
  correct-but-unwanted ones. An `else` that says "nothing happened" is a claim
  the application was silent, and it is usually a claim that the check was not
  listening.
- **Read the census field before declaring an absence.** `on_screen=`,
  `total=`, `refused=` exist because somebody already hit this.
- **A sweep that measured nothing must SKIP, not pass.** After adding a
  legitimate "this rung could not be aimed at" branch, add the guard that
  refuses a green result when *every* rung took it.
