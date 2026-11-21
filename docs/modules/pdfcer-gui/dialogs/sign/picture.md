# `pdfcer-gui/dialogs/sign/picture`

The Sign window's *Picture in the box* row, shown when the signature has a box.

# Contract

- `show(ui, theme, visible)` draws nothing when `visible` is false. Otherwise
  it draws the chooser (`sign-picture`), the chosen file's name and pixel size
  (`sign-picture-name`) with a remove button (`sign-picture-remove`), or the
  sentence saying the box shows its text only.
- `after()` runs the pick or remove the frame asked for, outside the UI
  closure. The picker answers through `files::pick_image_source`, which is
  diagnostic-answerable via `PDFCER_DIAG_IMAGE_PATH`. It traces
  `sign-picture chosen= width= height= error=`. An unreadable file keeps the
  previous choice and shows the engine's reason.
- `for_request(visible)` is the chosen picture only when the signature is
  visible, so an invisible signature never carries one.
