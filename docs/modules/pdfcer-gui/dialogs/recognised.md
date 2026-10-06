# `dialogs::recognised` — the recognised-text choice

The Text and Word export windows offer *Include the recognised text*, *Only
the recognised text* and *Leave the recognised text out*, which become the
plan's `OcrLayerFilter` and reach the extraction as
`ExtractOptions::with_ocr_layer`. Every consumer of that extraction (reading
order, block layout, table detection, the tagged layout) sees the same subset.

## When it is drawn

Only when `layer_pages` counts at least one page: the count is the pages
`EditSession::find_ocr_layers` names, the same layers File ▸ Recognise ▸
Remove OCR text removes. A document with no OCR layer pdfcer wrote has
nothing to include or leave out, so nothing is drawn (R9). Invisible text
another tool wrote is not a pdfcer layer: the engine's filter keys on
`/pdfc_OCR` with `/Producer (pdfcer)`, so that text is page text on both
sides of the choice.

The count is taken when the window opens and frozen with the page count.

## Not remembered

The choice starts at *Include* each time. It is a statement about this
document's layer, and a remembered *Only* would export nothing from the next
document that has none.

## Disclosure (R8b)

When the choice is not *Include*, the receipt says which side was written
(`text::export_text::recognised_receipt`).

## Regions and trace

Each window passes its own `region_for_recognised`:
`export-text.recognised.all|only|without`,
`export-word.recognised.all|only|without`. The windows' `*-open` lines carry
`ocr_layer_pages=`, their `*-requested` lines `recognised=`
(`exporttext::ocr_layer_key`), and the text export's result line
`recognised=` too.
