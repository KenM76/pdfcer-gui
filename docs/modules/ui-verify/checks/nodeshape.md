# `ui-verify/checks/nodeshape`

`a_nodes_segment_can_be_curved_and_a_node_inserted`, on
`fixtures/line-and-curve.pdf`: one stroked open path, node 0 (100,100) by a
line to node 1 (250,100), by a curve to node 2 (250,200).

PASS needs, in order:

1. A click on the line, a double-click into its Part rung, and a double-click
   on `canvas.anchor.0` stand at the Node rung with one anchor selected:
   node 0, which the line leaves.
2. Format ▸ Segment to curve: `node-shape-applied shape=curve asked=0 done=0`,
   and `canvas-handles n=` rises — the new curve has a handle at node 0, the
   line had none.
3. Format ▸ Insert node: `shape=insert asked=0 done=0`, `canvas-anchors
   total=` rises by one, and `selection-set … node=1 … via=node-inserted`
   names the new node.

`canvas-handles` is written only on a frame that draws a handle, so no line
since a mark reads as zero handles — the state before the curve, and the state
a press that failed to curve the segment leaves.

## Falsified

- Segment to curve mapped to a line: FAIL at step 2, handles 0 → 0.
- The inserted node selected one too low: FAIL at step 3, `node=0`.

## What it does not prove

- Rendered pixels; the oracles are the anchor and handle census the canvas
  traces every frame.
- Corner, Smooth, Symmetric and Segment to line.
- A path inside a placed drawing.
- Several nodes at once, and the skipped-node sentence.
