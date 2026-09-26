# `panels::pages::thumbnails` — what gets drawn, when, and what is kept

The Pages panel's **rendering and caching policy**, separated from the
panel body so the decisions are readable and testable without an
`egui::Context`. The body asks three questions of this module — *what is
the state of tile N?*, *what should I draw next?*, *draw it* — and this
module owns every answer.

---

# ★ Small does not mean fast. The whole design follows from that.

The obvious mental model of a thumbnail grid — *200 pages, 200 cheap
little pictures* — is **wrong on this application's flagship document**,
and `BENCHMARK.md` measured how wrong:

| Render | Pixels | Cost |
|---|---:|---:|
| full page, scale 1 | 1,002,822 | 877 ms |
| a 400 × 300 pt region | 120,701 | 699 ms |
| **a 1 × 1 POINT region** | **2** | **691 ms** |

**A two-pixel render costs 691 ms.** On the benchmark CAD drawing ~99 % of
the cost is *resolution-independent*: 148,517 content-stream operators
walked through a sequential state machine at ~5 µs each, and the pixel
fill is the small remainder. Scaling a thumbnail down to a postage stamp
saves the fill and nothing else.

So a 200-page drawing set is not 200 cheap thumbnails. It is potentially
**two and a half minutes** of interpretation, and any design that starts
it eagerly has hung the application.

Two further measurements shape what is *not* done here:

- **Tiling is cancelled**, not deferred. `BENCHMARK.md`'s ceiling section
  records a 3 × 3 ring of regions as a **9× regression**, for the reason
  in the table above.
- **A display list is the real fix** and is not this panel's to build. It
  would make the second and every later render of a page nearly free, and
  it lands in `pdfcer-render`.

---

# The policy, stated

| Question | Answer |
|---|---|
| **What is rendered?** | one page, at [`THUMBNAIL_WIDTH_PTS`] wide, with annotations painted and no layer override — the reader's view of the sheet |
| **When?** | at most **one page per frame**, and only for a tile the operator can actually see |
| **In what order?** | the **current page first** if it is on screen, then the visible tiles in reading order |
| **What is cached?** | the uploaded texture, keyed by the page's [`RenderKey`], up to [`MAX_CACHED_THUMBNAILS`] of them |
| **What is evicted?** | the cached page **furthest from the middle of what is on screen** |
| **What does an undrawn tile show?** | *words* — see [`TileState`] and [`crate::text::pages`] |
| **When does a page get skipped?** | when it exceeds the operator's own per-page time limit — [`ThumbnailCache::budget`]. **There is no limit by default** ([`PAGE_BUDGET_DEFAULT`]); when one is set, that page alone is abandoned and the grid carries on with the next one |
| **When does the feature switch itself off?** | **never** — see "the skipping rule" below |

## What that policy does on real documents — measured, by driving the
binary

`PDFCER_DIAG=1 pdfcer-gui <file>`, release build, reading the
`pages-thumbnail` and `pages-panel` trace lines:

| Document | What happened |
|---|---|
| `SW41177.pdf` — 36 SolidWorks sheets | 12 tiles visible, 12 drawn, one per frame in 61 · 222 · 48 · 52 · 33 · 31 · 31 · 31 · 32 · 32 · 33 · 33 ms. Then `drawn=12` and **nothing further scheduled** — the other 24 pages were never touched. |
| `ncored-benchmark-cad-drawing.pdf` — 1 sheet | page 1 drew in **921 ms**, tripped the then-400 ms `SLOW_PAGE`, and the panel reported `previews=0` on the same frame. |


## ★ Why this renders on the UI thread, when a cancellable off-thread
worker already exists

`crate::render::worker::RenderWorker` is the right tool and this module
does not use it. That is a finding, not an oversight, and it is recorded
here because the next person to look at this file will reach for the
worker within a minute.

A [`crate::render::worker::RenderRequest`] carries
`session: Arc<EditSession>`, and the worker thread holds that clone **for
the whole render** — which is the point: it is what lets the borrow cross
the thread boundary. `crate::app::state::OpenDoc::session`'s own docs spell
out the consequence:

> every future mutation must go through a path that first calls
> `RenderWorker::cancel_and_wait` — `Arc::get_mut` fails while a render is
> running

This build has exactly two such paths — `crate::app::actions`' private
`vector_edit` and `crate::panels::forms::edit::apply` — and **both cancel
only `OpenDoc::render_worker`.** A *second* worker, owned by
[`crate::panels::PanelsState`], is invisible to both. So a thumbnail in
flight when the operator pressed Delete, dragged an object, or typed into
a form field would make `Arc::get_mut` return `None`, and the edit would
be **traced as `reason=session-borrowed` and silently declined**.

On the benchmark drawing that hazard window is not a millisecond. It is
~0.74 s per page for every page the operator scrolls past — tens of
seconds of a document that quietly refuses to be edited, on the exact
documents this application exists for.


**What would close this properly** is one of:

1. moving a second `RenderWorker` onto `OpenDoc` beside `render_worker`,
   so the existing `cancel_and_wait` calls can reach it (`app/state.rs`,
   `app/actions.rs`, `panels/forms/edit.rs`); or
2. `BENCHMARK.md`'s own recommendation 2 — a **thread pool** for
   thumbnails and adjacent-page prerender — which needs the same
   cancellation reach and would then fill the grid in proportion to core
   count, because pages are independent of each other.

Neither is this panel's to build, and doing half of either would ship the
silent-refusal defect.

## Why one page per frame rather than the old shell's two

The old shell used `THUMBNAILS_PER_FRAME = 2` with no time
bound at all. Two is twice the worst-case hitch for the same throughput,
and throughput is not what a thumbnail grid is short of — an operator
reads a rail one screenful at a time. One page per frame means the window
repaints, the scroll responds, and the operator can turn previews off,
*between* every page.

## ★★★ The skipping rule — per page, and the checkbox NEVER moves itself


> *"the drawing page previews checkbox should never automatically turn
> off. You can add a box next to the checkbox to enter a timeout value."*

What was there before: a page costing more than a hard-coded 400 ms set
`ThumbnailCache::slow`, **the checkbox went from ticked to unticked**, and
no further page was drawn until the operator ticked it again. The
reasoning was sound as far as it went — the operator cannot know in
advance which of their documents is the expensive kind, pdfcer can after
one page, and it said so rather than going quiet.

⇒ **What that reasoning missed is that it wrote its conclusion into the
operator's own control.** A checkbox is a record of an instruction. When
pdfcer clears it, the operator's next glance at the panel reads *"I must
have turned that off"*, and there is no state left that distinguishes
*they chose this* from *pdfcer chose this for them*. The old
`forced: Option<bool>` existed entirely to paper over that collision, and
its own doc comment admitted the shape of the problem — three states for a
two-state control.

The rule now:

- **The tick is the operator's, exclusively.** Nothing in this module
  writes it. [`ThumbnailCache::previews_on`] is a plain field read.
- **The cost is bounded per page, not per document.** Each render is armed
  with a [`RenderCancel`] at [`ThumbnailCache::budget`]; a page that
  exceeds it is abandoned, shows [`TileState::Abandoned`], and **the grid
  moves on to the next page**. One expensive sheet in a set of thirty-six
  no longer costs the other thirty-five their pictures — which is strictly
  better than the old rule even before the operator touches anything.
- **The budget is the operator's number**, typed into the box beside the
  checkbox, clamped to [`MIN_PAGE_BUDGET`]..=[`MAX_PAGE_BUDGET`]. Their
  machine, their choice — what pdfcer owes them is the measurement, and
  [`crate::text::pages::previews_budget_tooltip`] carries it.
- **Raising the budget retries what the old one skipped**
  ([`ThumbnailCache::set_budget`] drops every [`Unavailable::Abandoned`]
  entry). A dial that could only ever remove pictures would be a trap: the
  operator would raise it, see no change, and conclude it did nothing.
- **Stated, not silent** — [`crate::text::pages::previews_skipped_note`]
  names the page, what it was given, and the box that changes it.

⚠ The price, stated plainly because it is the operator's to pay now: on a
document like the benchmark drawing, twelve visible tiles at ~0.92 s each
is ~11 s of UI-thread work, spread one page per frame with the window
repainting between each. It is a slow grid, not a frozen one, and it was
**precisely the thing the old rule bought by unticking the box**. The box
is how the operator buys it back.
