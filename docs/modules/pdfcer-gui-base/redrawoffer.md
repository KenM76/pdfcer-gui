# `redrawoffer` — a refused restyle, held for the redraw button

`set_text_annot_style` and `set_marker_style` refuse a mark whose look pdfcer
cannot redraw faithfully, and accept the same call with `redraw_as_plain`
set. This module holds the refusal and the call so the Properties panel can
offer the redraw as an explicit choice; the default stays "leave another
program's drawing alone".

## Contract

- `RedrawOffer::of(error, id, refused)` is `Some` for exactly
  `EditError::FreeTextAppearanceForeign` (`Why::TextBoxForeign`),
  `FreeTextIsRichText` (`Why::TextBoxRich`) and `MarkerAppearanceForeign`
  (`Why::MarkerForeign`); any other refusal is not one a redraw answers.
- `retry()` is the refused call with `redraw_as_plain: true` and every other
  field unchanged.
- `Why::token()` is the stable trace token.

The action handler (`app::actions::annots::textannotstyle::settle_offer`)
stores the offer on `OpenDoc::redraw_offer` and clears it after the next
successful restyle of the same annotation.
