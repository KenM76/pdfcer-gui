# `canvas::markup` — what a markup annotation IS, and the pen it is drawn with

## The defect this module exists so that we never ship again

The old shell's `canvas.rs` records it in the doc comment of the tool
variant this one is modelled on, and it is worth carrying across verbatim
because it is the reason a markup *substrate* exists at all rather than
eight commands that each insert a shape:

> Until this variant existed, markup annotations did not go through the
> canvas at all: `Action::AddMarkupShape` called a function that derived a
> rectangle from the PAGE's own media box centre plus a per-author jitter,
> and inserted it. The shape therefore appeared in the middle of the page
> no matter where the operator had been pointing, and — because it never
> touched `active_tool` — it was invisible to every rule the other seven
> tools obey: Escape did not cancel it, it did not suppress the
> `ScrollArea`'s pan-by-drag, and it took no place in `TOOL_PRECEDENCE`.
> **The operator's report was exact: "they just drop things into the center
> of the pdf window."**

Two things in that paragraph are the whole design brief. First, a markup
command must **arm a tool**, not perform an insertion — so that the shape
lands where the pointer is and so that every rule the canvas already has
about tools (Escape, cursor, pan suppression, the gesture machine's
press/drag/release) applies to it for free rather than being re-implemented
badly. Second, a markup that appears *somewhere the operator did not point*
is not a cosmetic complaint: it is the feature not working, and it passed
whatever tests it had because a shape really was added to the document.

---


Until Ink, PolyLine and Polygon landed, this file held **one** family of
markup and its gesture together: the two-point rubber band, its preview, its
commit. That was right while there was one gesture. There are now **four**
families, and they differ in the only thing that matters here — *what the
operator does with the pointer*:

| family | gesture | module |
|---|---|---|
| Rectangle · Ellipse · Arrow · Highlight | press, drag out a band, release | [`band`] |
| PolyLine · Polygon | click, click, click, then **say when** | [`vertex`] |
| Ink | press, follow the pointer, release | [`ink`] |
| Underline · StrikeOut · Squiggly | no pointer at all — the operand is a text selection | [`text`] |

So the split is **by subject and not by line count**, and the subject is the
one this file is now left holding: *what a markup **is***. The kinds
([`MarkupKind`]), the geometry an authored markup carries ([`Geometry`]), the
one place a gesture becomes a `MarkupSpec` ([`spec`]), the one place a
completed gesture becomes an `Action` ([`action`]), the refusals, and the
**pen** every family draws with. Each submodule answers *"how is this family
gestured?"* and none of them decides what it authors.

That division is what keeps [`spec`] the single place a gesture becomes a
`MarkupSpec` — the property the whole equivalence argument rests on (§5
below) — while three genuinely different gestures feed it.

---

## The four obligations, in the shape [`crate::canvas::moving`] states them

1. **The geometry is PDF page space, never screen pixels.**
   [`band::endpoints`] and [`vertex::page_point`] are the only two places in
   this module tree that cross the boundary, and both do it through
   [`crate::viewer::canvas_to_pdf_space`] — the renderer's own transform —
   rather than by writing the Y-flip out again. A drag measured on screen and
   handed to `add_markup` compiles, runs, and merely scales with
   magnification: the same silent class as the hit-tolerance defect
   [`crate::canvas::mapping`] was built to make unavailable.
2. **The preview must describe what the release will actually commit.**
   `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 welcomes a
   pre-commit affordance — *"a snap indicator, a hover highlight, a
   rubber-band, a selection handle — these are the cursor; they describe
   what is about to happen"* — and forbids marking content that has already
   been applied. A rubber band is squarely in the first category. But it is
   only honest if it is drawn **in the shape being authored**: an ellipse
   previewed as its bounding box, or an arrow previewed as a plain segment
   with no head, misdescribes the thing the operator is about to commit. So
   [`band::draw_preview`] draws an ellipse as an ellipse and an arrow with
   its head on; [`vertex::preview`] draws a polygon's **closing segment**,
   because that segment is in the file and a polyline's is not; and
   [`ink::draw_preview`] draws the **simplified** trail rather than the raw
   one, because the simplified trail is what lands. All three draw in the
   **pen's own colour** rather than in a chrome tint, because the pen colour
   is what will land in the file.
3. **Escape abandons the gesture, and abandons exactly that.** A band drag
   and an ink drag are both a `DragKind` in [`crate::canvas::gesture`], so
   both are already Escape's claimant 1 — the *drag in flight* row of
   [`crate::canvas::keys`]'s precedence table — with no new mechanism and no
   second rule. A **vertex run** is a sequence of clicks and therefore has no
   drag for that claimant to cancel, exactly as a measure pick has none, so
   it takes its own rung beside [`crate::canvas::measure::abandon`]; see
   [`crate::canvas::keys`]'s header. Retiring the armed **tool** is a
   different act again and takes its own row.
4. **An arrow keeps its RAW endpoints.** See [`spec`]. This is the one
   decision in the module that a reader will be tempted to "tidy up", and
   tidying it up silently reverses half of all arrows the operator draws.

## A click with no drag places NOTHING for the band kinds, and that is a
## decision

The old shell answered the other way: `default_markup_at`
turned a bare click into a 120 × 60 point box centred on the pointer, with
a `MIN_DRAG` of 4 **PDF points** below which a real drag was also treated as
a click. Neither half is carried across, and the reasons are specific rather
than a matter of taste:

* **Its stated justification does not hold in this shell.** The old comment
  is explicit — the default box is *"obviously a placeholder the operator
  will resize (which slice 2 makes possible)"*. There is no slice 2 here.
  `EditSession` has the whole `move_*` family and **no scale or resize verb
  of any kind** ([`crate::canvas::handles`] consumes a grip drag and commits
  nothing), and an annotation is not even in the family those verbs address.
  So a default-sized box could not be resized, could not be moved, and could
  only be corrected by undoing it — which makes it not a placeholder but a
  wrong answer with a confident size.
* **The 4-point threshold is zoom-dependent in the wrong direction.**
  Measured in page space, 4 points is a 64 px screen drag at 16× — so a
  deliberate small mark on a title block would be silently replaced by a
  120 × 60 box, which is the same failure mode as the original centre-of-page
  defect wearing a smaller number. egui already applies the only threshold
  this gesture needs: a press-and-release that does not exceed **its** drag
  threshold is reported as `clicked` and never reaches a `DragKind` at all
  (see [`crate::canvas::gesture`]'s header). One threshold, in screen space,
  owned by the toolkit — exactly the argument
  [`crate::canvas::moving::PageDelta::is_travel`] makes for refusing a
  second one.

What a click does instead is **nothing, out loud**: [`band::drag`] is never
reached, and the tool stays armed with its crosshair, so the operator's next
gesture — a drag — does what they asked. The cost is that a click is a
no-op; the alternative is authoring a shape nobody chose the size of and
cannot change.

**The two vertex kinds are the exception, and it is not an inconsistency**:
for them a click is the *whole* gesture, so of course it does something. The
rule above is about a gesture that has a drag and did not get one. See
[`vertex`]'s header, and [`crate::canvas::gesture::press_kind`], which gives
the vertex kinds a live click and **no drag at all** — the same shape it
gives the measure tools, and for the same reason.

The same rule guards the degenerate *drag*: a press, a wander, and a release
back on the origin has zero extent on both axes, and
`pdfcer-core`'s `positive_rect` would quietly expand it to the 1-point
minimum — an invisible annotation holding a slot on the undo stack. That is
refused here as [`Refusal::NoExtent`] rather than committed, which is also
how the list-driven kinds' `EditError::EmptyGeometry` is kept off the
operator's screen: the shell never sends the engine geometry that draws
nothing, so the engine never has to refuse one. **Our guard is upstream of
theirs and is strictly the stricter of the two** — `validate_geometry`
accepts a `/Polygon` with two vertices, and [`action`] does not, because a
two-vertex closed polygon is a line drawn there and back and is not a shape
any operator meant to place.

## Which kinds are here, and which are deliberately not

[`MarkupKind`] carries **seven**, in three gesture families, and the
remaining Phase 6 kinds are absent for reasons that are each different rather
than one blanket "later":

| kind | where it is |
|---|---|
| Rectangle · Ellipse · Arrow · Highlight | here, gestured by [`band`] |
| ~~Polygon · PolyLine · Ink~~ | **Built 2026-08-14**, gestured by [`vertex`] and [`ink`] |
| ~~Underline · StrikeOut · Squiggly~~ | **Built 2026-08-14**, in [`text`], and they are still not variants of [`MarkupKind`] — see below |
| ~~Revision cloud~~ | **Built 2026-08-19.** Was *"blocked on the engine — `/BE` is never written"*, which stopped being true when `MarkupSpec::Cloud` shipped and nothing in this shell noticed for weeks. See [`MarkupKind::Cloud`]. |
| Plain line | The engine has `MarkupSpec::Line` and this shell spends it on Arrow. A second command differing only in its `/LE` is a Style question, not a kind. |
| Note · text box · sticky · stamp | Text-bearing, not geometric. A different gesture (place, then type) and a different spec type (`TextAnnotSpec`). |

### The boundary this enum draws was RESTATED when the three new kinds
### arrived, and the restatement is the useful part

It used to read: *"a variant belongs in this enum when this rubber band can
draw it"*, and on that boundary Polygon, PolyLine and Ink were excluded in
terms — *"adding the variants now would put states into the type that no
`GestureOutcome` can reach."* That was exactly right **while the band was the
only gesture**, and it is the wrong boundary now, because the thing it was
really protecting was never the band: it was the pair of properties
`shell::commands::mapping` and `app::conditions` actually assert, namely that
**every variant has a command that arms this tool and a `selected:` condition
that lights while it is armed.**

So the boundary is now stated as the property that is tested:

> A variant belongs in [`MarkupKind`] when a `markup.*` command **arms the
> canvas tool with it**.

All three new kinds clear that: each has a command, each arms
[`crate::canvas::tool::CanvasTool::Markup`], each renders pressed, and each
has a `GestureOutcome` that reaches it — a `DragKind::Markup` for Ink, and a
`GestureOutcome::Click` for the two vertex kinds, which is the same outcome
the measure tools have been reached by since Phase 7. Nothing about the old
sentence's *caution* is abandoned; only its proxy. The old wording is kept
above rather than deleted, because the mistake it guards against — variants
nothing can reach — is real, and the next reader adding a kind should be made
to show which control arms it.

## The three text-markup kinds live in [`text`], and the boundary holds


[`text`]'s own header carries the interaction decision — *select first, then
mark*, which is Acrobat's — and the mode intersection it produces.

## The names are the operator's, not the PDF specification's

`Rectangle`/`Ellipse`/`Arrow` rather than `/Square`/`/Circle`/`/Line`. The
commands are `markup.rectangle`, `markup.ellipse` and `markup.arrow`, and
`text/commands.rs` calls them Rectangle, Ellipse and Arrow to the operator.
A type that spelled them the specification's way would make the ribbon, the
trace and the code disagree about the name of the same thing for no benefit;
the mapping to the subtype lives in exactly one place, [`spec`], where the
dictionary is built.

**`PolyLine` and `Polygon` are the exception, and they are the exception
because the operator's word and the specification's word are the same word.**
Bluebeam, Acrobat and every drafting office say "polyline" and "polygon";
there is no plainer name to prefer, so the rule above simply does not bite.
`Ink` is the specification's name for what the operator calls *freehand*, and
the **label** says Freehand (`text/commands.rs`) while the type says `Ink` —
which is the same split `Rectangle`/`/Square` makes, in the other direction.

## §5 — The split between the pure rules and the wiring

[`spec`] and [`action`] are pure functions of plain data, so every rule above
is testable with no window and no document — the same discipline that makes
[`crate::canvas::moving::eligible`] and
[`crate::canvas::selection::SelectionState::click`] pure. The submodules'
entry points are the ones that touch the frame, and they do nothing except
gather inputs, call the pure functions in order, and trace what happened.

**Nothing here builds an appearance stream.** [`spec`] hands `pdfcer-core` a
`MarkupSpec` and `EditSession::add_markup` does the rest, which is the same
route `pdfcer`'s `markup-add` takes with the same value — the equivalence
the measure salvage's tests exist to protect, and the reason a canvas-authored
annotation is byte-identical to a CLI-authored one.

## Item notes

### `fn coordinates`

One iterator so the finiteness check in [`action`] is written once rather
than three times — which matters more than it looks, because the failure
of a *missed* variant is a NaN reaching an annotation's `/Rect` and the
symptom is a document some readers refuse to open.

### `const CLOUD_INTENSITY`

`pdfcer-core` accepts any finite value in `0.0..=2.0` and refuses the rest by
name (`EditError::BorderEffectIntensityOutOfRange`), so this is a *choice*
inside a legal range rather than the only value that works.

**1.0 because that is Acrobat's default cloud.** The standing tie-breaker
for anything an operator will compare against the program they are replacing
is to make it behave the way that program does; a reviewer drawing the same
cloud in both must not be able to tell them apart by the size of the
scallop. It is a `const` rather than a literal in [`spec`] so the day a
Style control for it lands, the search for "what does this replace" finds
one name and one paragraph.

### `fn all_the_same`

The vertex and ink form of *"the drag ended where it began"*. A run of forty
identical points is what a press-and-hold with no movement produces once the
duplicate filter is off, and it authors an annotation with a zero-area
`/Rect` that `pdfcer-core`'s `bounds_of` then pads to the pen's half-width —
a 1-point blob nobody chose.
