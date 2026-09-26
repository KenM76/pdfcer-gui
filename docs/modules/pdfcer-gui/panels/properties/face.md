# `panels::properties::face` — the face chooser, once, for both surfaces


That divergence is the reason this module exists at all, and it is worth
stating as a rule rather than as a tidy-up: **a control drawn twice is a
control that will be built twice**, and the second build is always the one
that misses the disclosure. The two callers now differ in exactly two
things — the region prefix they publish under, and what they do with the
selector that comes back — and in nothing else.

## The list has two kinds of row now, and they are two different acts

`pdfcer-core` v0.15.0 (`Pass 162.0`):

> **FONTS** — text can be restyled to a face the document **DOES NOT
> CONTAIN**, for the fourteen faces every PDF reader is required to have.
> pdfcer authors the font resource on demand, with widths, embedding
> nothing. A face outside those fourteen still refuses by name — that needs
> a real font program.

So a row in this list is now one of:

| kind | what a click does to the file | what it costs |
|---|---|---|
| [`FaceOrigin::OnThisPage`] | rewrites one `Tf` operand | nothing — the resource is already there |
| [`FaceOrigin::PdfcerWouldAdd`] | rewrites the `Tf` operand **and writes a new `/Font` object** | a dictionary with widths and no glyph outlines, plus a face drawn from the reader's own copy |

Those are different enough that presenting them as one undifferentiated list
would be hiding a write behind a menu. [`choices`] tags every row and
[`popup_body`] draws them under two headings with the disclosure between.

## Who writes the resource — asked, and answered by reading the engine

`FormatPlan::created_font` is documented as *"a `/Font` resource the caller
must CREATE for `new_content` to be valid"*, and its own note says the caller
owns the write *"because only the caller can allocate an object number:
planning runs against an immutable `&Document`."* That is a real obligation
and it would not be this shell's to invent.

**It does not fall to this shell**, because `EditSession::format_text` is
itself that caller and it already performs the write — on both of its paths:

* the page-`/Contents` path (`pdfcer-core/src/edit.rs` ~7887) takes
  `plan.created_font`, calls `self.font_resource_writes(page.id, true, …)`
  and extends **the same command** with the writes, *"so one undo removes
  both the restyle and the resource it needed"*;
* the form-XObject path (~8015) binds the resource into the form's own
  `/Resources` before the stream is rebuilt, for the stated reason that two
  writes for one object id in one command would let the later silently win.

Both also push a disclosure when the target `/Resources` turns out to be
shared with other pages. `crate::app::actions::textstyle` already surfaces
the engine's disclosures verbatim, so that one arrives on the status bar with
no code here.

⇒ **The shell change is the chooser and nothing else.** Nothing in this
module allocates an object, writes a dictionary, or knows the shape of one.


Every [`FaceOrigin::OnThisPage`] row has been through `set_font`'s own
acceptance test for **this run's characters** — that is what
`preview_font_resources` is, and it is why a refused page font is absent from
the list rather than greyed. **The other half now matches it.**

[`choices`] reads `FontPreflight::standard_14`: one `Std14Entry` per face,
each carrying the engine's own spelling, a `presence` that says `OnPage` or
`WouldBeAdded` as a **fact**, and an `acceptance` run through `set_font`'s
gate with the embedded-subset floor included. Three local re-derivations
were deleted with it — the `Std14::ALL` walk, the `carried` filter built by
shortening `preflight.entries`, and the untested offer.

⚠ **Consequence, stated because it is a behaviour change:** a standard-14
face that cannot hold this run's characters is now **absent** rather than
offered-and-then-refused. That is the first half's behaviour, applied to the
second.

### What this paragraph said until the engine answered it, kept because the
### reasoning is why the engine answered it

> *"It cannot be done honestly from here. The engine offers no query that
> coverage-tests a face the page does **not** carry, so the shell's only
> route would be to re-derive the encoding rule — which face uses
> `WinAnsiEncoding`, which two use a built-in `FontSpecific` one, and which
> characters that leaves unmapped. `FontPreflight`'s own invariant forbids
> exactly that (`R221`), and a second copy of the rule in `pdfcer-gui` would
> drift from the commit path the first time the rule changed.*
>
> *⇒ So these rows are **offered, and a refusal is a sentence** … This is
> recorded as an engine ask rather than worked around: a
> `preview_font_resources` that also surveyed the fourteen would let this
> list be as exact as its first half already is."*

**That last sentence is the request, and `Pass 142.2` is the answer.**
The refusal to copy `R221`'s rule into this crate is why the fix arrived as
an engine capability rather than as drift — worth keeping, because the
tempting shortcut was one afternoon's work and would have been wrong on the
first day the encoding rule changed.


The old `carried` filter is subsumed, and more exactly. It compared
*shortened* names against the page's entries; `Std14Presence::OnPage` is the
engine answering the same question from the resource dictionary it actually
resolved. A face already on the page reaches the list through the
`accepted()` half if it works, and through neither if it does not — so
`Helvetica` is never offered as *"pdfcer can add"* on a page whose own
`Helvetica` was refused.

## Rule 4

Nothing here marks the canvas. The one inference an operator cannot see —
that an added face is drawn with the *reader's* copy rather than one carried
in the file — is discharged as a sentence at the point of choice
([`crate::text::panels::face::face_addable_disclosure`]), which is the
off-canvas report rule 4 requires and the one thing this feature could not
ship without.

## Item notes

### `const POPUP_MIN_WIDTH`

The ribbon's chooser button is **78** points wide
([`crate::app::fontband`]'s `FACE_WIDTH`, sized to fit inside the band's
custom-item budget), and an `egui` combo popup is otherwise no wider than its
button. The disclosure is a three-clause sentence; wrapped to 78 points it
would be a column of two-word lines, which is a sentence an operator does not
read.

So the popup states its own minimum and the two surfaces get the same one —
which is also what stops the panel's copy and the ribbon's copy from being
legible in one place and not the other, the divergence this module exists to
end.

### `fn preflight_for_paragraph`

The reason given at the time was honest and real: `FontPreflight` is
`#[non_exhaustive]` and cannot be built with a struct literal outside
`pdfcer-core`. ⇒ **The answer to "I cannot construct it" is to obtain a
real one, not to simulate the function under test.** `EditSession`
hands one over for the asking, and a fixture costs a millisecond.

⚠ It also means the input half — the half the old comment admitted was
uncovered — is the half that mattered, because that is where the change
landed.

### `fn the_faces_the_page_lacks_are_offered_and_come_from_the_engines_survey`

`fixtures/paragraph.pdf` carries `Helvetica` and nothing else, and its
text is plain ASCII that every text face can encode — so the expected
answer is the whole fourteen: one through `accepted()` as a page face,
thirteen as addable.

Asserted as `>= 13` addable rather than `== 13` on purpose. The
standard 14 contains `Symbol` and `ZapfDingbats`, whose acceptance for
ASCII text is the engine's ruling and not this shell's to pin — if the
engine decides a font-specific encoding cannot hold `its box.`, that is
a correct answer and must not fail this shell's test. What IS pinned is
that the twelve text faces all arrive.

### `fn the_pages_own_standard_face_is_offered_once_and_not_as_addable`

The duplicate would be the visible defect. The invisible one is worse:
a page `Helvetica` that this run's characters cannot encode into is
absent from `accepted()`, so a filter built on that list would offer
*"pdfcer can add Helvetica"* — and `plan_font` would resolve the
selector to the page's own refused resource and decline. An entry that
cannot work, described wrongly.

⇒ `Std14Presence::OnPage` is the engine answering that from the resource
dictionary it actually resolved, which is why [`choices`] no longer
compares shortened name strings to decide it.

### `fn no_offered_face_is_absent_from_the_engines_own_survey`

A rewrite that quietly reverted to walking the constant would satisfy
both tests above, because on this fixture the two answers coincide. This
one does not: it empties the survey's contribution by asserting the
offered labels are a SUBSET of what the engine reported, which a local
walk cannot guarantee.

### `fn no_preflight_offers_no_faces`

The tempting shape — *"we could not ask the page, so offer the standard
faces, they always work"* — is wrong twice. The standard-14 half is
filtered **by** the pre-flight, so without one the list would offer
`Helvetica` on a page whose own `Helvetica` will take the click; and a run
that did not pin cannot be restyled at all, so every row would refuse.

### `fn the_two_origins_are_not_equal`

A `FaceChoice` that lost its origin would render under whichever heading
it happened to sort beside, and the disclosure would then be attached to
rows it is not true of. The enum is `Copy` and cheap; this asserts it is
also actually compared somewhere, which is what a `derive(PartialEq)` on
an unused field would not be.

### `enum FaceOrigin`

Two variants and not a `bool`, because the call sites read as prose this
way and because a third origin is foreseeable: `Pass 142.0` would let pdfcer
subset and embed a face from the operating system, which is a third act with
a third set of consequences (a real font program in the file, and a face that
then renders identically everywhere). A `bool` would have to be replaced on
that day; this enum gains an arm and the compiler names every place that has
to say something new about it.

### `fn choices`

# What each half is, and why the second half is not simply "the fourteen"

**The page's own faces** come from `FontPreflight::accepted()` — every
`/Font` resource on this page that `set_font` would accept for this run's
characters, each carrying the selector that reaches *that* resource. A
refused one is deliberately absent rather than greyed: the refusals are
per-character encoding facts (*"'o' has no code in Times-Bold's encoding"*),
and a list of twelve faces with nine greyed rows each carrying a sentence
about a character is a control an operator cannot read. R9's
absent-rather-than-greyed case: for THIS run those faces are not a capability
that is temporarily unavailable, they are not applicable.

**The addable faces** are `Std14::ALL` minus every standard-14 name the page
already carries **in any form** — accepted or refused. That second word is
the load-bearing one and it is not a courtesy:

`plan_font` calls `resolve_target_resource` first and authors a face only
when that misses. So a page carrying a `Helvetica` that this run's characters
cannot be encoded into would resolve the selector `Helvetica` to **that**
resource and refuse it — while a row labelled *"pdfcer can add Helvetica"* had
promised the opposite. Filtering on `entries` rather than on `accepted()`
keeps the row out of the list entirely, which is the same answer the page
half gives for the same face.

The name comparison strips the §9.6.4 subset tag ([`super::text::shorten`])
before matching, because a page carrying `ABCDEF+Helvetica` is a page that
carries Helvetica as far as `set_font`'s own `/BaseFont` match is concerned.

# Ordering

Page faces first, in the engine's dictionary order; then the fourteen in
`Std14::ALL`'s order, which groups the families (Helvetica, Times, Courier,
then the two symbolic faces) and is a spec-frozen constant the engine
publishes for exactly this use. Nothing here re-types the fourteen names.

# `None`

An absent pre-flight means the run did not pin or the preview refused, and
the answer is an **empty list** rather than the fourteen on their own. The
standard-14 half is filtered *by* the pre-flight; without one, offering it
would mean offering `Helvetica` on a page whose own `Helvetica` would take
the click — the exact entry-that-cannot-work this function's second half is
written to avoid.

### `fn popup_body`

`current` is the run's `/BaseFont` **already shortened** — the string a row's
label is compared against to decide which row is the selected one. `prefix`
is the region namespace this surface publishes under, so the panel's copy and
the ribbon's copy are separately findable by a driven check.

Returns `Some(selector)` on the frame a row is clicked, and `None` on every
other frame — including the frames the operator spends reading the list,
which is most of them.

# Nothing is held between frames, because the document is the state

`selectable_label`, never `selectable_value`. A press here is an **edit**,
not a choice to be committed later, and a widget holding a pending value
would be a second place the current face is recorded — which is how a control
comes to disagree with the file it is about.

# The disclosure is drawn once, visibly, and only when it is owed

Not a hover, and not a hover repeated on fourteen rows. It is owed to every
operator who opens this list — including the one who reads it and chooses
nothing — and a hover is a sentence they would have to go looking for.
Fourteen copies of it would be the nag the brief for this work explicitly
rules out.

It is drawn only when at least one addable row exists, so a page already
carrying all fourteen never shows it. See
[`crate::text::panels::face::face_addable_disclosure`] for what the sentence
has to contain and why each clause is there.
