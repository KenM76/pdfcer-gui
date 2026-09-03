# Salvage inventory — what carries over from the old GUI

**Source:** `D:\Dev\pdfce\crates\pdfce-gui\src\`, measured 2026-08-12.
**Total:** 49,837 code lines + 12,273 test lines across 21 files.

This is not a from-scratch rewrite. **Roughly 45 % of the code comes
across with little or no change**, and most of what is rebuilt is one
file. The headline problem is `main.rs` at 25,005 code lines — half the
crate — and the rebuild is mostly about giving its contents somewhere
better to live.

---

## Summary

| Class | Code lines | Share | Meaning |
|---|---:|---:|---|
| **A — Lift** | 9,895 | 20 % | Comes across nearly as-is. Engine-facing, tested, correct. |
| **B — Lift and rework** | 12,507 | 25 % | Good bones, needs adapting to the new IA or new structure. |
| **C — Restructure** | 25,005 | 50 % | `main.rs`. Most of its *content* moves; its *shape* does not survive. |
| **D — Rebuild** | 2,430 | 5 % | Ribbon and dock — superseded by `RIBBON_IA.md`. |

Tests follow their subjects. The 12,273 test lines are salvaged with the
code they cover, and are a floor rather than a ceiling — see rule **R1**
in the agent charter.

---

## Class A — Lift nearly as-is

These are the parts of the old GUI that are **good**, and several are
things no competing product has. They come across with their doc
comments and tests intact. "Nearly as-is" still means each gets a
read-through and a `ui-verify` assertion before it is trusted.

| File | Code | Tests | Why it survives | Change needed |
|---|---:|---:|---|---|
| `print_flow.rs` | 1,854 | 168 | Three-tab print dialog with a zoomable live preview of real page content. Self-contained, works, nothing like it needs re-deriving. | Add imposition once `pdfcer-print` shares the sheet composition (a **C**-row in `FEATURES.md`). |
| `icons.rs` | 1,747 | 383 | SVG path data rasterized at physical pixel size rather than pre-baked PNGs. Mostly data. | New icons for the new commands. |
| `measure_tool.rs` | 1,230 | 814 | Dimension **groups** with shared scale and drafting standard — better than the comparison product has. Taubin best-fit circle. Snapping. **`TwoLinePick` (`:361`) is here, built and tested** — see the note below. | Add Area, Angular. **Carry the Two-line gesture across**; it does not need wiring, it needs salvaging. |
| `diag.rs` | 819 | 144 | The `PDFCER_DIAG` key=value channel. Off by default, one atomic load, never load-bearing. This is what made the Delete-key and render analyses possible. | Extend with page complexity per `BENCHMARK.md` §"instrument before optimising". |
| `settings_panel.rs` | 800 | 114 | The spec-ambiguity settings model — each row states what the standard leaves open and how well-founded the default is. A genuine differentiator. | ✅ **SALVAGED 2026-08-17** into `dialogs/settings/` (eight files) + `text/settings/` (four), with five deliberate departures: a seventh group, **Measuring and dimensioning**, because the parallel tolerance sat under *Copying and extracting text* where nobody with the symptom would look; **Colour** expanded rather than Appearance, resolving a contradiction where the source's prose said one and its code did the other; four guessed defaults that read as recommendations now admit it; and two engine facts the old window hid are disclosed. Heading contrast fixed — but by `DEFECTS.md` **D11** rather than D2: `.strong()` was the cause, and there is now a gate. ⬜ The **Render** group is still to come — its seven commands are registered and inert, and the window is now a real destination for them. |
| `object_provider.rs` | 694 | 313 | Front-to-back page object decomposition. Feeds the Objects panel, which is the single strongest thing pdfcer has. | Serve more than the current page, for continuous mode (Phase 4). |
| `object_summary.rs` | 520 | 276 | Per-object descriptions — type, text, font, colour, width, node count, winding. | Row text must not clip; the old panel truncated with no horizontal scroll. |
| `viewer.rs` | 509 | 413 | Zoom ladder with provable reversibility, fit modes re-derived per frame, per-page raster ceiling accounting for `pixels_per_point`. Well tested. | Cursor-anchored zoom (Phase 3.1); page *range* not `page_index` (Phase 4.1). |
| `render_worker.rs` | 466 | 116 | Generation counter + between-operator cancellation. **Measured**: six rapid zoom steps start six generations and complete one. Do not touch the design. | Add a thread pool for thumbnails and adjacent-page prerender (`BENCHMARK.md`). |
| `theme.rs` | 464 | 137 | Palette/preset model, chrome-vs-document colour separation enforced by CI. Sound design. | **Fix `widgets.active.bg_fill`** and add a rendered-pair contrast test — `DEFECTS.md` D2. |
| `redact_apply.rs` | 429 | 280 | Runtime-verified true-removal proof, forced full rewrite. **★ This file is currently the ONLY place the proof exists** — see below. | Canvas drag-to-mark (currently panel-driven only). |
| `raster.rs` | 363 | 0 | Premultiplied alpha handled correctly; stale texture scaled `LINEAR` during settle. This is *why* zoom feels smooth. | None. |

**Subtotal: 9,895 code lines, 3,158 test lines.**

### ★ Correction, 2026-08-14 — the Two-line gesture is *built*, not pending

The `measure_tool.rs` row used to say *"wire the Two-line gesture whose core
is already done"*, and four other documents said the same thing in stronger
words: **"the canvas gesture has no caller"** (`FEATURES.md`, `HANDOFF.md`,
`RIBBON_IA.md` §5.6 twice, and `shell/manifest/mod.rs`'s `PLANNED` entry for
`measure.two_line`).

**It was false, and it was false when written.** In the old shell:

| what | where |
|---|---|
| the `pick_line_in_page` call | `main.rs:23564` |
| the pick itself | `main.rs:23592` — `st.two_lines.offer_line(h, parallel_epsilon)` |
| hover highlight before any click | `main.rs:23574-23587` |
| picked-pair overlay, verdict disclosure, Escape to clear | `:23597-23604`, `:23175-23187`, `:23857` |
| the state type | `measure_tool.rs:361` `TwoLinePick`, tests at `:1717-2040` |

pdfcer's own `docs/FEATURES.md:104` marks that row **`gui [x]`**; the `[ ]` in
it is the *Acrobat* column. The gesture landed in their commit `c4ec3f5`,
2026-08-12 — the same day this file's survey was taken, which is the most
likely reason it was missed. The probable textual origin is a misread of
pdfcer's `ROADMAP.md:2778`, which explains why `pick_line_in_page` exists, one
paragraph above the commit heading that added its caller.

**What it changes.** The missing caller is *ours*, not theirs. This shell has
no measure tool at all: `canvas/tool.rs` has two `CanvasTool` variants, no
`measure.*` command has a dispatch arm, and `crates/pdfcer-gui` contains zero
occurrences of `linepick`, `PickedLine` or `author_from_two_lines`. So the
work is this row — carry `measure_tool.rs` across — plus the ~900 lines of
Class C canvas hosting at `main.rs:23100-23900`. The *"cheapest real feature
in the backlog"* claim that rode on the false premise is withdrawn.

**The lesson is the same one the `deletion_refusal` filing taught**, pointed
the other way: that time a claim about *their* code was wrong because it was
checked against the wrong function; this time a claim about *our own*
backlog was wrong because it was never checked at all, and it then travelled
into four more documents by being quoted. **A status word in a table is a
claim, and it decays.**

---

### ★ `redact_apply.rs` is load-bearing in a way the table understates

Flagged by the core team in the request channel, 2026-08-13:

> **`Pass 72.0` — the redaction true-removal proof is not in
> `pdfcer-core`.** It lives in `crates/pdfcer-gui/src/redact_apply.rs:269`,
> i.e. in the shell being replaced. **A shell calling
> `redact::apply_redactions` directly and writing the bytes ships an
> unverified redaction and will not know.** … `pdfcer`'s
> `redact-apply` does exactly that at HEAD and exits `SUCCESS` on a file
> it never verified. **Do not build a redaction UI against core's
> current surface**; wait for the verdict type to land in core.

Two consequences for this project:

1. **Salvaging this file is not optional and not merely convenient** —
   deleting it, or reimplementing redaction against core's current API,
   would ship an unverified redaction. It comes across whole, with its
   proof intact, and the proof is re-verified by a test before the
   redaction UI is reachable.
2. **When core lands the verdict type, this becomes a deletion**, not a
   parallel implementation. Two proofs that can disagree is worse than
   one in the wrong crate. Watch the channel for the `note_` that says
   Pass 72.0 closed, and file the migration as its own task rather than
   keeping both.

---

## Class B — Lift and rework

Good material whose hosting or structure changes.

| File | Code | Tests | Disposition |
|---|---:|---:|---|
| `ui_text.rs` | 7,912 | 3,913 | **The string catalog — 1,193 `pub fn` entries.** A large asset and the reason pdfcer's copy is as good as it is. Most strings survive verbatim; ribbon/tab/panel labels change with the IA. **Fix `shortcuts_reference()` — it omits six live bindings (`DEFECTS.md` D5) — and derive it from the keyboard map so it cannot drift again.** Split into modules by area; at 7,912 lines it breaks R2. |
| `canvas.rs` | 1,893 | 1,244 | The `CanvasTool` enum, dispatch, and the escape ladder are sound concepts. The *selection layer* is where Phase 1 lands — handles, context menus, `/Rect` move-and-resize — so this becomes several modules under `canvas/`. |
| `panels_structure.rs` | 1,807 | 0 | Bookmarks, Layers, Signatures, Fonts panel bodies. The bodies keep; the hosting changes, and Fonts moves to **File ▸ Document** per the IA. Note this file ships **zero tests** and three of its panels shipped with no operator-reachable control at all. |
| `canvas_overlay.rs` | 749 | 0 | Overlay drawing — theme-invariant by design because the page beneath is white regardless of chrome. Mostly keeps; grows for selection handles and marquee. |
| `vector_edit_tool.rs` | 146 | 91 | Node/handle editing. Keeps. Carries a measured hot spot at 6,681 anchors in one path object — re-check as selection gets richer. |

**Subtotal: 12,507 code lines, 5,248 test lines.**

---

## Class C — Restructure: `main.rs`

**25,005 code lines + 3,579 test lines.** Half the crate.

The *architecture inside it is good* and must be preserved:

- **Actions, not mutations.** No code path runs from a widget to a
  `Document`; everything is an `Action` applied after the frame draws.
  This is why the undo log is coherent, and it is the single best
  structural decision in the old GUI. **Keep it exactly.**
- **One `EditSession` command log**, 44 `CommandKind` variants, depth
  bounded at 256, undo tooltips naming the specific operation.
- **The five-rung Escape ladder** with documented precedence.
- **Fixed-height status and find panels**, because content-driven
  heights re-fit the page on every click — a measured defect, already
  solved.

What does not survive is the *file*. Its contents redistribute roughly:

| Content | Approx. lines | Goes to |
|---|---:|---|
| Form filling, field authoring, FDF/XFDF/CSV | ~1,600 | `panels/forms/` |
| Text editing — runs, caret, formatting, reflow host | ~3,500 | `tools/text/` — **partly landed 2026-08-15**, see below |
| Vector object editing, node/handle | ~1,200 | `tools/vector/` |
| Measure/dimension hosting | ~900 | `tools/measure/` |
| Page ops, thumbnail rail, selection action bar | ~1,400 | `panels/pages/` |
| Batch pane — merge, split, insert, font folders | ~700 | `panels/batch/` |
| Redaction hosting | ~600 | `panels/redact/` |
| Object tree panel | ~800 | `panels/objects/` |
| Frame composition, panel order, dock hosting | ~900 | `app/frame.rs` |
| Keyboard map, action dispatch, status narration | ~1,800 | `app/{keyboard,actions,status}.rs` |
| App state, open/save/close, password prompt, parked docs | ~2,200 | `app/state.rs` |
| Canvas hosting, hit-test dispatch, pan/zoom input | ~2,000 | `canvas/` |
| Dialogs — properties, print, export, reset, settings host | ~1,500 | `dialogs/` |
| Find | ~600 | `app/find.rs` |
| Remaining glue, helpers, types | ~5,300 | distributed |

**Honest framing:** perhaps 60 % of this moves with edits rather than
being rewritten. The genuinely *new* work is the selection model,
context menus, the properties panel, and the ribbon — and those are
additions, not replacements.

---

## Class D — Rebuild

| File | Code | Tests | Why |
|---|---:|---:|---|
| `ribbon.rs` | 666 | 47 | Tab/group model and the ownership test. The *mechanism* is good — one source of truth, a test asserting every group has exactly one owning tab. The *content* is superseded by `RIBBON_IA.md`: seven tabs plus a contextual Format tab, P1a amended so the QAT and status bar may mirror. Rebuild around the same invariants. |
| `ribbon_ui.rs` | 1,187 | 0 | Band rendering. Group-caption enforcement via one closure is worth keeping. Everything else follows the new IA. |
| `dock.rs` | 577 | 241 | Two independent `egui_tiles` trees, deliberately unbridgeable. Reconsider: the new panel set is larger (properties panel, comments, forms), layout must **persist** (Phase 3.6), and the two-pane-max constraint was a workaround for `egui_tiles` 0.16 hiding overflow tabs. Its constraints-as-tests approach is worth carrying forward regardless of the outcome. |

**Subtotal: 2,430 code lines, 288 test lines.**

---

## What is NOT salvaged, and is not lost either

Things the old GUI does not have. Listed so they are not mistaken for
salvage:

- Context menus — `grep context_menu` across the old crate returns
  **zero hits**.
- Move or resize anything carrying a `/Rect` — the one `FEATURES.md`
  row that gates markup, form widgets, redaction marks, links and
  dimensions all at once.
- A properties panel of any kind.
- Recent files, session restore, autosave, in-place save.
- Hand tool, rulers, grid, guides, go-to-page box.
- Six of ten markup kinds, and revision clouds.
- Page image export, attachments panel, canvas text selection — all
  **C**-rows: present in core or CLI, no GUI surface.

---

## Salvage procedure

For each file, in this order:

1. **Read it in full**, including the doc comments. They explain the
   defects that shaped it, and that reasoning is the most valuable thing
   being transferred.
2. **Copy it across with its tests and its documentation**, into its new
   module home.
3. **Apply the known fixes** for that file from `DEFECTS.md`, and add
   the regression test the defect implies.
4. **Split if it exceeds 1,500 lines** (R2) — find the seam, do not
   raise the limit.
5. **Assert it in `ui-verify`** before calling it done (R1). A green
   unit test is the floor.
6. **Record it here** — move the row to a "landed" state with the date
   and what changed. This file tracks reality, not intent.

Never salvage a file by pasting a snippet out of it. The old GUI's
value is disproportionately in its doc comments; a snippet leaves those
behind and the next engineer re-derives a decision that was already
made and already paid for.

---

## Landed

Step 6 of the procedure above. This section tracks **reality**: what has
actually been moved, where it now lives, and what changed on the way.

### Stage S0 — 2026-08-13

Built against `D:\Dev\pdfcer` as of 2026-08-13 (`pdfcer-render` 0.5.3).
All of the below builds, `cargo test -p pdfcer-gui` is green (56 tests),
`cargo fmt -p pdfcer-gui --check` and
`cargo clippy -p pdfcer-gui --all-targets -- -D warnings` are clean, and
the binary renders the 5.6 MB CAD benchmark drawing.

| Source (old crate) | New home | State | What changed |
|---|---|---|---|
| `viewer.rs` (509 + 413 test) | `src/viewer/mod.rs` (945) | **complete** | `use eframe::egui` → `use egui`. `zoom_percent`, `page_to_screen`, `pdf_space_to_canvas` carry `#[allow(dead_code, reason = …)]` naming the stage of their first consumer. No arithmetic changed; every test carried across and passing. |
| `render_worker.rs` (466 + 116 test) | `src/render/worker.rs` (595) | **complete, three keys deferred** | Generation counter, `RenderCancel` token, `IN_FRAME_BUDGET` and the single-slot design untouched. `RenderKey` compares **two** keys (page, raster scale) rather than five: `annotations`, `font_env_generation` and `layers_generation` land **with the surfaces that vary them** (S2/S3) — the module docs tabulate all three, the defect each prevents, and the rule that the key ships in the same commit as its control. `cmyk_intent`/`fonts`/`view_magnification` left to `RenderOptions`' defaults for the same reason. New: a `render-spawn gen=N page=P scale=S` trace line, so "six zoom steps start six generations and complete one" is checkable from outside the process. |
| `raster.rs` (363) | `src/render/raster.rs` (214) | **page half complete** | `pixmap_to_color_image`, `PageTexture`, `texture_from_pixels` carried with the premultiplied-alpha and LINEAR-filtering sections verbatim. `ThumbnailCache` deliberately left behind — it belongs with the Pages panel (S3). `texture_from_pixmap` left behind — it exists for the print preview (S5). The stale *"Why rendering is synchronous"* section was replaced by an accurate one that **keeps the original prediction on the record** and notes it was vindicated. New: two unit tests pinning the premultiplied read, which the original had none of. |
| `canvas.rs` — `pan_offset`, `zoom_anchor_offset` (Class B) | `src/canvas/geometry.rs` (334) | **these two complete** | Lifted verbatim with all eight tests. The rest of `canvas.rs` (tool dispatch, selection, escape ladder) stays behind for S4/S5. |
| `diag.rs` (819 + 144 test) | `src/diag.rs` (119) | **trace channel only** | `enabled()`/`trace()` and the full header rationale. The `PDFCER_DIAG_SCRIPT` harness grammar (`Step`, `ScriptTool`, …) lands with `tools/ui-verify` at S1 — a script language with no interpreter is not salvage. New: a test that a disabled trace never builds its message. |
| `main.rs` — eframe bootstrap, `ViewportBuilder`, `PDFCER_DIAG_VIEWPORT`, `configure_context`, `open_path`, `settle_and_rasterize`, `is_unsupported_structure`, the canvas `ScrollArea` (Class C) | `src/main.rs` (149), `src/app/{mod,state,actions,keyboard}.rs`, `src/canvas/mod.rs` | **the S0 slice** | The three-way open-failure distinction, both staleness policies, the `ZOOM_SETTLE` debounce, the discrete-command bypass, the manual page centring (and the ~105 px selection-offset defect its comment records) all carried with their reasoning. `main.rs` is 149 lines against the old 25,005. |
| — (new) | `src/text/mod.rs` (205) | **new** | The ui-string catalog, a directory from the first commit so the old `ui_text.rs`'s 7,912-line R2 breach cannot recur as a migration. |

**`DEFECTS.md` fixes applied at salvage time (procedure step 3):**

- **D1 — the keyboard guard.** `src/app/keyboard.rs` uses
  `ctx.text_edit_focused()`, never `ctx.egui_wants_keyboard_input()`. The
  module header records the whole causal chain and the egui line numbers.
  The regression test
  `a_focused_non_text_widget_does_not_suppress_unmodified_keys` drives a
  real `Context` through two frames, **asserts that
  `egui_wants_keyboard_input()` is genuinely `true`** (so it cannot pass
  vacuously the way the old single test did), and then asserts the
  unmodified bindings still fire.
- **D1 "Not defects" — zoom anchor.** Ctrl+wheel now anchors on the
  **cursor**, via the salvaged `zoom_anchor_offset`. The *discrete* zoom
  commands (Ctrl+Plus/Minus/0) are still unanchored and carry a TODO in
  `src/canvas/mod.rs` naming `GUI_ROADMAP` Phase 3.1, where the zoom
  buttons and zoom-to-selection/region land and the anchor rule can be
  decided once for all four.

**Still owed on the S0 salvage (procedure step 5):** every one of the
above needs a `ui-verify` assertion, which cannot be written until the
harness exists at S1. A green unit test is the floor, not the ceiling.

### Phase 7 — the measure salvage, 2026-08-14

Class A `measure_tool.rs` and the Pass 12.M1 snap primitives, carried across
in one pass. `cargo test -p pdfcer-gui --lib` green, all eight gates green.

| Source (old crate) | New home | State | What changed |
|---|---|---|---|
| `measure_tool.rs` (1,230 + 814 test) | `src/canvas/measure/pick.rs` (1,290), `scale.rs` (607), `state.rs` (377) | **complete** | Every `///` and `//!` paragraph carried verbatim; **all 36 tests carried, none dropped** — verified by diffing the complete function-name set (public, private and test) old against new: identical. Both load-bearing CLI-equivalence tests pass, so a canvas-authored `DimensionKind` is still byte-for-byte the one `pdfcer dimension-add` builds. **No `pdfcer-core` API had moved** — every one of the ~25 imported items checked against the engine at this workspace's path dependency, unchanged signatures, no adaptation invented. Three adaptations, all documented in the files: `CanvasTool::MeasureLinear` and `GestureInterrupt` became prose (neither exists here), and cross-module doc links were repointed. |
| `canvas.rs:1584-1892` + tests at `:3046-3136` (12.M1 snap) | `src/canvas/snap.rs` (587) | **complete, not yet queried** | The zoom-invariant catch radius, the master/Alt gate, the Tab cycle, the two-click confirm, the indicator glyph. `#[allow(dead_code, reason = …)]` kept where the item is still unused, with **the reason rewritten** to name this shell's consumer — an inherited reason pointing at a pass in another repo is a stale claim. `screen_tolerance_to_page` deliberately **not** salvaged: `canvas/mapping.rs` already has it, and that module's header states there is no second place in `canvas/` that divides by zoom. |

**Three departures from the source, each deliberate:**

1. **One `CanvasTool::Measure(MeasureKind)` variant**, where the old shell had
   three variants plus `is_measure()` plus three `tool_builds_measure_*`
   predicates. Five helpers replaced by a value. This is the one place the
   salvage deliberately improves on its source rather than carrying it.
2. **A third file.** The planned two-way split leaves `pick.rs` about twenty
   lines over R2. Rather than shave prose to fit a threshold — the incentive
   `check-file-size.sh` says in its own header it refuses to build in — the cut
   was made at a seam **the original had already drawn for itself**, its own
   `// ---` banner separating the three pick machines from the container that
   owns them.
3. **No Accept/Reject box.** The old hosting held a completed pick in
   `MeasureState::pending` and waited for an explicit Accept in a property bar.
   The third click commits instead. `pending` survives on the type, with its
   tests, for a future property surface that is not a floating box.

**One collision the salvage surfaced, and how it was resolved.** The old shell
had **two** axes — a `CanvasTool` *and*, inside the linear tool, a
`LinearPickMode` — so `set_linear_pick_mode`'s discard guarded one and the tool
switch guarded the other. This shell has **one**: `MeasureKind`, with two-line
as a kind rather than a mode. Had arming become the axis while the discard
stayed attached to the old one, a half-finished point pick would have survived
into two-line mode — and the original's own docs warn that this surfaces not as
an error but as *"something strange"* on the operator's **next** click.
`MeasureState::set_kind` is that rule restated over the axis this shell
actually has, delegating to `set_linear_pick_mode` for the pair it already
owns.

**Still owed:** a `ui-verify` assertion (procedure step 5) for the placed
dimension, and the snap candidate query, which is the one thing standing
between the salvaged snap primitives and a pick that snaps.

### The redaction salvage, 2026-08-15

Class A `redact_apply.rs` — **the row this file's own ★ note calls "load-bearing
in a way the table understates"** — carried across whole, with its proof, its
refusal taxonomy and every paragraph of its reasoning. `cargo test --workspace`
green (1,715 passing), all ten gates green, and the apply path driven end to end
against the real binary.

**Both halves of the `Pass 72.0` warning were re-checked against `D:\Dev\pdfcer`
rather than quoted**, because acting on a stale claim is this project's
documented failure mode. Both still hold on 2026-08-15:
`pdfcer_core::redact::apply_redactions` still returns
`Result<(Vec<u8>, RedactionReport), RedactError>` — a **report**, not a verdict;
`RedactionVerdict` and `verify_redaction` appear nowhere in `pdfcer-core` (the
only `verify_absence` in either tree is still the old shell's, at
`redact_apply.rs:361`); and
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` is empty, so nothing is
owed and Pass 72.0 has not closed.

| Source (old crate) | New home | State | What changed |
|---|---|---|---|
| `redact_apply.rs` (429 + 280 test) | `src/redact/mod.rs` (825), `src/redact/proof.rs` (447), `src/redact/sealed.rs` (487) | **complete** | Every `///` and `//!` paragraph carried. The classification table, the four-character floor, the wide stream sweep, the local `contains` ("an absence proof that shared its search routine with the code it is auditing would be a weaker proof"), the two-full-rewrite shape and all five `RedactApplyRefusal` variants are the source's. **All seven tests carried**, plus twelve new ones for the halves the source had none for. No `pdfcer-core` API had moved — every imported item checked against the engine at this workspace's pin, unchanged signatures, no adaptation invented. |
| `main.rs::redact_panel` (~600, Class C) | `src/panels/redact.rs` (515) | **complete, panel-driven** | Mark whole page, find-and-mark (literal or pattern), the review list with go-to-page and remove. `Panel::Redact`, `ALL` 9 → 10, command `edit.redact`. |
| `main.rs::redaction_apply_confirmation` (Class C) | `src/dialogs/redact.rs` (826) | **complete** | The measured report, both acknowledgements, the consequence-labelled confirm control, and the save-as write. |
| `ui_text.rs:6876-7410` (Class B) | `src/text/redact.rs` (748) | **complete** | The three wording rules carried verbatim into the module header, and each is now a **test** rather than a comment: nothing on the marking surface claims a removal, "verified" appears in exactly one place, no post-apply sentence offers Undo. |

**Three deliberate departures from the source, each documented at its site:**

1. **The proof decodes the document once**, where the source decoded every
   stream twice for one question (`verify_absence` and
   `leaked_in_decoded_streams` each parsed and inflated independently). The two
   halves stay separate functions with separate tests; only the evidence is
   shared.
2. **The write is not atomic**, where the source routed through a shared
   `write_atomic`. This shell has no such helper — both its existing writers
   call `std::fs::write` — and a truncated write of an already-redacted buffer
   can only lose trailing bytes, never introduce un-redacted content. Argued at
   `PreparedRedaction::write_to`; a shared atomic writer for all three call
   sites is recorded as an improvement rather than smuggled in here.
3. **`⚠` is the only non-ASCII character used**, and `✔`/`✕` are gone. The
   source's `✔`-prefixed success line and `✕` remove button were written before
   `DEFECTS.md` D12's corrected table, which lists U+2715 as having **no
   supporting face** in the bundled stack. `⚠` is measured drawable and the
   catalog-wide glyph gate sweeps this file with the rest.

**The proof is unskippable, structurally, by four mechanisms** — the brief's
second requirement, and the one copying the file across does not satisfy. The
source's own docs end *"nothing in this module can reach the filesystem"*, which
means the proof was enforced by the **caller remembering**; `pdfcer`'s
`redact-apply` is the counter-example in the same repository. In order of how
hard each is to defeat: `PreparedRedaction::bytes` is **private with no
accessor** and a hand-written `Debug` that reports a length; the **only** way
out is `write_to`, which **re-runs the decoded-stream proof between the buffer
and the syscall**; the residual acknowledgement is a **required argument**
(`ResidualAcknowledgement::{Given,Withheld}`), so a caller that forgets the
checkbox gets a named refusal rather than a partially-redacted file; and
`redact::sealed` parses **every `.rs` file in the crate** with `syn` and asserts
`apply_redactions` is *called* in exactly one place — failing closed on both a
short sweep and a zero count, with seven self-tests including a planted second
call site inside a closure. A second test asserts nothing in `redact/` reaches
for `to_incremental_bytes`, which in this shell (unlike the source's) is an
idiomatic verb one autocompletion away.

**Canvas drag-to-mark is NOT in this landing**, which is the row's "change
needed" and is stated rather than left to be found. The brief's own instruction
was followed: *"if the canvas gesture is more than a modest addition, ship the
panel-driven version and say so."* It is more than modest — a `CanvasTool`
variant, an `app::modes::capability` entry, a rung on the Escape ladder, an
overlay preview and an `Action` carrying page-space quads, i.e. `HANDOFF.md`
§8's tool-substrate warning applied to a substrate that would arm the one
irreversible verb in the program. `edit.redact`'s **shipped tooltip** enumerates
three marking routes — a whole page, every occurrence of some text, everything
matching a pattern — and all three are built. `panels/redact.rs`'s header
records what the gesture would take.

**Obligation 6 discharged, and the register went down again.** Both commands
left `shell::commands::reach::SCAFFOLDED`: **35 → 33**, `★ P3` unchanged at 8
(neither carried the mark — they were controls with a stated blocker, not
controls that should not have been drawn). Their entries were **deleted rather
than reworded**, which is what `no_scaffolded_entry_is_stale`'s middle assertion
exists to force. `edit.redact` is routed by the `Panel::from_command_id` guard
arm; `edit.redact_apply` has a literal arm. **The registry count did not
change** — both were already registered — the group count is still 31, `PLANNED`
is untouched, and the RON was regenerated and came back byte-identical.

**Driven, not merely tested.** `tools/ui-verify`'s new
`redaction_removes_and_proves_it` is the third two-process check in the suite
and the only one whose verdict is a **byte scan of a file on disk**. Measured
output, release binary, 2026-08-15:

```
phase A: pdfcer's own extraction reads 24 character(s) from page 1 of the fixture
phase C: redact-panel marks=1 pages=1 epoch=1
phase D: redact-prepared marks=1 pages=1 glyphs=24 streams=1 checked=1 short=0
         residuals=0 verified=true bytes=943
phase E: the confirm control is not offered until the acknowledgement is given
phase F: redact-written … bytes=943 marks=1 glyphs=24 checked=1 residuals=0 verified=true
phase G: the document that was opened is byte-for-byte unchanged
phase H: the output is 943 bytes — CONFIDENTIALWITNESSALPHA is absent from them
         and UNTOUCHEDWITNESSBETA is present
phase I: the redacted file re-opens (drawn=2), and a SECOND PROCESS extracts
         NOTHING from the redacted page — text-copy-declined reason=nothing-to-copy
```

The falsifying phase is built in and runs on every invocation: the **same byte
scan** is run three times, and two of the three exist only to stop the third
passing vacuously — the secret must be **present** in the fixture before
anything happens, and the untouched page's string must be **present** in the
output before its absence means anything. A build that marked and reported
success without removing anything fails phase H; a *check* that could not see
the secret at all fails phase 1 and is reported as a harness defect rather than
as a pass. The check was also **observed to fail** during development, at phase
D, when the apply control was declared below its own pane.

**★ One defect found only by driving the binary**, which is `HANDOFF.md` §2's
founding rule paying for itself an eleventh time. The panel's first cut put the
apply control *second* rather than *last*; `ui-verify` reported it declared at
`y = 801.7` inside a panel body ending at `y = 770.0` — **off the bottom of its
own pane**, on a 1100×800 window, with one mark made. Every unit test passed,
because a unit test cannot see where a control landed. The census and the apply
control are now the **first** things in the panel, with everything that can grow
below them.

**Still owed:**

- **The search-and-mark route is not driven.** It needs a query typed into a
  field, and synthetic keystrokes do not reach the target window from the
  session that writes them on this machine (`HANDOFF.md` §8). Its rule is
  unit-tested; the field itself is verified by nothing. The check's header says
  so rather than leaving it to be discovered.
- **When core lands the verdict type, this becomes a deletion**, not a parallel
  implementation — the instruction at the top of this file, unchanged. Watch the
  channel for the `note_` that says Pass 72.0 closed. `redact::sealed`'s
  monopoly assertion is what will make that migration visible rather than
  gradual.

### The text-editing salvage, 2026-08-15 — **partial, and the split is the point**

Class C's `tools/text/` row is **~3,500 lines**, and roughly 400 of them
landed. That is not a shortfall against the estimate; it is the estimate
being read correctly for the first time. `DEFECTS.md` D4 had already
split "text editing is weird" into three problems with different causes,
and only one of them is a salvage job at all:

- **The disposition fix is not salvage — it is a line the old shell never
  wrote.** `FollowerDisposition::Pin` lives in the *engine*, documented
  for exactly this case, and `main.rs` passed `EditOptions::default()`
  from its only call site. There was nothing to lift. What landed is a
  new pure chooser, `canvas/textedit/disposition.rs`, plus the rotation
  guard **ported verbatim** from `reflow_apply.rs`
  (`check_uniform_axis_aligned`, `MTX_EPS = 1e-6`) rather than
  re-derived — the one genuinely salvaged piece, and it was salvaged
  from the *reflow* path, not the edit path, because the edit path never
  had one.
- **The caret and the keystroke loop lifted cleanly.** Raw
  `egui::Event::Text`, no `TextEdit` widget in the typing path, a caret
  painted in PDF space: the old shell's approach was right and is kept.
- **The ghost-text renderer was deliberately not lifted.** It is
  ~600 lines of the estimate and it is the thing D4 names as the second
  contributor to "weird". Salvaging it would have carried the defect
  across intact.
- **The reflow host — the largest share — is untouched**, and stays
  Class C. So does multi-run editing, which needs an engine request
  first.

**A defect introduced during the salvage, and caught before it shipped**,
because it is the kind that survives a green test run:
`ExtractOptions::capture_provenance` **defaults off**, and this shell's
shared `page_text()` cache is built with `default()`. Fed from that
cache, the new chooser would have seen a `None` pin and identity
matrices — so the rotation guard could never fire on any document, while
its own unit tests passed against hand-built matrices. The lesson is now
`HANDOFF.md` §10's twelfth bite: **a pure function's tests prove the
function, not its inputs.**
