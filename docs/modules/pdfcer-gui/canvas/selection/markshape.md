# `canvas::selection::markshape` — the drawn ink of a page's markups

`markup_shapes(doc, page)` returns, by annotation id, the canvas-space
segments each shaped markup draws, for the hit test in
[`super::annot::under_pointer`]. A markup with no entry claims its `/Rect`.

Shaped: Line (start to end), PolyLine (its vertices), Polygon with no
interior colour (closed), Ink (each stroke), Square with no interior colour
and no cloudy border (its `/Rect` inset by half the pen, where the stroke's
centre runs), and Circle with no interior colour (that inset rectangle's
inscribed ellipse as 48 segments, close enough that the chord error stays
inside the click tolerance).

Unshaped, so the `/Rect` stands: anything filled, a cloudy square, a revision
cloud (its scallops bulge past the vertices by an amount the standard does
not fix), and every other subtype.

The spec is read with `pdfcer_core::annot_author::spec_from_dict`; a
dictionary it cannot read keeps the rectangle.
