# `freehand_detail_sets_how_many_segments_a_stroke_keeps`

**Detects:** a Markup ▸ Style with no Detail control, or a Detail control
whose value does not reach the freehand simplifier.

## Drive

1. Open a copy of `fixtures/square-no-appearance.pdf` in Review.
2. Open the Markup tab if `markup.style.detail` is not drawn.
3. Arm Freehand and draw a five-period sine wave (60 scripted moves, 12 pt
   amplitude) at page height 620; read its `markup-commit kind=Ink raw= kept=`.
4. Drag the Detail control 150 points right, which clamps it at 100; require
   the last `markup-pen` line to read `detail=100`.
5. Re-arm Freehand if the tool trace says it is no longer armed, and draw the
   same wave at page height 700.

## Oracle

The second stroke's `kept` must exceed the first's. Both strokes are the same
scripted path, so their `raw` counts agree and only the tolerance differs. The
`markup-pen` line separates "the control did not move the pen" from "the pen
did not reach the stroke".
