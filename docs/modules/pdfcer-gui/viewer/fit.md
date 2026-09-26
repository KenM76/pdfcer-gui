# `pdfcer-gui/viewer/fit`

## Item notes

### `fn fit_height_uses_the_height_ratio_only_and_lets_the_width_overflow`

The overflow is the point. A "fit height" that quietly refused to let
the page run off the side would be fit-page under a second name, and
the operator asked for it precisely because fit-page leaves a landscape
sheet as a band across the middle of a tall window.

### `fn each_fit_pins_exactly_the_axes_it_decides`

Asserted as a table rather than four separate tests because the
property that matters is the RELATIONSHIP between them: each
single-axis fit must pin exactly the axis it names and leave the other
alone, and a copy-paste slip that made `Height` pin the horizontal
would pass any test written about `Height` on its own.

### `fn a_pinned_axis_is_one_the_scale_makes_fill_the_viewport`

The link between [`FitMode::pinned_axes`] and [`fit_scale`], which are
two independent `match`es over the same enum and would otherwise be
free to disagree: a mode that pins an axis must produce a scale that
makes the page exactly fill the viewport on it, or fit inside it.
