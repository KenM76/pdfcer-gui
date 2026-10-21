# `ui-verify/checks/objects_multiselect`

`objects_tree_gathers_rows_for_one_menu` — **rows in the Objects tree gather
with Ctrl+click and Shift+click, and a right-click on a gathered row keeps
the whole selection for its menu.**

## What it drives

Opens `fixtures/two-photos.pdf` (two image XObjects; object rows 0 and 1) in
Edit mode with the default layout, entirely off-screen through the scripted
pointer.

1. Open the Objects tab, click row 0, Ctrl+click row 1; require
   `objects-panel-selection op=add sel=2 level=object`.
2. Click row 0 (selects it alone), Shift+click row 1; require
   `objects-panel-selection op=set sel=2 level=object`: the run from the
   anchor.
3. Right-click row 0; require no new `objects-panel-selection` line. Press
   `menu.item.objects.row.format.shrink_images`; require
   `shrink-images-opened pictures=2`, then after Apply `images-shrunk asked=2`.

## What a failure means

| arm | failure |
|---|---|
| Ctrl+click | the modifier is ignored, so the click replaced the selection |
| Shift+click | no run is selected from the anchor |
| right-click | the menu's click replaced the selection, so its commands act on one row |
| menu | Make pictures smaller is missing from the row menu or reached fewer than both pictures |

Modified clicks are driven only in the main window: the scripted pointer
carries modifiers and the secondary button there alone, so the check refuses
a tree drawn in a separate window rather than clicking the wrong place.
