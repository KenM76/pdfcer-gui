# `pdfcer-gui::app::dispatch::snapshotsave`

`view.snapshot_save_pdf`, *Save as PDF…* on the snapshot box's right-click
menu (O284). Writes what the box holds as a one-page PDF: the engine cuts the
region out of its page, so nothing outside the box is in the file, and the
page is the box's size. The document is not changed.

## Contract

- `save(app)`:
  1. cuts the box with `clipboard::snapshot::snapshot_pdf`, which calls the
     engine's `pageops::extract_region` on the session's view (unsaved edits
     included) with the viewer's annotation setting and hidden layers, exactly
     as the snapshot copy does;
  2. asks where through `files::pick_save_path`, suggesting
     `<stem>-snapshot.pdf` beside the document;
  3. refuses a target equal to the document's own path;
  4. writes the bytes.
- The cut runs before the dialog, so a refusal never follows a choice of file.
- Trace on success: `snapshot-save-pdf w= h= bytes= glyphs_removed= residual=
  path=`, where `w` and `h` are the saved page's size in points (the engine's
  `RegionReport::rect`) and `residual` is 1 when the engine reports content
  outside the box it could not cut away.
- Trace on refusal: `snapshot-save-refused reason=` with `no-box`, `engine`,
  `over-document` or `write`. `snapshot-save-cancelled` when the dialog is
  dismissed.
- Status sentence, `text::snapshot::saved_pdf`: the path, the page size, the
  characters crossing the edge that were left out, and whether everything
  outside the box was cut away; the engine's notes follow. Residual content
  sits outside the saved page's MediaBox, so it is invisible on the page and
  still in the file, and the sentence says so (R8b).

## Reach

Only the box's menu, `canvas.snapshot`. Its operand is the box the pointer is
inside, so it has no ribbon home; it is registered in `TAB_SCOPED`. The menu
row is enabled whenever pages exist; a press with no box (by a route that
never consults the menu) is refused with `no-box`.

Driven by `a_snapshot_saves_as_a_one_page_pdf`.
