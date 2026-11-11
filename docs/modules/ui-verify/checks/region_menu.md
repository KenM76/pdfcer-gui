# `ui-verify/checks/region_menu`

`the_region_menu_offers_every_region_verb` — the left rail's Select group arms
the Region tool, and the region's right-click menu carries every region verb
in Edit and only the copies and Save as PDF… in Read (O406).

## Steps

1. Launch in Edit (`PDFCER_DIAG_INVOKE=mode.edit`) off the desktop. Require
   `rail.select.view.tool_snapshot` to be drawn and click it.
2. Drag a region over the fixture's drawing; require the `snapshot-box` line
   and a non-zero `region-selection`. That box is laid only because the rail
   row armed the tool, so step 2 is the rail row's oracle.
3. Right-click inside the region; the `canvas-menu` line must name
   `canvas.snapshot`, and the menu must draw Crop to region, Redact region,
   Redact outside region and Delete in region.
4. Click the menu's Delete in region; the `region-delete-requested` line must
   ask for exactly the objects the region selected.
5. Launch in Read with the tool armed; lay the same region and open its menu.
   It must draw Copy, so the menu is known to be up, and none of the four
   verbs from step 3.

## Falsification

- Dropping the four items from `canvas.snapshot` fails step 3, listing the
  missing ones.
- Removing their `mode.edit_content` gating fails step 5, naming the leaked
  verbs.
- Moving the tool back to the rail's Navigate group fails step 1, listing the
  Select rows.

## What it does not cover

What crop, redact and redact-outside do once chosen: `region_commands` drives
each through Tool Options, which dispatches the same commands.
