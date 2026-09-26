# `text::panels::fonts` — the Fonts panel's inventory report

Every string the Fonts panel shows. Salvaged verbatim from the old
shell's `ui_text.rs`, with the doc comments that record why each is
worded as it is.

## What is here, and what deliberately is not

The old panel had two halves: a **report** (what fonts this document
declares, what their embedded programs cost, and what would block
removing one) and two **destructive/constructive controls** (unembed a
font's program, embed a missing one). Only the report came across.

That is not an oversight and it is not a snippet-salvage. Both controls
push a mutation through [`pdfcer_core::edit::EditSession`], and at stage
S3 [`crate::app::actions::Action`] has seven variants, all of which are
zoom or page navigation — there is no mutating action, no command log, no
undo, and `crate::app::state::OpenDoc::edit_epoch`'s own doc comment
names itself as the documented seam the *first* mutating arm must bump.
A control that cannot commit is an affordance for something that cannot
work, which `RIBBON_IA.md` P3 forbids by name.

The report is where nearly all the panel's value is anyway, and the
reason is measured: from a 64-file survey of the PDFBox corpus, of the 30
files that embed fonts, **87 % embed subsets, 40 % use `Identity-H`, and
only 50 % carry `/ToUnicode`**. So the common case for "just remove the
embedded fonts" is a case where removal destroys the document, and the
operator has no way to know that from a font list alone. Telling them
*why* a font is refused is the panel's main reason to exist — and it does
that with no verb attached.

## Why the panel says *why*, when the parity reference does not

Acrobat refuses to unembed a font whose character codes are glyph indices
into its own embedded program, and it refuses **silently** — the font
simply is not in its unembed list, with no reason shown anywhere
(`Acrobat_Features/optimize__font_unembedding.md`, sourced to a former
Adobe Principal Scientist; independently corroborated by a user whose
largest, most size-costly font was absent from the list with no
explanation offered).

A shorter list is not actionable. "This font's character codes are
positions inside this specific embedded program" is. That is project rule
4 applied to a refusal rather than to a suggestion.

## The verdict words are TWO WORDS EACH, and that is a measurement

[`font_verdict_removable`] and its four siblings were sentences ("No
blocking condition found.", "Locked to this embedded program.") until a
screenshot of the running panel showed the row **clipped at the dock's
edge**: the byte size, the field an operator opens this panel for, was
cut to `59`. egui lays an over-wide row out anyway, off the edge of its
parent, and hands back a perfectly ordinary `Response` — so every
headless assertion was green while the row was unreadable. That is the
failure mode
`D:\dev\rag\egui\headless_trace_asserts_reached_not_visible_a_clipped_widget_needs_a_pixel_oracle.md`
records, reproduced exactly.

The full sentences survive as the `font_reason_*` copy inside the
disclosed row, where there is width for them. Nothing was cut, only
moved.

Field order on the row is **verdict, size, name last** — deliberately, so
that if a very long `/BaseFont` overflows anyway, what clips is the one
field recoverable from somewhere else ([`font_full_name_tooltip`] carries
the full name). See [`font_row_header`].

## Every verdict is drawn at the SAME visual weight

As a plain label, never as a button and never in an error colour. Two
reasons, and both are requirements rather than taste:

- A blocked verdict is a fact about the **file**, not a pdfcer failure, so
  it must not carry error styling.
- There is no removal control here. A "safe" verdict rendered as a
  checkmark or an accent colour reads as an invitation to click something
  that does not exist, which is worse than saying nothing.

The wording is non-agentive throughout — no "cannot", no "failed" — for
the same reason.

## Item notes

### `fn every_verdict_word_stays_short_enough_for_a_narrow_dock`

This is the clipped-row incident turned into an assertion. The row is
`verdict · size · name`, a dock pane is ~370 pt, and the byte size —
the field the panel exists for — is the one that got cut when the
verdicts were sentences. Twenty characters is a generous ceiling that
still catches a sentence.

A width in characters is a proxy for a width in points, and a poor
one. It is used anyway because the honest measurement needs a live
frame and a pixel oracle, and a proxy that fires on the regression
that actually happened beats no check at all — the incident was
sentences of 30 and 34 characters replacing words of 10 and 17.

### `fn no_two_verdicts_say_the_same_thing`

Two verdicts reading alike would collapse two different facts about a
file into one, and the whole panel is an argument that the difference
between them is what the operator needs.

### `fn an_unreadable_fstype_says_unknown_and_is_never_blank`

The single most dangerous confusion in this file: `0` means
Installable — the *broadest* embedding right the field can express —
so a blank or a dash for "we could not read the bits" would assert
that right on the strength of bytes nobody read. Both failure states
say the word "Unknown" in their own sentence, and neither is empty.

### `fn every_fstype_permission_attributes_the_claim_to_the_vendors_bits`

Claim-bearing copy. The OpenType specification is explicit that
`fsType` is the vendor's machine-readable assertion of intent and not
the licence, and that a face may permit more or less than its bits
say. Every permission sentence must therefore attribute the claim to
the bits, and none may use the word "licence"/"license" as a verb
about what is permitted.

`Restricted License` is the `fsType` value's own proper name, so the
check is on the attribution clause rather than on the word.

### `fn the_identity_reason_distinguishes_having_a_tounicode_map_from_not`

Two independently-bad outcomes stack: the text cannot be drawn, and
without `/ToUnicode` it cannot be recovered either. Saying the second
about a font that does carry the map would be a false alarm on the
panel whose entire credibility rests on its refusals being accurate.

### `fn the_row_header_ends_with_the_name_so_the_size_cannot_clip`

This pins the *overflow decision*: whatever clips must be the field
recoverable from elsewhere. A future reordering that reads better in
isolation would silently reintroduce the clipped byte size.
