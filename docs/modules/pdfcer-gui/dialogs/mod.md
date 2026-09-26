# `dialogs` — the shell's stationary, screen-anchored surfaces

## What belongs here, and what does not

A **dialog** is a single transaction with a start and an end: it is opened
deliberately, it holds one job's worth of answers, and closing it forgets
them. A **panel** is somewhere an operator dips in and out of while
working, and it keeps its state across documents. The distinction decides
where a surface lives, and getting it wrong is not cosmetic — a print
configuration that persisted across documents would let a range typed for
one file silently apply to another.

## Every dialog here is screen-anchored, never page-anchored

Made in response to a specific operator objection: **controls whose position
is derived from the page move on every zoom and scroll.** A surface an operator is reading and
typing into must stay where they put their eyes. Each dialog therefore
anchors to the viewport rather than being positioned relative to the
canvas, and none of them is drawn inside the canvas's coordinate space.

## Where dialog state lives, and why it is one field

[`DialogsState`] is the whole dock-side surface of this module: one field
on `PdfcerApp`, one `open_*` call per dialog from the command dispatcher,
and one [`DialogsState::show`] call per frame. It follows
`crate::panels::PanelsState` exactly — same idiom, second instance, not a
new convention — and the reason it is a struct rather than a bare
`Option<PrintDialog>` is that the *next* dialog is then a change to this
file rather than to `app/mod.rs`, which is the file every parallel task
already contends over.

## Why a dialog does not push an `Action`

`crate::app::actions`' invariant is that **no code path runs from a widget
to a document**, and the four things it buys are all about *document*
state: a coherent undo log, an aliasing problem turned into a queue,
explicit ordering between changes, and a greppable answer to "what can
change this?".

A print changes no document state. It reads the document — the pages, the
edited view — and writes to a spooler, so it contributes nothing to the
undo log and has nothing to order against. Routing it through the funnel
would add an `Action` variant that `apply` could only answer by reaching
back into a dialog for the state it needs, which is the funnel pointing the
wrong way.

What the funnel's *reason* does still demand is that the irreversible work
not happen part-way through a layout pass, and [`print::PrintDialog`]
honours that in its own scope: the button sets a flag, and the spool runs
after the window's closure returns. See that field's documentation.

**A dialog that edits the document is a different case and must use the
funnel.** The properties dialog and the settings host will both raise
`Action`s; this note is about printing specifically, not about dialogs in
general.

## Item notes

### `fn retire`

`open` is what the dialog's own `show` returned — *"should I still be on
screen?"* — and `answered` is whether it is holding a decision its owner has
not collected yet. A dialog is retired only when **both** say no: it is off
screen *and* it has nothing left to hand over.

# WHY THIS IS NOT `!open`

`!open`, expressed at each call site as
`if …map(|d| d.show(ctx)) == Some(false) { self.slot = None; }`, is exactly
right for eleven of the thirteen dialogs: they act through `actions` while
they draw, so a closed one has nothing left in it.

The two **confirmation** windows are different in kind, and the difference
is the whole defect. `unsaved` and `signature` deliberately do NOT act. They
*park* an answer and let `crate::app::PdfcerApp` perform it —
`resume_after_unsaved` and `resume_after_signature`, both later in the same
frame — because the acts in question (closing a document, writing over the
operator's own file) are the two most destructive things this shell does and
must have exactly one route each. A window that could call `save_in_place`
would be a second route.

So for those two, `show` returning `false` and the dialog being *finished*
are different facts. Pressing the button sets the answer, which is what
makes `show` answer `false` — so a `!open` branch destroys the dialog **and
the answer inside it** before the drain three call frames later can look,
and `take_signature_answer` finds an empty slot and returns `None`.

⇒ The observable result, and what `an_invalidating_save_is_warned_about`
drives the binary to check: the signature warning opens, holds the save,
draws its proceed button, takes the click, **closes** — and traces no
`signature-confirmed` and writes no file. A signed document cannot be saved
at all by any route the guard covers, which is worse than having no guard:
it stops the save and never lets it through.

Neither half is wrong on its own, which is why no unit test can see it. The
dialog returns its answer when asked; the drain performs whatever it is
given; the defect lives entirely in the **lifetime between them**, and a
lifetime is not a value any assertion over either half can name. That is the
same shape `PROJECT_PLAN.md` §4 built the driving harness for.

# The invariant this creates, stated where it can be checked

> **Every caller of [`DialogsState::show`] must drain the parked answers in
> the same frame.**

There is one caller — `crate::app::frame` — and it drains both, immediately
after. A retained-because-answered dialog therefore lives for zero frames:
it is emptied and dropped by `take_*_answer` before anything can draw it
again. A caller that did not drain would see the window redraw for as long
as it ignored it, which is a loud failure rather than a silent one, and that
direction was chosen deliberately over discarding the answer.

### `fn close_document_scoped`

One place, so a document-scoped dialog added later cannot be forgotten
by whichever of the close paths its author did not think of.
Application-scoped dialogs are deliberately absent — see
[`Self::show`].

### `mod compact`

Its header carries the reason it has no settings: the only configuration an
embed has is *which folders*, and that lives in Settings.
The window before a full rewrite. Its header carries the reason it writes
the file BEFORE it opens: when a window asks somebody to trade something
irreversible for a benefit, the benefit must be measured, not predicted.

### `mod export_dxf`

Its header carries the decision a reader will question first: why placement
is numeric rather than a drag, and why a drag is a second **route** to the
same action rather than the one that should have shipped.
The Export-DXF window — the page's vector geometry, at a scale somebody
can defend.

Its header carries the sentence the whole feature turns on, quoted from
`pdfcer-core`: every generic PDF-to-DXF converter exports at paper scale and
says nothing, so a 1:2 detail arrives at half size **looking plausible**.

### `mod stamp_collection`

Its header carries the finding the whole feature turns on: **there is no
interchange format because Acrobat has none.** A stamp collection is an
ordinary PDF, so the operator's "same import/export" was never a converter
— it was a way to author one of those files.

### `mod sign`

`#[cfg]` for `crate::sign`'s reason, and it is the module boundary rather
than the ribbon: `SHELL_FRAMEWORK.md` §5b's rule is satisfied by
`file.sign` simply not being registered, which drops the ribbon item
through the ordinary merge with a `CapabilityAbsent` skip reason.

### `mod signature`

Its header carries the gap it closes, why the engine says the question can
only be asked at save time, the table of which impact earns which surface,
and why the compacted-save path needed nothing from it.

### `mod unsaved`

Its header carries the defect it closes, why `save_pending` was NOT the bug
and must not become the fix, and why the first button says *Save a copy…*
rather than *Save*.

### `mod scale`

Phase 7 shipped three tools that place dimensions and no way to say what
scale they are at, so every label read in PDF points: a measurement of the
**paper** rather than of the thing drawn on it. A plausible answer to a
question nobody asked, which is worse than a missing feature.

### `mod shortcuts`

**Application-scoped and not held in [`DialogsState`]**, which is the one
departure in this directory and is forced rather than chosen. Its draft has
to be readable at the *top* of the frame, before any widget is built,
because the theme is installed there and a draft theme must take effect
immediately — you cannot judge a theme from a radio label. So the draft
lives on `PdfcerApp` as `settings_draft`, and this module is a renderer with
no state of its own.
The keyboard reference, **derived from the keymap that dispatches**.

Application-scoped, beside [`about`]: a keyboard reference is meaningful
with nothing open, and is one of the two things a new operator reaches for
before opening a file.

Its header carries why it holds no list — `DEFECTS.md` D5 is not fixed
there, it is made unrepresentable.

### `struct DialogsState`

One field per dialog, each an `Option` whose `Some` *is* the "open" state —
there is no separate visibility flag that could disagree with whether the
state exists. Closing a dialog drops its state, which is what makes
"closing forgets the job" true by construction rather than by remembering
to reset fields.

## The fields are in two groups, and the split is load-bearing

A **document-scoped** dialog is about the open file: a print job is a job
on *these* pages. An **application-scoped** dialog is about pdfcer itself
and is meaningful with nothing loaded.

A shell whose dialogs were all document-scoped could take the shortcut of
dropping every one of them the moment the document went away.
[`about::AboutDialog`] is why [`DialogsState::show`] cannot: an operator who
has just launched pdfcer and wants to know what version they are running, or
under what terms, has no document — and a control that did nothing in that
state is a placeholder, which this project does not ship.

So the two groups are drawn separately rather than the rule being softened
for everything. Print still closes with its document; About does not, and
cannot be made to without breaking the command that opens it.

### `struct Frame`

# Why this exists

[`DialogsState::show`] took seven loose parameters and, when O166 gave the
Print window a preferences file to write to, an eighth. Clippy's
`too_many_arguments` fired at that point, and the honest reading of that
lint is not *"the limit is seven"* — it is that **a parameter list nobody
has to name at the call site is a list that grows by accident.** Eight
positional arguments at one call site in `app::frame` is a place where
transposing two `Option`s of the same type compiles.

So the arguments are named here, once, with what each is for. Adding a
ninth is still possible — it should be — but it now costs a field with a
doc comment rather than a comma.

# Lifetimes

One, shared. Every borrow here is taken from disjoint fields of the same
`PdfcerApp` for the duration of a single call, and none of them outlives
the frame; distinguishing them would buy nothing and would put four
lifetime parameters on a struct that lives for one statement.

### `fn take_place_request`

Read-and-clear, and it answers the page the asking dialog is placing
on so `canvas::placing` can record it. One arm per kind, listed rather
than wildcarded so a second `PlaceKind` has to be wired rather than
silently never asked.

### `fn deliver_placement`

The window is not reopened, because it was never closed — see
`dialogs::placing`. It simply starts drawing again on the next frame,
with the numbers this writes into it.

### `fn has_requester`

The one guard that closes every exit route nobody enumerated. If the
document is closed under a pending placement the dialog is dropped
(`forget_document`), and without this the canvas would sit in a
placement tool waiting for a window that no longer exists. `app::frame`
checks it once a frame and cancels, which costs one `Option` read.

### `fn show`

Called once per frame from frame composition, **after** the canvas and
the docks: a dialog is an overlay, and egui's `Area` ordering follows
the order things are added within a frame.

# Why a closed document closes the DOCUMENT-SCOPED dialogs

A print job is a job on this file's pages. A dialog left up over a
closed document would be configuring a job against pages that no
longer exist, and the honest response is to close it rather than to
freeze it or to let it act on whatever is opened next.

# …and why About is drawn either way

It is about pdfcer, not about a document. Closing it when the document
closes would make `file.about` — a command every mode offers, with no
`enabled_when` — open a window that vanished on the same frame
whenever the canvas was empty. That is a control that does nothing,
and it would look exactly like a bug in the command dispatch rather
than like a rule about dialog lifetime.

The early return therefore covers only the first group. Both are drawn
first and closed after, rather than closed inside the borrow that drew
them: a dialog decides whether it stays open *while* it draws (the
title-bar cross and its own Close button are both widgets), so the
answer arrives out of the same call that needs `&mut` on the state
being dropped.

### `fn take_signature_answer`

Drained by `crate::app::PdfcerApp` immediately after [`Self::show`], for
the reason [`Self::take_unsaved_answer`] is: the act it authorises — a
write — belongs to the application, not to a dialog. A window that
could call `save_in_place` would be a second route to the one operation
in this shell that can destroy the operator's file.

**It clears the window on the way out.** The answer and the window's
lifetime are one fact, and separating them is how a confirmation gets
answered once and acted on every frame — which here means writing the
operator's file sixty times a second.

### `fn ask_signature`

Returns `true` when the question was raised and the caller must
**stop** — the save is now this window's to authorise. `false` means
there was nothing to ask about and the caller saves as before.

# The return value is "did I interrupt you", exactly as
[`Self::ask_unsaved`]'s is

And for the identical reason, restated because it is the property that
makes the guard safe to add to a third save route later: a guard read
as *"may I proceed"* fails **open** when somebody inverts it or forgets
it, and the unannounced write happens. Read this way it fails
**closed** — a missing `if` raises the question and its answer performs
the save anyway, so the operator sees one redundant window rather than
a signed document silently rewritten.

The already-open guard is the same one, and it matters here for the
same reason it matters there: `Ctrl+S` held down while the question is
on screen would otherwise replace the pending save with a second one,
and an operator who asked for a copy would get an in-place write.

### `fn take_unsaved_answer`

Drained by `crate::app::PdfcerApp` immediately after [`Self::show`], for
the reason every hand-over in this module has one: the act it authorises
— closing, opening, replacing — belongs to the application, not to a
dialog, and a window that could call `close_document` would be a second
route to the most destructive operation this shell has.

**It clears the window on the way out.** The answer and the window's
lifetime are one fact, and separating them is how a confirmation gets
asked twice — or, worse, answered once and acted on every frame.

### `fn ask_open_in_acrobat`

Returns `true` when the question was raised and the caller must
**stop**: the handover is now this window's to authorise.

# The return value is "did I interrupt you", exactly as
[`Self::ask_unsaved`]'s and [`Self::ask_signature`]'s are

And for the identical reason, which is worth restating because this is
the third guard to take the shape: a guard read as *"may I proceed"*
fails **open** when somebody inverts it or forgets it, and here that
means a document closed and handed to another program with nothing
asked. Read this way it fails **closed** — a missing `if` raises the
question, and the operator sees one redundant window rather than losing
a document off their screen without warning.

The already-asking guard is the same one its two siblings carry: a
second press while the question is on screen is impatience, not a
second request, and stacking it would leave a window nobody can dismiss.

### `fn take_open_in_acrobat_answer`

Drained by `crate::app::PdfcerApp` immediately after [`Self::show`], for
the reason every hand-over in this module has one: the acts it
authorises — a save, a close and a process launch — belong to the
application. A window that could call `close_document` would be a
second route to the most destructive operation this shell has, and one
that could spawn a process would be the only place in the crate that
did.

**It clears the window on the way out**, so a confirmation cannot be
answered once and acted on every frame — which here would mean starting
Acrobat sixty times a second.

### `fn take_open_in_acrobat_cancelled`

Draining rather than peeking, so one cancel produces one trace line.
A flag that stayed set would have the application reporting a
cancellation on every frame until the next question was asked.

### `fn ask_for_password`

Called from the frame when the active document is
[`crate::app::state::Status::NeedsPassword`]. Idempotent on the path, so
the frame can call it unconditionally every frame — which is what makes
it safe to drive from a state rather than from an event, and is why the
prompt survives the operator clicking away and coming back.

Re-asking for a **different** document replaces the prompt. One
password box at a time is the convention everywhere, and two would leave
the operator guessing which file each belonged to — the prompt names its
file for the same reason.

### `fn reject_password`

Called when a retry comes back refused. Returns `false` when there is no
prompt to tell — which happens if the operator cancelled between
submitting and the load returning, and is not an error.
