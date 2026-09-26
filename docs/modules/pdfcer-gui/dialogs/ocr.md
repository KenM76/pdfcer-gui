# `dialogs::ocr` — the Recognise-text transaction

One dialog, three states, and a shape that is chosen rather than
conventional. It is the surface for `file.ocr`, and it is also the
**enforcement point for two rules** that would otherwise have nowhere to
live in this build.

## The three states

| state | what the operator sees | what exists |
|---|---|---|
| **ready** | what OCR does to the page, and one button that starts it | nothing |
| **working** | *Recognising…* | a thread |
| **answered** | the disclosure, then a Save-as button — or a named refusal | bytes, in memory |

## Why the recognition is disclosed BEFORE it is written, not after

This is the whole reason the dialog has a third state instead of running
OCR and immediately opening a file picker.

Project rule 4 is *"fuzzy, never sneaky"*, and `pdfcer-core`'s own OCR
header sharpens it for exactly this feature: **every word an OCR layer
contains is a guess**, and this engine reports no confidence for any of
them. A surface that recognised a page and dropped the finished file in
front of the operator would be technically disclosive — the report would be
*somewhere* — while being, in practice, a program that silently inserted
several hundred unreviewed inferences into a document. This project's
characteristic failure is a surface that is *correct* and *unreadable at the
moment it matters* (`DEFECTS.md`).

So the order is: recognise, **show what was inferred**, and only then offer
to write it. The operator reads the disclosure while holding the one thing
that gives it force — the ability to not save. That is not a nicety; it is
the difference between a disclosure and a receipt.

## Why the write is a Save-as, in every mode

The standing rule is *Read may produce a new document; it may not modify
this one*, with the enforcement at the **save** rather than at the
operation.

The rule is **vacuous** in this shell, and that is worth saying rather
than leaving as an apparent guarantee: `file.save_copy` asks for a
destination too, and `crate::app::save::suggested_path` guarantees the
*suggestion* is never the file that was opened, exactly as
[`suggested_path`] below does here. The two surfaces share one picker,
`crate::app::files::pick_save_path`, and the only thing that differs between
them is the dialog's title.

It bites here in the only way that is
honest: the destination is a path the operator names, so the rule holds in
Read **and** in Edit **and** in Review by construction rather than by a mode
check. Nothing here consults the mode, and nothing here should — the rule is
about what a save may overwrite, not about who is asking.

What is deliberately **not** done: no second save command, no in-place path,
no `Save`-labelled control anywhere. The day in-place `Save` lands it will
need its own Read-mode gate, and that gate belongs beside it rather than
being invented here in advance against a command that does not exist.

## Why OCR is available in Read, with no capability flag

`app::modes::capability` governs **gestures** — what a press on the canvas
means. OCR is not a gesture; it is a command with a dialog, and it changes
no document that is open. Adding a capability flag for it would put a rule
about *saving* into the machinery that decides what a drag does, where the
next reader would neither look for it nor believe it.

Read is therefore offered OCR exactly as Edit is, and that is the operator's
instruction rather than an omission.

## Why this dialog does not push an `Action`

[`super`]'s rule: a dialog uses the action funnel when it edits **this**
document, and this one never does. The recognised bytes are a *new*
document; the open one is untouched, its `edit_epoch` does not move, and
there is nothing to order against or to undo. What the funnel's reasoning
does still demand is that irreversible work not happen part-way through a
layout pass — and it does not: the button sets a flag, and the file is
written after the window's closure returns.

## What is document-scoped about it

Everything. A recognition is of *this page* of *this file*, so
[`super::DialogsState`] holds it in the document-scoped group and closing
the document closes it. A finished-but-unsaved recognition is discarded
with it, which is the right answer: writing it afterwards would produce a
file derived from a document the operator has already put away.

## Item notes

### `const REGION_RUN`

Declared **only while it exists**, which is itself the assertion a harness
wants: this control is drawn if and only if the dialog is in its ready
state with a resolvable page scope, so its absence from the trace is
evidence that the run could not have been started rather than that the
harness missed it.

There is deliberately no region for a save control. Recognition is an edit
to the open session, so there is no transaction to complete and nothing for
a second button to do.

### `const REGION_PROGRESS`

Published so a driven check can assert the operator can SEE the run
moving. A feature whose entire purpose is to show that the program has
not frozen needs an oracle that is about visibility.

### `enum Phase`

A state machine rather than three `Option`s, because the states are
mutually exclusive and an `Option` triple has five nonsense combinations
that would all compile.

### `fn poll_worker`

Separate from [`Self::body`] so the transition happens once per frame
regardless of what the window drew, and so that a dialog scrolled out
of view still notices its own worker finishing.

### `fn ready`

The order of the checks is the order of the questions, and it is not
arbitrary. *Can this build look at all* comes before *are the files
there to look with*, because asking them the other way round would
report a missing model directory in a build that has no recogniser to
use it — a true statement and the wrong diagnosis.

### `fn scope_group`

# Why radios and not a dropdown

Three choices, all short, all mutually exclusive, and one of them opens
a text field. A dropdown would hide two of the three behind a click and
would have nowhere sensible to put the range field; radios show the
whole decision at once, which is what every surveyed recogniser does
with the same three options.

# Why the range field is always visible

Rather than appearing when **Pages** is chosen. A field that appears
changes the dialog's height mid-interaction, and the operator reaching
for the radio has to then find where everything moved to. It is
disabled instead when another scope is active — and clicking it selects
[`Scope::Range`], because typing into a range field is an unambiguous
statement of intent and making them click the radio first would be
pedantry.

### `fn preflight`

Returns `None` when recognition may proceed. Pulled out of [`Self::ready`]
so that the decision is a pure function of the document and is therefore
reachable from a test — the button and the window are not.

### `fn answered`

**The confidence sentence is drawn first and separately, above the
list.** `OcrLayerReport::disclosures()` already contains a sentence
making the same point, and this is deliberate duplication rather than an
oversight: the engine's version sits fourth in a list of counts, and the
one fact that must survive a skim is that **nothing here was scored**.
A reader who takes in one line takes in that one.

Drawn in the plain text role, never `.strong()` — `DEFECTS.md` D11
records that role as unusable in this theme, and a named palette exists
so a surface written later does not rediscover it.

### `const LIST_FLOOR`

See [`OcrDialog::answered`] — without a floor, a small window produces a
list that draws nothing at all and looks like a recognition that disclosed
nothing, which is the exact opposite of what this dialog is for.

### `fn sentence`

One place, so the dialog cannot word a refusal differently from anywhere
else that grows a need for it, and so `text::ocr`'s catalog is the only
thing that has to be read to know what pdfcer says when OCR declines.

### `fn user_data_dir`

`None` today, and that is a statement rather than a stub: this shell has no
user-data directory of its own — `app::persistence` writes its layout beside
the executable — so there is no second place to look and reporting one would
name a path in a "searched here" list that was never searched.

It exists as a function because `ocr::resolve_models` takes the parameter
and the day a user-data location appears there is one call site to change
rather than three.

### `fn an_unsaved_edit_no_longer_refuses_recognition`

# Why refusing would be the defect, not the safeguard

A recognise path that reads the document's **base** revision —
`ocr::layer::add_ocr_layer` does — produces a recognised copy that
silently omits every edit made since, so refusing an edited document
looks like the safe answer.

It is not: the base never becomes current, not even after a save, so
such a refusal is permanent from the first edit onward and the operator
is stuck in it. `EditSession::add_ocr_layer` plans against the
**session graph** instead, which removes the divergence rather than
policing it.

So this test now pins the *absence* of the guard, and it is worth
having as a test rather than as a deletion: the trap was re-introduced
once already, in a different spelling, and a named assertion is what
makes a third spelling fail rather than ship.

### `fn the_suggested_name_is_never_the_source_file`

The standing rule as a default. An operator who accepts the suggestion
without reading it must not overwrite their scan, and this is the
assertion that says so — the label and the tooltip say it in words, and
words are not a mechanism.

### `fn each_named_refusal_says_something_different`

The property the whole `Refusal` enum exists for: `pdfcer-core`'s error
types refuse by name because "OCR failed" is unactionable, and a shell
that mapped four named causes onto one sentence would throw that away at
the last step.

### `fn a_missing_model_directory_names_every_place_that_was_tried`

`models::ModelsNotFound` carries them precisely so the operator learns
where to put the files; dropping them at the display boundary would
undo that in the last inch.

### `fn this_page_only_is_the_captured_page`

The operator can page the document while the window is up. A scope that
resolved against the live index would recognise a page they had moved
away from — which is why `page_index` is captured at `open` and threaded
here rather than read from the document.

### `fn the_picked_pages_are_the_pages_picked`

The operator: *"the pages I have selected in the thumbnails."*

Asserted as passed through **unchanged and in order**, because the two
tempting mistakes are both silent: re-deriving the set here would put a
second page selection in the program, and sorting or de-duplicating it
would hide a caller handing over something malformed.

### `fn a_picked_page_the_document_lost_is_dropped`

The selection is captured when the dialog opens and the document can be
edited underneath it — deleting a sheet from the rail while this window
is up would otherwise hand the engine an index past the end. Filtering
rather than refusing, because the pages that survive are still the ones
he asked for; refusing the whole run would punish him for an edit he
made deliberately.

### `fn the_range_is_parsed_by_the_print_dialogs_parser`

Two page-range parsers in one program would accept different things on
two surfaces and the operator would have to learn which one they were
talking to. This asserts they are the same parser rather than merely
similar: the expectations below are `parse_page_range`'s own, taken
from its behaviour rather than restated.
