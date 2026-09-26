# `canvas::annotnodes::ink` — **a freehand mark's points, addressed two
ways at once**

## The operator's report, and the day it stopped being a refusal

> *"the draw a line that follows the pointer tool — I can't edit the nodes
> that make it"* (O158, 2026-09-08)

The freehand tool authors an `/Ink` annotation, and until `pdfcer-core`
`Pass 278.0` (`c8a6697`, 2026-09-09) that was the one markup kind whose
points the engine could *read* (`Annotation::ink_list`) and could not
*change* — `reshape_annotation` refused it by name. The refusal had been a
decision argued from Acrobat, and the engine's reply overturned it in as
many words: *"Parity with Acrobat is this project's floor, not its
ceiling."* What shipped is a second verb family, not a widened first one:

```text
EditSession::reshape_ink(annot, &InkEdit::…, modified)   // the planner
EditSession::reshape_ink_preview(annot, &InkEdit::…)     // its preflight
EditSession::move_ink_point   / insert_ink_point / remove_ink_point
EditSession::replace_ink_stroke / move_ink_stroke / remove_ink_stroke
```

This module is the **addressing** half of consuming it. Everything about
*asking the engine*, *previewing* and *committing* stays in [`super`], which
treats an ink stroke exactly as it treats a polyline once the addressing is
settled — and the engine says that is correct rather than approximate:

> pdfcer draws an `/InkList` as a **polyline**, not as a smoothed curve …
> a point drag moves exactly the two segments either side of it, and a
> front end that previews with a polyline is exactly right.

## Two index spaces, one anchor list

`/InkList` (§12.5.6.13, Table 182) is an array **of** arrays — one point
list per stroke — so the engine addresses a point as `(stroke, point)` and
every [`pdfcer_core::edit::InkEdit`] variant carries both. The canvas,
though, has **one** anchor list: `canvas::painting` enumerates whatever
[`super::nodes`] returns and publishes anchor *n* as `canvas.markup-node.n`;
`canvas::pressing` resolves a press to one `usize`; the gesture machine
carries one `usize` for the life of the drag. Teaching all of those a pair
would have put a sentence about `/Ink` into five modules that have nothing
to do with it — which is exactly the argument the engine made for **not**
widening `VertexEdit` with an optional stroke index.

⇒ So the flat index stays the canvas's currency, and [`StrokeTable`] is the
**side table** that converts it. Every point of every stroke is one anchor,
in file order — stroke 0's points, then stroke 1's, and so on — and the
table remembers where each stroke starts so that

* [`StrokeTable::address`] turns the anchor the operator grabbed into the
  `(stroke, point)` the engine wants, and
* [`StrokeTable::flat`] turns an engine address back into an anchor, which
  is what a test uses to prove the round trip and what a driven check will
  use to aim.

## Stroke boundaries are load-bearing in two places

A flat list loses one fact a list of lists carries — *where one stroke ends
and the next begins* — and two things go wrong the moment it is lost:

1. **The preview draws a bridge.** `super::preview_of` joins consecutive
   points; across a stroke boundary that is a segment from the end of one
   pen stroke to the start of the next, which the file does not contain and
   the release would not commit. The honesty contract in [`super`]'s header
   — *the preview is a shape the release would commit, or it is the shape
   already there* — forbids it. [`StrokeTable::segment_pairs`] yields only
   within-stroke pairs, so there is nothing to bridge.
2. **"Insert after" crosses into the next stroke.** A flat
   `insert(index + 1)` at the last point of stroke 0 would — in a flat
   list — land the new point where stroke 1's first point is. The engine's
   rule is the opposite and is stated on `InkEdit::InsertPoint`: *"`after
   == len - 1` extends the stroke past its current end, which is the 'keep
   drawing where I stopped' gesture."* [`StrokeTable::after_edit`] grows
   the **grabbed** stroke by one, so the flat list and the table agree that
   the new point belongs to the stroke whose end was grabbed.

Both rules are tested in `super::tests` against a two-stroke `/Ink`
authored through the real engine, at the first point of the second stroke
— the one place a flat-minus-one or a flat-plus-one error is visible.

## Anchor density — every point is drawn, and that is a known follow-up

A freehand stroke can hold hundreds of points; `canvas::markup::ink`
simplifies on authoring, but a stroke from another producer arrives as it
was saved. The engine's reply put the decision here — *"anchor density and
decimation are yours; both grains are ours"* — and this module draws **every
point**, deliberately, for now:

* A decimated anchor set would have to be a **second** index space — the
  anchor the operator grabbed would not be a point the engine knows — and
  the mapping from it back to `(stroke, point)` is precisely the kind of
  derived-twice geometry this canvas has shipped wrong before.
* The engine offers the right primitive for the density problem,
  `InkEdit::ReplaceStroke` through `EditSession::replace_ink_stroke`
  (*"simplify once, then edit per-point"*), and offered to add a
  stroke-level `simplify` in core. Doing it there keeps one simplifier in
  the ecosystem; `canvas::markup::ink` already has one for authoring and a
  third copy would be the drift.
* Nothing the operator can do with a dense anchor set is *wrong* — each
  anchor is a real point and every drag commits — so the cost is visual
  clutter rather than a defect, and R9 has no rule against a control that
  works.

⇒ `InkEdit::ReplaceStroke`, `InkEdit::MoveStroke` and
`InkEdit::RemoveStroke` — and their wrappers `replace_ink_stroke`,
`move_ink_stroke`, `remove_ink_stroke` — are **not called** by this shell
yet. The whole-stroke grain is the natural home for a decimation pass and
for a *"remove this stroke"* menu row, and `EDITABLE_SURFACES.md` carries the
register entry.

## Rule 15

An `/Ink` is a **markup shape**. It is never a ce dimension and never page
content, and [`super::geometry`] has already refused anything that is not
[`crate::canvas::selection::AnnotKind::Markup`] before this table is built.

## Item notes

### `struct StrokeTable`

Holds one length per stroke, in file order. Starts are derived (a prefix
sum) rather than stored beside the lengths, so there is one number per
stroke and no pair of numbers that has to agree.

A stroke of length 0 or 1 is kept rather than dropped. `Annotation::
ink_list` reads a malformed stroke as *empty* precisely so that stroke
indices stay aligned with the file's — its doc says so — and a table that
dropped it would put every later stroke one index off from what the engine
calls it. The one-point stroke draws no anchor-to-anchor segment and its
single anchor is draggable, which is the honest picture of what the file
holds.

### `fn address`

`None` for an index past the end of the list, which is a press the
painter could not have drawn an anchor for; the caller treats it as
the engine would treat a bad index — a refusal, never a panic.

### `fn segment_pairs`

This is the one function that knows where a stroke ends, and both the
preview painter and the right-click segment pick read it, so the
segment a menu offers *"Add a point here"* on is always one the preview
draws. A bridging pair here would be a segment the file does not hold.

### `fn after_edit`

* a **move** changes no length;
* an **insert** grows the grabbed point's stroke by one — the engine's
  rule that inserting after a stroke's last point *extends that stroke*
  rather than starting or joining another;
* a **remove** shrinks it by one.

`None` for an index the table does not hold, matching
[`Self::address`].

### `fn plan`

The displacement of a move is measured from `from` — the point as it
stands — to `target`, exactly as `super::planned` measures a
`VertexEdit::Move`, so the two families cannot disagree about what a
delta is.

`None` for a flat index the table does not hold. The engine would also
refuse that (`InkPointIndexOutOfRange`), but it cannot be *asked* about
an address this table cannot produce, so the caller words the refusal
itself.
