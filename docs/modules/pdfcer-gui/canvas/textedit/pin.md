# `canvas::textedit::pin` — naming the exact show operator, and the exact
buffer it lives in

## What a pin is, and why nothing that edits text may go without one

`pdfcer-core`'s text verbs — [`EditSession::edit_text`] and
[`EditSession::format_text`] — locate their operand two ways. Given only a
search string they find the **first** show operator on the page whose
decoded text matches. Given a `pinned_span` they find the one whose byte
span in the decoded content buffer is exactly that.

The difference is not an optimisation. On a title-block sheet with two runs
reading `REV A`, the unpinned form edits the wrong one, silently, with no
error anywhere. This operator's own benchmark drawing carries **3,007
single-character show operators** in one page stream, so "the first match is
the one the operator meant" is false on the documents this program exists
for.

## Why this is its own module rather than a private detail of the caret

It was a private detail of the caret until 2026-08-27, inline in
[`super::plan`], and that was correct while exactly one thing edited text.
`format_text` is the second: restyling an existing run takes **the same
`pinned_span` and the same `EditTarget`** as replacing its text — the engine
shaped the two verbs that way deliberately, *"so a shell that has decided
which stream a caret is in does not have to translate that decision between
two verbs."*

A second copy of that decision is the thing to avoid. The `EditTarget` arm
below is nine lines of code and sixty of argument, and the argument is what
makes it right; a paraphrase of it beside the restyle verb would compile,
would look correct, and would drift.

## The extraction here is NOT the shared page-text cache

`crate::app::cache`'s extraction runs with `ExtractOptions::default()`, and
`capture_provenance` **defaults to off** — the engine's own words:
*"`None` unless the extraction set `ExtractOptions::capture_provenance`;
this keeps the default Pass 4 output byte-for-byte unchanged."*

With it off, `provenance()` answers `None` for every glyph, and a caller
built on the shared cache would get **no pin at all** while every line of it
kept compiling. That is this project's canonical failure shape: a correct
decision function wired to a value that is always the same.

Widening the shared cache is the other option and is worse. Every consumer
of `page_text()` — Find, both copy verbs, the text sweep — would then pay
for provenance on every page, and extraction is the expensive thing this
shell does (392 ms on the benchmark sheet). Paying it **once per edit**, in
[`resolve`], is the whole cost, and an edit is already an operation that
saves and re-rasters.

The run index is shared between the two extractions, which is safe and is
worth stating: `capture_provenance` populates a field and changes no
segmentation, so `runs[i]` names the same run under both options.
