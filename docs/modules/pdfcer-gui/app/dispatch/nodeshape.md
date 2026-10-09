# `app::dispatch::nodeshape` — Format ▸ Nodes

Six commands, each mapped to a `NodeShape`:

| id | does |
|---|---|
| `format.node_insert` | a node halfway along the segment after each selected node |
| `format.node_corner` | the selected nodes become corners |
| `format.node_smooth` | the selected nodes become smooth |
| `format.node_symmetric` | the selected nodes become symmetric |
| `format.segment_line` | the segment after each selected node becomes a line |
| `format.segment_curve` | the segment after each selected node becomes a curve |

## Routing

`shapeable_nodes` is the one test, used by the dispatcher and by
`app::conditions` for `selection.nodes_shapeable` (`NODES_SHAPEABLE`): the
selection stands at the Node rung, the entered object is a page object or a
leaf of a placed drawing, and at least one node is selected. Only a path has
nodes, so a non-empty node set is the path test. In a mode that does not edit
content the press is declined with `command-declined reason=no-nodes-selected`;
the commands are not shown there (`ribbontabs::format`).

The six are on the Format tab's Nodes group only (`RIBBON_IA.md` §5.8). They
are not on the canvas object menu: none has a glyph, and six blank rows would
leave that menu's icon column mostly empty
(`menus_wiring_tests::a_reserved_icon_column_is_never_mostly_empty`).
