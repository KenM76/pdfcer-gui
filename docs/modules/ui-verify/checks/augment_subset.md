# `a_key_the_subset_lacks_comes_from_its_installed_face`

A Word subset carries only the letters the document used. A letter it never
carried is appended to the subset from the face in the font folders whose
PostScript name is the subset's, once the engine has proved they are the
same font. The line then keeps its own font; without this the shell sets the
letter in the nearest other face (`canvas::textedit::reface`).

## The drive

`fixtures/augment-subset.pdf` shows `ABC` in `ABCDEF+pdfcerAugFace`, a simple
TrueType subset that outlines only those three letters.
`fixtures/augment-fonts/face.ttf` is the full face it was cut from. The check
points `PDFCER_DIAG_FONT_DIR` at that folder, waits for the
`installed-faces-indexed` line, arms Edit Text, clicks into `ABC`, presses
End, types `D` and presses Escape.

## What it asserts

- No `refused-char` line names `'D'`.
- No `text-edit-reface-planned` line: the letter was not sent to another face.
- The `edit-text` line's disclosures say one glyph was added from `face.ttf`.
- An `edit-text-left-edge` line says `committed=yes`.

## What makes it fail

Returning the options unchanged from `installedfaces::augmenting`. The
keystroke query then has no glyph for `D`, the caret plans it into another
face, and the second assertion fails.
