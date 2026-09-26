# `egui-shell/dock/tear`

## Item notes

### `fn outside_the_dock`

Each side is grown by [`super::plan::SPLITTER_THICKNESS`] before the test,
because [`super::geometry::DockGeometry::side_rect`] holds the side's
*columns*, and the side's width handle sits outside them on the edge facing
the document. Without the growth the handle would be a few points of tear
zone lying along the whole height of the dock's inner edge — the one place a
drag crossing between two compartments passes through.

### `const GRAB_PTS`

The window's top-left goes this far up and to the left of the release, which
leaves the cursor on the platform's own title bar — the window's outer frame
begins above its inner rectangle by about this much. So the gesture ends with
the pointer holding the thing it just made, and an operator who keeps the
button down can carry on dragging the window itself.

### `struct TearPreview`

Published on [`super::DockFrameReport`] for [`super::overlay::DropPreview`]'s
reason: the visible form of the affordance is an outline, which is precise
to look at and nothing a harness can assert on.

### `fn draw`

Runs from [`super::Dock::show`] after [`super::overlay::draw`] and before
[`super::drag::settle`], so that the two affordances it defers to have
already had their say and the settlement reads one decision.

## The stand-down is redundant today, and is kept anyway

Both of the other two affordances require the pointer to be *inside* a
compartment — a caret needs the strip it is inserting into, a compass needs
the body it divides — so [`outside_the_dock`] already excludes every frame
on which either of them published. The guard below therefore cannot be
reached by any input, which means no test can falsify it: planting a defect
in it leaves the suite green.

It stays because the implication is a property of a predicate that is
expected to move — the module header names the case that will tighten it —
and a release build where it no longer holds should draw one affordance
rather than two. What measures the implication is the assertion in
[`super::drag::settle`], which runs on every debug frame and fails by name
the moment two of the three answer one drag.
