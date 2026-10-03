# `ui-verify/checks/snapshot_box`

`a_snapshot_box_stays_on_the_page_through_a_zoom` — View ▸ Snapshot arms from
the ribbon, a drag lays a box at the page points dragged over, and after a zoom
the box is drawn over the same page area (O272). The window is off the desktop
and driven only through `ScriptedPointer`, so the check runs under `--no-input`.

## Steps

1. Open `fixtures/blank-overhang.pdf`; click `ribbon.tab.view`, then
   `ribbon.item.view.tool_snapshot`, opening collapsed `ribbon.group.view.*`
   buttons until it shows. The last `snapshot-tool` line must say
   `armed=true`.
2. Drag from (0.42, 0.44) to (0.58, 0.56) of the page box (y up). The last
   `snapshot-box` line must name page 0 and the dragged corners within 1.5 pt.
3. The `canvas.snapshot` region must lie within 3 logical points of where the
   current canvas mapping puts those corners.
4. Press the status bar's zoom-in twice (the right end of
   `status-group:zoom`). The box's drawn width must change — otherwise the zoom
   did not happen and the check would prove nothing.
5. Rebuild the mapping: the region must again lie within 3 points of the mapped
   corners, unbounded by the viewport, and the `snapshot-box` fields must be
   unchanged.

## Falsification

Making `canvas::snapshot::screen_rect` return the box's canvas-space rect
without `PageMapping::rect_to_screen` fails step 3. Storing the box in screen
points at the release, so it no longer follows the page, fails step 5.
