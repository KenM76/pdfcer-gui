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
