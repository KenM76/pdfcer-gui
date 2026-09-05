# pdfcer GUI — roadmap

**Written:** 2026-08-12
**Companions:** `RIBBON_IA.md` (where commands live), `DEFECTS.md` (what
is broken, with `file:line`), `mockups/ribbon.html` (what it looks
like), `evidence/` (screenshots of both pdfcer and the comparison
product).

---

## The thesis

pdfcer's engine is ahead of its shell. The Objects panel, the Fonts
panel and the spec-ambiguity Settings dialog are things no competing
product has, and the parsing underneath them is demonstrably better. On a
shared test file the comparison product reported `Pages -`, `Page Size -`
and every metadata field blank, while pdfcer read it correctly.

What is missing is not capability. It is the layer of ordinary
conventions a user brings with them: click a thing and press Delete;
right-click for options; find page operations under something called
Pages; zoom without losing your place; type and watch the text move.
Each of those is individually small. Together they are the difference
between a tool that feels like an inspector's instrument and one that
feels unfinished.

So the roadmap is ordered by **how much user-visible behaviour each unit
of work buys**, not by architectural interest. Phase 0 is four days of
work that removes the two worst impressions the product currently makes.

---

## A standing rule this investigation earned

**Every GUI change needs a verification that drives the running binary,
not only a test that passes.**

Two defects in `DEFECTS.md` were invisible to a green test suite and
obvious within thirty seconds of using the app:

- **D1** — the Delete key. The only test of `collect_keyboard_actions`
  builds a bare `egui::Context` with no widgets, so the focused-widget
  condition that breaks the real app cannot occur in the harness. The
  regression commit says so itself: *"analysis-confirmed, NOT
  empirically verified."*
- **D2** — invisible headings. Two theme tests sit adjacent to the bug
  and neither measures a rendered foreground/background pair.

The project already has the ingredients: `PDFCER_DIAG=1` emits a
`key=value` stderr trace, and the 2026-08-08 screenshot audit found two
ribbon groups rendering with no caption at all — caught by a screenshot,
not a test.

**Proposal.** A `tools/ui-verify/` harness that launches the release
binary, opens a fixture, drives a scripted sequence via the OS, captures
the window, and asserts on both the diag trace and the pixels. Start
with the smallest useful set:

1. Click an object on the canvas → assert selection non-empty → press
   Delete → assert the object count dropped. (This is D1's regression
   test, and it must run through the real event loop.)
2. Screenshot each ribbon tab and assert every group has a caption whose
   rendered contrast against its background exceeds a threshold.
3. Screenshot the Settings dialog and assert the same for every section
   heading.

This costs perhaps two days and it is the highest-leverage item on the
whole list, because it changes what "done" means for every phase below.

---

## Phase 0 — Stop the bleeding

**Target: one week. Nothing here is architectural.**

| # | Work | Ref | Size |
|---|---|---|---|
| 0.1 | `let typing = ctx.text_edit_focused();` | D1 | 1 line |
| 0.2 | Exclude text tools from the `canvas_delete_target` hole | D1 | ~6 lines |
| 0.3 | Check `editing_enabled` in `Action::DeleteSelection` | D6 | ~3 lines |
| 0.4 | Keyboard test through a focused context | D1 | small |
| 0.5 | Set `widgets.active.bg_fill = accent`; add a rendered-pair contrast test | D2 | small |
| 0.6 | README: remove Bates and PDF/A from "Working today"; qualify imposition as CLI | D3 | 3 lines |
| 0.7 | Derive `shortcuts_reference()` from `collect_keyboard_actions`, or test they agree | D5 | small |
| 0.8 | Fix `ROADMAP.md` Pass 33.0's "no on-canvas caret" claim; footnote `FEATURES.md:73`; correct `FEATURES.md:119` | D7 | 3 edits |
| 0.9 | Delete the stale worktree under `.claude/worktrees/` | D8 | 1 command |
| 0.10 | Status-bar diagnostics collapse by default | — | small |

**0.1–0.4 must ship together.** Fixing the Delete key without 0.3 turns
a dormant review-mode hole into a live one.

**Why 0.10 is here.** The first line a user reads today is *"This page:
119 bundled substitute glyph(s), 0 operator-supplied glyph(s)…"*. The
disclosure triangle already exists; it should start closed. The
information is genuinely valuable and stays one click away.

---

## Phase 1 — Make selection mean something

**Target: three to four weeks. The largest usability return in the plan.**

Right now, selecting an object produces a highlighted tree row and no
way to act on it. Everything a user would try next — right-click, drag a
handle, press Delete, change the colour — either does nothing or does
not exist. Phase 0 fixes the keyboard path; this phase builds the rest.

| # | Work | Notes |
|---|---|---|
| 1.1 | **Context menus** | Canvas, thumbnail rail, Objects tree, Comments list, annotations. `grep context_menu` currently returns zero hits across the crate. Each carries Cut/Copy/Paste/Delete plus its type's commands. |
| 1.2 | **Move and resize anything carrying a `/Rect`** | `FEATURES.md:208`. One row unblocks markup, form widgets, redaction marks, links and ce dimensions at once. Highest structural leverage in the backlog. |
| 1.3 | **Selection handles and cursor feedback** | Eight handles plus move, per the convention every drawing tool shares. Cursor changes over a handle, over a movable object, over the canvas. |
| 1.4 | **Object clipboard** | Cut / Copy / Paste / Paste in place for canvas objects. |
| 1.5 | **Properties panel** | Decided: **both** surfaces ship, panel first. It holds the full property set including editable X/Y/W/H, which is how `/Rect` resize becomes reachable by typing a number rather than dragging. The tab's contents are a subset, so building the tab first would mean writing the editors twice. |
| 1.6 | **`Format` contextual tab** | The mid-gesture subset — colour, width, style, align, delete. Appears on selection, disappears on deselect. |
| 1.0 | **Two deferred deadlines fall due here** — both were deferred *with a stated rule*, and S4 is the commit the rule names. **(a)** `PanelsState`'s decomposition and font caches move onto `OpenDoc`, deleting `DocKey` and its `Arc`-address ABA hazard: an identity key exists only because a cache outlived what it described. **(b)** `RenderKey` gains `annotations` and `layers_generation`, because *"the key ships in the same commit as its control"* and the Layers visibility checkbox is now due. | in progress 2026-08-13 |
| 1.7 | **Remove the `Editing on` master toggle** | Decided: *"make it work the same way other programs do."* Delete `editing_enabled` (`main.rs:3235`, `3624`), the ribbon toggle (`ribbon_ui.rs:721-736`) and the four gate sites (`7095`, `8169`, `8194`, `16920`). **This supersedes D6** — with no review mode there is nothing to enforce. If a read-only mode is ever wanted it should be a *document* state with a visible badge, not a hidden global switch. |

### ✅ Move and node editing are unblocked — indices are safe across `move_*`

**Answered 2026-08-13.** `request_stable_object_identity.md` asked whether a
paint-order index survives an edit. It does, for the family that matters,
and the answer is **proved by a test that decomposes, edits, and
decomposes again** rather than read off the planner:

| family | mechanism | renumbers? |
|---|---|---|
| `move_object` · `move_objects` · `move_subpath` · `move_node` · `move_nodes` · `move_handle` | rewrites operator **operands** in place | **NO** |
| `delete_object` · `delete_objects` · `delete_subpath` · `delete_node` · `delete_text_run` | excises byte **spans** | **YES** |

A move changes numbers *inside* existing operators, so no operator is added
or removed and every other object keeps its exact fingerprint at its exact
index. **So build move and node editing against indices — the selection
survives them unchanged.** No token, no invalidation.

For the delete case, `pdfcer_core::vector::remap_index_after_delete(i, &deleted)`
returns `None` for "it is gone" and never a different object's index. It
handles unsorted and duplicated input, which matters because a shell
unioning two overlapping selections would otherwise shift a survivor twice.

**Still blocked, but on a different thing:** the eight resize grips have no
verb. `EditSession` has the whole `move_*` family and **no scale or resize
verb at all** — so resize is blocked on a *capability*, not on identity, and
that is a separate request when the grips are built.

Also noted for whoever builds them: **reorder and paste do not exist yet**,
and when they do they must be checked against that same test file and the
table above extended. A new verb that renumbers without saying so re-opens
this hazard exactly.

### Selection survives navigation — an invariant, not a feature

**Operator requirement, 2026-08-13:** *"if I select a node or something
for a tool, I should be able to pan and zoom out without losing my first
selection."*

> **Navigation is not an edit. Panning, zooming, changing fit mode,
> rotating the view, switching page-display mode and changing ribbon tab
> must never alter the selection.**

This has to be stated as an invariant because the natural implementation
loses it in three separate ways, each of which looks reasonable in
isolation:

1. **Selection stored in screen coordinates.** Zoom changes the mapping,
   so the stored point stops naming the thing it named. Selection must be
   held as *object identity* — page + object index + sub-path + node —
   never as a position. The `canvas-pointer` trace already reports
   `screen=/page=/pdf=` together precisely because those three are
   different spaces and conflating them is the classic defect here.

   **Core confirms this is a live trap, from the other side.** Per
   `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`: *"Every
   hit-test and snap `tolerance` is a PAGE-space radius, and nothing
   checks it. Pass raw screen pixels and it compiles, runs, and merely
   drifts with zoom"* (`hit.rs:118-120`). The screen→page conversion is
   the shell's job. So the same confusion breaks selection persistence
   **and** silently changes hit tolerance with zoom — a selection that
   survives a zoom but was made with a tolerance 8× too large is not a
   fix. Convert once, at the boundary, and keep page space inward.

   Related, and equally silent: **PDF user space is y-UP; image and
   screen space are y-DOWN.** The page looks perfect until someone
   selects a line and gets a different one.
2. **Selection cleared by a click that was really a drag.** A pan gesture
   begins with a press on the canvas. If press-on-empty clears the
   selection, every pan that starts on blank paper destroys it. The clear
   must be driven by a *completed click* with no drag, not by a press.
3. **Selection invalidated by re-decomposition.** The object provider
   rebuilds on page change and on edit. A rebuild triggered by zoom — or
   by a continuous-scroll page change that is not a page *change* in the
   operator's sense — must not drop the selection. Identity must be
   re-resolved against the new decomposition rather than discarded.

A multi-select and an entered sub-path/node level must survive the same
operations. Escape is the only thing that ascends the selection ladder,
and it is already specified that way.

**Acceptance:** select a node, zoom out three rungs, pan across the
sheet, switch to Continuous, come back, switch ribbon tab — the node is
still selected and still the entered level. Add this to `ui-verify` as
its own check; it is exactly the kind of property that unit tests pass
and a running application fails.

**Acceptance for the phase:** place a rectangle, click away, click it
again, drag it, resize it, type an exact width, change its colour,
right-click it, delete it. Every step of that sequence fails today.

---

## Phase 2 — Rebuild the ribbon

**Target: two to three weeks. Specified in full in `RIBBON_IA.md`.**

| # | Work |
|---|---|
| 2.1 | Adopt amendment **P1a** — the QAT and status bar are shortcut surfaces and may mirror a tab command. Unblocks a File tab with Open/Save and a View tab with zoom. |
| 2.2 | Seven tabs: File · View · Pages · Edit · Markup · Measure · Tools |
| 2.3 | **Pages tab** — surface the page operations that already work but live only in the thumbnail rail and the Batch pane |
| 2.4 | **View tab gets view controls** — page display modes, view rotation, read mode, full screen, targeted zooms, and the new **Render** group |
| 2.5 | Relabel `Aa` → **Edit text**, `I⁺ Aa` → **Add text**, `Obj` → **Edit objects**; label the two unlabelled rotate icons |
| 2.6 | Move: copy-text → Edit, Redact → Edit, rotate page → Pages, Fonts panel → File ▸ Document, reset layout → View ▸ Window |
| 2.7 | Rename Review → **Markup** |
| 2.8 | Update the group-ownership test and the `RibbonTab::groups()` source of truth |

**What does not change:** P2 (ribbon picks the activity, sidebar holds
its controls), P3 (no placeholders), P4 (mandatory group captions), P5
(nothing floats over the canvas). Those rules are good and this phase
keeps every one of them.

---

## Phase 3 — Viewer conventions

**Target: two to three weeks. Small items, disproportionate effect.**

| # | Work | Why |
|---|---|---|
| 3.1 | **Cursor-anchored zoom** | Zoom buttons currently pin the page's top-left, so zooming in loses your place. The comparison product treats this as a measured metric with a published budget — under 3 px of anchor drift. That is a reasonable bar. |
| 3.2 | **Hand tool + space-to-pan** | There is no hand tool at all; panning is middle-drag only. |
| 3.3 | **Editable page-number box** | Reaching page 37 of 42 currently means the thumbnail rail or 36 keystrokes. |
| 3.4 | **Zoom to selection; marquee zoom to region** | Neither exists. Both are core drafting-review gestures. |
| 3.5 | **Recent files** | `grep -i recent` finds nothing anywhere in the crate. |
| 3.6 | **Persist dock layout** | Requires turning on `eframe`'s `persistence` and `egui_tiles`' `serde` features. The in-app notice saying the layout will be lost can then be deleted. |
| 3.7 | **Thumbnail rail reflows to a grid and narrows** | It reserves ~390 px of a 1936 px window to show one thumbnail. See `evidence/pdfcer_max.png`. |

---

## Phase 4 — Page display modes

**Target: four to six weeks. The only genuinely architectural item in
the first half of this plan.**

**Single page stays the default.** Paging one sheet at a time is the
right model for drafting review and the existing navigation is good.
Continuous becomes a *mode you choose* on the View tab, for the case
where the document is a 40-page specification rather than a sheet set.

| # | Work | |
|---|---|---|
| 4.1 | `ViewState` holds a page *range* rather than one `page_index` (`viewer.rs:90-98`) | ✅ **and the answer was that a range is not a field.** Which pages are on screen falls out of where they are laid out and where the viewport is, so `viewer::strip` computes it and `page_index` stays one index — now meaning *the page the operator is looking at*, derived from the scroll under a continuous mode |
| 4.2 | Object provider serves more than the current page (`object_provider.rs:392-399` currently returns nothing for any other page) | ✅ **without changing the provider.** Pressing on a page makes it current before the hit test runs, so the provider is still asked for one page and it is still the right one. One decomposition per `(page, epoch)`, unchanged |
| 4.3 | Scroll-driven current-page tracking; find-navigation stops assuming a page change is never a scroll | ✅ Greatest visible area wins, lowest index breaks a tie. Find's two-frame reveal was **not** modified: the canvas converts the strip offset into the one-page-at-the-origin world that solve is written for, and converts the answer back |
| 4.4 | Four modes on View ▸ Page display: Single · Continuous · Facing · Facing continuous | ✅ A radio, with the active position rendering pressed through the `selected:` convention |
| 4.5 | Mode persists **per document**, not globally — opening a drawing set must not inherit a report's setting | ✅ `page-display.txt`, a third store beside `layout.ron` and `recent.txt`. Read mode's continuous default applies only when a document has no remembered choice |

**Done.** The estimate above ("four to six weeks", "the only genuinely
architectural item in the first half of this plan") was right about the
shape and wrong about the size: the architectural weight turned out to be
in *what is rasterized and when*, not in the view state. The four modes
are a strip layout; the affordable part is that only visible pages
rasterize, one at a time, nearest first.

---

## Phase 5 — Text editing

**Target: staged. See `DEFECTS.md` D4 for the full diagnosis.**

Split by cost, because the three complaints have very different prices.

### 5a — Correctness bugs (days)

| # | Work |
|---|---|
| 5a.1 | Detect alignment on the edit path; pass `FollowerDisposition::Pin` for right/centre/justified tails. The variant exists (`edit.rs:301-303`); the GUI never uses it (`main.rs:12438`). |
| 5a.2 | Add the rotation guard to the edit path. `reflow_apply.rs:757-760` already has one; `edit.rs:1503` shifts rotated `Tm` along the wrong axis. This bites rotated CAD title-block text. |
| 5a.3 | Fixtures that reproduce both: a right-aligned paragraph, a rotated title-block string. |

### 5b — Live layout while typing (one to two weeks)

Today the draft is ghost text in an egui proportional font over a
translucent mask, and real layout runs once on commit
(`main.rs:17868-17899`, `18208-18210`). You type in the wrong typeface at
the wrong widths and it snaps to reality on Accept. Re-measure and
re-render the draft with the real metrics per keystroke. The metrics
path is already correct — this is about calling it more often.

### 5c — Reflow reachability (two to three weeks)

| # | Work |
|---|---|
| 5c.1 | **Decide Pass 33.0.** Options (b) median line width or (d) refuse-to-auto-detect. Only the disclosure shipped, and the roadmap admits *"an operator who does not read the disclosure still gets a re-wrap to a width they never chose."* |
| 5c.2 | **Plan reflow against staged session content.** Today it refuses after any edit and demands save-and-reopen, *after* showing a correct-looking preview (`edit.rs:4279-4285`, `main.rs:18660-18669`). This is the single most confusing sequence in the product. |
| 5c.3 | Treat `DerivedWordSpace` as a break opportunity. Reflow currently breaks only at real U+0020 glyphs (`reflow.rs:42-54`), so CAD output that positions words with `Td`/`TJ` offsets presents one unbreakable word and nothing wraps. |
| 5c.4 | Relax the uniform-font and uniform-size refusals (`reflow_apply.rs:669`, `:768`). |

**R75 is not in question.** Automatic re-wrap on edit would invent line
breaks the file never stated. Keep it operator-invoked; make it
*reachable*.

### 5d — Multi-run editing (large, schedule separately)

`PendingEdit` pins to one show-text operator (`main.rs:2386-2400`), so a
paragraph split across several `Tj` runs must be edited run by run, and
a cross-run selection silently disables typing altogether. Fixing this
means a multi-run edit request in core that groups runs into a line or
block and re-emits them as a set. It is the correct end state and it is
the most expensive item in this document.

### Fixtures, before any of the above

Every current text fixture is synthetic and single-run: one `Tj` per
line, real space glyphs, uniform font and size
(`tools/gen-reflow-fixtures.py:114-124`). Every condition that fails in
the field is absent by construction. Add fixtures with a multi-run
paragraph, mixed sizes in a block, rotated text, and words separated by
positioning rather than space glyphs — **first**, so the work above has
something to fail against.

---

## Phase 6 — Markup completeness

Six of ten markup kinds are deferred (`canvas.rs:255-262`): Ink,
Polygon, PolyLine, Underline, StrikeOut, Squiggly. **Cloud** is not on
that list and matters most for this audience — revision clouds are table
stakes on a drawing markup.

Also: markup cannot carry `/Contents` note text, and the Style group
sets width, fill and opacity for nothing (only colour exists). Both are
small once Phase 1 has given selection meaning.

---

## Phase 7 — Measure completeness

| Item | Cost |
|---|---|
| **Two-line dimensioning** | Shell only — core and CLI shipped and measured, `pick_line` has no caller. Cheapest real feature in the backlog. |
| **Area** and **Angular** | The conspicuous absences for takeoff work. |
| Count tool and a takeoff schedule | Larger; the comparison product has one worth studying. |

The **dimension-group** model — named groups carrying a shared scale and
drafting standard — is better than what the comparison product offers
and nothing here should dilute it.

---

## Standing backlog — shell-only work

These exist in `pdfcer-core` and/or `pdfcer` and need a GUI surface,
not an engine. Any of them can fill a gap in any sprint; each is small
and independently shippable.

| Capability | Status |
|---|---|
| Attachments panel | core ✓ CLI ✓ GUI ✗ — no surface at all |
| Page image export (PNG/JPEG/TIFF, DPI picker) | core ✓ GUI ✗ |
| Canvas text selection and copy | core ✓ GUI ✗ (`FEATURES.md:70`) |
| Imposition in the print dialog | CLI ✓ GUI ✗ — needs sheet composition lifted into `pdfcer-print` |
| Insert blank page | core ✓ GUI ✗ |
| Push-button field creation | CLI ✓ GUI ✗ |
| Move a form widget | core ✓ CLI ✓ GUI ✗ — folds into Phase 1.2 |
| Script-driven-field census | core ✓ CLI ✓ GUI ✗ |
| Unencrypted-wrapper warning | core ✓ CLI ✓ GUI ✗ |

---

## Rendering — a choice to expose, not a weakness to fix

**Corrected 2026-08-12, and now measured — see `BENCHMARK.md`.** An
earlier draft of this document called the whole-page raster a weakness
and proposed replacing it with a tile cache. That was inferred from
architecture, not measured — exactly the failure mode the standing rule
above exists to prevent — so it is corrected here rather than quietly
edited out.

The operator's report was that pdfcer's zoom and pan felt *faster and
more pleasant* than the tiled competitor's. Measured on a 5.6 MB dense
vector site plan, that is correct, and the reason is sharper than
"smoothness versus throughput":

> **Six rapid zoom clicks (1.0 → 4.0) started six render generations and
> completed exactly one — the last.** Total cost 1 899 ms, versus roughly
> 11 s if every step had rendered. Generations 3–7 were superseded by
> the generation counter and abandoned mid-flight by the cancellation
> token, while the user saw the previous texture linearly scaled.

So the original claim — *"no tile cache, so zooming re-rasterizes the
entire page"* — is misleading. It re-rasterizes the entire page **once,
at the destination**. The generation counter plus the 150 ms
`ZOOM_SETTLE` debounce (`main.rs:367`, `raster.rs:639-647`) already
solve the problem a tile cache would have been introduced to solve, by a
cheaper route: don't render what the user is scrolling past.

pdfcer also uses **2.5× less memory** on the same file — 170–231 MB in
one process against 569 MB across five for the tiled competitor, which
spends the difference on crash isolation.

So the plan is not to replace whole-page rendering. It is to **expose
the trade**, which is R169 applied to rendering: where the right answer
is genuinely undetermined, state it and let the operator choose.

### New: View ▸ Render group

| Control | Today | Proposed |
|---|---|---|
| **Strategy** | whole page, hard-coded | Whole page *(default)* · Tiled progressive |
| **Raster scale** | derived from zoom × `pixels_per_point` | exposed as a quality multiplier |
| **Settle delay** | `ZOOM_SETTLE = 150 ms` constant | exposed; lower on fast machines, higher on huge sheets |
| **Thin lines** | ✅ **2026-09-05** | CAD hairline rendering, one pixel per stroke at any zoom. **View ▸ Display ▸ Line weights**, on by default. AutoCAD's `LWDISPLAY` convention (thick → thin), NOT Acrobat's *enhance thin lines* (thin → thick) — they are opposites. Canvas only; every export keeps the real widths. |
| **Antialias** | — | text and vector, independently |

> ### ⛔ Corrected 2026-08-13 — the answer came back, and it is not tiling
>
> `render_page_region` shipped (`Pass 74.0`), so the zoom ceiling is gone:
> `MAX_PIXMAP_EDGE` now guards the **returned pixmap**, and at scale 32 a
> region renders in 1.07 s where the whole page would be 3.8 GiB and
> impossible.
>
> But the measurement that came with it kills the tile ring. **A 1 × 1
> *point* region — two pixels — costs 691 ms on the benchmark drawing**,
> against 699 ms for a 120,701-pixel region: ~99 % of the cost is
> area-independent. A 3 × 3 ring is a **9× regression**.
>
> **S6 is therefore one region per viewport, not a tile grid.** Tiling
> survives only as a way to *bound memory* on an enormous viewport, never
> to save time. And N region calls must not be driven across cores —
> without a shared display list that multiplies the interpretation floor
> instead of dividing it.
>
> **The optimisation that does pay is a reusable parsed handle**: by the
> pdfcer team's own numbers it takes second and subsequent renders of a
> page from ~700 ms to roughly fill cost — tens of milliseconds. It is
> **not built**; they asked whether S6 depends on it and the answer is
> filed in `open/request_reusable_parsed_handle.md`. **S6 should not start
> until that is scheduled**, because switching to per-viewport regions
> without it trades the smooth pan the operator praised for a 0.7 s
> gesture.
>
> Also settled: **`MAX_ZOOM` comes from performance, not numerics.**
> Sub-pixel accuracy holds to ~5,000× — three orders of magnitude past any
> plausible viewing zoom — so picking the limit from `f32` would repeat the
> exact error `MAX_PIXMAP_EDGE`'s original justification made.

The tiled path was scoped as a *feature to offer* rather than a rewrite
to survive, to be built only once someone had a sheet where whole-page
genuinely hurt.

**That sheet has been found, and it is the benchmark drawing.**
*Amended 2026-08-13.* The operator asked to zoom further than other
software allows, and `BENCHMARK.md` § "The zoom ceiling" works out what
actually stops it: a whole-page raster's edge scales with zoom against a
16,384 px allocation guard, so the A1 benchmark drawing caps at **6.9×
on a 1× display and 3.4× on HiDPI** — not the nominal 8×, and worse the
larger the sheet. Raising `MAX_ZOOM` changes nothing; the strategy is
the binding limit.

A tile's pixel size is fixed by the tile, not by the zoom, so tiling
removes the ceiling. **It is therefore a scheduled requirement, not an
option** — while whole-page remains the better default for *motion*,
which is why the choice stays exposed rather than replaced.

`PDFCER_DIAG` already emits `render-async-done gen=N ms=M outcome=…`,
which is most of a performance harness. Adding page complexity —
operator count, path count, resource count — to that line would make it
a complete one, and would let the decision about a tiled path be made
from data rather than from architecture diagrams. That is the next step,
not the tile cache.

### Would more processes help? No — see `BENCHMARK.md`

Measured with `tools/render-profile`: at scale 0.25, where fill is
negligible, the render still costs **0.74 s**. That floor is **148,517
paint operations walked through a sequential state machine** at ~5 µs
each, and it is 89 % of the cost at fit-page zoom.

- **Processes buy crash isolation, not speed.** pdfcer is one binary;
  threads share memory for free, while processes would ship a
  multi-megabyte pixmap across a pipe per render.
- **The dominant cost cannot be parallelised at all.** A content stream
  is a state machine — `q`/`Q`, `cm`, colour, clip all accumulate — so
  operator *N* cannot be interpreted without the state from 1…*N*−1.
- **Parallel band fill would buy 1.11× at 1× zoom and 1.64× at 2×** on
  ten cores. Real, but not where the money is.

**The bigger win is not parallelism.** `render_page(doc, page, scale)`
(`pdfcer-render/src/lib.rs:165`) retains nothing between calls, so every
zoom change re-walks all 148,517 operators although only the transform
changed. A **display list built once and replayed at any scale** turns a
zoom re-render into fill alone — ~90 ms at 1× instead of ~830 ms. Bigger
than ten cores, applies at every zoom, and composes with parallel fill.

**Where threads pay today, with no interpreter changes:** pages are
independent, so thumbnails (currently 2 per frame, serialised on one
worker — `main.rs:392`) and adjacent-page prerender parallelise cleanly.

Priority: display list → thread pool for thumbnails/prerender → parallel
band fill → processes only if crash isolation is wanted for its own sake.

### Still worth doing regardless

- **Find extracts the whole document's text synchronously** in the
  dispatch loop (`main.rs:9779-9812`). On a large document this is a
  visible stall and it has nothing to do with rasterization.
- **The node-grab hot spot** is already measured at 6,681 anchors in a
  single path object (`vector_edit_tool.rs:692-733`) and was already
  rescoped once. It will resurface as selection gets richer in Phase 1.
- **Adjacent-page prerender** is cheap, independent of strategy, and
  makes paging through a sheet set feel instant. Worth doing under
  either model.

The off-thread worker with its generation counter and
between-operator cancellation token stays exactly as it is. It is good
work, it is measured — 28.9 ms to cancel versus 10,367 ms to let a
render finish — and nothing here changes it.

---

## Explicitly not on this roadmap

Named so the omissions are decisions rather than oversights.

| Not doing | Why |
|---|---|
| **A Home tab** | Would mirror commands across tabs and re-create the Pass 47.1 defect. P1a gives the QAT and status bar the shortcut role instead. |
| **Automatic reflow on edit** | R75. Reflow invents line breaks the file never stated. Make it reachable, not silent. |
| ~~**Ribbon customisation**~~ | **Now in scope, 2026-08-13.** `ribbon.rs:42-52` deferred it because *"a customisable ribbon that also forgets itself would be worse than none"* — an objection about persistence, and persistence is now the first thing built. `SHELL_FRAMEWORK.md` makes the ribbon a serializable manifest, which delivers customisation and cross-project reuse with one mechanism. |
| **OCR** | ✅ **SHIPPED 2026-08-14** as `file.ocr` — **File ▸ Recognise**, not Tools ▸ Recognise, because Tools is not in Read's tab list and the operator asked for OCR in Read (`RIBBON_IA.md` §5.7 now carries that ruling). The model ships with credit: 12,240,008 B of `ocrs` weights in the package, the CC-BY-SA-4.0 section in `THIRD_PARTY_LICENSES.md` naming **Robert Knight** and linking the deed, and a `check-shipped-assets` gate proven to bite on each half separately. Find offers OCR only when the document has no extractable text — never on a merely-empty search. Saves to a new file (`<stem>-recognised.pdf`), never the source. **Two caveats that are not going away**: recognition quality on real scans is unproven because no scanned PDF exists in the tree, and `ocrs` collapses on sparse clean pages — the shape of a drawing sheet — see `DEFECTS.md` D15. |
| **JavaScript execution** | Standing refusal. |
| **Document comparison** | The one absence an AEC reviewer names first, and a large build. Deliberately deferred past Phase 5 — see Q4. |
| **i18n / CJK chrome coverage** | Decision 002 chose English-only with no locale detection. A real ceiling on adoption, but not a defect and not this year's work. |

---

## Questions

### Answered 2026-08-12

| Question | Answer | Lands in |
|---|---|---|
| `Editing on` master toggle | **Remove it** — work the way other programs do. Selection and Delete always live; tools arm and disarm. | Phase 1.7 |
| Format tab or properties panel | **Both**, panel first — the tab's contents are a subset. | Phase 1.5–1.6 |
| Continuous scroll | **An option, not a replacement.** Single page stays the default; the four modes sit together on View. | Phase 4 |
| Whole-page vs. tiled rendering | **Neither wins outright.** Whole-page stays the default because it measured better; tiled becomes an opt-in beside it, with quality and settle exposed. | Phase 2.4 + Rendering |

### Still open

**Q1 — Save semantics.** Is autosave plus true in-place `Save` wanted,
or is `Save a copy` the permanent model? A File tab whose Save group
holds only "Save a copy…" is honest but unusual, and the dependency is
already documented (`FEATURES.md:62`). Affects Phase 2.

**Q2 — document comparison.** Worth building, or out of scope? It is the
feature an AEC reviewer asks for first and it is a large build. If yes,
it needs its own phase and probably its own view mode.

**Q3 — how much of Phase 5d?** Multi-run text editing is the most
expensive item here. Is "edit one run at a time, clearly disclosed" an
acceptable resting state for another year, or is this the thing that has
to be right?

*(The former Q4 — locating the benchmark drawing — is answered.
`D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf`, measured in
`BENCHMARK.md`.)*
