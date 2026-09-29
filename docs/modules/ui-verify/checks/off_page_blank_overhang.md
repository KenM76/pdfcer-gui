# `ui-verify/checks/off_page_blank_overhang`

`off_page_blank_overhang` — **the off-page census leaves out a picture that
shows nothing past the edge, lists one that does, and counts the one it left
out.**

## What it guards

The engine's off-page scan omits an image whose part beyond the page edge is
blank, as a clean leaves it, and reports how many it omitted
(`inkless_overhang`). The census window sums that count into
`blank_overhangs`, writes it on its `offpage-scanned` trace line, and states
it under the headline (`text::offpage::blank_overhang_note`). A census that
listed the cleaned picture would invite a second clean of something already
clean; one that dropped it silently would hide an inference the operator
cannot see.

## How it drives

1. It opens `fixtures/off-page-blank-overhang.pdf`: one 200 × 200 page, a
   picture blank past the right edge and a picture inked past the left edge.
   Its provenance script builds it; the engine's `scan-offpage` calibrates it
   at one partial, one blank overhang.
2. `PDFCER_DIAG_INVOKE=mode.edit,edit.offpage` opens the census window. It
   requires `offpage-opened`, then waits up to 30 settles for
   `offpage-scanned`.
3. It captures the window and requires `objects=1 blank_overhangs=1`.

## Not covered

- The note's wording is judged from the screenshot, not asserted.
- Cleaning from the window; other checks drive that.

## Falsification

Summing `0 * scan.inkless_overhang` fails step 3 with `blank_overhangs=0`.
