# pdfcer-gui — project charter

The charter for this workspace: what it is, where it sits relative to the engine,
the invariants every stage honours, what "done" means, and the runbook for
folding the result into the engine repository. Read it before changing the
topology, the dependency form, or the stage order. `DEVELOPING.md` is the
day-to-day engineering guide; `FEATURES.md` is authoritative for what is built.

---

## 1. What this project is

The `pdfcer` GUI, built in its own Cargo workspace at `D:\Dev\pdfcer-gui\`,
against the `pdfcer` engine at `D:\Dev\pdfcer\`. It is the only GUI the engine
has: the engine's workspace is `pdfcer-core`, `pdfcer-render`, `pdfcer-cli`,
`pdfcer-print`, `pdfcer-fetch` and produces the `pdfcer` CLI binary alone.

The genuinely new work is the shell — ribbon information architecture, selection
model, context menus, properties panel, dock. Substantial parts of the body
behind it were salvaged from a predecessor GUI crate and re-verified rather than
rewritten (R3).

### Why a separate workspace rather than a crate inside the engine

1. **The engine keeps shipping.** A GUI mid-rebuild inside the engine's
   workspace makes every intermediate state the shipping state, with no fallback.
2. **The module split cannot be done incrementally without churn.** Breaking a
   25,005-line file into a module tree touches essentially every line; done in
   place, that is a series of commits that make `git blame` useless.
3. **`cargo test` and the licence audit stay honest.** The engine's workspace
   membership is what `cargo-about` reads to generate its notice file and what
   its `cargo tree` separation checks read. A GUI crate in that graph changes
   both answers.

**The cost is divergence.** `pdfcer-core` keeps moving. §2's dependency form is
the mitigation: a build always resolves against a real engine commit, and a
breaking change surfaces at compile time rather than at fold-in.

---

## 2. Topology

```
D:\Dev\pdfcer-gui\                  this workspace
├── Cargo.toml                      workspace root; five members
├── crates\
│   ├── egui-shell\                 the REUSABLE shell framework. Knows nothing
│   │                               about PDF. Extracted to its own repository
│   │                               at or before fold-in. See SHELL_FRAMEWORK.md
│   ├── pdfcer-gui\                 the application
│   ├── native-window\              Win32 window ownership
│   └── native-clipboard\           the ordered clipboard transaction
├── tools\
│   ├── ui-verify\                  drives the release binary
│   └── gates\                      the CI gates; run-all.sh runs every one
├── fixtures\                       built by tools/gen-*.py, each with PROVENANCE.md
└── *.md                            the reference documents

D:\Dev\pdfcer\                      READ-ONLY from here
└── crates\{pdfcer-core, pdfcer-render, pdfcer-cli, pdfcer-print, pdfcer-fetch}
```

`native-window` and `native-clipboard` are separate crates for one structural
reason: `pdfcer-gui` carries `#![forbid(unsafe_code)]`, `forbid` cannot be
relaxed from the inside, and Win32 window ownership and clipboard placement are
both `unsafe`. They are split from each other by subject, not convenience.

### The engine dependency is a `file://` git dependency on a branch

```toml
pdfcer-core   = { git = "file:///D:/Dev/pdfcer", branch = "main", default-features = false }
pdfcer-render = { git = "file:///D:/Dev/pdfcer", branch = "main", default-features = false }
pdfcer-print  = { git = "file:///D:/Dev/pdfcer", branch = "main" }
```

**Do not "simplify" these to `path = `.** A path dependency compiles the
engine's *working tree*, and another session edits that tree live; a build then
either fails on somebody's half-written function or — worse — succeeds, and
produces a binary no commit describes. The `file://` + `branch` form resolves
**committed history only**, so:

- the engine tree's cleanliness is irrelevant, and `BUILD-INFO.txt` can always
  name the commit a build came from;
- unpushed engine commits are still reachable, which a GitHub remote pin is not;
- `Cargo.lock` records an exact revision, so picking up new engine work is a
  deliberate `cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print`
  rather than something that happens silently under a rebuild.

`pdfcer-print` takes no `default-features = false` because it declares no
features; the flag would be a claim about a feature set that does not exist.

Rust links all three statically, so the release binary already carries the
engine. There is no integration step before shipping, and a packaged build is
self-contained. What the git form does cost: **source cloned from GitHub will
not build** without `D:\Dev\pdfcer` present.

### There is exactly one claimant for "where is the engine"

`crates/pdfcer-gui/Cargo.toml`'s git URL — the same answer the compiler used.
`tools/engine_path.py` reads it there and refuses rather than returning a
default, so an instrument that cannot find the engine is loud. Nothing else in
this repository may hard-code the path: three instruments once did, a rename
rewrote the literals, and a gate reported `PASS: all 0 uncalled verbs` having
examined nothing.

### Version pinning

`rust-toolchain.toml` pins an exact patch channel and is kept identical to the
engine's. A dependency this workspace adds that is not already in the engine's
lockfile is an operator decision, not a convenience: the engine's dependency
posture is deliberate — all-permissive licences, no GPL PDF engines, `skrifa`
pinned to epaint's. `about.toml`'s `accepted` list is permissive-only, so a
copyleft dependency entering the graph makes `cargo about` fail and name the
crate.

---

## 3. Module architecture and the invariants

**No source file over 1,500 lines** (R2), gated by
`tools/gates/check-file-size.sh` from every commit. When a file approaches the
limit, find the seam; do not raise the limit. `DEVELOPING.md` §2 has the current
tree layout.

`pdfcer-gui` has a **library target**. `src/main.rs` reads `argv`, carries the
`windows_subsystem` attribute (a property of the binary, which cannot move) and
calls `pdfcer_gui::run`. Everything else is in `src/lib.rs`'s module tree, so
`tools/ui-verify` and integration tests can `use pdfcer_gui::…` instead of going
through the process boundary for a unit-level question, and `cargo doc` has
something to document. Argument handling stays in the binary because anything
answerable without a window must be answered before one exists — a terminal
invocation must never open a window it then has to be told to close.

### The invariants

These govern the program and are not up for renegotiation.

- **Actions, not mutations.** No code path runs from a widget to a `Document`.
  Every gesture produces an `Action`, actions are collected while the frame is
  drawn, and they are applied after it in one place. Four things fall out of it:
  a coherent undo log (one gesture, one action, one command-log entry); the
  borrow checker stops fighting an immediate-mode toolkit that is reading the
  document to draw the widget that wants to change it; ordering between two
  actions raised in one frame is explicit; and every state change is greppable.
  Retrofitting it is expensive — every widget written under another discipline
  has to be found and rewritten, and the ones missed are exactly the invisible
  holes in the history.
- **Panel composition order is load-bearing** for both geometry and Tab focus.
  `app::frame` is the one file that answers "what happens, in what order, sixty
  times a second": theme before any widget, keyboard before any widget can
  consume a key, dialogs after the docks so they paint over rather than under,
  the zoom anchor after the commands that raise one, rasterize last so it
  measures a settled frame. Document the order where it is written.
- **A panel whose size feeds a fit-to-viewport computation has a fixed size**
  (R128). `FitMode::Page`/`FitMode::Width` recompute zoom from the canvas
  viewport every frame they are active, so one extra status line on frame N
  produces a smaller fit scale on frame N+1 — a page that visibly shrinks across
  frames, and click coordinates that go stale between capture and render. The
  symptom reads as a selection bug and gets investigated in the selection code,
  where nothing is wrong. Two defences, both required: the caller passes
  `exact_size` (`default_height` is only a starting value, `min_height`/
  `max_height` bound a range the panel still varies inside, and
  `resizable(false)` only stops the operator dragging the edge); and the content
  cannot grow anyway — one allocated row, disclosure drawn on the same row with
  elision and hover, no `CollapsingHeader` anywhere, whose entire behaviour is to
  change its own height.
- **The application's content area is not inside a dock compartment.** The dock
  draws side panels; the application draws its canvas in whatever remains. That
  is the other half of R128, respected by omission.
- **One `EditSession` command log**, bounded depth, undo tooltips naming the
  specific operation.
- **The ribbon picks the activity; the sidebar holds its controls.**
- **No placeholders** (R9). An unavailable capability renders nothing. Greying is
  reserved for *temporarily* unavailable and is always explained on hover. This
  applies to enum variants and to prose as much as to labels: an `Action` variant
  nothing can honour is a placeholder.
- **Floating is two independent settings, not one law.** *Floating panels*
  (Off · Allowed, default Allowed) governs whether the operator may tear a panel
  out. *App initiative* (Never · Ask · Allowed, default **Never**) governs
  whether the application may float a surface over the canvas on its own — tool
  option boxes, transient property bars, notifications. The second carries the
  original complaint (an accept/reject box that appeared over the drawing and
  moved on every zoom), and its default preserves that outcome as shipped
  behaviour while making it a choice. Both are per-operator, not per-document.

---

## 4. Build stages

Each stage produces a **runnable program**. There is never a period where the
crate is a pile of modules that does not launch. `FEATURES.md` is authoritative
for which stages are behind us.

| Stage | Contents | Gate to pass |
|---|---|---|
| **S0 — Skeleton** | Workspace, crates, CI gates, bootstrap, `diag`. Opens a PDF and renders page 1 through `render/worker`, `render/raster` and the viewer. | Renders the benchmark drawing; every gate green; tests pass. |
| **S1 — ui-verify** | The harness, before any UI is built. Drives the release binary, scripts input, captures the window, asserts on the trace **and** the pixels. | Its founding checks fail when pointed at a binary carrying the defects they were written for. A check whose precondition is absent reports SKIPPED, never a false pass. |
| **S2 — Shell** | Ribbon per `RIBBON_IA.md` — every tab, group, caption, ownership test. QAT, status bar with editable page box, dock with persistence, theme, icons. | Every command in the IA's migration map reachable; the band's full width ladder (re-wrap, collapse, scroll) per `RIBBON_SCALING.md`. |
| **S3 — Panels** | Pages, Objects, Properties, Bookmarks, Layers, Signatures, Fonts, Comments, Forms — salvaged bodies, new hosting — plus the flexible-dock foundation (§4.2). | Every panel-reachable capability in `FEATURES.md` works; layout survives a restart. |
| **S3b — Modes** | The Read / Review / Edit selector (`MODES_AND_PANELS.md` Part 1), built on S3's named-workspace mechanism, on `Ctrl+1/2/3`. | All three modes render; switching preserves undo and unsaved work; a signed document opens in Read with a stated reason. |
| **S4 — Selection** | Context menus, handles, move/resize/rotate, node editing, object clipboard, the Format contextual tab. No editing master toggle. | Place a rectangle, click away, click it, drag, resize, type a width, recolour, right-click, delete. Every step, driven. |
| **S5 — Tools** | Text, vector, measure, markup, forms, redact — salvaged and rehosted, with the disclosure surfaces the predecessor lacked. | Parity on all tool capabilities. |
| **S6 — Viewer** | Cursor-anchored zoom, hand tool, zoom to selection and region, recent files, rulers, grid, guides. | Anchor drift under 3 px, driven. |
| **S7 — Parity audit** | `ENGINE_BACKLOG.md`: a written verdict on every capability the engine has that this shell does not reach. | §7.1 complete, and `bash tools/gates/check-engine-backlog.sh` green. |
| **S8 — Fold-in** | §7. | Ships from the engine repository. |

**S7 is an instrument, not a document.** The engine's own `docs/FEATURES.md` is
a table whose first three columns are `core | cli | gui`; a row reading `[x]`
under `core` and `[ ]` under `gui` is the engine stating in a machine-readable
place that it has something this shell does not. `ENGINE_BACKLOG.md` gives every
such row a verdict — *wanted*, *declined with the argument*, or *blocked on
something named* — and `tools/gates/check-engine-backlog.sh` fails the build when
a row appears in none of those states. A document answers "what is missing
today"; a gate answers "what appeared since".

**Never retype a count out of that register.** `bash
tools/gates/check-engine-backlog.sh` prints the row and entry totals on every
run, and `python tools/walk-engine-backlog.py` prints the five verdict headings
(`--check` fails when a heading disagrees; `--write` is the only supported way to
move one). The headings went wrong seven times while the walk existed only as
prose, and the seventh was a commit that honoured the rule and still moved the
numbers by arithmetic rather than by re-walking. A walk described in prose is a
walk that will be replaced by arithmetic.

**A capability announced in an API has a gate. A capability announced in prose
does not.** `tools/gates/check-verb-coverage.sh` reads the engine's API and fails
when this shell names none of a new verb, within hours. A capability the engine
ships and describes in a note reaches nobody unless somebody reads the note —
which is the asymmetry `check-engine-backlog.sh` exists to close.

**What lands after fold-in, in the engine repository, as ordinary work:** page
display modes, live layout while typing, reflow reachability, multi-run text
editing, the remaining markup kinds, area and angular measure, the display list,
OCR, comparison. They are improvements, not prerequisites.

### 4.1 A gate that finds nothing prints what a clean gate prints

This is the failure mode every gate in `tools/gates/` is built against, and it is
the reason the runner is a script rather than a list of steps in CI YAML.

- **A grep over source fails silently.** A pattern that stops matching, a path
  that stops resolving, and a `find` that walks an empty tree all print exactly
  what a clean run prints. A gate whose glob is flat (`for file in "$SRC_DIR"/*.rs`)
  stops seeing a module tree the moment the first subdirectory exists — and
  reports success. Scan recursively.
- **Every gate that is a grep over source carries a `--self-test`** that plants a
  violation and asserts the gate catches it. The self-tests run *first*, before
  any gate is trusted: a gate that cannot detect its own planted violation has no
  verdict worth reading, and finding that out after a green run is finding it out
  too late.
- **SKIPPED is not PASSED.** `run-all.sh` has three states: `0` pass, `1` fail,
  `3` at least one gate was skipped because its precondition was absent. Skips
  are printed in their own block with their reasons. CI must not go green on a
  gate set that did not fully run, and the machine does not get to decide that a
  skip was expected.
- **A sweep that omits a gate is byte-indistinguishable from a green one.** Run
  the runner, never a hand-typed list.
- **A `pub const` nothing uses is invisible to the whole toolchain** — `pub`
  suppresses `dead_code`. `check-region-names.py` is why a declared trace region
  must be reached by something.

### 4.2 Panel flexibility — what is left, and the order it must come in

Full analysis in `MODES_AND_PANELS.md` Part 2. Built: layout persistence, two
columns per side, vertical resizable stacks, tabs within a stack with a reserved
overflow menu (which is what retired the two-panes-per-side cap), named
workspaces as the mode selector, per-scope layout reset, collapse to an icon
rail, and tear-out to a floating window.

Two items remain, and they are ordered:

| Item | Why it is next, or why it is blocked |
|---|---|
| **Fit-zoom cache (R128)** | Convert the fit computation from recompute-every-frame to cached-recompute-on-explicit-trigger. Its own landing. Prerequisite for anything that makes the canvas rect user-variable. |
| **Cross-dock drag, via one wide tree** | The real unlock, and it puts the canvas inside a resizable pane, which fires R128 directly. Blocked on the fit-zoom cache. |

Tear-out was built as a **command, not a drag** — "Float this panel…" on a tab's
secondary menu. That captures most of the value at a fraction of the cost, dodges
the focus-gated `StartDrag` primitive, and sidesteps the ambiguous-drag-handle
failure mode, which is the most-reported docking complaint in the product used as
the benchmark. A drag-to-tear gesture can be added on top without changing the
model, because the model's question is *where is this panel and where did it come
from* and a drag is only one way of answering it. A floated panel remembers where
it came from, and docking it back puts it there.

**Calibration.** "As flexible as Inkscape" is a **floor**. Inkscape is
best-in-class on multi-column docking and tear-out and has no named workspaces,
no in-app layout reset and no per-dock collapse. The target is Inkscape's
flexibility plus Photoshop's and Affinity's layout management. Twelve specific
failure modes to design against are tabulated in `MODES_AND_PANELS.md` Part 2.

### 4.3 What the application owes the harness

Three contracts `pdfcer-gui` honours so that `tools/ui-verify` needs no
workarounds. Each was discovered by *building* the harness, not by reading code.

| # | Requirement | Why |
|---|---|---|
| 1 | **Trace the canvas layout unconditionally**, at least once per document open | Tracing it only on pointer events means the harness cannot aim until it clicks and cannot click until it can aim. |
| 2 | **Trace `ui-rect name=… rect=…`** per named UI region — ribbon group captions, settings headings, panel bodies | A rect measured on the frame it is reported for stays correct under every layout change. A fraction hard-coded in the harness is stale the first time a panel is resized. |
| 3 | **Trace a page object count** | It measures the property a check is about, rather than the verb meant to change it — strictly better evidence than a `delete-objects` event. |

Three standing prerequisites that every capability above would otherwise
invalidate:

1. **Scripts are written in document-space coordinates, never absolute screen
   coordinates.** Two reasons, and the second is the expensive one. Every screen
   coordinate in this application is variable — multi-column docks, overflow
   menus, workspaces, collapse, tear-out each move where the canvas begins, and a
   harness that says `click at 819,513` has to be re-baselined by hand after every
   layout change, which in practice means the checks quietly stop testing
   anything. And **a stale screen coordinate is symptom-identical to a broken
   coordinate conversion**: the trace shows a hit test returning nothing, which is
   exactly what a genuinely broken document-to-screen conversion looks like. This
   project has already filed and retracted a false coordinate-space defect from
   that confusion.
2. **The harness has a screenshot oracle** for layout and clipping. There are
   recorded cases where a traced rect was correct and the control was still
   clipped out of its pane.
3. **Every new dockable surface gets a reachability test** that excises the
   harness driver and asserts the state-changing assignment survives. Three panels
   once shipped unreachable in real builds, for their entire lifetime, with all
   gates green.

A seam exists for what OS input cannot reach. **There were 26 of them when this
paragraph was last measured (2026-09-16)** and the number only ever grows, so
what follows names the *kinds* rather than the members. Measure the membership
rather than quoting this list:

```bash
grep -rhoE 'PDFCER_DIAG[A-Z_]*' crates/pdfcer-gui/src crates/pdfcer-gui-base/src \
  tools --include=*.rs -r | sort -u
```

That command reports 28 lines: the 26 seams, plus `PDFCER_DIAG` itself (the
channel switch, not a seam) and a bare `PDFCER_DIAG_` fragment. Note also that
several seams are read through a `const` whose own name omits `DIAG`
(`FONT_DIR_ENV` is one), so a grep for the *variable* name under-counts.

- `PDFCER_DIAG_VIEWPORT` — a real, laid-out, invisible window. It cannot be
  driven by OS input at all, so a headless run reads its trace and presses
  nothing. Every other seam exists to give that window something to do.
  — **and its height is silently clamped to the monitor's work area**, so a
  control below the fold cannot be revealed by asking for a taller window. See
  `D:/dev/rag/egui/`; a check that assumes otherwise fails identically on a
  correct build and a broken one.
- `PDFCER_DIAG_INVOKE` — a comma-separated list of command ids, rung one per
  frame through the same `dispatch_command` a chord reaches. A list of
  doorbells, deliberately not a grammar.
- **A family of path seams** — a native file picker, an Explorer drop and a
  certificate store are each a hard wall for synthetic input, so the surface
  that would have opened one reads a path instead. `PDFCER_DIAG_OPEN_PATH` is
  the original and there are fifteen more, which is over half the family.
- **State and decision seams** — where the state a check needs is reachable
  **only** by a pointer gesture or a keystroke the harness cannot aim.
  `PDFCER_DIAG_TYPE` seeds a text draft, `PDFCER_DIAG_SELECT_FIELD` selects a
  form field by name, `PDFCER_DIAG_FIND` seeds a search, and
  `PDFCER_DIAG_FORM_ACCEPT` / `PDFCER_DIAG_PASTE_CHORDS` supply a decision the
  operator would otherwise make in a dialog or a preference. This is the same
  wall as a file picker arriving from the other direction: the Properties pane
  for a form field had no headless route to it **at all** until
  `PDFCER_DIAG_SELECT_FIELD` landed, so the largest editing surface in the shell
  was R1-unreachable on any day the operator was at his machine — which is most
  days.

The four kinds are a description, not a partition: `PDFCER_DIAG_DROP_AFTER_MS`
is a *parameter of* a path seam rather than a seam, and
`PDFCER_DIAG_CERTIFICATE_PASSPHRASE` supplies a secret where a modal would ask
for one. A new seam that fits none of the four is a signal about the surface it
was added for, not a flaw in this list.

Without these, a feature would be implemented, unit-tested, and never once
exercised in a running window, which is the state R1 exists to forbid.

---

## 5. What "done" means

Fold-in is gated on **parity plus the defects fixed**, not on the whole roadmap.

1. Every `gui`-column capability in `FEATURES.md` works, with no regression (R6).
2. Every known defect closed with a named regression test — and the two founding
   defects (the Delete key dying the moment the canvas is clicked; section
   headings and dock tab labels invisible in the default theme) with a
   `ui-verify` assertion specifically, because each was a defect a passing test
   suite could not see.
3. `RIBBON_IA.md` implemented, including the Format contextual tab and the
   properties panel.
4. `MODES_AND_PANELS.md` implemented — the Read/Review/Edit selector, and panel
   layout that survives a restart.
5. All gates green in the engine workspace after the move (§7.2).
6. No source file over 1,500 lines.
7. The `ui-verify` suite green, and demonstrably able to detect the two founding
   defects when pointed at a binary that has them.

**Explicitly not required:** continuous scroll, multi-run text editing, the
missing markup kinds, area and angular measure, the display list, OCR,
comparison, cross-dock drag. All post-fold-in.

---

## 6. Rules while this workspace is separate

1. **`D:\Dev\pdfcer` is read-only.** The governing rule. Cargo *reads* that
   repository and clones from it into `~/.cargo/git/`; it writes nothing there,
   and builds into this workspace's `target/`. **Do not `cd` into the engine tree
   to build** — that is the one way this arrangement can breach the rule.
2. **Engine needs are written up and handed to the operator**, land in the engine
   repository as their own work, and are picked up by a deliberate `cargo update`
   of the three engine crates. Nothing is applied there from here.
3. **Re-sync deliberately.** At each stage boundary, `cargo update` the engine
   crates and record the revision built against.
4. **Docs stay current in the same commit as the code.**
5. **A UI change is done when it has been asserted in `tools/ui-verify` against
   the running binary** (R1) — not when a test passes.

---

## 7. Fold-in procedure

Executed once, deliberately, with the operator present. Not by an agent acting
alone.

Fold-in is an **addition, not a swap**: the engine workspace has no GUI crate to
remove. It brings this workspace's crates into `D:\Dev\pdfcer`, registers them as
workspace members, and converts the three `file://` git dependencies into
ordinary path dependencies.

`egui-shell` folds in **differently, on purpose.** It is extracted to its own
repository (`D:\Dev\egui-shell`) and consumed by the engine as a path or git
dependency — *not* copied into `crates/`. A crate living inside another project's
tree is not reusable in any practical sense, and reusability is the whole point of
the split. `tools/gates/check-shell-purity.sh` is what keeps the extraction cheap:
while it stays green, extraction is a `git mv`. The failure it catches is not a
crash but one `use pdfcer_core::PageSize` in a layout helper, added because it was
convenient — which makes the standalone repository not compile, inverts the
dependency the architecture rests on, and is invisible to every other gate.

### 7.1 Pre-flight

- [ ] `FEATURES.md` `gui` column audited row by row.
- [ ] `ENGINE_BACKLOG.md` complete: every live `[x] core` / `[ ] gui` row in the
      engine's `docs/FEATURES.md` carries a verdict, and
      `bash tools/gates/check-engine-backlog.sh` is green.
- [ ] Driven on the benchmark drawing
      (`D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf`) and on a form-heavy, a
      signed and an encrypted document.
- [ ] Every known defect closed with a named test.
- [ ] `ui-verify` green; confirmed to fail against a binary carrying the founding
      defects.
- [ ] Performance no worse — first render, zoom-settle, memory — measured by
      `BENCHMARK.md`'s method.
- [ ] Operator has personally used the build on real work and signed off.
- [ ] `egui-shell` extracted to `D:\Dev\egui-shell` and building standalone.

### 7.2 Gates

In this workspace, before the move:

```sh
cargo build --release
cargo test --workspace
bash tools/gates/run-all.sh
```

In the engine workspace, after the move — and the sweep is the runner, never a
hand-typed list, because `tools/run-gates.sh` is derived from the CI workflow
rather than remembered:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
bash tools/run-gates.sh
```

The engine's filing gates (`check-passes-filed.py`, `check-commits-filed.py`,
`check-one-commit-per-command.py`) are part of that run, which means the fold-in
is filed in `docs/ROADMAP.md` like any other work there.

### 7.3 The move

```sh
cd /d/Dev/pdfcer
git checkout -b gui-foldin
git tag pre-gui-foldin                                   # the rollback point

cp -r /d/Dev/pdfcer-gui/crates/pdfcer-gui        crates/pdfcer-gui
cp -r /d/Dev/pdfcer-gui/crates/pdfcer-gui-base   crates/pdfcer-gui-base
cp -r /d/Dev/pdfcer-gui/crates/native-window     crates/native-window
cp -r /d/Dev/pdfcer-gui/crates/native-clipboard  crates/native-clipboard
cp -r /d/Dev/pdfcer-gui/tools/ui-verify          tools/ui-verify
cp -r /d/Dev/pdfcer-gui/tools/gates              tools/gates
cp -r /d/Dev/pdfcer-gui/fixtures                 fixtures/gui
```

Then four edits, and no others:

1. **Engine root `Cargo.toml`** — add `crates/pdfcer-gui`,
   `crates/pdfcer-gui-base`, `crates/native-window`, `crates/native-clipboard`
   and `tools/ui-verify` to
   `[workspace] members`, and add `egui-shell` to `[workspace.dependencies]`
   pointing at `D:\Dev\egui-shell`.
2. **`crates/pdfcer-gui/Cargo.toml` and `crates/pdfcer-gui-base/Cargo.toml`** —
   in each, replace every
   `git = "file:///D:/Dev/pdfcer"` line with `path = "../pdfcer-core"`,
   `path = "../pdfcer-render"`, `path = "../pdfcer-print"`, keeping each line's
   `default-features` setting exactly as it is; replace `version = "0.1.0"` with
   `version.workspace = true`, since the crate is versioned by the workspace it
   folds into.
3. **`tools/engine_path.py`** — the engine is now the containing repository; the
   derivation collapses to the workspace root. It must still refuse rather than
   return a default.
4. **The CI workflow** — add the gate jobs from `tools/gates/`, and keep
   `check-ci-parity.py` satisfied so `run-gates.sh` stays derived from it.

Then:

```sh
cargo build --release && cargo test --workspace
```

and §7.2 in full, then the documentation:

- `docs/ARCHITECTURE.md` — the decision record for the GUI's arrival.
- `docs/ROADMAP.md` — the fold-in filed.
- `docs/FEATURES.md` — `gui` column re-audited against reality.
- `README.md` — any capability claim this project corrected.
- This workspace's reference documents move alongside the crates they describe.

### 7.4 Rollback

`git reset --hard pre-gui-foldin`. **Keep `D:\Dev\pdfcer-gui` on disk for at
least one release cycle after fold-in.**

---

## 8. Risks

| Risk | Mitigation |
|---|---|
| **Scope creep** — the roadmap is far larger than parity. | §5 states what is *not* required. Everything else lands afterwards, in the engine repository. |
| **Core divergence** during a long build. | The git dependency compiles against a real engine commit; `cargo update` at every stage boundary; a breaking change surfaces at compile time. |
| **Salvaged code carries its bugs across.** | R3: every salvaged file is read in full and re-verified, and its known defects fixed at salvage time, not later. |
| **The rebuild loses hard-won correctness** living in details nobody remembers. | The doc comments are the memory; carry them across with the code. Never salvage by pasting a snippet. |
| **`ui-verify` is flaky** — OS-driven input tests often are. | Assert on the `PDFCER_DIAG` trace first and pixels second; keep pixel assertions to contrast thresholds and presence, not exact images. |
| **It never ships** — the classic rewrite failure. | Every stage is runnable; the workspace already packages a portable build with the engine statically linked; fold-in is gated on parity, not perfection; S7 is a hard audit rather than a judgement call. |
| **egui version skew** between the two workspaces. | The same `rust-toolchain.toml`; no dependency not already in the engine's lockfile without an operator decision; `UI_TOOLKIT_PINS.md` and `check-ui-toolkit-drift.sh`. |
| **`egui-shell` quietly learns what a PDF is**, and extraction is cancelled on the day it is attempted. | `check-shell-purity.sh` from every commit. Add an extension point, not an exception (R7). |

---

## 9. Open questions for the operator

1. **Does fold-in still happen?** The engine has already removed its GUI crate
   and this workspace ships a self-contained portable build, so the outcome
   fold-in was meant to deliver — a shipping GUI in front of the operator —
   arrives without it. The remaining arguments for it are one repository, one CI
   run, one filing discipline; the argument against is that the separation is
   what keeps the engine's licence audit and workspace-separation checks
   unambiguous.
2. **`egui-shell`'s destination.** Its own public repository, or a private one?
   §7 assumes a repository at `D:\Dev\egui-shell` either way, but the licence
   posture and the notice obligations differ.
3. **The three open scope questions in `GUI_ROADMAP.md`** — comparison, how much
   of multi-run text editing, and whether the phased plan's remaining phases are
   wanted at all — do not block fold-in but do shape what follows it.
