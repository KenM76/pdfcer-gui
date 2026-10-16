# `a_select_click_on_recognised_text_takes_its_paragraph`

**Defect it guards.** O295 item 2: in an OCR layer a click reaches the line or
paragraph, not only the word. The engine writes each recognised block as one
`BT…ET` (`ocr::layer_content`), so a Select-tool click on any of its words must
select that one object: not a word, and not the scan picture beneath it.

**Fixture.** `fixtures/ocr-layer.pdf` (`tools/gen-ocr-layer-fixture.py`): a
full-page picture (object 0), a visible stamp, and an invisible layer of three
lines in one `BT…ET`.

**Steps.**

1. With Edit armed (Select tool), click the scan away from any text: the
   control, `first=object:0`.
2. Click `SITE` (line 1), `SHEET` (line 2) and `SCANNED` (line 3), with a scan
   click between each because the `canvas-selection` line is de-duplicated.
   Each must give `sel=1` and the same `first=object:N`, and N must differ
   from the scan's.

**Falsified** twice on the check: one word pointed at the visible stamp fails
the same-object comparison (`object:1`); one pointed at bare scan fails with no
`canvas-selection` line, because the de-duplicated trace repeats the control's.

**What it does not prove.**

- The text tool's half: a click there opens the recognised line as a run
  draft and declines the paragraph (`promote::widen`, `recognised-word`),
  so an OCR paragraph is not edited whole.
- A layer from an older pdfcer, written one block for the whole page or one
  operator per word.
