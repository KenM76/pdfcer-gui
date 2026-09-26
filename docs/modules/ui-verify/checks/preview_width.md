# `ui-verify/checks/preview_width`

`preview_width_ignores_zoom` — the driven proof of `OPERATOR_REQUESTS.md`
**O184**: the blue outline that follows your hand while you drag an object
is **the cursor**, and a cursor does not grow when the document is
magnified.

# The report


> *"The live preview blue outlines that appear when we drag an object scale
> with zooming in and out of the page instead of being independent of zoom —
> at high zoom levels they end up being the width of the canvas. I think
> they keep the same size as the line widths they are moving and that is ok
> — but if we set the line width view to the one pixel width option the
> preview lines should also be affected by this setting."*

Two rulings, and they are not the same ruling:

| # | the ruling | what this check asserts |
|---|---|---|
| 1 | the preview may take its width from the object's own line width, but **zoom must not enter** | phases **A** and **B**: the same drag at two zooms produces the **same** preview width |
| 2 | the one-pixel line-weight view governs the preview too | phase **C**: with `view.line_weights` off, the preview is **1.00 px** |

# ★★★ Why this cannot be a screenshot, and why it needs the trace

Zoom-invariance is **a claim about two frames at two zooms.** No single
capture can carry it: a 12-pixel outline at 100 % and a 12-pixel outline at
900 % are individually unremarkable, and it is only the *pair* that says
anything. So `canvas::shapes::draw` publishes the widest preview stroke it
painted, in device pixels, on every frame it draws one:

```text
canvas-shape-drawn shapes=3 segments=88 erased=3 zoom=8.412 real_widths=true widest_px=2.00
```

and this check reads that number at two zooms and asserts it did not move.

⚠ **The number is computed by the same function that sizes the stroke**
([`StrokeRule::preview_px`]), which makes it a report of the decision rather
than an independent measurement of the pixels. That is a deliberate and
stated limitation: the alternative — counting blue pixels across a stroke in
a capture — cannot separate the preview from the erase band beneath it,
which is *supposed* to scale. What this check owns is the decision; what a
human owns is that the decision is drawn. ★ The `zoom=` field on the same
line is the guard against the degenerate reading: if `zoom` did not move
either, the check SKIPs rather than passing, because two readings at one
zoom assert nothing at all.

# ★★ Why phase C can SKIP, and why that is honest rather than weak

[`StrokeRule::preview_px`] floors the width at one device pixel — a hairline
(`0 w`, PDF 32000-1 §8.4.3.2) is one device pixel and a zero-width egui
stroke vanishes under antialiasing. So on an object whose stroke is already
at or below 1 pt, **the hairline view cannot be distinguished from the
normal view**, both answer 1.00, and an assertion that they differ would be
asserting something the correct build does not do.

⇒ Phase C therefore asks phase A what it measured first. If phase A already
read 1.00, phase C reports that it cannot measure this ruling **on this
object** and says which fixture would. This project has written down twice
that a check which cannot fail is not evidence; saying so out loud is the
only version of that which anybody ever reads.

# The gesture, and why it is the top rung and not a node drag

`shape_preview` descends two rungs and drags an **anchor**, because O63 was
about node editing. O184 is about *"when we drag an object"* — the plain
gesture, one click and a pull — so this check drives that and nothing else.
The two checks exercise different `MoveSubject` arms into the same painter,
which is worth having: `for_move_subject`'s object arm and its node arm
reach [`super::super`]'s stroke rule by different routes.

# Fixture requirements

`--pdf` and `--doc-point PAGE,X,Y` (★ **0-based page**) naming a point on a
**stroked vector object**. The sweep's default aim,
`fixtures/a1-titleblock.pdf` at `0,2000,320`, is the title block's linework
and is what this was measured on. A point on an image or on text selects
something with no stroked geometry, `for_move_subject` answers with an erase
and no shapes, and the check SKIPs naming that.
