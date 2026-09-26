# `text::export_image` — the words the Export-image window shows, and the
sentences an image export owes afterwards

## Why this is a NEW catalog module rather than lines added to an old one

Two reasons, and the second is the load-bearing one.

The small one is R2: [`crate::text::commands`] sits at the ceiling and
[`crate::text::status`] is close to it, so a feature's worth of copy had
nowhere to go in either.

The real one is that this catalog has a **subject**, and it is not "another
export". `crate::text::export_dxf` is about a *scale somebody can defend*;
`crate::text::export_form` is about a *spreadsheet that must not execute the
values it was handed*. This one is about **what a picture format can and
cannot hold**, and every sentence in it is a consequence of that: JPEG has
no alpha channel, PNG has one and a `pHYs` chunk that decides how large the
page lands in Word, and SVG has geometry but no text.

## The sentence this whole catalog exists for


> *"note that there had better be full support (including transparency
> where supported!)"*

**The parenthesis is the instruction.** *Where supported* is an admission
that one of the four formats cannot do it, and what it asks for is that
pdfcer say which — not that pdfcer quietly pick a background and hand back
a file that looks right on screen and prints with a white box round the
drawing.

The engine's note says the same thing in the imperative: *"refuse a
'transparent' JPEG **by name** in your UI, never flatten silently."* So
[`jpeg_has_no_alpha`] is drawn beside the control that would offer the
impossible combination, and [`transparent_jpeg_refused`] is what an export
that somehow reached the writer with both set says instead of writing
anything. Two sentences for one rule is deliberate: the first is a
**prevention** and lives in the window, the second is a **refusal** and
lives in the receipt, and a build that lost the window would still not
flatten anybody's page without saying so.

## Rule 4's shape: the disclosure is off-canvas, after the fact

Nothing here is drawn on the page or on the preview. Everything in the
second half of this file is a line for the disclosure slot —
`app::actions::record_edit_disclosure` — which is where an export's
consequences are already reported (`export_dxf::exported`,
`export_form::neutralised`). An operator sees the picture they asked for,
and then reads what could not be expressed exactly.

**The one that is easiest to leave out is [`svg_text_is_outlines`], and it
is the one most owed.** `pdfcer-render`'s SVG writer says it in its own
header — *"Text is glyph outlines"* — and the consequence is invisible in
every way an operator would check: the SVG opens in Inkscape, the words are
there, they are the right shape, and they cannot be selected, searched,
re-typed or re-flowed, and they do not carry the font. That is exactly the
inference Rule 4 exists for: *a change pdfcer made that the operator cannot
see and would not guess.*

## Rule 15

No sentence here says "dimension". Nothing in an image export reads the
**ce dimensions** the operator has drawn or the **pdf dimensions** a CAD
exporter left on the page — a raster is what the renderer paints and an SVG
is what the recorder recorded, and neither consults the dimensioning model
at all. That is a real difference from `export_dxf`, whose whole window is
arranged round the ce dimensions, and it is why this catalog offers no
scale control: **a picture has no scale to get wrong.** It has a
resolution, which is a different claim, and [`dpi_hint`] says which.

## Item notes

### `fn a_transparent_jpeg_is_refused_by_name`

The assertion this catalog exists for, and the operator's own
parenthesis — *"(including transparency where supported!)"* — is what
makes it a requirement rather than a nicety. The engine's note is
imperative: *"refuse a 'transparent' JPEG by name in your UI, never
flatten silently."*

Both halves are asserted, because they are two different mechanisms:
the window's prevention and the writer's refusal. A build that lost one
must still not flatten anybody's page in silence.

### `fn a_raster_receipt_says_the_resolution_is_in_the_file`

The engine's note names the defect: *"without `pHYs` Word places a 300
DPI page four times too large."* An operator who is told only "300 DPI"
learns what was asked for; one who is told it is recorded in the file
learns that the paste will be the right size.

### `fn an_exact_svg_still_discloses_that_text_became_outlines`

The trap this asserts against is a disclosure that only lists counted
things. Text-as-outlines has no counter and never will, so a tally-only
implementation would go completely silent on the single largest
surprise the format has.

### `fn a_gradient_that_went_out_natively_is_not_reported_as_a_loss`

`ExportTally::is_exact` zeroes `shadings_as_gradients` before comparing,
on the engine's own reasoning — *"a native gradient is exact; it is
counted, not confessed"* — and this catalog must agree with it. A
disclosure that listed the good outcome beside the bad ones would train
the operator to skim, which is how a disclosure stops working.

### `fn every_format_is_named_and_described_in_its_own_words`

The sweep that makes adding a fifth format safe. A `match` arm that
fell through to another format's wording would compile, would draw a
radio, and would describe somebody else's file — and nothing but this
test would notice.

### `fn the_metafile_radio_does_not_read_as_a_bare_acronym`

Unlike PNG, JPEG and SVG, "EMF" is a name almost no operator has met,
and a radio reading only "EMF" is a radio nobody presses. The long form
is the one the Windows *Paste Special* list itself uses.

### `fn an_exact_metafile_still_discloses_that_text_became_outlines`

`svg_fidelity`'s clause 3, restated for EMF because the always-true
loss is the same and the sentence is not: the SVG's names the SVG, and
a receipt about somebody else's format is a receipt about somebody
else's file.

### `fn a_rasterised_metafile_names_all_five_reasons_with_their_counts`

The engine's note is imperative about this: *"`EmfOutcome` carries what
became a bitmap (translucent solids, blend modes, gradients, images,
groups) — **disclose those**"*. A disclosure that gave only the total
would tell an operator that thirteen things went wrong and nothing
about which knob to turn.

### `fn an_exact_tally_does_not_make_a_rasterised_metafile_exact`

The trap `EmfCounts::is_exact` exists for. `ExportTally` describes the
recording, which is shared with the SVG writer and knows nothing about
EMF's missing per-primitive alpha — so a page that recorded perfectly
and then had forty translucent rectangles turned into bitmaps has an
**exact tally** and an **inexact metafile**.

⇒ Asking the tally alone is how a disclosure comes to say *"nothing had
to be approximated"* over a file that is half pictures.

### `fn the_libreoffice_hole_warning_names_the_program_and_the_versions`

`nonzero_fills_multi_subpath` is the only entry in the EMF disclosure
that is not a loss — the metafile records the fill rule correctly. It
is a warning about one named reader, and that reader is the entire
reason this format is offered. The failure it predicts (a solid shape
drawn with holes) is the kind an operator blames on pdfcer.

### `fn the_shared_recording_losses_read_the_same_in_both_formats`

They are the same losses arriving by the same route — the shared export
recording — and an operator who has read one receipt should recognise
the other. Divergent wording for an identical fact reads as two
different problems.

### `fn the_metafile_receipt_reports_what_stayed_lines_and_what_did_not`

An SVG's `ops` is the whole file; a metafile's is only the part that
stayed lines. A receipt reporting `ops` alone would describe a file
that is half `EMR_ALPHABLEND` in exactly the same words as one that is
all geometry.

### `fn each_vector_format_confesses_its_outlines_in_its_own_name`

They describe the same loss and must not be the same string: a receipt
that told an operator about "an SVG" after they exported a metafile is
a receipt about a file they do not have.
