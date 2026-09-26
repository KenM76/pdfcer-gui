# `pdfcer-gui-base/redact/reach`

## Item notes

### `fn each_reach_does_what_its_label_promises`

Each value carries a label the operator reads and acts on, and every one
of those labels is a claim about what the engine will do. Hand-written,
they are a snapshot of the engine on the day they were typed. Measured
against `scrubs_hidden_carriers` and `blanks_content_streams`, a
redefinition on the engine's side fails here — in the one place that can
still stop the wrong sentence reaching him — rather than shipping as a
setting that quietly no longer does what it says.

### `fn every_reach_is_offered_and_the_ladder_only_widens`

The settings group iterates `ALL`, so a value missing from it is a value
the operator cannot choose and no other test would notice. The order is
asserted too: the radio list reads as a ladder from least to most
destructive, and a list that stopped being monotonic would be a control
whose shape lies about what it does.

### `fn every_key_round_trips_and_is_unique`

`from_key` is the parser for `preferences.txt`; a duplicate key would
make one value unreachable from a saved file while the settings window
went on offering it.

### `const ALL`

Narrowest to widest, so the control reads top to bottom as *less … more*
destruction. Unlike `super::quality::RenderQuality::ALL` the default is
**not** in the middle by accident: it is second because that is where
the scale puts it, and a reader who stops at the first two has met the
only two values that never edit a page they did not look at.

### `fn scope`

Exhaustive on this crate's enum by construction, which is the whole
reason the shell carries one: the engine's `ResidualScope` is
`#[non_exhaustive]` and a match on it would compile with a catch-all
that silently swallowed a fourth value.
