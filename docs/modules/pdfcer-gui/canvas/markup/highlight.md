# `canvas::markup::highlight` — what the highlighter draws

The Highlight tool has five shapes, chosen in Tool Options' Highlight block:

| Shape | Drawn by | Authored as |
|---|---|---|
| Text (default) | `markup::text::swept`, falling back to `markup::band` | `/Highlight` from line quads, or one quad |
| Area | `markup::band` only | `/Highlight`, one quad |
| Ellipse | the Ellipse band | `/Circle`, washed |
| Freehand | `markup::ink` | `/Ink`, washed |
| Polygon | `markup::vertex` | `/Polygon`, washed |

**Washed** is `Pen::washed`: the highlighter colour as the ink, a 12 pt
solid stroke with no arrowheads, and `Wash::Shape`, which makes
`markup::spec` fill the ellipse or polygon with no outline and makes the
commit set `MarkupOptions::highlighter`, so the engine writes `/BM /Multiply`
and the shape darkens the drawing under it rather than covering it — the
same blend a `/Highlight` has.

## Why the armed tool stays `Markup(Highlight)`

The shape is a property of the highlighter, not a separate tool, so the
ribbon keeps the highlighter pressed and Tool Options keeps its block.
`drawn_as` is applied where the canvas decides behaviour — `tool::active`
and the vertex run's `pending` — and `pen_for` where the pen is sampled
(`canvas::present`, and the Finish command). Nothing else reads the shape.

## Alt, and the text sweep's clamp

Over text, a Text-mode drag sweeps in reading order, which on a drawing
reaches lines in other columns and across the sheet. Two remedies:

- **Alt held** during a Text-mode drag gives `Wash::Area`, and
  `markup::route` then skips the text sweep: the drag is a box.
- **The clamp** in `markup::text::swept` keeps only the swept lines that
  meet the drag's box grown by a quarter of its size on each side
  (`CLAMP_MARGIN`); a sweep the clamp would empty is left whole. When lines
  are dropped the commit leaves the status *"Highlight: clamped to the lines
  under the drag (n lines left out)"*, the off-canvas disclosure that the
  highlight is not the whole sweep.

## The polygon's keys

`markup::vertex::keys`, run before the canvas keys: with a run in progress,
Enter finishes it and Backspace removes its last vertex. Each key is consumed
only when it acted, so Backspace still deletes the selection otherwise.
Escape abandons the run, as for every vertex tool.

Choosing another shape abandons a run in progress.
