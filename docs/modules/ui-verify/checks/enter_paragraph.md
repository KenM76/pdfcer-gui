# `ui-verify/checks/enter_paragraph`

`enter_breaks_a_paragraph_on_the_page` — a click on a line of a paragraph
already on the page opens the paragraph, and Enter at the line's end breaks it
there.

The fixture is `fixtures/paragraph.pdf` (six lines, one font, one text object),
copied to the check's output folder so `Ctrl+S` saves the copy. The window is
off the desktop and driven only through `ScriptedPointer`, with
`PDFCER_DIAG_INVOKE=mode.edit,edit.text`, so the check runs under `--no-input`.

## Steps

A click on the third line at (120, 668), `End`, `Enter`.

1. **The paragraph opens on the click.** `text-edit-widened` carries `len=` the
   six lines with one character between each, a space or a kept break
   (`promote::joint`), so a draft that kept only the line fails. The next
   `text-edit-typing` length is one more: the break went in.
2. **It commits and saves.** `Ctrl+S` traces `edit-block-text-applied ...
   paragraphs=` the widened `breaks=` plus two, and `save-in-place
   outcome=ok`.
3. **The file breaks there.** The bytes the save appended hold a string ending
   `often)` and one starting `(than`, and never `often than` on one line.

## Falsification

With `promote::widen` not called, step 1 fails: the click opens one line. With `blocktext::commit` sending the text with its line
breaks replaced by spaces, step 3 fails.
