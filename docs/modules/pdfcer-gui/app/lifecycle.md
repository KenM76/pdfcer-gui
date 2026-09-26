# `app::lifecycle` — opening a document, closing it, and the three ways an open can fail

Three methods on [`PdfcerApp`] and one predicate: what happens when a
document arrives, what happens when it leaves, and how a load failure is
told apart from a file pdfcer has not finished supporting.

## ★ Why this is its own file

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
