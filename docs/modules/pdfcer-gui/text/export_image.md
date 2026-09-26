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

### `fn intro`

Says what leaves and what does not, in the order an operator meets it: a
picture of the page, one file per page, and — the part they cannot check
afterwards — that pdfcer will say what it could not carry exactly.

### `fn format_hint`

Not decoration. The operator's own examples were *"copy and paste vector
graphics into word or inkscape"*, and the choice between these three is
exactly the choice that decides whether that works — so the line says what
the receiving program will be able to do with the file, which is the
question being asked, rather than restating the acronym.

### `fn pages_range_invalid`

Drawn where the range is typed, not saved for the receipt: this is a
mistake the operator can fix in the box in front of them, and a refusal
that arrives after a save dialog has been answered is a refusal that wasted
their time.

### `fn dpi_hint`

For PNG and JPEG it decides the size of the picture and is written **into**
the file, which is the whole of why Word places the result correctly. For
SVG the geometry is exact at any value and the number only governs anything
that had to be embedded as a picture — a shading with no gradient form, a
soft mask, an image the PDF already carried.

Saying the same thing for both would be wrong in one of the two cases, and
the case it would be wrong in is the one where an operator wonders why
raising the number did nothing to the sharpness of their lines.

### `fn dpi_pixels`

A resolution is an abstraction; a number of pixels is the thing that
actually lands on disk and the thing that can be refused. Shown live so the
operator who types 1200 finds out here rather than from a failure.

### `fn dpi_too_large`

`pdfcer_render::MAX_PIXMAP_EDGE` is 16,384 pixels and a render past it is
refused by the engine. Said here, in the window, with the number that would
have to come down — because the alternative is a save dialog followed by a
failure the operator cannot act on.

### `fn jpeg_has_no_alpha`

The operator asked for *"full support (including transparency where
supported!)"*, and the engine's note is imperative about the other half:
*"refuse a 'transparent' JPEG by name in your UI, never flatten silently."*

So the checkbox is drawn for JPEG and drawn **disabled**, with this under
it. Three separable claims, and each is there for a reason:

1. **It is the format, not pdfcer.** *"has no way to store one"* — no
   future version of pdfcer changes that, and a sentence that read like a
   gap somebody might fix would be a promise.
2. **What would happen instead**, said before it happens: a solid
   background. This is the fact an operator otherwise discovers in print.
3. **What to do about it** — the two formats that can, by name.

### `fn quality_hint`

A CAD drawing is line art, and line art is the content JPEG treats worst.
The sentence names that rather than talking about compression ratios,
because the decision being made is *"can I send this to somebody"* rather
than *"how does the codec work"*.

### `fn multi_page_naming`

One file per page, and the operator names one of them. Every reference
application does this — Acrobat's *Export ▸ Image* asks for a base name and
writes `Base_Page_1.png` — but "the name you type is a stem" is not
something a save dialog can say, so the window says it here, while the
operator can still change their mind about the range.

### `fn wrote_raster`

Names the file, the page, the pixels and the resolution **that is written
into the file**. The last of those is the one the engine's note singles
out: *"without `pHYs` Word places a 300 DPI page four times too large"*, so
the receipt states that the number went in rather than leaving the operator
to discover on paste whether it did.

### `fn wrote_emf`

Its own line rather than [`wrote_svg`]'s, and the difference is the
second number. An SVG's `ops` count is the whole file; a metafile's is
only the part that stayed geometry, and the balance became bitmaps. A
receipt that reported `ops` alone would describe a file that is half
pictures in exactly the same words as one that is all lines.

⇒ So both are named, side by side, and the reader can see the ratio
without doing arithmetic. `emf_fidelity` then says *what* the pictures
were.

### `fn transparent_jpeg_refused`

The second half of the rule [`jpeg_has_no_alpha`] states. That one prevents
the combination in the window; this one is what the writer says if it is
ever handed the combination anyway — a keyboard route, a restored plan, a
later build with a different window.

It refuses rather than flattening, and that is the whole point. Flattening
would produce a file the operator can open, that looks almost right, and
whose white background they meet when it is already in somebody else's
document. A refusal costs them one press and tells them which press to make
instead.

### `fn refused`

One arm today. It is a `match` rather than a bare call so that the day a
second impossibility is named in
[`crate::app::actions::imageexport::Impossible`], the compiler asks for its
sentence rather than letting it inherit this one.

### `fn svg_fidelity`

`ExportTally` is the engine's own count of what a recording had to
approximate, and every field below is a fact the operator cannot get any
other way: the file opens, the picture looks right, and the thing that was
approximated is invisible until somebody prints it on a press or edits it
in Inkscape.

# Four decisions in this function, each of which could have been made
wrongly and quietly

1. **`shadings_as_gradients` is not a shortfall and is not reported as
   one.** The engine's own doc calls it *"a count of fidelity, not
   shortfall"*, and `ExportTally::is_exact` deliberately zeroes it before
   comparing. A gradient that went out as a real `<linearGradient>` is the
   good outcome; listing it beside the losses would teach the operator to
   skim the list, which is how a disclosure stops being read.
2. **`soft_masks_kept` is likewise fidelity** — the mask survived as a
   `<mask>` element — but it is still mentioned, because `is_exact()`
   counts it as inexact and because Word's SVG importer is where it will
   fail. The sentence says *kept*, and then says *where it will not be
   honoured*, which is a different and more useful claim than "lost".
3. **The always-true one is stated unconditionally.** Text is glyph
   outlines in every SVG this writer produces, so there is no counter for
   it and there never will be; a disclosure that only listed *counted*
   things would silently omit the largest single surprise in the format.
4. **An exact page says so, in one line.** The alternative — saying nothing
   — is indistinguishable from a build where the disclosure broke.

`dashed` and `blends` are passed separately because they live on
`SvgOutcome` rather than on the tally; the engine's note put them in the
tally's paragraph and the source does not, and the source wins.

### `fn svg_text_is_outlines`

`pdfcer-render`'s SVG writer states it in its own header — *"Text is glyph
outlines. That is what 'renders identically everywhere' costs"* — and the
consequence is invisible to every check an operator would make. The file
opens. The words are there. They are the right shape and in the right
place. And they cannot be selected, searched, corrected or re-flowed, and
the font did not travel.

So the sentence names all four consequences rather than the mechanism: an
operator who reads "text is converted to paths" learns nothing they can
act on, and an operator who reads "you will not be able to edit the words"
knows immediately whether it matters to them.

### `fn emf_fidelity`

The engine's note names the obligation in one clause: *"`EmfOutcome`
carries what became a bitmap (translucent solids, blend modes, gradients,
images, groups) — **disclose those**"*. This is that.

# Why this is not `svg_fidelity` with a different noun

The two formats fail at different places, and folding them into one
function would mean each page's disclosure was worded for the other one.

| | SVG | EMF |
|---|---|---|
| a translucent solid | `fill-opacity`, exact | **a bitmap** |
| a gradient | `<linearGradient>`, exact | **a bitmap** |
| an image the PDF carried | embedded, exact | a bitmap (same pixels, but counted) |
| a blend mode | `mix-blend-mode`, Inkscape honours it | **dropped, and the element rasterised** |
| a nonzero fill with several subpaths | `fill-rule`, exact | exact — but **LibreOffice 24.x ignores it** |

The right-hand column is why an operator would choose EMF and what it
costs them, and none of it is visible in the file: a metafile that is half
`EMR_ALPHABLEND` opens, plays, and looks correct at 100%.

# The five reasons are given as a breakdown of one total, not five
sentences

`rasters_embedded` is the engine's own sum and the number that matters —
*how much of my drawing stopped being lines*. The five causes are the
diagnosis, and a reader who does not need it should be able to stop after
the first clause. Five separate sentences would bury the total in the
middle of them.

# The LibreOffice clause is unconditional on its counter and is NOT
folded in with the rest

`nonzero_fills_multi_subpath` is the only entry here that is not a loss at
all — the metafile records the fill rule correctly. It is a warning about
**one named reader**, which is the very reader this format exists to
serve, and the failure it predicts (a solid shape drawn with holes in it)
is the kind an operator blames on pdfcer. So it gets its own sentence, it
names the program and the version, and it says what will look wrong.

### `fn emf_text_is_outlines`

Worded separately rather than sharing the SVG's sentence, because the
SVG's names the format — *"Text in an SVG"* — and a receipt that told an
operator about their SVG after they exported an EMF is a receipt about
somebody else's file. The consequences are identical and are listed in the
same order, deliberately, so the two read as the same fact about two
formats rather than as two different problems.
