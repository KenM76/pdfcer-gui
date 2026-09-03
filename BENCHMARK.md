# Rendering benchmark — real CAD drawing

**Measured:** 2026-08-12
**Subject:** `D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf`
**Raw trace:** `evidence/bench-gui-diag.txt` (773 lines, `PDFCER_DIAG=1`)

This file exists because an earlier draft of `GUI_ROADMAP.md` asserted
that pdfcer's whole-page raster was a performance weakness and should be
replaced with a tile cache. That was reasoned from architecture and
never measured. The operator's contrary report — that pdfcer's zoom and
pan felt *faster and more pleasant* than the tiled competitor's —
prompted this measurement, and **the operator was right**.

---

## ★★★ 2026-08-30 — the EDIT half, measured for the first time

**Instrument:** `crates/pdfcer-gui/src/app/actions/latency.rs`, a `#[ignore]`d
release-mode test. Run it with:

```
cargo test -p pdfcer-gui --release edit_latency -- --ignored --nocapture
```

**Why it was written.** `OPERATOR_REQUESTS.md` **O63** asks for a live preview
that stays on screen *"while the update to the pdf structure runs in the
background"*. Two completely different fixes follow depending on whether the
delay an operator feels is the **commit** (`pdfcer-core` rewriting the page) or
the **raster** (`pdfcer-render` redrawing it). Nobody had a number for the
commit. This file exists because the last time this project answered a
performance question from architecture rather than measurement, it was wrong.

### The numbers

| | ordinary A1 title block | **dense CAD site plan** |
|---|---:|---:|
| `Document::load` | 0.4 ms | **3.6 ms** |
| `EditSession::new` (incl. load) | 0.3 ms | 3.6 ms |
| `EditSession::view` | 0.000 ms | **0.000 ms** |
| `decompose_page` | 0.5 ms | **500.9 ms** |
| `move_objects` (incl. load + session) | 0.9 ms | **434.3 ms** |

Medians of 3–5 samples. Same machine, same session, release build.

### ★★★ What this says, and it is not what the prior said

**The prior was wrong.** *"It is obviously the raster"* was the comfortable
answer — a second to rasterise this page is already recorded above — and it is
**not** the whole story. One drag-move on this drawing costs the engine
**430 ms** before anything is drawn at all.

Three findings, in order of how much they change the plan:

1. **The commit half is real and it is on the UI thread.** `vector_edit` runs
   `move_objects` synchronously inside the frame. Four hundred and thirty
   milliseconds is not a delay, it is a **freeze** — the window stops answering
   the pointer. Nothing about a preview fixes that; it has to move off the UI
   thread, or get cheaper, or both.

2. **Opening the file is free and reading it is free.** `Document::load` is
   3.6 ms on a 5.6 MB drawing and `EditSession::view` is *unmeasurably* fast.
   ⇒ **The cost is not the file and it is not the object graph. It is the
   CONTENT STREAM.** `decompose_page` — parsing 129,758 objects' worth of
   operators — is 501 ms, and `move_objects` is 434 ms, and the two numbers
   being within 15 % of each other is the whole story: **the verb's cost is
   essentially one decomposition.**

3. ★★ **And the shell then pays for a second one.** `app::cache::page_objects`
   is keyed on `(page, edit_epoch)` and `vector_edit` bumps the epoch, so every
   commit throws away the decomposition it just implicitly performed and the
   next frame rebuilds it from scratch. **That is ~500 ms of duplicated work per
   edit, and it is pure loss** — the same page, the same content, parsed twice
   because the two parsers cannot see each other across the crate boundary.

### The bill for one drag-move on this drawing

| step | cost | where |
|---|---:|---|
| `render_worker.cancel_and_wait()` | 28.9 ms | shell, UI thread (`worker.rs:836`) |
| `move_objects` | ~430 ms | engine, UI thread |
| re-`decompose_page` for the selection outlines | ~500 ms | shell, UI thread |
| re-rasterise the page | ~1,000 ms | engine, **worker thread**, stale frame stays up |
| **≈ 1 s frozen, then ≈ 1 s stale** | | |

★ Only the last row is already off the UI thread, and it is the only row O63's
existing machinery covers. The first three are the freeze.

### What has been filed as a result

`request_one_edit_costs_two_decompositions_of_the_same_page.md` — the engine
cannot see the shell's decomposition and the shell cannot see the engine's, so
one edit parses one content stream twice. Neither side can fix that alone.

---

## The document

| Property | Value |
|---|---|
| Size | 5.6 MB |
| Version | PDF 1.7 |
| Pages | 1 |
| Content | Dense vector site / utilities plan — services overlays, hatched zones, tree symbols, a full legend table, coordinate schedule, title block |
| pdfcer verdict | **"Rendered faithfully — no font substitutions or unsupported content on this page"** |

This is a genuinely hard page, not a synthetic stress test. At scale 1 it
takes pdfcer roughly a second to rasterize.

---

## Headless rasterization (`pdfcer render-page`)

Wall time includes process start, parse and PNG encode, so treat these
as relative rather than absolute engine cost.

| Scale | Wall time | Output |
|---:|---:|---:|
| 1× | 966 ms | 1.34 MB |
| 2× | 1 506 ms | 3.72 MB |
| 4× | 2 187 ms | 9.24 MB |
| 8× | 3 836 ms | 21.36 MB |

Cost grows sub-linearly in pixel count — 64× the pixels from 1× to 8×
costs about 4× the time — so the fixed parse and content-stream walk
dominate, not the fill rate.

---

## In the GUI — the result that matters

From `evidence/bench-gui-diag.txt`. The whole session produced exactly
**eight render generations** and **three completed renders**:

```
render-async-started gen=1 budget_ms=12
render-async-done    gen=1 ms=830  outcome=done      ← open, initial
render-async-started gen=2 budget_ms=12
render-async-done    gen=2 ms=827  outcome=done      ← fit-page settle
render-async-started gen=3 budget_ms=12              ┐
render-async-started gen=4 budget_ms=12              │ six zoom clicks,
render-async-started gen=5 budget_ms=12              │ 1.0 → 1.25 → 1.5
render-async-started gen=6 budget_ms=12              │ → 2 → 3 → 4
render-async-started gen=7 budget_ms=12              │
render-async-started gen=8 budget_ms=12              ┘
render-async-done    gen=8 ms=1899 outcome=done      ← ONE render, at the destination
```

**Six rapid zoom steps started six render generations and completed
one.** Generations 3 through 7 were superseded by the generation counter
and abandoned mid-flight by the cancellation token. The user saw the
previous texture linearly scaled throughout, then one crisp render at
the final zoom.

Total cost of that zoom sequence: **1 899 ms**. Rendering each step
would have cost roughly 6 × 1.9 s ≈ 11 s of work for five frames nobody
asked to look at.

### Why this refutes the original critique

The claim was *"no tile cache, so zooming a large sheet re-rasterizes
the entire page."* The first half is true and the second half is
misleading. It re-rasterizes the entire page **once, at the
destination** — not once per step. The generation counter plus the
150 ms `ZOOM_SETTLE` debounce already solve the problem a tile cache
would have been introduced to solve, by a different and cheaper route:
don't render what the user is scrolling past.

What a tile renderer buys is a better *time to first pixel* at the
destination. What it costs is smoothness — it paints piece by piece, and
it does work at every intermediate step. On this document, pdfcer's model
is the better trade, which is exactly what the operator observed in use.

The in-frame wait budget is 12 ms (`budget_ms=12`), so a page fast
enough to render inside one frame is indistinguishable from synchronous
behaviour, and this page — which is not — never blocks the UI.

---

## Resource use, head to head

Same file, same window size, both at 95 % fit-page zoom.

| | pdfcer | the comparison product |
|---|---:|---:|
| Processes | **1** | 5 (a UI process + 4 render workers) |
| Working set | **170 MB** at open, 231 MB after zooming to 4× | **569 MB** total |
| CPU to steady state | 7.4 s | 5.0 s |
| Render fidelity | faithful, self-reported and correct | correct |

pdfcer uses roughly **2.5× less memory** in a single process. The
comparison product spends its extra memory on crash isolation — a bad page cannot
take down its UI — which is a real benefit pdfcer does not have. Both are
defensible; neither is free.

### A comparison-product defect, reproduced

On this file, as on the A1 title-block frame tested earlier, the comparison
product's Properties panel reports `Pages -`, `Page Size -`, and every
metadata field blank, plus `Annotations Total 0`. pdfcer reports
`PDF 1.7, 1 page(s)` correctly. Two different documents, same failure —
this is not file-specific.

---

## Would more processes make rendering faster?

**No — and threads only help the smaller half.** Measured with the
project's own `tools/render-profile` (which enables the
`pdfcer-render/profile` feature), `--repeat 3`, fastest run:

```
load      : 1.686 ms   (object graph + xref only)

  scale        pixels      render      step    per Mpx
   0.25         62878       0.74s         —     11.79s
    0.5        250916       0.76s     1.02x      3.02s
      1       1002822       0.83s     1.09x      0.83s
      2       4011288       1.31s     1.58x      0.33s

content (at 2x):
  paints            : 148517
  clip operations   : 24128
    mask cache      : 24087 hits + 41 built (99.83% served)
clip construction, timed per phase: 0.02s total
```

### The decomposition

At scale 0.25 the page is 62,878 pixels — fill is negligible — and the
render still costs **0.74 s**. That is the floor, and it is
resolution-independent:

| Component | Cost | Nature |
|---|---:|---|
| Load (xref + object graph) | 1.7 ms | trivial |
| **Content-stream interpretation** | **~0.74 s** | **inherently sequential** |
| Rasterization fill | 0.09 s at 1×, 0.57 s at 2× | parallelisable |
| Clip construction | 0.02 s | already solved — 99.83% cache hit rate |

The 0.74 s is **148,517 paint operations at roughly 5 µs each**, walked
through a state machine. That is where essentially all the time goes at
fit-page zoom, and it is 57 % of the time even at 2×.

### Why that part cannot be parallelised

A PDF content stream is a sequential state machine. `q`/`Q` maintain a
graphics-state stack; `cm` concatenates the CTM; colour, line width and
the clipping path all accumulate. **Operator *N*'s meaning depends on the
state left by operators 1…*N*−1**, so you cannot start a worker in the
middle of the stream — it does not know where it is or what colour it is
drawing in. Splitting the stream requires first walking it, which is the
thing you were trying to split.

### Why processes specifically are the wrong tool

Processes buy **crash isolation**, not throughput. The comparison product runs
four `pdfium-worker` processes and pays 569 MB for it — a bad page cannot
take down its UI, which is a genuine benefit. But pdfcer is a single Rust
binary: threads already share memory for free, while processes would have
to ship a multi-megabyte pixmap back across a pipe on every render. There
is no speed argument for processes here, only a robustness one.

### What threads would actually buy — Amdahl, on real numbers

Parallel band rasterization is embarrassingly parallel: split the pixmap
into *N* horizontal strips and replay the draw list into each. On 10
physical cores, assuming perfect scaling of the fill only:

| Zoom | Now | Parallel fill, 10 cores | Speed-up |
|---|---:|---:|---:|
| 1× (≈ fit page) | 0.83 s | 0.75 s | **1.11×** |
| 2× | 1.31 s | 0.80 s | **1.64×** |

Worth having at high zoom. Nearly pointless at the zoom the operator
actually sits at. Ten cores buy 11 % at fit-page because 89 % of the work
is the sequential walk.

### The bigger win is not parallelism at all

`render_page(doc, page, scale)` (`pdfcer-render/src/lib.rs:165`) is a pure
function with **no state retained between calls**. Every zoom change
re-walks all 148,517 operators from scratch — even though the geometry
has not changed. Only the transform has.

**Build a display list once, replay it at any scale.** That turns a
re-render at a new zoom from ~830 ms into just the fill — roughly 90 ms
at 1×, 570 ms at 2×. It is a larger win than ten cores, it applies at
every zoom rather than only at high zoom, and it composes with parallel
fill rather than competing with it. Invalidation is already a solved
problem here: the GUI's raster cache keys on five staleness inputs and
knows exactly when the content changed.

### Where parallelism pays immediately, today

**Pages are independent of each other** — no sequential dependency — so
multi-page work parallelises cleanly with threads and no new
architecture:

- **Thumbnails.** Currently rasterized at `THUMBNAILS_PER_FRAME = 2`
  (`main.rs:392`) on the single render worker. A 42-page sheet set is
  ~21 frames of serialised work. A thread pool would fill the rail
  roughly in proportion to core count.
- **Adjacent-page prerender.** Page ±1 rendered speculatively on spare
  cores makes paging through a sheet set feel instant, and costs nothing
  the user waits for.

### Recommended order

1. **Display list, built once and replayed per zoom** — kills the 0.74 s
   on every zoom change. Biggest single win, at every zoom.
2. **Thread pool for thumbnails and adjacent-page prerender** — pages are
   independent; this is free parallelism with no interpreter changes.
3. **Parallel band fill (rayon)** — then attacks what remains; 1.6× at
   high zoom, marginal at fit-page.
4. **Processes** — only if crash isolation is wanted for its own sake.
   Costs memory and IPC, buys no speed.

---

## The zoom ceiling — and why it makes tiling necessary after all

**Operator requirement, 2026-08-13:** *"I want to be able to zoom in as
much as feasibly possible — preferably further than other software
allows."*

`MAX_ZOOM` is nominally 8.0 (800 %). It is not the real limit.
`viewer::max_zoom_for_page` lowers the ceiling per page so a render
cannot exceed `pdfcer_render::MAX_PIXMAP_EDGE` (16,384 px), because a
whole-page raster's edge scales with zoom:

```
ceiling = (16384 − 1) / (longest_page_edge_pt × pixels_per_point)
```

Worked through for real sheet sizes:

| Sheet | Longest edge | Ceiling @ 1× DPI | Ceiling @ 2× DPI |
|---|---:|---:|---:|
| A4 portrait | 842 pt | 19.5× | 9.7× |
| **A3 landscape** *(the benchmark drawing)* | 1191 pt | **13.8×** | **6.9×** |
| A2 | 1684 pt | 9.7× | 4.9× |
| A1 landscape | 2384 pt | 6.9× | 3.4× |
| A0 | 3370 pt | 4.9× | 2.4× |

> **⚠ Corrected 2026-08-13.** This table originally labelled the benchmark
> drawing **A1 landscape, 2384 pt**, and quoted its ceiling as 6.9× / 3.4×.
> That was an assumption, never measured. Its `MediaBox` is
> **1190.55 × 841.89 pt — A3 landscape** (`region_bench` reports it on the
> first line, and `1191 × 842 = 1,002,822 px` matches the scale-1 pixel
> count exactly). Every figure for it was out by 2×.
>
> Caught by driving the running application: the status bar read **147 %**
> for fit-page and that looked impossible for an A1 sheet in that window —
> so the number that turned out to be wrong was mine, not the status bar's.
> The arithmetic in the other rows was always right; only the row label
> was.
>
> **The conclusion is unaffected**, which is worth stating so the
> correction is not read as bigger than it is: 6.9× on a HiDPI display is
> still below `MAX_ZOOM`, still falls as sheets grow, and the region API
> was still the right answer. The pdfcer team quoted the 3.4× figure back
> in their reply; `open/request_reusable_parsed_handle.md` carries the
> correction to them.

**So on the operator's own benchmark drawing, on a HiDPI display, zoom
stops at 6.9× — not 8×, and an A1 sheet would stop at 3.4×.** The larger
the sheet, the lower the ceiling, which is exactly backwards from what a
drafting reviewer needs: the big sheets are the ones with detail worth
magnifying.

Raising `MAX_ZOOM` alone changes nothing. The constant is not the
binding limit; **the whole-page rasterization strategy is.**

### This is what the tiled path is actually for

`GUI_ROADMAP.md` deferred tiling on the grounds that whole-page
rendering measured better for pan and zoom, and that finding stands —
for *navigating* a sheet. But a tile's pixel dimensions are fixed by the
tile size, not by the zoom, so the pixmap ceiling stops scaling with
magnification and the practical zoom limit becomes memory for the
visible region alone. Tiling is what makes "further than other software
allows" achievable rather than aspirational.

That reverses the earlier conclusion in one specific respect, and it is
worth being explicit about which: whole-page stays the better default
for *motion*; tiling is required for *magnification beyond the pixmap
ceiling*. They are answers to different questions, which is precisely
why the strategy is exposed as a **View ▸ Render** choice rather than
decided once.

**Consequence for the plan:** the tiled path moves from "build only if
someone finds a sheet where whole-page hurts" to **a scheduled
requirement**, because a sheet where whole-page hurts has been found —
it is the benchmark drawing, at 3.4×.

> ## ⛔ SUPERSEDED, 2026-08-13 — do not tile
>
> Everything in the two sections below reasons about *tile geometry* on the
> assumption that a tile's cost is dominated by its **fill**. It is not.
> The pdfcer team shipped `render_page_region` and measured the floor
> directly, and the answer overturns the plan:
>
> | case | pixels | time |
> |---|---:|---:|
> | full page, scale 1 | 1,002,822 | 877 ms |
> | region 400×300 pt, scale 1 | 120,701 | 699 ms |
> | **a 1 × 1 POINT region** | **2** | **691 ms** |
>
> **A two-pixel render costs 691 ms.** On the benchmark drawing ~99 % of
> render cost is area-independent, so a 3 × 3 ring costs
> **9 × 0.7 s ≈ 6.2 s** against ~0.7 s for one region covering the same
> area. Tiling on this engine is a **9× regression**, not an optimisation.
>
> Re-measured on a text-heavy A4 page (`iso32000-2-preview.pdf`) at this
> project's end, because a dense CAD sheet is the best case for that
> argument: floor 3.2 ms of an 8.97 ms full page — **36 %, not 99 %**, and
> a ring penalty of ~1.9× rather than 9×. So *never tile for speed* holds
> on both; *tiling is a catastrophe* holds only where interpretation
> dominates.
>
> **The plan is now one region per viewport**, and the over-render ring
> below is retained only as the reasoning that had to be discarded — the
> hypothesis was sound given what was known, and what was known was wrong
> about which term dominated. `GUI_ROADMAP.md` § Rendering carries the
> current plan.

### Tile geometry — the over-render ring, and why overlap is the expensive way to get it

**Operator hypothesis, 2026-08-13:** overlap tiles by ~⅓ on their outer
edges, so some rendering exists beyond the visible window; a pan then
reveals detail rather than fuzz, and memory is freed behind as tiles
leave in the opposite direction.

**The goal is right and it is the single most important property of a
tiled viewer.** Rendering only what is visible guarantees that every pan
shows stale or blank pixels at the leading edge. Some quantity of
already-rendered content beyond the viewport is not an optimisation, it
is the difference between a tiled viewer that feels good and one that
feels worse than what pdfcer does today.

**The mechanism is where it gets expensive, and it can be had cheaper.**
What produces coverage beyond the viewport is *rendering a larger area*.
Overlap does not add coverage — it re-renders the same content into two
neighbours. Reaching the same ring with a wider grid of **non-overlapping**
tiles costs strictly less.

Worked through on the operator's own window (3456 × 1408 device px,
4.87 Mpx, and this crate's measured fill rate of ≈ 7 Mpx/s
single-threaded on the benchmark drawing):

| Scheme | Rendered area | Memory | Fill, 1 core | Fill, 10 cores |
|---|---:|---:|---:|---:|
| Viewport only | 4.9 Mpx | 19 MiB | 0.70 s | 70 ms |
| **⅓ ring, non-overlapping** | 13.5 Mpx | **54 MiB** | 1.93 s | **193 ms** |
| ⅓ ring, ⅓-overlapping tiles | 30.5 Mpx | 122 MiB | 4.35 s | 435 ms |

A ⅓ overlap means a stride of ⅔ of a tile, so every unit of page area is
covered `(3/2)² = 2.25×` over. That is 2.25× the memory **and** 2.25× the
rasterisation, for identical coverage. It more than doubles the latency
of the thing the idea exists to make fast.

**The cheaper form of the same idea:** keep tiles non-overlapping, and
make the *resident set* the viewport expanded by a margin — a ring of
tiles around the visible ones. Eviction works exactly as described:
tiles that fall outside the ring on the trailing side are dropped. Same
behaviour, 2.25× less work.

### Where a small overlap is genuinely required

Not for coverage — for **seams**. A stroke that straddles a tile
boundary, rasterised independently in each tile with a hard clip, gets
its anti-aliased coverage computed twice against two different
backgrounds, and the join shows as a visible hairline. The fix is a
**bleed** of a few pixels — enough to cover the widest stroke half-width
plus any blur radius — rendered and then discarded on composite.

So the answer to "should tiles overlap?" is *yes, by a few pixels, for a
completely different reason than the one proposed*. Pixels, not a third.

### Is it fast enough that a fast pan keeps detail?

Plausibly, and the arithmetic says where the risk is.

During a steady pan only the **newly exposed strip** needs rendering,
not the whole ring. For a 1-tile-deep ring at 512 px tiles, crossing one
tile boundary exposes one row or column — 7 tiles on this window,
1.8 Mpx, ≈ 26 ms on ten cores. That comfortably outruns a pan.

**The risk is not the fill, it is the 0.74 s interpretation floor.** If a
tile render re-walks the content stream, one exposed strip costs 7 ×
0.74 s and the idea collapses. Everything above assumes a **shared
display list** — parse once, replay clipped per tile. That is why the
region-rasterisation request filed at
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\request_region_rasterisation.md`
leads with that property rather than with the region parameter itself:
the parameter without the property would be worse than what exists.

Tiles are also embarrassingly parallel once a display list exists, which
is where the earlier "parallel fill buys 1.11× at fit-page" finding
inverts — at high zoom with tiling, fill is *all* there is, so it scales.

### What still needs deciding

- **A sensible upper bound.** Unbounded zoom is not a feature; at some
  magnification a PDF's own coordinate precision and the renderer's
  fixed-point arithmetic stop being meaningful. The number should come
  from where the *output* degrades, measured, not from a round figure.
- **Whether the ladder extends** or the top rungs become continuous.
  `ZOOM_LADDER` currently tops out at 8×; provable reversibility of
  ladder stepping is a tested property that must survive the change.

## What this changes in the plan

1. **Whole-page rendering stays the default.** It is not a stopgap; it
   is the better model for this workload.
2. **A tile path becomes an option to offer, not a rewrite to survive.**
   It belongs on the View ▸ Render group as an opt-in for someone whose
   sheet is big enough that time-to-first-pixel beats smoothness. It
   should be built only after a document is found where whole-page
   genuinely hurts — and this 5.6 MB drawing is not that document.
3. **`ZOOM_SETTLE` and the raster-scale multiplier should be exposed.**
   At 830 ms per render, 150 ms of settle is well judged. On a lighter
   sheet it is dead time; on a heavier one it is too eager. That is a
   setting, per R169.
4. **Instrument before optimising further.** `PDFCER_DIAG` already emits
   `render-async-done gen=N ms=M outcome=…`, which is most of a
   performance harness. Adding page complexity — operator count, path
   count, resource count — to that line would make it a complete one.

---

## Method, for repetition

```powershell
$env:PDFCER_DIAG = "1"
Start-Process "D:\Dev\pdfcer\target\release\pdfcer-gui.exe" `
  -ArgumentList '"D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf"' `
  -RedirectStandardError diag.txt
# …drive the app…
Select-String "render-async" diag.txt
```

Headless timing:

```powershell
$exe = "D:\Dev\pdfcer\target\release\pdfcer.exe"
foreach ($s in 1,2,4,8) {
  Measure-Command { & $exe render-page $f --page 1 --scale $s -o "out_$s.png" }
}
```

**Caveat.** Single run per data point, on a warm cache, on one machine,
with the process-start cost included in the CLI figures. These are
sound enough to overturn an unmeasured architectural claim; they are not
sound enough to tune a constant against. If `ZOOM_SETTLE` is going to be
chosen from data rather than judgement, take repeated runs across
several documents first.
