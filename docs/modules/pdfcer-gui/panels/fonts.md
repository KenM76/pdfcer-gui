# `panels::fonts` — what fonts the document declares, and what they cost

Salvaged from the old shell's `panels_structure.rs`. **The report came
across; the two controls did not.**

# Where this panel lives, and why it moved

`file.fonts`, on File ▸ Document, beside Properties — not View ▸ Panels
where the old shell had it. `RIBBON_IA.md` §7's migration map moves it,
and the reason is one sentence in the tab's own module docs: the Fonts
panel answers *"what is inside this file"*, not *"what is on my screen"*.

# Why the panel says *why*, when the parity reference does not

Acrobat refuses to unembed a font whose character codes are glyph indices
into its own embedded program, and it refuses **silently** — the font
simply is not in its unembed list, with no reason shown anywhere.
Corroborated by a user whose largest, most size-costly font was absent
from the list with no explanation offered.

A shorter list is not actionable. *"This font's character codes are
positions inside this specific embedded program"* is. That is rule 4
applied to a refusal rather than to a suggestion, and it is this panel's
main reason to exist.

`pdfcer-core` is built for it: `Removability`'s own doc comment says the
unembedding verb *"consumes this exact value, so a font pdfcer shows as
blocked and a font pdfcer declines to unembed are the same set by
construction."*

The measured stakes, from a 64-file survey of the PDFBox corpus: of the
30 files that embed fonts, **87 % embed subsets, 40 % use `Identity-H`,
and only 50 % carry `/ToUnicode`**. So the common case for "just remove
the embedded fonts" is a case where removal destroys the document, and
the operator has no way to know that from a font list alone.

# The coverage note is above the list, not beneath it

A font inventory that quietly misses a surface and prints a confident
list is this project's most-repeated defect shape. So the panel states
which font-bearing surfaces were searched **and the one that was not**,
unconditionally, before the list. Acrobat's own coverage here is recorded
as an unconfirmed gap, so pdfcer states its own scope rather than assuming
parity with a behaviour nobody has measured.

The page-scan failure sits above even that, because it changes what an
empty list beneath it *means*: without it, "0 fonts" and "pdfcer could not
walk the page tree" render identically and an operator reads the second
as the first.

# What did NOT come across: unembed, and embed

The old panel carried two controls — remove a font's embedded program,
and embed a missing one — each in a batch form under the summary and a
per-row form at the foot of an expanded row, with a confirmation window
for the destructive half. Neither is here.

Both push a mutation through `pdfcer_core::edit::EditSession`, and at S3
[`crate::app::actions::Action`] carries zoom and page navigation and
nothing else. There is no command log, no undo, and
`crate::app::state::OpenDoc::edit_epoch` names itself as *the documented
seam* the first mutating arm must bump. A control that cannot commit is
an affordance for something that cannot work (`RIBBON_IA.md` P3, R83) —
and the destructive one would be worse than that: the old shell's
confirmation window exists because **three of unembedding's four
consequences are invisible on the canvas** (a broken PDF/A claim, an
invalidated signature, a renamed font), so a control that appeared to
work and did not would be indistinguishable from one that had.

Two consequences of the omission are worth naming, because both are
recoveries the report itself has to make:

1. The old panel's **batch embed block** was the only place a document's
   "how many fonts are missing a program" count appeared. It is now a
   plain sentence — [`crate::text::panels::fonts::fonts_missing_programs`]
   — because that count is a fact about the file, not about a plan.
2. The old panel's per-row **"you did this in this session"** lines are
   gone with the actions that produced them, and correctly: after an
   unembed the row's verdict is the one a font that *arrived*
   non-embedded carries, so the line existed to stop the panel erasing
   the operator's own action. With no action, there is nothing to erase.

# Every verdict is drawn at the same visual weight

As a plain label, never error-styled. A blocked verdict is a fact about
the **file**, and error styling would make it read as a pdfcer failure.
With no removal control, that rule is easier to keep than it was: the old
panel's only difference between a removable row and a refused one was the
presence of the control, and now there is none.

Discoverability does not suffer, because the **collapsed header already
carries the verdict word** — "No blocker" against "Locked to program"
answers the question at a glance, without opening a single row.

# Rows are largest first

The operator opening this panel is usually asking *"which font is costing
me the most"*, and that ordering answers it with no control to find. Ties
keep discovery order, which `sort_by_key` preserves.

## Item notes

### `fn row_body`

Split out so [`body`]'s loop stays readable, and because the reason
ladder is the substantive part of the panel: it is where the sentence
this panel exists to say actually gets said.

### `fn fs_type_lines`

Four states, and **none of them may look like `0`.** `fsType == 0`
genuinely *means* Installable — the most permissive value the field can
express — so a blank, a dash, or an empty line for "we could not read it"
would assert the broadest embedding right there is on the strength of
bytes nobody read. The OpenType specification defines no default for the
absent case, so pdfcer defines none either: unknown says the word
"Unknown" in its own sentence, and "this format has no such field" says
that instead.

A free function because it needs nothing but the bits and a `Ui`.

### `fn an_identity_encoded_font_is_blocked_and_says_which_tier`

The sentence this panel exists to say, asserted against a real file.
Acrobat refuses the same font and shows no reason at all; a shorter
list is not actionable and this is the difference.

Asserted on the verdict and on `to_unicode`, not on the wording:
`crate::text::panels::fonts` already tests that the two tiers of the
sentence differ, and a copy edit should not break a panel test.

### `fn the_missing_program_count_matches_the_row_verdicts`

Two independent readings of the same inventory — the summary counts
`Program::NotEmbedded`, the rows print `Removability::NotEmbedded` —
and they are computed in different places. `Removability`'s own docs
promise they mean the same thing ("There is no embedded program"), so
a document where they disagree is a document where the summary is
lying about the list beneath it.

### `fn the_document_total_is_the_sum_of_the_rows`

`embedded_bytes()` and the per-row `stored_bytes()` are two
computations over one inventory, and the panel prints both. A total
that does not add up is the fastest way to lose an operator's trust
in every other number on the panel.

### `fn every_font_gets_a_verdict_word_in_its_collapsed_header`

The panel's discoverability argument in one assertion: every font
gets a verdict word, so the question is answerable from the collapsed
list. A record that fell through to no verdict at all would be a row
an operator has to open to learn nothing.
