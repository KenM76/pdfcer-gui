# `text::export_dxf` — the words the Export-DXF window shows

## Rule 15, and it decides a sentence here

The scale this window offers is inferred from the **ce dimensions** the
operator has drawn — the ones pdfcer authors. It is *not* read from **pdf
dimensions**, the CAD-exported page content pdfcer reads and must not alter,
and it could not be: those are anonymous vector geometry with no recorded
measurement. So the copy says *"the dimensions you have drawn on it"*, which
is both unambiguous and the honest boundary — an operator who has drawn none
is told pdfcer has no evidence rather than being given a number.

## The one sentence this whole window exists for

`pdfcer-core`'s own doc on `DxfOptions::scale`:

> **This is the field the whole feature turns on.** Every generic PDF→DXF
> converter exports at paper scale and says nothing, so a **1:2 detail
> arrives at half size and looks plausible.**

*Looks plausible* is the whole problem. A DXF at the wrong scale opens
cleanly, measures consistently, and is wrong — and the person who finds out
is whoever cuts from it. Everything below is arranged so that the scale is
either stated with its evidence, or stated as unknown.

## Three answers, never two

`DxfScaleSuggestion` is deliberately not an `Option<f64>`, and this catalog
keeps its three cases apart because they ask different things of an
operator:

| | what it means | what the window does |
|---|---|---|
| `Calibrated` | every calibrated group agrees | states the number **and the group it came from** |
| `Uncalibrated` | nothing on this page carries a scale | says pdfcer has no evidence, and that 1:1 is a *choice* rather than a finding |
| `Conflicting` | calibrated groups disagree | lists every candidate and makes the operator pick |

Collapsing `Uncalibrated` into "1.0" is exactly the silent paper-scale
export the feature exists to beat.

## Item notes

### `fn an_uncalibrated_page_is_not_told_a_number`

The assertion this catalog exists for. `pdfcer-core` names the failure it
prevents — every generic converter exports at paper scale and says
nothing, so a 1:2 detail arrives at half size *looking plausible* — and
the only defence is a sentence that refuses to present a default as a
finding.

### `fn skipped_text_and_unreadable_text_are_different_sentences`

`skipped` is *you asked*; `unreadable` is *pdfcer could not read it*. The
second is a fact about the source PDF and the reason labels the operator
can see on screen are absent from the file — and rolling them together
would let it hide inside a sentence about their own choice.

### `fn a_skipped_picture_names_the_formats_limit_not_pdfcers`

*"The format has no way to carry a raster"* rather than *"images are not
supported"*: the first is a fact about DXF that no future version of
pdfcer will change, and the second reads as a gap somebody might fix.
