# `panels::layers::tree`, `row`, `drag`, `folders`

The Layers list as the document's `/D /Order` arranges it. Drawn while the
search field is empty; a query draws the flat filtered list instead, with no
moves (a move in a filtered list has no visible destination).

## `tree`

- A **folder** is a fold: folder icon and label, no check box. Its menu:
  Rename, Remove (its contents lift into its place), then the moves.
- A **layer with sublayers** is a fold whose header is the layer's row.
- A **leaf** is the row. An unlabelled **grouping** indents what it holds.
- Layers `/Order` does not list are drawn flat after the tree.
- Folds start open. When the canvas selection moves to a layer inside a
  closed fold, its ancestors open, once per highlight change, so the
  operator can close them again.
- Fold state is keyed: a folder by `(path, label)`, a layer by its `ObjId`.
- Rows are recorded in draw order for the drag; a fold's header row is
  inserted at a slot reserved before its body is drawn.

Every row's menu carries `layerorder::offers`: Move up, Move down, Move out
of *holder*, and a Move into folder ▸ submenu.

## `drag`

A drag starts on a row's name. The pointer's row is split into the bands
every tree control uses (`bookmarks::reorder::band_at`): top quarter lands
before it, the middle half at the end inside it, the bottom quarter after it
and its open subtree. The caret is `bookmarks::reorder::paint_line`, dimmed
for a drop that changes nothing, fainter for a drop into itself. Release
raises one `OrderAction::Move`.

## `folders`

New folder (footer) adds a folder at the end of the top level, named by the
box or the first unused "New folder N". Rename is a small window whose draft
lives in egui temp memory.

## `row`

One layer's row: padlock when `/Locked`, check box, state word, and the name
as a frameless button sensing click and drag. Emits `layer-row`.

## Trace

| Line | When |
|---|---|
| `layer-node path= kind=folder\|layer\|group name= open=` | each node drawn |
| `layer-drag-begin path=` | a drag starts |
| `layer-drop from= parent= index= landing=` | released over a row |

Regions: `panel.layers.folder.<label>`, `panel.layers.fold.<name>`,
`panel.layers.menu.{move_up,move_down,move_out,move_into,rename_folder,remove_folder}`,
`panel.layers.menu.into.<label>`, `panel.layers.drop-caret`,
`panel.layers.new_folder[.name]`, `panel.layers.rename.{name,apply}`.
