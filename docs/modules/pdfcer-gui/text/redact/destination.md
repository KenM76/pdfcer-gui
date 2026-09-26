# `text::redact::destination` — where the redacted document goes


> *"why does it have to save to a new file right away? Why can't it just
> wait on saving until I choose to save over the existing file or save as a
> new file?"*

Until that day the apply had exactly one destination — a new file — and
`crate::dialogs::redact::commit` recorded the absence of any alternative as
a design property: *"There is no 'save over the original' branch to find,
because there is none to write."* `crate::dialogs::redact::Destination`
carries the whole of why that was overruled and what survives of it.

## Its own file, and the seam it was cut along

`text/redact.rs` reached 1536 lines with this group in it, over rule R2's
1500-line ceiling. The seam is not arbitrary and it is not "the last thing
added": these ten strings are the only ones in the catalog that describe a
**destination** rather than a **removal**, they are consumed by one region
of one dialog, and every other sentence in the parent file would read the
same if they did not exist. A split along "what was added most recently"
would have put `confirm_button` here and `confirm_button_replace` there.

Re-exported by the parent with `pub use`, so every call site spells these
exactly as it did before — `crate::text::redact::destination_replace(..)` —
and the split is invisible to consumers, which is the property that makes it
a mechanical change rather than a rename.

## The wording rule this group adds to the three it inherits

`crate::text::redact`'s rules 1–3 bind here unchanged. This group adds a
fourth of its own, and every string below obeys it:

> **Name the file.** *"Replace the original"* is a sentence about a role.
> *"Replace sheet-01.pdf"* is a sentence about a file, and the operator is
> about to destroy a file. Every string here that refers to the document
> being replaced takes its name as an argument, including the button label —
> which is how `crate::dialogs::redact::file_name_of` came to exist, so that
> one file is spelled one way across the choice, the acknowledgement, the
> button and the outcome.
