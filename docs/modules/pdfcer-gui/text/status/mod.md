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
