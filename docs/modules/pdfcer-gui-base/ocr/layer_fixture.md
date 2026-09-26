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
