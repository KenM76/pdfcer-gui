# `pdfcer-gui/app/handsigned`

Which signature boxes carry a hand signature, read from the document.

`refresh` runs once a frame, beside `OpenDoc::refresh_content_generation`. It
does nothing while `OpenDoc::hand_signed` was measured at the current
`edit_epoch`. Otherwise it reads `pdfcer_core::hand_sig::hand_signatures` on
each page holding a box in `canvas::forms::placed(..).unsigned` and stores the
field names found. Every edit, undo and redo bumps the epoch, so a signature
undone or redone is measured rather than tracked, and a file opened with one
already signed reads it on its first frame.

Only pages with an unsigned `/Sig` box are read, and the engine returns at
once from a page with no tag, so a form without signature boxes costs
nothing.

A page whose content cannot be parsed is counted in `failed=` and contributes
no signature; its boxes then read unsigned, which offers signing again rather
than hiding a box.

## Trace

- `hand-signed-read signed= pages= failed=` on change. No field names: they
  are the operator's text.
