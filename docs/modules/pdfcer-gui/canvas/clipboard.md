# `canvas::clipboard` — **cut, copy and paste on the canvas**

## What this closes

The operator, 2026-08-19: *"also the standard copy/paste and I didn't try cut
so possibly that one too aren't implemented."*

They were not. `Ctrl+C` copied **text** — a swept range, through
`canvas::textsel::clipboard` — and that was the whole of this shell's
clipboard. `Ctrl+X` and `Ctrl+V` did nothing anywhere, and no ribbon control
offered any of the three: `RIBBON_IA.md`'s Edit ▸ Clipboard group had been
deleted rather than shipped empty, on the correct P3 grounds that a caption
over nothing is worse than no caption.

## What is expressible, and what is not — measured, not assumed

**This table was written 2026-08-19 and every "blocked" row in it has since
expired. Corrected in place on 2026-09-05 rather than left standing beside
its correction, per R5.** The 2026-08-19 reading — *"markup can be copied
through `spec_from_dict`/`add_markup`, page content cannot be put back at
all"* — was true when taken and is kept only in this sentence, because its
shape is the lesson: **a capability claim about another crate is a dated
citation with a shelf life measured in hours.**



**What the annotation row changed, in the operator's terms.** Until
2026-09-05 this module copied an annotation by reading a `MarkupSpec` out of
its dictionary and authoring a *new* annotation from it — so the eight
subtypes `pdfcer-core` models could be copied and **everything else could
not**. A sticky note, a stamp, a text box, a link and a file attachment all
answered `Ctrl+C` with *"that annotation is not one pdfcer authors."* A
sticky note is the most-copied comment in a review workflow.

**And the lossless route turned out to be lossy in the other
direction.** `copy_selection` carries a markup pdfcer *models* as a spec and
plants it with `add_markup` — not `add_markup_with` — so it drops `/CA`,
`/T`, `/M` and `/Contents` on exactly the kinds this module could already
copy faithfully. The fork that keeps both halves is
[`crate::canvas::annotclip`], whose header carries the whole account and the
`file:line` for every claim in it. **Read that before changing anything
here.**

## A mixed marquee — content AND annotations in one gesture

The copy below is **one call to `copy_selection` with both index lists**,
which is the engine's own prescription: *"the verb a shell should call when
a marquee caught both — which on a marked-up drawing is the ordinary case,
not the exotic one."*

⇒ **It cannot be reached from the canvas today, and the reason is the
selection model rather than the clipboard.**
`canvas::selection::SelectionState` holds `annot: Option<AnnotSelection>`
and makes the two mutually exclusive by construction — *"One canvas, one
selection"* — so a marquee that sweeps a line and a revision cloud selects
the line and drops the cloud before `Ctrl+C` is ever pressed. Nothing in
this file can change that, and nothing in this file pretends otherwise: the
clipboard is the half that is ready. Recorded on the clipboard row of
`ENGINE_BACKLOG.md` and in [`crate::canvas::annotclip::selected`].

## Why the clipboard is in `egui::Memory` and not the OS clipboard

Because a `MarkupSpec` is not text and the OS clipboard carries bytes with a
declared format. Putting one there would mean inventing a pdfcer-specific
flavour, which is a real feature (it is how you would paste between two
pdfcer windows) and is not what was asked for. What was asked for is
*"copy this cloud onto sheet 12"*, which is one process.

It is **application-scoped**, like the armed tool and the text pen: a spec
copied in one document pastes into the next one opened. That is what every
editor does and it is the behaviour that makes copying between two drawings
possible at all — this shell opens one document at a time, so a
document-scoped clipboard would make cross-drawing copying impossible rather
than merely awkward.

## Where the paste lands, and why it is not "in place"

Offset by [`PASTE_OFFSET_PT`], down and to the right, **except** when the
paste is onto a different page — where it lands at the original coordinates.

Both halves are the convention and both have a reason:

- **Same page → offset.** A paste that landed exactly on the original is
  invisible: the operator presses `Ctrl+V`, sees no change, presses it four
  more times, and has five stacked copies they cannot separate. Every editor
  offsets for this reason.
- **Different page → in place.** The whole point of copying a revision cloud
  to sheet 12 is that it should be *where it was on sheet 1*. Offsetting
  would move it for no reason the operator asked for, and they would have to
  drag it back.

## What `Ctrl+C` does when text is swept

**Text wins.** `canvas::textsel::clipboard` owns `Ctrl+C` and keeps it: a
swept range is a more specific statement than a selected annotation, the
operator made it more recently, and every program in the class resolves the
collision the same way. This module's copy runs only when no text is swept.

## Item notes

### `fn paste_clip`

# The offset rule is the markup one, and the geometry is not

Same page offsets so the copy is visible; a different page or document lands
in place, so a shape copied to sheet 12 is where it was on sheet 1. That
rule is shared with [`paste`] deliberately — two answers to *"where does a
paste land"* would be two things for the operator to learn.

What is **not** shared is how the offset is expressed. A markup carries a
`/Rect` and moves by a pair of numbers; page content moves by a **page-space
matrix**, which is the same contract `transform_objects` takes and the same
reason: `cm` composes into the CTM in force at that point in the stream, so
the engine conjugates by each item's own captured matrix and the caller
passes page space or nothing.

`Matrix::IDENTITY` is paste-in-place; `translate` is paste-with-offset. That
the same verb also gives paste-scaled and paste-rotated through
`Matrix::about` is why the request asked for a matrix rather than a
displacement, and it is what a future *paste special* is already built on.

# Errors

None today — the deserialisation happens in the apply arm, where the session
is. A clip this shell wrote is a clip this shell can read; one it cannot is
the engine's `ClipError::NotAClip`, and that reaches the status row through
`vector_edit` like every other engine refusal.
