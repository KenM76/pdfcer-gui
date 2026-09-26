# `trust::tests` — the decision table, and the two things a green test here
must not be read as proving

## What these DO cover

The pure half: which path wins, what happens when a configured path is
wrong, and — the important one — that every no-anchors situation produces a
**distinct** [`Anchors`] variant rather than sharing one. That last property
is the whole safety argument of the surface built on it, because the four
states call for four different actions.

## ★★ What a pass here does NOT prove

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

## ★ Why the environment is not mocked

[`super::candidate_paths`] reads `%APPDATA%`, and a test that set it would
be mutating process-global state that every other test in this binary shares
— `cargo test` runs them on threads. So the tests below either take a path
as an argument (which is why [`super::locate`] takes one) or assert a
property that holds whatever `%APPDATA%` says: that the list is either empty
or every entry ends in the address book's file name. A test that asserted
*four* candidates would be a test about whichever machine ran it.

★★ **The inner `#![cfg(test)]` below is load-bearing and is not a duplicate
of the outer `#[cfg(test)] mod tests;`.** Without it,
`tools/gates/check-ui-strings.sh` walks this file as ordinary source and
reports every assertion message as a user-visible string that belongs in the
catalog — exclusion 2b in that gate, whose own comment records the day a
split under R2 reintroduced 28 such false hits. It is the same line every
other split test file in this crate carries.
