# `pdfcer-gui-base/measure/place`

Where a ce dimension lands when the operator clicks to place it. One function,
`placed_at(kind, p)`, turns a page-space point into the kind's placement
fields; the canvas's live preview and its commit both call it, so the text
under the pointer is where the click puts it.

## Contract

- Only placement fields change; the measured geometry, and so the value,
  never depends on where the dimension is dropped.
- **Linear, Perimeter:** `offset` and `text_along` from
  `DimensionKind::placement_from_point`, so `label_anchor()` is `p`.
- **Circular:** `leader_angle` turns to face `p` and `text_distance` is `p`'s
  distance past the rim, then clamped by `circular_text_distance` exactly as
  the engine clamps it (a click inside a radius stops the text at the
  centre, `-radius`).
- **Angular:** `radius` becomes `p`'s distance from the apex; below 1 pt the
  old radius stays. The engine centres the text on the arc whatever
  `text_along` holds (request G064), so only the arc follows the click.

`Placing { kind, disclosures }` is what `MeasureState::placing` holds between
the gesture completing and the placing click; `disclosures` travel to the
commit unchanged.
