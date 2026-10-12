# `canvas::respan`

A selected `/Highlight`, `/Underline`, `/StrikeOut` or `/Squiggly` shows two end
grips. Dragging either one changes which words the mark covers.

# Contract

- `ends(doc, selection)` returns the mark's quads, or `None` in these cases:
  - the selection is not a text markup;
  - the mark is locked;
  - its page is not the current page;
  - the engine cannot read its quads.
- Grip 0 sits at the middle of the first quad's leading edge. Grip 1 sits at
  the middle of the last quad's trailing edge. Both are in page space.
- Each grip is published as `canvas.markup-end.<index>` and drawn as a vertex
  handle square.
- A press is resolved with the same tolerance the node anchors use
  (`annotnodes::grab_tolerance`).
- On every drag frame, the text selection is re-run from the other end to the
  pointer:
  - The fixed end is stepped 1 pt into the text (never more than a quarter of
    its quad). Without the step, the caret lands on the edge the mark shares
    with the next word and takes one word too many.
  - The grip moves by the pointer's delta, so the point that was grabbed
    stays under the pointer.
- While dragging, the preview shows the outline of the new span. Where the
  pointer is over no text, it shows the current span instead.
- On release:
  - Over text, it traces `markup-respan id= index= quads=a->b` and raises
    `AnnotAction::RespanTextMarkup`. That calls
    `EditSession::respan_text_markup` and makes one undo entry.
  - Over no text, it traces `markup-respan-declined … reason=no-text` and
    raises `DeclineNodeEdit { NoTextThere }`.

# Why the node route

The grips use the existing `DragKind::MarkupVertex` gesture. `pressing` asks
`annotnodes::node_at` first, then `respan::grip_at`. `vertexroute` sends the
drag here whenever `ends` answers. The two families never claim the same
selection: text markups have no `annotnodes::geometry`. Shift and Alt are
applied once, above the route's fork.

The grips are kept out of `annotnodes::node_at`. That way the right-click
menu's add-node and remove-node rows never appear for a mark that has no
nodes to add or remove.

# Current page only

`OpenDoc::page_text` extracts only the page in view. On any other page, a
selection cannot be computed against the right text, so `ends` answers
`None` there and no grips are drawn.
