# `panels::properties::geometry` — **X, Y, W and H, typed rather than
dragged**

## What this closes

`FEATURES.md`'s Phase 1 remainder, verbatim:

> **Editable geometry** — X/Y/W/H in the Properties panel, typed rather
> than dragged.

And it closes it *because the resize gesture landed first*. Every number in
this section is a call into machinery that already exists and is already
tested: a position change is [`VectorAction::MoveSelection.into()`], the same variant a
move drag raises, and a size change is [`crate::canvas::resizing::action`],
the same function the eight grips raise. **This module computes two scale
factors and a delta and contributes no geometry of its own.**

That is the whole design and it is deliberate. A properties panel that
reimplemented "make it 40 points wide" would be a second scale
implementation with a second set of rounding, a second pivot convention and
a second answer to *what happens to line weights* — and the two would drift,
silently, because nothing compares them.

## ★ Why an operator needs this even though the grips work

Because a grip cannot express *exactly 40.0 points*. The resize gesture is
excellent for "about this big" and incapable of "the same as the one above
it", and a drawing is full of the second kind. It is also the only route
for a **small** object: at fit-page zoom on an ISO A1 sheet a 6 mm symbol is
about four pixels across, so its eight grips overlap each other and the
gesture is unusable at the zoom where the operator can see the sheet.

It is additionally the only **accessible** route to a resize. A gesture that
requires a sub-pixel drag is a gesture some operators cannot perform, and
`MODES_AND_PANELS.md` §7's accessibility line asks for a typed equivalent to
every direct-manipulation edit for exactly that reason.

## ★★ Why there is an Apply button and the fields do not commit as you type

Because **every commit is an undo entry**, and a `DragValue` the operator
scrubs from 40 to 120 would raise eighty of them. That is not a theoretical
objection: `app::actions`' `MoveNodes` doc comment makes the identical
argument about looping the singular verbs — *"N undo entries for one drag,
and each planned against byte offsets the previous one invalidated"* — and a
live-committing spinner is that loop with a nicer face on it.

So the four fields edit a **draft**, and one press turns the draft into at
most two commands. Two rather than one because a move and a scale are
different verbs in `EditSession` and this shell does not have a combined
one; the operator who changes only X gets exactly one entry, and the
operator who changes X and W gets two, in that order, which is the order
that makes the second one's pivot mean what the preview said.

## ★ Why the draft is discarded when the object changes underneath it

The draft is stamped with `(page, object, edit epoch)`. If any of the three
moves — the operator selects something else, or *anything at all* edits the
document — the draft is dropped and re-seeded from the object's current
bounds.

The epoch is the one that matters and it is the one that is easy to leave
out. Without it, this sequence silently destroys work: type `W = 40`, do not
press Apply, press `Ctrl+Z` to undo something unrelated, press Apply. The
draft would still hold the numbers computed against the *pre-undo* bounds,
and the scale factor would be `40 / (a width that no longer exists)`. The
object would end up some third size that the operator never typed and cannot
predict. Re-seeding on the epoch makes that unrepresentable rather than
merely unlikely.


This section drew **nothing at all** over a selected annotation, and said
why, in a comment at the head of [`section`] which is reproduced here in
full because the shape of the correction is the useful part:

> *An annotation's geometry is its `/Rect`, which no verb in this build
> rewrites — see `FEATURES.md`'s Format-tab row. Showing editable X/Y/W/H
> over one would be a control that accepts a value and discards it.*

**That was true when it was written and stopped being true.**
`EditSession::move_annotation` and `EditSession::resize_annotation` both
ship, and this shell was *already calling both of them* —
`canvas::annotdrag` raises
[`AnnotAction::Move`](crate::app::actions::annot::AnnotAction::Move) on the
release of a drag and `canvas::resizing` raises
[`AnnotAction::Resize`](crate::app::actions::annot::AnnotAction::Resize) on
the release of a grip. So the refusal above had outlived its premise by the
width of two verbs, and the operator could **drag** a mark to a place and
could not **type** one.

⇒ On an ISO A1 CAD sheet, typing the number is frequently the only accurate
route. The module header's own argument for the content half applies
unchanged and harder: a revision cloud that must sit exactly 25 mm inside
the title block cannot be placed by hand at fit-page zoom, where 25 mm is
about seven pixels.

★★ **The correction is recorded rather than the comment quietly deleted**,
which is this project's standing practice: a claim that was right and
expired teaches something a clean file does not — that a refusal written
against a missing capability must name the capability, so that whoever adds
it can grep for the refusals it unblocks. That comment named the verb
("no verb in this build rewrites `/Rect`") and was still not found for the
nine days between `move_annotation` landing and this change.

### What the annotation half shares, and what it cannot

| | page-content object | markup annotation |
|---|---|---|
| the four numbers | anchors' bounding box | the annotation's `/Rect`, normalised |
| the Y convention | **bottom edge, Y up** | *the same* — [`crate::text::panels::properties::geometry_units_note`] is drawn once and describes both |
| a move | `VectorAction::MoveSelection`, a delta | `AnnotAction::Move`, a delta |
| a resize | `resizing::action`, pivot + factors | `AnnotAction::Resize`, anchor + factors |
| refusals | off-canvas, not a path, no node model | **locked** (`/F` bit 8), and the engine's foreign-appearance refusal |

★★★ **`move_annotation` takes a DELTA and `resize_annotation` takes an
ANCHOR plus FACTORS.** Neither takes the absolute rectangle the operator
typed, so both fields are converted here, by [`annot_plan`], using the same
two helpers ([`delta`] and [`factors`]) the content half uses — one
arithmetic, two callers, so a typed move and a typed scale cannot acquire
different rounding from a dragged one.

★★ **A ce dimension is not this section's**, and the guard is a `match` on
[`AnnotKind`](crate::canvas::selection::AnnotKind) rather than a comparison
of `/Subtype` strings. `pdfcer-core` refuses both verbs by name for one —
*"a ce dimension must RE-MEASURE when it moves"* — and points at
`move_dimension` and `move_dimension_vertex`, which are
[`super::dimension`]'s to reach. A string test would compile, would read
`"Line"` for a dimension exactly as it does for a plain line, and would
route a measurement into a verb that scales its rectangle and leaves the
number it displays saying something else.

## What this deliberately does NOT offer

- **Rotation.** `EditSession` has no rotate verb, and expressing one as
  `move_nodes` is not the same edit: it would rotate the anchors and leave
  every glyph, dash pattern and line cap in the original orientation. That
  is a shear of the outline, not a rotation of the object.
- **A units picker.** Every number here is in **PDF user-space points**,
  which is what `move_objects` and `move_nodes` take and what the rest of
  this panel already shows. A millimetre field would be a conversion this
  module owns, and the measure tools already own a scale model with a
  *different* answer — a drawing at 1:50 has a page millimetre and a world
  millimetre and they are not the same length. Two conversions, one label.
- **Multi-object geometry.** The same refusal `resizing` makes and for the
  same reason: `move_nodes` addresses one object, and *"set both of these to
  40 wide"* is a different feature (align/distribute) with a different
  surface.
