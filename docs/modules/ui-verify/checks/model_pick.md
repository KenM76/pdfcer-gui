# `ui-verify/checks/model_pick`

`the_3d_viewer_selects_and_hides_from_the_picture` — a click on a part in the
3D picture selects it, the picture and the part list open the same menu, the
list follows the picture, and the hidden parts can be shown alone and brought
back by clicking them.

# What it drives

The engine corpus's `fixtures/synthetic/prc/named-tree.prc` (Assembly 0 over
Bolt 1, Bracket 2, Plate 3, Pin 4 stored hidden, Nut 5 stored suppressed),
opened with *View…* under `--no-input` with a `ScriptedPointer`. Every part is
the same square in the same place, so the centre of the picture is over
whichever shown part is nearest; which one is not asserted.

Each step reads the next `model-view-parts-state` line unless it says otherwise.

1. Click the centre of `model3d.image`: `why=pick`, `highlight` is one of 1, 2, 3.
2. Right-click the same point, click `model3d.menu.hide`: that part joins
   `hidden` and stays in `highlight`.
3. Click the same point: `highlight` is one other shown part.
4. Ctrl+click it: `highlight=-`.
5. Click `model3d.parts.hidden_only`: `view=hidden`.
6. Click the centre: `why=restore`, `hidden` loses exactly one part and
   `highlight` stays empty. Click the switch again: `view=shown`.
7. Click `model3d.part.name.1`, Shift+click `.name.3`: `highlight=1|2|3`.
   Ctrl+click `.name.3`: `highlight=1|2`.
8. Right-click `.name.2`, click `model3d.menu.isolate`: `hidden=3|4|5`,
   `highlight=1|2`.
9. Right-click a row, click `model3d.menu.fold`: `model-view-parts-fold`
   reads `folded=0`. Click the centre of the picture: the next fold line reads
   `why=reveal folded=-`.

# Falsified

The build before the picture took clicks fails step 1: the click traces no
parts line.
