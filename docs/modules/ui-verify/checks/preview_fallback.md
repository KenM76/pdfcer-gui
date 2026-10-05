# `ui-verify/checks/preview_fallback`

`a_stand_in_preview_is_the_texts_size_and_says_why` — when an edit to existing
text cannot be previewed in the text's own font, the stand-in is set at the
text's size times the zoom, and the status bar says why.

The fixture is `fixtures/retype-seam.pdf`. Its word `Hello` is 12 pt, drawn as
`Hel` in Helvetica and `lo` in Times-Roman; the engine refuses an exact edit
across the two fonts, so the preview falls back with `reason=refused`.

The window is placed off the desktop and driven only through
`ScriptedPointer`, with `PDFCER_DIAG_INVOKE=mode.edit,edit.text` arming the
Edit Text tool, so the check runs under `--no-input`.

## Steps

1. **Open a caret and type.** A click inside `Hel`, then `_`. A
   `text-edit-preview-fallback` line naming a reason other than `none` must
   be traced.
2. **Size.** Its `font_pt` must be within 3 % of 12 pt times the canvas zoom
   the trace reports.
3. **Disclosure.** The status bar must have declared the region
   `status-group:preview-fallback` while the draft was open.

## Falsification

Scaling the stand-in's `font_pt` by 0.72 in `canvas::textedit::paint` fails
step 2 (7.23 pt against 10.04). With the status line removed, step 3 fails.
