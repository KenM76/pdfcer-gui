# `pdfcer-gui/dialogs/settings/ocr`

**Settings › OCR models: the extra folders Recognise text searches.** One
catalogued setting (`text::settings::ocrmodels::{title, silence, radius}`).

- Rows: Remove, then the folder (truncated, full path on hover).
- Add… opens `app::files::pick_ocr_folder`; a new folder traces
  `ocr-folder-added folders=N`. Disabled at 16 with the reason on hover.
- Below, the models the extra folders hold, by label, unrunnable ones marked,
  then discovery's notes (a folder that does not exist, a shadowed name). The
  list is cached per folder list: the search index draws this page on every
  search and discovery reads the disk.
- Regions: `settings.ocr` (page), `settings.ocr.add`.

Changes apply when Save is pressed and the next time Recognise text opens.
See `docs/modules/pdfcer-gui-base/ocr/catalog.md` for search order and
runnability.
