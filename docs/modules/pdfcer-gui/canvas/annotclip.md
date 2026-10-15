# `canvas::annotclip` — **the annotation half of the canvas clipboard**

The seam against [`crate::canvas::clipboard`] is **annotation versus
content**, not copy versus paste:

* `canvas::clipboard` owns the **clipboard as a thing** — what it can hold,
  who owns `Ctrl+C`, where a paste lands, which refusals exist, and the
  routing between the four operand families.
* this module owns **what an annotation costs to carry**: which of the
  engine's carriers it lands on and what each carrier drops.

A reader asking *"why did my sticky note's author survive and my cloud's
not?"* finds the whole answer here, in one file, rather than interleaved
with the paste offset and the OS-clipboard marker.

## THE RULE THIS MODULE EXISTS TO HOLD

**Which carrier an annotation lands on is the engine's answer to read, never
this shell's to predict.**

`EditSession::copy_selection` classifies every annotation it is asked for
through `clip_annotation`, whose last act is to try
`annot_author::spec_from_dict` and fall back to `clip_raw_annotation` when
that refuses. So the model carrier — `ClipAnnotation::Markup` — claims
`/Square`, `/Circle`, `/Line`, `/Ink`, `/Polygon`, `/PolyLine`, `/Cloud` and
text markup, which is **every kind this shell authors**, and the raw carrier
gets exactly the kinds this shell cannot author: sticky notes, stamps, text
boxes, links, file attachments.

That split is not stable and has already inverted once in the direction that
matters to an operator. A `MarkupSpec` describes a *shape*, so while the
model carrier held nothing else, a copied revision cloud arrived anonymous,
undated and opaque — `/CA`, `/T`, `/M` and `/Contents` dropped — while the
kinds the shell could *not* author round-tripped byte for byte. The engine's
`ClipAnnotation::Markup` now carries a `MarkupCarry` beside the spec (border
dash, `/CA`, `/Contents`, `/T`) and its paste applies it through
`add_markup_with`, so both carriers are faithful today.

⇒ **The hand-written repair is the trap.** A list — *"`/Square`, `/Circle`,
`/Line` … take one path, everything else the other"* — is wrong the moment
`spec_from_dict` learns a ninth subtype or gives up an eighth, and nothing
goes red. Instead the copy runs `copy_selection` first, unconditionally, and
asks the returned [`ObjectClip`](pdfcer_core::vector::ObjectClip) which
carrier each annotation landed on — see [`Plan::of`], whose wildcard arm is
what makes a carrier this build has never heard of the safe case.

The classification therefore tracks the engine automatically, in both
directions, and the only thing this file hard-codes about subtypes is
nothing at all.

## What each route can carry

| route | reached when | carries | drops |
|---|---|---|---|
| the clip, `Markup` carrier | `spec_from_dict` reads the dictionary | the modelled geometry, colours and widths, plus `MarkupCarry`'s border dash, `/CA`, `/Contents` and `/T` | `/M` — and a paste authors a fresh mark, so a new date is the right answer |
| the clip, `Raw` carrier | `spec_from_dict` refuses the dictionary | the whole dictionary, its baked `/AP` and the object closure it reaches | `EditSession::CLIP_STRIPPED_ANNOT_KEYS`: `/P`, `/Parent`, `/StructParent`, `/NM`, `/Popup`, `/IRT` — all six name something in the *source* document |
| the clip, `Dimension` carrier | it is a **ce dimension** | the group by name, its scale, format, standard, the per-object style and the text override | nothing this shell can author |
| refused | `/Widget`, `/Popup`, `/Redact` | — | the whole annotation, **by name** |

## Two address spaces, and this module resolves one of them

`copy_selection` takes **two index lists** and the engine's own doc comment
says why they cannot be merged: *"an annotation is not content, so it has
no paint-order index."* The shell holds annotations by
[`ObjId`](pdfcer_core::object::ObjId) and content by paint-order index, so
[`selected`] is the one place that converts the first into the position
`page_annotations` would return it at. It is deliberately the **only** such
conversion: an index taken from anywhere else is an index whose numbering
nobody can name.

It refuses rather than guesses when the id is not on the page. `R168` —
`copy_annotations` refuses the whole call on one bad index rather than the
valid remainder — and matching that here means a stale selection produces a
sentence instead of a clip that is quietly missing a member.

## Item notes

### `const FIXTURE`

Not a `state::fixtures` constant, deliberately: those name fixtures
several modules share, and this one has exactly one subject and one
consumer. Its generator, `tools/gen-annots-with-everything-fixture.py`,
carries the argument for every key in it.

### `fn the_engine_models_a_square_and_carries_a_sticky_note_whole`

This is the test that pins the module header's rule, and it is written
against `pdfcer-core`'s behaviour rather than against a sentence about
it, because a sentence about another crate is a claim with a shelf life
measured in hours (`RESUME.md`).

Three annotations, three questions:

| `/Annots` index | subtype | expected carrier |
|---|---|---|
| 0 | `/Square` | `Markup` — modelled, and carried **whole** because the carrier holds `MarkupCarry` |
| 1 | `/Text` | not modelled, so the clip carries it **whole** |
| 2 | `/FreeText` | not modelled, so the clip carries it **whole** |

Every index must read `whole`. One reading `thin` means a lossy
carrier is back and the disclosure on [`Plan::thin`] has a subject
again — the moment this project most needs to notice and is worst at
noticing.

### `fn a_sticky_note_survives_the_clipboard_key_by_key`

The vacuous shape to avoid: a fixture annotation carrying only the keys
a `MarkupSpec` can already express makes *"the copy is lossless"* pass
under a plant that still re-authors from the spec. So the subject is the
`/Text` sticky note, which carries `/CA 0.4`, `/T`, `/M`, `/Contents`,
`/Name`, `/C` **and a baked `/AP`** — none of which any authoring verb
in `pdfcer-core` would reproduce — and the assertion iterates the source
dictionary rather than an expected list.

# Why the six exceptions are the engine's list and not ours

`EditSession::CLIP_STRIPPED_ANNOT_KEYS` drops `/P`, `/Parent`,
`/StructParent`, `/NM`, `/Popup` and `/IRT`, each because it names
something that exists only in the source document. This fixture
deliberately carries **none** of the six except `/P`, so the exception
list needed here is one key long — which is the difference between an
assertion and a hand-maintained allow-list, and is why the generator
refuses to put a `/Popup` on it.

# What it does NOT assert

Byte equality of the `/AP` stream's contents. The engine renumbers the
appearance object on import, so the *reference* legitimately differs;
what is asserted is that `/AP` is present and resolves to a stream, which
is the property whose absence renders a sticky note as nothing at all.

### `fn duplicating_an_unmodelled_annotation_takes_the_whole_carrier`

The `/Text` sticky note at index 1 is not modelled by `spec_from_dict`,
so the engine carries its whole dictionary and baked `/AP`, and the
duplicate (`canvas::duplicate`) must travel as that clip. The assertion is on the *action
kind*, so any second route added beside the clip — one that re-authors
from a spec, say — has to pass it too.

### `fn a_stale_selection_refuses`

A selection naming an annotation no longer on its page refuses with
`Unreadable` rather than resolving to whichever annotation now sits at
that `/Annots` position. Copy, Cut and Duplicate all lift through
`selected`, so this one guard covers the three.

### `struct Selected`

The `index` is the position in `pdfcer_core::annot::page_annotations`'
output for the page — which is `/Annots` in document order, and is the
numbering `EditSession::copy_annotations` documents itself as addressing.
The `id` travels beside it because the **cut** half needs it: a delete is
raised by `ObjId` through the funnel, and re-deriving one from an index
after the clip was taken would be a second walk that could disagree with
the first.

### `fn selected`

Empty when nothing is selected or the selection is page content. At most
one entry today, and the plural is not aspirational padding — see the note
below, which is the finding a reader of this signature most needs.

# Why this returns a `Vec` when the selection can hold exactly one

`canvas::selection::SelectionState` makes content and annotations
**mutually exclusive by construction**: `select_annot` clears the content
entries, the content paths clear `annot`, and the field itself is an
`Option<AnnotSelection>` rather than a list. Its own doc says so — *"One
canvas, one selection."*

⇒ **So a marquee that catches a line AND a revision cloud is not a state
this shell can be in**, and the mixed copy the engine's `copy_selection`
exists for cannot be exercised from the canvas today. That is a fact about
the *selection model*, which is a different subject and a different file,
and it is recorded here rather than in a commit message because this is
where a reader will ask.

What this module does about it is the one thing it can: the copy is written
as **one call with both lists**, so the day the selection model gains a
mixed set, the clipboard needs no change and cannot silently take half.
Writing it as two calls — one for content, one for annotations — would have
produced two clips, two pastes and two undo entries, and would have had to
be unpicked later.

# Errors

[`Refusal::Unreadable`] when the selected id is not among the page's
annotations — a selection outliving the annotation it names, which is
reachable after an undo or an external reload.

### `fn of`

The wildcard arm is required — `ClipAnnotation` is
`#[non_exhaustive]` — and it counts toward [`Self::whole`] rather than
toward [`Self::thin`] or [`Self::refused`], which is the safe direction
on all three counts: a carrier this build has not heard of is one the
engine added *because* it carries something the old ones could not, so
treating it as whole neither refuses a paste that would work nor
disclosing a loss that is not happening. The alternative — counting it
as `thin` — would put a false warning on the status row for every
annotation of a kind a newer engine handles better.

**When an engine bump makes an arm here stop compiling, the compile
error is a notification, not a chore.** The tempting repair — widening
the pattern to swallow the new field and keep the old count — compiles,
and leaves this shell warning the operator about a loss that is no
longer happening. Re-check what the carrier now carries, then decide
which count it belongs in.

### `fn nothing_to_carry`

An empty clipboard that reports success is the worst outcome available
here: the operator presses `Ctrl+C`, sees nothing said, presses
`Ctrl+V`, and gets *"nothing has been copied yet"* — a sentence about a
keystroke they made two seconds ago and which appeared to work.

### `fn rect_centre_of`

`None` for a dictionary with no readable `/Rect`, which falls the paste
back to the offset rule rather than guessing. That direction is deliberate:
an unrecognised annotation pasting at the old offset is a mild surprise, and
one pasting at `(0, 0)` — the bottom-left corner of the sheet — reads as
data loss.

Read from the raw dictionary rather than from a `MarkupSpec`: a spec is a
*translation* of the annotation, and every kind translates its geometry
differently — an ink stroke into a point list, a line into two ends, a
square into corners. `/Rect` is the one place every annotation states its
extent in the same terms (§12.5.2), so reading it needs no per-kind match
and therefore cannot silently omit a kind.

Not used by the clip route, which anchors on the clip's own
`ObjectClip::bbox` — unioned by the engine over both content items and
annotation rects. One number from the payload rather than a second reading
of the document is what makes a clip pasted after the source document was
closed still land where the operator pointed.

