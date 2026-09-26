# `pdfcer-gui/app/actions/fonts`

## Item notes

### `fn embed`

No pre-flight refusal check here. `embed_fonts` runs `embed_refusal` itself
before any mutation and returns the refusal as an `Err`, so calling it first
would be a second implementation of a guard the engine already owns — the
failure `dispatch::routes`' header names in a different register.

### `fn unembed`

No pre-flight refusal check, for [`embed`]'s reason: `unembed_fonts` runs
`unembed_refusal` itself before mutating.

And **no PDF/A gate here either**, deliberately. The engine leaves PDF/A
out of its refusal and says why: *"unembedding genuinely breaks that
conformance … but it is a consequence the operator may knowingly accept, not
a structural impossibility. The core reports it and **the shells gate on
it**."* This shell's gate is the sentence in `dialogs::unembed`, which the
operator reads before pressing the button — a disclosure, not a refusal,
because the decision is theirs and the engine says so.
