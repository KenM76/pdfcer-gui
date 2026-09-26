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

### `fn fonts_total_size`

The equivalent of the parity reference's Audit Space Usage "Fonts"
bucket, which is a paid-tier feature there and gives no per-font
breakdown at all. Summed over DISTINCT font objects, so a font used on
four hundred pages is counted once.

### `fn fonts_coverage_note`

Not a caveat and not a footnote. An operator reading a font inventory to
decide what to delete needs the shape of the evidence, and "there is one
place pdfcer did not look" is part of the answer rather than a hedge on
it. A list that quietly missed a surface and looked complete is this
project's most-repeated defect shape.

Acrobat's own coverage here is recorded as an unconfirmed GAP, so pdfcer
states its own scope rather than assuming parity with a behaviour nobody
has measured.

### `fn fonts_page_scan_failed`

Without this, "this document has no fonts" and "pdfcer could not look"
render identically, and an operator would read the second as the first.
It goes FIRST, above everything, because it changes what an empty list
beneath it means.

### `fn fonts_all_embedded`

Deliberately NOT "ready to submit", "passes embedding checks", or
anything naming PDF/A or a print service. Those are claims about a third
party's acceptance that pdfcer has not verified. This states only what
pdfcer measured.

### `fn fonts_missing_programs`

**New at salvage.** The old panel answered this only through a control —
the embed block's "n exact, n substitute" summary, which is a statement
about a *plan*, not about the document. With no embed control here, the
document-level fact would otherwise be recoverable only by opening every
row, and it is the fact that sends an operator to a print service's
rejection notice.

It states the count and nothing more. Naming a remedy pdfcer cannot
perform in this build would be the placeholder rule broken in prose
instead of in a widget.

### `fn font_verdict_unknown`

"Unclassified" rather than the bare word "Unknown", because the `fsType`
line inside the same row independently reads as unknown for an unrelated
reason. Two bare "Unknown"s in one row look like one fact stated twice.

### `fn font_reason_blocked_identity`

Two tiers, because two independently-bad outcomes stack here and a
64-file survey found them stacking on most real files: without the
embedded program the text cannot be DRAWN, and without a `/ToUnicode` map
it cannot be RECOVERED either. The parity reference refuses these fonts
too and shows no reason at all — it simply leaves them off its list.

### `fn font_fstype_unknown`

Must never be mistaken for value 0 — which genuinely means Installable,
the most permissive value the field can express. The word "unknown" is in
the sentence itself, not carried by styling, and the sentence says pdfcer
read nothing rather than implying it read a permissive value.

### `fn font_composite_type`

`Type0 / CIDFontType2`. Both halves are shown because the parent alone
says nothing about the glyph source — the descendant is where the
outlines and the font descriptor actually live (§9.8.1).

### `fn font_size_line`

The rounded figure is for ranking two hundred rows at a glance; the exact
figure is the measurement, and this project shows the measurement. Below
1024 the two are the same number, and printing `474 B (474 bytes)` is
noise that teaches an operator to stop reading the parenthesis on the
rows where it carries information.

### `fn font_decoded_size_line`

Shown only when the two differ, which is when the program is compressed.
A line repeating the number above would be noise, and noise is how the
lines that matter get skimmed past.

### `fn font_no_pages_line`

**New at salvage.** `FontRecord::pages` being empty is NOT "unused" (the
core API map's trap T-9.4 says so explicitly: a font reached only through
the AcroForm `/DR` has no page list but is a live form-default font). The
old panel simply omitted the pages line in that case, which left an
operator to infer *"this font is on no page"* from an absence — and the
three "also used in…" lines below it are easy to miss.

Stated rather than inferred, because the inference is wrong.

### `fn font_full_name_tooltip`

The row shows the family name with the six-letter subset tag stripped,
because the tag reads as noise when scanning. But two independent subsets
of one face de-prefix to the SAME name, so back-to-back identical-looking
rows would read as a rendering fault rather than as the real and useful
fact that the document subsetted the face twice. The tag has to resurface
somewhere, and this is where.

### `fn font_row_header`

Field order is the scanning order, and it is deliberate. The verdict
leads because it is the field an operator sweeping two hundred rows is
looking for and the one no other tool shows them; the size ranks it; the
name identifies it. Putting the name first would read better in isolation
and scan worse in bulk, which is the case that matters here.

The name is the DE-PREFIXED family name — the six-letter subset tag reads
as noise at a glance. It resurfaces in [`font_full_name_tooltip`], which
is what keeps two subsets of one face from looking like a duplicated row.

**The name is LAST, and that is the overflow decision.** A dock pane is
~370 pt and a `/BaseFont` can be arbitrarily long; something has to be
allowed to clip. Putting the name last means what clips is the one field
an operator can recover from elsewhere (the tooltip), rather than the
byte size — which is what actually clipped when this row was first laid
out verdict-name-size.
