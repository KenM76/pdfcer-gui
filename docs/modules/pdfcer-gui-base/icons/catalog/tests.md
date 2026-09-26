# `pdfcer-gui-base/icons/catalog/tests`

## Item notes

### `fn all_is_exhaustive_and_free_of_duplicates`

Everything catalogue-wide — "every asset parses", "every asset
rasterizes to something visible", "redaction is the only filled one"
— iterates `ALL`. A variant left out of it is therefore not merely
untested: it is *silently* untested, and a broken asset behind it
ships green.

There is no reflection in Rust to count enum variants, so this checks
the two things that would actually go wrong: a duplicate entry (a
copy-paste that hid the variant it was meant to add) and a count that
no longer matches the number of distinct keys.

### `fn every_name_round_trips_through_from_key`

[`Icon::from_key`] is documented as the inverse of [`Icon::name`].
This is what keeps that true if `from_key` is ever rewritten as a
`match` or a map for speed.

### `fn an_unknown_key_resolves_to_nothing`

The whole missing-icon story downstream ([`super::super::paint`])
depends on this returning `None` instead of guessing at a nearest
match: a fuzzy resolver would draw the *wrong* glyph for a typo,
which is undetectable, where `None` is drawn as a visible mark and
traced.
