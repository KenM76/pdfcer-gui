# `ui-verify/checks/enter_paragraph`

`enter_breaks_a_paragraph_on_the_page` — Enter at the end of a line of a
paragraph already on the page breaks the paragraph there.

The fixture is `fixtures/paragraph.pdf` (six lines, one font, one text object),
copied to the check's output folder so `Ctrl+S` saves the copy. The window is
off the desktop and driven only through `ScriptedPointer`, with
`PDFCER_DIAG_INVOKE=mode.edit,edit.text`, so the check runs under `--no-input`.

## Steps

A click on the third line at (120, 668), `End`, `Enter`.

1. **The paragraph opens.** `text-edit-promoted` carries `len=` the six lines
   joined by single spaces and `caret=` the end of the third line in that text.
   Both are computed from the fixture's lines, so a draft that kept only the
   line, or shifted the caret wrongly, fails. The next `text-edit-typing` length
   is one more: the break went in.
2. **It commits and saves.** `Ctrl+S` traces `edit-block-text-applied ...
   paragraphs=2` and `save-in-place outcome=ok`.
3. **The file breaks there.** The bytes the save appended hold a string ending
   `often)` and one starting `(than`, and never `often than` on one line.

## Falsification

With `promote::open` answering `Err(EnterRefusal::NoParagraph)` at once, step 1
fails with the decline. With `blocktext::commit` sending the text with its line
breaks replaced by spaces, step 3 fails.
