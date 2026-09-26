# `dialogs::redact` — the Apply-redactions transaction

The body of `edit.redact_apply`, and the **irreversible** half of the
redaction feature. Its reversible twin is [`crate::panels::redact`], and the
split between them is the distinction
`crate::text::commands::edit_redact`'s shipped tooltip already draws:
*"Marking is reversible; applying is not."*

This is the only surface in pdfcer-gui that commits an operation nothing can
take back, and its whole shape follows from that.

## The five states

| state | what the operator sees | what exists |
|---|---|---|
| **prepared** | the measured report, a destination choice, up to three checkboxes, and a control whose label is the consequence | the finished redacted bytes, **in memory** |
| **staged** | that a removal is already armed, what that means, and one control that calls it off | a flag on the session, and nothing else |
| **refused** | a named refusal, and nothing to confirm | nothing |
| **written** | where the file went, whether it replaced the open one, and what is still in it | a file |
| **write failed** | why no file appeared | nothing |


There is deliberately no *ready* state. Opening this dialog **runs the whole
removal** — see §2 — so by the time anything is drawn the numbers on screen
are measurements of the exact bytes that will be written, not predictions
about bytes that do not exist yet.

## 1. Why the report comes BEFORE the write, and the operator chooses the
destination

`crate::dialogs::ocr`'s argument, one operation further along the scale of
consequence. That dialog recognises, discloses what it inferred, and only
then offers to save — so *"the operator reads the disclosure while holding
the one thing that gives it force: the ability to not save."*

Here the disclosure is not about inference, it is about **what will be
destroyed and what pdfcer could not destroy**, and the residual half of it is
the whole reason the feature can be trusted. A surface that redacted and
dropped a file picker in front of the operator would be technically
disclosive and practically a program that quietly shipped a partially
redacted document.

**2026-09-04 — the destination is the operator's, not this dialog's.**
This section used to end by arguing that the write must always be to a new
file. The operator overruled that: *"why does it have to save to a new file
right away? Why can't it just wait on saving until I choose to save over the
existing file or save as a new file?"* [`Destination`] carries the whole
argument and what survives of the old ruling (the safe default, and
[`suggested_path`] never proposing the source).

**CORRECTED the same evening.** That paragraph ended, at midday, by
naming *"the one half of his request the engine cannot express — deferring
the write to a later Save, which would need a redaction that mutates an
`EditSession` and there is no such verb."* **There is now.** `Pass 250.1`
shipped `EditSession::apply_redactions` the same afternoon, in answer to
this shell's filing, and [`Destination::OpenDocument`] is the deferred
destination it makes possible — **and it is the default**. There are three
destinations now, not two, and the write-now pair is what is left of the
original design rather than the whole of it.

## 2. Why the removal runs synchronously, on open

It is the salvage source's shape and it is kept, with the trade stated
rather than inherited.

The alternative is `crate::ocr::Job`'s: a worker thread, a spinner and a
poll. Everything needed for it is available — `OpenDoc::session` is an
`Arc<EditSession>` and every field of [`crate::redact::PreparedRedaction`]
is `Send`. It is not done, for one reason that decides it: **a report
computed on another thread is a report about a document that may have
changed by the time it is read.** OCR can tolerate that because it refuses
outright when `edit_epoch != 0`; a redaction cannot, because the marks the
operator is applying are the ones they have just made and the epoch is
moving by construction.

Running it inside the dispatch that opens the dialog gives the report and
the bytes one consistent snapshot, taken at a moment the operator caused. The
cost is a frame that takes as long as a full rewrite of the document —
visible on a large sheet, and paid once, on a deliberate click, for the one
operation in the program where a stale answer would be a security defect.

## 3. What confirmation actually consists of, and why it is not one click

Four gates, and each closes a different failure:

1. **[`crate::text::redact::confirm_checkbox`]** — always present. Its
   wording targets the exact misunderstanding the feature exists to prevent:
   that applying removes the *marks* rather than the *content*.
2. **[`crate::text::redact::residual_acknowledgement_checkbox`]** — present
   **only when the report has residuals**. Showing it always would make it a
   box operators tick without reading, which is how every acknowledgement in
   a program becomes worthless. It is also enforced below the UI, at
   [`crate::redact::PreparedRedaction::write_to`], because a greyed control
   is a drawing decision and not a mechanism.
3. **[`crate::text::redact::overwrite_acknowledgement_checkbox`]** —
   present **only when the operator has chosen to replace the open file**
   (2026-09-04). A different fact from gate 1: that one is about the
   *content*, this one is about the *document*. Somebody can have taken in
   that the text is going for good without noticing that the file they
   opened is going with it. Conditional for gate 2's reason — a box that is
   always there is a box that is always ticked.
4. **A control whose label is the consequence** — never "OK", never
   "Apply". One label per destination, and the punctuation is part of the
   claim: *"Permanently remove & save as…"* on the new-file destination,
   where the ellipsis promises the picker that really is coming;
   *"… & replace `<name>` now"* on the replace destination, which names the
   file and drops the ellipsis because no further question follows; and
   *"Set up the removal — it happens when I save"* on the default, which
   promises nothing further because no file is involved and **claims no
   immediate removal, because there is none**. An ellipsis on a control that
   asks nothing more is a lie the operator acts on, and so is a label
   claiming a removal that has not happened.

…and, between the destination choice and the button, **a disclosure
rather than a gate**: [`crate::text::redact::removal_happens_at_save`], drawn
only on the deferred destination. It is deliberately NOT a fourth checkbox —
§3's own argument about conditional boxes applies to their multiplication
too, and four acknowledgements is a form, which is filled in rather than
read. What the operator is owed here is the FACT, before the click.


And a fourth thing that is an absence: **no keyboard shortcut, and no Enter
binding.** The footer says so in words rather than leaving it to be noticed.
Every other destructive verb in this shell is chorded and reversible; this
one is neither, and the asymmetry is deliberate.

## 4. The `ready` flag is read one frame late, on purpose

[`RedactDialog::show`] computes whether the confirm control may be enabled
**before** the checkboxes are drawn, so a checkbox ticked on this frame does
not enable the button until the next one. A fast double-click on the box
would otherwise land its second press on a control that became enabled
between the two — which on this dialog means an irreversible operation
reached by a gesture the operator made at a disabled control.


What stood here at midday, and it was right about the world it described:

> *"[`super`]'s rule: a dialog uses the action funnel when it edits **this**
> document, and this one never does. Applying produces *bytes on disk*; the
> open document keeps its marks, its undo log and its epoch whichever
> destination was chosen."*

[`Destination::OpenDocument`] edits **this** document, so it takes the
funnel, and by [`super`]'s own rule rather than despite it. The two
write-now destinations are unchanged and still push nothing: they produce
bytes on disk and leave the session alone.


| press | what it changes | route |
|---|---|---|
| confirm on [`Destination::OpenDocument`] | the session's pending-redaction flag | `RedactAction::Pending(Staging::Stage)` → `crate::app::actions::redact` → `vector_edit` |
| *call the removal off*, in [`Phase::Staged`] | the same flag, back off | `RedactAction::Pending(Staging::Cancel)` → the same arm |
| confirm on [`Destination::NewFile`] | a file | [`crate::redact::PreparedRedaction::write_to`], here |
| confirm on [`Destination::ReplaceOriginal`] | the source file | the same, atomically |

What the funnel's reasoning demanded and still demands is that irreversible
work not run part-way through a layout pass — and it does not, on any of
them: every control sets a flag, and the push, the picker and the write all
happen after the window's closure returns.

**After a replace, the open document is deliberately STALE, and the
outcome sentence says so.** The session was not touched, so the canvas goes
on drawing the marks and the content underneath them while the file those
bytes came from contains neither.

The reason that used to be given for it — *"`EditSession` has no verb
that could"* — is no longer true, and the staleness is now a **consequence
of the destination the operator chose** rather than a limit of the program.
It is still not tidied away by swapping the session underneath, and the old
argument for refusing that manoeuvre stands untouched: a swap discards the
whole undo log without saying so, and `crate::app::save::save_as` refuses it
for the same reason. An operator who wants the open document to change now
has a control that says so. One who chose to write a file gets a file, and
the divergence is **disclosed** rather than hidden, in
`crate::text::redact::applied_clean`'s replace form, which tells him by name
which file to reopen. Rule 4: report separately, and do not pretend.

## 6. It is document-scoped, and closing the document discards the bytes

`crate::dialogs::ocr`'s ruling, and it matters more here: a redaction is of
*these marks* on *this file*, and writing prepared bytes after the operator
has put the document away would produce a redacted file derived from a
document nobody is looking at any more.
