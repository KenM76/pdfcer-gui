# Modes and panels

**Status:** proposal, 2026-08-13. Operator additions to `RIBBON_IA.md`.
Nothing built.

Two additions, and they turn out to be one system:

1. A three-position **Read — Review — Edit** selector at the far right of
   the tab row, which changes what the interface contains so pdfcer can
   look like a reader rather than an editor.
2. **Left and right panel areas as flexible as Inkscape's.**

They combine cleanly: **a mode is a named workspace layout.** The panel
system has to be flexible enough for three genuinely different layouts
to be worth having, and the mode selector is what makes that flexibility
reachable without the operator arranging anything by hand.

---

## Part 1 — The Read / Review / Edit selector

### Placement

Far right of the tab-strip row, in line with the tab labels:

```
[Open][Save][↶][↷]  File View Pages Edit Markup Measure Tools      ( Read ─ Review ─●─ Edit )  [⌃]
```

The one-line document summary that sits there today (`file.pdf — PDF
1.7, 42 pages`) **moves to the status bar**, where document facts belong
and where it does not compete with a control.

The ribbon collapse chevron stays at the extreme right, outboard of the
selector, because it acts on the ribbon rather than on the document.

### What each position contains

| | **Read** | **Review** | **Edit** |
|---|---|---|---|
| **Tabs** | File · View | File · View · **Pages** · Markup · Measure | all seven + Format |
| **Page display default** | **Continuous scroll** | Single page | Single page |
| **Canvas gestures** | pan, zoom, text selection for copy, follow links | + place and edit **your own** markup and dimensions | + full content selection and editing |
| **Panels offered** | Pages, Bookmarks, Layers, Signatures, Fonts, Attachments, Comments *(read)* | + Comments *(authoring)*, Properties *(markup only)* | + Objects, Forms, Redact, Batch, Properties *(everything)* |
| **Status bar** | page nav, zoom, find | + tool state, snap | + selection count, edit state |
| **Badge** | `Read` | `Review` | — |

**Read** is the point of the whole feature: a PDF viewer, with pdfcer's
inspection panels available but nothing that authors anything. This is
the mode a person is in when someone sends them a drawing.

**Read defaults to continuous scroll; Review and Edit default to single
page.** *Operator decision, 2026-08-13.* This resolves the earlier
tension cleanly rather than by compromise. Reading a document is a
continuous act — you scroll through it, and a page boundary is an
interruption. Marking up and editing a drawing is a per-sheet act — you
work on one sheet, and paging is how you move between them deliberately.
The right default was never global; it was per mode, and the mode
selector is what makes that expressible.

Each mode remembers the operator's own choice, so someone who prefers
paging in Read gets it after they say so once. The defaults above are
what a fresh profile starts with.

**Review** is the markup stance — comment, dimension, measure, cloud —
plus the **Pages** tab. *Operator decision, 2026-08-13.* An earlier draft
excluded Pages on the reasoning that delete, extract and merge are
structural; that was overruled, and correctly. Reviewing a drawing set
means rotating a sheet to read it, extracting the two pages you were
asked about, and inserting a marked-up revision — all reviewer work.
The stance that matters is *the page content is not yours to alter*, and
page operations do not alter content.

**Edit** is everything, and is what `RIBBON_IA.md` specifies today.

### The rule that makes this safe

> **A mode changes what is *visible*. It never makes a visible control
> silently inert.**

This is the whole difference between this feature and the
`editing_enabled` master toggle that was removed two turns ago at the
operator's instruction. That toggle was a hidden binary that left the
editing tools on screen and made gestures quietly do nothing. This is a
named, visible, three-position control that **removes the tools it
disables**. In Read mode there is no Edit tab to click, so there is no
click that mysteriously fails.

It also fulfils, rather than contradicts, what `RIBBON_IA.md` §5.4 said
when the toggle was removed:

> *If a genuine read-only mode is ever wanted it should be a **document**
> state with a visible badge, not a hidden global toggle.*

That is exactly what this is.

### Behavioural rules

1. **Switching modes never destroys work.** Read ⇄ Edit is a view
   stance, not a save boundary. Unsaved edits survive a trip through
   Read mode untouched. If a mode change would hide a *pending,
   uncommitted* gesture — a half-placed dimension, an open text edit —
   that gesture is committed or cancelled first, with a prompt, exactly
   as Escape already does.
2. **The undo stack is not cleared, ever**, and undo continues to reach
   across a mode change. Anything else would make the mode a trap.
3. **Each mode remembers its own panel layout**, per Part 2. Leaving
   Edit and coming back restores the arrangement, not a default.
4. **A document may open into a mode.** A signed, encrypted, or
   read-only-on-disk document opens in **Read** with the badge stating
   *why* — "signed: editing would invalidate the signature",
   "encrypted: pdfcer cannot write encrypted documents yet". Today those
   conditions are disclosed in the status bar and the operator can still
   arm tools that will later refuse. Opening in Read tells the truth
   earlier.
5. **The default mode is a setting**, not a fixed choice. Someone whose
   job is drafting wants Edit on open; someone who receives drawings
   wants Read.
6. **Keyboard**: `Ctrl+1` / `Ctrl+2` / `Ctrl+3`, and the selector is a
   real focusable control with arrow-key movement — not a mouse-only
   affordance.

### Why a slider rather than three buttons or a dropdown

The three positions are **ordered by capability** — each is a superset
of the one before. A slider says that; three toggle buttons do not, and
a dropdown hides the current position behind a click. The ordering is
the information, and it is what makes "slide left to calm the interface
down" an obvious gesture rather than a learned one.

It must still render as a real segmented control with all three labels
visible — not a bare track with a knob, where the available positions
are invisible until you drag.

### What this does *not* do

It is not a permissions system. Read mode does not protect a document
from anything; a determined operator moves the slider. It is an
interface-complexity control, and the badge says which stance you are
in — nothing more. Anything stronger would need document-level
protection, which is a different feature.

---

## Part 2 — Flexible panel areas

Direct observation of Inkscape 1.4 is archived at
`evidence/inkscape-dock-observed.png`. Source was deliberately **not**
read: Inkscape is GPL-2.0-or-later, pdfcer is MIT, and
`pdfcer-inkscape-librarian` carries a binding rule against GPL code and
GUI mimicry. Nothing is liftable anyway — Inkscape is GTK retained-mode
C++; pdfcer is egui immediate-mode.

### This is not a new ask

`docs/decisions/017-tabbed-dockable-panel-system.md` **Amendment A**
records the operator's own trigger, verbatim (`:363`):

> *"Use egui_tiles. You're building something to compete with Acrobat
> and is open source, and has the flexibal docking that works as well as
> inkscape's."*

`dock.rs` is the partial answer. What shipped was deliberately
conservative, and A.8 explicitly deferred the hardest part: *"§10 Q2
(undock into separate OS windows / multi-monitor) — NOT answered here
and NOT granted by adopting `egui_tiles`, which has no `Surface::Window`
equivalent."* This section closes the gap between what was asked for and
what shipped.

### Observed target behaviour, from driving Inkscape 1.4

- The right dock carries **many dialogs as icon tabs** in one strip, with
  an **overflow chevron** when they do not fit. The active dialog shows
  its name and an individual close control; the others are icons.
- **A floating dialog window is itself a dock.** The XML editor, torn
  out, has its own tab strip and its own overflow chevron — so several
  floating dialogs can be grouped into one window rather than
  proliferating windows.
- Dialogs are **individually closable** from their tab.
- The dock **collapses** from a control on the toolbar row.
- Tool rails are **separate from the dialog dock** — a tool rail on the
  left and a command rail on the right, each with its own overflow —
  so tools and panels do not compete for the same space.

### What this means for pdfcer

The current dock (`dock.rs`, 818 lines) is deliberately much more
restrictive: two independent trees that cannot exchange panes, no
body-drag undocking, default tab groups capped at two panes, and a
layout that is not persisted at all. Those constraints were each
reasoned and are documented in the file — but together they are roughly
the opposite of what is being asked for here.

### Inkscape is the floor, not the ceiling

Inkscape's docking is genuinely flexible — two-sided docks, multiple
columns per side, vertical stacks, tabbed notebooks, tear-out to a
floating window that is itself a dock. Its current system is a 1.1
rewrite (GSoC 2020) that replaced an older one.

But calibrated against its peers, **it is worst-in-class on exactly the
three things this project needs most**:

| | Inkscape 1.4 | Blender | VS Code | Photoshop | Krita | Affinity |
|---|---|---|---|---|---|---|
| Dock left **and** right | ✅ | free tiling | ✅ | ✅ | ✅ 4 edges | ✅ |
| Multiple columns per side | ✅ | — | ❌ | ✅ | ✅ | ? |
| Tabs within a stack | ✅ | ❌ | panel only | ✅ | ✅ | ✅ |
| Tear out to OS window | ✅ any | ✅ areas | editors only | ✅ | ✅ | ✅ |
| **Collapse dock to icons** | **❌ lost in 1.1** | — | ✅ | ✅ | partial | ✅ |
| **Named layouts** | **❌** | workspace tabs | profiles | **✅ + shortcuts/menus** | **✅ `.kws`** | **✅ Studio Presets** |
| **In-app reset layout** | **❌ delete a file** | ✅ | **✅ Reset View Locations** | ✅ two-tier | ? | **✅ Reset Studio** |

Inkscape has **no named workspaces** — the community workaround is
copying `dialogs-state-ex.ini` aside and back, awkward because the file
is rewritten on every exit. It has **no in-app layout reset** — the
documented route is to quit and delete that file. And per-dock
**collapse-to-icon existed in 1.0 and was removed in the 1.1 rewrite**;
the regression is still open five releases later, with only an
all-or-nothing `F12` hide-everything as substitute.

**pdfcer already beats Inkscape on all three, by design.** The
Read/Review/Edit selector *is* named workspaces. `Action::ApplyResetLayout`
with per-scope checkboxes already exists and is a better reset than any
product in that table. The icon rail is scoped at ~1 week. So the target
is Inkscape's *flexibility* plus Photoshop's and Affinity's *layout
management* — not Inkscape wholesale.

### Twelve failure modes to design against

From Inkscape's own issue tracker and release notes. These are worth
more than the feature list, because each is a trap this project would
otherwise walk into.

| # | Failure | Design rule it implies |
|---|---|---|
| 1 | **Ambiguous drag handle** — the OS title bar and Inkscape's own stack vertically; grabbing the wrong one silently does nothing. *The single most common "docking is broken" report.* | One unambiguous grab affordance. Never nest a drag handle under OS chrome that looks the same. |
| 2 | **Weak drop feedback** — pre-1.2 there was effectively none; users concluded the feature did not exist. | Feedback must encode the *outcome*, not merely "valid target". |
| 3 | **Widest hidden tab dictates minimum width** — an inactive tab you cannot see holds the whole dock open; you must close it to narrow the dock. | Size a container to its **active** child; let inactive children scroll. |
| 4 | **Minimum widths of 450–500 px** — "up to a third of my screen width." | Budget minimums per panel and test at 1280 px wide. |
| 5 | **No per-dock collapse** (see above). | Ship the icon rail; do not defer it indefinitely. |
| 6 | **Layout not stable under window resize** — un-maximise and re-maximise loses panel proportions. | Store proportional sizes with pinned minimums. Restore, do not recompute. |
| 7 | **Coupled splitters** — dragging one divider resized every column. | A splitter affects its two neighbours only. |
| 8 | **Tab overflow has no escape** — past ~6 tabs the overflow *button itself* gets hidden, leaving no route to the hidden tabs. | The overflow affordance is reserved space, never the first thing squeezed out. |
| 9 | **Crashes in the stacking path** — docking under another dialog was a 100 % crash on macOS until 1.3.1. | Fuzz the drop grammar. |
| 10 | **Floating windows are second-class** — they minimise with the parent, sink behind other document windows, and *swallow keyboard shortcuts when focused*. | Decide deliberately whether torn-out panels are transient children or peers, and make shortcuts window-agnostic. |
| 11 | **Focus-existing shows stale content** — reopening a stacked dialog selected its tab but kept rendering the previous one. | Selecting a tab must invalidate what is painted. |
| 12 | **No named workspaces, no in-app reset** (see above). | Both are table stakes, not luxuries. |

### Three of these pdfcer avoids by construction

Worth recording, because they are accidental advantages of the stack
rather than decisions anyone made:

- **#3 (widest hidden tab)** — `egui_tiles`' `Behavior::min_size` is a
  single **global** scalar applied only as a resize-drag floor, and
  `Tabs::layout` lays out *only the active tab*. An inactive pane cannot
  impose a width.
- **#10 (shortcuts swallowed by a floating window)** —
  `show_viewport_immediate` keeps the torn-out panel inside the same
  `App::ui` call and the same action dispatcher, so keyboard handling is
  window-agnostic without any extra work.
- **#1 (ambiguous handle)** — pdfcer draws its own chrome; there is no
  second OS title bar to confuse with the panel header.

And one it walks straight into: **#8, tab overflow.** `egui_tiles` 0.16
answers an overflowing tab bar by hiding tabs behind scroll arrows with
`ScrollBarVisibility::AlwaysHidden` — the same class of failure. The
existing `dock.rs` already caps default tab groups at two panes
specifically to dodge it, with a test enforcing the cap. **The ~1-day
overflow menu is what retires that cap safely**, and it must reserve its
own space rather than compete for it.

### ⚠ Decision, 2026-08-13: the dock was built **without** `egui_tiles`

**How it happened, stated plainly:** `egui_tiles` is declared in the
workspace root but was never added to `crates/egui-shell/Cargo.toml`
(its own comment says *"arrives with the `dock` module at S3"*), and the
S3 dispatch put `Cargo.toml` on the do-not-touch list to prevent
concurrent agents contending for it. So the dependency never arrived and
the dock was built on `egui` primitives directly. That was a dispatch
error, not a design decision.

**It is being kept anyway, on the merits.** Judged against what was
actually specified rather than against the tool the verdict table
happened to assess:

| Capability | Specified for | Status |
|---|---|---|
| (a) two columns per side | fold-in | ✅ built |
| (b) vertical stacks, resizable | fold-in | ✅ built |
| (c) tabs within a stack | fold-in | ✅ built |
| (f) layout persistence | fold-in | ✅ built |
| (g) named workspaces | fold-in | ✅ built |
| (h) collapse to icon rail | S6 | achievable, not yet built |
| (d) cross-dock drag | **post-fold-in** | ✗ lost |
| (e) tear-out to a window | **post-fold-in** | ✗ was never `egui_tiles`' anyway |

**Everything lost was already post-fold-in**, and (e) had *zero*
`egui_tiles` support in the first place — the verdict table says so.

Four independent reasons the accident is defensible:

1. **`egui_tiles`' simplification actively fights persistence.**
   `prune_single_child_containers` and `join_nested_linear_containers`
   mean *a column is not a durable identity* — it is whatever the tree
   currently happens to be. A named workspace that silently loses the
   column the operator made is worse than no workspaces.
2. **Persistence had to be hand-written regardless.** The crate's serde
   support sits behind its default feature, which this workspace
   disables. An owned schema versions independently and does not persist
   internal tile ids that mean nothing across a restart.
3. **Its tab overflow was a defect to work around, not a feature to
   use** — `ScrollBarVisibility::AlwaysHidden`, i.e. failure mode #8 in
   the dependency itself. The previous dock capped groups at two panes
   purely to dodge it. The new one reserves the affordance and **the cap
   is retired** (nine panels in one stack, tested).
4. **It ships no accessibility instrumentation in 0.16.** Names had to be
   supplied from outside either way.

**What is genuinely given up:** panel drag-and-drop between
compartments, and arbitrary-depth tiling. Depth is fixed at
side ▸ column ▸ stack ▸ tab.

**The migration stays cheap if it is ever wanted:** the loader builds a
tree instead of a `DockLayout`; the persisted form does not change. So
adding `egui_tiles` later costs the dock's internals and nothing
operator-visible — which is the property that makes keeping this
reversible rather than a lock-in.

### Capability verdicts — egui 0.35 / egui_tiles 0.16

Assessed against the vendored crate sources and the empirical findings
in `D:\dev\rag\egui\`.

| | Capability | Verdict | Cost |
|---|---|---|---|
| **a** | Two independent dock columns on one side | **Achievable** — two sequential `Panel::left(…)` each hosting a tree (the shipped left/right pattern), or one tree rooted in a horizontal `Linear` of two vertical `Linear`s. Vertical-in-horizontal survives simplification. | ~½ day |
| **b** | Vertical stacking within a column, resizable | **Already shipped** — `default_left_tree()` is exactly this, with draggable splitters and double-click-to-centre. | zero |
| **c** | Tabbing panels together within a stack | **Already shipped**, with a caveat: an overflowing tab bar **hides** tabs behind scroll arrows, which is why the current default caps groups at two panes. Raising the cap needs a "⌄ N more" overflow menu, buildable in `top_bar_right_ui`. | zero as-is; ~1 day for the overflow menu |
| **d** | Drag a panel between left and right docks | **Achievable with work.** Not native — drag identity is tree-scoped, so tree B cannot recognise tree A's dragged tile. Two routes below. | 1–2 weeks (wide tree) or ~3 days (hand-rolled) |
| **e** | Tear out to a floating OS window, re-dock | **Achievable with work** — and the constraint I expected to kill it does not apply. See below. | 2–4 weeks; ~1 week for a cut-down version |
| **f** | Persist the arrangement across sessions | **Achievable.** The whole tree derives serde behind the crate's *default* feature, which pdfcer currently disables. Persists topology, splitter shares, active tab per group, and the pane payload. | **2–3 days — highest value per hour on this list** |
| **g** | Named / savable workspace layouts | **Achievable** — strictly a superset of (f). | +3–4 days on top of (f) |
| **h** | Collapse a dock to an icon rail | **Achievable with work** — not an `egui_tiles` feature, but it does not need to be: a narrow panel of icon buttons drawn *instead of* the tree, leaving tree state untouched. | ~1 week, mostly icons and tooltips |

### The two findings that change the design

**Tear-out is not blocked by the immediate-mode borrow.** I expected
`Send + Sync + 'static` on the viewport callback to force app state
behind an `Arc<Mutex<…>>`, which would have been fatal.
`show_viewport_deferred` does carry that bound — but
**`show_viewport_immediate` does not.** It takes `FnMut` with no
lifetime bound, so it can be called from inside `App::ui` capturing
`&mut self`. **A torn-out panel therefore keeps the identical
`panel_body(&mut self, panel, ui, actions)` signature as the docked
one** — the one-dispatcher rule survives intact. Re-docking is simply
ceasing to call it; egui garbage-collects the unused viewport and eframe
destroys the window. glow supports it on Windows, and one painter is
shared across viewports, so page textures cost nothing extra.

What *is* missing is all of `egui_tiles`' side: it has no tear-out
concept whatsoever (`grep viewport|window|float|tear` → zero hits), and
its drop-target set is hard-coded and not extensible. So the gesture,
the detached state, and the re-dock targeting are all bespoke.

**R128 is the sharpest constraint, and it gates the best route to (d).**
A content-driven panel adjacent to a per-frame fit-to-viewport zoom is a
feedback loop — measured at 230 % → 224 % → 215 % zoom drift from a
growing status line. Only `Panel::exact_size` closes it, and that is
precisely what a *user-resizable* dock cannot use. The elegant answer to
(d) is one wide tree spanning left ▸ canvas ▸ right, which makes
cross-dock drag native and free — but it puts the canvas inside a
resizable pane and fires R128 directly. **So the fit-zoom must be
converted to cached-recompute-on-explicit-trigger first, as its own
landing, before the wide tree is attempted.**

### Three prerequisites, before any of this

1. **Harness coordinates must become document-space.** Every capability
   here makes panel widths arbitrary at runtime. The RAG records this
   exact class producing a filed-then-retracted false "coordinate-space
   defect" — a stale scripted coordinate is symptom-identical to a
   broken screen→document conversion.
2. **A screenshot oracle for panel layout.** Two recorded instances
   where a traced rect was correct and the control was still clipped out
   of its pane: *"layout/clipping defects have exactly one oracle: a
   rendered screenshot."* This is `ui-verify`'s job.
3. **The fit-zoom cache (R128)**, before anything makes the canvas rect
   user-variable.

### One tension worth naming

Decision 024 and `shell-redesign.md` §2.4 exist **because the operator
disliked floating surfaces** — the tool accept/reject boxes were *"a
separate accept / reject box somewhere on the screen"*, and
`FEATURES.md:182` now states "nothing floats over the canvas" as an
invariant.

That is not in conflict with (e), but the distinction has to hold:
**a panel the operator deliberately tears out is not the same thing as a
box the application decides to float.**

**Operator decision, 2026-08-13: both become settings under View, not
invariants.** `FEATURES.md:182`'s "nothing floats over the canvas" is
retired as an absolute and replaced by two independent options:

| Setting | Values | Default | What it governs |
|---|---|---|---|
| **Floating panels** | Off · Allowed | Allowed | Whether the operator may tear a panel out into its own window at all. Off restores today's behaviour exactly. |
| **Application initiative** | Never · Ask · Allowed | **Never** | Whether pdfcer may float a surface over the canvas *on its own*, without the operator having asked — tool option boxes, transient property bars, notifications. |

The second is the one that carries the original complaint, and its
default is **Never**, which preserves decision 024's outcome as the
shipped behaviour while making it a choice rather than a law. The
operator who disliked a tool's accept/reject box appearing over the
drawing still never sees one; the operator who wants a torn-out
Properties panel on a second monitor is no longer blocked by a rule
written about a different problem.

Both live in **View ▸ Window**, beside the other layout controls, and
both are per-operator rather than per-document.

### How the modes map onto this

**A mode is capability (g).** Read, Review and Edit are three built-in
named workspaces, shipped as defaults, each remembering the operator's
arrangement of it. That is why the two requests are one system, and it
sets the build order: **(f) persistence is the foundation for both the
flexible panels and the mode selector**, which is convenient, because it
is also the cheapest item and R15's settings partition already landed to
unblock it.

### Recommended order

1. **(f) persistence** — cheapest, highest value, unblocks everything.
   A rearrangeable layout that forgets itself each restart is worse than
   a fixed one.
2. **(a) + (c) overflow menu** — small, and the overflow menu is what
   safely retires the two-pane cap distorting today's defaults.
3. **(g) named workspaces → the Read/Review/Edit modes.**
4. **(h) icon rail** — self-contained; budget the harness re-baseline.
5. **The fit-zoom cache (R128)** as its own landing, then
   **(d) via the single wide tree** — the real unlock.
6. **(e) tear-out** last, starting with a stationary "Float this
   panel…" command rather than drag-to-tear. It captures most of the
   value at a quarter of the cost and dodges the focus-gated
   `StartDrag` primitive entirely — and it sidesteps failure mode #1
   (ambiguous drag handle), which is the most-reported docking complaint
   in the product being used as the benchmark.

### Sources

Inkscape capability and pain points: `wiki.inkscape.org` release notes
1.0–1.4, `gitlab.com/inkscape` issue tracker, and the Inkscape
Beginners' Guide. Peer calibration: Blender, VS Code, Photoshop, Krita
and Affinity user documentation. **No Inkscape source was read or
downloaded** — the licence position in Part 2's header is binding, and
observation of the running 1.4 build plus published documentation
answered every question asked.

---

## ★★★ Amendment, 2026-09-05 — auto-hide, and the tab strip the rail replaces

Four changes to this document's subject matter, all from one operator message:

> *"we should also add the capability to auto hide the ribbon until we hover over
> top of it. our smart selector should be visible with the other navigate
> controls in our left rail. also we don't need tabs in the left side bar when
> the left rail is visible. left rail should also have the option to auto hide as
> well."*

### The model, and which product it is taken from

**Microsoft Office's *Show Tabs***, the middle of its three *Ribbon Display
Options*. The tab strip stays on screen permanently and only the band goes;
touching the strip brings the band back **over** the document; it goes when the
pointer leaves. The rail's version is VS Code's activity bar one surface over: a
permanent narrow edge, and a wide body that overlays rather than displaces.

Three properties are carried across deliberately, each answering a way this is
usually got wrong:

| Property | The failure it prevents |
|---|---|
| the trigger is **permanent** — a tab strip, a 10 pt sliver — and never hides | Office's full *Auto-hide Ribbon* is the setting people get stuck in, because the thing you must touch to leave it is invisible. **A mode you cannot leave is a trap.** |
| the body **overlays**, it does not displace | a canvas that resizes on hover moves every coordinate under the pointer as the pointer approaches |
| a reveal can only be **started** by the trigger | R128. See `egui_shell::peek`'s invariant: `revealed(n+1) = in(trigger) ∨ (revealed(n) ∧ in(overlay(n)))`. The overlay term is conjoined with last frame's state, so it can only *keep* a reveal — it cannot start one, and there is no cycle to oscillate around. |

Both settings are **off by default**, are persisted in `settings.txt`
(`ribbon_auto_hide`, `rail_auto_hide`), are checkboxes in Settings ▸ Display
under one heading, and have a command each on View ▸ Window.

### ⚠ Failure mode #13 — a rail that hides while it is the only route to a panel

This document's Part 2 lists twelve failure modes to design against. This
amendment adds one, because the two features above create it between them:

> **#13 — chrome that can hide, beside a switch that has been suppressed.** The
> rail is the *only* route to `markup.comments` in Read mode (that command is on
> the Markup tab, which Read does not show, and `RIBBON_IA.md` P1 forbids a
> second tab placement). Suppressing the dock's tab strip removes the other
> switch. If the rail could then *disappear*, the panel behind another in that
> stack would be reachable by nothing at all.

**Three mechanisms hold it, and all three are asserted:**

1. A hiding rail is never *gone*. It reserves `rail::PEEK_WIDTH_PTS` = **10 pt**
   of permanent, chevron-marked edge — above `Peek::MIN_TRIGGER_PTS` = 8 — and
   publishes it as `dock.<side>.railtrigger` on **every** frame, hiding or not.
   `a_hiding_rail_always_publishes_a_trigger_wide_enough_to_hit` asserts the
   published rectangle's width, not the constant, at six side widths.
2. The tab strip is suppressed only when **all three** of: a rail is configured
   for that side; the rail was **actually drawn** this frame (`resolve_width`
   returns zero on a side too narrow, so the rail is *absent rather than
   squeezed*); and the application's `with_rail_reach` predicate answers `true`
   for **every** panel in the stack — not the active one, not most of them.
   `a_side_too_narrow_for_the_rail_keeps_its_tab_strip_at_every_width` walks a
   width series and asserts the implication *suppressed ⇒ a rail was drawn*, and
   asserts that the series actually straddled the threshold.
3. Raising a specific panel still works: a rail row dispatches the panel's own
   command, `PdfcerApp::toggle_panel` asks `is_on_screen`, and that is `false`
   for a panel mounted **behind** another tab — so the press activates it rather
   than closing it. Closing is the same control pressed again.

### The reach predicate is an extension point, not a pattern match

R7: `egui-shell` may not learn what a PDF is. The dock holds opaque `PanelId`s
and the rail holds opaque command ids, and the map between them is application
knowledge — the Fonts panel is `file.fonts` and the Comments panel is
`markup.comments`, neither of which a `view.panel_*` pattern finds. The
application supplies `Dock::with_rail_reach`, derived from the **live manifest**
including any operator overlay, so a rail somebody customized stops suppressing
strips over panels they removed from it.

### The smart selector

`view.smart_select` is now the **last** row of the rail's `navigate` group,
mirroring View ▸ Navigate row for row. The comment that said it was
*"deliberately absent"* was right about the **pin** and wrong about the
**membership**, and conflated the two; it is corrected in place and dated at
`shell::manifest::rail`. At `Rung::Cramped` the group folds to the row whose
`selected:` condition holds, **first match wins**, and the four tools are listed
first — so the pin goes to the armed tool and the toggle folds behind the
chevron, one click away. A test asserts both halves against the real fold
planner.
