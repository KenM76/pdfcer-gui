# `dimlinetype` — the line-type editor

One control for every ce dimension stroke that takes a line type: a group's
style, a member's dimension lines, and its circle, centre-line and centre-mark
parts. It edits a `pdfcer_core::dimension::LineTypeRef`: one of the six named
`LineType`s (continuous, hidden, phantom, chain, center, stitch), listed from
`LineType::ALL` so a type the engine adds appears without a change here, or
*Custom*, which keeps a `DimDash` pattern.

A named type is stored by name and drawn by the engine to its standard at the
cascaded line-type scale, so the editor never builds a dash array for one.

**Regions.** With a region it publishes `<region>.combo` for the drop-down and
`<region>.option.<token>` for each row, `<token>` being `LineType::token()`
(`hidden`, `center`, …). A driven check picks a type by that name.
