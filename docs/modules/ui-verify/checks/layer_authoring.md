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

# Driven off-screen

The check drives the scripted pointer in a window placed off the desktop, so it
runs under `--no-input`. Footer controls (`panel.layers.new.name`,
`panel.layers.new`, `panel.layers.flatten`) are scrolled wholly inside the dock
body with `bring_into_body` before they are clicked, and a control that still
is not wholly inside fails the check by name. That containment is what found
the New layer button clipped at the panel's right edge; with the field fixed at
140 pt it fails at step 2, and it passes on `fixtures/four-pages.pdf`.

The setup helpers it shares (`open_from_tab`, `click`, `right_click_row`, the
band search) take `&impl input::Click`, so `layer_combine` still drives them
with the OS pointer.

# Why rows are witnessed by region, not by `layer-row`

A row's presence is the declared-and-not-retired state of its region, which
covers absence as well as presence. The `layer-row` trace line cannot witness
an absence.
