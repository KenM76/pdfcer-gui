# `pdfcer-gui/shell/commands/tests`

`shell::commands::tests` — the properties every registration in this
catalogue must hold.


> **the assertions about a catalog are a different subject from the
> catalog**, and the catalog is the half a reader opens to find out what a
> control says.

It is the same cut [`super::catalog`] took a week earlier, one level up.
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


## `#![cfg(test)]` as well as the parent's `#[cfg(test)] mod tests;`

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

## Item notes

### `fn every_handler_token_is_unique`

The shell explicitly permits it — two ids may share a token if the
application wants two names for one handler — which is exactly why
this needs asserting on *our* side. pdfcer has no such pair, so a
collision here is a typo in a hand-assigned number, and its symptom
would be one command silently doing another's work. Nothing else in
the system can detect that.

### `fn every_handler_token_is_in_its_tabs_block`

The blocks are what make a collision improbable in the first place
and what makes a raw token in a trace readable — `4xx` is an Edit
command without looking anything up. A number in the wrong block is
how the next one gets assigned on top of an existing command.

### `fn every_predicate_names_a_documented_condition`

A predicate naming a condition the application never publishes is a
command that is permanently greyed — and it fails silently, because
an unset condition and a false condition are the same value. The
vocabulary is small on purpose; this is what keeps it small.

### `fn with_no_document_only_the_document_free_commands_are_enabled`

The headless equivalent of launching pdfcer and looking at the
ribbon. It is asserted as an exact set rather than a count, because
the interesting failure is a *specific* command escaping its
predicate — `pages.delete` live with nothing open — and a count
would pass as long as some other command lost one.

### `fn an_empty_document_arms_nothing_that_needs_a_page`

`/Count 0` is valid PDF. pdfcer opens such a file and says "This
document has no pages" rather than reporting a failure — so the
condition set it publishes has `doc.open` and not `doc.pages`, and
this asserts the consequence.

### `fn every_command_has_a_tooltip`

The catalog type makes this structurally true, so the test is
guarding the *wiring*: a command built with `Command::new` and
never given `.with_tooltip` would compile.

### `fn every_icon_key_a_command_names_resolves_to_real_art`

| test | question |
|---|---|
| the split | *does this command name a glyph at all?* |
| this one | *and does that name resolve to a picture?* |

# What a wrong key actually does, which is why this is not cosmetic

It does **not** crash and it does **not** draw nothing.
`icons::paint_ribbon_icon` falls through to `paint_missing_mark`, which
draws a rounded square with a diagonal slash — a deliberate, visible
mark, argued at length in `icons::paint`'s header as *not* a
placeholder: it says "there is no glyph for this", which is a true
statement about the build rather than an invitation to believe a
control is coming.

That is the right behaviour at run time and it is exactly why a test
is needed. The failure is **legible on screen and silent everywhere
else**: a typo in a `with_icon("…")` string compiles, registers,
renders, passes the coverage split (the key is `Some`), passes the
kebab-case check (the typo is kebab), and ships as a slashed box in
the middle of the File tab. The only oracle was a screenshot, and
`MODES_AND_PANELS.md` is clear that a defect an oracle found deserves
a test that would have found it too.

Asserted over the **whole registry** rather than over the ribbon
manifest, and that is the wider claim on purpose: a command's icon is
drawn wherever the command is drawn — the band, the quick-access
toolbar, the overflow menu, a context menu, the collapsed-group popup,
the shortcuts dialog. Scoping this to the ribbon would bless a broken
key on any of the other five surfaces.

### `fn a_plausible_but_absent_icon_key_does_not_resolve`

`PROJECT_PLAN.md` §4.1 records a gate that printed "clean" while
checking a handful of files, and the standing lesson from it is that
*finding nothing looks exactly like finding no violations*. So the
predicate the test above is built on — `Icon::from_key` returning
`None` for a name that is not in the set — is asserted directly,
against a key shaped exactly like the typo this is guarding against:
plausible, kebab-case, and absent.

### `fn icon_keys_are_kebab_case`

A key that does not match the set's spelling resolves to nothing at
run time and renders as a missing glyph — a placeholder arriving
through the back door, and one that no headless test would
otherwise see.
