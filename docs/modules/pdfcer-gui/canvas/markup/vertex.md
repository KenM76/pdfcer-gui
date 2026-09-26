# `canvas::markup::vertex` — polyline and polygon, and the gesture the
operator has to end

The two **click-shaped** markup kinds: what one click does to the run, what
the two endings do, and what the preview has to draw that a band's does not.
[`super`] owns what they author; this file owns when.

---

## 1. ★ THE ENDING IS A SOLVED PROBLEM IN THIS CODEBASE, AND IT IS SOLVED
## HERE THE SAME WAY

A band drag ends when the button comes up. A polyline does not: click, click,
click, and then *something* has to say "that was the last one". This shell has
met that exact problem once before — [`crate::canvas::measure::circular`], the
radius/diameter tool, whose pick set has no natural arity either — and the
operator settled it:

> **Two endings, routed through one commit path**: a **double-click** on the
> canvas, and a registered **command**.

That is what this module implements, deliberately down to the shape of the
functions, and the reason for the deliberateness is worth stating because it
outranks a marginally better third answer: **an operator who has learned that
a double-click ends a radius pick must not have to learn something else to end
a polyline.** Two tools with the same problem and two different answers is a
product that has to be memorised rather than understood.

| ending | entrance | why it exists |
|---|---|---|
| **double-click** on the canvas | [`click`]'s `double` flag | what every drawing package's multi-point tool uses; the standing *"make it work the way other programs do"* tie-breaker |
| **`markup.finish`** on the ribbon | [`finish`], via `app::dispatch` | discoverable without knowing the double-click, and reachable when the last vertex sits somewhere awkward to double-click |

Both call [`commit`] and nothing else raises a vertex `Action::CommitMarkup`.
Two arms that each assembled a [`super::Geometry::Vertices`] would be two
derivations of one answer: they would agree on the day they were written,
diverge at the first change to either, and **the operator would have no way to
see it** — a polygon drawn from the same clicks looks the same whichever code
wrote it.

Neither ending is an accept box floating over the canvas, which is what
decision 024 retired at the operator's instruction.

### 1.1 The reference applications, and where they disagree

Under the standing instruction to match Inkscape, Acrobat and SolidWorks, in
its sharpened form — *ask which of the three actually has the surface in
question*:

| | polyline / polygon gesture | ends by |
|---|---|---|
| **Acrobat** | Comment ▸ Drawing tools ▸ Polygon / Polyline | click per vertex, **double-click** to finish |
| **Inkscape** | the Bézier/pen tool, in its straight-line mode | click per node, **double-click** to finish, `Enter` also finishes, clicking the **first node** closes |
| **SolidWorks** | the sketch polyline / line chain | click per point, **double-click** or `Esc` ends the chain, clicking the start point closes |

**All three double-click, so the double-click is not a judgement call.** What
they disagree about is the *second* way out, and this is the interesting half:

* *Inkscape and SolidWorks both close a shape by **clicking the first
  vertex**.* Acrobat does not, because its Polygon tool closes by subtype —
  the operator never draws the closing segment, the file does. **pdfcer is in
  Acrobat's position**: `/Polygon` closes back to `/Vertices[0]` by
  §12.5.6.13, so a click-the-first-vertex rule would author a *duplicate*
  vertex and a zero-length closing segment, which is worse than not having the
  affordance. Two of three do it and neither of the two is answering this
  question — the majority *has never faced the surface*, which is the lesson
  `CanvasTool::Text` records from the day the same trap was set the other way
  round. **Acrobat wins, and it wins on applicability rather than head-count.**
* *SolidWorks ends the chain with `Escape`.* This shell's Escape ladder
  already means something here and it means the more transient thing: §3
  below, where `Escape` **abandons** the run rather than committing it.
  Committing on `Escape` would make the one key an operator presses to say
  *"no"* write to the document, which is the least recoverable reading of a
  key that exists to be recoverable.
* *Inkscape ends with `Enter`.* No chord is invented for it, on exactly the
  argument the operator's own zoom-to-selection decision settled: this shell's
  manifest chords are `Ctrl`-modified by construction, `Enter` is not one, and
  **keyboard input cannot be driven into this window from a harness on this
  machine**, so a key-only ending would be a way out that nothing outside the
  process can ever prove works. `markup.finish` is a control, and a control
  is clickable.

### 1.2 ★ Polygon closes and polyline does not — what that means for the last
### click and for the preview

It means **nothing at all for the gesture** and **one segment for the
preview**, and separating those two is the whole of this decision.

*For the last click*: the two kinds take the identical run of clicks and the
identical ending. The operator does not draw the closing segment for a polygon
any more than they draw the fourth side of a `/Square`; the closure is the
subtype's, applied by `pdfcer-core`'s `polygon_like(…, closed: true)`. So the
last click is the last **vertex**, in both kinds, and [`super::Geometry`]'s
own docs carry the rule that the first point is never appended again.

*For the preview*: rule 4 says the affordance must describe what will actually
commit, so a polygon's preview draws the segment from the last vertex back to
the first **and a polyline's does not**. That single segment is the entire
visible difference between the two tools while a run is in progress, and
without it an operator would have no way to tell which one they had armed —
two crosshairs, two identical runs of rubber-banded segments, and a shape that
closes only after they commit it. See [`preview`].

*And it means one more vertex is required.* [`super::action`] refuses a
two-vertex polygon where it accepts a two-vertex polyline; the argument is at
[`super::Refusal::TooFewVertices`], and the practical form of it is that
[`finishable`] answers `false` — so the ribbon's Finish is greyed after two
clicks of a polygon and live after two clicks of a polyline, which is the
difference stated where the operator can see it *before* pressing anything.

---

## 2. The state is the tool's own, and it is transient

`egui::Memory`, beside the armed tool and the gesture machine, for the reason
[`crate::canvas::tool`]'s header gives and [`crate::canvas::measure`] repeats:
this is **transient UI state**, not document state. A half-finished run is not
part of the document and a document saved mid-gesture must not carry one.

It is discarded on three transitions, each of which is a real thing an
operator does:

| transition | why the run cannot survive it |
|---|---|
| the armed **kind** changes | a polyline's vertices are not a polygon's; carrying them would close a shape the operator drew open |
| the **page** changes | a run begun on sheet 1 means nothing on sheet 2, and authoring it there would put a shape on a page it was never drawn on |
| **Escape** | §3 |

The first two are [`load`]'s two synchronisations and are lifted from
`measure::load` unchanged, including their order: the kind first, because a
kind change is what invalidates the vertices, then the page.

★ Note what is **not** on that list: retiring the tool. `disarm_markup` puts
the pen down and does not discard work, exactly as `disarm_measure` does not —
which is why [`finishable`] has to check that the tool is still armed rather
than merely that a run exists. Without that check the ribbon would offer
Finish for a run nothing is drawing any more.

---

## 3. Escape takes **two** presses, and that is the same rule the measure pick
## follows

A band drag is a `DragKind`, so `Escape` abandons it through the gesture
machine's claimant 1 and nothing new is needed. A vertex run is a sequence of
**clicks**, so there is no drag for that claimant to cancel — and yet a
polygon with three vertices taken and the fourth not is unmistakably a gesture
in flight.

So [`abandon`] takes its own rung, immediately beside
[`crate::canvas::measure::abandon`] and above the rung that retires the tool.
One press discards the run and leaves the pen armed; a second press puts the
pen down. That ordering is the ladder's own *"retire the most transient thing
first"* rule, and it is the one an operator means: a mis-aimed third click is
corrected without leaving the tool.

Without the rung, one Escape would discard the run **and** put the tool down —
two effects from one press, which is exactly what decision 025's L1 forbids.
`canvas::keys`' header carries the full precedence table.
