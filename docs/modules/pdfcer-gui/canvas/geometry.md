# `pdfcer-gui/canvas/geometry`

## Item notes

### `const PASTEBOARD_FRACTION`

**A whole viewport is also, exactly, the placement at which the sheet
stops being visible at all** — and that is what O186 turned out to be. The
fraction is NOT reduced to fix it: it is still one viewport, because the
operator's sentence is still the rule. [`MIN_SHEET_ON_SCREEN`] is
subtracted from it instead, so the page corner arrives at the opposite
corner and **can be seen there**, which is what he was asking for in the
first place. See that constant for the measurement.

### `fn sheet_sliver`

[`MIN_SHEET_ON_SCREEN`] normally, and **half the viewport** on a canvas
narrower than twice it. The second case is not hypothetical — a docked
panel can be dragged down to a few points, and a frame measured before
layout reports a viewport of zero — and the `min` is what keeps the
pasteboard non-negative there without a separate guard. At `viewport = 1.0`
the pasteboard becomes `0.5`: still a pasteboard, still positive, and the
interval [`visible_origin_range`] returns is still non-inverted, which is
the property the rest of this module is entitled to assume.

Deliberately a function rather than an expression inlined at its one call
site, so the tests can measure the rule directly instead of inferring it
from a pasteboard that has already had the overhang branch applied to it.
