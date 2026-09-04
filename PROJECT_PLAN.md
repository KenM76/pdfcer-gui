# pdfcer GUI rebuild — project plan

**Status:** ★ **BUILT AND SHIPPING.** This file was written on 2026-08-13 as a
proposal and opened with *"Nothing has been built"*; it said so for thirteen
days after that stopped being true, which is the drift this project has now
corrected in three separate documents. Last reconciled against the tree on
**2026-08-26**.

The shell is a working application the operator uses on real drawings, published
as a portable build. **159 capabilities are ticked in `FEATURES.md`**, the stage
table below is current, and the only stages still open are the last two — the
parity audit and the fold-in, both of which are deliberate scheduling decisions
rather than unfinished work.

★★ What has NOT changed is the governing rule: `D:\Dev\pdfcer` remains
**read-only** until fold-in day. Every engine change since has gone through the
request channel and come back as a released revision.
**Written:** 2026-08-12

The charter for the `pdfcer-gui-engineer` agent
(`.claude/agents/pdfcer-gui-engineer.md`). Read with `SALVAGE.md` (what
carries over), `GUI_ROADMAP.md` (phase order), `RIBBON_IA.md` (the shell
spec), `DEFECTS.md` (what was wrong), `BENCHMARK.md` (measured
performance).

---

## 1. What this project is

A new `pdfcer-gui` crate, built in `D:\Dev\pdfcer-gui\`, that replaces
`D:\Dev\pdfce\crates\pdfce-gui\` when it is at least as capable and
substantially more usable.

**It is not a rewrite.** Per `SALVAGE.md`, about 45 % of the old crate's
49,837 code lines come across with little or no change, and most of the
rest is one file — `main.rs`, 25,005 lines — whose *contents* mostly
move rather than get rewritten. The genuinely new work is the shell:
ribbon IA, selection model, context menus, properties panel.

### Why a separate project rather than refactoring in place

Three reasons, in order of weight:

1. **The working program keeps working.** pdfcer ships. The operator uses
   `pdfcer-gui.exe` on real drawings. Refactoring in place means every
   intermediate state is the shipping state, and there is no fallback on
   a bad day. Here the old GUI is untouched until the swap.
2. **The module split cannot be done incrementally without churn.**
   Breaking a 25,005-line file into ~40 modules touches essentially every
   line. Done in place, that is a series of enormous commits that make
   `git blame` useless and review impossible.
3. **The IA change is a break, not an evolution.** Seven tabs replacing
   six, commands changing tabs, a contextual tab appearing, the master
   editing toggle removed. There is no sensible half-migrated ribbon.

### The cost, stated honestly

**Divergence.** While this runs, `pdfcer-core` keeps moving and the old
GUI may gain fixes. Mitigated by §2's path dependency (you always build
against current core) and §6's rule that the old GUI is frozen for
*features* during the project — bug fixes to it are fine and get
replayed into the new shell via `SALVAGE.md`.

---

## 2. Topology

```
D:\Dev\pdfcer-gui\                    ← this project, its own cargo workspace
├── .claude\agents\
│   └── pdfcer-gui-engineer.md
├── Cargo.toml                      ← workspace, three members
├── crates\
│   ├── egui-shell\                 ← REUSABLE shell framework. Knows nothing
│   │                                  about PDF. Extracted to its own repo at
│   │                                  or before fold-in. See SHELL_FRAMEWORK.md
│   └── pdfcer-gui\                  ← THE ARTEFACT. Folds in verbatim.
│       ├── Cargo.toml
│       └── src\
├── tools\
│   ├── ui-verify\                  ← built first; folds in as pdfcer/tools/
│   └── gates\                      ← CI gates, incl. the fixed ui-strings gate
├── fixtures\                       ← GUI-specific fixtures only
└── *.md                            ← the planning docs

D:\Dev\pdfcer\                       ← READ-ONLY until fold-in
└── crates\{pdfcer-core, pdfcer-render, pdfcer, pdfcer-gui}
```

**Dependency direction:** `pdfcer-gui/crates/pdfcer-gui` depends on
`pdfcer-core` and `pdfcer-render` **by relative path**:

```toml
pdfcer-core   = { path = "../../../pdfcer/crates/pdfcer-core" }
pdfcer-render = { path = "../../../pdfcer/crates/pdfcer-render" }
```

This is deliberate and has one important consequence: **you always build
against the live engine.** If `pdfcer-core` changes under you, you find
out at compile time rather than at fold-in. That is the right trade — a
vendored copy would hide divergence until the worst possible moment.

The crate is named `pdfcer-gui` and its directory is `crates/pdfcer-gui`,
identical to the target. **Fold-in is therefore a directory swap** for
that crate, and pdfcer's root `Cargo.toml` needs no edit for it.

**`egui-shell` folds in differently, on purpose.** It is extracted to
its own repository (`D:\Dev\egui-shell`) and consumed by pdfcer as a path
or git dependency — *not* copied into `crates/`. The directive was that
this work be reusable by other projects, and a crate living inside
pdfcer's tree is not reusable in any practical sense. That adds one line
to pdfcer's root `Cargo.toml` at fold-in and is the only edit it needs.
The purity gate (`tools/gates/check-shell-purity.sh`) is what keeps the
extraction cheap: if it stays green throughout, extraction is a `git mv`.

### Version pinning

`rust-toolchain.toml` is copied from pdfcer and kept identical. Any
dependency the new crate adds must already be in pdfcer's lockfile, or it
is a decision that goes to the operator — pdfcer's dependency posture is
deliberate (all-permissive licences, no GPL PDF engines, a pinned
`skrifa` matched to epaint's).

---

## 3. Module architecture

The answer to a 25,005-line file. **No file over 1,500 lines**, gated in
CI from the first commit.

```
crates/pdfcer-gui/src/
├── main.rs                 eframe bootstrap ONLY. Target < 150 lines.
├── app/
│   ├── mod.rs              PdfcerApp — the one owner of state
│   ├── state.rs            open/save/close, parked docs, password prompt
│   ├── frame.rs            panel composition order (load-bearing — see below)
│   ├── actions.rs          the Action enum + dispatch
│   ├── keyboard.rs         the keyboard map
│   ├── status.rs           status-bar narration
│   └── find.rs
├── shell/
│   ├── ribbon/
│   │   ├── model.rs        tabs, groups, ownership — ONE source of truth
│   │   ├── render.rs       band rendering, mandatory group captions
│   │   └── tabs/           file.rs view.rs pages.rs edit.rs markup.rs
│   │                       measure.rs tools.rs format.rs
│   ├── qat.rs
│   ├── docbar.rs           document switcher
│   └── dock.rs             panel host + persistence
├── panels/
│   ├── pages/ objects/ properties/ bookmarks/ layers/
│   ├── signatures/ fonts/ comments/ forms/ redact/ batch/
├── canvas/
│   ├── mod.rs viewport.rs input.rs
│   ├── selection.rs        ← Phase 1 lives here
│   ├── handles.rs          ← and here
│   ├── context_menu.rs     ← and here
│   └── overlay.rs
├── tools/
│   ├── text/ vector/ measure/ markup/ form_field/
├── render/
│   ├── worker.rs raster.rs texture_cache.rs
│   └── display_list.rs     ← BENCHMARK.md's biggest win
├── theme/ icons/ text/     (text/ = the split ui_text catalog)
└── dialogs/
```

### Invariants carried forward from the old GUI

These are the good parts and they are not up for renegotiation:

- **Actions, not mutations.** No path from a widget to a `Document`.
  Everything is an `Action` applied after the frame draws. This is why
  the undo log is coherent.
- **Panel composition order is load-bearing** for both geometry and Tab
  focus. Document it where it is written, as the old GUI does.
- **Fixed-height status and find panels** — content-driven heights
  re-fit the page on every click. Already-measured defect, already
  solved, do not re-open.
- **One `EditSession` command log**, bounded depth, undo tooltips naming
  the specific operation.
- **The ribbon picks the activity; the sidebar holds its controls.**
- **No placeholders.** Unavailable renders nothing; greying is for
  *temporarily* unavailable and always explained on hover.
- **Nothing floats over the canvas.**

---

## 4. Build order

Each stage produces a **runnable program**. There is never a period
where the crate is a pile of modules that does not launch.

| Stage | Contents | Gate to pass |
|---|---|---|
| ✅ **S0 — Skeleton** | Workspace, three crates, CI gates, bootstrap, `diag`. Opens a PDF, renders page 1 via salvaged `render_worker` + `raster` + `viewer`. | **DONE 2026-08-13.** Renders the benchmark drawing in 1,193 ms; 6/6 gates green; 113 tests pass. Theme and icons deferred to S2 (both are ribbon/panel-facing). |
| ✅ **S1 — ui-verify** | The harness, before any UI is built. Drives the release binary, scripts input, captures the window, asserts on the trace **and** pixels. | **DONE 2026-08-13.** `delete_key_after_canvas_click` and `settings_headings_legible` both **FAIL against the old binary** — the acceptance criterion. `ribbon_group_captions_legible` reports SKIPPED pending §4.3 requirement 2, never a false pass. |
| ✅ **S2 — Shell** | Ribbon per `RIBBON_IA.md` — all seven tabs, groups, captions, ownership test. QAT, status bar with editable page box, dock with persistence. Commands wired where they already exist. | Every command in the IA's migration map reachable. — **DONE.** Ribbon, tabs, QAT, theme and icons all shipped; the band has since gained the full width ladder (re-wrap → collapse → scroll) documented in `RIBBON_SCALING.md`. |
| ✅ **S3 — Panels** | Pages (grid thumbnails), Objects, Properties, Bookmarks, Layers, Signatures, Fonts, Comments, Forms. Salvaged bodies, new hosting. **Plus the flexible-dock foundation** — see §4.2. | `FEATURES.md` gui column: every panel-reachable capability works; layout survives a restart. |
| | **Done 2026-08-13.** Dock + layout persistence (7,274 lines, 130 tests): columns per side, stacks, tabs, reserved-space tab overflow (**the two-pane cap is retired** — nine panels in one stack, tested), per-item fail-soft loading, named workspaces, scoped reset. Built **without `egui_tiles`** — decision recorded in `MODES_AND_PANELS.md` Part 2. Six panels salvaged and wired. **571 tests, 6/6 gates.** Verified in the running app on the benchmark drawing: left dock Bookmarks∣Layers, right dock Objects reporting **129,758 objects — 129,515 paths, 242 text, 1 form**. | |
| ✅ **S3b — Modes** | The Read / Review / Edit selector (`MODES_AND_PANELS.md` Part 1), built on S3's named-workspace mechanism. | All three modes render; switching preserves undo and unsaved work; a signed document opens in Read with a reason. — **DONE.** Read / Review / Edit ship as one control, tab sets nested by capability, `Ctrl+1/2/3`. |
| ✅ **S4 — Selection** | `GUI_ROADMAP` Phase 1. Context menus, handles, `/Rect` move-and-resize, object clipboard, Format contextual tab. Editing master toggle removed. | Place a rectangle → click away → click it → drag → resize → type a width → recolour → right-click → delete. Every step. — **DONE.** Handles, move/resize/rotate, node editing, object clipboard, context menus and the Format contextual tab are all in `FEATURES.md` and driven by `ui-verify`. |
| ✅ **S5 — Tools** | Text, vector, measure, markup, forms, redact — salvaged and rehosted. Includes Phase 5a text correctness fixes. | Parity with the old GUI on all tool capabilities. — **DONE.** Text, vector, measure, markup, forms and redaction all rehosted, with disclosure surfaces the old shell did not have. |
| ✅ **S6 — Viewer** | Phase 3: cursor-anchored zoom, hand tool, zoom-to-selection/region, recent files, rulers/grid. | ui-verify: anchor drift under 3 px. — **DONE, and past the original scope.** Cursor-anchored zoom, hand tool, zoom to selection and region, rulers, grid and guides — plus deep zoom to 10¹² % on an `f64` anchor, which was not in this plan. |
| ◑ **S7 — Parity audit** | Line-by-line `FEATURES.md` gui column audit against the old GUI. Fill every gap or get an operator decision to drop it. | §7.1 checklist complete. — **STARTED 2026-09-04**, and the first measurement is below. |
| | **The gap has a number for the first time: ~90 rows read `[x] core` and `[ ] gui`** in the engine's own `docs/FEATURES.md`. That table is a machine-readable statement of what the engine has and this shell does not, and until 2026-09-04 nothing on this side read it. ★★★ **This stage was always going to be a document; it is being built as an INSTRUMENT instead** — `ENGINE_BACKLOG.md` triages every row as *wanted*, *declined with the argument*, or *blocked on something named*, and `tools/gates/check-engine-backlog.sh` fails the build when a row appears that is in none of those states. A document answers "what is missing today"; a gate answers "what appeared since". ★★ The provocation was O120: the operator asked the ENGINE session for PNG/JPEG/SVG export, the engine shipped it the same day and sent a note, and this shell built nothing and filed no row — for a day, invisibly. `check-verb-coverage.sh` catches a new *verb* within hours; nothing caught a new *capability announced in prose*. | |
| **S8 — Fold-in** | §7. | Ships. |

### 4.1 S0 prerequisite — `check-ui-strings.sh` fails open on a module tree

**Verified 2026-08-12.** The gate that enforces "every operator-visible
string lives in the catalog" scans with a **flat, non-recursive glob**:

```bash
# D:\Dev\pdfcer\tools\check-ui-strings.sh:76
for file in "$SRC_DIR"/*.rs; do
```

`src/*.rs` does not match `src/app/state.rs`. The moment the first
subdirectory exists, the gate stops seeing almost the entire crate —
**and reports success**, because finding nothing looks exactly like
finding no violations. It would print `ui-strings: clean` while checking
a handful of files.

This is the single strongest convention gate in the project, and the
module split in §3 would silently switch it off.

I checked every sibling gate for the same shape. The rest are fine:

| Gate | Method | Recursive? |
|---|---|---|
| `check-ui-strings.sh` | `for file in "$SRC_DIR"/*.rs` | ❌ **flat** |
| `check-theme-colors.sh` | `find "$GUI_SRC" -name '*.rs'` | ✅ |
| `check-bypass-paths.sh` | `find ./crates -name '*.rs'` | ✅ |
| `check-disclosure-channel.sh` | `grep -rn … crates/pdfcer-gui/src/` | ✅ |
| `check-settings-consumed.py`, `check-shipped-assets.py`, `check-one-commit-per-command.py` | `rglob` | ✅ |
| `check-commits-filed.py`, `check-passes-filed.py` | `walk` | ✅ |

**The fix is one line** — `find "$SRC_DIR" -name '*.rs'`, with the
`ui_text.rs` exclusion generalised to the catalog *directory* once
`ui_text.rs` is split (§9, Q4).

**But it is a change to `D:\Dev\pdfcer\tools\`, which this project may not
write to.** So it is a hand-off to `pdfcer-engineer`, filed as its own
Pass in the pdfcer repo, and it must land **before S0**. It is also worth
doing regardless of this project: the gate is currently one refactor away
from silently protecting nothing.

### 4.2 Panel flexibility and modes — where each piece lands

Full analysis in `MODES_AND_PANELS.md` Part 2. Sequenced into the stages:

| Capability | Stage | Cost | Note |
|---|---|---|---|
| Layout **persistence** | **S3** | 2–3 days | Foundation for everything, including modes. Drop `default-features = false` on `egui_tiles`; write `userdata/layout.json` beside `settings.txt`; trigger from `Behavior::on_edit`. R15's settings partition already landed to unblock this. |
| Two columns per side; tab-overflow menu | **S3** | ~1½ days | The overflow menu is what safely retires the current two-panes-per-group cap. |
| **Named workspaces → Read/Review/Edit** | **S3b** | 3–4 days | Modes *are* named workspaces. |
| Collapse to icon rail | **S6** | ~1 week | Mostly icons, tooltips and AccessKit names. Budget the harness-coordinate re-baseline. |
| **Fit-zoom cache (R128)** | **S6** | own landing | Prerequisite for anything that makes the canvas rect user-variable. |
| **Tiled rendering** | **S6** | 1–2 weeks | *Promoted 2026-08-13 from post-fold-in.* It is what lifts the zoom ceiling — the A1 benchmark caps at 3.4× on HiDPI under whole-page raster. See `BENCHMARK.md` § "The zoom ceiling". |
| Cross-dock drag, via one wide tree | **post-fold-in** | 1–2 weeks | The real unlock, but it puts the canvas in a resizable pane. Do R128 first. |
| Tear-out to a floating window | **post-fold-in** | 1 week cut-down / 2–4 weeks full | Start with a stationary "Float this panel…" command, not drag-to-tear. |

**Calibration note.** "As flexible as Inkscape" is a **floor**.
Inkscape is best-in-class on multi-column docking and tear-out, but has
**no named workspaces, no in-app layout reset, and no per-dock
collapse** — the last a regression from its own 1.0, still open five
releases later. pdfcer already beats it on all three: the mode selector
*is* named workspaces, and `Action::ApplyResetLayout` with per-scope
checkboxes is a better reset than any product surveyed. The target is
Inkscape's flexibility plus Photoshop's and Affinity's layout
management. Twelve specific failure modes to design against are tabulated
in `MODES_AND_PANELS.md` Part 2.

### 4.2b Known structural item — `pdfcer-gui` has no `lib.rs`

Modules are declared in `main.rs`, so the crate is a binary with no
library target. Consequences, none urgent, all compounding:

- `ui-verify` and any integration test cannot `use pdfcer_gui::…`; every
  assertion has to go through the process boundary even when it is
  really a unit-level question.
- `cargo doc` documents a binary, so the module docs this project
  insists on are not browsable.
- `main.rs` becomes the one file every new module must touch, which is a
  contention point for parallel work and the one place a merge conflict
  is guaranteed.

**Fix:** a `lib.rs` holding the module tree and a `main.rs` reduced to
`fn main() { pdfcer_gui::run() }`. Cheap in isolation, but it changes
visibility on every module, so it wants a quiet moment rather than a
mid-stage one. **Do it at the S2→S3 boundary**, before the panel modules
multiply.

> ✅ **Done 2026-08-13**, at the boundary as planned. `main.rs` went from
> 154 lines to 12 and now holds only `argv` handling plus the
> `windows_subsystem` attribute, which is a property of the binary and
> cannot move. 103 tests unaffected. Argument parsing deliberately stayed
> in the binary: anything answerable without a window must be answered
> before one exists, so a terminal invocation never opens a window it
> then has to be told to close.

### 4.3 What the application owes the harness

Discovered by **building** `ui-verify` at S1, not by reading code. Each
is a small change in `pdfcer-gui` that removes a harness workaround, and
each must land before the check that needs it can stop being a
workaround.

| # | Requirement | Why | Lands |
|---|---|---|---|
| 1 | **Trace the canvas layout unconditionally**, at least once per document open | The old binary traces it only on pointer events, so the harness cannot aim until it clicks and cannot click until it can aim. It currently works around this with one documented layout-probe click. One line removes it. | S2 |
| 2 | **Trace `ui-rect name=… rect=…`** per named UI region — ribbon group captions, settings headings, panel bodies | A rect measured on the frame it is reported for stays correct under every layout change. A fraction hard-coded in the harness is stale the first time a panel is resized — exactly the hazard §4.2 prerequisite 1 names. This is what un-skips `ribbon_group_captions_legible`. | S2 |
| 3 | **Trace a page object count** | Strictly better evidence than a `delete-objects` event: it measures the property the check is about rather than the verb meant to change it. | S2 |

**Three prerequisites that belong in S1, not later**, because every
capability above invalidates the assumption each one rests on:

1. **`ui-verify` scripts document-space coordinates**, never absolute
   screen coordinates. User-rearrangeable panels make widths arbitrary
   at runtime, and the RAG records this exact class producing a
   filed-then-retracted false coordinate-space defect.
2. **`ui-verify` has a screenshot oracle** for layout and clipping. Two
   recorded cases where a traced rect was correct and the control was
   still clipped out of its pane.
3. **Every new dockable surface gets a reachability test** that excises
   the harness driver and asserts the state-changing assignment survives
   — three panels shipped unreachable in real builds for their entire
   lifetime with all gates green.

**Phase 4 (page display modes), Phase 5b–d (text), Phase 6 (markup),
Phase 7 (measure), and the display list all land *after* fold-in**, in
the pdfcer repo, as ordinary Passes. They are improvements, not
prerequisites — and holding fold-in for them would keep the old GUI in
front of the operator for months longer than necessary.

---

## 5. What "done" means

Fold-in is gated on **parity plus the defects fixed**, not on the whole
roadmap.

**Required:**
1. Every `FEATURES.md` `gui`-column capability works. No regressions.
2. `DEFECTS.md` D1–D8 fixed, each with a regression test, and D1/D2 with
   a `ui-verify` assertion specifically.
3. `RIBBON_IA.md` implemented, including the Format tab and the
   properties panel.
4. **`MODES_AND_PANELS.md` implemented** — the Read/Review/Edit
   selector, and panel layout that survives a restart.
5. All pdfcer CI gates green (§7.2).
6. No file over 1,500 lines.
7. `ui-verify` suite green, and it demonstrably detects the two founding
   defects when pointed at the old binary.

**Explicitly not required:** continuous scroll, multi-run text editing,
the missing markup kinds, area/angular measure, the display list, OCR,
comparison — and, from the panel work, **cross-dock drag and tear-out to
a floating window**. All post-fold-in.

---

## 6. Rules while the project runs

1. **`D:\Dev\pdfcer\` is read-only.** The governing rule.
2. **The old GUI is feature-frozen** by agreement — bug fixes are fine
   and get replayed into the new shell, tracked in `SALVAGE.md`.
3. **Engine needs go to `pdfcer-engineer`** as a written hand-off, land
   in pdfcer as their own Pass, and are picked up via the path dependency.
4. **Re-sync deliberately.** At each stage boundary, rebuild against
   current `pdfcer-core` and record the commit built against.
5. **Docs stay current in the same commit as the code**, per the
   documentation-first rule.

---

## 7. Fold-in procedure

Executed once, deliberately, with the operator present. Not by an agent
acting alone.

### 7.1 Pre-flight

- [ ] `FEATURES.md` gui column audited row by row, new vs old.
- [ ] Both binaries driven side by side on the benchmark drawing and on
      a form-heavy, a signed, and an encrypted document.
- [ ] `DEFECTS.md` D1–D8 each closed with a named test.
- [ ] `ui-verify` green; confirmed to fail against the old binary.
- [ ] Performance no worse: first render, zoom-settle, memory, measured
      per `BENCHMARK.md`'s method.
- [ ] Operator has personally used the new GUI on real work and signed
      off.

### 7.2 Gates — all green in the pdfcer workspace after the swap

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/check-ui-strings.sh
tools/check-theme-colors.sh
tools/check-settings-consumed.py
tools/check-disclosure-channel.sh
tools/check-bypass-paths.sh
tools/check-shipped-assets.py
tools/check-fmt-excluded.py
```

Plus pdfcer's filing gates — `check-passes-filed.py`,
`check-commits-filed.py`, `check-one-commit-per-command.py` — which
means the fold-in is filed as Passes in `ROADMAP.md` like any other work.

### 7.3 The swap

```bash
cd /d/Dev/pdfcer
git checkout -b gui-rebuild-foldin
git tag pre-gui-rebuild                      # the rollback point
git rm -r crates/pdfcer-gui
cp -r /d/Dev/pdfcer-gui/crates/pdfcer-gui crates/pdfcer-gui
cp -r /d/Dev/pdfcer-gui/tools/ui-verify tools/ui-verify
# path deps become workspace deps in crates/pdfcer-gui/Cargo.toml
cargo build --release && cargo test --workspace
```

Then §7.2 in full, then the documentation:

- `docs/ARCHITECTURE.md` §12 — a dated decision record for the rebuild.
- `docs/ROADMAP.md` — the fold-in filed as Passes.
- `docs/FEATURES.md` — gui column re-audited against reality.
- `docs/SESSION_LOG.md` — the append-only record.
- `README.md` — the `DEFECTS.md` D3 corrections (Bates, PDF/A,
  imposition) land here if they have not already.
- `.claude/agents/pdfcer-gui-engineer.md` moves in, or is retired and its
  standing rules merged into `pdfcer-engineer.md`.

### 7.4 Rollback

`git reset --hard pre-gui-rebuild`. The old GUI is intact in git and in
`D:\Dev\pdfcer` until the merge is pushed. **Keep `D:\Dev\pdfcer-gui` on
disk for at least one release cycle after fold-in.**

---

## 8. Risks

| Risk | Mitigation |
|---|---|
| **Scope creep** — the roadmap is Phase 0–7, fold-in needs only parity. | §5 states what is *not* required. Everything else lands after, in pdfcer. |
| **Core divergence** during a long build. | Path dependency compiles against live core; re-sync at every stage boundary; old GUI feature-frozen. |
| **Salvaged code carries its bugs across.** | R3: every salvaged file is read in full and re-verified, and its `DEFECTS.md` fixes applied at salvage time, not later. |
| **The rebuild loses hard-won correctness** that lives in details nobody remembers. | The doc comments are the memory, and R5 requires carrying them across with the code. Never salvage by pasting a snippet. |
| **ui-verify is flaky** — OS-driven input tests often are. | Assert on `PDFCER_DIAG` first and pixels second; keep pixel assertions to contrast thresholds and presence, not exact images. |
| **It never ships** — the classic rewrite failure. | Every stage is runnable; fold-in is gated on parity, not perfection; S7 is a hard audit rather than a judgement call. |
| **egui version skew** between the two workspaces. | Same `rust-toolchain.toml`; no dependency not already in pdfcer's lockfile without an operator decision. |

---

## 9. Open questions for the operator

1. **Timeline and appetite.** S0–S7 is a substantial build. Is this a
   continuous push, or something that runs alongside pdfcer work? It
   changes how hard the feature-freeze in §6.2 has to be.
2. **Git.** Should `D:\Dev\pdfcer-gui` be its own repo, a branch of pdfcer,
   or untracked working space? The plan assumes its own repo. A branch
   would make the fold-in a merge instead of a copy, which is tidier in
   history but means the old GUI and new live in one tree.
3. **Feature freeze on the old GUI.** §6.2 assumes it. Acceptable?
4. **`ui_text.rs` split.** 7,912 lines breaks R2 and must be split into
   a `text/` module directory. That requires the §4.1 gate fix to also
   generalise its single-file exclusion to a directory. Both changes are
   one hand-off to `pdfcer-engineer`; confirm you want them filed as a
   pdfcer Pass before S0 starts.
5. **The three still-open roadmap questions** — Save semantics,
   comparison, and how much of multi-run text editing — do not block
   fold-in but do shape S2 and S5.
