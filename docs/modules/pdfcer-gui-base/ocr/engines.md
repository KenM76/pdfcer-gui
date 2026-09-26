# `pdfcer-gui-base/ocr/engines`

## Item notes

### `fn reports_confidence`

The value its `OcrEngine::reports_confidence` returns, stated per
engine so the dialog can word its disclosure before a model is loaded.
Each page is stamped from the LOADED engine instead, and `recognise`
debug-asserts the two agree.
