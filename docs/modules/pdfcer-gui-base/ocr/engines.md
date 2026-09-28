# `pdfcer-gui-base/ocr/engines`

## Item notes

### `fn reports_confidence`

The value its `OcrEngine::reports_confidence` returns, stated per
engine so the dialog can word its disclosure before a model is loaded.
Each page is stamped from the LOADED engine instead, and `recognise`
debug-asserts the two agree.

### `fn model_shipped`

False for PaddleOCR only: its models are the operator's to supply, so a
missing model is worded as "not included with pdfcer" rather than as a
damaged install.

### `enum Dictionary`

Which character dictionary a PaddleOCR run read through — a `dict.txt` file
beside the models, or the one embedded in the recognition model. Disclosed in
the dialog because a wrong dictionary misreads every character silently.
