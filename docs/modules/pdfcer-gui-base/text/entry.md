# `text::entry` — the sentences a value box shows

`describe(&EntryError)` gives one sentence per refusal, each naming what to
type instead. `length_help` and `number_help` are the hover text for the two
box families and list what the box accepts, since arithmetic and units in a
number box are invisible until someone says they exist.

The wording lives in the text layer so a translation touches one file; the
reader in `entry` never formats prose.
