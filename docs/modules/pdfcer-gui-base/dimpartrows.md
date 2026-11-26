# `dimpartrows` — a ce dimension's per-part style rows

The Properties rows for the parts a ce dimension draws besides its own lines:
a radius or diameter **circle**, a **centre line**, a **centre mark** (its
stroke, the gap at the centre, how far the extension lines reach, and whether
they are drawn in the line font), and the **line-type scale** every named line
type is drawn at. Called last from `dimensionoverrides::show`.

**Which rows appear.** `Parts::of` reads the kind: a `Circular` draws its
circle, and its centre mark when it carries one; a `CentreLine` its line; a
`CentreMark` its mark. Every other kind gets only the line-type scale.

**An unset row** follows the dimension's own lines, as the engine's cascade
resolves it (`DimensionStyle::circle_stroke`, `centre_line_stroke`,
`centre_mark_stroke`), and names the tier it came from like every other
override row. A centre line's unset dash seeds as `Center` and a centre mark's
as `Continuous`, the types the engine draws them in.

**Regions.** `dimension-properties.<part>.<dash|width|colour>.set` for each
*Set here* box, `<part>` being `circle`, `centre-line` or `centre-mark`;
`dimension-properties.centre-mark.<gap|extension|line-font>.set`; and
`dimension-properties.line-type-scale.set`.

Whether the circle is drawn at all is not a style: it is the dimension's
`show_circle`, switched by *Draw the circle* in the Properties panel
(`DimensionAction::SetCircleShown`).
