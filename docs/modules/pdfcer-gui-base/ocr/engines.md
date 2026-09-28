# `pdfcer-gui-base/ocr/engines`

## Item notes

### `fn reports_confidence`

The value its `OcrEngine::reports_confidence` returns, stated per
engine so the dialog can word its disclosure before a model is loaded.
Each page is stamped from the LOADED engine instead, and `recognise`
debug-asserts the two agree.

### `enum Dictionary`

Which character dictionary a PaddleOCR run read through — a `dict.txt` file
beside the models, or the one embedded in the recognition model. Disclosed in
the dialog because a wrong dictionary misreads every character silently.
