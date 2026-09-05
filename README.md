# pdfcer-gui — GUI design and remediation workspace

The **GUI rebuild** for **`D:\Dev\pdfcer`**: a new `pdfcer-gui` crate that
will replace `D:\Dev\pdfce\crates\pdfce-gui\` when complete, built on a
new reusable `egui-shell` crate that knows nothing about PDF and will be
extracted for use by other projects.

Started 2026-08-12 as a design workspace. It became the code on
2026-08-13.

## What it does today

Measured 2026-09-05 against `pdfcer-core` **v0.38.0** (`b01964f`), at commit
`a926423`: **3,390 tests passing**, **29 of 29 CI gates**, **175 driven checks**
in `tools/ui-verify`, **138 registered commands** across seven ribbon tabs plus
a contextual Format tab, **12 dockable panels**, three modes.

**Read** — open, navigate, zoom, search, thumbnails, bookmarks, layers,
attachments, page display modes, rulers, grid and guides. Encrypted documents
open with a password. **Comments are readable here** — click a sticky note on
the page and it opens with its author, its date and its words.

**Review** — sticky notes, text boxes, shapes, arrows, clouds, freehand,
highlight/underline/strike, stamps. A comment list that filters by author, by
type and by whether the comment actually carries words, sorts, and jumps to the
comment on the page. Copy and paste any annotation, with its appearance intact.

**Measure** — ce dimensions on a scaled drawing, with the scale read from the
document where the document states one.

**Edit** — text editing and reflow, vector node editing, object selection with
properties, colour on text and on paths (including a real indeterminate state
when a selection disagrees, and a refusal by name over a spot ink), form field
authoring and filling, page insert/extract/rotate/delete, redaction, digital
signature reading with trust evaluation, passwords and permissions.

**Out** — print with a live preview that can pop into its own window; export to
PDF, DXF, PNG, JPEG, SVG, EMF, plain text and form data; copy page content to
the clipboard as editable vector for Word, Inkscape and LibreOffice.

★ `FEATURES.md` is the authoritative list, and it distinguishes three states
rather than two: ✅ shipped **and driven in a running window**, ⬜ built and
**undriven**, and absent. That distinction is the project's founding rule (R1)
made visible — this shell has shipped features that every unit test passed and
that did not work.

## Deep zoom — to a trillion percent, with the detail actually there

The viewer magnifies to **1,000,000,000,000 %**. Reaching the number is the
easy half; three things make it usable rather than a setting nobody can
spend:

| | |
|---|---|
| **the raster follows the window, not the page** | past the point where a whole-page raster would exceed the backend's texture limit, only the visible region is rasterized. A viewport is a fixed number of pixels whatever the page behind it is doing, so **render time does not grow with zoom** |
| **panning stays smooth** | the last good picture is kept and drawn *where it belongs* while the next one renders, so detail never has to be waited for after moving. The region is quantised to a half-viewport grid, so a small pan re-uses the raster it already has |
| **the position survives the arithmetic** | a scroll offset is `f32` and gives out around 2²⁴ content points — measured, at a trillion percent it moved in 2,048-pixel jumps. Above that threshold the view's position becomes an `f64` anchor: a page point and the screen pixel it sits under, which does not decay with magnification |

**Measured, by driving the release binary rather than by testing the
arithmetic:** the point under the cursor is held to within half of a
per-notch tolerance across a climb of 128 wheel notches from page-fit to the
ceiling, crossing both internal tier hand-overs; a 960-pixel pan at
999,999,995,904 % moves 960 pixels and stays there.

The maximum is an **operator setting** — the status bar's percentage opens it
— because how much performance to spend on magnification is not a decision
this program should be making.

★ **What is actually out of reach**, since the honest version of a claim like
this needs a boundary: anything inside an atom. A carbon nucleus rendered
200 pixels across would need about 7 × 10¹⁴ %, roughly 700× past the ceiling;
a proton needs 4 × 10¹⁵ %. At the maximum zoom one screen point spans 35
femtometres. **Atoms are comfortable — a benzene molecule drawn at true scale
is legible on screen at 3.7 × 10¹⁰ % — and their contents are not.**

⚠ **A retraction, kept rather than deleted.** This section shipped on
2026-08-22 saying the limit was the renderer's own `f32` path precision — 11
nanometres near the middle of a letter sheet — and stated it as a property of
the design. It was a property of *that revision*: pdfcer removed it the
following day, and the same molecule that rendered as blank paper now draws
its bonds. **A measured limit is a fact about a revision, not about a
design**, and this project has now been wrong in that particular way once.

**➡ New session? Read `RESUME.md` first** — measured state, what to do
next, what not to do, on one screen. `HANDOFF.md` is the long-form record
behind it.

Run these rather than trusting a count written in prose; every status number
this project has written down has been overtaken within a day or two.

```
cargo test --workspace
bash tools/gates/run-all.sh
cargo run --release -q -p ui-verify -- --exe target/release/pdfcer-gui.exe --pdf D:/Dev/temp/pdfcer/SW41177.pdf --doc-point 0,300,500
```

⚠ **This paragraph used to say "measures nothing yet" and "Phase 5 — text
editing … has not been started."** Both were true when written and were still
on the front page weeks after they stopped being true, which is the exact
failure this project keeps finding in its own prose. The capability list above
is dated and was measured, not remembered. **`FEATURES.md` is authoritative;
this page is a summary with a shelf life.**

Stages **S0–S5** are complete, along with Phase 3 (viewer conventions, Find,
thumbnails, rulers/grid/guides), Phase 4 (page display modes) and Phase 5 (text
editing — the defect that began the project). `PROJECT_PLAN.md` §4 has the
eight-stage plan; `GUI_ROADMAP.md` has the phases.

## Builds

```
python tools/package-portable.py --verify --note "what this milestone added"
```

Writes `D:\builds\pdfcergui-<stamp>-<engine>-<shell>\` — one folder per
build, never an overwrite, because on Windows a running executable
cannot be replaced and a half-updated folder is worse than either
version.

**"Integrated with pdfcer as a single exe" needs no fold-in.**
`crates/pdfcer-gui` depends on `pdfcer-core` and `pdfcer-render` **by path**
into `D:\Dev\pdfcer`, and Rust links them statically — so the release
binary already carries pdfcer's engine. Integration here is a property of
the dependency graph, not a merge that has to happen first.

That is worth stating because the alternative — folding this shell into
`D:\Dev\pdfcer` and packaging from there — would ship a **regression**
today: `FEATURES.md` § "Not salvaged yet" still lists measure,
redaction, the settings dialog and text editing as living only in the
old shell. Shipping from here costs nothing and leaves a pdfcer build
installable beside it. Fold-in happens when `FEATURES.md` says nothing
regresses, per `PROJECT_PLAN.md` §5.

**Two identities, because there are two source trees.** `<engine>` is
`D:\Dev\pdfcer`'s short HEAD; `<shell>` is this workspace's. Either gets
a `-dirty` / `-enginedirty` marker when its tree carries changes that
can reach a compiler — narrowly defined, so a documentation edit does
not raise a warning about the binary.

A **source digest** is recorded alongside them, and joins the folder
name when the shell tree is dirty. It exists because commits here are
taken at milestones while several agents write concurrently, so the tree
is dirty more often than not — and for a dirty tree the commit names the
last checkpoint, not what was compiled. The digest names what was
compiled. It cannot say *what* the code was, only whether two builds
came from identical bytes, which is the question a bug report actually
asks.

`--verify` runs the workspace tests and the CI gates **before** building,
so a failure costs nothing and leaves no folder behind, and records the
result in `BUILD-INFO.txt`. When it is not run the file says so in those
words — an omitted line would read as "nothing to report", when it means
nobody checked.

`python tools/package-portable.py --self-test` asserts the script's own
two invisible invariants: that the folder name is **not** caught by
pdfcer's own `pdfcer-*` glob (it would otherwise diff *its* changelog
against a commit from this tree), and that the source digest is
deterministic and moves on a rename. Both failures package successfully
and run fine — the damage lands elsewhere, later — which is why they are
asserted rather than reasoned about.

## Licensing, and the material this binary carries that is not ours

pdfcer-gui is **MIT** — see `LICENSE`, `Copyright (c) 2026 Ken Mantle`.
That covers everything in this repository, including the icon set, which
is the operator's own art (`crates/pdfcer-gui/src/icons/assets/PROVENANCE.md`).

It does **not** cover everything `pdfcer-gui.exe` contains. The binary
links `pdfcer-core` and `pdfcer-render` statically, and those crates embed
third-party font faces and data tables with `include_bytes!` — so this
program redistributes work whose licences require their notices to
travel with it.

Two surfaces carry those notices, and they are not redundant:

| Surface | Carries | Reached by |
|---|---|---|
| `THIRD_PARTY_LICENSES.md`, copied into every build | every licence **text**, in full | anyone who opens the package folder |
| **File ▸ pdfcer ▸ About pdfcer**, in the program | the **attribution** — who made it, what it is, on what terms, and whether pdfcer changed it | anyone who runs it |

`THIRD_PARTY_LICENSES.md` is generated: `cargo about generate about.hbs
-o THIRD_PARTY_LICENSES.md`, from this workspace's real `Cargo.lock`.
Never edit it by hand. Its `accepted` list in `about.toml` is
permissive-only, so a copyleft dependency entering this workspace makes
generation **fail and name the crate** — that failure is the licence
audit. `about.hbs`'s static epilogue carries the non-Cargo assets, which
no generator can see.

`tools/gates/check-shipped-assets.py` enforces the whole arrangement: a
`PROVENANCE.md` beside every redistributed asset directory, a citation
in both notice surfaces unless the asset is our own work, the notice
present in the packager's payload, and the generated file not stale
relative to its template.

**When the OCR feature lands** it brings the `ocrs` model weights, which
are **CC-BY-SA-4.0** and which the operator accepted into this MIT
package on 2026-08-14. Shipping them *unmodified* is distribution of a
verbatim work in a collection and leaves pdfcer's own licence untouched.
**Modifying them — fine-tuning, retraining, quantizing to shrink the
file, or converting them to another runtime's format — creates Adapted
Material, and the result must be released under CC-BY-SA-4.0 or a
compatible licence.** That is an engineering constraint, not a footnote:
see `crates/pdfcer-gui/src/text/about.rs`.

## Version control

Under git since `2a504ef` (2026-08-13). `.gitattributes` **predates that
commit deliberately**: `core.autocrlf` is true globally on this machine,
and pdfcer's 2026-08-02 finding records that CRLF normalization of PDF
fixtures lands **in the index at add time**, not only at checkout. A
PDF's cross-reference table stores absolute byte offsets, so a
normalized fixture is a corrupt one, and 18 evidence PNGs would have
gone the same way. The first `git add` here was made without the file,
noticed, and unwound with `git rm -r --cached` before anything was
committed.

The rebuild is owned by the **`pdfcer-gui-engineer`** agent
(`.claude/agents/pdfcer-gui-engineer.md`). Its governing rule:
`D:\Dev\pdfcer\` is **read-only** until fold-in day, so the working
program keeps working for the whole life of the project.

## Read in this order

| Document | What it is |
|---|---|
| **`HANDOFF.md`** | **Start here in a new session.** Current state, the standing instructions, how the parallel agent work was actually run, the five obligations of registering a command, and what is left in order. It carries what the other documents cannot: the working agreements and the judgement calls. |
| **`FEATURES.md`** | **What works today, and what is next in order.** A row is ticked only when an operator can reach it in a real build — not when the code exists or a test passes. Start here for status. |
| **`PROJECT_PLAN.md`** | The charter: topology, module architecture, eight build stages, the fold-in procedure, risks, open questions. **Review this first.** |
| **`SALVAGE.md`** | What carries over from the old GUI's 49,837 code lines, file by file, in four classes. ~45 % comes across with little change. |
| **`DEFECTS.md`** | What is broken today, with `file:line` for every claim. Start here — it contains the diagnosis of the Delete key and of text editing. |
| **`GUI_ROADMAP.md`** | Phased plan, Phase 0 through Phase 7, plus a standing shell-only backlog and five open questions. |
| **`RIBBON_IA.md`** | The full information architecture: seven tabs plus one contextual tab, every command assigned, every current command migrated. |
| **`MODES_AND_PANELS.md`** | The **Read / Review / Edit** selector and the Inkscape-class flexible panel system. Operator additions, 2026-08-13. A mode is a named workspace layout, which is why the two are one system. |
| **`SHELL_FRAMEWORK.md`** | `egui-shell` — the reusable application shell. The ribbon, dock, modes and keymap are a **serializable manifest**, not code, which delivers cross-project reuse and operator ribbon-customization from one mechanism. |
| **`mockups/ribbon.html`** | Open in a browser. Interactive — click a tab to see its band. Colour-coded by whether each command exists today, exists in core/CLI only, or is new. |
| **`mockups/app.html`** | Six full-window scenes: object selected (Format tab + properties panel + context menu), View ▸ Render options, in-place text editing, the Pages tab, Measure, and placing a revision cloud. |
| **`mockups/modes.html`** | The same document rendered in Read, Review and Edit, with the selector at the far right of the tab row. |
| **`BENCHMARK.md`** | Measured rendering performance on a real 5.6 MB CAD site plan. This is the evidence that overturned an earlier, unmeasured claim about whole-page rendering. |
| **`evidence/`** | Screenshots backing every observational claim, plus `bench-gui-diag.txt`, the raw `PDFCER_DIAG` trace. |

## Evidence index

| File | What it shows |
|---|---|
| `pdfcer_max.png` | pdfcer, maximised, the shared test drawing |
| `ops_max.png` | The comparison product, same window size, same drawing |
| `crop_settings.png` | pdfcer Settings dialog at 3× — the invisible section headings (D2) |
| `crop_tabs_left.png`, `crop_tabs_right.png` | Dock tab labels at 3× — same defect |
| `pdfcer_settings.png`, `pdfcer_panels.png` | Settings dialog and the Objects + Fonts panels in place |
| `ribbon_*.png` | Every pdfcer ribbon tab: edit, review, measure, tools, view |
| `ops_*.png` | Every comparison-product ribbon tab: view, drawing, annotation, edit & combine, settings |

The shared test document is `grootformaat_a1_liggend.pdf`, an A1 landscape
title-block frame, chosen because it exercises vector paths, subset fonts
and annotations together at drawing scale.

## Headline findings

**The Delete key is broken by one line.** `main.rs:13777` guards the
unmodified-key bindings with `ctx.egui_wants_keyboard_input()`, which in
egui 0.35 means *any widget has focus* — not *a text field has focus* —
and the canvas takes focus on the very click that selects an object. The
same guard also kills PageUp/PageDown, Home/End and `[` / `]`. Fix is
`ctx.text_edit_focused()`. Full chain in `DEFECTS.md` D1.

**Section headings and dock tab labels are invisible** in the default
Quiet theme: `widgets.active.fg_stroke` is set to a near-white
`label_backdrop` while `widgets.active.bg_fill` is never assigned the
accent. `DEFECTS.md` D2.

**Three of the six ribbon tabs are underfilled, page operations are not
on the ribbon at all, and the View tab contains no view controls.**
`RIBBON_IA.md` §3.

**Text editing has three distinct problems**, not one: the edit unit is
a single PDF show-text operator rather than a visual box; nothing
re-lays-out while you type, and aligned or rotated text is moved wrongly
on commit; and reflow is blocked behind three gates including one open
filed defect. `DEFECTS.md` D4.

**Two of these were invisible to a green test suite.** `GUI_ROADMAP.md`
proposes a `tools/ui-verify/` harness that drives the real binary as the
highest-leverage item in the plan.

## Decisions taken, 2026-08-13

| Decision | Effect |
|---|---|
| **Pages belongs in Review mode** | Reviewing a set means rotating a sheet to read it and extracting the pages you were asked about. The stance that matters is *the content is not yours to alter*, and page operations do not alter content. |
| **"Nothing floats over the canvas" becomes two settings, not an invariant** | **Floating panels** (Off · Allowed, default Allowed) governs whether *you* may tear a panel out. **App initiative** (Never · Ask · Allowed, **default Never**) governs whether pdfcer may float something *on its own*. The second carries the original complaint and its default preserves today's behaviour — as a choice rather than a law. Both under View ▸ Window. |
| **The shell becomes a reusable crate** | `egui-shell` — ribbon, dock, modes, layout persistence, theme, command registry — knowing nothing about PDF, enforced by a CI gate, extracted to its own repo at fold-in. |
| **The ribbon becomes data** | Tabs, groups, commands, modes and keymap are a serializable manifest. This is what makes the shell reusable *and* the ribbon customizable — one mechanism for both. Retires the deferral at `ribbon.rs:42-52`, whose objection was about persistence. |

## Decisions taken, 2026-08-12

| Decision | Effect |
|---|---|
| **Continuous scroll is an option, not a replacement** | Single page stays the default — it is the right model for drafting review. Four page-display modes sit together on the View tab, persisted per document. |
| **Whole-page rendering stays the default** | Now **measured** (`BENCHMARK.md`): six rapid zoom steps started six render generations and completed exactly one, at the destination — 1.9 s instead of ~11 s. The generation counter and settle debounce already solve what a tile cache would have been built to solve. pdfcer also uses 2.5× less memory than the competitor on the same file. Tiled progressive becomes an opt-in in a new **View ▸ Render** group. This corrects an earlier draft that called whole-page a weakness on architectural grounds without measuring it. |
| **The `Editing on` master toggle is removed** | *"Make it work the same way other programs do."* Selection and Delete are always live; tools arm and disarm. Supersedes defect D6. |
| **Format tab and properties panel both ship** | Panel first — it holds the full property set including editable X/Y/W/H, and the tab's contents are a subset. Context menus are the third surface. |
