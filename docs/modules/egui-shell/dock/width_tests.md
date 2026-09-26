# `egui-shell/dock/width_tests`

## Item notes

### `const SLACK`

`egui` rounds widget rectangles to whole physical pixels, so an edge
can land a fraction of a point beyond an exact arithmetic boundary
without anything being wrong. One point is well below anything a
person could see and well above the rounding.

### `fn registry`

Deliberately **not** `"Panel 0" … "Panel 8"`: labels of equal length
make every tab the same width, which is the one property real
proportional text does not have and the one the arithmetic must not
assume.

### `fn text_width`

**The text style matters, and getting it wrong is silent.** The tab
bar resolves [`egui::TextStyle::Button`] against the live style; a
test that measured at a hard-coded `proportional(14.0)` would compute
a *different* flip point from the one the renderer uses, and the
binary-searched assertion below would then be aimed a few points away
from the boundary it exists to probe — passing, while testing an
ordinary width. A search at a hard-coded 14 pt against a renderer
drawing at the style's button size reports "nothing overflowed" at a
width where something must, and reports it in green.

### `fn the_dock_measures_real_proportional_text`

Everything below is worthless if this is not true, and "worthless"
here means "passing" — which is why it is asserted rather than
assumed.

### `fn the_reservation_covers_the_widest_label_with_real_metrics`

`"⏷ 8 more"` is wider than `"⏷ 9 more"` in any face whose digits are
not tabular. With no font installed the two measure the same and this
test is vacuous, which is exactly why it lives in this file.

### `fn the_affordance_is_inside_the_bar_at_the_exact_width_where_tabs_start_hiding`

A sweep is too coarse: the RAG entry this test is written from records
an 11 pt sweep step walking straight over an 8 pt estimation error.
The flip point is where the arithmetic and the drawing are most likely
to disagree, because it is the only width at which a one-point error
changes the answer.

### `fn the_affordance_is_always_within_the_window`

This is the `max_rect`-inflation trap stated as an assertion. A
control positioned by subtraction (`right − width`) lands at a
negative x the moment the bar is narrower than its reservation; it is
still laid out, still allocated, still reported with a plausible
`Rect` — and painted where nobody can see or click it. Nothing errors
and nothing warns. Only a comparison against the **window** catches
it.

### `fn an_inactive_tab_with_a_huge_label_does_not_widen_the_dock`

Failure mode #3 is *"an inactive tab you cannot see holds the whole
dock open"*. Here one panel is given a preposterous label and left
**inactive**, and the dock is still drawn at exactly the width the
layout asked for. With no font installed this test cannot fail,
because the preposterous label measures the same as every other one.
