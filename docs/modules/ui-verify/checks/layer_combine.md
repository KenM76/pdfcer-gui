# `ui-verify/checks/layer_combine`

`layers_can_be_merged_and_flattened` — the Layers panel merges one layer into
another and flattens every layer into the page, and one Ctrl+Z undoes the
flatten.

# What it drives

A `--pdf` with no hidden layers; the check adds its own two, both shown.

1. Edit mode; open the Layers panel from the View tab if it is not on screen.
2. Create `Welds` and `Frame` through `panel.layers.new.name` and
   `panel.layers.new`; both rows must appear.
3. Right-click `Welds`, choose `panel.layers.menu.merge`, open
   `panel.layers.merge.target`, pick `panel.layers.merge.option.Frame`, click
   `panel.layers.merge.go`. A `layer-merged … changed=true layers=1` line must
   follow; the `Welds` row must be gone and `Frame` still there.
4. Click `panel.layers.flatten`, then `panel.layers.flatten.go`. A
   `layer-flattened … changed=true` line must follow and `Frame` must be gone.
5. One Ctrl+Z: `Frame` must be back. The engine coalesces a flatten into one
   undo entry; a second step needed would fail here.

# Why `flatten.go` and not the two hidden-layer choices

With no hidden layer the dialog offers only Flatten. A document with a layer
hidden when it opens offers Remove and Show instead, and this check would stop
at step 4 saying so.
