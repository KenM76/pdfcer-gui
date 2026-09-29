# `pdfcer-gui-base/settingspages/forms`

## Item notes

### `fn the_slider_spans_everything_the_store_accepts`

The regression test for the silent-edit hazard every slider in this
window carries: a control whose range is narrower than what the engine
accepts rewrites a value the operator chose deliberately, the moment
they open the window to look at something else. `egui::Slider` clamps
on draw, so the narrowing would be silent and the value would be gone.

### `fn row_tolerance`

A slider rather than a typed number, and not because typing is hard: the
useful range is a single inch and the useful resolution is coarse, so the
control that shows the whole range at once is the one that answers the
question *"is a point enough?"* without the operator having to know what
the bounds are.

Linear, for [`super::measuring::parallel`]'s reason — half a point against
one point matters exactly as much as ten against twenty, because both
answer *how crooked may this form be before I stop calling it a row?*
