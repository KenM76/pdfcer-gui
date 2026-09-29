# `ui-verify/checks/spots_flattened`

`spots_flattened` — **a page naming more spot inks than the renderer keeps
separate says so in the status bar, and a page within the limit does not.**

## What it guards

`pdfcer-render` keeps at most `compositor::MAX_SPOTS` spot inks as separate
colorant planes per page, and fewer if the buffer's byte ceiling cannot fit
another. Any further ink still paints, but through its tint transform, so
overprint and blending treat it as process colour. The engine counts the
distinct inks it flattened in `RenderDiagnostics::cmyk_spots_flattened`. The
shell shows the `status-group:spots-flattened` line while the current page's
texture carries a non-zero count (`spots_flattened_disclosure`).

## How it drives

The check writes its own fixtures, so it needs no `--pdf`. It writes the same
one-page file twice, with five `/Separation` inks and with four. Each ink is
painted twice, and the page group is CMYK so the colorant buffer is used.

It opens each file in turn and waits for the render worker's
`raster-blend-space` line, whose `spots_flattened=` field is the engine's
count for that raster. It then reads the ui-rect trace for the region.

| page | engine count must be | region must be |
|---|---|---|
| five inks | > 0 | declared |
| four inks (control) | 0 | absent |

## Failure versus skip

- **SKIP:** the fixture no longer exercises the limit, because the engine
  flattened nothing on five inks or something on four. That means the roster
  changed, so the fixture needs updating. It is not a shell defect.
- **FAIL:** a non-zero engine count with no region, or a region on the
  control. These are the shell's own defects.
