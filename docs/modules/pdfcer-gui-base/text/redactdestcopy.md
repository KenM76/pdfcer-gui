# `text::redactdestcopy` — where the redacted document goes


> *"why does it have to save to a new file right away? Why can't it just
> wait on saving until I choose to save over the existing file or save as a
> new file?"*

Until that day the apply had exactly one destination — a new file — and
`crate::dialogs::redact::commit` recorded the absence of any alternative as
a design property: *"There is no 'save over the original' branch to find,
because there is none to write."* `crate::dialogs::redact::Destination`
carries the whole of why that was overruled and what survives of it.

## Its own file, and the seam it was cut along

`text/redact.rs` reached 1536 lines with this group in it, over rule R2's
1500-line ceiling. The seam is not arbitrary and it is not "the last thing
added": these ten strings are the only ones in the catalog that describe a
**destination** rather than a **removal**, they are consumed by one region
of one dialog, and every other sentence in the parent file would read the
same if they did not exist. A split along "what was added most recently"
would have put `confirm_button` here and `confirm_button_replace` there.

Re-exported by the parent with `pub use`, so every call site spells these
exactly as it did before — `crate::text::redact::destination_replace(..)` —
and the split is invisible to consumers, which is the property that makes it
a mechanical change rather than a rename.

## The wording rule this group adds to the three it inherits

`crate::text::redact`'s rules 1–3 bind here unchanged. This group adds a
fourth of its own, and every string below obeys it:

> **Name the file.** *"Replace the original"* is a sentence about a role.
> *"Replace sheet-01.pdf"* is a sentence about a file, and the operator is
> about to destroy a file. Every string here that refers to the document
> being replaced takes its name as an argument, including the button label —
> which is how `crate::dialogs::redact::file_name_of` came to exist, so that
> one file is spelled one way across the choice, the acknowledgement, the
> button and the outcome.

## Item notes

### `fn destination_open_document`

It replaced [`destination_new_file`] as the default, and the swap is the
safer direction rather than the more convenient one: a new file is a write,
and this is not one. Nothing on disk changes until the operator saves, so
the default answer is now the only one of the three that cannot lose
anything.

Worded as *"this document"* rather than *"the session"* or *"in memory"* —
he is looking at a document, and the two nouns that would be accurate are
both ours rather than his.

2026-09-05: *"the removal happens when you Save"* rather than *"Save
decides where it goes, and when"*. The old half-sentence was true of the
collapsing verb, where the removal had already happened and only its
destination was outstanding. On `Pass 250.2` the removal itself is what is
outstanding, and a label that talks only about *where* invites the operator
to believe the *what* is already done.

### `fn destination_open_document_now`

Its wording carries the whole difference from its neighbour, because the
two destinations differ in **when**, not in **where**. *"Now"* is the
operative word and it does the work of a whole sentence: the row above puts
the removal off until a save, and this one performs it at the click.

⚠ It says *"the page changes"* rather than *"applies the redaction"*, because
what the operator reported losing was the **visible** result — he pressed the
deferred button and *"nothing happened"*. A label describing the mechanism
would have been true of both rows and would not have told them apart.

### `fn destination_open_document_now_tooltip`

It names the price **at the control**, not after the press. His own ruling
is on the record — *"finalizing the document and can't be undone is ok for
now"* — so this is a cost he has already accepted, and a cost accepted in
advance is still one that belongs where the choice is made.

*"No file is written"* is the clause that separates this from *Replace the
original*: two rows both remove content immediately, and only one of them
touches the disk.

### `fn permanence_statement_now`

Its own arm rather than reusing either neighbour, and the match it
belongs to says why in its own comment: *"a fourth destination that fell
through to the wrong arm here would be a false claim in the one place a
false claim is worst."*

### `fn confirm_button_into_document_now`

No ellipsis: nothing further is asked. An ellipsis on this button would
promise a picker that is not coming, which is the convention its `NewFile`
sibling relies on.

### `fn destination_open_document_tooltip`

**REWRITTEN 2026-09-05.** The sentence it replaces ended *"applying clears
the undo history, so nothing before it can be stepped back"*, which was the
price of `Pass 250.1`'s collapsing verb and is now false in the strongest
possible way: this route is the one that **keeps** the undo history, and
keeping it is the entire reason the engine shipped `Pass 250.2`.

Three facts, in the order an operator needs them: nothing is written and
nothing is removed *yet*; the page will not change, which is the surprise;
and undo still works, so this is reversible until it reaches a file.

### `fn removal_happens_at_save`

What the old sentence carried was the *price* of the collapsing verb: the
whole undo log went, and the operator had accepted that on condition he was
told the step count first. There is no price to state any more. What has
taken its place is a **surprise**, and it is arguably the more important
disclosure of the two:

> He presses a button labelled *"Permanently remove from this document"* and
> **the page does not change.**

Every redaction tool he has used draws a black box the instant he confirms.
This one arms a save. Without this sentence the two readings available to
him are *"it did not work"* and *"it worked and the marks are just still
drawn"* — and the second is the one that ships a marked file.

# Why no count, where the old one had two forms

The old sentence branched on the number of undo steps because *"this will
discard 14"* and *"this will discard 1"* are different decisions. Nothing
here is a quantity: the removal is armed or it is not, the page does not
change either way, and Save is the moment either way. A number would be
decoration on the one sentence in this dialog that must be read.

### `fn confirm_button_into_document`

No ellipsis, for [`confirm_button_replace`]'s reason inverted: there no
further question was coming because the file was already named, and here
none is coming because **no file is involved at all**. Promising a picker
with a punctuation mark would be a lie the operator acts on either way.

2026-09-05: *"Set up"* rather than *"Permanently remove from this
document"*. The old label was the consequence of the collapsing verb and is
now a description of something the button does not do — it arms a removal
that happens at the save — and on this dialog the standing rule is that
**the label IS the consequence**. A label claiming an immediate removal on a
control that performs none is the same defect as an ellipsis promising a
picker that never opens, one order of magnitude worse.

It still says *"the content"* rather than *"the marks"*, because the whole
misunderstanding this feature exists to prevent is that applying does
something to the marks rather than to the content.

### `fn permanence_statement_deferred`

A third form of [`super::permanence_statement`], and the only one of the
three whose first clause is about something that does **not** happen. The
middle clause — the impossibility of getting the content back — is worded
identically to its two siblings, deliberately: it is the part an operator
must not have to read twice to compare.

**REWRITTEN 2026-09-05.** Its first clause used to read *"Applying
removes the marked content from the document you have open, and writes
nothing"*, which was true of the collapsing verb and is now false in the
most expensive direction available: it claims a removal that has not
happened, at the top of the report, in the warning role, which is the one
sentence a reader who takes in nothing else takes in.

It still says the file on disk *"still contains that content"*, which is a
rule-4 disclosure and not reassurance. An operator who arms a redaction,
does not save, and hands over the original file has redacted nothing — and
that is a genuinely reachable state on this destination and on no other.

### `fn staged_into_document`

[`super::applied_clean`]'s sibling for the route where nothing has been
removed yet, and it is drawn in the edit-disclosure slot by the action
funnel rather than in a dialog — because arming a save is an ordinary edit
now, and an ordinary edit reports where every other one does.

**RENAMED from `applied_into_document` and rewritten, 2026-09-05.** The
old sentence began *"Redacted — N region(s) … removed from this document,
and verified absent from it"* and both halves of that are now false: nothing
has been removed, and nothing has been verified, because the engine's
staging verb discards the bytes there would have been to sweep
(`crate::redact` §1.0.1). Saying *"verified"* here would have been the
catalog's rule 2 broken at the one place it is load-bearing.

Rule 1 is kept mechanically: `residuals` picks between two genuinely
different sentences rather than putting a number into one.

Both forms say the same two things at the end — **the page has not
changed** and **Save is what does it** — because those are the two things
this route can get wrong that the write-now routes cannot.

### `fn saved_applying_redaction`

New 2026-09-05, and it carries three facts an operator has no other way to
learn. It is recorded by `crate::app::save` on every one of the three save
verbs, and on `file.save` it is recorded **after** the ordinary
*"saved to …"* receipt, deliberately — the slot holds one disclosure, and
this is the one rule 4 says wins.

1. **It happened.** The word *verified* is earned here and nowhere else on
   this route: `crate::redact::save_applying_pending` swept the exact bytes
   for the removed text before returning them, and `crate::app::save`
   swept them again between the buffer and the syscall.
2. **The window is now stale**, in the same way and for the same reason
   [`super::applied_clean`]'s replace form is stale: the session was never
   mutated, so the canvas goes on drawing the marks and the content while the
   file no longer holds either. Getting this wrong in the reassuring
   direction — *"the document is redacted"* — teaches him that a page which
   still shows a name is a page whose name was removed.
3. **It is still armed.** `save_applying_redaction` takes `&self` and does
   not clear the flag, so the next save applies the removal again and the
   ordinary save modes stay refused until he cancels. *"I saved it, so it is
   done"* is the assumption that would otherwise stand, and it is wrong in
   the direction that surprises him at the next `Ctrl+S`.

### `fn staging_cancelled`

New 2026-09-05 with [`cancel_button_staged`]. Short, and it says the two
things a cancel has to say: the removal will not happen, and **the marks are
still there** — because taking the arming off is not taking the marks off,
and an operator who read this as *"never mind, that is dealt with"* would
have a marked document he believes is a clean one.

### `fn staged_body`

It deliberately does **not** repeat the report. The numbers were measured at
the moment of consent and the removal re-runs at the save over whatever the
document says then, so quoting them here would present a stale measurement
as a current one — which is the failure `crate::redact::StagedRedaction`'s
own doc comment calls a preview rather than a receipt.

### `fn cancel_button_staged`

It exists because **a stageable operation that cannot be un-staged is a
trap**, and the trap has teeth here: while a removal is armed the engine
refuses both ordinary save modes by name, so an operator who changed his
mind and had no way to say so could not save his document at all.

Worded as *"call off"* rather than *"cancel"*, because this window already
has a *Don't apply yet* control and two buttons whose labels both read as
*cancel* on one screen is how the wrong one gets pressed.

### `fn destination_heading`

> *"why does it have to save to a new file right away? Why can't it just
> wait on saving until I choose to save over the existing file or save as a
> new file?"*

Worded as a question about **this document** rather than as a settings
label, because it is asked once, here, about one operation.

### `fn destination_replace`

It **names the file**. *"Replace the original"* is a sentence about a
role; *"Replace sheet-01.pdf"* is a sentence about a file, and the operator
is about to destroy a file.

### `fn destination_replace_tooltip`

Two facts, in the order they matter: the file being replaced is the **only
remaining copy** of what is being removed, and nothing in pdfcer brings it
back. Neither is a scold and neither is a refusal — the operator asked for
this control, and a control that argues with the person using it is a
control that gets clicked past.

### `fn overwrite_acknowledgement_checkbox`

Distinct from [`confirm_checkbox`] because it is a different fact.
That one is about the *content* — that applying removes what is underneath
rather than the marks on top. This one is about the *document*: that the
file the operator opened will not be there afterwards.

A person can perfectly well have understood the first and not noticed the
second, which is exactly the operator this box exists for. It is asked for
**only** while the replace destination is selected, so it never becomes a
box that is always there and therefore always ticked.

### `fn confirm_button_replace`

No ellipsis, and that is the point of the wording. On
[`confirm_button`] the ellipsis is *"a promise that a further question is
coming"* — the file picker. Here no further question is coming, so promising
one would be a lie told by a punctuation mark, and the operator would press
it expecting a chance to change their mind.

It names the file for [`destination_replace`]'s reason.
