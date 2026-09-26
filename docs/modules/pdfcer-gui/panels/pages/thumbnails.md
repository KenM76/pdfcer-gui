# `panels::pages::thumbnails` — what gets drawn, when, and what is kept

The Pages panel's **rendering and caching policy**, separated from the
panel body so the decisions are readable and testable without an
`egui::Context`. The body asks three questions of this module — *what is
the state of tile N?*, *what should I draw next?*, *draw it* — and this
module owns every answer.

---

# Small does not mean fast. The whole design follows from that.

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


## Why this renders on the UI thread, when a cancellable off-thread
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

## The skipping rule — per page, and the checkbox NEVER moves itself


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

## Item notes

### `fn default`

`#[derive(Default)]` would give `on: false`, a build that draws nothing
and says nothing about why. That is the kind of default only ever
discovered by an operator.

### `const THUMBNAIL_WIDTH_PTS`

Carried over from the old shell's `raster::THUMBNAIL_WIDTH_PTS`, and the
number is a *raster* width rather than a *layout* width. The grid draws
tiles at whatever size the dock gives it and lets the texture scale
(`LINEAR`, via [`texture_from_pixels`]), so **resizing the dock does not
re-rasterize anything**.

That distinction is worth more here than it was there. Re-rendering on
resize would mean a drag of the dock splitter costing ~0.74 s per visible
tile per frame on a dense drawing — a resize gesture that freezes the
application, from a panel whose entire design is about not doing that.

140 pt at a typical 1.5–2× `pixels_per_point` is a 210–280 px picture,
which is enough to recognise a title block and a sheet layout by. Larger
costs fill time (the *only* part of the render that scales) and memory;
smaller stops being recognisable, which is the one job a thumbnail has.

### `const PAGE_BUDGET_DEFAULT`

# No limit, because a missing preview is a worse answer than a slow one

The operator's rule: *"draw page previews should be set to 'no limit' by
default."* A budget that trips produces a tile with no picture, and the
operator has no way to tell that from a page that failed — the mode they
are in is *looking for a sheet*, and a blank where a sheet should be is the
one outcome the panel exists to prevent. Waiting is visible and
self-explaining; an abandoned render is neither.

The budget itself is unchanged and is still the right control to *have*.
`BENCHMARK.md` records ~10 s at 1× and ~58 s at 2× for a full-size CAD
raster, nothing in the format bounds a thumbnail's cost, and an operator
working through a set of pages that each cost that much wants the dial.
What changed is who reaches for it.

## What a page actually costs, so the size of the risk is on the record

This panel's own measurements — [`tests::thumbnail_cost_on_the_benchmark_documents`]
re-runs them. **Release build, 280 px-wide thumbnails, one core:**

| Document | Page | Thumbnail |
|---|---|---:|
| `ncored-benchmark-cad-drawing.pdf` | 1 | **918 ms** |
| `SW41177.pdf` (36-sheet SolidWorks set) | 2 | 238 ms |
| `SW41177.pdf` | 1, 3, 4 | 58–72 ms |
| `fixtures/a1-titleblock.pdf` | 1 | 9 ms |
| `pageops/four-pages.pdf` | 1–4 | < 1 ms |

⇒ The worst real page measured is under a second, so on every document in
the table an unbounded budget and a two-second one draw exactly the same
thing. The difference only appears on a page worse than anything here, and
there the operator now waits rather than being shown nothing.

# The mechanism, for when a budget IS set

Real rather than nominal: every render is armed with a [`RenderCancel`] and
a one-shot watchdog thread that trips at the budget. `pdfcer-render` polls
the token **between content-stream operators**, and its own docs put the
worst-case latency at one operation — ~360 µs for the most expensive kind
measured.

A render that trips it is [`Unavailable::Abandoned`], which is *not* a
failure: nothing is wrong with the page, and the tile says so in those
terms.

### `const MIN_PAGE_BUDGET`

A tenth of a second. Below this the dial stops being a time limit and
becomes an off switch wearing a number: `SW41177.pdf`'s *cheapest* sheets
cost 58–72 ms, so 50 ms would abandon an ordinary drawing-office sheet
set wholesale while the checkbox still read "on" — which is the exact
confusion between *pdfcer decided* and *the operator decided* that this
whole rewrite exists to remove. An operator who wants no previews has a
checkbox for it, one control to the left.

### `const MAX_PAGE_BUDGET`

A minute. Not a performance judgement — it is the point past which a
*single* frame hitch stops being distinguishable from a hang, and an
application that appears hung is one the operator kills. Anyone who
genuinely wants an unbounded render has the canvas, which is where a
full-size raster of that page belongs anyway.

### `fn budget_from_millis`

`crate::app::prefs::Prefs::page_preview_budget_ms` holds the operator's
number as a plain `u64` because the preferences file is a text file they
type into. [`ThumbnailCache::budget`] holds it as an `Option<Duration>`
because the code that consults it must not be able to forget the special
case. This function is the join, and it is the whole of the conversion:

| Input | Result | Why |
|---|---|---|
| `0` | `None` | the operator's own instruction — no watchdog is armed |
| `1`…`99` | `Some(100 ms)` | below the floor an ordinary sheet already fails |
| `100`…`60 000` | `Some(that)` | in range, untouched |
| over `60 000` | `Some(60 s)` | above the ceiling a hitch reads as a hang |

# Why `0` is not simply clamped like every other out-of-range value

Because it is not out of range — it is a different *kind* of answer.
Clamping it to [`MIN_PAGE_BUDGET`] would give the operator who typed the
number he was told to type the **most aggressive** limit in the control's
whole span, which is the exact opposite of what he asked for and is the
defect O187 reported in advance: *“today 0 presumably means give up
immediately”*. Every other number is a time limit and is bounded like
one; `0` is an instruction and is obeyed.

# ⚠ What `None` costs

[`ThumbnailCache::render_one`] does not spawn the watchdog thread and
does not arm a [`RenderCancel`] at all, so a page renders to completion
on the UI thread however long that takes. `BENCHMARK.md` records ~10 s at
1× for a full-size CAD raster; a thumbnail is much cheaper, but nothing
in the format bounds it. The control says **never** in words rather than
showing a `0`, so this is not a state anybody arrives in by mistyping.

### `fn millis_from_budget`

`None` becomes `0` — the same number he types, going back out to the same
file. Kept beside [`budget_from_millis`] rather than inlined at the call
site so the two halves of one convention cannot drift apart: a reader who
changes what `0` means has both directions in front of them.

### `const MAX_CACHED_THUMBNAILS`

The arithmetic, because a texture cache with no stated size is a leak
waiting to be discovered: a 140 pt tile at `pixels_per_point` 2 is 280 px
wide, so a portrait A4 is 280 × 396 px = 443 KB of RGBA and a landscape
A1 is 280 × 198 px = 222 KB. Sixty-four of the larger kind is ~28 MB of
GPU memory, which is a reasonable standing cost for a panel that is
usually open and is dwarfed by a single full-size page raster at high
zoom.

It also bounds something less obvious. Every entry here holds an
`egui::TextureHandle`, and a cache that grew without limit over a
900-page document would hold 900 live textures — a number egui will
accept and a driver may not.

### `enum Unavailable`

Recorded rather than retried, and the distinction between the two
variants is what a tile says. A failure is deterministic — same bytes,
same code — so retrying it every frame would peg a core to produce the
same error sixty times a second.

### `enum TileState`

Returned by [`ThumbnailCache::state`] so the body has one `match` over a
closed set rather than three `Option` lookups whose combinations it would
have to reason about. Every variant that is not [`Self::Ready`] carries a
**sentence** in [`crate::text::pages`] — see this module's header on why a
blank rectangle is not an option.

### `struct ThumbnailCache`

Lives on [`crate::panels::PanelsState`], which is application-scoped, so
it outlives a document — and is therefore **forgotten** rather than keyed:
`PanelsState::forget_document` runs `*self = Self::default()` from the one
place a document is opened. Within one document, [`Self::sync`] drops
everything when the edit revision or the display density changes. Between
those two there is no identity comparison anywhere, which is the same
posture `PanelsState`'s own header argues for and the same reason the old
`DocKey` was deleted rather than repaired.

### `fn sync`

Called once per frame before any tile is drawn, so no two tiles can
disagree about which revision they are pictures of.

**The operator's tick, their budget, and the skip note all survive.**
The first two for the same reason: they are instructions, and an
instruction that evaporates on the next edit was not honoured. The
note survives because an edit to sheet 12 does not make sheet 4 cheap,
and re-deriving that would cost a whole abandoned render to reach a
sentence already on screen.

⚠ The pictures themselves do NOT survive their own page's edit — that
is this function's entire job, and the two rules are independent.

### `fn previews_on`

One field read, so "is the control ticked" and "will anything be
drawn" cannot come apart — the panel seeds its checkbox from this and
writes the answer straight back through [`Self::force_on`].

### `fn skipped`

`None` while previews are off, because the sentence it feeds
([`crate::text::pages::previews_skipped_note`]) names a page that has
no picture *for a specific reason*, and with previews off no page has
one for a much simpler reason the operator already knows. Two
explanations for the same blank tile is one too many.

### `fn force_on`

The only writer of [`Self::on`], and every one of its three callers is
the operator's instruction rather than a judgement pdfcer formed:

1. the checkbox, which is where the instruction is given;
2. `PdfcerApp::new`, replaying the last one out of `preferences.txt`;
3. `PanelsState::forget_document`, carrying it across the reset that a
   new document performs.

### `fn set_budget`

Three things happen, and the second and third are the ones that matter:


Idempotent by design — the panel calls this from a `DragValue` that
reports a change on every pixel of a drag, so an unchanged value must
cost nothing.

### `fn next_to_render`

The scheduling rule, isolated so it can be asserted without a window.
`visible` is in reading order — the order the tiles are laid out —
and `current` is the page the canvas is showing.

The current page wins when it is on screen, and the reason is not
politeness. It is the tile carrying the highlight ring, so it is the
one the operator is using to answer *"where am I?"*; a ring around a
tile that says "not drawn yet" answers that question with the page
number alone, which is what they already knew. Everything else fills
in reading order, because a grid that filled in some other order would
look like it was choosing at random.

Returns `None` when everything visible is settled — which is the
steady state, and the reason this is cheap to call every frame.

### `fn render`

The one place this panel renders anything. Runs on the **UI thread**
— see the module header for the `Arc<EditSession>` argument that put
it there — and holds the frame for as long as the page takes, bounded
by [`RENDER_CEILING`].

Four steps, in this order:

1. **Arm the watchdog** at [`Self::budget`] — the operator's number,
   read at the start of each render so a change takes effect on the
   next page rather than on the next document. A one-shot thread that
   cancels the render at the deadline and exits the moment the render
   returns, so no thread outlives the call. `recv_timeout`
   distinguishes *the deadline passed* from *the sender was dropped*,
   which is what makes the disarm free rather than a second message.
2. **Render**, through `session.view()` — never `session.document()`.
   The view composes the edit overlay, so a thumbnail shows the file
   as *edited*. The old shell shipped the other read for a while and
   recorded what it cost: *"the page rail showed the file AS OPENED
   while the canvas beside it showed the file as EDITED. Two pictures
   of the same page, disagreeing, is worse than the original defect:
   it invites the operator to trust the wrong one."*
3. **Record the outcome** — a texture, or a reason there is none.
   A render the watchdog cancelled is [`Unavailable::Abandoned`] and
   is also noted in [`Self::skipped`] for the panel's sentence.

Returns how long the render took, for the caller's trace.

### `fn raster_scale_for`

[`THUMBNAIL_WIDTH_PTS`] divided by the page's own width, put through
`crate::viewer::raster_scale` so the display's `pixels_per_point` is
applied by the same function the canvas uses — one definition of "device
pixels per user-space unit" rather than two that can drift.

A degenerate page — a zero or negative `/CropBox` width, which real files
do contain — would otherwise divide to infinity and produce a raster the
size guard refuses. Falling back to 1.0 draws the page at its natural size
instead, which is wrong-looking and *present*, and the tile is scaled into
its box anyway.

# The rail is pinned to `Normal` quality, and that is a decision

Every other raster in the application follows the operator's
`RenderQuality`. This one does not, and the reason is the same one that
keeps annotations and layers out of the thumbnail's invalidation key: **a
thumbnail answers "which sheet is this?"**, and that answer must not change
with a setting made for a different surface.

Concretely, both directions are wrong here. `Sharper` multiplies the whole
rail — forty tiles, not one page — by 2.25× in pixels, to add detail to an
image already scaled down to a hundred points wide, where none of it is
resolvable. `Faster` saves almost nothing, because a thumbnail is already
the cheapest raster in the program, and buys that nothing at the cost of the
one thing a rail has to do: be recognisable at a glance.

This is not the module second-guessing the operator. It is the same
distinction the settings window itself draws between a preference and a
property of a surface — and it is stated here rather than being an omission
somebody later reads as a missed call site.

### `fn evict_victim`

The one eviction rule, as a pure function over the held page indices.

# Why distance from the viewport rather than least-recently-used

Both are one line; they differ on the gesture that matters. Scrolling a
rail *back up* re-draws tiles that were cached and then dropped — and
under LRU those tiles are exactly the ones evicted first, because they
were touched longest ago. The operator scrolls down a 200-page set and
back, and every tile on the way home is re-rendered at ~0.74 s.

Distance from the middle of what is on screen keeps the neighbourhood the
operator is working in, in both directions, which is the property a rail
actually needs.

Ties break toward the **older** entry (`order` is oldest-first), so an
equidistant pair does not evict at random.

Returns `None` when there is nothing to evict, and never returns
`incoming` — evicting the page about to be inserted would be a cache that
is permanently full and permanently empty.
