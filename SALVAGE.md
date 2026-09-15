# Salvage inventory — what the old GUI has that this shell does not

This file answers two questions and nothing else: **what is still in the old
crate that this shell has not got**, and **where did a given old-crate
capability end up**. It is read by someone scheduling work against the first
question and by someone who greps for a capability, finds nothing, and needs to
know whether that silence means absent or means *filed somewhere else*.

**Source:** `D:\Dev\pdfce\crates\pdfce-gui\src\` — 21 files, **50,583 code
lines + 12,319 test lines** at `c0c67d3`.

**The source crate is not frozen.** It is still edited, which is why every
figure below names the revision it was measured at and why the figures are
given with the command that reproduces them:

```bash
OLD=/d/Dev/pdfce           # old-name-exempt: the salvage source is the pre-rename repo, at its own path, and it is correct exactly as written
SRC=crates/pdfce-gui/src   # old-name-exempt: the crate inside it still carries the old name
cd "$OLD"
git show <rev>:"$SRC/<file>.rs" \
  | awk '/^#\[cfg\(test\)\]/{print NR-1; exit}'    # code lines
git ls-tree --name-only <rev> "$SRC/" | grep -c '\.rs$'
```

The split is *lines before the first `#[cfg(test)]` at column 0*. The `^` is
load-bearing: `main.rs`, `object_provider.rs` and `dock.rs` each carry an
indented `#[cfg(test)]` on an inner module long before the file's own test
module, and an unanchored match cuts `main.rs` at 12,215 rather than 25,063 —
a wrong answer that looks like a right one.

Roughly **45 %** of the source comes across with little or no change. The
headline is `main.rs` at 25,063 code lines — half the crate — and what this
shell did with it was give its contents somewhere better to live.

---

## Summary

| Class | Code lines | Share | Meaning |
|---|---:|---:|---|
| **A — Lift** | 10,254 | 20 % | Comes across nearly as-is. Engine-facing, tested, correct. |
| **B — Lift and rework** | 12,836 | 25 % | Good bones, adapted to the new IA or new structure. |
| **C — Restructure** | 25,063 | 50 % | `main.rs`. Its *content* moves; its *shape* does not survive. |
| **D — Rebuild** | 2,430 | 5 % | Ribbon and dock — superseded by `RIBBON_IA.md`. |

Tests follow their subjects. The 12,319 test lines are salvaged with the code
they cover, and are a floor rather than a ceiling — see rule **R1** in the
agent charter.

---

## ★★ A status word in this file is a CLAIM, and it decays

Every **Status** cell below asserts something about `crates/pdfcer-gui` and
`crates/egui-shell`. Re-measure before quoting one. A wrong status here does
not stay here: this file is the survey `FEATURES.md`, `RESUME.md`,
`RIBBON_IA.md` and a `PLANNED` entry in `shell/manifest/` all quote, so one
stale cell travels by being copied rather than checked.

**The mechanism that makes a stale cell invisible, and it is not the obvious
one.** A row does not merely age — it **predicts an address**. The work then
lands at a different one, and a grep for the predicted path returns silence
that reads as confirmation. Fourteen rows in this file once read as
un-salvaged for exactly that reason: text editing was predicted at
`tools/text/` and is at `canvas/textedit/`; measure at `tools/measure/` and is
at `canvas/measure/`; vector editing at `tools/vector/` and is distributed
across `canvas/`; find at `app/find.rs` and is a top-level `find/`; batch at
`panels/batch/`, which does not exist and is not planned. Four more — theme,
ribbon, ribbon rendering and the dock — landed in the **other workspace
crate**, `crates/egui-shell/`, and are invisible to any grep confined to
`crates/pdfcer-gui/src/`.

So the measurement is:

- Grep for the **capability** — the type, the command id, the verb — and never
  for the path a row guessed.
- Sweep **both crates**. `crates/egui-shell/` holds everything that is not
  about PDF.
- Read the **call site**, not the declaration. A `pub` item with a unit test
  and no production caller reads exactly like a shipped feature.

---

## Class A — Lift nearly as-is

The parts of the old GUI that are **good**, several of them things no
competing product has. They come across with their doc comments and tests
intact.

| File | Code | Tests | Why it survives | Status |
|---|---:|---:|---|---|
| `print_flow.rs` | 1,877 | 168 | Three-tab print dialog with a zoomable live preview of real page content. Self-contained, nothing like it needs re-deriving. | ✅ **In `dialogs/print/`** — fifteen files, including a `spooler/` that talks to the device and `verdicts.rs` for what the printer refused and why. ⬜ **Imposition** is the one outstanding item: n-up, booklet and poster are `cli [x] · gui [ ]`, blocked on `pdfcer-print` sharing sheet composition so the preview and the spool path cannot disagree. It is one new tab, not a change to the three that exist; the dialog's own header argues the absence at the site. |
| `icons.rs` | 1,747 | 383 | SVG path data rasterized at physical pixel size rather than pre-baked PNGs. Mostly data. | ✅ **In `icons/`** — `svg/` for the path parser, `assets.rs`, `cache.rs`, `paint.rs`, `glyphs.rs`, and a `catalog/` whose `mapping.rs` carries 283 command→icon bindings with its own tests. Grown past the source by an `IconWeight` axis, so a selected toggle draws the same asset at a heavier stroke rather than at a different colour. |
| `measure_tool.rs` | 1,244 | 814 | Dimension **groups** with shared scale and drafting standard — better than the comparison product has. Taubin best-fit circle. Snapping. | ✅ **In `canvas/measure/`** — nine files. `MeasureKind` carries `Linear`, `Circular`, `Perimeter`, `PathLength`, `TwoLine` and `Scale` behind eight registered commands, and the two-line gesture is armed: `canvas/measure/mod.rs` offers each picked line to it and `scale.rs` authors through `author_from_two_lines`. The snap primitives in `canvas/snap.rs` are queried, not merely carried. ⬜ **Area and Angular** are the conspicuous absences for anyone doing takeoff; `shell/manifest/measure.rs` names both and says why the Quantity axis is the harder half. |
| `diag.rs` | 819 | 144 | The `PDFCER_DIAG` key=value channel. Off by default, one atomic load, never load-bearing. This is what makes the Delete-key and render analyses possible. | ✅ **In `diag.rs`.** The `PDFCER_DIAG_SCRIPT` harness grammar is deliberately left behind: `tools/ui-verify` was built without it, driving the real binary through `launch.rs` and `input.rs`. ⬜ **Page complexity** per `BENCHMARK.md` §"instrument before optimising" is still absent — `render-spawn` says what was started and no line says what it cost to draw. |
| `settings_panel.rs` | 1,088 | 129 | The spec-ambiguity settings model — each row states what the standard leaves open and how well-founded the default is. A genuine differentiator. | ✅ **In `dialogs/settings/` + `text/settings/`**, with five deliberate departures: a seventh group, **Measuring and dimensioning**, because the parallel tolerance sat under *Copying and extracting text* where nobody with the symptom would look; **Colour** expanded rather than Appearance; four guessed defaults that read as recommendations now admit it; and two engine facts the old window hid are disclosed. Heading contrast is fixed by `DEFECTS.md` **D11** rather than D2 — `.strong()` was the cause, and `tools/gates/check-strong-text.sh` is the gate. Thirteen groups draw, including Fonts, reached directly from Tools ▸ Font folders through `Draft::focused_on`. ⬜ The **Render** group is still to come, and its commands are **not** registered and inert: `shell::commands::reach::SCAFFOLDED` is empty and `shell/commands/reach/mod.rs` asserts `total == 0`, so an unbuilt render setting renders nothing anywhere. The group is a destination waiting for commands, not commands waiting for a destination. |
| `object_provider.rs` | 694 | 313 | Front-to-back page object decomposition. Feeds the Objects panel, which is the single strongest thing pdfcer has. | ✅ **In `panels/objects/provider/`.** ⬜ **Still single-page**: a provider is built for `view.page_index` and rebuilt on page change, and a query for any other index returns nothing. Continuous display landed without it — `viewer::display::PageDisplay` carries `Continuous`, `Facing` and `FacingContinuous`, and Read mode defaults to continuous — so the panel is the half that lags rather than the half that waits. |
| `object_summary.rs` | 534 | 276 | Per-object descriptions — type, text, font, colour, width, node count, winding. | ✅ **In `panels/objects/summary.rs`.** The old row-clipping question is **settled the other way**: `panels/objects/mod.rs` chooses `ScrollArea::vertical`, not `both`, because a horizontal axis would exist to scroll to content nothing puts there, and a bar with nothing beyond its viewport is a control that cannot do anything. Truncation is **disclosed** instead — `ObjectSummary::text_truncated` on the row, a census line on the panel — because a shortened list is otherwise indistinguishable from a complete one. Rows are virtualised through `show_rows`. |
| `viewer.rs` | 509 | 414 | Zoom ladder with provable reversibility, fit modes re-derived per frame, per-page raster ceiling accounting for `pixels_per_point`. Well tested. | ✅ **In `viewer/`, both open questions closed.** The anchor rule is decided once, in `canvas::zoom::anchor_point`, and all five zoom paths route through it — Ctrl+wheel anchors on the pointer, the discrete commands on the viewport centre, and `anchor_step` gates consumption so an unspent anchor cannot be spent on an unrelated layout. A page *range* is not something a view holds: `viewer::strip::Strip::page_at_view` derives which pages are on screen from the layout and the viewport, and `ViewState` keeps exactly one index meaning *the page the operator is looking at*. What it gained instead is `ViewState::display`, a fourth axis. |
| `render_worker.rs` | 472 | 116 | Generation counter + between-operator cancellation. **Measured**: six rapid zoom steps start six generations and complete one. Do not touch the design. | ✅ **In `render/worker/`.** `RenderKey` carries `page_index`, `raster_scale_bits`, `annotations`, `layers_generation` and a line-weights key, each landing in the same commit as the control that varies it. ⬜ The **thread pool** for thumbnails and adjacent-page prerender (`BENCHMARK.md`) is the one item still owed; `render/mod.rs` and `panels/pages/thumbnails.rs` both name it as absent. |
| `theme.rs` | 464 | 137 | Palette/preset model, chrome-vs-document colour separation enforced by CI. Sound design. | ✅ **In `crates/egui-shell/src/theme/`** — the reusable crate, not `pdfcer-gui`, because none of it is about PDF. D2 is fixed (`v.widgets.active.bg_fill = p.accent`) and the rendered-pair contrast test is a module of its own, `theme/contrast.rs`, swept by `tools/gates/check-theme-colors.sh`. |
| `redact_apply.rs` | 429 | 288 | Runtime-verified true-removal proof, forced full rewrite. **★ Still the only place the proof exists** — see below. | ✅ **In `redact/`** — `mod.rs`, `proof.rs`, and a `sealed.rs` that parses every `.rs` in the crate with `syn` and asserts each removal verb is called from exactly one file. **Four marking routes ship**: whole page and find-and-mark in `panels/redact/`, `edit.redact_selection` over a canvas selection, and `edit.offpage` for content outside the page boundary. ⬜ A **drag-out rectangle** on the canvas is the one route absent: it needs a `CanvasTool` variant, an `app::modes::capability` entry, a rung on the Escape ladder and an `Action` carrying page-space quads — a tool substrate armed over the one irreversible verb in the program. |
| `raster.rs` | 377 | 0 | Premultiplied alpha handled correctly; stale texture scaled `LINEAR` during settle. This is *why* zoom feels smooth. | ✅ **In `render/raster.rs`**, and the module around it is the growth: `render/` is thirteen files, with `settle.rs` and `strategy.rs` deciding when a frame is worth re-rastering, `region.rs` and `strip.rs` deciding what, and `ceiling.rs`, `halo.rs` and `hairline.rs` deciding how. |

**Subtotal: 10,254 code lines, 3,182 test lines.**

### ★ `redact_apply.rs` is load-bearing in a way the table understates

**The redaction true-removal proof is not in `pdfcer-core`.** It lives in the
shell. `pdfcer_core::redact::apply_redactions` returns
`Result<(Vec<u8>, RedactionReport), RedactError>` — a **report**, not a
verdict — and `RedactionVerdict` and `verify_redaction` appear nowhere in
`D:\Dev\pdfcer`. **A shell calling `apply_redactions` directly and writing the
bytes ships an unverified redaction and will not know**; `pdfcer`'s
`redact-apply` does exactly that and exits `SUCCESS` on a file it never
verified.

Two consequences:

1. **Deleting this proof, or reimplementing redaction against core's current
   surface, ships an unverified redaction.** The proof is unskippable here by
   four mechanisms, in order of how hard each is to defeat:
   `PreparedRedaction::bytes` is private with no accessor; the only way out is
   `write_to`, which re-runs the decoded-stream proof between the buffer and
   the syscall; the residual acknowledgement is a required argument, so a
   caller that forgets the checkbox gets a named refusal rather than a
   partially-redacted file; and `redact::sealed` asserts the call-site
   monopoly across the whole crate, failing closed on both a short sweep and a
   zero count.
2. **When core lands the verdict type, this becomes a deletion**, not a
   parallel implementation. Two proofs that can disagree is worse than one in
   the wrong crate. `redact::sealed`'s monopoly assertion is what will make
   that migration visible rather than gradual.

---

## Class B — Lift and rework

Good material whose hosting or structure changes.

| File | Code | Tests | Status |
|---|---:|---:|---|
| `ui_text.rs` | 8,241 | 3,913 | ✅ **In `text/`**, split across 117 modules with nothing in the crate over the 1,500-line R2 ceiling. The source's `shortcuts_reference()` defect (**D5**, six live bindings omitted) is not *fixed* but **made unrepresentable**: `dialogs/shortcuts.rs` holds no list at all and folds over the same `Keymap` that `app::keyboard::commands` resolves a keystroke against, so a binding that exists is listed because listing is a fold, and a chord whose command is unregistered is dropped with the count disclosed rather than shown greyed. There is no second copy to drift. |
| `canvas.rs` | 1,893 | 1,244 | ✅ **In `canvas/`**, 153 modules. Every selection-layer item shipped: `handles.rs` and `handledrag.rs` for handles, `menus.rs` against `shell/menus.rs`' four context menus for right-click, and `moving/` + `resizing.rs` + `annotdrag.rs` + `widgetdrag.rs` + `dimdrag.rs` for `/Rect` move-and-resize across every carrier. The escape ladder is `canvas/escape.rs`; tool dispatch is `canvas/tool/`. |
| `panels_structure.rs` | 1,807 | 0 | ✅ **Rehosted, not kept whole.** Bookmarks is `panels/bookmarks/` with add, edit, reorder, clipboard and a tree; Layers is `panels/layers.rs` + `panels/layers/{highlight,search}.rs`; Signatures is `panels/signatures.rs`; Fonts is `panels/fonts.rs` and moved to **File ▸ Document** as `file.fonts`, per the IA. The source's "three panels with no operator-reachable control" defect is closed structurally: `Panel::ALL` holds thirteen panels, each naming the command that shows it, and a test asserts every one of those commands is registered *and* referenced by the manifest. |
| `canvas_overlay.rs` | 749 | 0 | ✅ **In `canvas/overlay.rs`**, with `overlay/anchors.rs` for the node marks and `overlay/tests.rs` for the coverage the source shipped none of. It grew for handles and marquee as predicted; `canvas/overlays.rs` is the thin ordering layer above it. |
| `vector_edit_tool.rs` | 146 | 91 | ✅ **Distributed rather than kept whole**: `canvas/annotnodes/` for annotation vertices and ink, `canvas/vertexroute.rs` for the routing, `canvas/handledrag.rs` for Bézier handles, and `panels/objects/provider/node_rung.rs` for the rung the Objects panel exposes. ⬜ The source's measured hot spot at 6,681 anchors in one path object is **unmeasured here** — no check in `tools/ui-verify` times anchor handling, so the claim is neither carried nor refuted. |

**Subtotal: 12,836 code lines, 5,248 test lines.**

---

## Class C — Restructure: `main.rs`

**25,063 code lines + 3,601 test lines.** Half the crate.

The *architecture inside it is good* and is preserved:

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

What does not survive is the *file*. Where its contents are:

| Content | Approx. lines | Where it is |
|---|---:|---|
| Form filling, field authoring, FDF/XFDF/CSV | ~1,600 | `panels/forms/` — eleven files, including a `tab_order/` subtree the estimate did not anticipate |
| Text editing — runs, caret, formatting, reflow host | ~3,500 | `canvas/textedit/` — nineteen files. Caret, keystroke loop, disposition chooser, plan, proof, pin, place, glyph and face walls, and the reflow host. ⬜ Cross-run editing (**D4a**) still needs a multi-run engine request |
| Vector object editing, node/handle | ~1,200 | Distributed across `canvas/` — `vertexroute.rs`, `annotnodes/`, `handledrag.rs`. There is no `tools/vector/` module |
| Measure/dimension hosting | ~900 | `canvas/measure/` |
| Page ops, thumbnail rail, selection action bar | ~1,400 | `panels/pages/` |
| Batch — merge, split, insert, font folders | ~700 | **Not a panel, and `panels/batch/` is not planned.** `app/dispatch/batch.rs` arms a Tools ▸ Batch band; `tools.merge_files` is wired and `tools.split_files` is deliberately *unregistered* until a boundary chooser exists (R9); font folders became `dialogs/settings/fonts.rs` + `app/prefs/fonts.rs` |
| Redaction hosting | ~600 | `panels/redact/` |
| Object tree panel | ~800 | `panels/objects/` |
| Frame composition, panel order, dock hosting | ~900 | `app/frame.rs` |
| Keyboard map, action dispatch, status narration | ~1,800 | `app/keyboard.rs`, `app/actions/`, `app/status/` |
| App state, open/save/close, password prompt, parked docs | ~2,200 | `app/state.rs` + `app/state/` |
| Canvas hosting, hit-test dispatch, pan/zoom input | ~2,000 | `canvas/` |
| Dialogs — properties, print, export, reset, settings host | ~1,500 | `dialogs/` |
| Find | ~600 | `find/` — a top-level module, not `app/find.rs` |
| Remaining glue, helpers, types | ~5,300 | distributed |

**Honest framing:** the redistribution happened, and **the addresses moved**.
Text editing went to `canvas/textedit/`, not `tools/text/`; measure to
`canvas/measure/`; find to a top-level `find/`; vector editing into `canvas/`
rather than a module of its own; and batch to a ribbon band rather than a
panel. The genuinely *new* work — the selection model, context menus, the
properties panel and the ribbon — is additions rather than replacements, and
all four shipped.

---

## Class D — Rebuild

| File | Code | Tests | Status |
|---|---:|---:|---|
| `ribbon.rs` | 666 | 47 | ✅ **Rebuilt** across `crates/egui-shell/src/manifest/` and `pdfcer-gui/src/shell/`. The invariants survive — one source of truth, one owning tab per command (**P1**), no placeholders (**P3**) — and the content is the IA's: **eight tabs** (seven ordinary plus the contextual Format tab) and **thirty-seven groups**, asserted in `shell/manifest/mod.rs`, which holds the only copy of that number. The same manifest round-trips through `shell/ron/built_in.ron` with a test that the two agree. |
| `ribbon_ui.rs` | 1,187 | 0 | ✅ **Rebuilt** in `crates/egui-shell/src/ribbon/`, with overflow, collapsed bands, a QAT, a mode selector, sizing and rhythm. Group-caption enforcement is kept as the one closure that cannot emit a body without a caption: `band.rs`'s `captioned_group` emits the caption itself, after the body, with no branch that can skip it. |
| `dock.rs` | 577 | 241 | ✅ **Rebuilt** in `crates/egui-shell/src/dock/`, and **`egui_tiles` is not a dependency of either crate** — so the two-pane-max constraint and the hidden-overflow-tab workaround it was built around are both gone. Floating windows, a collapsible rail, overflow probing, tab menus and splitters are first-party. Layout **persists** through `app/persistence.rs`'s `DockLayout` + `PanelCatalog`, reconciled against the panel catalog on load so an id the dock never drew cannot come back from disk. The constraints-as-tests approach carried forward: `railhide_tests.rs`, `width_tests.rs`, `scroll_fade_repro.rs`. |

**Subtotal: 2,430 code lines, 288 test lines.**

---

## What neither crate has

Things the old GUI does not have and this shell has not built, listed so they
are not mistaken for salvage. **This list is deliberately short.** Everything
the migration survey originally named here has since shipped — context menus,
`/Rect` move-and-resize, the properties panel, recent files, session restore,
in-place save, hand tool, rulers, grid, guides, the go-to-page box, the
remaining markup kinds and revision clouds, page image export, the attachments
panel and canvas text selection. A bullet here is a claim and it decays the
same way a status cell does, so the register rather than this list is the
authority:

**`shell::manifest::PLANNED`** is the live list of every specified command
this shell has not built, each with the reason. It is enforced in both
directions — nothing listed there may be registered, and nothing listed there
may be referenced by the manifest — so an entry for a shipped command fails
the suite rather than decaying into a stale sentence. Read it, not this
paragraph.

The two worth naming here because they are asked about:

- **Autosave.** Absent. In-place save ships (`app/save.rs`), recent files
  ship (`app/recent.rs`) and dock-layout restore ships
  (`app/persistence.rs`); what is missing is the *asynchronous* save, and
  `file.revert` sits in `PLANNED` behind it because a revert is meaningless
  until there is a save point to revert to.
- **`tools.split_files`.** Unregistered until a boundary chooser, a
  destination directory and a name template exist — half a chooser is a
  control that splits somebody's drawing set the way pdfcer guessed. Batch
  merge ships.

---

## Salvage procedure

For each file still in Class A or B, in this order:

1. **Read it in full**, including the doc comments. They explain the defects
   that shaped it, and that reasoning is the most valuable thing being
   transferred.
2. **Copy it across with its tests and its documentation**, into its new
   module home.
3. **Apply the known fixes** for that file from `DEFECTS.md`, and add the
   regression test the defect implies.
4. **Split if it exceeds 1,500 lines** (R2) — find the seam, do not raise the
   limit.
5. **Assert it in `ui-verify`** before calling it done (R1). A green unit test
   is the floor.
6. **Record the landing address in this file's Status cell**, in one clause:
   where the capability now lives and what of it is still owed. Not when, and
   not by whom — a reader arrives at this file with a capability in hand and
   needs an address for it.

Never salvage a file by pasting a snippet out of it. The old GUI's value is
disproportionately in its doc comments; a snippet leaves those behind and the
next engineer re-derives a decision that was already made and already paid
for.
