# `egui-shell/ribbon/trailing`

## Item notes

### `fn shown`

One helper rather than the same `filter` written twice, because
[`measure`] and [`render`] disagreeing about which items exist is exactly
the class of defect this region's reservation is supposed to make
impossible.

### `fn measure`

`0.0` when there is nothing to draw — no trailing list, an empty one, or
one whose every item is hidden or names a command that is not registered.
A zero width is what makes the region **disappear** rather than leave a
gap, which is R9 in the layout rather than in the painting.

### `fn min_width`

The first control's own floor, exactly as [`super::qat::min_width`]
computes it — see [`super::plan::row`]'s header on why a region granted
less than a control's floor gets a control drawn *outside* its rectangle
rather than a smaller one.

### `fn render`

Dropping rule, disclosure and containment check are [`super::qat::render`]'s
verbatim, and the reasoning there applies here unchanged: a control below
its floor is drawn outside the rectangle it was given, so the loop stops
rather than truncating, and what it dropped is announced.
