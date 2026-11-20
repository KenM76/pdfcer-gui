# `ui-verify/checks/spanned_edit`

`a_line_of_one_letter_operators_edits_as_one`: a line a producer wrote one
letter per show operator takes a whole-line edit as one run. The commit says
so off-canvas and undoes in one step.

`EditSession::edit_text` matches a `find` across consecutive show operators
and reports how many it ran across in `EditReport::operators_spanned`.
`canvas::textedit::report::trace_target` puts that count on the
`edit-text-target` line. The operator reads it as the engine's `span:`
disclosure, which `app::actions::textcommit` passes through with the
report's other disclosures to the status line.

# What it drives

`fixtures/per-glyph-operators.pdf` (ignores `--pdf`): `ABC` at (72, 700),
12 pt Helvetica, one `Tj` per letter.

1. Caret into the `A` at (76, 704), Home, Shift+End, type `DEF`, Escape.
   Owed: `edit-text-target ... operators_spanned=N` with N > 1, and an
   `edit-text` line whose disclosures hold `span: the text was written
   across`.
2. Ctrl+Z. Owed: an `undo` line with `undo_depth=1`.

The whole line is replaced because the engine trims an edit to the part that
differs: appending one letter touches only the last operator and spans one.

# Falsified

- `textcommit` dropping disclosures that begin `span: the text was` → FAIL:
  the commit's disclosures do not carry the sentence.
- Appending `D` at End instead of replacing the line → FAIL:
  `operators_spanned=1`. This is why step 1 selects the line.

# What it does not prove

A run the same page holds twice keeps its pin and cannot span; that refusal's
sentence is `EditRefusal::SplitAcrossPieces`, proved by the unit tests in
`canvas::textedit::glyphwall`.

# Driven off-screen

Scripted pointer and keys, window off the desktop; it runs under
`--no-input`.
