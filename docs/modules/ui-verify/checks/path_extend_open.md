# `ui-verify/checks/path_extend_open`

Three checks for Format ▸ Nodes ▸ Continue path and Open path here, each in its
own launch.

## What it drives

`a_path_is_continued_from_its_end`, on `fixtures/line-and-curve.pdf`: one
stroked open path, node 0 (100,100) by a line to node 1 (250,100), by a curve
to node 2 (250,200).

1. A click on the line, a double-click into its Part rung, and a double-click
   on `canvas.anchor.2` stand at the Node rung with node 2 (the last node)
   selected.
2. Format ▸ Continue path traces `path-end-preflight … ok=true`, then
   `path-extend-armed`.
3. Two canvas clicks at (330,250) and (150,260), then Enter: `path-end-applied
   … edit=extend … ok=true` and the funnel's `extend-path page=` line, and
   `canvas-anchors total=` rises by two.
4. Ctrl+S saves in place (`save-in-place outcome=ok`). The saved page stream
   holding the path carries three `l` operators: the fixture's one and the two
   new segments.
5. Ctrl+Z traces `undo-applied`, and `canvas-anchors total=` is back to its
   value before the run.

`a_closed_path_is_opened_at_a_node`, on `fixtures/closed-square.pdf`: one
stroked square, `100 100 m 250 100 l 250 200 l 100 200 l h S`.

1. The same descent, to `canvas.anchor.1`, the corner at (250,100).
2. Format ▸ Open path here: `path-end-applied … edit=open … ok=true` and the
   funnel's `open-subpath page=` line.
3. Ctrl+S saves in place, and the saved stream holds no `h` operator.
4. Ctrl+Z traces `undo-applied` and restores the anchor count.

`a_middle_node_is_refused_at_the_press`, on the same open path: the descent
to `canvas.anchor.1`, the middle node, then Format ▸ Continue path. It must
trace `path-end-preflight … ok=false` and `path-edit-declined reason=…`, and
no `path-extend-armed`.

The saved file is the oracle for "the edit reached the content". The traces
are the oracle for "this verb did it" and for undo.

## Falsified

Both checks were run against a build that has neither command, `D:/scratch/b1/new-pdfcer-gui.exe`
with `--allow-stale`. Each FAILed at its press, with no
`ribbon.item.format.path_continue` / `path_open` on the Format tab or in its
overflow.

A build whose preflight always answers yes FAILed the middle-node check with
`armed=true`.

## What it does not prove

- Rendered pixels. The rubber-band preview is drawn and not asserted.
- Continuing from a path's first node, and the renumbering that follows.
- A path inside a placed drawing (the `_in_form` verbs).
- The refusal sentences for a closed subpath and an already-open one; only
  the middle node is driven.
- Backspace, double-click and Escape as finishers.
- The canvas right-click route to the same commands.
