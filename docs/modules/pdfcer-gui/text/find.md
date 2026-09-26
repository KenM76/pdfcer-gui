# `text::find` — every string the Find bar shows

One area of the catalog described in [`crate::text`]'s header. Two
consumers, and the split between them is the same one
[`crate::text::status`] draws:

| Consumer | What it reads from here |
|---|---|
| [`crate::find::bar`] | the floating box — the field, the two step buttons, the position readout, the options menu and everything in it, the close button |
| [`crate::app::status`] | the **Find toggle**, which `RIBBON_IA.md` §6 puts on the status bar |

The command's own label and tooltip are **not** here: `edit.find` is a
registered command, so its copy lives in [`crate::text::commands`] with
every other command's, keyed by command id and consumed by
`crate::shell::commands`. This file holds the copy the *controls* own.

## Why the wildcard control's label says what `#` and `?` do

Because the alternative shipped once and was a defect. `pdfcer-core`'s
[`pdfcer_core::edit::TextSearchOptions::wildcards`] records it in full:
the old shell's Find bar ran through `EditSession::find_text`, which
passes `with_wildcards(true)`, so typing a literal `?` matched **every
character on the page** and nothing on screen said why. Core's own
conclusion is that the fix belongs in the front end — the verb keeps its
documented pattern behaviour and the *default* moved to off — so this
build searches for what was typed, and the operator who wants patterns
ticks a box whose label names the two characters and what each does.

A control called "Wildcards" with no further explanation would be the
same defect with a checkbox in front of it: the operator still would not
know that `#` is a digit class, and would still be surprised by `?`.

## Why the whole-word rule is worded as three plain descriptions

ISO 32000-1 §14.8.2.5 NOTE 1 declines to define "word", and
[`pdfcer_core::edit::WordBoundary`] carries the whole argument. What
reaches an operator must not be the enum's names — `Alphanumeric`,
`NonSpace`, `NonSpaceOrDash` are the *implementation's* vocabulary — but
a description of the consequence they will actually notice: whether
`well-known` contains the word `known`, and whether `A-12/B` is one token
or three. So each label states the rule and each tooltip states the
consequence, with a worked example.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the chord only when the chord works.** [`toggle_tooltip`] names
  `Ctrl+F` because `crate::shell::manifest::built_in`'s keymap binds it
  and `crate::app::keyboard::parse_chord` can spell it. Both halves are
  required — the keymap alone was not enough for `Ctrl+O`, which was
  named in a tooltip and did nothing for the whole of the ribbon's first
  life.
- **Never state a capability the build does not have.** There is no
  "Find all", no "Search open documents", no results list, and no string
  here for any of them.

## The glyphs

`⏴` and `⏵` are the step buttons' entire visible text, and `×` is the
close button's. A codepoint the bundled font set cannot draw renders as a
tofu box, which is defect D2's shape — an invisible label — on a control
an operator has to hit. `crate::text::status`'s header records that the
obvious choices (`◀ ▶ ▸ ▾`) are **absent** from egui's bundled fonts and
that `⏴ ⏵ ⏶ ⏷` are present; the same three glyphs are re-asserted by
[`crate::find::bar::tests::every_glyph_the_find_bar_draws_has_a_glyph`],
because a test that lives beside the status bar cannot see this file.
