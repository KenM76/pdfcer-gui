# `pdfcer-gui/shell/commands/tests`

`shell::commands::tests` — the properties every registration in this
catalogue must hold.


> **the assertions about a catalog are a different subject from the
> catalog**, and the catalog is the half a reader opens to find out what a
> control says.

★ It is the same cut [`super::catalog`] took a week earlier, one level up.
That split moved the *entries*; this one moves the *rules about them*, and
what is left in [`super`] is what a caller of this module actually reaches
for: `register`, `FILE_RECENT`, the `mapping` re-exports and the `reach`
declaration.


This file went past 1,500 lines again, and the entry that did it was
`file.stamp_collection`'s (O169). Nine hundred of those lines were the two
**counters** — `registration_succeeds_and_registers_every_command` and
`the_icon_coverage_split_adds_up_to_the_registry` — and almost all of that
was their ledgers rather than their code.

Those two moved whole to [`super::ledger`]. What stayed is everything that
asserts a *property* rather than a *count*: the handler-token blocks, the
condition vocabulary, the with-nothing-open enabled set, the tooltip rule
and the icon-key rules.

⚠ **The cut is between whole tests, never between a literal and the notes
that justify it.** The ledgers stay with their assertions rather than moving
to the registrations: the running commentary is written *at the literal it
explains*, and that literal is an assertion. Splitting a number from its
argument is exactly the drift those ledgers exist to record — which is why
`every_handler_token_is_unique` and `every_handler_token_is_in_its_tabs_block`
are still here, carrying their own block table, rather than being swept into
a file called *ledger* because the word fits.

## Nothing moved but the module wrapper


## ★ `#![cfg(test)]` as well as the parent's `#[cfg(test)] mod tests;`

Redundant to the compiler and load-bearing to a gate.
`tools/gates/check-ui-strings.sh` stops scanning a file at
`#[cfg(test)]` — assertion messages are prose read by whoever is staring at
a failing test, never by an operator — and a whole-file test module has no
such line to stop at. Its own header records what happened when
`canvas/selection/tests.rs` was split without one: **28 assertion messages
reported as operator-facing copy**, and *"the noise is the actual hazard"*,
because a report full of false positives trains people to ignore it.

The inner attribute is the marker that gate recognises, and
`check-theme-colors.sh` recognises the same one from the AST. Both state why
it is the marker rather than the filename: the property that earns the
exemption is *"not in the shipped binary"*, and a filename is a restatement
of that which goes stale the moment a third such module is written.
