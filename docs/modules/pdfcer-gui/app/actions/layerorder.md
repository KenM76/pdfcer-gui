# `app::actions::layerorder` — folders and moves in the Layers panel

Each `OrderAction` is one `EditSession` arrangement verb, so one undo entry
that rewrites `/D /Order` and nothing else:

| `OrderAction` | Verb |
|---|---|
| `AddFolder{parent, index, label}` | `add_layer_folder` |
| `RenameFolder{at, label}` | `rename_layer_folder` |
| `DeleteFolder{at}` | `delete_layer_folder` — the folder's contents lift into its place |
| `Move(Move)` | `move_layer_node`, after `layerorder::engine_move` |

`worth_moving` runs before the session is borrowed mutably: a move that
changes nothing is traced and dropped; one into itself or to a vanished path
is refused with its own sentence.

Every receipt names where the entry now is, read back from the document
after the edit (the outcome's `path`), not where the gesture aimed. When the
engine reports `follows_layer` (a folder whose array follows a layer, so the
label belongs to that layer's sublayer list) the receipt says so.

Refusals map onto `LayerRefusal`: `NotALayerFolder` → `NotAFolder`,
`LayerOrderPathNotFound` → `OrderPathNotFound`, `LayerOrderInexpressible` →
`OrderInexpressible`, `LayerOrderNotEditable` → `OrderNotEditable`,
`EmptyLayerName` → `EmptyName`.

## Trace

`layer-order op=add-folder|rename-folder|delete-folder|move from= path= changed= follows_layer=`,
and `layer-order op=move from= changed=false reason=` for a move dropped
before the engine.

Verified by `ui-verify` check `layer_folders_show_and_reorganise`.
