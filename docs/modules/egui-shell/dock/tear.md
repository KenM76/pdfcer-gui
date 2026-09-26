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
