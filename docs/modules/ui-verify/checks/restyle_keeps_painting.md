# `ui-verify/checks/restyle_keeps_painting`

`a_large_restyle_keeps_painting` — O310, driven.

In Edit mode an object click can select a whole text object of hundreds of
runs, and a size typed into Properties restyles every one of its show
operators. Done in one frame that froze the window for tens of seconds. The
check types a size for such an object and requires:

1. **the window keeps painting** — no `frame-long ms=` line above 800 ms
   (`HOLD_MS`) between the Enter and the first pointer step acknowledged after
   the verdict. A frame's `frame-long` line is written when the frame ends, so
   the frozen frame's line follows the verdict it emitted; the hover the check
   sends after the verdict is acknowledged only once that frame is over, which
   is what puts the line inside the window. An unacknowledged Enter is the
   freeze itself and is not an error;
2. **the whole gesture applies as one engine call** — a `text-style-batch`
   line with `applied == ops`, `planned_once=1` and `calls` of at least 1. A
   size change is pinned and creates no resource, so `EditSession::format_texts`
   plans it once; a build restyling operator by operator has no such line;
3. **one undo entry** — once Properties re-reads the typed size, one Ctrl+Z
   brings the re-read size back to the original.

By default the check writes its own page: one `BT … ET` of 120 lines at 4 pt.
`--pdf FILE --doc-point PAGE,X,Y` drives any other file; the check copies it to
`target/ui-verify/restyle-keeps-painting.pdf` and never touches the original.

Driven off the desktop through the scripted pointer, under `--no-input`.

## Driven

| input | doc-point | verdict |
|---|---|---|
| fixture | default | PASS: no frame 50 ms or more; `ops=120 applied=120 calls=1 planned_once=1 wall_ms=7`; one Ctrl+Z 6.00 → 4.00 |
| `D:/Dev/pdfTests/SW41177/SW41177.pdf` (private, never checked in) | `0,1215,595` | PASS: longest frame 219 ms (the post-commit Properties re-read of the 425-run object); `ops=238 applied=238 calls=1 planned_once=1 wall_ms=131`; one Ctrl+Z 6.00 → 13.23 |

A restyle the engine must apply one operator at a time (a decoration, a new
font or opacity resource) above `MAX_UNDO_DEPTH` operators is refused as one
call and split (`text-style-chunked`); no fixture reaches that branch, so it
is not driven here.

## Falsification

| control build | expected verdict |
|---|---|
| the build published from `ac1f5e49` (per-frame slices), `--exe … --allow-stale`, SW41177 | FAIL at step 2: "the restyle was not one planned engine call: `… slices=121 engine_ms=10960 wall_ms=17835 …`" — watched red |
