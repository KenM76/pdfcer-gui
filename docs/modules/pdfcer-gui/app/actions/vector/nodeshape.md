# `app::actions::vector::nodeshape` — Format ▸ Nodes, committed

One press acts on every selected node of one path, through
`apply::vector_edit_on_page`, as one undo step.

| shape | page path | path inside a placed drawing | undo kind |
|---|---|---|---|
| Insert | `EditSession::insert_node` at `t = 0.5` | `insert_node_in_form` | `CommandKind::InsertNode` |
| Corner, Smooth, Symmetric | `convert_node` with `NodeKind` | `convert_node_in_form` | `CommandKind::ConvertNode` |
| Line, Curve | `convert_segment` with `SegmentKind` | `convert_segment_in_form` | `CommandKind::ConvertSegment` |

## Contract

- **In:** a page, a `NodeHost` (page-object or leaf index), the selected
  nodes (object-scoped, ascending), a `NodeShape`.
- **Order:** nodes are visited in descending order, so an insert never
  renumbers a node still to visit.
- **Partial success:** a node the engine refuses is skipped and the rest are
  changed. When at least one changed, the entries are folded into one undo
  step (`fold_undo`) and the status line says how many were skipped and why
  (`text::nodeshape::nodes_skipped`). When none changed, the first refusal is
  the funnel's error, as for any refused edit.
- **Selection after an insert:** each new node is selected
  (`SelectionState::select_nodes`, via `node-inserted`). The node added after
  the `i`-th smallest selected node is `node + i + 1`, because each smaller
  node's insert shifted it up by one. A convert renumbers nothing and leaves
  the selection alone.
- **Undo and redo** of an insert clear the selection (`actions::history`),
  because the restored node list no longer matches its indices.
- **Trace:** `node-shape-applied page= host=object:N|leaf:N shape= asked= done= refused=`,
  lists comma-joined, `-` for none.

## Disclosure

The engine's sentences pass through unchanged: a rectangle rewritten as lines,
straight sides turned into curves before a smooth or symmetric node, a
clipping path reshaped. Inside a placed drawing every placement of the form
changes; the funnel line carries the engine's disclosure for that.
