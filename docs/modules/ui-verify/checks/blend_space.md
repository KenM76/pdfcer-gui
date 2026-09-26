# `ui-verify/checks/blend_space`

`blend_space` — **a page blended in ink keeps its ink at every zoom, and
says so on the rare occasion it cannot.**


> *"seems I get different results depending on Zoom level. The [shading]
> boxes … on zoom out the colors between our rendering and the
> references don't match, but they do when I am zoomed in. up to 474% they
> are mismatched, but at 579% they match."*

# What is actually happening, measured before this check was written

`pdfcer-render` composites a page containing transparency in a **subtractive
CMYK buffer**, which is the correct space for it. That buffer has a
documented ceiling — `MAX_CMYK_BUFFER_BYTES`, 256 MiB at 20 B/px, i.e.
**13,421,772 pixels**. Past it the renderer falls back to compositing in
sRGB and counts that it did (`cmyk_buffer_refused`).

On an A4 page that ceiling is crossed at **zoom 534 %** — dead centre of the
band the operator bracketed. Bisected with the CLI to the pixel: buffer used
at scale 5.33 (13,394,232 px), refused at 5.34 (13,444,992 px).

Crossing it changes the rendered colour. Measured on the conformance suite's
composite page by
box-averaging **every pixel** of both renders into a common grid — so that
resampling could not masquerade as the effect — the transparency patches
move by up to **16 levels out of 255**.

Two earlier measurements got this wrong in opposite directions and are
recorded because both are tempting. Sampling a sparse lattice reported
358 of 576 cells differing, all of it text sampled at two pixel sizes.
Excluding every cell that was not flat removed that noise correctly — and
removed **the gradients**, which are the thing the operator is looking at.
Only a full box average is stable for flat patches, gradients and text
alike.


Everything above is still true of `pdfcer-render`. What changed is the
**shell**, and it changed in the direction that makes the operator's report
not happen at all rather than be apologised for.

`render::strategy::for_page` now ends the whole-page tier at the colour
ceiling as well as at `MAX_PIXMAP_EDGE`, for a page it has observed
compositing in ink. A *region* raster of the same view stays under the
ceiling at any zoom, because its buffer is sized to the region rather than
to the page — so on the composite conformance page the driven trace now
reads:

```text
raster-blend-space cmyk_buffer=true refused=0 wrong_space=0 scale=0.752
ink-page page=0
raster-blend-space cmyk_buffer=true refused=0 wrong_space=0 scale=8.013
```


So the primary assertion is now the **stronger** one — *the ink survives*
— and the disclosure assertion has become a fallback for the cases where it
genuinely cannot: an operator who sets a very small ceiling, a very large
display whose region raster plus overscan exceeds the ceiling on its own
(measured by the engine at ~281 MB at 1440p and ~633 MB at 4K, both above
the 256 MiB default), or a page opened directly at a high zoom before the
shell has observed that it is blended in ink.

**The three outcomes are told apart by the trace, and the difference
matters.** Before this, a page with no transparency and a page whose ink
survived were indistinguishable to this check — both reach the ceiling zoom
with no disclosure — and it reported FAIL for both. It did exactly that on
`SW41177.pdf` during the full run of 2026-08-26, with a report reading *"the
page's colours have changed and nothing on screen says so"* about a line-work
drawing that has no transparency anywhere on it.

| `cmyk_buffer` seen true? | `refused` seen? | verdict |
|---|---|---|
| no | no | **SKIP** — the fixture has no transparency; nothing to measure |
| yes | no | **PASS** — the ink survived past the ceiling zoom, which is the repair |
| yes | yes, and the disclosure appeared | **PASS** — the fallback engaged and was declared |
| yes | yes, and it did not | **FAIL** — the reported defect, unchanged |

# What this check asserts, and what it deliberately does not

It asserts that when the fallback engages, **the operator is told** — the
`status-group:blend-space` disclosure appears — and that when it has not
engaged, the line is **absent**. Both halves, because a disclosure that is
always on says nothing.

It does **not** assert that the colours are right on either side of the
ceiling. That is the engine's question and it is filed as one. What this
shell owes today is rule 4's surviving half: an inference the operator
cannot see — a screenshot says nothing about which space a page was
composited in — owes an off-canvas report.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | open a page with transparency at fit zoom | **no** blend-space line |
| B | Ctrl+wheel up until the raster passes 13.4 Mpx | the line appears |
