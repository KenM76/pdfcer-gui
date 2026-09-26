# `app::lifecycle` — opening a document, closing it, and the three ways an open can fail

Three methods on [`PdfcerApp`] and one predicate: what happens when a
document arrives, what happens when it leaves, and how a load failure is
told apart from a file pdfcer has not finished supporting.

## Why this is its own file

`app/state.rs` crossed the 1,500-line gate (rule R2) when canvas text
selection added the page-text cache and the text selection to [`OpenDoc`].
The rule's own justification is why the split is here rather than at
whichever line the count happened to reach: *"the value of the limit is that
the file has to have a single subject"*.

`state.rs`'s subject is **what an open document is** — the fields, the
render keys derived from them, the view overrides, the caches that hang off
it. This file's subject is **the document's lifetime on the application**:
`self.status` moving between [`Status::Empty`], [`Status::Open`] and
[`Status::Failed`], and everything that has to be forgotten on the way. The
two change for entirely different reasons — a new per-document cache is a
`state.rs` change, a new thing to forget on close is a change here — and
they are read at different times.

It is the same seam `app/mod.rs` has already been split along three times,
producing `dispatch.rs` (*what does this verb do*), `conditions.rs` (*what
is true right now*) and `gating.rs` (*what is this mode allowed to do*). The
test for whether a split was along a seam is whether the tests came with it,
and they did: the four below are all about the *transition*, and none of them
reads a field of [`OpenDoc`] except to check it was reset.

## The three ways an open fails, and why they are three

`crate::text`'s header carries the copy argument — *the file is wrong*, *the
file is fine and pdfcer is not finished*, *the file is encrypted and pdfcer has
not been told the password*. What lives here is the **branch**, and its one
rule: it is made on **structured error data** from `pdfcer-core`, never by
inspecting a message string. [`is_unsupported_structure`] is that rule, in
one place, so a new refusal from the engine is added to a `matches!` rather
than to a substring search that decays silently.

## Item notes

### `fn active_document_path`

A created document's `path` is a *name*, not a location — see
[`crate::app::state::OpenDoc::origin`] — so re-reading it would open
whatever happens to sit at `Untitled 1.pdf` in the working directory.

### `fn open_path_inner`

See the latter for why the two share one, and for what `None` means as
distinct from `Some` of an empty password.

# Returns

Why the supplied password did not work, when one was supplied and it did
not. `None` on success **and** on every failure that is not about the
password, because the prompt has nothing to say about a damaged file.

The two password failures are carried out separately rather than
collapsed into "it did not open", and `pdfcer-core` went to some trouble
to make that possible: `PasswordRequiresNormalisation` exists, in its own
words, *"so that failure does not masquerade as `PasswordRequired`'s 'you
typed it wrong', which would send the operator to re-check a password
that was correct."* Flattening them here would undo that on the last
step, which is the only step the operator sees.

**`options` is not a defaulted argument and must never become
one.** It is *which reading of a self-contradicting file this is*, and
both call sites state it: `open_path` writes `LoadOptions::new()`
because the ordinary open takes pdfcer's documented choices, and
[`Self::reread_active_document`] passes the operator's. The value is
stored on the resulting [`OpenDoc`] and carried by
[`Status::NeedsPassword`], so no route through this function can lose
it — which is the property that stops the feature from silently
declining itself on an encrypted file.

### `fn adopt_created`

Extracted when the size chooser arrived, for the reason [`Self::adopt`]
itself was extracted: every statement here is something that must be
true of a created document, and two copies would eventually agree about
four of the five. The naming, the counter, the trace and the failure
arm are identical for both verbs; only the bytes differ, and the bytes
arrive already made.

### `fn adopt`

Extracted when `file.new` arrived, and the extraction is the point
rather than a tidy-up: every statement below is something that has to be
**forgotten or re-derived because the open document changed**, and
leaving them inside `open_path` would have meant `new_document` either
duplicating five of them or silently skipping one. The panels keeping a
previous document's expanded rows after a New is the same defect as
keeping them after an Open, and it would have been found later and by an
operator.

### `fn is_unsupported_structure`

Matched on the structured error, never on its message. Today the live
case is an encryption configuration pdfcer will not decrypt (§7.6) —
reached either as the cross-reference layer's capability-gap refusal or
as a `crypto::EncryptionUnsupported` in its own right.

### `fn open_path`

The document is loaded **read-only**: `Document::load` maps the
bytes, `page_tree::pages` flattens the page tree, and nothing here
writes. S0 is a viewer.

Note the deliberate structure of the match: each `Err` arm is chosen
by *structured* error data, never by inspecting a message. See the
module docs on the three-way failure distinction.

### `fn open_path_with_password`

`OPERATOR_REQUESTS.md` O108. The retry half of [`Self::open_path`], and
its only caller is the dispatch of the action
[`crate::dialogs::password::PasswordDialog`] raises.

# Why the two are one function underneath

Because everything after the load is identical — the page-tree check, the
three-way failure branch, the settings funnel, the new tab, the adopt —
and the *only* difference is which loading verb is called. Two copies
would be two places to update when the failure branch grows a fourth
case, and this shell has already paid once for a predicate with two
claimants (`text_edit_focused`, which cost the Delete key and then the
space bar).

# `Some(pw)` and `None` are different requests, not a defaulted one

`Document::load(path)` means *"try the empty user password, then give
up"* — which every conforming reader does silently before prompting.
`load_with_password(path, Some(pw))` means *"try this one"*. So a caller
with nothing to offer passes `None` and gets the silent attempt; the
prompt refuses an empty box locally rather than passing `Some(b"")`,
because that would ask the engine a question it has already answered and
return a rejection the operator reads as *"my password was wrong"*.

# Returns

Why the password did not work, or `None` when the document opened.

### `fn reread_active_document`

# The ruling, and which third of it was missing

> *"We should be making pdfcer so that it opens pdfs that have errors,
> and have a way that it manages those errors such that they aren't
> fatal, and if the user can intervene in a decision that should always
> be an option along with them not having to intervene."*

Three obligations. Two of them shipped here on 2026-09-09: the document
opens (**not fatal**), and nothing has to be answered before it does
(**intervention is not required**). The third — **intervention is
possible** — had no route at all: `crate::panels::docprops` printed
*"pdfcer kept /UseOutlines and left /UseOC"* and there was nowhere to
say *use the other one*. A disclosure the operator cannot act on is the
difference between being told and being asked, and the engine went to
the trouble of carrying **both** values precisely so this could exist.

# Why it is a re-load rather than an edit, in the engine's words

> *"A decision made during parsing is not a value that can be edited
> afterwards, because the discarded one was never built into the
> document. Carrying both would make every dictionary lookup ambiguous
> for the life of the session."*

So this closes the tab and opens the same path again. It is deliberately
the **same mechanism** the password retry above uses, down to the order
of the two statements, because the two are the same act: *these bytes,
read again, under something the operator supplied*.

# ⚠ What the caller owes, and what this does NOT do

**It does not ask about unsaved edits.** This is the mechanic; the two
guards are the caller's, and they live with every other document-
discarding arm in [`crate::app::actions::document`] where the table of
which arms guard is kept. Calling this from a new site without them
would destroy an afternoon's work silently — which is the exact defect
that file exists to prevent, found once already by an audit rather than
by a test.

# Returns

`false` when there was nothing to re-read — no document open, or one
with no file behind it (`file.new`'s blank sheet has no bytes to read
again). Traced either way, so a control that appears to do nothing can
be told from one that was never reached.

### `fn new_document`

The `file.new` half of this module, and the third member of the family
whose other two are [`Self::open_path`] and [`Self::close_document`].

It is a sibling of `open_path` rather than a branch inside it because
the two answer different questions — *load this file* against *make a
document* — and they share the only part that is genuinely common, the
[`Self::adopt`] tail. What differs is one line: where the bytes come
from.

# Where the bytes come from, and why not from the engine

[`crate::app::blank`] carries the whole argument. In one sentence:
`pdfcer-core` has no way to create a document and states in
`pdfcer_core::document`'s module header, under *"THE named invariant:
one `Document`, both directions"*, that it never will (*"No separate
builder/generation model may ever be introduced"*), so New parses a
443-byte template that ships as an asset — which makes it an **open**,
which is the thing this shell already does well.

# Failure is a build defect, not an operator's

The `Err` arm is unreachable in a correct build —
`crate::app::blank::tests` pins that the compiled-in bytes parse and
hold one page — and it still produces [`Status::Failed`] rather than an
`expect`. The state it describes is *"this binary was built with a
corrupt asset"*, and an operator who somehow meets it gets the shell's
ordinary explanatory sentence instead of a process that vanishes.

# What it does NOT do


Read is nevertheless the right mode to offer this in, and not by
tolerance: standing instruction 5 is *"Read may produce a new document;
it may not modify this one"*, and `file.new` is the most literal
instance of that rule there could be.

### `fn new_document_sized`

The other half of `RIBBON_IA.md` §5.1's *New (blank / from template)*
row, and Inkscape's own split: `Ctrl+N` makes a document, this one asks
what kind. Reached from [`crate::dialogs::new_document`], which is where
the size, the orientation and the custom-size validation live.

# It is not "New, then resize"

[`blank::document_sized`] serializes and re-parses, so what arrives here
is an ordinary freshly-parsed document that simply is that size —
nothing pending, nothing undoable. See that function's own header for
why handing over an edited session would have been wrong.

Everything else about it is `file.new`: same name sequence, same
`Untitled` naming, same [`Self::adopt`], and the same rule that it does
not change the mode.

### `fn close_document`

The other half of [`Self::open_path`], and it forgets exactly what
that function forgets — which is the whole of why it exists as a
sibling rather than as `self.status = Status::Empty` at the call site.

# What closing has to forget, and why each thing is here

- **The document itself.** Dropping the [`Status::Open`] box drops the
  `Arc<EditSession>`, the page vector, the cached texture, the
  decomposition and the font inventory, the selection, and the render
  worker — every one of which lives *inside* `OpenDoc` precisely so
  that this is a single move rather than a checklist. `OpenDoc::new`'s
  own docs make the argument from the other direction: state that dies
  with the document belongs on the document.
- **The panels' view state**, through [`crate::panels::PanelsState::forget_document`].
  Expansion sets and the Properties focus are paint-order indices —
  positions on one page of one revision — and they hang off the
  *application*, so they genuinely do outlive a document. Leaving them
  behind means the Objects panel keeps rows expanded for a file that is
  no longer open, which is the same staleness `open_path` forgets for
  the same reason.
- **The search results**, through
  [`crate::find::FindState::forget_document`], for a stronger version
  of that argument: a hit's page index and its page-space rectangle are
  positions in one file, and the epoch test that catches an *edit*
  cannot catch a *different document* — a freshly opened one's
  `edit_epoch` is 0, so stale hits would read as current. The query and
  the search options survive, because those describe the operator
  rather than the document.
- **The de-duplicated trace slots**, so the next document opened in
  this session gets its own canvas line and its own region
  declarations rather than inheriting these because the numbers
  happened to match.

# What it deliberately does NOT forget

The **recent list**. Closing a document is not disowning it; it is the
single most likely moment for an operator to reach for the one they
had before it.

The **dock arrangement** and the **mode**. Those belong to the
operator and outlive every document, which is what
[`crate::app::persistence`] exists to make true across restarts, let
alone across a close.

### `fn resume_after_unsaved`

Called from [`Self::ui`]'s frame, immediately after the dialogs draw.
`crate::dialogs::unsaved`'s header carries the defect this closes; this
function is the half that acts.

# Why the resume calls the lifecycle functions directly rather than
re-raising the `Action`

Re-raising would be the tidier-looking answer and it does not work:
`Action::Close` and its three siblings now consult
`DialogsState::ask_unsaved` at the top of their arms, so a re-raised
action would be asked the same question again and the operator would be
in a loop they can only leave by pressing Cancel. Adding a
*"but not this time"* flag to the action would put a second, invisible
meaning on a value the funnel's whole discipline says is plain data.

So this calls `close_document`, `open_path`, `new_document` and
`new_document_sized` — **the same four functions those arms call**, one
line below their guards. There is no second implementation of any of
them; what is skipped is exactly the guard that has just been answered.

# The save branch, and why a cancelled picker cancels everything

[`crate::app::save::save_copy`] answers `false` for a cancelled picker,
an unavailable picker and a failed write, and this proceeds on `true`
alone. Its own docs argue why none of the three is safe to proceed on;
the operator-facing form is that **pressing Cancel in a file dialog must
never be a way to destroy a document.**

A cancelled save leaves the window closed and the document open, which
is the state the operator is in the middle of anyway — they pressed
Close, then thought better of naming a file. Re-asking would be the
application insisting on finishing a transaction the operator abandoned.

### `fn write_in_place`

The body of the `Action::Save` arm, lifted out of
`crate::app::actions::apply` on 2026-08-28 so that the signature
warning's answer can resume **the same save** rather than re-raise the
action. `resume_after_unsaved`'s own header carries the argument in its
general form: re-raising would meet the guard again and put the
operator in a loop they could only leave by pressing Cancel, and a
*"but not this time"* flag on the action would put a second, invisible
meaning on a value the funnel's whole discipline says is plain data.

So there is one implementation with two callers, and what the second
caller skips is exactly the guard it has just answered.

# The defect the move surfaced: `Action::Save` never returned

Worth recording where the body now lives, because nothing about it is
visible from the arm any more.

`crate::app::actions::apply` matches a handful of actions **before** the
guard that narrows `self.status` to an open document, and every one of
those arms `return`s — because the `match` further down lists all of
them together under
`unreachable!("handled before the document guard")`. `Action::Save` was
added between two arms that both return (`SaveCopy` and `Find`) and
**did not return**, so it fell through the guard and into that
`unreachable!`.

The consequence: **every in-place save of a document that had a file
behind it panicked** — which is the path `Ctrl+S` takes for every
document opened from disk, since `crate::app::dispatch` routes
`file.save` to `Action::Save` exactly when
`crate::app::save::has_a_file` is true and to `Action::SaveCopy`
otherwise.

It is fixed as part of this change because the guard above it needed
the same `return` and it would have been dishonest to add one and leave
the other. The class is worth naming: a fall-through arm in a `match`
whose *later* twin asserts unreachability is a defect the compiler
cannot see, because both halves type-check and the panic is reached
only at run time on one input.

# The `bool` is READ here, where [`Self::write_copy_somewhere`]'s is
discarded

And the difference is the whole point: an in-place save that succeeded
means the file on disk now holds this revision, and `OpenDoc::saved_epoch`
is the only record of that. A failed save must not move it — the disk
still holds the older bytes, and claiming otherwise would let
`dialogs::ocr` read a file that does not have the operator's work in
it.

# Why the no-document case traces rather than dropping silently

A keymap reaches any command from any state, and an operator who
presses the save chord over an empty shell must not be
indistinguishable, in the trace, from one whose keystroke never
arrived. That is the same argument the arm this came from makes for
sitting above the document guard in the first place.

### `fn write_copy_somewhere`

The body of the `Action::SaveCopy` arm, lifted out for
[`Self::write_in_place`]'s reason and at the same time.

The `bool` is DISCARDED here, deliberately, and that is not the same
as ignoring it. `crate::app::save::save_copy` answers *"did a file get
written"* for exactly one caller — `crate::dialogs::unsaved`, which must
not destroy a document on the strength of a save that did not happen.
A plain `file.save_copy` has nothing waiting on the answer: it
succeeded or it reported its own failure, and either way the next thing
that happens is the operator's choice rather than this function's.

### `fn save_as_somewhere`

# The rebinding is the command, and it is four statements

`save::save_as` writes the bytes; everything that makes this a *move*
rather than a copy is below, in one place, on purpose. A document whose
path moved while something else did not is a document whose next
`Ctrl+S` writes a file the operator is not looking at, so this is a
list to be read as a whole rather than four changes scattered across the
frame:

1. **`doc.path`** — the binding itself. The window title and the tab
   label are both recomputed from it every frame
   (`app::frame` and `app::doctabs`), so neither needs telling.
2. **`doc.saved_epoch = doc.edit_epoch`** — the new file contains every
   edit, so the document is clean. Without this the tab keeps its unsaved
   marker over a file that is on disk and complete, and the unsaved-close
   guard would ask about a document with nothing outstanding.
3. **the recent list** — he will look for the new name there, not the
   old one, and `Self::open_path` joins the list this same way.
4. **a receipt**, because the rebinding is otherwise **invisible until
   the next save**, and by then the surprise has already happened.

What is deliberately NOT here: any form of close or reopen. The
session, its undo history and the operator's selection all continue —
see [`crate::app::save::save_as`]'s own on why a round trip would be
an unannounced data loss.

And nothing is written to `viewer::remembered` for the new path. The
per-document page display belongs to a document the operator has
*looked at*; inventing an entry for a file that has existed for a
millisecond would put a record in that store for every Save As, and the
standing preference already answers for a file with no entry.

### `fn resume_after_signature`

Called from [`Self::ui`]'s frame, immediately after the dialogs draw
and immediately after [`Self::resume_after_unsaved`], for the reason
that one is called there: it is not a command but a **frame-level
observation** that a window the operator was looking at has been
answered, and the act it authorises — a write over their own file —
belongs to the application rather than to a dialog.

# Why it runs AFTER the unsaved drain and not before

The two questions cannot be live at once — `crate::dialogs::signature`'s
§7 records why the unsaved window's *Save a copy…* button deliberately
does not raise this one — so the order cannot matter today. It is
nonetheless fixed, in the direction that stays correct if that ever
changes: the unsaved drain may **close or replace the open document**,
and a save resumed after that would write the document the operator is
now looking at rather than the one they were asked about. Running this
second means a stale save can never outlive its subject; running it
first would mean it could.

# There is no cancel branch, and there does not need to be one

`SignatureDialog::take_confirmation` answers `Some` only when the
proceed button was pressed. A cancel, and the window's ✕, close the
window and answer nothing — so this function simply does not run, and
**no file is written**. That is the shape `crate::dialogs::unsaved`
uses for the same reason: the destructive act does not happen until a
button is pressed, which is a property of the control flow rather than
of the window.

### `fn save_pending`

# The rule, stated where it will be needed

A document is written by appending an incremental update to a file the
operator names. While that is happening, the bytes on disk are a
partial revision and the `EditSession` the writer is reading from must
not be dropped or replaced. So:

> **An Open, a New or a Close must not proceed while a save is
> pending.** The operator is asked what to do about it — wait, or
> discard — and the action is applied afterwards or not at all. It is
> never applied underneath the save.


# Why it answers `false`, and why that is not a stub


The predicate asks *"is a save **in flight**"*: is there a moment at
which the bytes on disk are a partial revision and the `EditSession` the
writer is reading from must not be dropped or replaced.
[`crate::app::save::save_copy`] is **synchronous** — it is entered and
finished inside one [`crate::app::PdfcerApp::apply`] call, and no frame is
drawn while it is part-way through — so there is still no state in which
this could be true, and `PROJECT_PLAN.md`'s no-placeholders invariant is
explicit that the answer to that is **nothing**: not a confirmation
dialog wired to a condition that cannot occur, and not an
`unimplemented!()` waiting for an operator to find it.

It is also **not** *"are there unsaved edits?"*, and conflating the two
would be the expensive mistake here. A successful save-a-copy leaves the
document exactly as unsaved as it was, at its own path: the copy went
somewhere else. See [`crate::app::save`] §3, which carries the whole
argument and the live consumer that would break —
`dialogs::ocr`'s `UnsavedEdits` refusal reads `edit_epoch != 0`.

What is still absent is an **asynchronous** save. `file.save` is in
`crate::shell::manifest::PLANNED`, blocked on autosave and crash
recovery, and it is the one that will make this predicate live.

What this is instead is the **seam**: one predicate, consulted by
[`crate::app::actions::Action::Open`], `Action::New` and
[`crate::app::actions::Action::Close`], carrying the rule in its own
docs. When the save lands, it reads that subsystem's state and the three
arms grow their confirmation — in one place, already wired, rather
than in three arms somebody has to remember to find. `file.close`'s own
tooltip already promises the operator this behaviour
("You are asked what to do about unsaved edits first"), which is the
other reason the rule is written down here rather than left implicit:
the promise exists on an operator-visible surface today.
