# `canvas::markup::vertex` — polyline and polygon, and the gesture the
operator has to end

The two **click-shaped** markup kinds: what one click does to the run, what
the two endings do, and what the preview has to draw that a band's does not.
[`super`] owns what they author; this file owns when.

---

## 1. THE ENDING IS A SOLVED PROBLEM IN THIS CODEBASE, AND IT IS SOLVED
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

### 1.2 Polygon closes and polyline does not — what that means for the last
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

Note what is **not** on that list: retiring the tool. `disarm_markup` puts
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

## Item notes

### `fn load`

The two synchronisations of §2, in the order that matters: the **kind** first,
because a kind change is what invalidates the vertices, then the **page**.
Lifted from `measure::load`, whose own comment carries the ordering argument.

### `fn to_screen`

The same pair `measure::page_to_screen` makes, and it is spelled here rather
than shared because that one is `measure`-private and the two modules are
otherwise independent. **Both** hops matter: `viewer::pdf_space_to_canvas`
lands in *canvas* space — page top-left origin, no zoom — and the painter
speaks screen, so a preview that stopped after the first hop would draw every
segment offset by wherever the page sat in the window and at 100 % whatever
the magnification. That is the defect `measure::page_to_screen`'s own docs
record, shipped once already.

### `fn each_click_adds_a_vertex_and_authors_nothing`

The half a build that committed on every click would fail, and the half
that makes the double-click mean anything: if a click already authored,
there would be nothing for an ending to end.

### `fn a_double_click_finishes_and_the_first_click_of_the_pair_still_counts`

The reading §1's "order of the two questions" argues for, asserted as a
count: a build that swallowed both clicks of the pair would place two and
the operator would lose their last corner; a build that placed a vertex
on the double as well would place four, with the last two coincident.

### `fn the_double_click_and_the_command_author_the_same_annotation`

The property the one-commit-path design exists for, asserted the only way
that means anything: run *both* endings over identical runs and compare
the actions they raise. Two arms that each built a `Geometry::Vertices`
would agree on the day they were written, drift on the first change to
either, and the operator would have no way to see it — a polygon drawn
from the same clicks looks the same whichever code wrote it.

### `fn finish_lights_after_two_clicks_for_a_polyline_and_three_for_a_polygon`

§1.2's third consequence, at the surface the operator reads: after two
clicks the ribbon's Finish is live for a polyline and greyed for a
polygon, because a two-vertex closed shape is a line drawn there and back.
Asserted through `finishable`, which is the condition the control is
registered against, rather than through `action` — the rule is only worth
anything if it reaches the button.

### `fn finish_needs_the_tool_armed_and_asking_creates_nothing`

The fourth row is the one that is easy to miss: putting the pen down does
**not** discard the run (§2), so without the armed-tool check the ribbon
would keep offering Finish for a run nothing is drawing any more. The last
assertion is `measure`'s
`asking_whether_finish_is_available_creates_no_measure_state` for this
tool: `finishable` runs on every frame, for every document, and a version
that went through `load` would leave a run in memory for a tool nobody
armed.

### `fn changing_kind_or_page_discards_the_run`

The failure without them is the one `MeasureState::set_kind`'s docs name:
not an error, but *"something strange"* on the operator's next click — a
polygon closing over vertices they drew as a polyline, or a shape landing
on a sheet they had paged away from.

### `fn escape_abandons_a_run_and_says_whether_it_took_the_key`

Both halves. The `false` with nothing in progress is the load-bearing one:
without it Escape would be consumed by a tool that has nothing to abandon,
and the ladder below would need two presses to move one rung.

### `struct VertexRun`

Page-space, not canvas-space, and that is the same decision
[`crate::canvas::measure::state::MeasureState`] makes about its picks: a
vertex has an *absolute place on the page*, so storing it in the frame's
canvas coordinates would make it silently zoom-dependent — the class of defect
`canvas::mapping`'s header exists to make unavailable. The conversion happens
once, at the click, in [`page_point`]; the preview converts back the same way
a committed dimension is drawn.

### `fn read`

The half of [`load`] that has no side effects, for the two callers that must
not manufacture a run merely by asking: [`finishable`] is evaluated on **every
frame** by the ribbon, and [`finish`]'s arm may be reached by a chord with no
tool armed at all. `load` would build a fresh `VertexRun` for either, which
the next `store` would then persist — leaving the canvas holding a run for a
tool nobody armed. `measure::read`'s own docs make the identical argument, and
`measure`'s `asking_whether_finish_is_available_creates_no_measure_state` is
the test that caught the shape of it there.

`pub` for `crate::panels::tool`'s stage row — *"3 corners placed.
Double-click the last one to finish."* That is the one place a live vertex
count may be rendered: a number floated near the pointer would be
pdfcer putting a surface over the drawing on its own initiative, which
`MODES_AND_PANELS.md` sets to **never**, and the count is a real need because
a polygon and a revision cloud both refuse at two corners.

The read-only-ness is what makes widening it safe. Every caution above is
about **`load`**, which manufactures a run; this observes one and cannot
create state.

### `fn page_point`

The vertex family's half of obligation 1. Through
[`viewer::canvas_to_pdf_space`] — the renderer's own page transform — for
exactly the reason [`super::band::endpoints`] states at length: writing any
part of the crop-box origin, the `/Rotate` or the Y flip out here would be a
second derivation of the page transform, and the symptom would be a polygon
mirrored about the page's horizontal centre line that the operator finds after
saving.

`None` for a page whose device transform cannot be inverted.

### `fn commit`

Pure over the run and the action list — no `egui`, no context, no memory —
which is what makes both endings assertable without a window, and which is why
this function exists rather than two arms that each build a
[`super::Geometry::Vertices`]. See §1.

Returns `false` and raises nothing when [`super::action`] refuses the run —
too few vertices for the kind, or no extent at all. That refusal is the same
one [`finishable`] asks, so the ribbon's control cannot be live while pressing
it would do nothing.

**The run is emptied on success and kept on refusal**, which is the same
asymmetry `measure::circular::commit` has and for the same reason: a second
Finish must not author the same shape again from a run the operator believes
they have spent, and a refused run must survive so they can add the vertex
that was missing.

### `fn click`

The whole of the gesture's input, called from `canvas::interact`'s `Click`
arm when [`MarkupKind::is_vertex`] says one of these two is armed. That arm is
the same one that would otherwise hit-test for a selection, so a vertex click
and a selecting click are mutually exclusive by construction rather than by a
guard either could forget — the property
[`crate::canvas::gesture::press_kind`] establishes by giving these kinds a
live click and **no drag**.

# The order of the two questions

A **double**-click finishes and places no further vertex. The first click of
the pair has already been through here as an ordinary click and has already
placed its vertex — which is the right reading rather than an accident of how
`egui` reports the pair, and it is `measure::circular::click`'s reading too.
The alternative, swallowing both clicks of the pair, would make the operator's
*last* vertex need a separate click and then a double-click somewhere
harmless — i.e. it would require them to click somewhere they did not want a
vertex in order to say they had finished.

So `click, click, double-click` places **three** vertices and commits: A, B,
then the first click of the pair as C, then the second as the ending.

# Nothing is hit-tested

Unlike the circular pick, which toggles *objects* into a fit, a vertex is a
**point**: it lands where the operator clicked and no decomposition is
consulted. That is why `canvas::interact`'s `needs_targets` does not grow a
term for these two — a polygon drawn over the 129,758-object benchmark sheet
decomposes nothing.

### `fn finishable`

Published by `crate::app::PdfcerApp::conditions` and read by `markup.finish`'s
`enabled_when`. Three conditions, and each rules out a state that really
occurs:

1. **A vertex tool is armed.** The run outlives disarming (§2), so without
   this the ribbon would offer Finish for a run nothing is drawing.
2. **A run exists** and belongs to the armed kind.
3. **[`super::action`] would accept it** — the *same* derivation [`commit`]
   uses, so the control is live exactly when pressing it authors something.
   Two spellings of "is there something to finish?" would eventually
   disagree, and the way they would disagree is the worst available: an
   enabled control that does nothing when pressed, which is precisely the
   placeholder the no-placeholders invariant forbids.

Condition 3 is what makes the polygon/polyline difference visible **before**
the operator presses anything: two clicks leave Finish live for a polyline and
greyed for a polygon, because a two-vertex polygon is a line drawn there and
back.

### `fn finish`

The second entrance to [`commit`], and the only thing it adds is the trip
through `egui::Memory`. The page comes from the **run**, not from the current
view, because the vertices were clicked on that page — reading
`doc.view.page_index` here would be a second source of truth for a fact the
run already carries, and `load` discards a run whose page has been left behind
on the next frame anyway.

Returns `false` when there is nothing to finish, so the dispatcher can say
which kind of nothing happened rather than tracing a success it did not have.

### `fn abandon`

Escape's claimant — see §3. It sits *below* the drag-in-flight rung and
*above* the rung that retires the tool, exactly where
[`crate::canvas::measure::abandon`] sits, and for the identical reason: the
run is the more transient of the two things a press could mean, and a
mis-aimed click should be correctable without leaving the tool.

Returns `false` when there is no run, so the key falls through to the next
claimant rather than being silently eaten by a tool that has nothing to
abandon.

### `fn preview`

Rule 4's pre-commit affordance, and for this gesture it is the *whole* of the
feedback: a vertex run writes nothing to the document until it is finished, so
without this function a click changes nothing on screen at all and the tool is
a gesture the operator cannot aim.

Three things are drawn, and each answers a question the other two cannot:

1. **The committed run**, segment by segment, from the vertices themselves —
   so the operator can see the figure they have built.
2. **The rubber segment** from the last vertex to the pointer, which is what
   the next click would add. Absent when the pointer has left the widget,
   which is honest: there is no next click to describe.
3. **The closing segment, for a polygon only** — from the last vertex back
   to the first. §1.2: the closure is in the file and the operator never draws
   it, so a preview that omitted it would describe a polyline while a polygon
   was being authored, and the two tools would be visually identical until the
   moment of commit.

The whole preview is drawn in the **pen's** colour and width
([`super::pen_color`], [`super::pen_px`]) rather than in a chrome tint, for the
reason [`super::band::draw_preview`] states: what is painted here is the thing
that is about to be written into the file.

### `fn plant_run_for_test`

`canvas::keys` owns Escape's precedence and has to assert that a vertex run is
abandoned one press *before* the tool is put down; `app::conditions` has to
assert that a finishable run is still not offered with no document open.
Neither can assemble one the real way — that needs a laid-out page and a real
click — so the state they must react to is planted directly, exactly as
`measure::circular::plant_pick_for_test` and `guides::plant_drag_for_test` do.

`#[cfg(test)]` so it cannot become a second way for production code to build a
run. The real one is [`click`], and a second entry point is how two code paths
come to disagree about what a vertex is.

The three points are a non-degenerate triangle, so the planted run is one
[`finishable`] answers `true` for under **either** kind — a two-point run
would make every polygon assertion pass for the wrong reason.

### `fn plant_short_run_for_test`

[`plant_run_for_test`]'s deliberate other half. The three-point run is
finishable under both kinds, which is what makes it a good fixture for the
Escape ladder and a useless one for the difference between the two tools —
and that difference is the thing `app::conditions` has to assert reaches the
ribbon. Two fixtures rather than a parameter, so a test that wants "a run that
is finishable" and a test that wants "a run that is finishable for exactly one
of the two kinds" name what they mean.
