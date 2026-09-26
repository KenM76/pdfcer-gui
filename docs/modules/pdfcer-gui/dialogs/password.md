# `dialogs::password` — the box that lets an encrypted document be opened

## The defect this closes, and it is the shape this project keeps finding


**An encrypted PDF could not be opened at all.** The shell detected the case
perfectly: `Document::load` returns `DocError::PasswordRequired`,
[`crate::app::lifecycle::PdfcerApp::open_path`] branches on it by structured
error data rather than by message, and produces
[`crate::app::state::Status::NeedsPassword`] — a tab whose tooltip reads, in
this build, *"This document is encrypted and pdfcer has not been given the
password."*

And then **nothing could give it one.** `Document::load_with_password` and
`from_bytes_with_password` were named in exactly one place in this crate: a
doc comment in `crate::app::blank` listing the four loading entry points.
Nothing called either.

**That doc comment is why the coverage tool now strips comment-only lines
before it searches.** Its first run reported `load_with_password` as
*reached*, on the strength of that one sentence — which would have recorded
the single most important missing capability in the whole area as already
built, in the instrument written to find exactly this.

⇒ The general form, and it is a third instance: **a mention is not a call, a
backlog row is not evidence, and a sentence about a limit is a dated
citation.** Every one of the three is a case of prose being mistaken for
mechanism.

## Why a real OS window

Every dialog in this shell has been one since 2026-08-21, and this one earns
it twice over: it appears in answer to an **Open**, which is a gesture an
operator makes and then looks away from, and a modal question hidden behind
the application window with no taskbar entry is the classic *"the program has
frozen"* report. [`crate::dialogs::host`] gives it the entry.

## What is deliberately NOT here

- **No "remember this password".** It would have to be stored, and the only
  places to store it are a settings file in plain text or an OS keychain this
  project has no binding to. A checkbox that wrote a document password into
  `settings.txt` would be a security defect authored on purpose.
- **No attempt limit and no delay.** pdfcer is reading a local file the
  operator already has. Rate-limiting their guesses at their own document is
  theatre that costs them time and stops nobody.
- **No "show password" eye.** It is one line of code and it is a real
  shoulder-surfing surface in an office; the value of it here is low because
  the field is short-lived and the failure is cheap to retry.

## The password never reaches a log

It travels in [`crate::secret::Secret`], whose entire purpose is a `Debug`
that cannot print the value. See that module: this crate traces liberally to
stderr under `PDFCER_DIAG`, and `tools/ui-verify` **captures that stderr to a
file it keeps as evidence** — so one `format!("{action:?}")` on the action
queue would write the operator's password into `target/ui-verify/`, in plain
text, in a directory whose purpose is to be kept and read.

The trace lines below say the length and the outcome and never the value.

## Item notes

### `fn submit`

The empty case is refused **here** rather than sent on, and the reason
is [`Secret::is_empty`]'s: `Document::load(path)` already tried the empty
password before this prompt existed — every conforming reader does that
silently — so submitting it again asks the engine a question it has
answered and returns an identical rejection, which the operator reads as
*"my password was wrong"* about a password they never supplied.

### `fn pressing_open_with_nothing_typed_raises_no_action`

The whole of [`PasswordDialog::submit`]'s argument, asserted: pdfcer has
already tried the empty password, so sending it again would produce a
rejection the operator reads as *"my password was wrong"* about a
password they never typed.

### `fn the_action_that_carries_a_password_never_formats_it`

`crate::secret` proves the type is safe; this proves the type is the one
actually used on this path. A variant that took a bare `String` would
pass every test in that module and write the password into the evidence
file on the first `{:?}`.

### `fn a_rejection_clears_the_field_and_counts_the_attempt`

A wrong password left in the box is one an operator re-submits by
reflex; and without the count, a second rejection is indistinguishable
from a press that did not register.

### `enum Rejection`

Two variants, not one, because `pdfcer-core` reports two errors and its own
doc comment says why: `PasswordRequiresNormalisation` exists *"so that
failure does not masquerade as `PasswordRequired`'s 'you typed it wrong',
which would send the operator to re-check a password that was correct."*
Collapsing them here would undo that on the last step.

### `struct PasswordDialog`

`password` is a plain `String` here rather than a [`Secret`] because that is
what `egui::TextEdit` binds to; it becomes a `Secret` at the moment it leaves
this struct, which is the boundary that matters — the value never enters an
`Action`, a queue or a trace unwrapped.

### `fn reject`

Called by the application when its retry comes back refused. Clears the
field, because a wrong password left in the box is one an operator
re-submits by reflex, and increments the attempt count so the message can
say which try this was — without that, a second rejection produces a
dialog identical to the first and the operator cannot tell whether their
press registered.

### `fn show`

Returns `false` when the dialog should close — cancelled, or dismissed by
the window's own ✕.

The ✕ is a **Cancel**. The window's close control must mean the
non-destructive answer, which is the rule `dialogs::unsaved` states: it is
the control an operator presses reflexively to make a surprise go away.
Here nothing is destroyed either way, and the tab stays in the document
list saying why it did not open.
