# `redact` — the APPLY pipeline, and the reason a redaction in this shell
cannot be shipped unverified

**This module is the ONLY place the proof of removal exists.** The absence
proof lives in [`proof`]; the call-site monopoly that makes it unskippable
lives in `redact::sealed`.

The engine supplies no verdict. [`pdfcer_core::redact::apply_redactions`]
returns `Result<(Vec<u8>, RedactionReport), RedactError>` — a **report**, not
a verdict — and neither `RedactionVerdict` nor `verify_redaction` exists
anywhere in `pdfcer-core`. ⇒ **A shell that calls
`redact::apply_redactions` directly and writes the bytes ships an unverified
redaction and will not know.** This module is therefore not a convenience;
it is the difference between a redaction that is proven and one that is
asserted.

---

# 1. The three properties carried across from the source

## ★★★ 1.0 There are THREE routes, and the third one STAGES

There is deliberately no route that applies the removal into the open
session by collapsing it onto a clean redacted base: that clears the undo
log, and two apply routes with different undo semantics, on one dialog, on
the one operation that cannot be undone, is a choice an operator would have
to understand in order to make it safely. So the three routes differ in
*where the bytes go*, never in *what undo means*:

| route | what it produces | what it touches | who calls it |
|---|---|---|---|
| [`prepare_redaction_apply`] | the finished redacted **bytes**, in memory, proven | nothing | the dialog on open — the MEASUREMENT — and the two write-now destinations |
| [`stage_into_session`] | a **staged** removal: a flag, and nothing else | the session's pending-redaction flag; **not** its base, overlay, or undo/redo stack | the deferred destination, through `crate::app::actions::redact` |
| [`save_applying_pending`] | the finished redacted **bytes**, proven, at save time | nothing — it takes `&self` | `crate::app::save::write_copy`, on every save verb, while a redaction is staged |

The measurement still has to exist *before* the confirmation on every path
(§2 of [`crate::dialogs::redact`]), so the dialog still runs
[`prepare_redaction_apply`] on open and the numbers on screen at the moment
of consent are measurements rather than predictions. The confirm click
commits nothing: it arms a save.

### ★★★ 1.0.1 The proof MOVED, because the bytes moved

`apply_redactions_deferred` runs the removal to produce its preview report
and **discards the bytes**. There is therefore nothing for [`proof`] to
sweep at staging time, and this module does not pretend otherwise: nothing
in [`stage_into_session`] says *"verified"*, and
[`crate::text::redact::staged_into_document`] does not either. The word is
earned at the save, by [`save_applying_pending`], over the exact buffer that
is one statement from the file system — which is §2.2's own rule arriving at
the only place the deferred route can still keep it.

### ★★★ 1.0.2 The §4.1 guard is REAL now, and it is a refusal

A staged redaction leaves the un-redacted content **live in the session** —
that is the whole point of preserving undo — so every ordinary write out of
that session would leak it. The engine refuses each one by name:

* `EditSession::to_incremental_bytes` returns `WriteError::RedactionPending`;
* `EditSession::to_full_bytes` returns the same;
* the removal happens only through `EditSession::save_applying_redaction`,
  which takes `&self`, so undo survives the save.

⇒ **The leak surface is larger and the guard is stronger, and neither was
taken on trust.** `tests` measures both directions on a synthetic fixture
and on `fixtures/a1-titleblock.pdf`: the two ordinary save modes refuse **by
name**, and the bytes `save_applying_redaction` produces carry no `/Prev`
and none of the removed text, with a positive control on each.

### ★ 1.0.3 What a staged redaction does NOT do, and it is the one
surprising thing about it

**It does not change the page.** The session is untouched, so the content is
still drawn, the `/Redact` marks are still drawn, and a screenshot taken one
frame after the operator presses the confirm control is identical to one
taken a frame before it. That is rule 4 satisfied rather than violated —
this shell adds no badge, tint or provisional layer to say *"awaiting
removal"* — and it is a fact the operator has to be told in words, because
every redaction tool he has ever used changed the picture. The saying is
[`crate::text::redact::staged_into_document`]'s and
[`crate::text::redact::saved_applying_redaction`]'s.

## 1.1 Apply is a FULL REWRITE, or it does not happen

The engine's `ARCHITECTURE.md` §5 corollary and standing rule R35: an
incremental save **structurally preserves superseded content** — the old
bytes of every replaced object stay in the file by construction, in the
prior revision. For an ordinary edit that is a feature, and it is exactly
what [`crate::app::save`] relies on and promises in `file.save_copy`'s
tooltip. For a redaction it is the defeat of the entire operation: the
"removed" text would sit in the saved file one `startxref` hop away,
trivially recoverable by any parser that walks `/Prev`.

So there are exactly two full rewrites in this pipeline and no third path:

```text
  EditSession (marks may be UNSAVED)
    │
    │  (1) EditSession::to_full_bytes   ← full rewrite #1: materialise
    ▼                                      this session's edits as ONE
  Vec<u8>  (one revision, no /Prev)         revision so apply can see the
    │                                       marks the operator just made
    │  Document::from_bytes
    ▼
  Document
    │
    │  (2) redact::apply_redactions     ← full rewrite #2: core's own
    ▼                                      forced full rewrite, which is
  Vec<u8>  (redacted, one revision)         where the removal happens
```

If **either** rewrite fails, this module returns a refusal and nothing is
written. There is deliberately no `to_incremental_bytes` call anywhere in
this file, and no fallback that could introduce one: a redaction that
silently degraded to an incremental save would produce a file the operator
has been told is redacted and which is not.

★ That is worth restating in this shell's terms, because this shell has an
incremental writer and the old one's *"there is no parameter anywhere that
could make an apply write incrementally"* has to stay true here.
[`crate::app::save::save_copy`] is incremental **by a promise printed on a
tooltip**; this path is full-rewrite **by the engine's own construction**,
and the two share no function, no options value and no code path. They are
two writers, deliberately, and neither can inherit the other's default.

## 1.2 Why the session must be materialised first (the un-saved-mark trap)

[`pdfcer_core::redact::apply_redactions`] takes a `&Document` — a parsed file
— not an [`EditSession`]. The shell's marks, however, may exist only in the
session overlay: an operator can open a document, mark three regions and
press Apply without ever having saved. Handing `session.document()` (the
BASE revision) to `apply_redactions` would therefore apply **zero** of those
marks and report success — not a disclosure that stayed silent, but an
*apply* that removed nothing while saying it had.

`to_full_bytes` is what closes it: it is the session's own edits rendered
into a real single-revision file, which `Document::from_bytes` then
re-parses into exactly the document the operator is looking at.

## 1.3 Absence is VERIFIED on the actual output bytes, not assumed

See [`proof`], which owns the whole of that argument and the table of what
each class of survivor means.

---

# 2. ★ How the proof is made **unskippable**, rather than merely available

The salvage brief's second requirement, and the one that is not satisfied by
copying the file across. The old shell's own module docs end with *"Nothing
in this module can reach the filesystem"* — true, and it means the proof was
enforced by the **caller** remembering to run it. `pdfcer`'s
`redact-apply` is the counter-example living in the same repository: it
calls `apply_redactions`, writes the bytes and exits `SUCCESS` on a file it
never verified.

Four mechanisms, in increasing order of how hard they are to defeat. Each
one alone would be a convention; together they are a structure.

## 2.1 The bytes are private, and there is no accessor

[`PreparedRedaction::bytes`] is a private field of a type whose only
constructor is [`prepare_redaction_apply`], which always proves. There is no
`pub fn bytes()`, no `Deref`, no `AsRef<[u8]>`, no `IntoIterator`, and
[`PreparedRedaction`]'s [`std::fmt::Debug`] impl is **hand-written** so that
`{:?}` reports a length rather than emitting the buffer into a log. The only
expression in this crate that can obtain the redacted bytes is inside this
module. That is a compile-time fact, not a convention:
`PreparedRedaction { bytes: … }` does not typecheck outside `redact`, and
nor does `prepared.bytes`.

## 2.2 The write is a method on the proof, and it re-proves

[`PreparedRedaction::write_to`] is the only way bytes leave this module, and
it runs the decoded-stream half of the proof **again**, over the exact
buffer it is one statement away from handing to the file system.

That is not belt-and-braces about a check that already passed. It is what
moves the guarantee from *"the constructor proved it"* to *"the write
proves it"* — a distinction that matters the day someone adds a second
constructor, a `set_bytes`, a `#[cfg(test)]` builder or a deserialisation
path. Any of those would defeat 2.1 silently; none of them defeats this,
because the check is between the buffer and the syscall rather than at the
far end of the type's history.

## 2.3 The acknowledgement is a parameter, not a convention

A disclosed residual (module [`proof`]'s middle row) requires the operator's
explicit acknowledgement, and the writer cannot be reached without stating
whether it was given: [`ResidualAcknowledgement`] is a required argument, and
[`WriteRefusal::ResidualsNotAcknowledged`] is what a `Withheld` produces when
there is something to acknowledge. A caller that forgets the checkbox does
not write a partially-redacted file believing it is clean; it gets a named
refusal.

It is an enum rather than a `bool` for [`crate::app::actions`]' stated
reason: `write_to(path, true)` at a call site says nothing, and this is the
one call site in the program where a transposed boolean is a security
defect.

## 2.4 The call-site monopoly is asserted from the syntax tree

`redact::sealed` parses **every `.rs` file in this crate** with `syn` and asserts
that each of the engine's removal verbs is *called* in exactly one FILE —
this one — and exactly the number of times this module accounts for. A call
from anywhere else is a test failure naming the file.

★★★ The monopoly is *one file*, not *one call*, and it has **four
subjects**. The removal is split across a verb that stages it, a verb that
performs it at save time, and a verb that un-stages it, so a monopoly pinned
to one identifier would watch three quarters of the feature walk out of the
module:

| subject | calls | where |
|---|---|---|
| `apply_redactions` (the free function) | 1 | [`prepare_redaction_apply`] |
| `apply_redactions_deferred` | 1 | [`stage_into_session`] |
| `save_applying_redaction` | 1 | [`save_applying_pending`] |
| `cancel_pending_redaction` | 1 | [`cancel_staged_redaction`] |

★ The counts are exact and not ceilings, because a ceiling lets a *removed*
call site pass unremarked. An exact count makes any movement in either
direction an edit somebody has to write down.

★ `cancel_pending_redaction` is in the table even though it removes nothing
— it *disarms* a removal, which is the same surface seen from behind, and a
second caller that un-staged a redaction the operator had confirmed would be
the quietest possible way to ship a file he believes is redacted. The reader
counts method calls as well as free calls, because an engine that moves a
removal verb onto a type would otherwise slip the monopoly silently.

It reads the abstract syntax tree rather than the text, for
`crate::shell::commands::reach`'s reasons applied to a different question: a
doc comment quoting `[`pdfcer_core::redact::apply_redactions`]` is not a call
and a grep cannot tell (this very header contains several). And it **fails
closed** — a sweep that finds *zero* call sites fails, because "the proof is
nowhere" and "the sweep read nothing" print the same thing otherwise, which
is `run-all.sh`'s three-state lesson arriving inside a test.

## 2.5 What is deliberately NOT claimed

None of this stops a future author writing `pdfcer_core::redact` calls in a
*different crate*, and none of it stops a determined edit to this module. It
is not a sandbox. What it does is make the unverified path **impossible to
reach by accident and impossible to add quietly** — which is the failure
mode that matters: a shell that calls the engine directly *and will not
know*.

---

# 3. What this module deliberately does NOT do

It does not implement any part of the removal. The surgery, the carrier
sweep, the object-stream decomposition and the forced full rewrite all live
in [`pdfcer_core::redact`] and are called, never re-derived — the GUI/core
separation rule, plus the plain fact that a second implementation of
security-critical byte surgery is how the two quietly diverge.

It does not decide **where** the file goes. [`PreparedRedaction::write_to`]
takes a path; asking the operator for one is [`crate::dialogs::redact`]'s
job. See §4.

★★★ **It does not mutate the open document, and that is an observation, not
a principle.** [`stage_into_session`] sets one flag; base, overlay, undo and
redo are left exactly as they were. The distinction matters because the
sentence reads like a safety rule and is not one: it describes the engine
surface this module happens to call, so it is re-checked against
`D:\Dev\pdfcer` rather than relied on. The load-bearing rule — what actually
protects the operator from an irreversible write — is §4.

---

# 4. ★★★ The rule is *warn at the overwrite*, not *never overwrite*

The source file **is** the only remaining copy of the content being removed,
and it does not follow from that that the shell must refuse to overwrite it.
Forcing a copy does not protect the operator from the decision; it makes him
perform it in two steps and leaves a stray file behind, on every save of a
document he is still working in. The choice of destination is his
(`OPERATOR_REQUESTS.md` O125). What the shell owes him is a *mechanism*
rather than a prohibition:

1. **Nothing in this module can produce a path.** [`PreparedRedaction::write_to`]
   takes one; it never invents one. That is unchanged and is the structural
   half of the rule.
2. **The suggested name is never the file that was opened**
   ([`crate::text::redact::suggested_suffix`], asserted by
   [`crate::dialogs::redact`]'s own test, in the shape
   `crate::app::save::suggested_path` and `crate::dialogs::ocr::suggested_path`
   both established).
3. **The write is atomic** — temp file, then rename — precisely because the
   destination may now be the source. See [`PreparedRedaction::write_to`].
4. **The overwrite is warned about at the moment it is chosen**, in words,
   at a control the operator had to select. A warning, not a refusal: that
   distinction is the whole of O125.
