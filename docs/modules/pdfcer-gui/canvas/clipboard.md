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

### `fn text_owns_the_chord`

Defect O18's enforcement point. **This module's header has always**
*claimed* that text wins; this is the function that makes the claim true,
and it lives here, beside the claim, rather than beside the caller that
consults it — so a reader who arrives at the header's promise finds its
enforcement in the same file rather than having to trust it.

It also has to live somewhere other than `app::dispatch`, which R2's
1,500-line ceiling put over the limit the moment this was added. That was
the forcing function; this is the right home independently of it.

# The two claimants, and why both count

| claimant | what the operator did | who copies it |
|---|---|---|
| a composing draft | put a caret in a text box, possibly with a selection | `canvas::textedit::keys` |
| a live text sweep | dragged across text on the page | `canvas::textsel::clipboard` |

A draft counts **even with no selection inside it**, and that is deliberate.
`canvas::textedit::composing` is also true for a focused ordinary widget —
the Find field, the status bar's page box — and an operator pressing Ctrl+C
in the Find field is copying their search term. Letting the object clipboard
answer that keystroke would copy something off the page instead, which is
`DEFECTS.md` D1's failure exactly: a canvas taking a chord that belonged to
the widget the operator was looking at.

# Why "the sweep produced no text" is not checked here

A live-but-empty selection still counts as text owning the chord. The
alternative — falling through to the object clipboard when the sweep turns
out to be empty — would make one keystroke mean two different things
depending on a property the operator cannot see, and the failure would be
silent and destructive on the cut path. `textsel::clipboard::copy` refuses an
empty string on its own and traces the refusal; that is the right place for
it, because it is the only place that has the string.

### `const PASTE_OFFSET_PT`

Ten — a little over three millimetres. Large enough to be unmistakable at
fit-page zoom on an A1 sheet (where it is about four screen pixels, which is
small but is a visible step against a hairline), and small enough that the
copy is plainly *the same mark, moved* rather than something placed
elsewhere. Acrobat uses roughly this; Illustrator's default is 10 pt exactly.

### `enum Clipped`

One variant today. It is an `enum` rather than a bare `MarkupSpec` because
the module header's table has three more rows in it, and the day page
content becomes pasteable this type is where that arrives — a `Vec<u8>` of
content-stream operators, or an image handle, sitting beside this. A bare
spec would make that a rewrite of every caller.

### `enum Refusal`

Each is a **sentence on the status row**, never a silence — the standing
answer in this shell since `DEFECTS.md` D4a, and the same posture
`canvas::resizing`'s six refusals take. A `Ctrl+C` that does nothing and
says nothing is indistinguishable from a broken keyboard.

### `fn copy`

# ONE ENGINE CALL, TWO ADDRESS SPACES

`EditSession::copy_selection` takes an object-index list **and** an
annotation-index list, and its own doc comment says why they cannot be one
list: *"an annotation is not content, so it has no paint-order index."*
This function is the only place in the shell that fills both, and it fills
them in **one call** rather than two, so that:

* a mixed selection is one clip, one paste and one gesture — the day the
  selection model can hold one (see the header);
* neither list can be built from the other's numbering, which is the mistake
  the two-argument signature exists to make impossible.

# AND THEN IT ASKS THE ENGINE WHAT IT DID


⇒ **The fork is read off the payload, never off a subtype list here.** A
list would be a fourth copy of a taxonomy `pdfcer-core` owns, and would be
wrong and silent the first time the engine modelled a ninth subtype — which
is the exact defect shape this whole route was built to remove.

# Errors

Every member of [`Refusal`] except [`Refusal::NothingCopied`], which only a
paste can raise.

### `fn cut`

# Why this is copy-then-delete and not a verb of its own

Because a cut *is* those two acts, and expressing it as two calls to
functions that are each independently tested is how it stays correct. The
one thing that must not be two acts is the **undo**: a cut the operator
takes back with one `Ctrl+Z` must return the annotation, not leave them
pressing it twice.

That is already true and is not this module's doing — `Action::DeleteAnnot`
goes through `vector_edit`, which lands one `EditSession` command, and the
copy half changes no document at all. So the cut is one undo entry because
only one half of it is an edit.

# Errors

As [`copy`].

### `fn paste`

# Errors

[`Refusal::NothingCopied`] when the clipboard is empty.
