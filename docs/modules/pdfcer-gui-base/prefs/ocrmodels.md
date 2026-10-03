# `pdfcer-gui-base/prefs/ocrmodels`

**Extra folders to find OCR models in, and the model last run.**
`preferences.txt` lines:

```
ocr_folder = E:/models/extra
ocr_folder = F:/more
ocr_model = paddle-vl
```

# Contract

- `ocr_folder` repeats, one folder per line, in search order. At most
  `MAX_FOLDERS` (16); `add_folder` refuses a duplicate or a seventeenth and
  returns `false`.
- `ocr_model` is the discovery name (`OcrModel::name`) of the model last run.
  Absent means "none remembered"; `ocr_engine` then decides the start.
- An empty value is a `BadValue` note and the line is ignored.
- The bundled `models` folder is never stored: it is always searched first.

# Where it is set

Settings › OCR models (`dialogs/settings/ocr`): the list with Remove per row,
Add… (native folder picker; `PDFCER_DIAG_OCR_FOLDER` answers it for driven
checks), and the models the folders hold, discovered once per list and cached
in egui temp data because the search index draws every page. The model is set
by running Recognise text, not on this page.
