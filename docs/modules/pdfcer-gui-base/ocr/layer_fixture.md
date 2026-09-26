# `ocr::layer_fixture` — what the engine makes of `fixtures/ocr-layer.pdf`

`fixtures/ocr-layer.pdf` is written by `tools/gen-ocr-layer-fixture.py`,
which carries the argument for the document's shape. This module carries the
**numbers** — measured out of the engine, not computed by hand — and pins
them, so that a change in either the fixture or the extractor is a red test
rather than a check that quietly starts asserting something else.

## Why the fixture exists

The overlay's selector is `ExtractedGlyph::invisible`, which the engine sets
from the ambient text rendering mode. Before this fixture there was nothing
in `fixtures/` with a mode-3 run on it, so the selector had no oracle:
`synthetic-image-only.pdf` has no text at all, and producing an OCR layer
from it costs the model weights, which are not in this repository.

## The three controls, and the wrong build each one catches

| control | where | a build that fails it |
|---|---|---|
| a visible run on the same page | `VISIBLE CONTROL STAMP` | reports every glyph invisible |
| a hidden run outside the OCR stream | ladder, first run | decides invisibility by content stream |
| a visible run after a hidden one, same `BT`…`ET` | ladder, second run | treats mode 3 as latching to `ET` |

The second and third are the ones the overlay turns on. A shell that
reimplements the mode judgement from the content stream — rather than
reading the flag the extractor already sets — gets the ladder wrong, and the
ladder is the only part of the page where getting it wrong is visible.

## Two instruments, not one

The counts are asserted twice: once by filtering the glyphs, and once
against `TextDiagnostics::invisible_glyphs`, which the extractor tallies on
its own path. They are the same claim reached two ways, so a change that
moved one and not the other is a finding rather than a silent agreement.

## Item notes

### `fn the_fixture_splits_into_the_invisible_and_visible_counts_it_claims`

Both numbers, not one: a build that reported everything invisible passes the
first assertion alone, and a build that reported nothing invisible passes
the second alone.

### `fn the_engines_own_diagnostic_counts_the_same_invisible_glyphs`

A second instrument on the same claim. `invisible_glyphs` is counted on the
extractor's own path, so agreement here is evidence and disagreement is a
finding about the engine rather than about this fixture.

### `fn rendering_mode_is_ambient_and_the_ladder_proves_both_directions`

A hidden run in a stream that is not the OCR layer, followed inside the same
`BT`…`ET` by a visible one. Asserted by reading the text back off the glyphs
rather than by position, because a position assertion would pass on a build
that had the two runs' flags swapped.

### `fn the_invisible_layer_reaches_no_pixel_of_the_raster`

This is the property the whole overlay rests on: the operator sees the
invisible layer only because the shell draws it, never because the
rasterizer does. If mode-3 text reached the pixmap, the overlay would be a
second rendering of content the page already shows, the slider would be
compositing a picture with itself, and R8b's "applied content renders
exactly as saved content will render" would be false of every OCR'd page.

Measured against a control built from the fixture's own bytes: the page's
`/Contents` array is rewritten to drop the OCR stream, padded to the same
length so every offset in the cross-reference table stays right. The two
rasters must be identical, pixel for pixel.

The control also guards the assertion from the other side. A build that
rasterized nothing at all would satisfy "identical", so the ink count is
asserted non-zero first: the page has a picture on it and a visible stamp,
and both must be in the pixmap for the comparison to mean anything.

### `fn the_horizontal_scale_reaches_the_advances_rather_than_being_dropped`

The engine sets a per-word horizontal scale so the glyphs span the
recognised box. A reader that drops `Tz` gets every OCR run's extent wrong
without getting anything visibly wrong, which is the failure this catches.

The fixture writes `SCANNED` twice — same text, same size, `Tz` 100 on the
upper baseline and 86.4 on the lower — so the ratio below has exactly one
cause. Comparing two *different* words would confound the scale with the
letters.
