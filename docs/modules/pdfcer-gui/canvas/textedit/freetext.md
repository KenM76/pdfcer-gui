# `canvas::textedit::freetext` — a markup text box typed on the page

A drag with Markup ▸ Text box on an upright page opens a `Draft` anchored by
`Anchor::FreeText` (the dragged rectangle, page points). It shares the caret,
selection, keys and IME of every other draft; only its picture differs.

## The picture is the engine's

Each frame the draft's words go through `wordmarkup::spec` (the builder the
commit uses) and `annot_author::free_text_preview`; the appearance that comes
back is painted by `edit_preview::paint_text_annotation_preview` into a
texture laid under the caret. Nothing is drawn by the shell except the caret,
the selection and the accent outline, so the box being typed in cannot differ
from the box saved. An empty draft is measured with a probe letter and drawn
as outline and caret only.

## Rectangle

The box grows downward when the layout needs more height than was dragged
(`rect_grown_downward`); `commit_rect` gives the commit the same grown
rectangle. Width never grows: words wider than the box are clipped by the
appearance, and the commit says so on the status line.

## Offsets

The engine lays out the trimmed text and reports byte offsets; the draft
holds char indices into the untrimmed text. `stops` maps one to the other and
is where a mismatch would put the caret in the wrong place.

## Not handled

A rotated page (the commit turns the annotation upright, which the preview
does not model) and every other text-annotation kind keep the dialog.
Retyping an existing box is not routed here.
