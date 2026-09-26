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

## Item notes

### `fn registry`

Duplicated from [`super::tests`] rather than shared, and deliberately.
It is four lines, it is the *subject* of both files, and a helper imported
across the seam would make one of the two modules unable to state its own
setup — which is the property that lets either file be read alone. The
obligation it creates is that both must call `register` and nothing else;
a divergence would make the two files disagree about what they are
counting, and `the_icon_coverage_split_adds_up_to_the_registry` asserts an
identity over the same registry `registration_succeeds_…` sizes.

### `fn the_icon_coverage_split_adds_up_to_the_registry`

This module's header quotes a split — *"N of M named, K refused"* — and
the three dated notes below it are a record of that pair drifting. It
drifted a fourth time and in a way the earlier notes make embarrassing:
the header read *82 of 93 named, 12 refused*, and 82 + 12 = **94**, while
the registry held 93. The truth was 81 named.

Nothing caught it, and the reason is stated in the note it sits under: the
registry's *size* is pinned by
[`Self::registration_succeeds_and_registers_every_command`] and its
*split* was pinned by nothing. So this pins the split — as an identity
rather than as two more literals, which is deliberate. Two literals would
be two more numbers to drift; an identity is a property, and the only way
to make it false is to make one of its terms genuinely wrong.

The refused list lives in this module's header table rather than in code,
because each entry is an **argument** and arguments belong at the
registration they justify. What this asserts is that the arithmetic
closes: every command either names a glyph or is one of the refusals, and
the two counts partition the registry with nothing left over.
