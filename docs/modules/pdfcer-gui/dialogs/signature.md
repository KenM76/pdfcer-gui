# `dialogs::signature` — the question Save has never asked about a signed
document

## The gap


```text
session.signature_impact_of_save(mode: SaveMode) -> SignatureImpact
session.changes_structure() -> bool
```

So `file.save`, `file.save_copy` and the *Save a copy…* button inside the
unsaved-edits window all wrote a revision over, or beside, a digitally
signed document **and said nothing at all** — not before, not after, not on
any surface. The compacted-save path was the one exception, because
[`crate::dialogs::compact`] measures a census and shows a sentence before
its picker opens; see §5 below for why that path needed nothing from this
module.

An operator's signed drawing is a legal artifact. A structural edit and a
save is a normal afternoon. The two together produced a file whose
signature pdfcer believed to be invalidated, and pdfcer kept that belief to
itself.

## 1. Why the engine says this can only be asked at save time

[`EditSession::signature_impact_of_save`]'s own documentation:

> A front end asks this **immediately before Save**, not at edit time: per
> §11.1 the dirty set is a diff computed at save time, so "does this save
> change structure?" is not knowable when the edit is made.

That single sentence settles the architecture of this module and rules out
the design a reader would otherwise expect. There is **no** flag on
`OpenDoc` recording *"an invalidating edit has happened"*, no marker painted
when a page is deleted, and no state accumulated across the session — all of
which would be wrong for the same reason: an edit that has since been undone
is not a change, the dirty set is a **diff against the base document**, and
only the moment before the write knows what that diff is.

The consequence worth naming, because it looks like an omission: **an
operator who deletes a page from a signed document is told nothing at the
moment they delete it.** They are told when they save. That is not this
shell declining to be helpful; it is the only moment at which the answer
exists.

## 2. The three impacts and the three surfaces

[`disclosure_for`] is the whole decision, as a pure function, and this is
the table it encodes:

| `SignatureImpact` | Surface | Why |
|---|---|---|
| `None` | **nothing at all** | the engine's own instruction: *"Nothing to say, and a front end should add no friction at all."* Most documents this operator opens are unsigned drawings, and a save that paused to tell them so would be a nag on the commonest path in the application |
| `ByteRangePreserved` | a status-bar **note, after** the save | the fact is real but it is not a decision — there is nothing to consent to, because the save does not disturb what any signature covers. See [`crate::text::signature::preserved_note`] for why this shell pairs the fact with its uncertainty rather than taking the permitted option of saying nothing |
| `Invalidated` | a **window, before** the save, plus a note after | there is an irreversible-looking decision to make and the operator is the only one who can make it |

### Why the middle row is not a window

Because there is no question. A confirmation dialog asks the operator to
choose, and here both choices lead to the same document: the save changes
nothing about the bytes any signature covers whether they proceed or not.
A window would be friction charged for information, which is how
confirmations come to be dismissed unread — and the one that matters, the
row below it, is the one that would then be dismissed.

### Why the last row is a window and not a louder note

Because it is the **conventional interaction**, which is a standing
instruction on this project rather than a preference: *use the conventional
interaction, never invent one*. Every document application that can
invalidate a signature asks first. A note after the fact would tell the
operator about a choice at the moment it stopped being available.

## 3. Why the window's copy branches on `documentation_basis`

`SignatureImpact::Invalidated` is one variant reached on two very different
footings, and the engine exposes
`SignatureImpact::documentation_basis(&census)` — in its own words —
*"because the two deserve different operator-facing wording and a front end
cannot tell them apart from the variant alone"*:

* **`SpecSourced`** — a certification signature is present, and Table 254's
  permitted-change lists are closed (*"other changes shall invalidate the
  signature"*). pdfcer can state the outcome as fact.
* **`ConservativeReport`** — only approval signatures are present, and ISO
  32000-1 defines stage 1 and only stage 1 for them. pdfcer reports
  `Invalidated` anyway, as *"a product decision under rule 4
  (fuzzy-never-sneaky), not a spec citation"*. The copy must therefore
  report the verdict **and** whose it is.

[`crate::text::signature`]'s header carries the wording rules that follow
from this, and its tests guard them. What lives here is the plumbing: the
basis is computed once, at the moment the question is raised, and travels
with the dialog — because the census it was computed from describes the
document as it stood when the operator was asked, which is what a
confirmation's text is for.

## 4. Why `documentation_basis` is NOT consulted on a full rewrite

A trap, recorded because the next reader will reach for it. The helper takes
only the impact and the census — **it cannot see the `SaveMode`** — so for a
document carrying nothing but approval signatures it answers
`ConservativeReport` even when the invalidation comes from a full rewrite,
where stage 1 genuinely fails outright under §12.8.1 and the answer is
`SpecSourced` by any reading. That is not a defect in the helper; it is a
consequence of its signature, and it is exactly why this module never routes
the compacted path through it (§5).

Everything this module *does* classify is an **incremental** save, where the
helper is precisely right: an incremental save reaches `Invalidated` only
through a structural change, and there the presence of a certification
signature really is what separates a spec citation from a cautious report.

## 5. Why the compacted path is untouched

`file.save_compacted` is a full rewrite, and §12.8.1 makes that destroy
every signature outright. [`crate::dialogs::compact`] already:

* takes a `signature_census()` when it opens,
* draws [`crate::text::compact::signature_line`] full-size and conditionally
  when the count is non-zero,
* says the loss **cannot be repaired**, which is stronger than anything this
  module says and is correct there and only there,
* and does all of it **before** its file picker opens.

That is the same disclosure this module makes, one command over, already
shipped and already correct on the spec's own terms. Adding a second window
in front of it would put two modals on one gesture — and the second would be
the weaker of the two, which is the wrong one to leave standing.

## 6. The shape is `dialogs::unsaved`'s, deliberately and exactly

[`crate::dialogs::unsaved`] is this shell's existing *ask before
proceeding* machinery and this module copies it rather than paraphrasing
it: a parked intent, a one-shot answer drained by the application, a guard
returning *"did I interrupt you"*, and the destructive act performed by
`crate::app::lifecycle` rather than by the window.

Its header also carries the argument this module inherits wholesale — that
a second predicate beside `save_pending` was correct rather than a
redefinition of it, because *"is a save in flight"* and *"are there unsaved
edits"* are different questions with different answers, and conflating them
would have broken a live consumer. The same holds a third time here:
**"will this save invalidate a signature"** is a third question, it is
answered by the engine rather than by this shell, and it composes with the
other two rather than replacing either.

⇒ The guard therefore returns `true` for *"stop, the window is up"*, for
[`crate::dialogs::DialogsState::ask_unsaved`]'s stated reason: read as
*"may I proceed"*, a guard that somebody inverts or forgets fails **open**
and the destructive thing happens. Read this way it fails **closed** — a
missing `if` asks a question whose answer performs the save anyway, so the
operator sees one redundant window instead of an unannounced write.

## 7. The one route that is told afterwards rather than asked first

`crate::app::lifecycle::resume_after_unsaved` writes a copy when the
operator presses *Save a copy…* inside the unsaved-edits window. That call
does **not** raise this window, and the reasoning is stated here rather than
left to be discovered:

1. **The operator has just answered a modal**, on this same gesture, about
   this same document. `DialogsState::ask_unsaved` already codifies the
   rule for that situation one question earlier — a second request while one
   is on screen is *swallowed* rather than stacked, because *"the operator is
   looking at a question and has not answered it"*. Stacking a second window
   on the answer to the first is the same failure one step along, and its
   result is a confirmation dismissed unread.
2. **That button writes a copy, and only a copy.** This build has no *Save*
   inside that window and the whole of `dialogs::unsaved`'s §is the
   argument for why. So the operator's signed original is untouched by that
   write, no matter what the answer here would have been — which is the fact
   that makes deferring the disclosure safe on this route and would not make
   it safe for save-in-place.

What that route gets instead is [`crate::text::signature::invalidated_note`],
recorded by [`crate::app::save`] on every successful write, so the operator
is told — after, rather than before. That is a real difference and it is
**named as a difference** rather than smoothed over. If `file.save` ever
joins that window's buttons, this paragraph stops being true and the guard
has to move; the note stays either way.

## Item notes

### `fn spec_sourced`

One predicate rather than three `match`es on `basis`, so the headline,
the explanation and the button cannot come to disagree about which
footing they are on — which is the specific way a two-wording surface
goes wrong, and it goes wrong silently because each string is correct
in isolation.

`ImpactBasis` is `#[non_exhaustive]`, and the wildcard answers `false`:
the cautious wording is correct for a footing this build cannot read,
because it asserts less.

### `fn body`

The order is fixed and each position is argued:

1. **the headline** — what is happening, in the sentence read first;
2. **the footing** — why pdfcer says so, and whose claim it is;
3. **the target** — which file this is about to touch;
4. **the buttons**, non-destructive-to-destructive left to right, which
   is `dialogs::unsaved`'s ordering rule and every application the
   operator uses;
5. **the footnote** — that pdfcer verified nothing — below the buttons,
   because it answers a question an operator only has *after* noticing
   the window is making a claim.

### `fn an_unsigned_document_is_never_interrupted`

The engine's instruction for `SignatureImpact::None`, asserted as the
property it is. Every other row of §2's table is a disclosure this
module owes; this row is the one it owes *nothing*, and it is the row
covering most documents this operator opens — so a regression here
would put a window in front of the commonest save in the application.

The basis is varied across every variant to make the point that the
answer does not depend on it: `documentation_basis` answers
`NotApplicable` for `None`, but a build that passed the wrong basis
must still not produce a window.

### `fn a_preserved_byte_range_is_a_note_and_not_a_window`

Both halves are load-bearing and they fail in opposite directions.
`Silent` would be the engine's permitted option and this shell's
choice against — see [`crate::text::signature::preserved_note`] for the
argument. `WarnBeforeSaving` would be worse: a window asking the
operator to consent to something that changes nothing they could
decline, which is exactly the friction that teaches a person to dismiss
the window that matters.

### `fn an_invalidating_save_asks_first_and_carries_its_footing`

The conventional interaction — every document application that can
invalidate a signature warns before saving, and the operator's standing
rule is to use the conventional interaction rather than invent one.

The second assertion is the one the engine commissioned: the basis must
travel *through* the decision into the surface, because
`documentation_basis` exists precisely so the two footings can be
worded differently, and a decision that discarded it would leave the
window unable to tell them apart no matter how carefully the catalog
was written.

### `fn the_window_words_the_two_footings_apart`

[`SignatureDialog::spec_sourced`] is one predicate for exactly this
reason: three independent `match`es on the basis would each be correct
and could still disagree — a certified headline over a cautious
explanation over an *anyway* button is a window that reads as though
pdfcer is unsure what it just asserted.

Asserted through the public strings rather than through the private
flag, because the flag is not what an operator reads.

### `fn a_footing_this_build_cannot_read_asserts_less`

`ImpactBasis` is `#[non_exhaustive]`, so a future variant compiles into
[`SignatureDialog::spec_sourced`]'s wildcard. It must land on the
wording that asserts **less**: pdfcer stating Table 254 as fact about a
footing it cannot identify would be the one failure this whole module
exists to prevent, arriving through a language feature rather than
through a sentence.

Asserted with `NotApplicable`, which is a real variant that cannot
reach here through `ask_for` — the only value available today that
stands in for "something this code did not plan for".

### `fn the_two_save_routes_describe_different_risks`

The distinction an operator is actually deciding on: *is the file I
already have the one being written over?* A build that used one
sentence for both would tell somebody saving a copy that their original
was at risk, or — far worse — tell somebody saving in place that it was
not.

### `fn the_confirmation_fires_once`

`dialogs::unsaved`'s one-shot property, and the failure it prevents is
the same one: an answer read every frame would re-enter the save on
each of the next sixty, which for save-in-place means sixty rewrites of
the operator's file and for save-a-copy means a file picker that will
not go away.

### `fn an_answer_is_visible_until_it_is_taken_and_not_after`

[`crate::dialogs::retire`] is the fix and it reads [`Self::answered`],
so the two edges asserted here are the ones it depends on:

* **`true` before the drain** — or the window is dropped and the save is
  lost, which is the defect above;
* **`false` after it** — or the window is kept forever, redrawing an
  answered question every frame.

It is asserted against `take_confirmation` rather than alone, because
the property that matters is that the *pair* agrees about what "parked"
means.

### `fn a_cancelled_window_is_holding_nothing`

The other half of [`crate::dialogs::retire`]'s input: `answered()` must
distinguish *closed because answered* from *closed because dismissed*,
or the ✕ would keep a window alive that has nothing to say.

### `fn cancelling_does_not_save`

The ✕ and the Cancel button must be separable from an answer, or the
control an operator presses reflexively to dismiss a surprise becomes
the one that performs the write.

### `fn the_signed_fixture_moves_from_a_note_to_a_window_when_a_page_goes`

The one test here that goes through the **engine** rather than over the
two enums, and it is worth its cost for three reasons that no amount of
pure-function testing reaches:

1. **It proves the fixture.** `tools/gen-signed-fixture.py` asserts in
   prose that it produces one approval signature over two pages. A
   generator's prose is not evidence; `signature_census()` is. If a
   future edit to that script produced a `/SigFlags` declaration with no
   signature dictionary — which the engine deliberately does not count —
   every other test in this module would still pass and `ui-verify`'s
   `signature_save` would SKIP with a reason blaming the shell.
2. **It proves the arm the copy was written for.**
   `documentation_basis` answering `ConservativeReport` is what makes
   the cautious wording the one an operator sees, and it is computed
   from `census.certifications`, which is read from `/Reference` and
   never from `/Perms`. A fixture that accidentally acquired a `/DocMDP`
   would silently switch the whole surface to the assertive wording.
3. **It proves the transition.** The same document, one page-delete
   apart, must move from a note to a window. That is
   `EditSession::changes_structure` doing its job, and it is the fact
   the engine says can only be known at save time — so it is the one
   claim in this module that cannot be checked any earlier.

### `fn nothing_is_asked_about_an_empty_shell`

The `Status::Empty` guard, asserted for `ask_for`'s stated contract:
`None` means *proceed unchanged*, and the save arms that reach here
trace their own no-document decline one line later.
