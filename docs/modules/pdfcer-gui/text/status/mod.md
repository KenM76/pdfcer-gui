# `text::status` — every string the status bar shows

One area of the catalog described in [`crate::text`]'s header, and the
sole consumer is [`crate::app::status`]. Nothing here is read by the
ribbon: the status bar **mirrors** three View-tab commands under
amendment P1a (`RIBBON_IA.md` §2), and a mirror is a second surface for
one command, not a second command.

## The one place this file deliberately repeats the ribbon

[`fit_actual_size`] and [`fit_actual_size_tooltip`] say what
`crate::text::commands::view_zoom_actual` says, in the same words. That
is not an oversight and it is not a copy-paste that should be
de-duplicated into a shared constant:

- The two surfaces are mirrors of **one** command, so an operator who
  reads the ribbon's tooltip and then hovers the status bar's button
  must be told the same thing. Two paraphrases of one command is how a
  product acquires two different mental models of the same verb.
- They are nevertheless two *entries*, because the ribbon's catalog is
  keyed by command id and consumed by `crate::shell::commands`, while
  this one is keyed by control and consumed by a widget. Reaching across
  would make `text::status` depend on `text::commands`' `CommandText`
  type for no gain, and would put the first cross-area dependency in a
  catalog whose whole organising principle is one area per consumer.

**Both entries are now true**, and the wording being identical to the
ribbon's is what made fixing them one edit rather than two. The action
behind them raises `Action::ZoomTo(1.0)` (see the section of
[`crate::app::status`]'s module docs for that half), and the chord they
name has one owner (see [`crate::app::keyboard`]'s section for this
one). The same holds for the three mirrored fit tooltips: each is now
word-for-word its `crate::text::commands` twin, chord included.

## Why the arrows and the minus sign are in the catalog

`⏴`, `⏵`, `⏷`, `−`, `+`, `·` are *labels*: they are the entire visible
text of a control, and a control's visible text is exactly what rule R1
governs. The `check-ui-strings.sh` heuristic would never catch them —
it flags literals containing whitespace, and these contain none — so
they are here by the rule rather than by the gate, which is the
distinction that file's own header draws.

They are also the reason
[`crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`]
exists, and **that test has already paid for itself**. This file was
written with the obvious glyphs — `◀` `▶` for the page steps and `▸` `▾`
for the disclosure — and every one of the four is **missing from egui's
bundled font set** (Ubuntu-Light + NotoEmoji + emoji-icon-font). On
screen they would have rendered as tofu boxes: defect D2's shape, an
invisible label, with the operator's page position behind it. What the
font set does carry, measured rather than assumed, is `⏴ ⏵ ⏶ ⏷ ‹ › « »
○ • · – — − + % /`.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.** Every tooltip below is prose and ends in a
  full stop; every label is a name and does not.
- **Name the chord only when the chord works.** Actual size names
  `Ctrl+0` because the manifest keymap binds it there and
  `crate::app::keyboard::commands` enacts what the keymap says. Fit page
  and Fit width name **none**, because none reaches them: `Ctrl+0` is
  actual size's and `Ctrl+2` is `mode.review`'s. A string that named
  either here would be claiming half of a chord that has another owner.
  Do not invent a replacement chord to fill the gap; bind one in the
  manifest first, and it may be named the same day.
- **Never state a capability the build does not have.** The Find toggle
  `RIBBON_IA.md` §6 specifies has **no strings here**, and that is now a
  filing decision rather than an absence: the toggle exists, and its label
  and tooltip live in [`crate::text::find`] beside the rest of the Find
  surface's copy. One area per consumer is this catalog's organising
  principle, and the toggle's consumer is a control the Find module owns.

## Item notes

### `mod diagnostics`

Re-exported whole, on [`selection`]'s precedent: a catalog area is keyed
by the consumer it serves — here `app::status::notes` and nothing else —
so no call site changed and `t::diagnostics_clean` resolves where it always
did.

### `fn a_page_with_no_recognised_text_says_so_instead_of_counting_to_zero`

A build that formatted `0 blocks` would satisfy every check that only
looked for a non-empty string, and would tell an operator staring at an
un-OCR'd scan that the mode is working.

### `fn the_disclosure_reads_differently_open_and_closed`

A toggle whose two states read identically is a toggle whose state
the operator has to discover by clicking it, which is exactly the
affordance a disclosure triangle exists to remove.

### `fn the_edit_disclosure_frames_cores_sentences_without_altering_them`

Two properties, and both are load-bearing:

- **Verbatim.** The disclosure prose is written where the fact is known
  — inside the planner that decided to rewrite a rectangle as four
  lines. A shell that paraphrased it would put two descriptions of one
  surgery into the product, and the one further from the code is the
  one on screen. So this asserts *containment*, which is the mechanical
  form of "we added framing and changed nothing".
- **One line.** `crate::app::status` draws this inside a row whose
  height may not vary (R128), eliding what does not fit. A newline
  would defeat that from the string side, where no layout assertion is
  looking — the label wraps, the row grows, and the page re-fits itself
  at the moment the operator finishes a drag.

### `fn a_decline_does_not_read_like_a_disclosure`

The whole reason the worded decline got its own strings — rather than
borrowing the line the vector verbs already put in this bar — is that
*"this happened, and here is the part you cannot see"* and *"this did
not happen"* are different speech acts. One wording in one slot would
make a completed gesture and a refused one indistinguishable, in the
same place, which is worse than the trace-only state these replace.

Four properties, and each would be silently lost by an edit that looked
harmless:

1. **The lead-in diverges.** Neither decline may open with the edit
   disclosure's *"About your last edit"*.
2. **The mark diverges.** `⊗` and not `⚑` — the mark is what tells the
   two apart at a glance, before either sentence is read.
3. **The two declines read differently from each other.** "Nothing is
   selected" and "the page is still drawing" have different remedies,
   and the operator gets one line to tell them apart — the same
   property [`the_two_page_box_notes_read_differently`] defends for the
   page box's pair.
4. **One line.** `crate::app::status` draws these inside a row whose
   height may not vary (R128), eliding what does not fit. A newline
   would defeat that from the string side, where no layout assertion is
   looking.

### `fn the_decline_reports_the_state_rather_than_instructing_the_operator`

`view.zoom_selection` is greyed on `selection.bounds`, so the sentence
is reached by a chord — or, worse, in the race where the selection
evaporates between the frame that drew the enabled control and the
frame that applied it. In that second case the operator clicked a
control that was offered to them, and an imperative telling them to
select something first would be instructing them to repeat what they
just did.

Asserted as an absence of imperatives rather than against a fixed
string, so the copy can be rewritten and the property survives.

### `fn every_counted_note_is_singular_at_one`

Not pedantry: these lines are read in a small weak font at the edge
of the window, and "1 glyphs" is the kind of thing a reader notices
*instead of* the number, which is the part that matters.

The property is asserted structurally rather than against a table of
expected sentences: **the singular must not be the plural with the
digit swapped.** That catches a missing branch on every entry,
including the ones whose noun is not the first word ("text from 1
font not drawn"), and it keeps working when the copy is edited.

### `fn the_clamp_note_names_what_was_asked_and_what_was_given`

The whole value of the note is that it distinguishes "your number was
out of range" from "the box ignored you". A note that named only the
page landed on could not do that.

### `fn the_two_page_box_notes_read_differently`

"I typed a page that does not exist" and "I typed something that is
not a page number" are different mistakes with different fixes, and
the operator gets one line to tell them apart.

### `fn the_three_fit_controls_are_distinguishable`

Three controls in a row that read alike is the failure the ribbon's
own salvage notes record (two adjacent Content buttons both reading
`Aa`, distinguished only by their tooltips). Here both halves are
asserted.

### `fn each_fit_tooltip_names_exactly_the_chord_that_reaches_it`

The property defended here is not that any particular one of the three
is silent. It is **"a status-bar tooltip names a chord if and only if
that chord reaches the control"** — which, with one owner per chord
(`crate::app::keyboard`), means Actual size names `Ctrl+0` and the
other two name nothing.

The three assertions below are the direct expression of that, and the
last two are the ones that matter most — a chord *removed* from the
keyboard leaves no compile error behind, so the only thing standing
between the operator and a tooltip that lies is an assertion that the
string stayed silent.

### `fn the_fit_mirrors_repeat_the_ribbon_word_for_word`

The header's claim, asserted rather than trusted. Three status-bar
controls mirror three View ▸ Zoom commands under amendment P1a, and a
mirror that paraphrases is how a product acquires two mental models of
one verb. It is also what let the chord defect live on one surface and
not the other for as long as it did.

### `fn the_page_box_tooltip_states_the_commit_rule`

It is the only place the operator can learn that typing does not
navigate, and that property is the reason the control is usable at
all — see `crate::app::status`.
