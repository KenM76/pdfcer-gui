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

## ★★ 1. Why the engine says this can only be asked at save time

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

## ★★★ 2. The three impacts and the three surfaces

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

## ★★ 3. Why the window's copy branches on `documentation_basis`

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

## ★ 4. Why `documentation_basis` is NOT consulted on a full rewrite

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

## ★★ 6. The shape is `dialogs::unsaved`'s, deliberately and exactly

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

## ★★ 7. The one route that is told afterwards rather than asked first

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
   inside that window and the whole of `dialogs::unsaved`'s §★★ is the
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
