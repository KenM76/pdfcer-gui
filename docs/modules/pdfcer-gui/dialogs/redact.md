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

## Item notes

### `const REGION_DESTINATION_INTO_DOCUMENT`

Declared **unconditionally**, unlike its two siblings, and that asymmetry is
the assertion: this destination is available on every document, including
one created in this session that has no file to replace, so its ABSENCE
from a trace is evidence about the build rather than about the document.

### `const REGION_DESTINATION_NEW_FILE`

Published so `tools/ui-verify` can **click** it. Its redaction check
drives the whole feature to a file — that the source was not touched, that
the output lacks the secret, that a second process extracts nothing from it
— and the default destination produces no file at all, so the harness has to
move off the default deliberately and needs a rect to move to.

### `const REGION_STAGING_NOTE`

It is a region rather than only a string so a harness can assert that the
sentence is *above the confirm control*, which is the whole of its value:
`tools/ui-verify`'s redaction check can compare this rect's bottom against
[`REGION_CONFIRM`]'s top and fail if the disclosure ever moves below the
button it is meant to precede.

**The name must keep describing the sentence.** A region name that
says one thing while the label below it says another aims a harness at a
sentence it will not find, and the check then passes while measuring
something else.

### `const REPORT_FLOOR`

Without a floor, a small window produces a scroll area that draws **nothing
at all** — `available_height()` minus a reservation goes negative, and a
negative `max_height` is a silently empty area rather than an error. On this
dialog that would be a confirmation with no report above it, which is the
one shape it must never take. The About and OCR dialogs record the same
trap.

### `enum Phase`

A state machine rather than several `Option`s, because the states are
mutually exclusive and an `Option` quadruple has combinations that would all
compile and none of which means anything.

### `fn open`

The whole removal runs here — see §2 — so this call is as expensive as a
full rewrite of the document, once, on a deliberate click.

`reach` is read from the operator's preferences by the caller, once,
at the moment the window opens. It is deliberately not re-read while the
window is up: every number on screen was computed at one reach, and a
value that could move underneath them would make the report describe a
removal other than the one *Apply* performs.

### `fn take_cancel`

A method rather than four lines inside [`Self::show`], and the reason is
this suite's standing one: [`Self::show`] needs an `egui::Context` and a
real viewport, so nothing inside it can be asserted headlessly, and the
one thing worth asserting about this press is **which action it
raises**. A build that raised [`crate::redact::Staging::Stage`] here
would re-arm the removal the operator just asked to call off, silently,
on a control whose label says the opposite.

It pushes an `Action` rather than touching the session. The engine's
`cancel_pending_redaction` takes `&mut EditSession`, `Arc::get_mut` is
the funnel's second step, and performing that from inside a dialog's
draw is exactly what the funnel exists to prevent.

It closes the window. The outcome is reported by the funnel's edit
disclosure like any other edit, and a window left open beside it would
be a second account of one event — and, worse, an account of a state the
document is no longer in, since this phase exists only while a removal
is armed.

### `fn ready_to_confirm`

Pure, and the whole of the gate's rule — so every property of it is
asserted headlessly, which is `crate::viewer`'s standing split applied
to the one control in the program that must not be enabled early.

### `fn staging_disclosure`

Pure, and a method rather than three lines inside [`Self::gates`], for
[`Self::choose_destination`]'s reason: this is the disclosure that
stands between a button labelled *"the removal happens when I save"* and
an operator who has never used a redaction tool that did not blacken the
page instantly. A property that load-bearing is asserted headlessly
rather than left to a reading of the draw order.


`None` on the two write-now destinations, and that is a claim rather
than an omission: those routes do produce a file at the click, so a
sentence saying nothing is written would be false there.

### `fn can_replace_original`

`is_file` rather than a flag, and the question is asked of the **file
system**, exactly as `crate::app::save::has_a_file` asks it and for the
reason recorded there: *"a `created_here: bool` flag is a second source
of truth… and the failure mode when it drifts is writing over the wrong
file."*

A document created in this session has a bare name rather than a path,
so there is nothing to replace and the choice is not drawn — an inert or
meaningless control being worse than an absent one (the no-inert-controls
rule).

### `fn choose_destination`

Pure, and a method rather than four lines inside [`Self::gates`], so
the rule can be asserted headlessly — `crate::viewer`'s standing split
applied to the one flag that stands between a click and the deletion of
the source document.

The rule: **changing the destination un-ticks
[`Self::overwrite_acknowledged`].** Without it, an operator could tick
the box, think better of it, select *a new file*, change their mind
again, and arrive back at *replace* with the button already live — the
consent standing from a decision they had explicitly withdrawn in
between. That is not a hypothetical sequence; it is what "I'll just look
at what the other option says" looks like from the program's side.

It fires on **any** change of destination rather than only on leaving
the replace choice. Retiring a tick that was not needed costs nothing;
deciding *which* changes matter is where a future edit gets it wrong.

### `fn report`

Every optional line is drawn **only when its count is non-zero**. A
report that listed "0 annotations removed" beside four real findings
would train the operator to skim it, and the skim is what this whole
surface exists to prevent.

### `fn commit`

> *"It asks, every time, and the suggestion is never the file that was
> opened — see [`suggested_path`]. There is no 'save over the original'
> branch to find, because there is none to write, and on this operation
> that is the difference between a copy and the destruction of the only
> remaining source of the content being removed."*

The operator overruled it, and the reasoning is in [`Destination`]. What
survives of the old ruling is the part that was a *mechanism* rather than
a *prohibition*: [`suggested_path`] still never proposes the source file.
What is gone is the refusal to write the branch at all.

⚠ **Corrected 2026-09-05.** This paragraph said *"[`Destination::NewFile`]
is still the default"* — and by then [`DEFAULT_DESTINATION`] two hundred
lines above it read [`Destination::OpenDocument`], moved on 2026-09-04.
**One file asserting two different defaults about itself**, which is
worse than a stale sentence in a document nobody reads: this is the
paragraph a future session consults *before* changing the default.

The claim it was making is still true of the mechanism, and that is
why it survived a rewrite of the surrounding argument: both defaults are
safe, for **different reasons** — `NewFile` never *overwrote*,
`OpenDocument` never *writes*. A sentence that is right about the
principle and wrong about the value is the hardest kind to notice.

So there are now two paths, and the asymmetry between them is the whole
safety argument:

| destination | how the path is obtained | what stands between the click and the write |
|---|---|---|
| [`Destination::NewFile`] | the save picker, suggesting `-redacted` | the picker itself, plus the OS's own overwrite prompt if the operator navigates onto an existing file |
| [`Destination::ReplaceOriginal`] | [`Self::source`], with no picker | a **third** checkbox naming the file, and a confirm button whose label names it too |

Replacing takes no picker **deliberately**. A picker pre-filled with
the source would be a dialog whose safe answer is to change the field,
which is the shape of every accidental overwrite there has ever been.
The consent is taken before the click, in words, at a control the
operator had to select; once taken, the program does what it said.

The write itself is atomic — temp file, then rename — because on this
path a torn write destroys the last remaining copy of the content being
removed. See [`crate::redact::PreparedRedaction::write_to`].

### `fn file_name_of`

The name rather than the whole path, because every sentence that needs one
is read in a window about 700 pt wide and a Windows path is routinely longer
than that. Falls back to the whole path when there is no final component,
which is the only case in which the longer string is the more informative
one.

Shared by [`outcome_line`] and by the destination controls so the file is
spelled the same way in the choice, in the acknowledgement, on the button
and in the outcome. Four different spellings of one file name on one screen
is how an operator ends up unsure which file the sentence is about.

### `fn outcome_line`

Free rather than a method so the catalog's rule 1 — *a residual is named in
the same sentence as the success* — is decided by a pure function a test can
drive, rather than inside a `match` on a window's state.

The branch is on `residuals`, and the two sentences are genuinely different
copy rather than one with a number in it. An operator who acknowledged a
residual in this dialog and then closed it is owed a standing record of what
remains, and *"…and verified absent from the saved file"* would be a lie in
that case rather than merely an omission.

The **file name** rather than the whole path, because the sentence is read
in a window that is about 700 pt wide and a Windows path is routinely longer
than that. The full destination is on the trace line
`PreparedRedaction::write_to` emits, which is where a reader who needs it
will look.

### `fn residual_lines`

The single expression that both gates the extra acknowledgement and prints
the section — one derivation, so a residual can never be listed without
being acknowledgeable or acknowledged without being listed. Three sources,
in this order:

1. **carriers the engine could not scrub** — `CarrierAction::
   DisclosedNotScrubbed`, the cardinal-rule-honest outcome for a carrier
   this build cannot fully redact;
2. **raw-byte residuals** — [`crate::redact::proof`]'s middle verdict, a
   byte run that survives outside every decoded stream and that pdfcer
   genuinely cannot classify;
3. **retained marks** — regions where nothing was removed because the image
   under them could not be decoded. The engine names this as the number to
   read before saying "redacted", and it is the strongest kind of residual
   on this list: the content is still there, under a rectangle that says it
   is not;
4. **vector geometry that could not be cut**, and **clips whose outline had
   to be kept** — an outline on a drawing can be as identifying as the text
   it surrounded;
5. **objects promoted out of a compressed container** by materialising the
   operator's unsaved edits (engine rule R38).

The last is the mildest and is listed anyway. Page content cannot live in
an object stream at all (ISO 32000-1 §7.5.7), so it cannot hold redacted
text — but it is a leftover of the operator's own edits, and a report that
silently drops the findings it judges harmless is a report whose judgement
the operator has no way to audit.
