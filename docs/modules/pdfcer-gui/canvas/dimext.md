# `canvas::dimext`

The extension-line grips of a selected linear ce dimension.

## Contract

- `grips(doc, selection)` gives one grip for each extension line that is
  drawn, in CANVAS space. The grip sits at the start of the segment from
  `DimensionKind::extension_segments(style)`, which is the end nearest the
  measured point. An end whose gap already reaches the dimension line has no
  segment, so it has no grip. Only `DimensionKind::Linear` has grips.
- `grips_drawn(ctx, doc, selection)` is `grips` with the dragged grip moved
  to where the drag has it this pass. It is read by `canvas::painting`, which
  draws square handles and publishes them as
  `canvas.dimension-extension.{0,1}` (end A, end B).
- `grip_at(doc, map, selection, screen)` returns which grip a press landed
  on. The tolerance is `VERTEX_HANDLE_PT / 2 + 3` screen points.
  `canvas::pressing` asks it after the vertex handles and before the body,
  so a press on a grip never becomes a body drag.
- `drag(Frame, actions)`:
  - Mid-drag, it returns a `dimdrag::Placed` whose `baked` field is
    `EditSession::dimension_preview` of the moved kind.
  - On `Phase::Complete`, it traces
    `dimension-extension-gap id= end= gap=` and pushes
    `DimensionAction::SetExtensionGap { gap: Some(gap) }`. The action arm
    commits `EditSession::set_dimension_extension_gap` through
    `apply::vector_edit`, which traces `set-dimension-extension-gap`.

## The gap

The gap is measured in points, in page space, along the extension line from
the picked point. A drag changes it by the pointer's page-space delta
projected onto the unit vector from the picked point towards the dimension
line. So the grab point is kept, and only motion along the line counts.

The gap is clamped to `[0, extension_reach − 0.5 pt]`. At the reach the
engine draws no extension line at all, and a grip with no line under it
cannot be picked up again.

## Why the grip is drawn live

The committed dimension stays in the page raster mid-drag. A shortened line
is a sub-segment of the committed one, so the baked preview adds no ink and
the drag looks inert. Drawing the grip at the dragged position is the only
cue until the engine can render a page that omits one annotation (request
G052).

The live position is held in egui temp data, stamped with
`ctx.cumulative_pass_nr()`. It is read only in the pass that wrote it, so a
drag that ends without a release frame leaves no stale grip. Interact runs
before painting in the same pass. The driven check asserts this by reading
the published grip rect with the button held.

## Not built

- Grips on circular ce dimensions. They have no extension lines.
- Resetting a gap to the style default (`gap: None`). The action carries the
  `Option`, but no gesture sends `None` yet.

Driven by `ui-verify/checks/dimension_extension_grip`.
