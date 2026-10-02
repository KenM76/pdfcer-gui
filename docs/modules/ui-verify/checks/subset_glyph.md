# `a_glyph_the_subset_outlines_but_never_showed_types`

Word embeds only the glyphs a document uses, but the font program it embeds
often still outlines more. The engine can type such a glyph when the edit
carries an embedded-program reader (`EditOptions::with_embedded_glyphs`), and
its keystroke query, `EditSession::run_repertoire_with`, accepts exactly what
that edit would add. The two must be asked with the same options, or the
caret refuses a key the commit would take.

The shell builds both from `editmodel::disposition::typing`: every commit and
preview through `disposition::options`, and the keystroke query in
`canvas::textedit::repertoire`.

## The drive

`fixtures/word-shaped-subset.pdf` shows `ABC` in a simple TrueType subset with
no `/ToUnicode`, `/Widths` covering only 65..67, and a program that also
outlines `D`. The check arms Edit Text, clicks into `ABC`, presses End, types
`D` and presses Escape, which commits.

## What it asserts

- No `refused-char` line names `'D'`. That line is what the caret writes when
  the keystroke query refuses a key.
- An `edit-text-left-edge` line says `committed=yes`.

## What makes it fail

Dropping the reader from `typing()`. The query then refuses `D` at the
keystroke, and the commit would refuse it too.
