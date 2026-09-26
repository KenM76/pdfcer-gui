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
