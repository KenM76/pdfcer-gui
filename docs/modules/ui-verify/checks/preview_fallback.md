# `ui-verify/checks/preview_fallback`

`a_stand_in_preview_is_the_texts_size_and_says_why` — when an edit to existing
text cannot be previewed in the text's own font, the stand-in is set at the
text's size times the zoom, and the status bar says why.

The fixture is `fixtures/ocr-layer.pdf`. Its run `HIDDEN RUN IN A VISIBLE
STREAM` is 10 pt Helvetica at rendering mode 3, so the engine's layout is
invisible ink and the preview must fall back to the shell's font
(`PreviewFallback::Invisible`).

The window is placed off the desktop and driven only through
`ScriptedPointer`, with `PDFCER_DIAG_INVOKE=mode.edit,edit.text` arming the
Edit Text tool, so the check runs under `--no-input`.

## Steps

1. **Open a caret and type.** A click inside the hidden run, `End`, then `_`.
   The last `text-edit-preview-fallback` line naming a reason must read
   `reason=invisible`.
2. **Size.** Its `font_pt` must be within 3 % of 10 pt times the canvas zoom
   the trace reports.
3. **Disclosure.** The status bar must have declared the region
   `status-group:preview-fallback` while the draft was open.

## Falsification

With the stand-in sized by the old rule (the glyph box's screen height clamped
to 11–40 pt, times 0.72), step 2 fails: at the default fit zoom the font is
about 0.83 of the run's size. With the status line removed, step 3 fails.
