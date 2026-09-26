# `text::forms` — every string the Forms panel shows

One area of the catalog described in [`crate::text`]'s header, covering
[`crate::panels::forms`] — the panel that lists an `/AcroForm`'s fields
and lets an operator **fill** them.

It sits beside [`crate::text::panels`] rather than inside it, which is a
deliberate placement rather than an oversight: the Forms panel is not one
of the six report panels that module covers, it is the first panel in this
build that **changes the document**, and its copy is dominated by
*disclosures* and *refusals* rather than by field labels. Keeping it in its
own file means the reviewer of a disclosure sentence is reading a file that
contains nothing but disclosure sentences.

## Almost every sentence here is salvaged verbatim

Carried across from the old shell's `ui_text.rs` **with its doc comments**,
because the doc comment is usually the record of a defect the wording was
changed to fix. ⚠ **Fresh words re-derive a decision already paid for and
have none of the evidence that bought it**, and this area carries more such
decisions per line than any other:

- [`forms_no_acroform`] does not say "no fields found". It says a page can
  *look* like a form without carrying one, because otherwise "no fields"
  reads as pdfcer failing to find something that is plainly on the page.
- [`form_field_password_tooltip`] exists because a masked box reads as
  "secure" to anyone not told otherwise, and the value really is stored as
  plain text in the file.
- [`forms_data_export_carries_rich_text`]'s counterpart in the old catalog
  carries a `★` recording that the sentence outlived the behaviour it
  described by one commit. That string is **not** salvaged here (this build
  has no export), and the lesson is: a disclosure that has gone stale is a
  false statement the operator has no way to check.

## The three sentences that are NEW, and why each had to be

| Function | Replaces | Why |
|---|---|---|
| [`forms_xfa_note`] | the old shell's post-fill `fill_xfa_may_disagree` | The old note appeared *after* a value was typed. Whether the document carries an XFA packet is a property of the FILE, knowable before the operator touches anything, so it is said up front. |
| [`form_field_no_on_state_note`] | the old shell's post-toggle `form_field_no_appearance_for_state` | The old one was computed from a predicate that could never be true (see [`crate::panels::forms::rows`]' header, "★ The salvaged appearance check could not fire"). This one asks a question the model can actually answer. |
| [`forms_no_fillable_fields`] | — | This build derives the "N you can fill here" count from what the panel actually offers, and the count can legitimately be zero on a form full of fields. A silent zero looks like a bug. |

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the thing and what the operator can do about it.**
- **Never state a capability the build does not have.** This build fills;
  it does not create, delete or rename a field, and no string here implies
  otherwise.
- **A warning glyph is never the only cue.** Every `⚠` sentence reads
  correctly with the glyph stripped, because a glyph is a colour-class cue
  and `RIBBON_IA.md` R84 forbids carrying state in one.
