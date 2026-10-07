# `an_edit_never_blanks_or_blocks`

**Defect it guards.** An edit made every page preview go blank and redraw, and
the canvas page blank until its new raster arrived, while the redraws held the
UI thread so the next click waited (O288 item 1). The required behaviour: an
old picture stays until its replacement is ready, only the pages an edit
touched re-render, and the next input is accepted at once.

**Fixture.** `fixtures/heavy-pages.pdf`, built by `heavy-pages.PROVENANCE.py`:
six letter pages drawing one shared, compressed stream that holds a single
stroked path of 150,500 two-point subpaths, so every render, decomposition and
hit test of a page costs measurable time; plus one filled rectangle per page at
`40 700 120 60`, in the page's own stream, that the move case drags. Because
the lattice stream is shared, a move on page 1 also exercises the engine's
copy-on-write of a shared stream.

**Steps.** Two cases, one launch each, off-screen, with the Pages panel open,
continuous layout and fit-page zoom.

1. Wait until `pages-tiles` reports `pending=0 blank=0`: every visible
   thumbnail drawn.
2. Mark the trace and make one edit on page 1:
   - *move*: Edit mode, drag the rectangle 60 pt right (`move-objects`);
   - *markup*: Review mode, draw a rectangle markup (`add-markup`).
3. Send twelve pointer steps 300 ms apart and time each from send to the
   app's `diag-pointer` acknowledgement.
4. Wait for the thumbnails to settle again, then judge the trace after the
   mark.

**Verdict.** FAIL when any of:
- a `pages-thumbnail` line re-rendered a page other than 1;
- a `render-spawn` named a page other than index 0;
- a `pages-tiles` line reported `blank>0`, or a `canvas-pages-blank` line named
  a page;
- a `frame-long` frame spent more than 120 ms on the GUI's own work: its `ms=`
  less the `ms=` of the engine lines traced since the previous long frame
  (`hit-test`, `page-objects-built`, and the edit's verb line);
- there were more than two `hit-test` lines per `canvas-press` (a drag
  repeating the press's hit test every frame);
- a probe step waited more than 300 ms beyond the edit's own engine time (its
  verb plus the page model rebuilt after it).

Engine time is taken out because the shell cannot move it off the UI thread:
the verb holds `&mut EditSession`, and the page model is needed synchronously
by every canvas query. It is filed as G139 (hit test), G140 (decompose after an
edit) and G141 (the verb on a shared stream), and the report states it.

The report carries the edit's engine time, each re-rendered thumbnail's cost
and every probe latency.

**Measured** on the v0.80.0 pin, the revision that added this check: move —
drag accepted in 621–639 ms, the edit 337–382 ms in the engine, first probe
438–481 ms, later probes 11–73 ms, page 1's thumbnail alone redrawn (186–203
ms on the background slot); markup — no engine time, probes 11–104 ms.

**Falsified.** With `ObjectModelProvider::hit_test_all`'s memo disabled, the
move case failed with *18 engine hit tests for 3 presses* and the drag took
1,589 ms to be accepted against 639 ms with it. Before the async thumbnails,
the funnel's `refresh_content_generation`, and the Objects panel's
`subpath_count` fix, the same check failed on held frames of 543 ms and on a
first probe of 585 ms.

**Traces it relies on.** `frame-long ms= phases=` (any frame of 50 ms or more,
timed from the top of `ui` to `end_ui_frame`, with each phase of 5 ms or more);
`hit-test page= ms=` (an engine point hit test of 5 ms or more that was not
answered from the provider's memo); `page-objects-built ms=`; `pages-tiles
visible= ready= stale= pending= blank=` (de-duplicated); `canvas-pages-blank
pages=` (the strip pages drawn this frame with no picture, `-` for none);
`canvas-press`; `ms=` on the funnel's applied line.

**What it does not prove.** The other edit routes (text, forms, ce dimensions,
page operations) beyond their shared funnel; a document-scoped verb that
genuinely touches every page; GPU upload time, which is not on the UI
thread's clock. Engine time is attributed to the next long frame, so a small
engine call in a short frame could be credited to a later one.
