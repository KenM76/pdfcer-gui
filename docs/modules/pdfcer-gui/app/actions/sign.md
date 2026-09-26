# `app::actions::sign` — the one arm that signs a document

[`Action::SignDocument`]'s body, kept beside its reasoning rather than
inside [`super::apply`]'s match.

The window is [`crate::dialogs::sign`] and the model is [`crate::sign`];
read the second of those first, because everything about *what a signature
is and which refusals exist* is argued there. This file is about what has to
happen, in order, on the far side of the action queue.

---

# 1. Why this is matched before the document guard — a borrow reason

[`super::apply`] takes `let Status::Open(doc) = &mut self.status` and keeps
that borrow for the rest of the function. This arm needs **two** of
`PdfcerApp`'s fields at once: the open document, to sign it, and
[`crate::dialogs::DialogsState`], to hand the outcome back to the window
that asked. Splitting the borrow has to happen while `self` is still whole.

That is exactly `Action::Find`'s argument, which is matched in the same
early block and says so at its arm. One reason, and it is about Rust rather
than about signing.

# 2. The steps, and the order is load-bearing

`EditSession::sign` takes `&mut self`, so this is a funnel path and it owes
the funnel's protocol — [`super::apply::vector_edit`]'s four steps, which
this arm performs **by hand** rather than through that function. The reason
it cannot use it is stated below; the reason it must still do the same
things is that each step is a separate way to end up with an edit that is
silently declined.

1. **Stop the render worker.** `OpenDoc::session` is an `Arc` precisely so a
   worker can hold a clone while it rasterizes, and
   `RenderWorker::cancel_and_wait` is *"the choke point that makes
   `Arc<EditSession>` sound"*: `Arc::get_mut` fails while any other strong
   reference exists, so a signing attempted mid-render would simply be
   **refused**. Cancelling first is what turns *"sometimes refused,
   depending on how fast the page rasterized"* into *"always applied"*.
2. **Reach the session through `Arc::get_mut`.** A `None` here is not a
   panic: it means something else still holds the session, which is a bug in
   the caller's ordering rather than in the operator's document. It is
   reported as a failure the window prints, because declining is
   recoverable and signing a document twice is not.
3. **Sign, and write.** [`crate::sign::prepare`] then
   [`crate::sign::Prepared::write_to`], which is atomic.
4. **Hand the outcome back.** The window is showing
   `Phase::Signing` until it hears, and that is its only way out.

## Why not `vector_edit`, when every other `&mut` verb uses it

Because two of that funnel's four steps would be **wrong here**, and both
wrongs are silent:

| `vector_edit` step | why not |
|---|---|
| bump `edit_epoch` | the epoch is what makes the canvas re-resolve its selection and rebuild its raster. Signing changes **nothing the canvas draws** — a visible signature's widget goes into the bytes that were written to disk, not into the session, which keeps only the zero-filled placeholder. Bumping it would re-rasterize a CAD sheet to draw an identical picture. |
| drop the cached texture | same fact, same cost. |

And the deeper reason, which is the one worth carrying: **the session
is left holding a placeholder, not a signature.** The engine says so —
*"the session still holds the staged placeholder objects (zeros in
`/Contents`) … a caller that wants to keep editing must re-open the returned
bytes."* So there is no state here for the canvas to catch up with. The
document on screen is, and remains, the version the operator started from,
and [`crate::text::sign::open_document_unchanged`] says so on the window
rather than leaving them to find out at the next `Ctrl+S`.

⚠ **Nothing is undone either.** The staged `CommandKind::AddSignatureField`
stays on the undo stack; the engine calls undoing it *"harmless and
pointless"*. Rewinding it here would look like tidying up and would put an
entry on the operator's undo stack for an act that produced a file.

# 3. The identity is loaded again, here, and that is the design

The dialog has already opened this `.pfx` — that is how the operator saw
whose certificate it is — and it does **not** hand the loaded key over.
[`Action`] derives `Debug`, `Clone` and `PartialEq`, and every one of those
is wrong for a private key: `Debug` writes it into a trace `tools/ui-verify`
keeps on disk, `Clone` makes copies nobody counts, and `PartialEq` is a
**non-constant-time comparison over secret bytes**.

So the action carries the path and a [`crate::secret::Secret`], and this
file opens the container a second time. The second read is not redundant:
it is the read that actually signs, so a file that changed under the
operator between the two is caught rather than assumed away.

# 4. What is traced, and what is deliberately not

`crate::sign`'s §5 binds here: **no line in this file carries the
passphrase, its length, or the certificate's path.** A trace is captured to
a file `tools/ui-verify` keeps as evidence, and a length is a search-space
reduction while a path is a durable pointer at where somebody keeps their
digital ID. What is traced is what a diagnosis needs — that a signing was
asked for, which step it reached, and what the engine said.

## Item notes

### `fn run`

Separate from [`apply`] so that every exit is a `return` of a value the
compiler counts, rather than a `return` after a call somebody has to
remember to make. Every early exit here leaves the window stuck in
`Phase::Signing` until its outcome is handed back, and the compiler is what
guarantees one exists.

### `fn worded`

Pure, so every arm is asserted headlessly rather than by driving a window —
which matters more here than usual, because some of these arms are only
reachable on documents this repository does not commit.

# 5. The one decision in this function: whose rule refused

`SignApplyError` has a distinct, already-written sentence per variant, and
[`crate::text::sign::engine_refused`] frames them all as *"pdfcer did not
sign the document: …"*. That framing is right for most of them and **wrong
for the seed-value pair**, and the wrongness is expensive rather than
cosmetic.

The engine enforces a signature field's `/SV` dictionary (Table 234) **in
full** and is deliberately **stricter than Acrobat**: a required constraint
unmet is refused by name, and a constraint pdfcer cannot evaluate is refused
**rather than skipped**. So an operator will meet refusals here on documents
Acrobat signs — and *"pdfcer did not sign the document"* beside one of those
tells him, in plain English, that pdfcer is broken. He would be right to
conclude that from the sentence and wrong about the program, and a working
feature would be reported as a defect.

So [`crate::text::sign::author_imposed`] puts **the person who prepared the
document** in the subject position, quotes the engine's message verbatim
(because it names the constraint AND the satisfying values, which are the
actionable half), states the strictness as a deliberate choice, and gives two
remedies that do not require pdfcer to change.

A few other variants get their own wording for smaller reasons, each noted
at its arm. Everything else keeps the general form: the engine's sentence is
already an operator-facing one and re-wording it here would be a second
spelling of a fact with one author.

### `fn only_a_seed_value_refusal_blames_the_documents_author`

The whole of §5, asserted rather than argued. The engine enforces
`/SV` in full and is deliberately stricter than Acrobat, so the operator
will meet these on documents another reader signs — and the general
wording, *"pdfcer did not sign the document: …"*, would tell him in
plain English that pdfcer is broken.

The negative half matters as much: a refusal that is genuinely
pdfcer's (the fixed reservation) must NOT be dressed up as somebody
else's rule. Blaming the document's author for a pdfcer limit is the
same defect pointed the other way.

### `fn an_unusable_field_offers_the_other_two_routes`

Reachable even though the window filters its list, because the list is
read once when the window opens and the document can change under it.
The engine's message says what is wrong; this adds the remedy that
exists on the screen the operator is still looking at.

### `fn the_overflow_refusal_offers_a_remedy_this_window_has`

The engine's message ends *"enlarge --visible, or drop
--reason/--location"*, and there is no control here that enlarges the
box — `crate::sign::default_rect` fixes it. So the engine's sentence is
shown and the remedy offered is one the operator can actually perform.
