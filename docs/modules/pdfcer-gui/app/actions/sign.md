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
