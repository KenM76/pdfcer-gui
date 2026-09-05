# The four shell-layout proposals — feasibility, seam, cost and check plan

**Status:** analysis, 2026-09-04. Nothing built, nothing changed. This
document exists so a scheduling decision can be made against measured facts
rather than against a mockup's impression.

**Source of the proposals:** `D:\Dev\FeatureRequests\pdfcer-gui\REVIEW.md`
row **A7** (`:187-219`) and the Part B board sketch (`:455-485`), rendered as
`D:\Dev\FeatureRequests\pdfcer-gui\mockups\board-shell.html` and captured at
`D:\Dev\FeatureRequests\pdfcer-gui\screenshots\mock-board-shell.png`. The
reversal that promoted them from "the shipped manifest re-typed" to "better
than ours" is `REVIEW_TRIAGE.md` §6b (`:164-221`).

**Method, and why it is stated.** Every assertion below about the current
build was read from the Rust source in this session and carries a
`file:line`. Nothing was taken from `HANDOFF.md`, `RESUME.md`,
`FEATURES.md` or any other prose in this tree, because three of the four
proposals turn on a claim that a document in this tree gets wrong. Where a
finding contradicts a settled document, the contradiction is named and the
document is cited so the operator can rule on it rather than discover it.

---

## 0. Three corrections that have to come first

The four proposals cannot be scheduled honestly until three factual errors
are cleared, because two of the four change shape completely once they are.

### ★★★ 0.1 The rail is **not** S6, and S6 is not in `GUI_ROADMAP.md`

The dispatch that produced this document asked me to find the icon rail in
`GUI_ROADMAP.md` and say whether the proposal matches it. Two things are
wrong with that premise, and both matter.

**First, `GUI_ROADMAP.md` does not contain the icon rail at all.** That file
is organised by *Phase*, not by *Stage* (Phases 0–7 at `GUI_ROADMAP.md:72`,
`:99`, `:211`, `:233`, `:249`, `:276`, `:333`, `:346`), so it has no stage
table. Its three occurrences of "S6" — `GUI_ROADMAP.md:437` and `:446-450` —
are all about **per-viewport rendering and deep zoom**, an unrelated subject.
The only "rail" in that file is the *thumbnail* rail, and it appears there as
a complaint: `GUI_ROADMAP.md:245` — *"Thumbnail rail reflows to a grid and
narrows — it reserves ~390 px of a 1936 px window to show one thumbnail."*

The S6 icon-rail entry lives in **`PROJECT_PLAN.md:258`**, in the §4.2
capability table:

> `| Collapse to icon rail | **S6** | ~1 week | Mostly icons, tooltips and AccessKit names. Budget the harness-coordinate re-baseline. |`

**Second — and this is the load-bearing half — S6's rail is a different
feature that happens to share the name.** The settling quote is the
capability verdict at `MODES_AND_PANELS.md:345`:

> `| **h** | Collapse a dock to an icon rail | **Achievable with work** — not an `egui_tiles` feature, but it does not need to be: a narrow panel of icon buttons drawn *instead of* the tree, leaving tree state untouched. | ~1 week, mostly icons and tooltips |`

*"Drawn **instead of** the tree, leaving tree state untouched"* describes a
**collapsed-state substitute for an existing dock**. It is the recovery of an
Inkscape regression, and `MODES_AND_PANELS.md:213-215` says so in those
terms: *"per-dock collapse-to-icon existed in 1.0 and was removed in the 1.1
rewrite; the regression is still open five releases later."* Failure mode #5
at `MODES_AND_PANELS.md:241` is *"No per-dock collapse"*, and the design rule
it implies is *"Ship the icon rail; do not defer it indefinitely."*

The proposal is the opposite object: a **permanently visible navigation
surface** that replaces the left dock's arrangement rather than substituting
for it while it is hidden. VS Code's activity bar, not VS Code's collapsed
sidebar.

⇒ **The proposal diverges from S6.** It is not "already on the roadmap"; it
is a new feature the roadmap's cost estimate does not cover, standing beside
an S6 rail that is *also* still unbuilt (`MODES_AND_PANELS.md:295` —
*"(h) collapse to icon rail | S6 | achievable, not yet built"*). The two are
not alternatives: a build could reasonably want both, and the second is
cheaper once the first exists.

★ **And S6 as a stage is closed.** `PROJECT_PLAN.md:205` marks it
`✅ … DONE, and past the original scope`, while the rail assigned to it is
recorded unbuilt. The project is at S7/S8. **The rail is an unbuilt item
inside a closed stage — it has no live owner**, which is exactly how a
scheduled feature becomes a permanently deferred one.

★★ **The one prior mention of a *left* rail carries a gate the S6 entry does
not.** `OPERATOR_REQUESTS.md:433-437`, about an earlier external mockup:

> *"the genuinely new parts are a left icon rail (~1 week, and gated behind
> the R128 fit-zoom cache), a bundled typeface, a palette, and an 11 pt type
> floor."*

The gate is the adjacent roadmap row, `PROJECT_PLAN.md:259` — *"Fit-zoom
cache (R128) | S6 | own landing | Prerequisite for anything that makes the
canvas rect user-variable."* §1.7 below argues that this particular rail does
**not** actually trip R128, and says why the earlier note was right to be
cautious anyway.

### ★★★ 0.2 "Ours lists far less per row" is false. Ours lists **more**

`REVIEW_TRIAGE.md:206-209` says the mock's Objects rows *"carry "Text · "A1"
· SpaceGrotesk-Bold 8 pt" and "Path · stroked #D97706, 1.00 pt wide · 2
nodes". Ours lists far less per row."*

Read against the source, that is backwards. `text/panels/objects.rs:436-471`
composes an object row from, in fixed order:

| fragment | where | shown? |
|---|---|---|
| paint style — *"filled (even-odd) and stroked"*, *"stroked"*, *"paints nothing (a clip or discarded path)"* | `objects.rs:271-280`, pushed `:439-451` | ✅ |
| colour hex `#RRGGBB` | `objects.rs:317`, pushed `:442-444` | ✅ |
| line width, `", {width:.2} pt wide"` | `objects.rs:446` | ✅ |
| node count, `" · {nodes} node(s)"` | `objects.rs:449` | ✅ |
| quoted text preview, 32 chars then `…` | `objects.rs:356-367`, `ROW_TEXT_CHARS` at `:336` | ✅ |
| **font name and size**, `"{name} {size:.2} pt"` | `objects.rs:407-417`, pushed `:461-463` | ✅ |
| image pixels, `"{w} × {h} px"` | `objects.rs:468` | ✅ |
| a trailing short note — *"bounds from metrics"*, *"zero height"*, *"a whole nested drawing"* | `objects.rs:536-566`, appended `:505` | ✅ **the mock has no equivalent** |

A real shipped row, quoted from this module's own header at
`panels/objects/mod.rs:80-81`:

```
#1382  Path · filled (even-odd) and stroked #1A73E8, 0.50 pt wide · 6681 node(s) · zero height
```

Against the mock's:

```
#25  Path · filled (even-odd) and stroked #FAEEDC
```

**Every attribute the review names is already on our row, and we carry a
diagnostic note the mock does not.** The screenshot that produced the
impression is `24-edit-selected-object.png`, where the row reads
`#27 Text · "A1" · AAAAAA+SpaceGrotesk-Bold 1` — **clipped at the dock's
right edge mid-number**. The mock's rows fit because the mock's column is
wider and because it strips the `AAAAAA+` subset prefix; ours do not fit
because 320 pt is not enough.

★ **So the delta in proposal 2 is room and presentation, not content.** That
is a completely different piece of work, an order of magnitude cheaper, and
it changes the ranking in §5.

★★ The one genuine content gap, found while checking the above: a path that
both fills and strokes shows **only the fill colour**. `summary.rs:540-548`
resolves `visible_colour` as *fill if there is one, else stroke, else none*,
collapsing an engine pair (`PathObject::fill_color` / `stroke_color`, both
read at `summary.rs:442`) into `ObjectSummary.colour: Option<Rgb>`
(`summary.rs:311`). Surfacing both is a GUI-local change — the engine data is
already in hand at that call site — and it is **not** something the mockup
asks for. The mock's row `#25` collapses them the same way we do.

### 0.3 Rulers on two edges with a corner box are **already built**

Proposal 4 is not a proposal. `crates/pdfcer-gui/src/canvas/rulers.rs` is
1,413 lines and ships a top gutter, a left gutter and an explicit corner
rect: `Gutters { outer, content, top, left, corner }` at `rulers.rs:312-329`,
constructed in `reserve` at `rulers.rs:376-407`, drawn by `draw` at
`rulers.rs:900`. §4 below gives the full delta, which is approximately zero
code and one settings question.

---

## 1. Proposal 1 — the left icon+word tab rail

### 1.1 What we ship today

**The Edit mode left dock is two vertically stacked tab groups, in one
column, 280 pt wide.** From `app/modes/defaults.rs:412-421`:

```rust
left: vec![
    vec![pages(), Panel::Bookmarks.command_id()],
    vec![
        Panel::Layers.command_id(),
        Panel::Signatures.command_id(),
        Panel::Fonts.command_id(),
    ],
],
```

with `left_width: NAVIGATOR_WIDTH` (`defaults.rs:478`), and
`const NAVIGATOR_WIDTH: f32 = 280.0` at `defaults.rs:253`. Read and Review
get one stack of two (`defaults.rs:329`, `:365`).

Consequences, each one measurable:

- **At most two of the five left panels are ever visible at once**, because a
  tabbed stack draws only its active tab. The dock's own model says so:
  `Stack { tabs, active, share }` at `model.rs:197-215`.
- **The bottom half of the left dock is permanently reserved for
  Layers / Signatures / Fonts**, which are empty on nearly every document —
  the review's observation at `REVIEW.md:197-198`, and visible in
  `24-edit-selected-object.png` as the pane reading *"This document has no
  layers."* under a three-tab bar.
- The two stacks share the column's height through a draggable splitter
  (`dock/mod.rs:933-936`) with a floor of `MIN_STACK_HEIGHT = 80.0`
  (`dock/plan.rs:104`). The operator can drag the lower stack down to 80 pt,
  but it never goes away, and the layout does not remember that they wanted
  it gone in any structural sense — only as a share.
- Every one of the five panels already has a command id, so **nothing new
  needs registering**: `Panel::command_id` at `panels/mod.rs:361-473` maps
  `Bookmarks → view.panel_bookmarks`, `Layers → view.panel_layers`,
  `Signatures → view.panel_signatures`, `Pages → view.panel_pages`, and
  — note this one — **`Fonts → file.fonts`** (`panels/mod.rs:376`).

**There is a collapsed-side rail already, and it is the wrong rail.**
`dock/collapse.rs:106` draws `draw_collapsed_rail`, 16 pt wide
(`RAIL_WIDTH_PTS` at `collapse.rs:58`), carrying one chevron and no panel
identity at all. It exists to answer *"how do I get the side back"*
(`collapse.rs:81-91`), not *"which panel do I want"*. Its infrastructure is
nonetheless the closest thing in the tree to what the proposal needs: a
narrow `Panel::left(...).exact_size(...)` drawn outboard of the dock side,
raising intents rather than mutating (`collapse.rs:38-43`).

### 1.2 What the proposal changes

A vertical rail at the window's leading edge, ~50 pt wide, one entry per
panel — icon above a word — for **Pages / Marks / Layers / Sigs / Fonts**,
with a `Hide` control at its foot. The dock beside it shows **one** panel at
full height. From the render: the rail's active entry is tinted, the panel
beside it carries its own header (*"Pages · 1 page"*) with a detach and a
close glyph, and no tab bar appears anywhere on the left.

Three things change, and they are worth separating because they have
different costs:

1. **Panel selection moves from a horizontal tab bar to a vertical rail.**
2. **The left dock stops sub-dividing vertically.** One panel, full height.
3. **Every panel is one click away** — no `⏷ 3 more` overflow, no second
   stack to look in.

★ **Point 3 is the one that cannot be bought any other way, and it is the
real argument for the feature.** The obvious cheaper alternative — A7's own
suggestion at `REVIEW.md:213-215`, *"Layers/Signatures/Fonts collapse to a
tab strip under Pages/Bookmarks (one dock, five tabs)"* — is a **one-line
change** to `defaults.rs:412-421` and is now legal, because the two-pane cap
was retired (`MODES_AND_PANELS.md:317` — *"the cap is retired (nine panels in
one stack, tested)"*). But five horizontal tabs do not fit in 280 pt: the
right dock at 320 pt shows three of six and pushes three into `⏷ 3 more`
(`76-panel-overflow-crop.png`). So the one-line version trades a permanent
half-height loss for an overflow menu.

**A vertical rail has no such limit.** Five entries at ~44 pt each is 220 pt
of a ~700 pt dock height. **The rail is the only arrangement in which all
five panels are simultaneously visible as affordances.** That is the sentence
the scheduling decision should turn on.

### 1.3 Feasibility against egui 0.35 / egui_tiles 0.16

**Achievable, and nothing in `MODES_AND_PANELS.md` is contradicted.**

`MODES_AND_PANELS.md:345` already rules the mechanism achievable — *"a narrow
panel of icon buttons drawn instead of the tree"* — and the version proposed
here is strictly easier than that one, because it does not have to preserve
and restore a hidden tree. It is an ordinary `egui::Panel::left(…)` with
`exact_size`, exactly as `collapse::draw_collapsed_rail` already does at
`collapse.rs:113-120`.

Specifics checked against the vendored crates rather than assumed:

- **No rotated text is needed.** The mock's words run horizontally under
  their icons; egui 0.35 has no vertical text API and does not need one here.
- **`egui_tiles` 0.16 is irrelevant.** The dock was built without it
  (`MODES_AND_PANELS.md:274-283`), so its lack of a rail concept costs
  nothing. Nothing in this proposal argues for adopting it.
- **`exact_size` is available and is the required API.** `dock/mod.rs:541-550`
  records why it is the only one that closes the R128 loop, quoting the RAG
  entry: *"Only `exact_size` closes it."* A rail with a constant width is
  therefore R128-neutral by construction — see §1.5.
- **The three-phase frame survives.** `Dock::show` snapshots, draws recording
  `Intent`s, then applies once (`dock/mod.rs:452-521`). A rail click is one
  more `Intent` variant beside `DragColumns`, `EqualizeColumns`,
  `DragStacks`, `EqualizeStacks`, `DragSide` (`dock/ctx.rs:95-136`). Nothing
  about the architecture resists it.

### 1.4 Shell or application — where the seam falls

**The rail belongs in `egui-shell`. Its icons and its words do not.** The
seam is already precedented, exactly, by the ribbon.

R7 is enforced by `tools/gates/check-shell-purity.sh`, which fails on any
`pdfcer-*` dependency key in `crates/egui-shell/Cargo.toml` (check 1,
`:98-115`) and on any non-comment mention of `pdfcer_core|render|print` in
shell source (check 2, `:126-141`). Its instruction on how to resolve exactly
this kind of pressure is at `:150-153`:

> *"If the shell needs something the application knows, INVERT IT: the shell
> declares a trait or a manifest type, and pdfcer-gui supplies the value. …
> The correct fix is never an import; it is a seam."*

**Three shell changes, all inversions, none of which teaches the shell what a
PDF is:**

**(a) `PanelInfo` gains an icon key.** Today it is deliberately three strings
and no more — `model.rs:725-728`:

> *"The shell needs three strings to draw a tab: a label, a tooltip, and the
> id it already has. It needs nothing else, and asking for nothing else is
> what keeps `PanelId` opaque."*

That sentence is a good rule and it is about to acquire a fourth member. The
addition is `pub icon: Option<String>` beside `label` (`model.rs:731`) and
`tooltip` (`model.rs:745`), with a `with_icon` builder mirroring
`with_tooltip` (`model.rs:761-765`). ★ It is a **key**, not an image, for the
same reason `Command::icon` is (`commands/mod.rs:232-240`): *"icon rendering
is the application's — an icon set is a licensing and rasterization decision.
… the shell knows only that a control has an icon and which one."*

**(b) `IconPainter` lifts out of `ribbon`.** It is currently
`ribbon::ctx::IconPainter` at `ribbon/ctx.rs:100`, with `IconRequest` at
`:52-99`. Nothing about either is ribbon-specific — `IconRequest` carries
`key`, `rect`, `tint`, `enabled`, `selected`, all of which a rail entry needs
verbatim, `selected` included (`ribbon/ctx.rs:83-95` argues for it precisely
so an icon set can render a heavier weight for *"a panel that is open"*, which
is the rail's own state). The move is mechanical: a shared module with
re-exports so `ribbon` keeps its current path.

★ The `Painter`-not-`&mut Ui` decision at `ribbon/ctx.rs:26-35` carries across
unchanged and is load-bearing: *"A `Painter` can draw and cannot allocate, so
the seam is safe by type rather than by instruction."* A rail entry is a
button with a reserved square slot; the same argument applies verbatim.

**(c) `SideLayout` gains a rail mode, and `Dock` gains
`.with_icons(&mut painter)`.** `SideLayout` today is
`{ columns, width_pts, visible }` (`model.rs:313-341`). The minimal honest
addition is a mode discriminant — a side is either *columns* or *rail* — so
that the persisted layout says which, and `layout/` versions it. **This is
the only part of the work that is genuinely new schema**, and it is the part
to get right, because a saved layout that cannot express the arrangement is
the difference between a change that fits the manifest and one that does not.

★★ **This matters more than it looks.** `SHELL_FRAMEWORK.md:21-24` is the
whole architecture: *"The shell is data. Tabs, groups, commands, panels,
layouts, modes and key bindings are a serializable document."* A rail that
were hard-coded in `pdfcer-gui` would be a layout the manifest cannot
describe, and §5's customization table (`SHELL_FRAMEWORK.md:146-157`) would
silently acquire an exception. Expressed as a `SideLayout` mode, the rail is
saveable, resettable, per-mode, and part of a named workspace for free —
`app/modes/mod.rs:355` `record_layout` already persists whatever the model
holds.

**What stays in `pdfcer-gui`:** the icon glyphs (`icons/**`), the five
`with_icon` keys registered alongside the existing `PanelInfo`s, the words
themselves under R1's catalog rule (`crate::text`, enforced by
`tools/gates/check-ui-strings.sh`), and the decision about which panels the
rail lists per mode — which is `app/modes/defaults.rs`'s existing job.

★ **A rail entry is not a ribbon tab, so P1 is not implicated.**
`SHELL_FRAMEWORK.md:158-163`: *"a command may appear on exactly one tab; the
QAT and status bar may mirror it."* A rail mirrors, like the QAT. This is
worth stating because `REVIEW_TRIAGE.md:85` records the mock's ribbon being
refused by `no_command_appears_twice_on_the_tabs` for putting Fonts and
Comments on two tabs each — **that refusal does not extend to the rail**, and
conflating the two would kill a legal feature on an illegal neighbour's
record.

### 1.5 Cost

| item | where | size |
|---|---|---|
| `PanelInfo.icon` + builder + tests | `egui-shell/src/dock/model.rs` | ~40 lines |
| lift `IconRequest`/`IconPainter` out of `ribbon` | `egui-shell/src/ribbon/ctx.rs` → new module, + re-exports | ~60 lines moved, 2 files touched |
| `SideLayout` rail mode + normalize + serialization + schema version | `dock/model.rs`, `layout/mod.rs` | ~150 lines + migration test |
| rail rendering, hit targets, selected state, `Hide` foot | new `dock/rail.rs` | ~250 lines |
| `Intent::ActivateFromRail`, apply arm | `dock/ctx.rs`, `dock/mod.rs` | ~30 lines |
| `Dock::with_icons` builder + plumb through `Ctx` | `dock/mod.rs`, `dock/ctx.rs` | ~30 lines |
| 5 panel icon keys + 5 glyphs + rationale comments | `pdfcer-gui/src/icons/**`, panel registration | ~120 lines, 5 SVGs |
| left-dock defaults switched to rail per mode | `app/modes/defaults.rs` | ~10 lines |
| harness coordinate re-baseline | `tools/ui-verify/**` | see §1.7 |

**~700 lines across ~10 files, plus five glyphs, plus the re-baseline.** The
`PROJECT_PLAN.md:258` figure of ~1 week is about right for the *build* and is
optimistic about the re-baseline, which that entry does at least name.

★ **Two of those glyphs may already exist.** `REVIEW_TRIAGE.md:148-163` and
`GLYPH_ADOPTION.md` record 36 glyphs adopted from the review's own sheet on
one rule — *"a glyph is adopted only when a command or role in this build
would use it today."* A rail is exactly such a role, and it would promote
some of the 26 deferred glyphs. Check `GLYPH_ADOPTION.md` before drawing
anything.

### 1.6 What could go wrong

**★★★ Hazard 1 — the rail ships unreachable, with every gate green. This is
the recorded history, and it is this exact feature.**
`D:\dev\rag\egui\enum_keyed_panel_with_no_production_setter_compiles_and_runs_cleanly.md`
describes the 2026-08-10 incident verbatim:

> *"Three GUI panels (Bookmarks, Layers, Signatures) each shipped with a
> `PaneSubject` variant, a full panel body, **a rail entry referencing an
> `Action`**, and a `diag` step for the observation harness. The panels drew
> correctly in every screenshot and every driven trace, for the panels'
> entire shipped lifetime — because the harness's own step handler set
> `self.pane_subject` directly. No production code path (rail click, menu,
> keyboard shortcut) ever set it."*

Three panels. A rail. The same three panels this proposal puts on a rail.
The RAG entry's explanation of why nothing catches it applies unchanged:
*"'the panel draws when the enum equals X' and 'the enum equals X' are two
independent facts joined only by runtime value equality."*

★ **And the current dock makes this worse, not better.** The dock's rect
sink is the **unguarded** channel: `app/surfaces.rs:245-247` passes
`crate::diag::ui_rect` to `reporting_rects_to`. The guarded sibling
`diag::ui_rect_visible` (`diag.rs:408`) intersects with the clip rect and
publishes nothing when the result is empty; `diag::ui_rect` (`diag.rs:546`)
does not. **So today every dock rect proves layout and not visibility**, and
a driven check that finds a rail button's rect would prove nothing at all.
This is the single most important line in this document for whoever builds
the rail.

**Hazard 2 — the harness re-baseline, and its false-defect signature.**
`D:\dev\rag\egui\scripted_click_coordinates_go_stale_when_a_dock_width_changes.md`:

> *"Widen a left dock by 100 pt and every previously-correct click coordinate
> now lands 100 px to the left of its target — with no error, no warning, and
> no failing assertion. … The symptom is identical to a coordinate-space
> conversion bug."*

The rail changes the left edge of the canvas on **every** mode, so this is
certain, not probable. The mitigation is already documented as prerequisite
#1 at `MODES_AND_PANELS.md:381-386` — *"Harness coordinates must become
document-space"* — and its sequel is worse:
`a_bulk_conversion_that_misses_one_call_site_stays_green_until_the_layout_moves.md`
records twelve checks converted to `frame_of` and a thirteenth missed, which
**passed for six days** because the stray click happened to hit anyway, then
failed on an unrelated layout change and **accused the application**.

⇒ **Before touching the rail, grep for every absolute coordinate in
`tools/ui-verify/src/checks/` and convert them all in one pass.** A partial
conversion is worse than none, because a wrong aim that happens to hit is a
green result reporting nothing.

**Hazard 3 — R128, and why I think this one is a false alarm.**
`OPERATOR_REQUESTS.md:435-437` gates a left rail behind the fit-zoom cache.
The R128 mechanism (`bottom_panel_height_change_retriggers_fit_to_viewport_zoom.md`,
measured at 230 % → 224 % → 215 %) needs a **content-driven** size adjacent
to a per-frame fit computation. A rail whose width is a constant — five icons
at a fixed `icon_pts`, words clipped rather than measured, drawn with
`exact_size` exactly as `collapse.rs:116-120` already does *"for the same
reason the side itself uses it"* — is content-independent, and
`rulers.rs:155-181` demonstrates the same discipline surviving in production
(`THICKNESS_PTS` constant at `rulers.rs:237`, asserted by
`the_gutters_are_a_constant_bite_out_of_the_viewport` at `rulers.rs:1068`).

★ **So the R128 gate should be discharged by argument and a test, not by
building the fit-zoom cache first** — provided the rail's width is a
constant and a test asserts it. The gate becomes real the moment anyone makes
the rail width follow its widest word. Do not.

**Hazard 4 — the panel behind the rail publishes nothing.**
`a_docked_pane_behind_another_tab_publishes_nothing_and_that_reads_as_a_silent_panel.md`.
With a rail, **four of five panels are always in that state**, where today it
is three of five. Any check written against a left panel must select it
through the rail first, and the failure message must distinguish *"not
selected"* from *"selected and not drawn"*.

**Hazard 5 — Inkscape failure mode #8, re-entering by the back door.**
`MODES_AND_PANELS.md:244`: *"Tab overflow has no escape — past ~6 tabs the
overflow button itself gets hidden."* The rail dodges it at five entries and
walks into it at fifteen. If the rail ever lists every registered panel
(`Panel::ALL` is 13 today, `panels/mod.rs:324`), it needs its own overflow
with reserved space, and reserved space at 44 pt an entry is expensive.
**Keep the rail per-mode and short.**

### 1.7 The driven check, and what would make it vacuous

**The check: `every_rail_entry_selects_its_panel`.** Modelled on
`tools/ui-verify/src/checks/preset_group_reachable.rs`, which is the only
reachability check in the 154-check suite and whose header (`:43-55`) is
about this exact hazard class.

Shape, per entry, for all five:

1. Launch with a fixture document and enter Edit mode.
2. `driving::declared(&trace, ui_rect, "dock-rail-entry-<panel-id>")`
   (`driving.rs:99`) → a rect. **Fail** if absent — the entry is not on
   screen.
3. `driver.click_at(frame.declared_center(rect))` (`input.rs:186`,
   `coords.rs:626`).
4. `session.settle`, re-trace, assert the **panel body's own region** now
   publishes: e.g. `panels/properties/mod.rs:534`'s
   `ui_rect_visible(REGION_OBJECT, …)` pattern, one per panel.
5. Assert the previously-active panel's region is **gone** —
   `driving::declared` returns `None` for a retired region, because the app
   emits `ui-rect-gone` (`driving.rs:110-127`).
6. Sample pixels at the rail entry before and after with
   `driving::fill_of` / `driving::delta` (`driving.rs:783`, `:794`) to prove
   the selected tint actually changed.

**Four ways this check would be vacuous, and each has a precedent:**

- ★★★ **If the rail publishes through `diag::ui_rect` instead of
  `diag::ui_rect_visible`.** Then step 2 succeeds for an entry laid out
  below the fold, and the whole check passes on an unreachable rail. **This
  is the default behaviour of the dock's current sink**
  (`app/surfaces.rs:246`) — the check is vacuous unless the rail is
  deliberately wired to the guarded channel and the sink is changed or
  bypassed for rail regions.
- **If step 4 asserts the rail entry's own selected state rather than the
  panel body's region.** The rail could then correctly highlight an entry
  whose panel never draws — exactly the Inkscape failure mode #11 at
  `MODES_AND_PANELS.md:247`, *"Focus-existing shows stale content"*.
- **If the harness sets the active panel directly rather than clicking.**
  That is the 2026-08-10 defect reproduced as its own test. The click must go
  through `Driver::click_at` against a rect the app published.
- **If the check runs at a window width where the rail is the only thing that
  fits.** `a_ribbon_group_that_collapses_at_the_default_window_width_makes_a_driven_check_SKIP_forever.md`
  is the same shape: pin the window size, and make a SKIP a reportable
  outcome rather than a silent pass.

⇒ **The check is worth roughly a day and it is not optional.** Without it the
rail is the 2026-08-10 defect with better art.

---

## 2. Proposal 2 — Objects over Properties as a real master–detail

### 2.1 What we ship today

**We already ship a master–detail, and it is already a vertical pair with a
draggable split.** `app/modes/defaults.rs:422-476`, Edit's right side:

```rust
right: vec![
    vec![Panel::Tool.command_id()],
    vec![Panel::Objects.command_id()],
    vec![
        Panel::Properties.command_id(),
        comments(),
        Panel::Forms.command_id(),
        Panel::Redact.command_id(),
        Panel::DimensionGroups.command_id(),
        Panel::Attachments.command_id(),
    ],
],
```

Three stacks in one column, `right_width: INSPECTOR_WIDTH` (`defaults.rs:479`),
`const INSPECTOR_WIDTH: f32 = 320.0` (`defaults.rs:260`). Splitters between
stacks with `MIN_STACK_HEIGHT = 80.0` (`plan.rs:104`, dragged at
`dock/mod.rs:933-936`).

**The selection linkage is real, and it goes through the canvas selection
rather than a panel-local variable.** Since 2026-08-26:

- Objects reads the highlight from the selection:
  `panels/objects/mod.rs:330` — `doc.selection.object_indices_on(page_index).first().copied()`,
  applied at `:385` via `selectable_label`.
- A row click raises an action, it does not mutate: `objects/mod.rs:482-493`,
  `Action::SelectObject { page, object }`, with a click on the already-selected
  row deselecting (`:483-488`).
- Properties reads the **same** selection: `panels/properties/mod.rs:450-454`,
  then describes it at `:471-476` through the same
  `summary::describe_object`.
- The retired panel-local focus is recorded at `panels/mod.rs:854`.

★ So *"rows above, the picked row's properties below"* (`REVIEW.md:212`) is a
description of what is on screen in `24-edit-selected-object.png`.

**What is actually wrong is room, and it is measurable.** In that screenshot
the right dock's ~800 pt of height is divided roughly in thirds: Tool ≈ 265,
Objects ≈ 253, the Properties stack ≈ 260. Objects therefore shows **five
rows** of a 28-object page, and Properties shows Position-and-size and
nothing else before the fold. In the mock, with the tool strip at 28 px,
Objects gets ~16 rows and Properties gets its full form down to *Clip*.

**And row content is not the problem — see §0.2.** Rows are also deliberately
*not* clipped: `ScrollArea::both()` at `objects/mod.rs:340`, intrinsic width
measured per row at `:350-353`, `ui.set_width(width)` at `:361`. The review's
ellipsis request (`REVIEW.md:213-214`) is already ruled against at
`REVIEW_TRIAGE.md:101`, citing `SALVAGE.md:44`, and the tooltip it asks for
already exists twice — always `t::objects_dock_row_tooltip()` at
`objects/mod.rs:386`, plus the **full row text** on hover when the row
measured wider than the pane (`objects/mod.rs:368`, `:387-389`).

### 2.2 What the proposal changes, once §0.2 is subtracted

Almost nothing that is not already built. What remains:

1. **More height for both panels** — which is proposal 3's subject, not
   this one.
2. **A wider dock.** A7's own last bullet (`REVIEW.md:216-217`): *"Default
   right-dock width in Edit mode: 360 px … and let the dock remember the
   width per mode."* The remembering is **already built** —
   `SideLayout.width_pts` (`model.rs:333`) is per side, per mode, persisted
   by `app/modes/mod.rs:355` `record_layout`. The default is one constant at
   `defaults.rs:260`.
3. **Row presentation:** an icon column and an index column, so the prose
   fragment starts at a consistent x; and stripping the `AAAAAA+` subset
   prefix from a font name, which today comes straight from
   `base_font`/`resource` (`text/panels/objects.rs:408-412`).
4. **A compressed panel header.** Ours draws two prose lines above the list —
   `t::objects_dock_intro()` and `t::objects_dock_summary(census)`
   (`objects/mod.rs:289-290`), rendering as *"Everything drawn on this page,
   front-most first — the object painted last is the first row."* plus
   *"28 object(s) on this page — 13 path(s), 15 text object(s)."* The mock
   compresses both into one header row: *"Objects · 28 on this page · front
   first"*.

★ **Item 4 is an amendment, not a fix.** `REVIEW_TRIAGE.md:99` records the
rule and its stated failure mode: *"Every disclosure above the list, without
exception … a caveat below a list arrives after the operator has already
drawn a conclusion."* The mock does not move the disclosure below the list —
it *shortens* it into the header — so this is compatible in spirit. And the
same triage row concedes the reviewer's point that *"no length policy
exists, and the project does shorten copy on screenshot evidence."* This one
is the operator's to rule on, and it is cheap either way.

### 2.3 Feasibility, seam and cost

**Trivially feasible; nothing new in egui.** `MODES_AND_PANELS.md` has no
verdict to contradict, because the capability (b) *"vertical stacking within
a column, resizable"* is listed as **already shipped, cost zero**
(`MODES_AND_PANELS.md:339`).

**Seam: entirely `pdfcer-gui`. No shell change at all.** The dock already
provides the master–detail container; what changes is a constant, a row
layout and two strings. ★ This is the cleanest R7 answer of the four: *the
content of an object row is precisely the thing `egui-shell` must never
learn*, and none of this work goes near the shell.

| item | where | size |
|---|---|---|
| `INSPECTOR_WIDTH` 320 → 360 | `app/modes/defaults.rs:260` | 1 line + the width test |
| icon + index columns on a row | `panels/objects/mod.rs:361-412` | ~60 lines, one glyph per `ObjectKind` (5) |
| strip the subset prefix | `text/panels/objects.rs:407-417` | ~10 lines + a test for `AAAAAA+Foo` → `Foo` |
| header compression | `objects/mod.rs:289-290`, `text/panels/objects.rs` | ~20 lines, R1 catalog entries |
| optional: separate stroke colour | `panels/objects/summary.rs:311`, `:540-548`, `text/panels/objects.rs:439-451` | ~40 lines |

**~130 lines in 4 files, one day.** The width change alone is under an hour
and delivers most of the legibility win, because §0.2's clipping is a width
problem.

### 2.4 What could go wrong

**Hazard 1 — widening the dock is a harness re-baseline too.** Same RAG entry
as §1.6 hazard 2: the *canvas* rect moves when the right dock widens, so
every canvas-relative click coordinate shifts. **A one-line constant change
is a suite-wide event.** This is the single most under-estimated line in this
document.

**Hazard 2 — the subset-prefix strip is a correctness claim, not a cosmetic
one.** `AAAAAA+SpaceGrotesk-Bold` and `SpaceGrotesk-Bold` are different
things: the tag says *subset*, and two different subsets of one face get
different tags. Stripping it makes two rows read identically when they name
different font objects. **Strip it in the row and keep it in Properties**, or
do not strip it. Do not strip it in both.

**Hazard 3 — `describe_object` is called per visible row, on demand**
(`objects/mod.rs:558`, no cache). Adding fields is free; adding a *scan* is
not. The stroke-colour addition reads a field already loaded at
`summary.rs:442` and is safe. Anything that walks the content stream is not.

### 2.5 The driven check, and what would make it vacuous

**★★ There is no oracle for this today, and that is the finding.**

The harness has four relevant facilities. Three exist:
locate-by-name (`driving::declared`, `driving.rs:99`), click
(`Driver::click_at`, `input.rs:186`), and pixel sampling
(`capture::window` `capture.rs:65`, `Image::pixel` `image.rs:112`,
`driving::fill_of` `driving.rs:783`). **The fourth — reading the text a
panel renders — does not exist.** There is no AccessKit reader, no OCR, no
text extraction from a screenshot; the only route is a trace line the
application publishes on purpose, read with `TraceLine::get`
(`trace.rs:83`).

And **the Objects panel publishes no `ui_rect` at all.** Its only diagnostic
is one aggregate line at `objects/mod.rs:301-303`:

```rust
format!("objects-panel page={page_index} objects={object_count} rows={total_rows}")
```

★ `rows=` is a **layout** count, produced before the draw, and
`a_per_item_diagnostic_line_is_not_a_list_of_what_you_can_click.md` is the
recorded lesson about exactly this: *"A panel emits one diagnostic line per
item during LAYOUT, including items scrolled out and items past the bottom of
the panel, so it answers what was computed, never what is on screen."*

**So the check has to be built alongside the feature:**

`the_objects_row_fits_the_inspector_at_its_default_width` —

1. Open a fixture with known objects, enter Edit.
2. Have the panel publish, per **visible** row, through
   `diag::ui_rect_visible(…, ui.clip_rect())`: a region
   `objects-row-<index>`, and separately the row's **measured intrinsic
   width** (already computed at `objects/mod.rs:350-353`) and the pane's
   viewport width (already computed as `overflows` at `:368`) into a trace
   line.
3. Assert: on the fixture, **zero rows report `measured > viewport`**.
4. Independently, capture and assert the rightmost 8 px column of the
   Objects pane is background — proving nothing is clipped, with a pixel
   oracle rather than the app's own arithmetic.

**Three ways it would be vacuous:**

- **If it asserts `rows=` from the existing aggregate line.** That counts
  laid-out rows and would pass on a pane showing one.
- **If it asserts only the app's own `overflows` flag.** That is the
  application marking its own homework; the same computation that decides
  whether to attach the recovery tooltip decides whether the check passes.
  Step 4's pixel assertion is what makes it non-circular — the same
  two-channel discipline `read_mode_chrome.rs:50-55` states: *"the rect is
  exact and cheap and would be satisfied by a build that moved the canvas
  without repainting anything; the pixels cannot be faked by an arithmetic
  error."*
- **If the fixture's font names are short.** Pin a fixture whose text objects
  carry subset-tagged names — `a1-titleblock.pdf` does — and assert the check
  fails on today's build before it passes on tomorrow's.

---

## 3. Proposal 3 — the one-line tool strip

> ### ★★★ OVERRULED AND BUILT, 2026-09-04 — read this before §3.4's verdict
>
> `OPERATOR_REQUESTS.md` **O123**. The operator read this section and reversed
> it, and the reversal is right in a way this analysis could not see from
> inside its own frame.
>
> §3.4 said *do not build the strip*, on three grounds. Two of them stand and
> were answered rather than argued away; the third was a misreading of what the
> proposal was for.
>
> | §3.4's ground | what happened |
> |---|---|
> | *"it deletes the armed options block"* | **It does not.** Every control moved to `panels::properties::tool` — the pen's face, size and colour swatch, the measure pick list, the three scale switches. His sentence is the whole argument: *"I never understood why there is a tool dock when everything can be in object and properties."* They were never the tool's; they are properties of what is about to be drawn. |
> | *"it orphans Block C into a surface R128 forbids"* | **It does not go to a row.** The disclosure moved to `panels::properties::disclose` — a dock panel, whose width is the dock's, decided before the body draws. The status bar keeps its existing **elided** copy, unchanged. See that module's header for the check that the bar could not have been the home. |
> | *"it reverses a recorded placement decision"* | **True, and it is his to reverse.** The tool LIST is gone. That list was the answer to a discoverability defect and its removal is a real subtraction; it is recorded in `app/toolstatus.rs`'s header, not glossed. |
>
> ⇒ **The analysis was right about the cost and wrong about the remedy**,
> because it took the panel's contents as fixed and asked only whether a 28 pt
> strip could hold them. The correct question was whether the panel should have
> been holding them.
>
> ★ Two details from §3 survived intact and were built as written: the strip
> **cannot be a dock stack** (§3.2's `MIN_STACK_HEIGHT` arithmetic), so it is a
> shell-side side banner — `egui_shell::dock::banner`,
> `Dock::with_side_banner`; and §3.3's catch that *"`put down` beside `Select`
> would be inert"* is honoured — the button is absent in the resting state.
>
> ★★ §3.5's warning was also honoured: *"any check written against the existing
> Tool panel must not be deleted along with it."*
> `the_first_frame_names_the_tools` was **rewritten**, not removed, into
> `the_first_frame_names_the_armed_tool`, and it gained the pixel assertion
> §3.5 asked for — because a constant-height banner publishes a rectangle
> whether or not it painted anything.


### 3.1 What we ship today

**The Tool panel is not a stack of tool buttons.** `panels/tool/mod.rs` is
338 lines with a 135-line header, over `idle.rs` (406) and `armed.rs` (812).
`body` at `mod.rs:196-258` draws one `ScrollArea::vertical` containing:

| block | where | content |
|---|---|---|
| **A — the pointer**, unconditional | `mod.rs:214` → `idle.rs:70-85` | heading + one prose line. Drawn first *deliberately*: `mod.rs:206-213` — *"the row an operator's eye lands on must not move."* |
| **B, idle** — the tool list | `mod.rs:220` → `idle.rs:119-235` | 3–6 rows, **each 4 widgets**: a `Button`, the tool's sentence, a `{tab} · {chord}` line, a space. Hard cap `MAX_ROWS = 7` (`idle.rs:250`), asserted at `:236-241`. |
| **B′, idle** — Select's own options | `mod.rs:245` → `armed.rs:314-343` | heading + **three checkboxes** (scale line weight, keep insets, allow distortion) + a note |
| **B, armed** — the armed block | `mod.rs:248` → `armed.rs:64-81` | heading, identity, stage, **options**, measure points, **put down** |
| **C — disclosures** | `mod.rs:256` → `mod.rs:280-298` | one label per note, verbatim and wrapped; renders **nothing** when empty |

**Two of those blocks cannot survive a 28 pt strip, and both have documented
reasons for existing.**

★ **The armed block carries real controls, not text.** `armed::options`
(`armed.rs:98`) draws the text pen's **font picker, size, and a colour
swatch** (`armed.rs:125-160`, including `ui.color_edit_button_srgb` at
`:157`); `armed::measure_points` (`:216`) draws the measure options; and
`armed::scale_switches` (`:314`) draws three live checkboxes. A 28 pt line
has nowhere to put a colour picker.

★★★ **Block C exists precisely because R128 forbids the alternative.** The
module's own header, `mod.rs:271-282`:

> *"It renders `last_edit_disclosure`, which is the same slot the status
> bar's one elided line reads — and which carries, among other things, **the
> text tools' refusal sentences**. Those were written well, are tested, and
> have never been readable: 47 words in a row R128 forbids growing. A dock
> panel's width is the dock's, decided before the body draws. So the sentence
> that has been telling operators why their click was declined finally has
> somewhere it fits."*

**A 28 pt strip is a row.** Moving the disclosure back into one puts a
47-word sentence into the surface that was measured to be unable to hold it.

★★ **And the panel's founding argument is the exact argument the strip
makes.** `panels/tool/mod.rs:11-21` is quoted in `REVIEW_TRIAGE.md:102`
against A7's neighbouring bullet:

> **A7** *"Drop the Tool panel's buttons — they are on the ribbon."* →
> **ruled:** *"That is the exact assumption the panel exists to refute. The
> text tools were registered, drawn, chorded and driven-verified — 'The
> feature works. He could not find it.'"*

The strip keeps the identity line (`armed.rs:365` `identity`), the stage line
(`:392` `stage`) and `put down` (`:641`), and drops **the list** — which is
the discoverability fix itself. `app/modes/defaults.rs:427-451` states the
placement rule the strip would reverse:

> *"Its entire purpose is being OFFERED rather than asked for — it exists
> because an operator could not find a command that was on the ribbon all
> along. A tab that is invisible until clicked cannot fix a discoverability
> defect: it has the same shape as the defect."*

### 3.2 What the proposal changes, and the hard blocker

*"Select · click to pick · drag to marquee… [put down]"*, 28 px, at the top of
the right dock, replacing the panel.

**★★★ It cannot be a dock stack.** `plan::MIN_STACK_HEIGHT = 80.0`
(`dock/plan.rs:104`) is a **layout** floor, not only a drag floor — it is
passed to `plan::resolve_spans` at `dock/mod.rs:702` and to
`plan::drag_boundary` at `:936`. Its doc comment states the reasoning:
*"Enough for a tab bar plus two rows of content. Below that a stack is a
header with nothing under it, which reads as a rendering fault rather than as
a small panel."* And `TAB_BAR_HEIGHT = 24.0` (`plan.rs:170`) means a 28 pt
stack would be a tab bar plus four points.

So the strip has to be **dock chrome**, not a panel — a per-side banner drawn
above the columns, exactly as the collapse chevron is drawn over them
(`dock/mod.rs:572-577`, *"Over rather than inside, because inserting it into
the column layout would take height from a panel body on every frame — and
it is dock chrome, not a panel's content"*).

### 3.3 Feasibility, seam and cost

**Feasible as a shell seam; the shape is `Dock::with_side_banner(side,
height_pts, &mut FnMut(&mut Ui))`.** `draw_side` already carves `area`
(`dock/mod.rs:569`) before handing it to `draw_side_contents` (`:571`); a banner takes
a constant slice off its top first. That is generic — a banner above a side's
columns knows nothing about tools — so it is R7-clean, ~80 lines in
`egui-shell`, and the strip's *content* stays in `pdfcer-gui`.

★ Note the shell change is real but small, and it is the **only** part of
this proposal that is not a regression.

**But the strings do not exist in the mock's form.** We have one prose
sentence per tool — `t::row_select()` at `text/tool.rs:105-107`, *"Pick a
shape, then drag it. The tool everything comes back to."* — and no
`·`-delimited gesture fragments anywhere. The nearest existing pattern is
`row_home`'s `format!("{tab} · {chord}")` (`text/tool.rs:96-101`). Writing
~9 new compressed strings under R1 is a half-day and a shortening decision.

**And `put down` is not a command.** `armed.rs:641` is a panel button writing
`canvas::tool::select(ctx, CanvasTool::Select)` directly, with a recorded
argument for why (`armed.rs:619-635`: a tool is not document state). It is
also **only drawn in the armed block** (`armed.rs:80`), so the mock's
`[put down]` beside `Select` would be inert — Select *is* the resting state
(`mod.rs:216`). That is a small error in the mock and it would become a
shipped dead control if copied, which R9 forbids.

### 3.4 Verdict, and the salvage

**Do not build the strip as a replacement.** As specified it is a capability
regression: it deletes the armed options block, orphans Block C into a
surface R128 forbids, reverses a placement decision recorded at length
(`defaults.rs:427-451`), and re-litigates a triage ruling
(`REVIEW_TRIAGE.md:102`) that the reviewer did not defeat.

**The salvage is that the strip is a good idea in the wrong place.** The
identity line — *what am I holding?* — is genuinely worth 28 pt of permanent
chrome, and the status bar already carries a `Select` indicator in both our
build and the mock's. So:

> **Move the identity+stage line to the status bar or to a dock banner, and
> leave the panel where it is.** That costs ~120 lines, buys the mock's
> readability at frame one, and takes nothing away. It does **not** solve
> A7's room complaint, because the panel still occupies its third — which is
> why §5 ranks the room fix separately and above it.

★ If the operator *does* want the panel's third back, the honest lever is
one line at `defaults.rs:427` making Tool a **tab of the Objects stack**
rather than a stack of its own. That is a direct reversal of the ★★ comment
at `defaults.rs:427-451` and should be put to him as such — not smuggled in
as a strip.

### 3.5 The driven check, and what would make it vacuous

If the banner is built: `the_armed_tool_is_named_in_the_dock_banner` —
arm a tool by chord, assert a `dock-banner-<side>` region publishes through
`ui_rect_visible`, and assert a trace line naming the armed tool matches the
armed state (`canvas::tool::selected`).

**It is vacuous if it asserts only the region's presence**, because the
banner has a constant height and will publish a rect whether or not it
painted anything — `legibility.rs:402`'s `absent` bucket exists for exactly
this: a uniform fill means nothing was drawn. Use
`pixels::region_not_uniform` (`pixels.rs:319`) on the banner rect.

**And any check written against the *existing* Tool panel must not be
deleted along with it.** `mod.rs:222-238` records that
`the_line_weight_switch_reaches_the_resize` caught `scale_switches` written
into a branch `CanvasTool::Select` **cannot reach** — *"an option row added
there is dead code that compiles, reads correctly, and draws nothing … Every
unit test in the chain passed. Nothing tested that the control is on
screen."* That check is the reason the switches work. A strip that removes
the panel would make it SKIP forever, silently.

---

## 4. Proposal 4 — rulers on two edges with a corner box

### 4.1 What we ship today: all of it

`crates/pdfcer-gui/src/canvas/rulers.rs`, 1,413 lines.

| the proposal asks for | we ship | where |
|---|---|---|
| a top ruler | ✅ | `Gutters.top` `rulers.rs:319`, built `:395-398` |
| a left ruler | ✅ | `Gutters.left` `rulers.rs:321`, built `:399-402` |
| **a corner box** | ✅ **as an explicit rect** | `Gutters.corner` `rulers.rs:327`, built `:406` |
| ticks and labels | ✅ major/minor ladder | `MAJOR_TICK_PTS` `:265`, `MINOR_TICK_PTS` `:273`, `ticks()` `:980` |
| the page's own span marked | ✅ as a tint | `PAGE_SPAN_ALPHA` `:282`, drawn `:926-939` |
| pointer position on both rulers | ✅ crosshair | `rulers.rs:960` |

★ The corner is held as a **rect rather than left implicit**, and
`rulers.rs:322-326` says why: *"the two rulers must not overlap in it: a tick
drawn into the corner by both would be drawn twice, at two different alphas,
and would read as a defect at exactly the place the eye starts."* That is a
finer point than the mock makes.

★★ And it is **R128-correct**, which is more than the mock can claim:
`THICKNESS_PTS = 22.0` at `rulers.rs:237` is a constant, is the only thing
`reserve` subtracts (`:376-407`), and the constancy is **asserted** by
`the_gutters_are_a_constant_bite_out_of_the_viewport` at `rulers.rs:1068`.
`rulers.rs:160-181` carries the reasoning. There is also a graceful
degenerate case: a canvas with less than three gutters' width returns the
no-ruler shape rather than clamping (`:380-390`), *"the honest answer to
'there is not enough room to draw this'."*

★★★ And the labels come from `pdfcer_core::dimension::format_measurement`
(`rulers.rs:53-61`), so a ruler reading and a placed dimension **agree to the
digit** — which the mock, being HTML, does not attempt.

### 4.2 The actual delta: rulers ship **off**

`PageChrome` derives `Default` (`app/prefs/opening.rs:180`), so `rulers` is
`false`, and the field's doc comment at `app/prefs/opening.rs:183-188` states
the decision:

> *"The one overlay with a **measurable** cost when on: it takes
> `canvas::rulers::THICKNESS_PTS` off two edges of every canvas, for every
> operator, on every document. That is why it ships off and why the setting's
> copy says what turning it on costs rather than presenting it as free."*

The toggle is `view.rulers`, registered at
`shell/commands/catalog/view.rs:374`, on **View ▸ Display** as an icon-only
control (`shell/manifest/view.rs:319`), and also in Settings ▸ Display
(`dialogs/settings/display.rs:311`). It is read at `canvas/present.rs:168`.

**So the delta is a decision, not a build:** should rulers default on? The
argument against is written down and is good. The argument for is that the
reviewer produced 80 screenshots without ever seeing them, which is a
discoverability data point — the same class of finding as A16b
(`REVIEW_TRIAGE.md:118`, *"That the reviewer did not find it is itself a
discoverability finding"*).

★ **Recommend: leave the default off, and do nothing else.** The cost
sentence is real, `THICKNESS_PTS` × 2 edges is 44 pt off a 1100 pt default
window, and a drafter who wants rulers will find a control that is on the
View tab and in Settings. If discoverability is the worry, the cheap answer
is a line in `MANUAL.md`, not a default change that costs every operator
44 pt.

**One genuine gap, and it is not the rulers'.** Guides can only be dragged
out of a ruler gutter, so `guides` without `rulers` is inert —
`app/prefs/opening.rs:194-198` says so and says the setting's copy states it.
Worth confirming that copy is still accurate; it is the kind of sentence that
goes stale.

### 4.3 Cost, hazards, and the check

**Cost: zero, unless the default changes, in which case it is a suite-wide
harness re-baseline** — 22 pt off the top and left of the canvas moves every
canvas-relative coordinate in `tools/ui-verify/`. §1.6 hazard 2 applies in
full. ★ **A one-boolean change is the most expensive-per-character item in
this document**, and that is exactly the shape of thing this project has been
caught by before.

**The check that should exist and I could not find:**
`the_rulers_take_the_canvas_and_the_zoom_does_not_drift`. Turn rulers on
through the UI while a fit mode is active; assert the fit zoom settles to
**one** new value within two frames rather than walking. That is R128's
signature and no unit test can see it — `rulers.rs:1068` asserts the
*constancy of the reservation*, which is the input, not the outcome.

**It would be vacuous if it sampled the zoom readout only once after
settling** — the drift the RAG entry measured (230 → 224 → 215) settles too.
It must sample across consecutive frames and assert monotone convergence in
one step, not eventual stability.

---

## 5. What to build first, and what not to build at all

Ranked. The operator reads this to schedule, so the ranking is by value per
risk-adjusted day, and the honest answer is that **the cheapest item is worth
more than the most impressive one.**

### 1st — Give the right dock its room. ~1 day of code, ~2 days of harness.

`INSPECTOR_WIDTH: f32 = 320.0` → 360 (`app/modes/defaults.rs:260`), plus the
row icon/index columns (~60 lines in `panels/objects/mod.rs`). Per-mode
width is already modelled and already persisted (`model.rs:333`,
`app/modes/mod.rs:355`), so A7's *"let the dock remember the width per mode"*
needs no work at all.

**Why first:** it is the entirety of what survives §0.2 from proposal 2, it
fixes the clipping the reviewer actually photographed, it needs **no shell
change**, and it touches no settled decision. ★ Budget the harness
re-baseline honestly — it is the larger half.

### 2nd — The left icon+word rail. ~1 week + the re-baseline.

**Worth building, and worth being told plainly that it is not S6.** It is the
only arrangement in which all five left panels are simultaneously one click
away, because a 280 pt horizontal tab bar cannot hold five tabs and a 700 pt
vertical rail can hold five entries with room to spare. It ends the permanent
half-height reservation for three panels that are empty on nearly every
document.

**Conditions on scheduling it:**

- **Fix the rect channel first.** The dock publishes through the unguarded
  `diag::ui_rect` (`app/surfaces.rs:246`). Until rail regions go through
  `diag::ui_rect_visible` (`diag.rs:408`), **no check can distinguish a
  working rail from the 2026-08-10 defect**, which was this feature.
- **Convert every harness coordinate in one pass**, not thirteen-of-fourteen.
- **The R128 gate at `OPERATOR_REQUESTS.md:435-437` should be discharged by
  argument and a constant-width test**, not by building the fit-zoom cache
  first. §1.6 hazard 3 gives the argument. It becomes a real gate the moment
  anyone sizes the rail from its widest word.
- **Do it as a `SideLayout` mode**, so the arrangement is expressible in the
  manifest. A rail that only `pdfcer-gui` knows about breaks
  `SHELL_FRAMEWORK.md:21-24` quietly.

★ The cheaper 80 % — merging the two left stacks into one five-tab stack, one
line at `defaults.rs:412-421` — is available today and is a reasonable
interim. It buys the height back and pays for it with an overflow chevron.
Offer it as the alternative; do not present it as the same thing.

### 3rd — Rulers. Nothing to build.

Already shipped on two edges with a corner box, R128-correct, unit-agreeing
with the dimension engine. The only open question is the default, and the
recorded reason for `false` is sound. **Recommend leaving it, and spending
nothing.** ⚠ If the default is changed, price it as a suite-wide harness
re-baseline, not as a boolean.

### 4th — The tool strip. ~~**Do not build it as proposed.**~~ **BUILT 2026-09-04**

> ⚠ **This ranking was overruled by the operator on 2026-09-04** —
> `OPERATOR_REQUESTS.md` O123. See the box at the head of §3 for what was
> answered and what was conceded. The paragraphs below are left standing
> because a recommendation that was overruled is more useful to the next reader
> than a gap where it used to be.


Stated plainly, because an optimistic assessment is worse than none:

- It **cannot be a dock stack** — `MIN_STACK_HEIGHT = 80.0` (`plan.rs:104`)
  against a 28 pt strip.
- It **deletes live controls**: the text pen's font, size and colour swatch
  (`armed.rs:125-160`), the measure options (`:216`), the three scale
  switches (`:314-343`).
- It **orphans the disclosure slot into the surface R128 forbids** —
  `mod.rs:271-282`, *"47 words in a row R128 forbids growing"*. The panel
  exists because that sentence had nowhere else to go.
- It **re-litigates a ruling the reviewer did not defeat** —
  `REVIEW_TRIAGE.md:102` on *"drop the Tool panel's buttons"* — in a
  different shape, and it reverses `defaults.rs:427-451` without addressing
  its argument.
- Its `[put down]` beside `Select` **would be inert**, because Select is the
  idle state (`mod.rs:216`, `armed.rs:80`) — a dead control R9 forbids.

**Salvage worth ~120 lines:** put the *identity + stage* line into a dock
banner or the status bar as an **addition**, and leave the panel alone. If
the operator wants the panel's third of the dock back, put the real lever to
him — Tool as a tab of the Objects stack, one line at `defaults.rs:427` —
and let him overrule his own recorded placement knowingly.

---

## 6. Two things the operator should be told regardless of scheduling

**★★★ The dock's rect channel is unguarded, and that is a live gap under all
four proposals.** `app/surfaces.rs:245-247` publishes every dock region
through `diag::ui_rect` rather than `diag::ui_rect_visible`. Every driven
assertion about a docked panel today therefore proves **layout**, not
**visibility** — and `preset_group_reachable.rs:43-55` is the file that
explains why those are different, citing *"this project shipping panels that
were unreachable in real builds with every gate green."* One line, and it
makes every future dock check mean what it says.

**★★ The harness cannot read text a panel renders, and three of these four
proposals are about text a panel renders.** No AccessKit reader, no OCR, no
extraction — only trace lines the application publishes deliberately
(`trace.rs:83`). The Objects panel publishes **one aggregate line**
(`objects/mod.rs:301-303`) whose `rows=` is a layout count. Any check written
for proposal 2 has to bring its own oracle with it, and the pixel channel
(`legibility.rs`, `pixels.rs:319`) is the only non-circular one available.
