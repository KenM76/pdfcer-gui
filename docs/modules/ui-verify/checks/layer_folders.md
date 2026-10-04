# `ui-verify/checks/layer_folders`

`layer_folders_show_and_reorganise` — the Layers panel draws the document's
`/D /Order` as a tree, and its folder and move controls rearrange it.

# What it drives

Its own fixture, `fixtures/layer-folders.pdf` (ignores `--pdf`), whose
`/Order` holds every element form ISO 32000-1 Table 101 permits: two labelled
folders, a layer with a sublayer array, and plain layers.

1. Edit mode; the Layers panel. The newest frame's `layer-node` lines must
   show the declared tree: `0 folder Sheet`, `0.0`/`0.1` its two layers,
   `1 layer Dimensions` with `1.0 Dimensions Reference`, `2 folder Services`
   with `2.0 Electrical`, `3 layer Notes`.
2. Right-click `Notes`, hover `panel.layers.menu.move_into`, click
   `panel.layers.menu.into.Services`. `layer-order op=move path=2.1
   changed=true`, and the tree shows Notes at `2.1`.
3. Type `Civil` into `panel.layers.new_folder.name`, click
   `panel.layers.new_folder`: `op=add-folder path=3`, a folder at `3`.
4. Drag `panel.layers.row.Electrical` onto the middle of
   `panel.layers.folder.Civil`: a `layer-drag-begin` line, then `op=move
   path=3.0`; Electrical at `3.0`, Notes at `2.0`.
5. Right-click the Civil folder, Rename folder, type `Civil2`, Apply:
   `op=rename-folder path=3`.
6. Right-click Services, Remove folder: `op=remove-folder path=2`; Notes now
   at `2` and Civil2 still at `3`.
7. Ctrl+Z: Services is back at `2` holding Notes.

# Reading the tree

`layer-node` is traced for every node every frame. The newest frame is read
by walking the trace backwards and stopping at the first repeated path, so a
path that no longer exists is not read from an older frame.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`. It shares `click` and `right_click_row` with `layer_authoring`.
