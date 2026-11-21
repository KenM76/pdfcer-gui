# `ui-verify/checks/fields_redrawn`

`fields_redrawn_for_need_appearances_are_disclosed`: **a form that asks
viewers to redraw its fields says so in the status bar, and the same form
without the ask does not.**

## What it guards

When a form sets `/AcroForm /NeedAppearances true`, `pdfcer-render` honours it
by default (`NeedAppearances::Honour`). It draws text and choice widgets from
their values instead of from the `/AP` they carry, and counts them in
`Diagnostics::widgets_redrawn_for_need_appearances`. A viewer that shows `/AP`
draws something else. While the current page's texture carries a non-zero
count, the shell shows the `status-group:fields-redrawn` line
(`fields_redrawn_disclosure`).

## How it drives

The check writes its own fixtures and needs no `--pdf` and no input. Both files
are one page with one `/Tx` widget at `/Rect [100 100 300 140]`. The widget has
an empty `/V` and an `/AP /N` that fills its box red. The value is empty, so no
glyph is drawn and no font finding joins the render notes. The two files differ
only in `/NeedAppearances`.

It opens each in turn and waits for the render worker's `raster-blend-space`
line, whose `fields_redrawn=` field is the engine's count for that raster. It
then reads the ui-rect trace for the region.

| `/NeedAppearances` | engine count must be | region must be |
|---|---|---|
| `true` | > 0 | declared |
| `false` (control) | 0 | absent |

A zero count on the first file, or a non-zero one on the control, is a harness
error: the fixture no longer exercises the line.
