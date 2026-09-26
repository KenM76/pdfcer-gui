# `trust::tests` — the decision table, and the two things a green test here
must not be read as proving

## What these DO cover

The pure half: which path wins, what happens when a configured path is
wrong, and — the important one — that every no-anchors situation produces a
**distinct** [`Anchors`] variant rather than sharing one. That last property
is the whole safety argument of the surface built on it, because the four
states call for four different actions.

## What a pass here does NOT prove

1. **That any real signature verifies.** No fixture in this repository
   carries a signature whose signer chains to a real AATL anchor, and one
   could not be committed: it would need somebody's real certificate and the
   verdict would expire with it. The cryptography is `pdfcer-core`'s and is
   tested there against pyHanko-signed fixtures whose verdicts were recorded
   from pyHanko's own validator first.
2. **That the operator can see any of it.** These are values. Only
   `tools/ui-verify`, driving the real binary, answers whether a rectangle
   was drawn — and this project has a standing record of tests that passed
   while the feature was unreachable.

## Why the environment is not mocked

[`super::candidate_paths`] reads `%APPDATA%`, and a test that set it would
be mutating process-global state that every other test in this binary shares
— `cargo test` runs them on threads. So the tests below either take a path
as an argument (which is why [`super::locate`] takes one) or assert a
property that holds whatever `%APPDATA%` says: that the list is either empty
or every entry ends in the address book's file name. A test that asserted
*four* candidates would be a test about whichever machine ran it.

**The inner `#![cfg(test)]` below is load-bearing and is not a duplicate
of the outer `#[cfg(test)] mod tests;`.** Without it,
`tools/gates/check-ui-strings.sh` walks this file as ordinary source and
reports every assertion message as a user-visible string that belongs in the
catalog — exclusion 2b in that gate, whose own comment records the day a
split under R2 reintroduced 28 such false hits. It is the same line every
other split test file in this crate carries.

## Item notes

### `fn every_candidate_path_names_an_address_book`

The floor that stops this file's other assertions being about nothing: a
`candidate_paths` that silently returned an empty vector on Windows would
make [`the_four_no_anchor_states_are_distinct`] pass by never finding a
store, which is a green result measuring the absence of an environment
variable.

### `fn a_configured_path_that_is_missing_does_not_fall_back`

The behaviour this pins is the one a well-meaning edit would "improve": if
the typed path is missing, try the usual places. That would make a typo
behave like a correct entry pointing somewhere else, and the operator would
have no way to tell which store was actually read — on the one surface where
which certificates were used is the entire question.

### `fn a_blank_path_asks_the_machine`

Clearing a text box is how a person un-sets it, and reading an empty value
as a positive choice would suppress the feature permanently with no way back
except hand-editing a file.

### `fn a_typed_path_is_trimmed`

`prefs` trims when it parses the file, and this trims again, and the
duplication is deliberate: the file is not the only route a value takes —
the Settings field writes it directly — and a trailing space is a path that
does not exist, which presents as *"the setting does nothing"* rather than
as *"that file is not there"*.

### `fn opting_out_is_not_reported_as_a_missing_store`

The single most important assertion in this file. `Off` is the shipped
default, so it is the state almost every operator is in, and it is the one
they can fix in five seconds. Reporting it as *"pdfcer found no trust list"*
would send them looking for an Acrobat install they already have.

### `fn a_missing_configured_path_is_distinguishable_from_no_store_at_all`

The two produce the same *outcome* — no anchors — and completely different
remedies. `NoStore { configured_missing: Some(..) }` is what lets the panel
say *"there is no file where you pointed"* instead of *"install Acrobat"*.

### `fn the_four_no_anchor_states_are_distinct`

Not a tautology over an enum: the cheap implementation of this feature has
one "trust not checked" state and this is the test that refuses it. Two of
the four are constructed here from real inputs; the other two need a
filesystem this test does not have, so their *distinctness* is asserted over
hand-built values, which is enough — the property under test is that the
type can tell them apart at all.

### `fn an_unsigned_document_still_reports_where_the_anchors_would_have_come_from`

The second half is the point. A report that carried an empty verdict list
AND no anchor state would leave the panel unable to distinguish *"this
document is not signed"* from *"pdfcer did not look"*, which is the same
collapse the four states exist to prevent, one level up.

### `fn a_store_date_is_a_calendar_date`

Pinned against a known instant rather than against "today", for
`app::clock`'s own stated reason: a test that formats the current date
passes for a year and then fails at a month boundary for reasons nobody
remembers.
