# BENCHMARK.md — rendering and edit performance

What a render and an edit cost on a hard page, which half of each an operator
can feel, and the instruments that answer both. Read it before proposing a
rendering optimisation: the standing rule is **measure first**, and an
architectural argument for a tile cache on this engine has already been
overturned by a stopwatch.

## The subject

`D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf` — 5.6 MB, PDF 1.7, one
A3-landscape sheet (1190.55 × 841.89 pt) of dense vector site plan: services
overlays, hatched zones, tree symbols, a legend table, a coordinate schedule
and a title block. A genuinely hard page, not a synthetic stress test, and the
one every figure below is keyed to.

It lives outside both repositories rather than in `fixtures/`, because a
benchmark corpus that grows by copying is a repository that grows without
bound. Every instrument names it by absolute path and **skips with a printed
reason** when it is absent — a machine without the drawing has not found a
defect. `fixtures/a1-titleblock.pdf` is the contrast page, so a slow number can
be attributed to the verb rather than to the sheet.

## The instruments

| Question | Command |
|---|---|
| where the render time goes, phase by phase | `cargo run --release -- <file.pdf> --page 1 --scales 0.25,0.5,1,2 --repeat 3` from `D:\Dev\pdfcer\tools\render-profile`, which enables the engine's `pdfcer-render/profile` feature |
| what one whole-page raster costs end to end | `pdfcer.exe render-page <file.pdf> --page 1 --scale S -o out.png`, timed by the shell |
| what the running GUI does per gesture | set `PDFCER_DIAG=1`, drive the app, filter stderr for `render-spawn`, `render-inline`, `render-async-done`, `page-objects-built` |
| which half of an edit is the slow half | `cargo test -p pdfcer-gui --release edit_latency -- --ignored --nocapture` |

`PDFCER_DIAG` writes one `render-spawn gen=N page=P scale=S` for every
generation started and one `render-inline`/`render-async-done gen=N ms=M` for
every generation finished, which is most of a performance harness; adding page
complexity — operator count, path count, resource count — to those lines would
make it a complete one. `page-objects-built page=P objects=N leaves=N ms=M` is
the decomposition's own line.

The edit instrument is `crates/pdfcer-gui/src/app/actions/latency.rs`, an
`#[ignore]`d release-mode test — `#[ignore]` because it wants a drawing that is
in neither fixture corpus, and `--release` because a debug-build measurement of
a release-build question measures nothing. It reports **medians**, not means
and not bests: a mean is dragged by one scheduler hiccup, and a best is a
number the operator never experiences. It asserts that a measurement *happened*
and never a threshold — a threshold here would be a number nobody chose on
hardware nobody specified, and the first slow machine would turn a measurement
into a red suite.

## Whole-page rasterisation is the default, and it is the right one

Order of magnitude on the subject drawing, rasterising at scale 1 through the
engine's region entry point. **A reference for the shape of the cost, not
values to cite** — re-measure with the commands above.

| What is rasterised | Pixels | Time |
|---|---:|---:|
| the whole page | 1,002,822 | ~880 ms |
| a 400 × 300 pt region | 120,701 | ~700 ms |
| a 1 × 1 **point** region | 2 | ~690 ms |

A two-pixel render costs nearly as much as the whole sheet. On this page ~99 %
of the render cost is area-independent, and five rules follow from that.

**Never tile for speed.** A 3 × 3 ring of tiles pays the floor nine times
against once for a single region covering the same area — a large regression,
not an optimisation. Re-measured on a text-heavy A4 page
(`iso32000-2-preview.pdf`), because a dense CAD sheet is the best case for that
argument: there the floor is roughly a third of a full page rather than nearly
all of it, so the ring penalty is about 2× rather than 9×. *Never tile for
speed* holds on both documents; *tiling is a catastrophe* holds only where
interpretation dominates.

**One region per viewport.** `render::strategy` decides the tier from numbers
rather than from a mode flag, and its `region_for` asks for the visible
rectangle grown by `OVERSCAN` on each side.

**The resident set is the viewport expanded by a margin.** Rendering only what
is visible guarantees every pan shows stale or blank pixels at the leading
edge. Coverage beyond the viewport is not an optimisation; it is the difference
between a viewer that feels good and one that feels worse than whole-page
rendering.

**Overlap is for seams, in pixels, not for coverage.** A stroke straddling a
region boundary, rasterised independently on each side with a hard clip, has
its anti-aliased coverage computed twice against two backgrounds and the join
shows as a hairline. The fix is a bleed of a few pixels — the widest stroke
half-width plus any blur radius — rendered and discarded on composite.
Overlapping by a third instead multiplies both memory and rasterisation by
`(3/2)² = 2.25` for identical coverage.

**Whole-page rendering is the better model for motion, not a stopgap.** A
whole-page texture is rasterised once per *zoom*; a region once per *position*.
Panning over a whole-page texture is free and at full detail, because the
raster does not depend on where the operator is looking.

### The generation counter is what a tile cache would have bought

`render::worker` numbers every render. A spawn takes the next generation, a
reply whose generation is not the current one is dropped, and the in-flight
render is cancelled through its `RenderCancel` token *before* the next is
started — two rasterisations of a CAD page competing for cores make both
slower, and the old one's output is already known to be unwanted. Six rapid
zoom steps therefore start six generations and complete **one**, at the
destination: the operator sees the previous texture linearly scaled throughout,
then one crisp render at the final zoom.

So *"no tile cache, so zooming a large sheet re-rasterises the entire page"* is
half true and wholly misleading. It re-rasterises the entire page **once, at
the destination**. The generation counter and the zoom-settle debounce already
solve what a tile cache would be introduced to solve, by the cheaper route:
don't render what the operator is scrolling past. What a tile renderer buys is
a better time-to-first-pixel at the destination; what it costs is smoothness,
because it paints piece by piece and does work at every intermediate step.

`render::worker::IN_FRAME_BUDGET` is 12 ms — deliberately under one frame at
60 Hz — and it is the one place the UI thread blocks on rendering. A page that
rasterises inside it is indistinguishable from synchronous behaviour; a page
that misses it hands control back to the event loop and never blocks the UI.

## Where the render time goes

**The floor is content-stream interpretation, and it is resolution-independent.**
At scale 0.25 the subject page is a few tens of thousands of pixels — fill is
negligible — and the render still costs essentially what it costs at scale 1.
Loading the xref and the object graph is a couple of milliseconds. Clip
construction is already solved by the engine's clip-mask cache, which serves
essentially every clip on this page. Everything else is the walk: on the order
of a hundred and fifty thousand paint operations at a few microseconds each,
driven through a state machine. That is nearly all the time at fit-page zoom,
and still the majority of it at 2×.

**Interpretation cannot be parallelised.** A PDF content stream is a sequential
state machine: `q`/`Q` maintain a graphics-state stack, `cm` concatenates the
CTM, and colour, line width and the clipping path all accumulate. Operator
*N*'s meaning depends on the state left by operators 1…*N*−1, so a worker
cannot start in the middle of the stream — it does not know where it is or what
colour it is drawing in. Splitting the stream requires first walking it, which
is the thing being split.

**Processes buy crash isolation, not throughput.** This is a single Rust
binary; threads already share memory for free, where processes would ship a
multi-megabyte pixmap back across a pipe on every render. There is no speed
argument for processes here, only a robustness one.

**Threads buy the fill, and Amdahl caps what that is worth.** Band
rasterisation is embarrassingly parallel — split the pixmap into *N* horizontal
strips and replay the draw list into each — but it scales only the part that is
not the walk. At fit-page zoom the fill is around a tenth of the render, so
even perfect scaling across ten cores buys roughly a tenth; at 2× the fill is
large enough to be worth about 1.6×. Worth having at high zoom, nearly
pointless at the zoom the operator sits at.

**The bigger win is not parallelism.** The engine's `render_page` is a pure
function with no state retained between calls, so every zoom change re-walks
the whole stream even though only the transform changed. **Build a display list
once, replay it at any scale**: that turns a re-render at a new zoom into just
the fill. It is larger than ten cores, it applies at every zoom rather than
only at high zoom, and it composes with parallel fill rather than competing.
Invalidation is already solved on this side — `render::worker::RenderKey`
carries every input a texture's staleness depends on, and comparing it against
the key the shell currently wants is the whole staleness contract.

**Parallelism pays across pages today**, because pages are independent of each
other and exploiting that needs no new architecture:

- **Thumbnails.** `panels::pages::thumbnails` rasterises at most one page per
  frame on the single render worker, under a per-page time limit the operator
  sets; a page that exceeds it is abandoned with a `RenderCancel` and the grid
  carries on with the next. A thread pool would fill the rail in proportion to
  core count, and would need the same cancellation reach the single worker has.
- **Adjacent-page prerender.** Page ±1 rendered speculatively on spare cores
  makes paging through a sheet set feel instant and costs nothing the operator
  waits for.

**Recommended order.** (1) Display list, built once and replayed per zoom —
biggest single win, at every zoom. (2) Thread pool for thumbnails and
adjacent-page prerender — free parallelism, no interpreter changes.
(3) Parallel band fill — attacks what remains. (4) Processes — only if crash
isolation is wanted for its own sake; costs memory and IPC, buys no speed.

## The region tier is for magnification, never for speed

A whole-page raster's edge scales with zoom, so the whole-page tier ends where
the pixmap would exceed the engine's `MAX_PIXMAP_EDGE` (16,384 px):

```
tier boundary = (16384 − 1) / (longest_page_edge_pt × pixels_per_point)
```

The larger the sheet the lower that boundary, which is backwards from what a
drafting reviewer needs, since the big sheets carry the detail worth
magnifying. Above the boundary `render::strategy::for_page` answers `Region`
and the raster becomes a multiple of the **window** rather than of the page, so
memory becomes a function of viewport area instead of zoom. Below it every
request carries `region: None`, and nothing about panning or detail changes.

The zoom limit itself is the operator's `max_zoom_percent` preference, not this
boundary. `viewer::zoom_ceiling` lifts a page's ceiling past
`viewer::max_zoom_for_page` only where the region tier can render, and only
when the operator has asked for more than `viewer::MAX_ZOOM` — below that the
pixmap ceiling still binds, because asking for a raster the engine refuses is
the failure this arrangement exists to prevent. Where a raster is refused
anyway, the ceiling learned from that refusal binds every later gesture and
`app::status::rasterstop` says so on the status bar: a clamp the operator
cannot account for is worse than the error it replaced.

## The edit half

Three rules, from the latency instrument.

**1. The commit is on the UI thread, which makes it a freeze rather than a
delay.** Every verb that changes a document goes through
`app::actions::funnel`'s `vector_edit`, and `apply_actions` calls it inside the
frame. A commit measured in hundreds of milliseconds stops the window answering
the pointer. No preview fixes that; it has to move off the UI thread, or get
cheaper, or both.

**2. The cost is the content stream — not the file, and not the object graph.**
On the subject drawing `Document::load` is a few milliseconds and
`EditSession::view` is unmeasurably fast, while `decompose_page` and
`move_objects` are each hundreds of milliseconds and within about 15 % of each
other. That closeness is the whole story: **a verb's cost is essentially one
decomposition.**

**3. A *content* edit pays for a second decomposition.** `app::cache`'s
`page_objects` is keyed on the engine's `EditSession::page_content_generation`
digest for the page — page id, every `/Contents` entry with its staged span,
the effective `/Resources` — with `edit_epoch` as the fallback when the digest
cannot be read. That digest holds still for an annotation, a stamp, a note or a
ce dimension, so those edits no longer rebuild the model. A content edit moves
it, and the commit's own decomposition has already been thrown away, so the
next frame rebuilds it: the same page parsed twice, because the two parsers
cannot see each other across the crate boundary. Closing that is an engine
change.

One drag-move on this drawing is, in order: `RenderWorker::cancel_and_wait` for
the in-flight raster, the commit, and the re-decomposition the selection
outlines need — all three on the UI thread — and then the page re-raster on the
worker thread, during which the stale frame stays up. Roughly a second frozen,
then a second stale.

## Figures that are routinely misread

- **The 355 ms traced at `canvas/interact.rs` is the page's text extraction**,
  cached on `(page, edit epoch)` thereafter — not the vector hit test, which is
  one linear pass over the page's objects. Cited as hit-test cost it says hover
  hit-testing is impossible on this sheet, and it says nothing of the kind.
- **Wall time from the CLI includes process start, parse and PNG encode.**
  Those figures are relative to each other, not absolute engine cost.
- **Scale figures grow far more slowly than pixel count**, because the fixed
  parse and content-stream walk dominate the fill rate. Sixty-four times the
  pixels is nothing like sixty-four times the time on this page.

## What this method will and will not support

Single run per data point, warm cache, one machine, process-start cost included
in the CLI figures. Sound enough to overturn an unmeasured architectural claim;
not sound enough to tune a constant against.

That is why the zoom settle delay and the raster-scale multiplier are operator
preferences — `app::prefs::quality`'s `zoom_settle_ms` and `RenderQuality` —
rather than compiled-in constants. A settle delay well judged against a page
that takes most of a second to render is dead time on a lighter sheet and too
eager on a heavier one, and choosing it from data rather than from judgement
needs repeated runs across several documents first.
