# `pdfcer-gui/shell/commands/ledger`

`shell::commands::ledger` — **the two counters, and the running record of
every time they moved.**

Two tests live here and they are not really two tests. They are one
**ledger**: a line per command ever added to or removed from this shell's
register, written at the literal it changed, saying what the control is,
which operator request asked for it, and — the part that is worth the
space — *why the number moved by that amount and not another*.

## Why this is a file of its own, and why it is mostly comments

A file of its own under **R2**, along the seam that file's own header
names for its parent: **the assertions about a catalog are a
different subject from the catalog**, and this is the third cut along it —
[`super::catalog`] moved the *entries*, [`super::tests`] moved the *rules
about them*, and this moves the *history of the counts*.

The ratio is the point rather than an embarrassment. Roughly nine hundred
lines here are commentary against four lines of assertion, because the
assertions are two integers and an integer records nothing. What a reader
needs when `registration_succeeds_and_registers_every_command` fails is not
the number — the failure prints that — but *whether the change that moved it
was supposed to*, and the only place that can be answered is beside the
previous forty answers.

## Why the ledgers stay with the assertions rather than moving to the
registrations

[`super::tests`]' header states it and it survives this move unchanged:

> the running commentary on the command count, the icon-coverage split and
> the handler-token blocks is written *at the literal it explains*, and that
> literal is an assertion. Splitting a number from its argument is exactly
> the drift those ledgers exist to record.

⇒ So the cut is made between **whole tests**, never between a literal and
the notes that justify it. `every_handler_token_is_unique` and
`every_handler_token_is_in_its_tabs_block` stay in [`super::tests`] with
their own block table for the same reason.

## `#![cfg(test)]` as well as the parent's `#[cfg(test)] mod ledger;`

Redundant to the compiler and load-bearing to two gates —
`tools/gates/check-ui-strings.sh` stops scanning a file at a
`#[cfg(test)]` line and a whole-file test module has no such line to stop
at, and `check-theme-colors.sh` recognises the same inner attribute from
the AST. [`super::tests`]' header records what omitting it costs: a split
of `canvas/selection/tests.rs` without one reported 28 assertion messages
as operator-facing copy, and `crate::stamps::tests` later paid it again for
19 — *"the noise is the actual hazard"*.
