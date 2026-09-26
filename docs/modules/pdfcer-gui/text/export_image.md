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

## ★★★ The sentence this whole catalog exists for


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

## ★★ Rule 4's shape: the disclosure is off-canvas, after the fact

Nothing here is drawn on the page or on the preview. Everything in the
second half of this file is a line for the disclosure slot —
`app::actions::record_edit_disclosure` — which is where an export's
consequences are already reported (`export_dxf::exported`,
`export_form::neutralised`). An operator sees the picture they asked for,
and then reads what could not be expressed exactly.

★ **The one that is easiest to leave out is [`svg_text_is_outlines`], and it
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
