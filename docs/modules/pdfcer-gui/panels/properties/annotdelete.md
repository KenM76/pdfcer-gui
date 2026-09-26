# `panels::properties::annotdelete` — whether the selected annotation can
be deleted, and what would go with it


| query | what it answers | what this section does with it |
|---|---|---|
| `EditSession::annotation_deletion_refusal` | *would `delete_annotation` refuse right now?* | withholds every Delete control and puts a sentence in their place |
| `EditSession::annotation_deletion_preview` | *what else would go with it?* | states the collateral **before** the press |

## The defect the first query closes, and it is the day-before defect
wearing a different `/Subtype`


**The annotation half was the same defect, one file along, and it was still
open.** `annotation_deletion_refusal` is `&self`, side-effect-free, and its
own doc comment names this call site by rule number — *"safe to call every
frame from a UI (R83: ask before offering the control)"*. Nothing called it.
So on a certified or encrypted drawing:

* the **Format tab's Delete** was drawn and enabled,
* the **canvas right-click's Delete** was drawn and enabled,
* the **Delete key** raised the action,


⇒ The generalisation, and it is the audit's rather than this file's: **a
query the engine wrote for a shell is not consumed by being read.** Both of
these carry doctests spelling out the call site, and both sat unused for the
whole life of the crate. The instrument that found them was
`tools/verb-coverage.py` — asking what the engine offers — not a re-reading
of this shell.

## Where a gate refuses, the control is NOT DRAWN and a sentence takes
its place (R9)

The established shape, set by the forms fix and followed here rather than
re-invented:

- Greying is for a capability that is **temporarily** unavailable, and is
  always explained on hover. A certification signature is not temporary and
  cannot be argued out of; nor is `/Encrypt`; nor is §12.5.3 bit 8.
- A permanently-refused capability renders **nothing**, or a sentence saying
  where the thing actually lives. There is no elsewhere here, so it is the
  sentence.
- **A sentence rather than a silence.** A panel that quietly omits half its
  controls looks half-drawn, and an operator who finds Delete missing with no
  explanation has found a broken program rather than a protected document.

The withholding of the ribbon and menu controls is not done here — it cannot
be, because a manifest item is drawn by `egui-shell` and this crate may not
reach into it. It is done by the condition
`crate::app::conditions`' `selection.delete_permitted`, which `format.delete`
carries as its `visible_when` on the Format tab and on both canvas menus.
**One question, two consumers**: the condition and this sentence are derived
from the same three facts in the same order, and [`gate`] is the one function
that derives them.

## Why the SENTENCE lives in a panel and not in the status bar's decline

`crate::app::status::decline` is this shell's worded-decline surface and it
would have been the reflexive choice. It is the wrong one here, and the
module's own header says why in the general case:

> A decline must be **repeatable** … and a decline changes no document, so
> the epoch never moves.

A decline is a report that *a gesture just failed*. What this section states
is not a gesture's outcome at all — it is a **standing property of the open
document**, true from the moment it was opened until it is closed, and true
whether or not the operator has pressed anything. A sentence that arrives
only after a press, and retires when the next command runs, would deliver
that fact at the one moment R83 exists to get ahead of.

⇒ So it sits in the panel that describes what is selected, permanently, and
the press it prevents is a press the operator never makes.

## ⚠ The cost of the preview, and what was chosen

`annotation_deletion_preview` is `&self` and mutates nothing, but it is **not
free**: it locates the annotation, walks the page's whole `/Annots` array to
find `/IRT` referrers, and validates the `/Popup`. That is O(annotations) per
call. The old shell computed exactly this and gated it on **hover**, one row
at a time, because its Comments panel would otherwise have paid
O(rows × walk) every frame.

This section does better than hover, and the reason it can is that **it is
not a list**. There is one selected annotation, so the worst case is one call
per frame rather than one per row — and even that is not paid, because the
answer is memoised in [`DeletionPreview`] on `(annotation id, edit epoch)`.
Both halves of that key are load-bearing:

* the **id** changes when the operator selects something else, which is the
  only other thing that can change the answer;
* the **epoch** changes on every accepted edit, which is what makes a reply
  added or removed since the last frame visible here.

⇒ In steady state — an annotation selected, nothing being edited — the cost
is one `Option` comparison per frame and no engine call at all. A hover gate
would have been cheaper only in the frames where the answer is not wanted,
and it would have hidden the fact behind a gesture the operator has no reason
to make.

## What this section does NOT do

**It raises no action and offers no control.** It reads and it draws
sentences; `super::body`'s contract is `&OpenDoc` shared, so that is a
compile-time fact rather than a convention. The Delete controls stay where
they are — the Format tab, the canvas menus, the Delete key — and this
section only decides what is *said* about them.

**It does not run the delegated route's gate.** `annotation_deletion_preview`
reports a `/Redact` mark or a ce dimension with its
`AnnotationDeletionRoute` and **zeroed counts**, and the engine states
plainly that it does not run the destination verb's certification gate for
those. Because this section says nothing at all when every count is zero, no
false claim is made on that path: a delegated target produces silence here,
not a promise that the delete would work. The engine's own note says the
honest fix is for those verbs to grow refusal queries of their own, and
*"inventing a half-answer here would be worse than a stated gap."*

## Item notes

### `const REGION`

A published region name is a cross-repo stability contract: the harness
asserts on it by string, so renaming one turns a check into a skip rather
than a failure.

### `const REGION_REFUSED`

"Only when refused" is the whole value of it. A driven check asserting
that a certified document withholds Delete reads two things: this region
**present**, and `properties.format.delete` — the ribbon control's region —
**absent**. An absence is admissible evidence only because [`TRACE`] is
written on every frame this section runs either way, so a check can tell
*"the control was withheld"* from *"the panel never drew"*. The harness's
own rule 4 states the same obligation from the other side.

### `const TRACE`

The name carries a verb suffix — `annot-delete-…` rather than
`delete-annotation-…` — because `tools/gates/check-trace-names.py` forbids a
module's own summary line from sharing its first token with a `vector_edit`
funnel label, and `delete-annotation` is exactly such a label. A harness
asking `last("delete-annotation")` would otherwise get the funnel's line,
which carries `page`, `n`, `epoch` and `disclosures` and none of the keys
this line exists to publish. That failure has happened three times on this
project and it presents as a confident false negative.

### `fn refusal_for`

# A total match with a named catch-all, not a `_ =>` with a guess

Every variant the query's own documentation names has an arm, **in the order
the engine checks them**, so this function and
`EditSession::annotation_deletion_refusal`'s two-line body can be read side
by side. That is the shape `crate::app::actions::xobject::refusal_for` uses
for `unshare_form`, and the failure it prevents is the one that is invisible
in a diff: somebody adds an arm, mistypes a variant name, the compiler is
happy because `_ =>` catches it, and the operator meets *"pdfcer cannot delete
comments from this document"* where they should have met the sentence about
the signature.

A free function rather than a `From` impl, for the same reason that one is:
a `From` invites the mapping to be reused for another verb's errors and it is
not reusable. `DocumentEncrypted` earns *this* sentence because the subject
is an annotation; the same variant out of `unshare_form` earns a sentence
about a drawing.

### `fn line`

`&mut self` and a borrowed return, so the string is neither cloned per
frame nor re-derived — the same shape
`crate::panels::properties::text::TextStyleDraft::sync` uses for the far
more expensive run inspection next door, and for the same reason.

### `fn each_documented_refusal_earns_its_own_sentence`

The failure this pins is the one that is invisible in a diff: somebody
adds an arm to [`refusal_for`], mistypes a variant name, and the compiler
is happy because `_ =>` catches it. The operator then meets *"pdfcer
cannot delete comments or markup from this document"* where they should
have met the sentence naming the certification signature — which is the
difference between a dead end and an explanation.

The same test guards `crate::app::actions::xobject::refusal_for` for
`unshare_form`, and it is the same test because it is the same hazard.

### `fn the_refusals_are_told_apart_by_their_words`

An encrypted drawing and a certified one look identical on the canvas.
If these collapsed to one wording the enum would be decoration, and an
operator would be sent hunting for a signature that is not in their file.

### `fn the_locked_sentence_is_the_one_that_leaves_a_next_step`

Asserted through [`Refusal::line`] rather than by re-reading the branch,
because what is being pinned is the *operator's* outcome: the more
actionable of two true facts is the one that gets said. An operator told
their comment is marked unchangeable can go and look at that comment; an
operator told the document is certified can do nothing about one
annotation.

### `const SQUARE`

Named by object number rather than found by a walk, deliberately. The
fixture is authored by this repository byte for byte, so the number is a
fact about a file in this tree rather than an assumption about a
document somebody else produced — and a test that *searched* for "the
square" would pass on a fixture that had lost its pop-up, which is half
of what these assert.

### `fn square_target`

`locked: false` matters. [`gate`] checks §12.5.3 bit 8 **first**, so a
locked target would be refused by the older half of the ladder and the
certification assertion below would pass without the certification being
consulted at all.

### `fn a_certified_document_withholds_the_delete_control`

The end-to-end assertion for R83 on this surface: the engine's query is
asked, its `EditError` is mapped, and the operator-facing sentence is the
one about a certification rather than the catch-all. Every link in that
chain is exercised on a real file rather than on a constructed error.

### `fn the_gate_reads_the_selection_it_is_given`

# What went wrong, in one sentence

`canvas::interact` moves the selection off the document for the length of
a canvas frame (`std::mem::take(&mut doc.selection)`), and it filled in
`canvas::keys::Keys::annot_delete_refused` by calling
[`refuses_selected`] — which reads `doc.selection`. So the flag was
`false` on every frame, on every document, and the Delete key's
annotation rung never declined once: it raised the action, the engine
refused it into `vector_edit`'s silent `Err` arm, and the selection was
cleared anyway, removing the panel sentence that explained the refusal.

# Why THIS shape of test, and why the old one could not have caught it

The assertion in `a_certified_document_withholds_the_delete_control`
above is `refuses_selected(&doc) == doc.selection.annot().is_some()`,
which on a freshly-opened fixture is `false == false` — true of the fixed
build and true of the broken one. Every unit test in `canvas::keys`
likewise sets `annot_delete_refused` **by hand**, so none of them was
ever downstream of the call that was wrong.

What this test asserts is the property the caller actually needs: that
the two arguments are independent, by putting the annotation in a
**detached** selection and leaving `doc.selection` empty — which is
precisely the state `canvas::interact` is in when it asks. A build that
reaches for `doc.selection` answers `false` here and fails.

### `fn the_same_document_without_the_certification_offers_it`

A `gate` that refused unconditionally would satisfy the certified
assertion perfectly. The two fixtures differ in exactly one dictionary,
so this pins that the difference the gate reacts to is that dictionary
and not the presence of a signature, or of an annotation, or of a
pop-up.

### `fn the_collateral_names_the_popup_and_the_orphaned_reply`

`annotation_deletion_preview` on the square must find the `/Popup`
companion (§12.5.6.14 makes taking it a `shall`) and the one `/IRT`
referrer that Table 170's default `/RT` of `R` classifies as a **reply**.
Two clauses rather than one, because one would not prove the joining is
right.

Asserted through [`DeletionPreview::line`] — the memo — rather than by
calling the engine directly, so what is pinned is the string an operator
would read on the frame the annotation is selected, stamp and all.

### `fn the_second_frame_costs_no_engine_call`

The whole cost argument rests on this: `annotation_deletion_preview`
walks the page's `/Annots` looking for `/IRT` referrers, and the panel
would otherwise pay that every frame the annotation stayed selected. A
stamp hit is asserted by *poisoning the payload* and reading it back —
if the engine had been re-asked, the real sentence would have returned
and the poison would be gone.

### `fn a_moved_epoch_re_asks_the_engine`

Without the epoch term the panel would show the collateral of a document
state that no longer exists — a reply deleted a moment ago would go on
being counted, which is exactly the failure that makes a properties
panel untrustworthy. Asserted the same way round: the poison must be
gone.
