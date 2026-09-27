# `ui-verify/checks/layer_authoring`

`a_layer_can_be_made_renamed_and_deleted` — the Layers panel creates, renames
and deletes a layer, and Ctrl+Z undoes the delete.

# What it drives

Any `--pdf`; the check adds its own layer, so a document with none works.

1. Edit mode; open the Layers panel from the View tab if it is not on screen.
2. Type `Welds` into `panel.layers.new.name`, click `panel.layers.new`. A
   `layer-added … name="Welds"` line must follow and the region
   `panel.layers.row.Welds` must be declared.
3. Right-click that row, choose `panel.layers.menu.properties`, select the
   name in `panel.layers.props.name`, type `Welds2`, click
   `panel.layers.props.apply`. A `layer-edited … changed=true` line must
   follow, `panel.layers.row.Welds2` must be declared and `…row.Welds` not.
4. Right-click the row, choose `panel.layers.menu.delete`, click
   `panel.layers.delete.keep`. A `layer-deleted … changed=true` line must
   follow and the row must be gone.
5. Ctrl+Z: the row must be back.

# Why rows are witnessed by region, not by `layer-row`

A row's presence is the declared-and-not-retired state of its region, which
covers absence as well as presence. The `layer-row` trace line cannot witness
an absence.
