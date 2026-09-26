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

## Item notes

### `fn field_label`

A label rather than placeholder text inside the field. Placeholder text
disappears the moment the operator types, so a box identified only by its
placeholder has no name for as long as it is being used — and this one
floats over the page rather than sitting in a bar the operator already
recognises, so "what is this?" is worth answering permanently.

### `fn field_tooltip`

Names the three keys the field itself owns, because none of them is
discoverable by looking: Enter searches (and then steps), Shift+Enter
steps backwards, Escape closes. The last one is qualified — it closes the
bar *while you are typing in it* — because once focus has left the field
Escape belongs to the canvas's selection ladder, and a tooltip that
promised otherwise would be describing a key fight this build
deliberately does not have.

### `fn close`

`×` (U+00D7 MULTIPLICATION SIGN), not `✕`/`✖`, and not the word "Close".
The word is three times as wide as the control needs on a box that floats
over the page and is kept deliberately narrow; and U+00D7 is Latin-1, so it
is present in every font a desktop toolkit ships, where the dingbat crosses
are not.

### `fn toggle`

`RIBBON_IA.md` §6 lists this first among the status bar's controls. It is
a *toggle* rather than a button because the box it opens is a persistent
surface the operator leaves up while working through hits, and a control
that showed no state would give them no way to tell "it is closed" from "it
is open at the other end of the window".

### `fn position`

**One-based**, because it is read next to the status bar's page counter,
which is one-based for the same reason: an operator counts hits from one.
The zero-based index is an internal fact and never reaches this function.

Deliberately not `3/47`: a slash reads as a fraction or a date, and this
row already carries `n/N` shapes in the page box a few points to its
right.

### `fn no_matches`

A sentence rather than `0 of 0`, because zero of zero is arithmetic and
the operator's question is whether the search ran at all. This says it
ran.

### `fn step_unavailable_tooltip`

`RIBBON_IA.md` P3 allows greying only for *temporarily* unavailable and
only when it *"is always explained on hover"*. Both hold: there is nothing
to step through until a search has found something, that state ends on the
next Enter, and this is the sentence that says so. A greyed control with
no explanation is the shape defect D1 took — an affordance that looks
available and is inert, with nothing on screen to say why.

### `fn no_matches_tooltip`

Names the two limits that produce a surprising empty result on real
files, because both are properties of the *document* rather than of the
query and an operator has no way to guess either. Both are
`pdfcer-core`'s documented limits on `find_text`, not this shell's.

### `fn stale`

**The hits are not merely out of date; their geometry may name text
that is no longer there.** A `delete_*` renumbers and re-splices the
content stream, so a quad recorded before an edit can cover different
glyphs after it — and rule 4 forbids painting a mark over content that
does not say what the mark claims. So the highlights stop being drawn the
instant `edit_epoch` moves, the readout says so, and the query is kept:
re-running it is one keypress, and it is the operator's keypress rather
than a search this shell decided to repeat on a 5.6 MB drawing.

### `fn options`

A word plus the disclosure triangle the status bar already uses, rather
than a bare chevron or a gear: `crate::find::bar` puts the four search
options behind this button, and an operator who cannot find "case
sensitive" will look for a word before they will click an unlabelled
glyph. `⏷` is U+23F7, measured present in egui's bundled font set —
`crate::text::status`'s header records that the obvious `▾` is not.

### `fn options_tooltip`

Names what is inside it, because the whole cost of putting the options
behind a menu is that they stop being visible — and a button labelled
"Options" with no further word is a button an operator has to open to find
out whether it is worth opening.

### `fn match_case`

Phrased as the thing the operator switches **on**, matching Acrobat's own
"Case-Sensitive" toggle: off means an ordinary, forgiving search, which
is what a find bar does everywhere.

### `fn wildcards`

The label **names the two characters and what each does**. See this
module's header: a bare "Wildcards" would leave the operator exactly
where the old shell's silent pattern search left them, one checkbox
later.

### `fn wildcards_tooltip`

The last sentence is the hazard note the brief asks be left where the
next person will see it: a future "redact every hit" control cannot be
built on a wildcard search, because `mark_redactions_by_search` matches
**literally** and would decline to mark hits this bar highlighted.

### `fn find_zoom`

**The operator named this control.** His words, 2026-09-09: *"add a
checkbox option to our search bar called zoom — when I uncheck it just jump
to the page and highlight the found item as before but don't change the
zoom."* A request that carries a name is a request for that name, so the
label is the word he used and the tooltip carries the explanation.

Phrased as the thing the operator switches **on**, like its three siblings
— see [`match_case`]. On is what this build has always done; off is the new
answer he asked for.

### `fn find_zoom_tooltip`

**It names the mechanism, because the mechanism is not guessable.**

Nothing in Find has ever set a zoom. What changes the zoom on a jump is
**Fit page** / **Fit width** still being switched on: a fit is a standing
instruction to re-scale to whatever page is showing, so landing on a sheet
of a different size re-scales, and on a drawing set whose sheets are not
all the same size that is a large jump. An operator reading at a size they
chose experiences that as Find having zoomed them out.

So the tooltip says the two consequences of switching it off rather than
describing an implementation: the reading size is kept, and the view stops
following the fit. The second half is disclosure — turning this off *does*
change a setting the operator can see in the zoom readout, and a control
that quietly dropped Fit width without saying so would be the shell being
sneaky about its own state.


Shipped, this control governed the zoom and nothing else, so with it off
the view still scrolled the hit to the middle of the canvas on every
step. Ken: *“instead of jumping to the page that the text is found on and
leaving the page in its current position on the canvas it still zooms and
repositions the page on the canvas.”* The control now also holds the
POSITION, which is what he read the word to mean, so the sentence has to
say so — a tooltip that promises only half of what a control does is a
second defect sitting beside the first.

### `fn word_rule_tooltip`

Says outright that there is no correct answer to import. That is not
hedging: `pdfcer-core`'s `WordBoundary` quotes §14.8.2.5 NOTE 1 saying the
notion of a word *"is not precisely defined"*, and NOTE 4 offers a menu
of reader strategies rather than a rule. Telling the operator that the
choice is theirs is the honest version of a setting that exists because
the standard refused to decide.

### `fn unsearchable_one`

Worded as a fact about the DOCUMENT, not about the search and not about
pdfcer. "No matches" is still the answer to what they asked; the second
clause tells them why the answer may be incomplete. It deliberately does not
say "pdfcer cannot read" — Acrobat cannot read it either, the file simply
does not carry the mapping, and phrasing a file's own gap as a tool
limitation invites an operator to go looking for a better tool.

Singular and plural are separate strings rather than "font(s)", because a
parenthesised plural in a sentence an operator reads under pressure is how
software sounds when nobody cared.

### `fn blanks_trimmed`

Trimming silently would be the reported defect wearing the other coat.
He typed something; pdfcer searched for something else; without this row
there is no surface anywhere that says so. Rule 4: the page renders
normally and the inference is reported **off-canvas**, here, in the
bar's own second row beside the unsearchable-fonts note.

Deliberately neutral about which end and about which character. A
clipboard hands over tabs and non-breaking spaces as readily as spaces,
and naming the character would mean either six sentences or a wrong one.
*“Blanks”* covers all of them in the operator's own register.

### `fn blanks_kept`

Owed for the mirror-image reason: with trimming off, an invisible
character is deciding the answer and the operator cannot see it. This is
the row that turns *“why does this find nothing”* into one glance, which
is the whole of what O180 reported.

### `fn blanks_tooltip`

One tooltip for both rows rather than two: the operator needs the same
two facts in either state — what a blank is, and where the setting lives
— and the row above has already told them which way it is pointing.
