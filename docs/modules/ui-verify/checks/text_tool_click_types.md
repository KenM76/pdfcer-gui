# `ui-verify/checks/text_tool_click_types`

`a_text_tool_click_on_text_types_there` — in Edit mode, a click on page text
with the Text tool leaves a caret that takes the next keystroke.

The Text tool sweeps text with a drag and places a caret with a click
(`canvas::clicking`). Only the caret tool paints and types into a draft, and
`app::frame` settles any draft whose tool is not armed, so a caret placed by
the Text tool alone was opened and settled away in the same frame. The click
now arms the caret tool, as a double-click on text does.

The fixture is `fixtures/subset-font-floor.pdf` (one run, `ABC`). The window is
placed off the desktop and driven only through `ScriptedPointer`, with
`PDFCER_DIAG_INVOKE=mode.edit,view.tool_text`, so the check runs under
`--no-input`.

## Steps

1. **The caret stays.** A click on the run. A `text-edit-caret` line must be
   written, and no `text-edit-abandon` after it, twenty frames on.
2. **It types.** `End`, then `A`. The canvas's last `text-edit-typing` line
   must report `draft=true` and `len=4`.

## Falsification

Without the arming in `canvas::clicking`, step 1 fails: `text-edit-abandon`
follows the caret in the same frame.
