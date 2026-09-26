# `pdfcer-gui/dialogs/redact/destination`

## Item notes

### `enum Destination`

> *"why does it have to save to a new file right away? Why can't it just
> wait on saving until I choose to save over the existing file or save as a
> new file?"*


[`RedactDialog::commit`] read, verbatim: *"There is no 'save over the
original' branch to find, because there is none to write, and on this
operation that is the difference between a copy and the destruction of the
only remaining source of the content being removed."*

The premise is true and the conclusion did not follow. Overwriting the
source **is** the destruction of the only remaining copy — but the person
entitled to decide that is the person who marked the content for
destruction in the first place, and forcing a copy does not protect him from
the decision, it only makes him perform it in two steps with a stray file
left over. Every other edit in this shell trusts him with Save and Save As
on exactly this reasoning; the redaction had quietly taken the decision away
on his behalf.

What the old ruling was *actually* protecting is kept, and kept in the
form it belongs in: [`Self::NewFile`] is still the **default**, and
`crate::dialogs::redact::suggested_path` still never suggests the source. A
safe default is a mechanism; a warning is something to click past. The
change is that the safe default is now a default rather than the only
option.


What stood here, verbatim, written at about midday:

> *"⚠ What this deliberately does NOT do, and why. He asked for the write to
> be deferred — applied into the session, saved later by Save or Save As
> like any other edit. **The engine cannot express that**, and this dialog
> does not fake it. [`pdfcer_core::redact::apply_redactions`] takes a
> `&Document` and returns `Vec<u8>`; `EditSession`'s only constructor is
> `new(Document)` and it has no `replace_document`, no `rebase` and no
> `reload`."*

Every clause of that was true when it was written and was filed as an engine
request the same morning. **The engine answered it that afternoon**:
`EditSession::apply_redactions` (`Pass 250.1`, `225db51`) applies the
removal into the session and leaves the write to the ordinary save verbs. So
the paragraph is not softened, it is **replaced** — [`Self::OpenDocument`]
is the destination it said was impossible, and it is now the default.

What the old paragraph got right and is worth keeping: the manoeuvre it
refused — *"building a second `EditSession` and swapping it under the open
document"* — is still refused, and the engine did not ship that either. Its
verb collapses the session in place, keeps the document identity, and clears
the undo log **by name** rather than by accident, which is the difference
between a disclosed consequence and a silent data loss. The refusal was
right; only its conclusion about what could exist was wrong.

The request is at `D:\Dev\FeatureRequests\pdfce_FeatureRequests\
open\request_apply_redactions_into_the_session.md` and the reply beside it.

### `const DEFAULT_DESTINATION`

A named constant rather than a literal inside [`RedactDialog::open`], so the
property that actually matters — *the default writes nothing* — can be
asserted without constructing a document, and so that changing it is a
visible edit rather than one word in a struct literal.

### `fn writes_now`

A method rather than three `== ` comparisons scattered through
[`RedactDialog`], because five separate places ask the same question —
which permanence sentence, which button label, which acknowledgements
are owed, whether the picker opens, and whether an `Action` is pushed —
and a fourth destination added later must be answered once rather than
found five times.

### `fn stages`

Added 2026-09-08 with [`Self::OpenDocumentNow`], and it is the
reason that variant needed more than a radio row. The confirm handler
branched on `!writes_now()` — *"anything that does not write a file is
staged"* — which was true while `OpenDocument` was the only
non-writing destination and became **silently wrong** the moment a
second one existed: `OpenDocumentNow` writes no file either, so it would
have taken the staging path and done exactly the nothing the operator
reported.

⇒ Named for what it MEANS rather than derived from what it is not. A
predicate written as the complement of another is a predicate that
changes meaning when a variant is added, without a compile error and
without a word.
