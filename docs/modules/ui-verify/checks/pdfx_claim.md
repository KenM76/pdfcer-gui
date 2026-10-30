# `document_properties_say_which_pdfx_level_the_file_claims`

Opens Document properties on `fixtures/print-shop-pdfx4.pdf` (PDF/X-4 stated
in the XMP packet only; no Info key, no output intent), then on
`fixtures/print-shop-pdfx.pdf` (a `/GTS_PDFX` output intent and no stated
level).

## Oracles

1. `pdfx-claim level="PDF/X-4" recognised=1 agree=1 intents=0` on the first
   file, and the `docprops.pdfx` region is published visibly — the row and the
   missing-intent sentence are drawn, not only read.
2. `pdfx-claim none` on the second, and no `docprops.pdfx` region: the engine
   (`pdfcer_core::prepress::pdfx_claim`) treats an output intent alone as no
   claim (ISO 15930-3 §6.2.2), and the panel must not invent one from it.

## What it cannot see

Conformance. The engine reads the claim and checks nothing; the panel's own
sentence says so.
