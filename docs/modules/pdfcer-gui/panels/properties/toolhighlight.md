# `panels::properties::toolhighlight` — the Highlight block in Tool Options

Shown while the highlighter is armed: **Draws: Text | Area | Ellipse |
Freehand | Polygon**, each with its hint on hover, and one line under it —
the Alt hint for Text, the corner keys for Polygon. The choice is
`canvas::markup::highlight::set_shape`, held for the session.

Trace regions: `properties.tool.highlight` for the block and
`properties.tool.highlight.shape.<i>` for each choice, `<i>` being the
shape's index in `Shape::ALL`.
