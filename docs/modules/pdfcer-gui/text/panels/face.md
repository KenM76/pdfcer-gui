# `text::panels::face` — every string the face chooser shows

One control, two surfaces, one catalog. The Properties panel's *This text*
section and the ribbon's Format ▸ Font group draw the **same** face chooser
through [`crate::panels::properties::face`], so its wording lives in its own
module rather than inside [`super::properties`]: the face chooser is the
largest single subject either surface has, and it owes a disclosure neither
of the others does.

## What this module is obliged to say

`pdfcer-core`'s release note for the capability these strings describe,
verbatim:

> **FONTS** — text can be restyled to a face the document **DOES NOT
> CONTAIN**, for the fourteen faces every PDF reader is required to have.
> pdfcer authors the font resource on demand, with widths, embedding
> nothing. A face outside those fourteen still refuses by name — that needs
> a real font program.

Three clauses in that note become three obligations on the wording here, and
every string below discharges one of them:

1. **"a face the document does not contain"** — the chooser now offers two
   *kinds* of row, and they are different acts. Choosing a face the page
   already carries changes a `Tf` operand and nothing else. Choosing one of
   the fourteen makes pdfcer **write a new object into the operator's file**.
   An operator who cannot tell those apart has been handed a control that
   does two different things under one appearance. [`face_group_on_page`]
   and [`face_group_addable`] are the two headings that separate them.

2. **"embedding nothing"** — [`face_addable_disclosure`], and it is the
   reason this module has a header this long. See its own doc comment.

3. **"a face outside those fourteen still refuses by name"** — not a string
   in this module, because that refusal is a *status-bar* sentence and lives
   with the others in
   [`crate::text::status::selection::TextStyleRefusal::FaceNotOnPage`],
   whose wording was corrected in the same change. It is named here so the
   reader of this header can find it.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the thing and what the operator can do about it.**
- **Never state a capability the build does not have** — and, the half that
  costs more here, never keep stating a *limit* the build does not have.

## Item notes

### `fn the_disclosure_states_the_act_the_omission_and_the_consequence`

Asserted by content rather than by exact text, because the wording will
be improved and the facts must not be lost in the improving. Each of the
three is a separate obligation and each has its own way of going missing:

1. **The act** — that choosing the row writes to the file. Lost if the
   sentence is ever rewritten as a description of what the fourteen
   *are*.
2. **Not embedded** — the fact `pdfcer-core` states and this shell relays.
   Lost first, because it is the least comfortable clause.
3. **The reader's own copy** — the inference the operator cannot see, and
   the whole reason rule 4 puts this sentence on screen rather than in a
   doc comment.

### `fn the_two_group_headings_say_different_things`

They are the only thing distinguishing two rows that may read
identically — `Helvetica` the page carries and `Helvetica` pdfcer would
add are one string apart on screen and two different acts in the file.
A pair of headings differing by a word an operator skims past would put
the whole distinction back where it was before this change: nowhere.

### `fn the_empty_sentence_accounts_for_the_standard_fourteen`

*"No other font on this page can show these characters"* is exhaustive
only while the page is the only source. An operator reading that beside
a list which elsewhere offers `Times-Roman` out of thin air would
reasonably ask why it is not offered here.

### `fn every_sentence_in_the_offer_names_the_character_itself`

The whole of what the status bar cannot do. A block that said *"a
character in this text"* would have moved the refusal to a wider surface
and added nothing: the operator already knows they typed something, and
what they do not know is **which** keystroke the document refused — on a
pasted line it can be a character they never saw themselves type.

Asserted over a character outside ASCII on purpose. A build that
formatted with `{:?}` or escaped for a byte-oriented surface would print
`'\u{20ac}'` and pass a test written against `'q'`.

### `fn the_offer_names_the_font_that_refused_and_the_font_that_replaced_it`

[`refused_char_named`] names the face that **refused**;
[`refused_char_swapped`] names the face that is **now in force**. A build
that fed either the wrong one would tell the operator that the font they
just chose is the font that cannot type their character — which reads as
the feature not working, on a swap that worked.

### `fn the_offer_promises_pdfcer_finishes_the_job_and_never_asks_for_a_retype`

The block re-applies the operator's edit itself
([`crate::panels::properties::refusedchar`]), so a *"type it again"*
instruction is not merely unnecessary — it is **harmful**: an operator
who follows it types the character into a document that already has it
and gets two.

Asserted in the negative as well as the positive, because a build that
re-applied the edit *and* kept the old wording would pass a
promise-only test while producing exactly that double edit.

### `fn the_blocked_sentence_names_no_cause_it_cannot_see`

The face swap landed and the character still would not go in, and
**pdfcer does not know why**: the block is reached by arithmetic — the
edit epoch not moving — rather than by recognising a refusal.

⚠ *Carrying no cause* is the harder property to hold, which is why it is
asserted rather than left to a comment. A later session improving the
wording will be tempted to put an explanation back, and the explanation
it reaches for will be one this surface cannot see. The negative
assertions below exist to stop exactly that.

### `fn the_dead_end_says_the_document_is_unchanged_and_what_is_still_possible`

# Why the dead end is a sentence and not a caveat

The offer is coverage-tested against the refused character itself, so no
row in the list can refuse it and there is nothing for a caveat to warn
about. What that exactness exposes instead is an **empty** offer, for any
character no standard-14 face can encode. This is the sentence drawn
there, and its load-bearing assertion is: **say that nothing was
changed.** A dead end that does not say so reads as a
failure the operator has to go and check.

Plus the half the old sentence could not have: a dead end owes the
operator the *next* thing to try, even when that thing is not pdfcer's
to do.
