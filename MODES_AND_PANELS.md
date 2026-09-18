# Modes and panels

Two things that are one system: the **Read — Review — Edit** mode selector, and
the left and right panel areas it selects between. Read it before changing a
mode's contents, the dock's geometry, or anything that hides chrome.

**A mode is a named workspace plus a capability set.** The panel system is
flexible enough for three genuinely different arrangements to be worth having,
and the mode selector is what makes that flexibility reachable without the
operator arranging anything by hand.

Each part ends with what is specified here and not yet built.

---

## Part 1 — The Read / Review / Edit selector

### Placement and form

Far right of the tab-strip row, in line with the tab labels:

```
[Open][Save][undo][redo]  File View Pages Edit Markup Measure Tools   ( Read - Review - Edit )  [collapse]
```

The ribbon collapse chevron stays at the extreme right, outboard of the
selector, because it acts on the ribbon rather than on the document.

`egui_shell::ribbon::mode_selector` draws an **N-position segmented control**
from `Shell::modes()`, taking each position's `Mode::label`. Every position is a
labelled, hit-testable, individually reported segment. There is no
`egui::Slider` in the module, because a bare track with a knob hides which
positions exist until you drag; `every_mode_gets_its_own_labelled_segment`
asserts the segments.

The control looks up no icon and contains no literal `"read"`, `"review"` or
`"edit"`. A different application ships *Draft · Proof · Press* and gets the
same control with no code change — Read/Review/Edit is a configuration, not a
built-in (`SHELL_FRAMEWORK.md` §4).

**Keyboard.** `Ctrl+1` / `Ctrl+2` / `Ctrl+3` are bound to `mode.read`,
`mode.review` and `mode.edit` in the built-in keymap. The selector is also a
real focusable control with arrow-key movement, built as a roving tab stop:
`Sense::click()` is `CLICK | FOCUSABLE`, while the bare `Sense::CLICK` flag is
clickable and **not** focusable.

**The row arithmetic.** The tab strip carries the quick-access toolbar, the
tabs, the overflow affordance and the selector, and no one of them may eat the
row. The selector's track compresses rather than disappearing, and discloses
that it did as `ribbon-mode-selector-compressed`. A zero-width selector region
is not drawn at all.

### What each mode contains

| | Read | Review | Edit |
|---|---|---|---|
| Tabs | File, View | + Pages, Markup, Measure | + Edit, Tools |
| `edit_content` | no | no | **yes** |
| `author_markup` | no | **yes** | **yes** |
| `author_measure` | no | **yes** | **yes** |
| Page display default | Continuous | Single | Single |
| Off-page indicators default | off | on | on |
| Left dock | Pages, Bookmarks | Pages, Bookmarks | Pages, Bookmarks, Layers, Signatures, Fonts |
| Right dock | Comments, Forms, Document properties | Comments, Properties, Forms, Dimension groups, Document properties | Objects; then Properties, Comments, Forms, Redact, Dimension groups, Attachments, Document properties |
| Left width | 280 pt | 280 pt | 280 pt |
| Right width | 320 pt | 320 pt | 360 pt |

The tab lists are the manifest's (`shell::manifest::built_in`); the arrangements
and widths are `app::modes::defaults::spec`. The contextual **Format** tab is in
no mode's list and appears in all three, its items gated on `mode.edit_content`.

Edit's right side is **two stacks in one column** — Objects above, its detail
below, with the stack splitter between them. That is the master-detail pair, and
`ui-verify`'s `master_detail` check is what catches a panel put at the front of
the lower stack, which would open Edit with the document's keywords where the
selected object's properties belong.

**Ordering within a tabbed stack is load-bearing**, because a stack draws only
its active tab. The front of a stack is what the mode is *for*; the end is
"reachable in one click, invisible until asked for". That is why Redact,
Attachments, Dimension groups and Document properties sit at the tail of Edit's
detail stack, why Comments leads Read's right stack, and why Forms leads
Review's.

**Read** is the point of the feature: a PDF viewer with the inspection panels
available and nothing that authors anything. It is the mode a person is in when
someone sends them a drawing. The three panels it mounts on the right each earn
it — a comment list reads what somebody else wrote, a form field exists in order
to be filled, and reading a document's title is reading. **The stance Read takes
is about authorship, not about information.**

That stance is why Read grants **text** selection and refuses **object**
selection, which looks inconsistent until the two are asked what they are for.
Selecting text is how a reader quotes a drawing into an email. Selecting an
object is how an author picks an operand — and in Read there is nothing to
hand it to: the Format tab's items are gated on `mode.edit_content`, Delete and
the grips are refused, and the right dock mounts no Properties panel. A
selection no surface can act on is a highlight that means nothing, and
offering it would be the placeholder R9 forbids rather than a capability.

**Review** is the markup stance — comment, dimension, measure, cloud — plus the
**Pages** tab. Reviewing a drawing set means rotating a sheet to read it,
extracting the two pages you were asked about, and inserting a marked-up
revision, all of which is reviewer work. The stance that matters is *the page
content is not yours to alter*, and page operations do not alter content.

**Edit** is everything, and is what `RIBBON_IA.md` specifies.

**A panel is mounted only in a mode that can reopen it.** Closing a panel must
not be one-way, so a mode mounts a panel only when the panel's own command sits
on a tab that mode is shown. That is why `view.panel_forms` lives on View ▸
Panels rather than the Edit tab, why Comments needs the rail in Read, and why
Redact and Attachments appear in Edit alone.

### Page display, and the three tiers that decide it

`PageDisplay::default_for_mode` returns `Continuous` for `"read"` and `Single`
for every other mode id, including an unknown one — an unrecognised mode is not
evidence that the operator wants a different default. `Single` is not a legacy
mode and is not on a path to removal: a drawing sheet is a unit of work, and a
page boundary there is a deliberate step rather than an interruption.

What a document actually opens in is decided by three tiers, and the order is
the design:

| Tier | What it says | What it wins over |
|---|---|---|
| `viewer::remembered`, per document path | this drawing is read facing | everything |
| `Prefs::default_page_display`, global | I read drawings one page at a time | the per-mode default |
| `PageDisplay::default_for_mode` | Read is for reading | nothing |

The global tier is an `Option` and collapsing it would be a regression: `None`
means *fall through to the per-mode default*, and an operator who has never
stated a preference must keep getting it. *Nothing recorded* and *recorded as
single* are different states, and `settings.txt` omits the key entirely for
`None` rather than writing a value that would silently override the mode.

`OffPagePrefs::default_for_mode` is the same shape of rule for a cosmetic:
off-page indicators are off in Read and on elsewhere, overridable globally.

### `view.read_mode` is not `mode.read`

The two sit one tab apart, and they are orthogonal rather than duplicates:

| | `mode.read` | `view.read_mode` |
|---|---|---|
| What it is | a named workspace plus a capability set | a view stance: chrome on or off |
| What it changes | which tabs exist, which panels are mounted, whether the canvas authors anything | whether the ribbon and the docks are drawn at all |
| What it does not change | it still draws a ribbon, still mounts panels, still has a status bar | nothing about capability — an operator in Edit with chrome hidden can still author |
| Persistence | the mode and its arrangement, on disk, per mode | per session and per window |
| Where it lives | the mode selector | View ▸ Window, beside Full screen |

Pressing one does not touch the other. Review with the chrome hidden is a
meaningful state, not a contradiction. Because read mode composes with full
screen — no ribbon *and* no title bar — the way back out is published in two
places that are each other's blind spot: the window title, and a status-bar line
that outranks every other left-hand item, on the rule that **a sentence about
how to reach the interface outranks every sentence about the document**.

### The rule that makes this safe

**A mode changes what is *visible*. It never makes a visible control silently
inert.** In Read mode there is no Edit tab to click, so there is no click that
mysteriously fails.

That is the difference between a mode and a master editing toggle. A hidden
binary leaves the editing tools on screen and makes gestures quietly do nothing;
a mode **removes** the tools it disables. A genuine read-only stance is a
**document** state with a visible badge, never a hidden global toggle.

### Capabilities, and what a mode deliberately does not gate

`app::modes::capability::Capabilities` is **three independent booleans**, not an
ordered enum, each derived from tab membership: `edit_content` from the `edit`
tab, `author_markup` from `markup`, `author_measure` from `measure`. The three
built-in modes happen to form a ladder; that is a property of *that manifest*,
not of the type, and a customized manifest may offer any combination.

`Capabilities::for_mode` returns `FULL` when there is no shell, no active mode,
or an undeclared mode id, and `FULL` is also the `Default`. **Removing is the
opinionated act**, so every unknown case lands on full capability rather than
none; a restricting `Default` would turn every `..Default::default()` in a test
into a silent assertion about modes.

Not gated by any mode:

- **Filling a form field.** A read-only product fills forms; `canvas::forms`
  reads no mode and must not learn to. Filling is not authoring.
- **Pan, zoom, the hand tool, marquee *zoom*, Find, guides, rulers, grid.**
  Marquee-zoom shares its rubber band with marquee-*select* and is branched at
  release, so the gate sits on the intent — `capability::content_gesture`.
- **Selecting a page in the Pages panel**, and every panel's own contents.
- **Reading markup, and reading a dimension scale.**

**Selection and content editing are one flag**, deliberately, while both of
these hold: selection is the only route to the verbs that take it as an operand,
so a mode that could select but not edit would need every verb gated again
separately; and Read and Review mount no Objects panel and no full Properties
panel, so a selection there would be an outline and nothing else. The day canvas
selection reaches **annotations**, that is a different operand space and gets
its own capability, gated on `author_markup`.

**Chords.** `capability::offers_command` lets a chord reach a command the active
mode shows, or one that lives on no ordinary tab. A command whose own dispatcher
asks the real question — of the operand, per press — escapes that tab proxy,
because there the proxy answers a *different* question and gets it wrong. The
class is a named constant rather than literals at the comparison.

### Behavioural rules

1. **Switching modes never destroys work.** Read to Edit and back is a view
   stance, not a save boundary. It is held **structurally**: `app::modes` is
   handed a `DockState` and a `LayoutStore`, and neither can reach a document, a
   selection, an edit session or an undo stack. There is no path from the module
   to any of them, and a test asserts the consequence against a real open
   document anyway, so an edit that *adds* such a path fails a test rather than
   passes a review.
2. **The undo stack is not cleared, ever**, and undo continues to reach across a
   mode change. Anything else would make the mode a trap.
3. **Each mode remembers its own panel arrangement.** The outgoing mode's
   workspace is written from what is on screen; the incoming one's is restored,
   or its built-in default is used. Read to Edit to Read restores *your* Edit.
4. **Re-adopting the active mode is a no-op**, so a caller may drive
   `on_mode_changed` from the ribbon's mode every frame without checking.
5. **An undeclared mode id is declined**, with a `mode-unknown` trace, rather
   than quietly accumulating arrangements nothing can ever restore.
6. **The active mode is persisted beside the arrangement.** Storing only the
   arrangement is what makes an application come back in Read with an Edit
   layout in it.
7. **A pending, uncommitted gesture is committed or cancelled first**, with a
   prompt, exactly as Escape already does — a half-placed dimension or an open
   text edit is not hidden mid-flight.

### Workspace naming, and the upgrade path

A mode's workspace is `mode:<mode id>` — the **id**, not the label, so renaming
or translating "Review" does not orphan the arrangement behind it, and the
machine-owned entries stay filterable against workspaces an operator named
themselves.

**A panel added in a new release must not be born invisible.** An operator who
upgrades restores an arrangement saved before the new panel existed, and the
portable build tells them to keep `userdata/`, so the more carefully they
upgrade the more reliably they miss it. Neither obvious fix works: forcing the
default over a remembered layout discards the arrangement, and leaving it means
every panel from now on ships hidden.

So `egui-shell` records which panels existed when a workspace was written, and
the two states a layout otherwise cannot distinguish — *closed on purpose* and
*did not exist yet* — become separable:

| What `unseen_panels` answers | What is adopted |
|---|---|
| `Unseen::New(ids)` | exactly those ids — the ordinary case, from the second launch on |
| `Unseen::Unknown` | every registered panel, once — the file predates the record |

`adopt` skips panels already mounted, so the net effect of the `Unknown` arm is
the panels missing from a layout that predates the record. It can re-open a
panel closed before the record existed, once per install; the alternative is
that the upgrade case silently does nothing forever. The stamp is written
whether or not anything was added, so the next launch reports an empty `New`.

Defaults are filtered through the live `PanelCatalog` (`layout_for_build`), so
an id nothing registers is simply not mounted, whether what is missing is the
body or the command. `ABSENT_PANELS` is the register of intended-but-absent
panels and is currently empty. Writing the *intended* arrangement and filtering
it beats writing only what exists today, because the alternative leaves the
intent in a document nobody re-reads when the panel lands.

### What a mode change deliberately does not do

- **It does not change which tabs the ribbon shows.** That is `Mode::tabs` and
  the ribbon renderer. A mode's tab set and its panel arrangement are two
  different things that share a name.
- **It does not set the page display.** That is `crate::viewer`.
- **It does not decide the start mode.** `app::modes::start` does: it adopts
  the mode remembered from the last session, and falls back to the manifest's
  first mode when there is none or the remembered id cannot be honoured.

### What this does *not* do

It is not a permissions system. Read mode does not protect a document from
anything; a determined operator moves the selector. It is an
interface-complexity control. Anything stronger would need document-level
protection, which is a different feature.

### Specified here, not yet built

- **The mode badge**, and with it opening a document *into* a mode. A signed,
  encrypted or read-only-on-disk document should open in **Read** with the badge
  stating *why* — "signed: editing would invalidate the signature",
  "encrypted: this program cannot write encrypted documents yet". Opening in
  Read tells the truth earlier than letting the operator arm a tool that will
  later refuse. Today those conditions are disclosed in the status bar only.
- **The default-mode setting.** The application starts in the manifest's first
  mode. Someone whose job is drafting wants Edit on open; someone who receives
  drawings wants Read.

The selector, the per-mode tab sets, the keymap, the capability triple, the
per-mode arrangements and the per-mode page-display default are built.

---

## Part 2 — The panel areas

Inkscape 1.4 is the behavioural benchmark, reached by **driving it**. **No
Inkscape source is read or downloaded**: Inkscape is GPL-2.0-or-later, this
project is MIT, and the standing rule forbids both GPL code and GUI mimicry.
Nothing would be liftable anyway — Inkscape is GTK retained-mode C++ and this
shell is egui immediate-mode. Peer calibration comes from each product's own
user documentation and from direct observation of the running build.

### What the dock is

`crates/egui-shell/src/dock/` hosts multiple columns per side, vertical stacks
within a column, tabbed groups within a stack, user-draggable splitters, a
tab-overflow menu whose space is reserved before the first tab is measured,
collapse to an icon rail, tear-out to a window and back, and a layout that is a
plain value the application can save, name, restore and reset. Depth is fixed at
**side ▸ column ▸ stack ▸ tab**.

The application supplies three things and never has to supply more: a
`PanelRegistry` of `(id, label, tooltip)` strings, a starting `DockLayout`
value, and **one** body callback for every panel on every side — docked,
overflowed or floating. A `PanelId` is an opaque string; `egui-shell` never
learns what a PDF is (R7), and `tools/gates/check-shell-purity.sh` enforces the
negative half.

### It is built on `egui` directly, not on `egui_tiles`

`egui_tiles` is pinned in `[workspace.dependencies]` and **no crate depends on
it**, so Cargo never resolves it and it is absent from `Cargo.lock`. The command
that says so:

```sh
grep -c 'name = "egui_tiles"' Cargo.lock
```

`UI_TOOLKIT_PINS.md` carries the register row declaring the pin unused, and
`tools/gates/check-ui-toolkit-drift.sh` fails the build if a pin resolves to
nothing without being declared, and again if this document names an egui-family
version that is not the locked MAJOR.MINOR. The version numbers live in that
register, not here.

Four reasons owning the layout is right on the merits:

1. **A tiling engine's simplification fights persistence.** Passes that prune
   single-child containers and join nested linear ones mean *a column is not a
   durable identity*. A named workspace that silently loses the column the
   operator made is worse than no workspaces.
2. **Persistence had to be hand-written regardless.** The crate's serde support
   sits behind its default feature, which this workspace disables, and an owned
   schema does not persist internal arena handles that mean nothing across a
   restart. Here the schema **is** the model; there is nothing to translate.
3. **Its tab overflow was a defect to work around** — overflowing tabs hidden
   behind scroll arrows with `ScrollBarVisibility::AlwaysHidden`, failure mode
   #8 below, in the dependency itself.
4. **It ships no accessibility instrumentation.** Names had to be supplied from
   outside either way.

**What is given up:** drag-and-drop rearrangement of panels between
compartments, and free tiling of arbitrary depth. The layout *model* expresses
every arrangement, so the first is a gesture gap rather than a capability gap —
an operator reaches those arrangements through a saved workspace.

**The migration stays cheap if it is ever wanted:** `crate::layout`'s loader
builds an engine tree instead of a `DockLayout` and the persisted form does not
change, so adopting the dependency later costs the dock's internals and nothing
operator-visible.

### Capability register

Each row is a capability assessed against the pinned egui, with the mechanism
that delivers it or the reason it is absent.

| | Capability | Status and mechanism |
|---|---|---|
| **a** | Two or more columns per side | **Built.** `SideLayout` holds a `Vec<Column>`; `plan` resolves their widths and `plan::drag_boundary` moves the boundary between two, writing exactly two slice entries. |
| **b** | Vertical stacking within a column, resizable | **Built.** A `Column` holds a `Vec<Stack>` with draggable splitters, floored at `plan::MIN_STACK_HEIGHT` (80 pt). |
| **c** | Tabbing panels together within a stack | **Built**, with the overflow menu (`dock::tabs`, `dock::tab_menu`) that reserves its affordance before the first tab is measured. The two-pane cap that dodged failure mode #8 is retired. |
| **d** | Drag a panel between compartments | **Not built.** Tabs can be selected, closed and restored; they cannot be dragged into a different stack. What is missing is a gesture and a queryable geometry, not a structure: one `DockState` owns both sides, so a drag begun on the left is readable on the right with no tree spanning them. See "What is left to build". |
| **e** | Tear out to a floating OS window, and re-dock | **Built.** `dock::float` owns the state machine and `dock::floatwin` the window; the gesture is a command (`view.panel_float`, `view.panel_dock`), not a drag. |
| **f** | Persist the arrangement across sessions | **Built.** `crate::layout` — topology, splitter shares, active tab per stack, widths, visibility, and any floating panel's home. |
| **g** | Named, savable workspace layouts | **Built.** `layout::workspaces` — save, load, list, delete. Names are unique and matched exactly. |
| **h** | Collapse a side, with a visible way back | **Built.** `dock::collapse` — a tab on the inner edge collapses the side, and a rail at the window edge brings it back. |

Capability (g) is what the modes are: Read, Review and Edit are three registered
workspaces. `egui-shell`'s workspace store ships **no names at all** — an
application that wants three modes registers three, one that wants eleven
registers eleven, one that wants none never calls the module.

### Reset has scopes

A single global "reset layout" has a blast radius always larger than the
problem: an operator who only wanted the right dock back must not lose their
left one. `layout::reset` takes a `ResetScope` and the arrangement to reset
*to* — the shell has no opinion about what a good starting arrangement is,
because it does not know what any panel is for. `Action::ApplyResetLayout` with
per-scope checkboxes is the in-app command.

A reset restores **the arrangement** for the scope named and nothing else. It
does **not** touch saved workspaces: an arrangement is scratch work and a named
workspace is something the operator deliberately kept, so a reset that emptied
the workspace list would be a command whose name promises one thing and whose
effect is unrecoverable. An application wanting "forget everything" builds it
from `delete_workspace`, visibly, as its own command.

### Peer calibration

What the products in this class ship, and where the bar is. A dash means the
concept does not apply to that product's model.

| | Inkscape 1.4 | Blender | VS Code | Photoshop | Krita | Affinity |
|---|---|---|---|---|---|---|
| Dock left **and** right | yes | free tiling | yes | yes | yes, four edges | yes |
| Multiple columns per side | yes | — | no | yes | yes | not established |
| Tabs within a stack | yes | no | panel only | yes | yes | yes |
| Tear out to OS window | any | areas | editors only | yes | yes | yes |
| Collapse dock to icons | **no** | — | yes | yes | partial | yes |
| Named layouts | **no** | workspace tabs | profiles | yes, with shortcuts | yes | yes |
| In-app reset layout | **no** | yes | yes | yes, two-tier | not established | yes |

The benchmark product is worst-in-class on exactly the three rows this project
needs most. It has no named workspaces — the community workaround is copying a
state file aside and back, awkward because the file is rewritten on every exit,
which is an operator hand-rolling a workspace store out of file copies and
losing a race with the application's own writer. It has no in-app layout reset:
the documented route is to quit and delete that file. And per-dock
collapse-to-icons existed in an earlier release and was removed in a rewrite,
leaving an all-or-nothing hide-everything key as the substitute.

So the target is that product's **flexibility** plus Photoshop's and Affinity's
**layout management** — not the benchmark wholesale.

### Thirteen failure modes to design against

Numbered, because source comments cite these numbers.

| # | Failure | The design rule it implies |
|---|---|---|
| 1 | Ambiguous drag handle — an OS title bar and an application's own handle stacked vertically, where grabbing the wrong one silently does nothing. | One unambiguous grab affordance. Never nest a drag handle under OS chrome that looks the same. |
| 2 | Weak drop feedback, to the point that users conclude the feature does not exist. | Feedback must encode the *outcome*, not merely "valid target". |
| 3 | The widest hidden tab dictates minimum width — an inactive tab you cannot see holds the whole dock open. | Size a container to its **active** child; let inactive children scroll. |
| 4 | Minimum widths that consume a third of the screen. | Budget minimums per panel and test at 1280 pt wide. |
| 5 | No per-dock collapse, so the only escape is hiding everything. | Ship the icon rail; do not defer it indefinitely. |
| 6 | Layout not stable under window resize — un-maximise and re-maximise loses panel proportions. | Store proportional sizes with pinned minimums. Restore, do not recompute. |
| 7 | Coupled splitters — dragging one divider resizes every column. | A splitter affects its two neighbours only. |
| 8 | Tab overflow with no escape: past a handful of tabs the overflow *button itself* gets hidden, leaving no route to the hidden tabs. | The overflow affordance is reserved space, never the first thing squeezed out. |
| 9 | Crashes in the stacking path — docking under another dialog. | Fuzz the drop grammar. |
| 10 | Floating windows as second-class citizens: they minimise with the parent, sink behind other document windows, and swallow keyboard shortcuts when focused. | Decide deliberately whether torn-out panels are transient children or peers, and make shortcuts window-agnostic. |
| 11 | Focus-existing shows stale content — reopening a stacked dialog selects its tab but keeps rendering the previous one. | Selecting a tab must invalidate what is painted. |
| 12 | No named workspaces and no in-app layout reset, so the documented recovery is quitting and deleting a file. | Both are table stakes, not luxuries. |
| 13 | Chrome that can hide, beside a switch that has been suppressed — see "Auto-hide" below. | A hiding surface always leaves a permanent, hittable trigger, and a suppressed switch is only ever suppressed while a live replacement is proven to reach every panel behind it. |

Five are answered here by construction:

- **#1** — this application draws its own chrome; there is no second OS title
  bar to confuse with a panel header.
- **#3** — `plan::MIN_COLUMN_WIDTH` is a constant this crate owns (140 pt), and
  a test asserts no minimum consults a label, so an inactive tab cannot impose a
  width.
- **#8** — the tab bar reserves the overflow affordance *before* the first tab
  is measured, which is what retired the cap on panes per default group.
- **#10** — `show_viewport_immediate` keeps a torn-out panel inside the same
  `App::ui` call and the same action dispatcher, so keyboard handling is
  window-agnostic with no extra work.
- **#12** — the Read/Review/Edit selector *is* named workspaces, and
  `Action::ApplyResetLayout` with per-scope checkboxes is the in-app reset.

### R128 — the feedback loop this whole area invites

A panel whose size is **content-driven**, sitting beside a per-frame
fit-to-viewport zoom, is a feedback loop: the panel grows, the fit shrinks, the
fit changes the content, the panel grows. It was measured at 230 % to 224 % to
215 % across three consecutive frames. **Only `egui::Panel::exact_size` closes
it** — `default_width`, `min_width`/`max_width` and `resizable(false)` all fail
to. The rule needs a *direction bound*, not a guard: a "do not ask twice" latch
merely halves the oscillation's frequency.

A resizable dock is not a violation, because the loop is driven by **content**,
not by **the operator**:

- The dock's outer width comes from a number in the layout and changes only when
  a splitter is dragged, which is the explicit, discrete trigger.
- Nothing a panel body draws can change that number, **and the dock enforces
  that itself, because `exact_size` alone does not**: `Panel::show` takes its
  rect back from the frame's content union and, when that union is wider than
  `exact_size`, keeps the width by sliding the panel inward by the excess — the
  width stays exact, the position does not, and the central panel beside it
  shrinks by the same amount. So `draw_stack` draws each body in a child ui
  whose union is never merged into the side (`ui.new_child`, **not**
  `scope_builder`) and allocates the compartment's own rect in its place.
  `overflow_probe` is the tripwire that names any widget which still gets past
  that.
- `rail::WIDTH_PTS` is a constant at every rung, and nothing in that module can
  make it anything else. What shrinks under pressure is the **row budget**,
  never the width. A "helpful" change to `WIDTH_PTS.max(widest_label)` re-opens
  the loop: `Signatures` is a wider word than `Sigs`, so a font, theme or
  localization change would move the rail, which moves the canvas, which
  re-fits the zoom.

And the harder half is respected by omission: **the application's content area
is not inside a dock compartment.** The dock draws side panels; the application
draws its canvas in whatever remains.

### Tear-out

**The immediate-mode borrow does not block it.** `show_viewport_deferred`
carries `Send + Sync + 'static` on its callback, which would force app state
behind an `Arc<Mutex<..>>`. **`show_viewport_immediate` does not** — it takes
`FnMut` with no lifetime bound, so it can be called from inside `App::ui`
capturing `&mut self`. A torn-out panel therefore keeps the identical
`panel_body(&mut self, panel, ui, actions)` signature as the docked one, and the
one-dispatcher rule survives intact. Re-docking is simply ceasing to call it:
egui garbage-collects the unused viewport and eframe destroys the window. One
painter is shared across viewports, so page textures cost nothing extra.

**The application calls two things per frame:**

```rust
// between the ribbon and the canvas:
Dock::new().with_registry(registry).show(ui, state, &mut body);

// at the top of the frame, beside the dialogs:
Dock::new().with_registry(registry).show_floating(ctx, state, &mut body);
```

A child viewport must be opened from the **top-level frame**, for the same
reason dialogs are: opening one runs a complete nested pass for another window,
and doing that from inside a half-composed panel closure makes the parent's
remaining layout depend on what a different window did.

**Forgetting the second call is a silent failure** — panels laid out, publishing
correct rectangles, unreachable, every gate green. Two numbers catch it, and
they catch different things:

| Number | What it catches |
|---|---|
| `DockFrameReport::floats_undrawn` | `show_floating` was never called. The application asserts it is zero. |
| `FloatFrameReport::empty_bodies` | the call was made, the window opened, the header drew, and `body` allocated nothing. Measured from the body `Ui`'s `min_rect` **after** `body` returns. |

A blank window is R9 broken at the scale of a whole window: an operator can read
an absent button, but nobody can read a blank window with a title bar — it is
indistinguishable from a crash.

**A floated panel remembers where it came from, and docking it back puts it
there.** `FloatingPanel` carries a `DockHome`, recorded at the instant of the
float and serialized with the rest of the layout, so it survives a restart. The
alternative — re-mounting it "somewhere sensible" — is what makes re-docking a
thing operators avoid elsewhere: a panel that came from the second column of the
left dock and returns to the first column of whatever side has room has not been
docked, it has been re-mounted, and the arrangement has been quietly edited by a
command whose whole promise was to put something back.

`DockHome` is four `usize`s: **an address, not a handle**, because `DockLayout`
has no stable identity for a stack. An address can go stale — float a panel out
of a column, close everything else in it, and `normalize` prunes the column. The
address is then handled **by rebuilding, not by refusing and not by clamping**:
`DockLayout::dock_back` inserts a column and a stack when they are missing and
clamps only what is genuinely past the end, so the operator gets their panel
back *with its compartment*. Delegating to the permissive `mount` clamp instead
reads correct and is not — floating a panel that was alone in its stack prunes
that stack, so the home is out of range one frame later with nothing having
moved.

### Three strips whose names look alike

Each dock side can carry three distinct strips, each with its own published rect
name. Confusing them is how a driven check reads the wrong trace line.

| Trace name | What it is | Width |
|---|---|---|
| `dock.<side>.toolrail` | the permanent command rail — navigate controls, panel tabs, rotate | `rail::WIDTH_PTS`, 52 pt |
| `dock.<side>.rail` | the sliver a **collapsed** side leaves behind, and the only route back | `collapse::RAIL_WIDTH_PTS`, 16 pt |
| `dock.<side>.railtrigger` | the permanent peek trigger of an **auto-hiding** rail | `rail::PEEK_WIDTH_PTS`, 10 pt |

The collapse pair are mirror images and their chevrons must agree: open, the
collapse tab sits on the side's **inner** edge and points **out**, where the
panel is going; collapsed, the rail sits at the window edge and points **in**,
where the panel comes back from. Both raise an intent; neither writes the
layout. Flipping visibility mid-frame would change the width of a panel that has
already laid out inside it, so the apply phase runs once, after every side has
drawn.

### The command rail's fold ladder

It reuses the ribbon's three rungs — re-wrap, collapse, scroll — rather than
inventing a second answer to the same question one surface over.

| Rung | What changes |
|---|---|
| `Roomy` | everything: captions and labels. The resting state. |
| `Tight` | **the words go first** — presentation is cheaper than reach, and every control is still one click away |
| `Snug` | every `RailFold::Whole` group folds, in the **authored** order — not right-to-left, and never the group the author marked as the floor |
| `Cramped` | every `RailFold::PinArmed` group collapses to its armed row. The last thing a tool strip may stop doing is say what you are holding. |

Below `Cramped` the strip **scrolls**, in the application's `ScrollArea`. A rail
that cut its last entry off the bottom edge of a short window would be the
unreachable-control defect with a different cause.

**Two things never fold.** A `RailFold::Never` group — here the panel tabs,
whose being one click away is the rail's whole argument for existing. And **the
chevron**, appended after the ladder has run and never a candidate: everything
the rail dropped is behind it, so a rail that folds its own overflow control is
failure mode #8 eating itself.

### The smart selector

`view.smart_select` is the **last** row of the rail's `navigate` group,
mirroring View ▸ Navigate row for row. At `Rung::Cramped` the group folds to the
row whose `selected:` condition holds, **first match wins**, and the four tools
are listed first — so the pin goes to the armed tool and the toggle folds behind
the chevron, one click away. A test asserts both halves against the real fold
planner.

### Auto-hide, for the ribbon band and the rail

The model is **Microsoft Office's *Show Tabs***, the middle of its three Ribbon
Display Options: the tab strip stays on screen permanently and only the band
goes; touching the strip brings the band back **over** the document; it goes
when the pointer leaves. The rail's version is VS Code's activity bar one
surface over — a permanent narrow edge, and a wide body that overlays rather
than displaces. One state machine, `egui_shell::peek`, serves both.

Three properties are carried across deliberately, each answering a way this is
usually got wrong:

| Property | The failure it prevents |
|---|---|
| The trigger is **permanent** — a tab strip, a sliver — and never hides | The full auto-hide-everything setting is the one people get stuck in, because the thing you must touch to leave it is invisible. **A mode you cannot leave is a trap.** |
| The body **overlays**, it does not displace | A canvas that resizes on hover moves every coordinate under the pointer as the pointer approaches. |
| A reveal can only be **started** by the trigger | R128. The overlay term is conjoined with last frame's state, so it can only *keep* a reveal — it cannot start one, and there is no cycle to oscillate around. |

The invariant, stated rather than described:

```text
revealed(n+1)  =  in(trigger)  or  ( revealed(n) and in(overlay(n)) )
```

`in(trigger)` is the only way in, and the trigger rectangle is required to be
**independent of the overlay** — the ribbon's is its tab strip, whose height
comes from the theme metrics and the mode selector rather than from the band;
the rail's is a reserved sliver. Nothing the overlay does can move it, so
revealing cannot cause revealing. On a frame where the surface is hidden the
overlay term is false whatever the pointer is doing, so the state is monotone
decreasing while the pointer is still.

**The floor fails open.** `Peek::resolve` refuses a trigger thinner than
`Peek::MIN_TRIGGER_PTS` (8 pt) in either axis by reporting the surface *shown*
rather than hidden. A trigger too small to hit is how a surface becomes
unreachable; the worst case of failing open is chrome the operator wanted
hidden, which is visible and one setting away. Keyboard focus inside the overlay
also holds it open — a keyboard is not a pointer.

Both settings are **off by default**, are persisted in `settings.txt` as
`ribbon_auto_hide` and `rail_auto_hide`, are checkboxes in Settings ▸ Display
under one heading, and have a command each — `view.ribbon_auto_hide`,
`view.rail_auto_hide` — on View ▸ Window. Registering the commands is what puts
the pair in the keymap, the menu document and the operator's own customization
(R8); a setting reachable only from a Settings window is a capability most
operators never find. Neither renders **pressed**: Office's own ribbon-display
control does not sit lit while *Show Tabs* is chosen, and a ribbon with no band
under its tabs, or a rail shrunk to a chevroned band, are not states an operator
can be uncertain about.

### Failure mode #13 in full, and the three mechanisms that hold it

The rail is the *only* route to `markup.comments` in Read mode: that command is
on the Markup tab, which Read does not show, and `RIBBON_IA.md` P1 forbids a
second tab placement. Suppressing the dock's tab strip removes the other switch.
If the rail could then *disappear*, a panel mounted behind another in that stack
would be reachable by nothing at all.

A live trace shows how close the two switches sit — two rows of the same switch,
three points apart, at the same height:

```text
ui-rect name=rail.tabs.view.panel_pages     rect=[[3.0 130.0] - [49.0 164.0]]
ui-rect name=dock.tab.view.panel_pages      rect=[[52.0 129.7] - [100.3 153.7]]
ui-rect name=dock.tab.view.panel_bookmarks  rect=[[102.3 129.7] - [180.8 153.7]]
```

1. **A hiding rail is never gone.** It reserves `rail::PEEK_WIDTH_PTS` of
   permanent, chevron-marked edge — wider than `Peek::MIN_TRIGGER_PTS` — and
   publishes it as `dock.<side>.railtrigger` on **every** frame, hiding or not.
   `a_hiding_rail_always_publishes_a_trigger_wide_enough_to_hit` asserts the
   published rectangle's width, not the constant, across a series of side
   widths.
2. **The tab strip is suppressed only when all three** of: a rail is configured
   for that side; the rail was **actually drawn** this frame (`resolve_width`
   returns zero when 52 pt would leave the panel body below
   `plan::MIN_COLUMN_WIDTH`, so the rail is *absent rather than squeezed* —
   asking "is a rail configured" instead would drop the tab strip at exactly the
   widths where the rail is not there); and the application's `with_rail_reach`
   predicate answers `true` for **every** panel in the stack — not the active
   one, not most of them, because the panel that is behind is precisely the one
   that needs the second route.
   `a_side_too_narrow_for_the_rail_keeps_its_tab_strip_at_every_width` walks a
   width series, asserts the implication *suppressed implies a rail was drawn*,
   and asserts that the series actually straddled the threshold.
3. **Raising a specific panel still works, and so does closing it.** A rail row
   dispatches the panel's own command, `PdfcerApp::toggle_panel` asks
   `is_on_screen`, and that is `false` for a panel mounted **behind** another
   tab — so the press activates it rather than closing it. Closing is the same
   control pressed again, and the tab's own close control is a second route to a
   verb that has one.

When a strip is suppressed, `draw_stack` sets its bar height to zero and counts
it in the frame report's `tab_strips_suppressed`.

### The reach predicate is an extension point, not a pattern match

R7: `egui-shell` may not learn what a PDF is. The dock holds opaque `PanelId`s
and the rail holds opaque command ids, and the map between them is application
knowledge — the Fonts panel is `file.fonts` and the Comments panel is
`markup.comments`, neither of which a `view.panel_*` pattern finds. The
application supplies `Dock::with_rail_reach`, derived from the **live manifest**
including any operator overlay, so a rail somebody customized stops suppressing
strips over panels they removed from it.

### Floating surfaces: the distinction that has to hold

`RIBBON_IA.md` P5 states the invariant: **nothing floats over the canvas.** Tool
options live in the dock, not in canvas-anchored overlays, because accept and
reject boxes that moved on every zoom were a reported defect.

Capability (e) does not contradict it, and the distinction is the whole reason:
**a panel the operator deliberately tears out is not the same thing as a box the
application decides to float.** The two are separate questions and they get
separate answers.

- **Operator initiative — allowed, and built.** `view.panel_float` and
  `view.panel_dock` sit on a panel's tab menu; the mechanism is under
  "Tear-out" above.
- **Application initiative — never.** This program opens no surface over the
  canvas the operator did not ask for. There is no setting for it, because there
  is no behaviour to switch off: a control whose only value is the one already
  in force is a control that does nothing whichever way it is set. Building the
  setting would mean building the behaviour first, and the behaviour is the
  thing that was objected to.

The same distinction governs spring-loading a document tab: it is gated on a
page drag being in flight, because changing documents because the operator
paused on their way to the ribbon is the application taking initiative nobody
asked for.

### Verification

A passing unit test is not evidence that a panel is reachable. Layout and
clipping defects have exactly one oracle: a rendered screenshot, which is
`tools/ui-verify`'s job. Its scripts are written in **document space**
(`tools/ui-verify/src/coords.rs`), because every screen coordinate here is
variable at runtime and a stale scripted coordinate is symptom-identical to a
broken screen-to-document conversion.

```sh
cargo test --workspace
cargo run --release -q -p ui-verify -- --list
bash tools/gates/run-all.sh
```

Driven checks that bear on this document's subject: `master_detail` (Edit's
Objects and Properties are adjacent stacks in one column, not one tab away from
each other), `panel_float` (a floated panel gets a real OS window *and* its body
is drawn inside that window's viewport), `comments_census` (the Comments panel
is reachable even when seeded behind another tab), `chords`, and `read_mode`,
`read_mode_chrome` and `read_mode_exit` for the chrome stance. `left_rail` is
written and not yet run; its own module header says so.

### What is left to build

**Dragging a panel between compartments** — capability (d). Two things are
missing and neither is structural:

1. **A queryable geometry.** `plan` is scalar-only and every rect the dock
   computes is a local in `dock::mod`, escaping only as a stringly-named
   `RectReport`. A drop needs to ask *which stack is under this point, and where
   would the panel land*, which needs the addresses and their rects retained for
   the next frame's hit test.
2. **A drop grammar** — take a panel from an address, insert it at a target that
   may be an existing tab position, a new stack splitting a column, or a new
   column. The preview is then the same arithmetic run against a clone, so what
   is highlighted is what will happen, which is what failure mode #2 demands.

**One `DockState` owns both sides**, so a drag begun on the left is readable on
the right with no tree spanning them, and the canvas need not move inside a
resizable pane. R128 is therefore **not** a prerequisite for this — the earlier
reading that it was came from `egui_tiles`, where drag identity is scoped to a
`Tree` and two docks are two trees. `UI_TOOLKIT_PINS.md` states the general
form: a feasibility verdict that turns on what `egui_tiles` can do is a verdict
about a library this shell never links.

**Drag-to-tear**, as a gesture layered on the existing float model. Starting
with a stationary command rather than a drag was deliberate: it captures most of
the value at a fraction of the cost, dodges the focus-gated `StartDrag`
primitive, and sidesteps failure mode #1. The model's question is *where is this
panel and where did it come from*; a drag is only one more way of answering it,
and adding one changes nothing underneath.
