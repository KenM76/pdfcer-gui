# `ui-verify/checks/layer_assign`

`layer_assign_moves_the_selection` — a selected page object goes onto a layer
from the Properties panel, Ctrl+Z takes it off, and a selected annotation goes
onto a layer from its right-click Move to layer… window.

# What it drives

Its own fixture, `fixtures/layer-assign.pdf` (ignores `--pdf`): two layers,
`Walls` (6 0) and `Notes` (7 0), an unlayered blue box at 100..400 × 100..300,
and a `/Square` annotation at 480..720 × 350..520 with no `/OC`.

1. Edit mode; click the box's centre; bring `dock.tab.file.properties`
   forward if the combo is not drawn.
2. `properties.layer.combo`, then `properties.layer.option.Walls`:
   `layer-assigned kind=objects moved=1 … layer=6_0`.
3. Ctrl+Z: `undo-applied`.
4. Walls again: `moved=1` once more. A Ctrl+Z that undid nothing leaves the
   combo on Walls, so the choice changes nothing and no line follows.
5. Click inside the annotation (`annot-select`), right-click it, press
   `menu.item.canvas.markup.format.move_to_layer`, then
   `layer-assign.window.combo`, `layer-assign.window.option.Notes`,
   `layer-assign.window.go`: `layer-assigned kind=annotation subtype=Square
   changed=true … layer=7_0`.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`. Falsified by a combo that raises nothing (red at step 2) and by
an annotation move that passes no layer (red at step 5, `changed=false`).

# `paste_goes_on_the_current_layer`

The second check in the module, sharing its launch and `Drive`.

1. Open Layers from View ▸ Layers when no Walls row is declared, and click
   Walls' name. The next `layer-row` line for Walls must carry `current=1`.
2. Select the unlayered box, Ctrl+C, Ctrl+V. The paste lands 5 pt up and on
   top of the box.
3. Click the box again, which picks the pasted copy. The next `layer-row` line
   for Walls must carry `highlighted=true`. The highlight is resolved from the
   selected object's content (`panels::layers::highlight`), so it reads where
   the engine put the copy, not what the shell asked for. The original box is
   on no layer, so only a copy on Walls can light the row.

Falsified by pasting with no layer (`paste_objects_on_layer(.., None)`): red at
step 3. The hidden-layer refusal is not driven here.

# `a_caret_goes_on_the_current_layer`

The third check in the module, sharing its launch and `Drive`, and
`make_walls_current` with the paste check.

1. Walls current, as above (`current=1`).
2. Markup ▸ Insert text, a click at (150, 450), `on walls` typed, and
   `text-annot.accept` clicked in the window's own viewport. Owed:
   `add-caret-annot-on-layer` and `caret-annot-placed`.
3. Escape, then a click 5 pt below the apex, inside the caret's body:
   `annot-select`, and the next `layer-row` for Walls carries
   `highlighted=true`. `panels::layers::highlight::on_annotation` reads the
   annotation's `/OC` from the document, so this is the engine's answer.

The caret stands for every adder `drawlayer::onto` serves; the others are not
driven one by one. Falsified by `onto` keeping `options.layer` instead of the
current layer: the add still traces its layer, and step 3 goes red.

# `a_drawing_goes_on_the_current_layer`

`PDFCER_DIAG_IMAGE_PATH` names the `vector-art.svg` fixture.

1. Walls' name clicked: `layer-row name="Walls" … current=1`.
2. Edit tab ▸ `ribbon.item.edit.insert_image` (found in the overflow when the
   band is narrow), then `insert-image.insert`.
3. `add-svg-on-layer` and `selection-set … via=placed`; Walls' row then reads
   `highlighted=true`: the selected drawing is on Walls.

Falsified by handing the engine `None` in place of the layer: the trace still
says `add-svg-on-layer` (the shell resolved the layer) and step 3's
`highlighted=true` fails.

