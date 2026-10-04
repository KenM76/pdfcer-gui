# `layerorder` — the Layers panel's tree, and where a move puts an entry

Pure functions over `pdfcer_core::layers::Layers::order` (the `/D /Order`
array, Table 101). No egui, no session.

## The tree

`tree(&order)` reduces each `OrderNode` to a `Node`:

| `OrderNode` | `Kind` | Drawn as |
|---|---|---|
| `group: Some(id)` | `Layer(id)` | the layer's row; children are sublayers |
| `label: Some(..)`, no group | `Folder(label)` | a label row with no check box: a folder has no visibility |
| neither | `Grouping` | indentation only |

A position is a **path**: child indices, root level first. `dotted(path)`
spells it `0.1.2` for traces and checks.

## Moves: two coordinate systems

The panel names a destination in the tree **as it is drawn**: `Move{from,
parent, index}` puts the entry before the child currently at `index` of
`parent` (`index` = child count means the end).

`EditSession::move_layer_node(from, parent, index)` reads `parent` and
`index` **after** the entry is removed, and after any unlabelled grouping the
removal empties is dropped. `engine_move` simulates that removal and returns:

| `Resolved` | Meaning | What the shell does |
|---|---|---|
| `Engine{parent, index}` | a real move | calls the verb with these |
| `Unchanged` | the entry is already there | raises nothing |
| `IntoItself` | the destination is the entry or inside it | refuses before the engine is asked |
| `NotFound` | a path names nothing (the tree changed under the gesture) | refuses |

Deciding the no-op in the shell matters: the engine would record an undo
entry for a move that changes nothing.

## Menu offers

`offers(tree, at)` lists what a row's right-click menu carries: Up and Down
among siblings, Out (to just after the folder, layer or grouping holding it)
and Into (the end of each folder that is not the entry or inside it). Each
offer carries its `Move`, so the menu and a drag raise the same action.
