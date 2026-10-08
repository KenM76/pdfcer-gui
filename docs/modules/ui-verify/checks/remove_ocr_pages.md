# `remove_ocr_text_takes_the_pages_chosen`

**Defect it guards.** O289 item 14: Remove OCR text offers no choice of
pages, or takes every page's layers off whatever was chosen.

**Fixture.** A copy of `fixtures/ocr-layers.pdf`: two pages, one pdfcer
layer on each, and a decoy written by "someone else" on page 2. Opened on
page 1 in Edit mode, off the desktop, through the scripted pointer.

**Steps.**

1. File ▸ Remove OCR text. The window must open and trace
   `remove-ocr-named layers=2 pages=2` — All pages, the decoy not counted.
2. Press `remove-ocr.scope.current` (This page), then `remove-ocr.commit`.
   The removal must trace `removed=1 pages=1`.
3. Open the window again. It must name `layers=1 pages=1`: page 2's layer is
   still there.

**Falsified** by deleting the page filter from
`app::actions::ocrlayers::remove_in`: step 2 reports `removed=2 pages=2`.

**What it does not prove.** The recogniser checkboxes: the fixture's layers
all record `ocrs`, so the window does not draw them.
