# `pdfcer-gui/dialogs/ocr_output`

Recognise text's output choice: *Searchable image* (default, invisible words
over the untouched scan) or *Editable text and images*
(`pdfcer_core::ocr::editable::OcrOutput::EditableText` with the engine's
default options). The choice rides `Action::ApplyOcr::output` into
`app::actions::ocrlayers::options`, which passes it to
`OcrLayerOptions::with_output`. Trace: `ocr-started … output=searchable|editable`;
regions `ocr-output-searchable`, `ocr-output-editable`.

An editable page cannot be recognised again or have its text removed (its
words left the scan): the engine refuses with `EditableLayerPresent`, worded as
`OcrLayerRefusal::EditableText`; Remove OCR text leaves such layers and says so.
