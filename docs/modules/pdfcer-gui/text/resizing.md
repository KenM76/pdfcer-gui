# `pdfcer-gui/text/resizing`

## Item notes

### `fn nothing_is_phrased_in_the_file_formats_vocabulary`

The rule in the module header, mechanised. An operator can see a shape,
a line of text and a picture; they cannot see a *node*, a *path object*
or a *show operator*, and a refusal phrased in those terms reads as an
internal error rather than as a limit.

The three that are left all describe the operator's own situation
rather than the engine's. Everything the ENGINE refuses is worded by the
engine and reaches the status row through `vector_edit` — see
`canvas::resizing`'s note on the preflight for why there is no
shell-side sentence standing in for it.

### `fn the_refusal_with_an_alternative_offers_it`

A refusal that does not say what to do instead is a shrug with a capital
letter — `text::commands`' own rule.

This test used to assert TWO of them, and the other one is gone with
its sentence. `ManyObjects` said *"select just the one you want"*, which
was a real alternative to a real limit until `transform_objects` took a
slice on 2026-08-20. **The assertion outliving the limit is the hazard
worth naming here**: a test that pins a refusal's wording is a test that
will keep that refusal alive through the release that made it false,
which is what happened to the engine's own `NoMatch` message twice
before it was split.

### `fn every_refusal_is_still_raised_somewhere`

The guard against the failure the row above names. Two of the six
sentences here became false the day a new verb shipped, and nothing in
this file would have noticed: a refusal is a claim with a date on it,
and the only thing that dates it is the code path that raises it.

So this asserts the pairing rather than the prose — every variant is
raised somewhere in `canvas::resizing`, which is the one file allowed to
raise them. It cannot check that the *reason* is still true, and does
not pretend to; what it catches is a variant whose call site has gone.
