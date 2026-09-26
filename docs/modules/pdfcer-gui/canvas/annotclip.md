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

## [`duplicate`] lives here, and the carrier question is why

`edit.duplicate` (`Ctrl+D`) puts a second copy of the selected comment on
the page **without touching the clipboard**, which is the whole point of it:
doing the same with `Ctrl+C`/`Ctrl+V` destroys whatever the operator was
carrying, once per mark on a row of revision marks.

It is in *this* module rather than beside the dispatcher because a duplicate
faces the identical carrier question a copy does, and the obvious
implementation — straight onto `paste_objects` with a translate matrix —
gets it wrong in the same invisible way: it hands back an **anonymous,
undated, opaque** copy of a signed revision cloud, which looks correct on
the page. So it runs the same `copy_selection` and asks the same
[`Plan::of`]. **No subtype list, in either verb.**

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
duplicate must travel as that clip. The assertion is on the *action
kind*, so any second route added beside the clip — one that re-authors
from a spec, say — has to pass it too.

### `fn duplicating_nothing_refuses_and_raises_nothing`

The `actions` emptiness is half the assertion. A verb that refuses and
still pushes is worse than one that does neither, because the refusal
sentence then contradicts the undo entry beside it.

### `fn the_duplicate_cannot_reach_the_clipboard`

Asserted **structurally** rather than by reading `egui` memory: this
function takes no `&egui::Context`, so it *cannot* read or write the
clipboard — `canvas::clipboard::store` and `read` both require one. A
test that opened a context and compared before/after would pass equally
well against a signature that could reach it, and would stop being
evidence the day somebody threaded a context through "for the trace".

⇒ So what is pinned here is the signature. If this stops compiling
because `duplicate` grew a `ctx` parameter, that is the review this note
is asking for, not a test to update.

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

### `fn duplicate`

# Why this is a verb and not "copy then paste"

Because the two are different acts and the difference is the clipboard.

Reaching a second revision cloud through `Ctrl+C` then `Ctrl+V` works, and
**destroys whatever the operator had copied**. An operator laying out a row
of identical revision marks is very often carrying something else on the
clipboard (a title-block string, a part number, a cell from a spreadsheet),
and every duplicate would cost them that. Every application in this class
separates the two for exactly that reason, and Acrobat has had `Ctrl+D` on a
comment for as long as it has had comments.

`mockups/app.html`'s approved canvas context menu already draws
*"Duplicate — Ctrl+D"*; this is the verb behind that line.

# Why it is NOT an extension of `edit.paste_duplicate`, which was checked
first

`app::dispatch::clipboard`'s header names `edit.paste_duplicate` as *"the
second sense of a form-field paste"* — `Ctrl+V` plants a copied field as a
**new** field, `Ctrl+Shift+V` plants it as **another widget of the same
field**. Its own header records what it does over a markup: *"falls through
to the ordinary paste … a markup has no second sense to duplicate into"*.

So it does already route by selection kind, and the route it takes for a
markup is *the plain paste*. Making it duplicate the **selection** instead
would be a paste verb that acts when the clipboard is empty and ignores the
clipboard when it is not — two unrelated behaviours behind one id, reachable
by a chord named for the one it would stop doing. This is a sibling command
instead, which is what a shell that registers, binds, places and mode-gates
per id can express and a modifier read inside a handler cannot (R8).

# The route is decided by the ENGINE, exactly as the copy's is

This runs the same `copy_selection` the copy runs and asks [`Plan::of`]
which carrier each annotation landed on, so a refusal is disclosed by name
exactly as a copy's is. It does **not** re-implement the classification,
and it does not hard-code a subtype list.

⇒ That is the whole reason this function lives in this module rather than
beside the dispatcher. The module header's rule — which carrier an
annotation lands on is the engine's answer to read, never this shell's to
predict — applies to a duplicate identically. A duplicate written the
obvious way, straight onto `paste_objects` with a translate matrix, is one
engine change away from an **anonymous, undated, opaque** copy of a signed
revision cloud, silently, and it would look right on the page.

# The clip is assembled before the refusal check, and that is the point

`copy_selection` takes `&self` and commits nothing, so the cost is one walk
and one allocation. Asking the engine first is deliberate: the alternative
is this shell deciding which carrier an annotation *would* land on, which is
the hard-coded subtype list the module header spends a section refusing.

# The offset

[`crate::canvas::clipboard::PASTE_OFFSET_PT`] down and to the right — the
**same** constant and the same signs a same-page paste uses, because a
duplicate is a same-page paste in everything but where the payload came
from. Down the page is **negative** in PDF user space; getting it
backwards produces a copy that goes up-and-right, which looks deliberate
and is the kind of thing nobody reports as a defect.

There is deliberately **no cursor rule** here, where a paste has one
(`OPERATOR_REQUESTS.md` O73). A paste is invoked with the pointer over the
place the operator wants the thing; a duplicate is invoked from a chord, a
menu row or a ribbon button while they are looking at the original, and
dropping the copy under a pointer that is resting on a ribbon icon would
put it wherever the mouse happened to be. The offset is the whole rule, and
it is what makes `Ctrl+D Ctrl+D Ctrl+D` walk a diagonal row of marks —
which is the gesture the feature exists for.

# One undo entry

Whichever route it takes, exactly one action is raised, and each of the two
goes through `app::actions::apply::vector_edit` as a single `EditSession`
command. `Ctrl+Z` after a duplicate takes back the duplicate.

# Errors

* [`Refusal::NothingSelected`] — no annotation is selected. Page content is
  *also* nothing to this verb today: the selection model makes the two
  mutually exclusive, and a content duplicate is a different feature with a
  different name for what "the same place" means.
* [`Refusal::Unreadable`] — the selection names an annotation that is no
  longer on its page, or whose dictionary will not read. Reachable after an
  undo.
* [`Refusal::EngineRefused`] — `copy_selection` would not assemble a clip.
* [`Refusal::CannotCarry`] — `/Widget`, `/Popup` or `/Redact`, refused by
  the engine **by name** and by this verb for the same three reasons the
  copy refuses them. A redaction in particular: duplicating one arms a
  second destructive operation nobody reviewed.
