# `pdfcer-gui/canvas/markup/tests`

## Item notes

### `fn the_three_families_partition_every_kind`

The property `canvas::interact`'s routing and
`gesture::press_kind`'s early return both rest on, asserted as a
partition rather than as three membership lists. A kind in **two**
families would be reached by two gestures — a press that both started a
band and placed a vertex — and a kind in **none** would arm a tool whose
press means nothing, which is the *"visible control, silently inert"*
failure with a crosshair on it.

### `fn an_arrow_dragged_backwards_keeps_its_head_at_the_end`

The salvaged arrow-direction decision, asserted in the direction a
normalising
implementation fails. A normalised rect would report
`start = (min, min)`, which for this drag is the **head**, so the
arrowhead would be at the tail — and, with a single head, nothing in the
document would say so.

### `fn every_rectangle_kind_is_normalised_in_all_four_drag_directions`

Asserted over all four kinds and all four directions rather than one
case, because the failure is per-kind: it is exactly the shape of
mistake that gets fixed for Rectangle and left in Ellipse.

### `fn a_vertex_run_and_an_ink_stroke_are_authored_in_drawing_order`

The one-derivation promise for the two list-driven families. `/Vertices`
and `/InkList` are sequences whose consecutive entries are joined by a
segment, so a build that sorted, de-duplicated or re-ordered them would
author a *different figure* from the one the preview drew — and the
difference would only be visible after saving.

The polygon row carries the extra claim: the closing vertex is **not**
appended, because `/Polygon` closes by specification and a duplicate
first point would author a zero-length segment.

### `fn a_polygon_needs_three_vertices_where_a_polyline_needs_two`

The engine's `validate_geometry` refuses `< 2` for both, so a two-vertex
`/Polygon` is legal PDF it would happily author: a closed shape from A to
B and back, which renders as a line. That is never what a gesture meant,
so the shell refuses it — and refuses it by **name**, so the trace
distinguishes "you double-clicked one click early" from "the run had no
extent".
