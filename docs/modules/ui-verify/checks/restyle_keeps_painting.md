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
2. **the whole gesture applies, at its own pace** — a `text-style-batch` line
   with `applied == ops` and `hurried=-`, and at least two slices whenever the
   engine time exceeds 400 ms (`SPREAD_MS`), so a job that ran in one frame
   cannot pass on a fast machine;
3. **one undo entry** — once Properties re-reads the typed size, one Ctrl+Z
   brings the re-read size back to the original.

By default the check writes its own page: one `BT … ET` of 120 lines at 4 pt.
`--pdf FILE --doc-point PAGE,X,Y` drives any other file; the check copies it to
`target/ui-verify/restyle-keeps-painting.pdf` and never touches the original.

Driven off the desktop through the scripted pointer, under `--no-input`.

## Driven

| input | doc-point | verdict |
|---|---|---|
| fixture | default | PASS: longest frame 112 ms; `ops=120 applied=120 slices=10 engine_ms=469 wall_ms=858 hurried=-`; one Ctrl+Z 6.00 → 4.00 |
| `D:/Dev/pdfTests/SW41177/SW41177.pdf` (private, never checked in) | `0,1215,595` | PASS: longest frame 291 ms (the post-commit Properties re-read of the 425-run object; slice frames ≤ 169 ms); `ops=238 applied=238 slices=124 engine_ms=11012 wall_ms=17963 hurried=-`; one Ctrl+Z 6.00 → 13.23 |

The frame prelude skips the page's content generation (~40 ms on SW41177) and
the selection's split operand (~45 ms) while a job runs (`app::frame`). With
them, slice frames on SW41177 measure 200–536 ms and the job 35–65 s of wall
time; the check's 800 ms bound does not see that difference, the batch line's
`wall_ms` does.

## Falsification

| control build | expected verdict |
|---|---|
| the v0.97.0 release (restyle in one frame), `--exe … --allow-stale`, SW41177 | FAIL at step 1: "the window stopped painting for 27951 ms (limit 800 ms)" — watched red |
