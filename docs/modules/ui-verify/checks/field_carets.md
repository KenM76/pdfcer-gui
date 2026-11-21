# `ui-verify/checks/field_carets`

`a_field_editor_sets_text_where_the_field_draws_it` — **while typing into a
form field, the text sits where the field's own appearance draws it once
committed, and a click places the caret on the character it lands on.**

# The fixture

`fixtures/field-carets.pdf` (`field-carets.PROVENANCE.py`): three empty text
fields whose layout differs from a left-set line at the box height —
`Comb` (Comb, `/MaxLen 6`, Helvetica 18, 50 pt cells), `Right` (`/Q 2`,
Helvetica 12) and `Para` (Multiline, Helvetica 14, wraps).

# The assertions

The document opens in Review, where a click fills; the pointer and keys are
scripted.

1. **Same place before and after the commit.** Per field: click near its
   bottom-right corner, type, screenshot; click empty page, await
   `form-fill-text`, screenshot. The bounding box of dark pixels (luma < 110)
   inside the box less 1.5 pt must agree between the two pictures within
   max(4 px, 6 % of the box's smaller side) on every edge.
2. **A click places the caret.** Click the left part of the comb's first cell,
   type `-`, commit. Cell 1 must hold the short hyphen (ink height at most
   half of cell 2's), cell 2 the `A`: a caret left at the end would have
   written `ABCD-`.

# Falsification

- The editor in egui's own layout (the code before `canvas::forms::caret`):
  FAIL at 1, Comb — `ABCD` packed left while the saved comb spreads it.
- `seat` skipped, so the caret lands at the end: FAIL at 2.
