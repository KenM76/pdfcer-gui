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

## Why an operator needs this even though the grips work

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

## Why there is an Apply button and the fields do not commit as you type

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

## Why the draft is discarded when the object changes underneath it

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

**The correction is recorded rather than the comment quietly deleted**,
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

**`move_annotation` takes a DELTA and `resize_annotation` takes an
ANCHOR plus FACTORS.** Neither takes the absolute rectangle the operator
typed, so both fields are converted here, by [`annot_plan`], using the same
two helpers ([`delta`] and [`factors`]) the content half uses — one
arithmetic, two callers, so a typed move and a typed scale cannot acquire
different rounding from a dragged one.

**A ce dimension is not this section's**, and the guard is a `match` on
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

## Item notes

### `fn near`

A tolerance rather than `==`, because these values make a round trip
through an `f64` spinner and back, and `40.0` typed into a field that was
seeded with `39.999999999999996` is the operator changing nothing. Without
it, merely selecting an object and pressing Apply would raise a move of
4 × 10⁻¹⁵ points — a real undo entry, a real content-stream rewrite, and a
real cache invalidation, for an edit with no effect at any zoom.

A tenth of a point is about 35 µm on paper: finer than any plotter this
operator's drawings are printed on, and coarser than every float artefact.

### `fn delta`

The bottom-left corner is the reference on both subjects: `x` is the left
edge and `y` is the **bottom** edge, Y increasing upward, exactly as
[`crate::text::panels::properties::geometry_units_note`] tells the operator
under the heading. There is one convention in this file and it is that one.

### `fn factors`

A zero-extent axis cannot be scaled and is not an error: a horizontal line
has no height, and asking for `h_new / 0` is how a NaN reaches `move_nodes`
— or, on the annotation side, how a NaN reaches `resize_annotation`, which
refuses a non-finite factor by name (`EditError::ResizeFactorInvalid`) but
would have been asked a question nobody meant. The factor is 1 — leave that
axis alone — which is also what the operator means, because a field showing
`0.0` for a flat line is describing a fact rather than offering an edit.

### `fn field`

`DragValue` rather than a `TextEdit`, because it accepts both — an operator
can scrub it *or* click and type an exact number — and the scrubbing costs
nothing here precisely because the fields edit a draft. On a live-committing
surface a scrubbable field would be the eighty-undo-entries problem the
module header describes; on a drafted one it is a free second input method.

`disabled` is `Some(reason)` when the control must be drawn and dead —
today, a locked annotation. R9 requires the reason on the hover, and it is
attached to **each field** rather than to a wrapper because
`add_enabled_ui` produces no response to hang a hover on: an operator
pointing at the greyed Width would get nothing, which is the dead control
with no explanation this project keeps finding.

### `const SPEED`

Half a point, so a hundred-pixel drag spans fifty points — about the range a
draughtsman adjusts a symbol by — and so the value visibly moves on a slow
drag rather than jumping. A driven check depends on this being **exact**:
scrubbing `n` pixels changes the field by `n × SPEED`, which is what lets the
harness assert the number it expects instead of merely that something
changed.

### `fn an_untouched_draft_plans_nothing`

The float round trip through the spinner is why this needs a test rather
than being obvious: a seed of `39.999999999999996` and a typed `40.0`
are different `f64`s and the same edit. Without the tolerance this would
raise a move of 4 × 10⁻¹⁵ points — a real undo entry and a real
content-stream rewrite for a change no zoom can show.

### `fn a_zero_height_object_scales_only_the_axis_it_has`

A horizontal line has zero height, and the obvious implementation
computes `h_new / 0`. `move_nodes` would accept the resulting NaN
coordinates and write them into the content stream, producing a page
that no viewer — including this one — can render.

### `fn an_annotation_and_a_content_object_with_the_same_number_do_not_share_a_draft`

This is the assertion the [`Subject`] enum exists for, and the defect it
prevents is a *coincidence of integers*: page-content objects are
numbered by paint order (0, 1, 2 …) and annotations by object id, whose
`num` is a small integer on almost every real document. With the stamp's
middle member a bare `usize`, selecting content object 7 and then
annotation `7 0 R` on the same page in the same epoch left `sync`
believing the draft already described the new selection — so the panel
showed the path's numbers over the annotation and Apply moved it by the
difference between two unrelated rectangles.

⇒ Not reproducible on most documents, so a fixture-based test would pass
and the operator would report it once and never again.

**Falsified** by changing the stamp back to `(usize, usize, u64)` and
keying on `id.num`: this went red on the `x` assertion — the draft kept
the content object's `999` — and every other test in this file stayed
green. Restored.

### `fn a_typed_position_becomes_a_delta_and_no_resize`

The field holds an absolute coordinate and the verb takes a
displacement, so the conversion is the whole content of this arm. A plan
that handed `draft.x` straight to the verb would move the annotation to
`x + draft.x` — off the sheet on any real drawing, and *further* off it
the further right the mark already was, which reads as a random jump.

### `fn a_typed_size_becomes_an_anchor_and_factors`

The anchor is the corner the operator pinned with Left and Bottom, so
the box grows to the right and upward — which is what "X, Y, W, H"
means in every properties panel: X and Y name a corner, and changing W
moves the *other* edge.

### `fn moving_and_resizing_anchors_on_the_typed_corner`

`resize_annotation`'s anchor is an absolute point, so this is sharper
than the content arm's equivalent: a factor tolerates a stale origin and
a point does not. Anchoring on `bounds.x0` here would pin a corner the
annotation is about to stop having, and the mark would land somewhere
neither number described.

**Falsified** by anchoring on `(bounds.x0, bounds.y0)`: red, and the
two single-change tests above stayed green — which is why this case
needs its own test rather than being implied by them.

### `fn a_flat_annotation_scales_only_the_axis_it_has`

A `/Line` drawn perfectly horizontally has a degenerate `/Rect`, and the
obvious implementation computes `h_new / 0`. `resize_annotation` refuses
a non-finite factor by name — `EditError::ResizeFactorInvalid` — so this
would surface as a worded refusal rather than as corruption, but it
would be a refusal for a question nobody asked.

### `fn the_two_arms_agree_about_what_the_operator_typed`

The failure this pins is not a crash: it is a mark placed by typing
landing a fraction of a point away from the same mark placed by
dragging, because one arm rounded through `f32` on the way to its verb
and the other did not. Nothing on screen would show it and nothing else
in this suite would catch it.

The content plan's factors are `f32` — `move_nodes` takes those — so
the comparison is at `f32` precision, which is the honest bound: what is
asserted is that the two arms agree to the precision the narrower one
can express, not that a widened `f32` equals an `f64`.
