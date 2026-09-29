# `ui-verify/checks/remove_ocr`

`remove_ocr` — **File ▸ Recognise ▸ Remove OCR text takes off every layer
pdfcer's OCR wrote, leaves a look-alike from other software, and a second
press says there is nothing left.**

## What it guards

`file.remove_ocr` dispatches the action that runs
`EditSession::find_ocr_layers` and `EditSession::remove_ocr_layer` for each
layer found, folded into one undo entry through the edit funnel. The engine
counts a stream as a layer only when the whole stream is one
`/pdfc_OCR … BDC … EMC` whose `/Producer` is `pdfcer`. Text written by other
software is not touched.

## How it drives

1. It opens `fixtures/ocr-layers.pdf` (built by its `PROVENANCE.py`). The file
   has two pages, each with visible text and one pdfcer layer. Page 2 also
   has a decoy stream with the same tag and another `/Producer`.
2. It selects Read mode, clicks `ribbon.tab.file`, and finds
   `ribbon.item.file.remove_ocr` on the band or in a collapsed group.
3. It presses it and requires `remove-ocr-layers-applied removed=2 pages=2`.
   A count of 3 means the decoy went too.
4. It presses again and requires no second `applied` line and a
   `remove-ocr-layers-refused` line whose detail is
   `no pdfcer OCR layer in the document`.

## Not covered

The other half of the capability, where a second recognition replaces the
layer instead of stacking another, needs a real OCR run. It is not driven here.

## Falsification

Making the removal loop stop after its first layer fails step 3 with
`removed=1 pages=1`.
