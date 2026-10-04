# `vector-art.svg` and `vector-art.emf` — two drawings for Insert image

`vector-art.svg`, written by hand: 200 x 100 px (150 x 75 pt at 96 px per
inch), a black stroked rectangle, a red diagonal line and one `<text>`
element. The text is there on purpose: `pdfcer_core::svg_import` does not
carry `<text>` and says so in its notes, which is the disclosure the check
reads.

`vector-art.emf`, 340 bytes: written by the engine's own CLI as
`pdfcer1 export-image fixtures/pure-k-square.pdf --format emf`, so it is a
metafile `pdfcer_core::emf_import` is known to read.

Used by `tools/ui-verify/src/checks/insert_drawing.rs`: each is picked
through `PDFCER_DIAG_IMAGE_PATH` and placed by Insert image.
