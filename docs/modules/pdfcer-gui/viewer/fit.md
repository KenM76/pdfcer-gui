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

### `fn pinned_axes`

# Why a fit mode has to answer a question about POSITION

`OPERATOR_REQUESTS.md` O28 — *"If I press the Fit width or fit page
button the view should center to the width as well or center the
page."*

Before O23's pasteboard a page no larger than the viewport had nowhere
to be except the middle, so *fit* and *centred* were the same act and
nobody had to decide which one the button meant. The pasteboard added a
whole viewport of slack on every side — deliberately, so any corner of
the page can be brought to any point of the screen — and with it the
state the operator reported: **the scale is right and the page is not
on screen.**

A pinned axis is one the fit has just decided the extent of, so there
is exactly one honest position for it and the view is placed there. An
unpinned axis is one the operator is still navigating, so their
position is **kept** — merely clamped to the page's own range, which is
what makes keeping it safe. Throwing them back to the top of a drawing
because they asked for a different scale would be a navigation they did
not ask for.

* [`Self::Page`] pins both: the page fits, and centred is the only
  answer.
* [`Self::Width`] pins the horizontal and keeps the vertical.
* [`Self::Height`] pins the vertical and keeps the horizontal.
* [`Self::None`] pins neither and returns `None` — it does not change
  the zoom at all (see [`ViewState::apply_fit`]'s early return), so
  there is no new extent to place against and moving the view would be
  a jump for a command that did nothing.

### `fn fit_scale`

Both arguments are in the same unit only by coincidence — `page_pts`
is PDF user-space units and `viewport` is egui logical points — and
the result is the ratio between them, which is exactly the "device
pixels per user-space unit" the renderer wants. (On a HiDPI display
egui's own `pixels_per_point` then multiplies again; that is handled
at the call site, not here, because it is a display property rather
than a document one.)

Returns `1.0` for a degenerate page or viewport rather than dividing
by zero. [`FitMode::None`] also returns `1.0`, though callers are
expected not to ask.
