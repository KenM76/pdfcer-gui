# `pdfcer-gui/canvas/textedit/paint`

## Item notes

### `const MIN_PREVIEW_PT`

A 4 pt note at 25 % zoom is a box two pixels high, and an operator cannot
type into a line. The box grows past the run it covers rather than becoming
illegible — the alternative is a preview that technically exists.

### `fn selection`

For an existing run this is the union of its glyph boxes; for a new-text
origin it is a nominal one-line box at the click. Both are converted through
[`crate::viewer::pdf_space_to_canvas`], the inverse of the bridge
[`resolve_run`] uses, so the caret lands on the glyphs it was resolved from.
**Highlight what is selected**, one rectangle per run of characters that
share a row.

# Why it is measured character by character rather than from two
# endpoints

Because a selection can wrap. Two endpoint rectangles describe a selection
on one row and say nothing useful about one spanning three — the middle rows
are not between the two x-coordinates in any sense a painter can use, and
reconstructing them means asking the galley for its row geometry, which is
a second derivation of what `pos_from_cursor` already answers.

So each character's own slot is asked for, and adjacent slots on the same
row are merged into one rectangle. The cost is O(n) in the SELECTION's
length, per frame — and a draft is one show operator, so n is tens of
characters. That is the same trade `Draft::caret` makes for character
indices, made again for the same reason.

A character's right edge is taken from **the next slot's left edge**,
not from its own `max.x`. The two differ where a row ends: the last
character of a wrapped row has a next slot on the row BELOW, which is how
the row break is detected at all.
