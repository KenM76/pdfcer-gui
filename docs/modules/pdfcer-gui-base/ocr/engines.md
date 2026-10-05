# `pdfcer-gui-base/ocr/engines`

**Which recognisers this build carries.** `EngineId` names the three
in-process engines for preferences and the legacy fixture path;
`available()` lists those compiled in. Model files, confidence and loading
are `pdfcer_ocr_host::OcrRunner`'s, not this module's.
