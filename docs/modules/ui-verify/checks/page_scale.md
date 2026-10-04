# `scaling_a_sheet_scales_the_drawing`

**Defect it guards:** choosing to scale the drawing onto a new sheet size in
the Sheet size window changes only the paper, scales by a different factor
than the window quoted, or does nothing.

## What it drives

Off-screen, scripted pointer, `PDFCER_DIAG_INVOKE=mode.edit`, on
`fixtures/cropped-sheets.pdf` (page 0: 200 × 280 pt, crop box equal to the
sheet, a 180 × 260 pt stroked rectangle; page 1 a different size).

1. Pages ▸ Sheet size; read page 0's sheet and the window's drawn extent
   (`page-size-opened extent=`).
2. Pick A6, Portrait, then "Scale the drawing to fit the new sheet".
3. Apply.
4. Reopen Sheet size on the scaled document.

## Oracles

| step | requires |
|---|---|
| 2 | `page-size-scaled mode=Fit scale_min=` equal to min(A6 w / sheet w, A6 h / sheet h), 1.4882, within 0.001 |
| 3 | `page-size-commit drawing=fit size_id=a6`; no `page-size-applied` (the box-only verb) after the click; `page-scale-applied n=1` with `scale_max` equal to the quote; `page-size-sheet index=0` at A6 portrait |
| 4 | the drawn extent is the step-1 extent times the factor, within 0.5 pt (267.88 × 386.93); page 1 unchanged |

The extent is read by the survey's decomposition of the document, not by the
scale code, so a commit that traced success and left the drawing alone fails
step 4.

## Falsification

Making `PageSizeDialog::commit` push `SetPageSize` whatever the choice fails
step 3 with *"the commit ran the box-only resize"*. Before the survey skipped
paths that paint nothing, step 4 read the scale's `re W n` clip as drawing
(297.64 × 416.69) and failed with *"It changed by some other factor"*; that is
request G109.
