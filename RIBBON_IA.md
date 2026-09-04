# pdfcer GUI — Ribbon Information Architecture

**Status:** proposal, 2026-08-12
**Scope:** where every user-reachable command lives, and why.
**Companion documents:** `GUI_ROADMAP.md` (when each part gets built),
`DEFECTS.md` (what is broken today), `mockups/` (what it looks like),
**`MODES_AND_PANELS.md`** (the Read/Review/Edit selector and the
flexible panel system — an operator addition of 2026-08-13 that layers
on top of everything here).

> **Amendment, 2026-08-13.** The seven-tab layout below describes
> **Edit** mode. `MODES_AND_PANELS.md` adds a three-position selector at
> the far right of the tab row; **Read** and **Review** are subsets of
> this layout, not different ones. No command specified here moves, and
> nothing here is superseded — the selector governs which tabs and
> panels are *present*, not where a command lives when it is.

---

## 1. What this document is for

pdfcer's ribbon today has six tabs and twenty groups. The grouping is
principled — there is a real rule set behind it, written down in
`crates/pdfcer-gui/src/ribbon.rs` — but the result does not match what a
user reaching for a command expects to find, and three of the six tabs
are underfilled to the point of looking unfinished.

This document specifies a replacement layout: seven tabs, each with a
stated question it answers, each command assigned to exactly one home,
and every currently-existing command explicitly migrated. It is written
so that the ribbon could be rebuilt from this document alone, including
the reasoning behind each placement, which is the standard the rest of
this project's documentation is held to.

It is a *specification*, not a wish list. Every command below is marked
with where it exists today:

| Mark | Meaning |
|---|---|
| **G** | Exists in the GUI now. Moving it is pure re-parenting. |
| **C** | Exists in `pdfcer-core` and/or `pdfcer`, but has no GUI surface. Needs a shell, not an engine. |
| **N** | Does not exist anywhere. Needs building. |

The **C** rows matter disproportionately: they are the cheapest possible
wins, because the hard half is already written and tested.

---

## 2. Principles carried forward

These come from the existing codebase and are kept, because they are
good and because breaking them silently would be worse than the problem
they cause.

**P1 — One command, one tab.** No command appears on two ribbon tabs.
The existing rule (`ribbon.rs:64-75`) exists because Pass 47.1 found
that mirroring undo/redo onto every tab was the reason the ribbon
rendered only the active band and undo became unreachable from the
Measure tab. A test asserts every group has exactly one owning tab
(`ribbon.rs:678`). Keep both the rule and the test.

**P2 — The ribbon picks the activity; the sidebar holds that activity's
controls.** Codified at `ribbon.rs:218-220`. The Measure tab arms
"Linear ce dimension"; the group picker, scale entry, number format and
drafting standard live in the Tool Options pane. This is correct and it
is why the Measure tab has one group. The fix for an underfilled tab is
never to move sidebar controls up into it.

**P3 — No placeholders.** An unavailable capability renders nothing, not
a disabled stub (`ribbon_ui.rs:241-244`). Greying is reserved for
*temporarily* unavailable — no document open, document encrypted, undo
stack empty — and is always explained on hover.

**P4 — Group captions are mandatory.** Every group draws its caption
beneath its controls, enforced by routing all groups through one closure
(`ribbon_ui.rs:76-117`) after two groups were found rendering without
one.

**P5 — Nothing floats over the canvas.** Stated at `FEATURES.md:182`.
Tool options live in the dock, not in canvas-anchored overlays, because
accept/reject boxes that moved on every zoom were a reported defect.

### One amendment is proposed

**P1 as written also forbids the Quick Access Toolbar and status bar
from carrying a command that appears on a tab.** That is why the File
tab today has no Open and no Save — they live only in the QAT — and why
the View tab has no zoom controls, because zoom lives only in the status
bar.

That reading is too strong, and it is the direct cause of two of the
worst discoverability failures in the product. A user who wants to open
a file looks under File. A user who wants to change how the page is
displayed looks under View. Finding neither there teaches them the
ribbon is not where commands live.

**Proposed amendment — P1a:** *the QAT and the status bar are shortcut
surfaces, not tabs. A command may appear on exactly one tab and
additionally on the QAT and/or the status bar.* This is precisely how
Office's QAT is defined, and it does not reintroduce the Pass 47.1
defect, which was about a command being reachable from **no** tab when
the band collapsed — the opposite failure.

With P1a, `RibbonTab::groups()` stays the single source of truth for
tab ownership and the existing uniqueness test is unchanged; the QAT and
status bar are simply outside its domain, which they already are.

---

## 3. What is wrong with the current layout

Observed directly by driving the built release binary (`evidence/`),
not inferred from source.

**The View tab contains no view controls.** It has two groups: `Panels`
(sidebar, Bookmarks, Layers, Signatures, Fonts, Objects) and `Show`
(annotations, points). There is no zoom, no page layout, no view
rotation, no read mode, no full screen. Read mode and full screen have
**no ribbon control at all** — they are keyboard-only (Ctrl+H, F11) on a
tab literally named View. This is the single most confusing thing in the
current ribbon.

**Page operations are hidden.** Insert, delete, extract, reorder,
split and merge exist and work, but live in the thumbnail rail's
selection action bar and in a `Tools ▸ Batch` pane. Nothing on any
ribbon tab says "pages". A user who wants to delete page 7 has no path
that starts at the ribbon. The Edit tab's `Pages` group contains only
Rotate left / Rotate right, which makes the absence louder — it looks
like page ops were considered and rejected.

**The File tab is a junk drawer.** Properties, Copy this page's text,
Copy the whole document's text, Export DXF, Print, Reset layout,
Settings, keyboard shortcuts. Text copying is not a file operation. Panel
layout reset is not a file operation. Meanwhile there is no New, no
Recent, no Close, and — per §2 — no Open and no Save.

**Three tabs are underfilled.** Measure has one group and four controls.
Tools has three groups of one control each. View has two groups. On a
1936 px window each of these leaves 1200–1700 px of empty band. An empty
ribbon band reads as an unfinished program regardless of how much
capability sits behind the controls that are there.

**Two Edit-tab buttons are unlabelled icons.** The `Pages` group's
rotate buttons carry no text, and the `Content` group's buttons read
`Aa`, `I⁺ Aa` and `Obj`. `Obj` is not a word. These are the primary
content-editing tools and they are the least legible controls in the
application.

**Group captions are set in a colour that is nearly invisible** in the
default Quiet theme — the same defect that hides the dock tab labels and
the Settings dialog headings. See `DEFECTS.md` §1. It affects the ribbon
too: the words `Document`, `Clipboard`, `Export`, `Print` under the File
tab's groups are present and technically legible, but only just.

---

## 4. The proposed tab set

Seven tabs. Each keeps the existing idiom of a one-line question it
exists to answer.

| # | Tab | The question it answers |
|---|---|---|
| 1 | **File** | What do I do with the file as a whole, or with pdfcer itself? |
| 2 | **View** | What is on my screen, and how is the page laid out? |
| 3 | **Pages** | What am I doing to the set of pages? |
| 4 | **Edit** | What am I changing about content that is already there? |
| 5 | **Markup** | What am I adding for someone else to read? |
| 6 | **Measure** | What am I measuring, and in what units? |
| 7 | **Tools** | What do I run across files, or configure once? |

Plus **one contextual tab**:

| Tab | Appears when |
|---|---|
| **Format** | A markup, dimension, image or vector object is selected |

### Why seven, and why these

Six tabs was one too few for the amount of capability behind them, and
the sixth was carrying two unrelated jobs. The change is essentially:
split page operations out of hiding into their own tab, give View the
view controls its name promises, and let File become an actual file tab.

`Review` is renamed **Markup**. What lives there is markup authoring —
shapes, notes, stamps. "Review" promises a review *workflow*: compare
revisions, resolve comments, track changes. pdfcer does not have that
yet, and when it does, it will want the name. `Markup` is also the term
this project's audience uses; Bluebeam and every drafting office call it
that.

`Format` as a contextual tab is how the P2 tension resolves. Today,
selecting a placed markup gives you nowhere to change its colour,
because the Markup tab's colour swatch sets the colour of the *next*
markup. A contextual tab is the standard answer and it is already named
in the code as an unbuilt slice (`ribbon.rs:42-52`).

---

## 5. Tab specifications

Notation: groups are `**Group**`, commands are listed with their status
mark. `⌄` means the control is a split button or dropdown.

---

### 5.1 File — *what do I do with the file as a whole, or with pdfcer itself?*

| Group | Commands | |
|---|---|---|
| **File** | New (blank) | **G** *(2026-08-14; `Ctrl+N`)* |
| | New from template… (page size) | **N** — see `manifest::PLANNED` |
| | Open… | **G** *(also QAT)* |
| | Recent ⌄ | **N** |
| | Close | **G** *(currently switcher-only)* |
| **Save** | Save | **N** — still absent, and the reason is unchanged: an in-place `Save` promises overwriting the operator's file, which needs autosave and crash recovery first. ★ **But `Save a copy` is no longer blocked and shipped 2026-08-14** — it had been registered, on the QAT and bound to Ctrl+S with no dispatch arm for the whole project. The engine was never the blocker: `to_incremental_bytes`/`to_full_bytes` both take `&self`. When in-place Save does land it inherits standing instruction 5 — *Read may produce a new document; it may not modify this one* — which is the rule that is currently vacuous precisely because every save is already a copy. |
| | Save a copy… | **G** *(also QAT)* |
| | Revert | **N** |
| **Export** | Export DXF… | **G** |
| | Export image… (PNG/JPEG/TIFF, DPI picker) | **C** |
| | Export text… | **C** |
| | Export form data ⌄ (FDF / XFDF / CSV) | **G** *(currently in Forms pane)* |
| **Print** | Print… | **G** |
| | Imposition… (n-up / booklet / poster) | **C** |
| **Document** | Properties | **G** |
| | Fonts | **G** *(currently View ▸ Panels)* |
| | Security | **N** |
| **pdfcer** | Settings… | **G** |
| | Keyboard shortcuts | **G** |
| | About | **N** |

~~**Moved off this tab:** `Copy this page's text` and `Copy the whole
document's text` go to **Edit ▸ Clipboard** — they are content
operations, not file operations.~~ `Reset layout…` goes to **View ▸
Window** — it resets panel geometry, which is a view concern.

★ **REVERSED by operator decision, 2026-08-14, and corrected here on
2026-08-25.** The two text-copy commands are `file.copy_page_text` and
`file.copy_document_text`, on **File ▸ Export**, and have been since that
date. This paragraph went on saying otherwise for eleven days, which is the
defect: a settled document that disagrees with the build is worse than an
unsettled one, because it is read as authority.

What the sentence above got wrong is that *a content operation is not
necessarily an **authoring** operation*. Copying reads the page and writes to
the clipboard; it cannot change a byte. What made the difference visible was
the chord/mode gate refusing `Ctrl+Shift+C` in Read — a mode whose whole
standard is Acrobat Reader, which copies text. It is the same rule §5.7 states,
and this is the second of its three instances.

**Why Fonts moves here from View:** the Fonts panel answers "what is
inside this file", not "what is on my screen". It sits with Properties
and Security as document-level inspection. This is a genuine
improvement in the current build — Fonts is excellent and nobody will
find it under View ▸ Panels.

**Note on Save.** A `Save` command that overwrites in place cannot ship
before autosave and crash recovery exist; that dependency is already
documented (`FEATURES.md:62`, `main.rs:49-54`). Until then the group
holds `Save a copy…` alone and `Save` does not render at all, per P3 —
it is not greyed out with a tooltip, it is absent.

---

### 5.2 View — *what is on my screen, and how is the page laid out?*

| Group | Commands | |
|---|---|---|
| **Page display** | Single page | **G** *(current behaviour, now named)* |
| | Continuous | **N** |
| | Facing | **N** |
| | Facing continuous | **N** |
| **Render** | Strategy: Whole page · Tiled progressive | **N** |
| | Raster scale ⌄ (quality) | **partial G** — exists as a constant |
| | Settle delay | **partial G** — `ZOOM_SETTLE` is a constant |
| | Thin lines | **N** |
| | Antialias ⌄ (text / vector) | **N** |
| **Rotate view** | Rotate view left / right | **N** |
| **Zoom** | Zoom to selection | **N** |
| | Zoom to region (marquee) | **N** |
| | Actual size · Fit page · Fit width | **G** *(status bar; P1a mirror)* |
| **Display** | Thin lines | **N** |
| | Show annotations | **G** |
| | Show points | **G** |
| | Rulers · Grid · Guides | **N** |
| **Panels** | Sidebar ⌄ | **G** |
| | Pages · Objects · Bookmarks · Layers · Signatures · Comments · Forms | **G** |
| **Window** | Read mode | **G** *(keyboard-only today)* |
| | Full screen | **G** *(keyboard-only today)* |
| | Floating panels: Off · Allowed | **N** — *default Allowed* |
| | App initiative: Never · Ask · Allowed | **N** — *default **Never*** |
| | Save workspace… · Load workspace ⌄ | **N** |
| | Reset layout… | **G** *(from File)* |

**Page display is the important entry.** Per your correction: single
page stays the **default**, because paging one drawing sheet at a time
is the right model for drafting review and the existing navigation is
good. Continuous becomes a *mode you choose*, sitting beside it, for the
case where the document is a 40-page specification rather than a sheet
set. The four modes are radio-style — exactly one is active — and the
choice persists per document, not globally, so opening a drawing set
does not inherit a report's setting.

This is a larger build than it looks; `viewer.rs` holds a single
`page_index` and `object_provider.rs:392-399` returns nothing for any
page but the current one. See `GUI_ROADMAP.md` Phase 4.

**The Render group is an operator decision, 2026-08-12.** pdfcer caches
one whole-page texture and scales it with linear filtering during the
150 ms settle. Measured in use on a large drawing, that is *smoother*
to pan and zoom than the comparison product's progressive tile
rendering — no seams, no piece-by-piece fill-in — at the cost of a full
re-raster once motion stops. Those are two legitimate trades, not a
better and a worse, and which one wins depends on the sheet and the
machine.

So the strategy becomes a **choice on the View tab**, with whole-page as
the default because it is what measured better. This is R169 applied to
rendering: where the right answer is genuinely undetermined, pdfcer
states the trade and lets the operator pick, rather than deciding
quietly. `ZOOM_SETTLE` (`main.rs:367`) and the raster-scale multiplier
are constants today and become the two knobs beside it.

**Zoom on this tab does not duplicate the status bar in spirit.** The
status bar keeps the continuous controls a user reaches for constantly
(−/%/+, fit toggles). The View tab adds the two *targeted* zooms that
have no status-bar home — zoom to selection and marquee zoom-to-region —
and mirrors the three named zoom levels under P1a so that a user looking
under View for zoom finds zoom.

---

### 5.3 Pages — *what am I doing to the set of pages?*

| Group | Commands | |
|---|---|---|
| **Insert** | Insert blank | **C** |
| | Insert from file… | **G** *(Tools ▸ Batch pane)* |
| | Insert scan | **N** |
| **Organise** | Delete | **G** *(thumbnail rail)* |
| | Extract… | **G** *(thumbnail rail)* |
| | Replace… | **N** |
| | Move up / Move down | **G** *(thumbnail rail)* |
| | Split… | **G** *(Tools ▸ Batch pane)* |
| | Merge into this document… | **G** *(Tools ▸ Batch pane)* |
| **Transform** | Rotate left / right | **G** *(Edit ▸ Pages)* |
| | Crop… | **N** |
| | Resize… | **N** |
| **Stamp** | Watermark… | **N** |
| | Header & footer… | **N** |
| | Bates numbering… | **N** — *see `DEFECTS.md` §2* |

Every command here operates on **the current document's page set**, and
every one of them respects the thumbnail rail's current selection when
there is one. That is the tab's organising rule and it is what
distinguishes it from Tools: Pages changes *this* document, Tools
produces *new* files.

The thumbnail rail keeps its selection action bar. That is not a P1
violation — the rail is a panel, not a tab, and a selection-scoped
action bar next to the selection is correct. But the ribbon becomes the
discoverable path, and the rail becomes the fast path.

---

### 5.4 Edit — *what am I changing about content that is already there?*

| Group | Commands | |
|---|---|---|
| **Content** | Edit text | **G** — *relabel from `Aa`* |
| | Add text | **G** — *relabel from `I⁺ Aa`* |
| | Edit objects | **G** — *relabel from `Obj`* |
| **Insert** | Image… | **G** *(drag-drop only today)* |
| | Shape ⌄ | **N** |
| **Arrange** | Align ⌄ · Distribute ⌄ | **N** |
| | Bring forward / Send backward | **N** |
| | Group / Ungroup | **N** |
| | Flip horizontal / vertical | **N** |
| **Clipboard** | Cut · Copy · Paste · Paste in place | **N** *(object clipboard)* |
| | ~~Copy page text · Copy document text~~ | ★ **NOT HERE — `file.copy_*`, on File ▸ Export.** Operator decision 2026-08-14; see §5.1. *Copying is not authoring* |
| **Forms** | ~~Fill form~~ | ★ **NOT HERE — `view.panel_forms`, on View ▸ Panels.** Operator decision 2026-08-14: Read fills forms, as Acrobat Reader does, and Read is shown File and View alone. It also stopped being a verb and became a **panel toggle**, which is why it sits with the other panel toggles rather than in a group of its own. *Filling is not authoring* — Edit ▸ Forms keeps create, manage and flatten |
| | Create field ⌄ | **G** |
| | Manage fields | **G** |
| | Flatten | **G** *(in Forms pane)* |
| **Protect** | Redact ⌄ (mark page / by text / by pattern) | **G** |
| | Apply redactions | **G** |
| | Sanitise… | **N** |

**The three `Content` buttons must get real labels.** `Aa`, `I⁺ Aa` and
`Obj` are the primary editing tools and are currently the least legible
controls in the application. `Edit text`, `Add text`, `Edit objects` —
with the icons kept.

**The `Editing on` master toggle is removed. Operator decision,
2026-08-12: "make it work the same way other programs do."**

No mainstream editor has a global editing switch. Acrobat, Bluebeam,
Word and Illustrator all work the same way: selection and Delete are
always live, and picking a tool arms *that tool* until you press Escape
or pick another. There is no state in which a click does nothing without
the application saying so.

Concretely:

- `editing_enabled` (`main.rs:3235`, `3624`) is deleted, along with the
  ribbon toggle at `ribbon_ui.rs:721-736` and
  `ui_text::authoring_disabled_note()`.
- The four sites that currently gate on it — `main.rs:7095`, `8169`,
  `8194`, `16920` — lose the gate. Nothing replaces it: an unarmed
  canvas already does modeless select-and-delete, and every authoring
  gesture already requires its tool to be armed.
- **This supersedes `DEFECTS.md` D6.** That defect was "review mode does
  not actually block object deletion". With review mode gone there is
  nothing to enforce, so D6's fix becomes "delete the check sites"
  rather than "add the missing one". If a genuine read-only mode is ever
  wanted it should be a *document* state (open read-only, encrypted, or
  signature-locked) with a visible badge, not a hidden global toggle.

It was already `true` by default, and it is **not** what breaks
click-then-Delete (`DEFECTS.md` D1). Removing it is about eliminating a
class of failure, not fixing a current one.

---

### 5.5 Markup — *what am I adding for someone else to read?*

| Group | Commands | |
|---|---|---|
| **Shapes** | Rectangle · Ellipse · Line · Arrow | **G** *(Arrow is `Arrow line`)* |
| | Polyline · Polygon | **N** |
| | Cloud | **N** — revision clouds; AEC table stakes |
| | Ink (freehand) | **N** |
| **Text markup** | Highlight | **G** *(`Highlight band`)* |
| | Underline · Strikeout · Squiggly | **N** |
| **Notes** | Text box | **G** |
| | Sticky note | **G** |
| | Callout | **N** |
| | Stamp ⌄ | **G** *(`Draft stamp`; needs a gallery)* |
| **Style** | Colour · Line width · Fill · Opacity | **partial G** — colour only |
| **Comments** | Comments panel | **G** |
| | Clear page · Clear all | **N** |

Six of ten markup kinds are missing (`canvas.rs:255-262` defers Ink,
Polygon, PolyLine, Underline, StrikeOut, Squiggly to "slice 3"). Cloud
is not in that list and matters most for this audience.

**The `Style` group sets defaults for the next markup.** Changing an
*existing* markup's style happens on the contextual **Format** tab. Both
must exist; today only the first does, which is why a placed markup
feels final.

---

### 5.6 Measure — *what am I measuring, and in what units?*

| Group | Commands | |
|---|---|---|
| **ce dimension** | Linear | **G** |
| | Aligned | **partial G** — constraint exists, not a separate tool |
| | Angular | **N** |
| | Radius / Diameter | **G** |
| | Two-line | **C** — *core + CLI done; the gesture exists in the OLD shell and must be salvaged (see the correction below)* |
| **Quantity** | Distance · Perimeter · Area | **N** |
| | Count | **N** |
| **Scale** | Set scale | **G** |
| | Calibrate from a known length | **partial G** |
| | Manage dimension groups… | **G** |
| **Takeoff** | Schedule panel · Export CSV | **N** |

The **Scale ▸ group** model — named groups carrying a shared scale and
drafting standard — is genuinely better than what the comparison product
does, and nothing here should dilute it. Area and Angular are the
conspicuous absences for anyone doing takeoff on a drawing.

**Two-line dimensioning is a C row and is at the top of the project's
own queue** — core and CLI shipped and measured. It is a shell-only task.

> **★ Correction, 2026-08-14.** This paragraph used to end *"`pick_line`
> has no caller"*, and the same sentence was in `FEATURES.md`,
> `HANDOFF.md`, `SALVAGE.md` and `shell/manifest/mod.rs`'s `PLANNED`
> entry. **It is false, and it was false when written.** The old shell
> calls `pick_line_in_page` at `D:\Dev\pdfce\crates\pdfce-gui\src\main.rs:23564`
> and takes the pick at `:23592`; the whole gesture is there — hover
> highlight, picked-pair overlay, the verdict disclosure, Escape to clear
> — with `TwoLinePick` in `measure_tool.rs:361` behind 814 lines of tests,
> and pdfcer's own `docs/FEATURES.md` marks the row `gui [x]`.
>
> The likely origin is a misread of pdfcer's `ROADMAP.md:2778`, which
> explains *why* `pick_line_in_page` exists, one paragraph above the
> commit that added its caller.
>
> What changes: the caller that is missing is **ours**. The new shell has
> no measure tool at all — `CanvasTool` has two variants, no `measure.*`
> command has a dispatch arm, and nothing in `crates/pdfcer-gui` mentions
> `linepick`. So this is a **salvage** of `measure_tool.rs` (Class A,
> 1,230 lines) plus the canvas hosting, not a one-line hookup, and the
> "cheapest real feature in the backlog" claim that rode on it is
> withdrawn. Recorded rather than deleted, because a wrong claim repeated
> in five documents is worth knowing about.

---

### 5.7 Tools — *what do I run across files, or configure once?*

| Group | Commands | |
|---|---|---|
| **Batch** | Merge files… | **G** |
| | Split files… | **G** |
| | Batch print… | **N** |
| **Compare** | Compare documents… | **N** |
| **Fonts** | Font folders… | **G** |
| | Embed fonts · Unembed fonts | **G** *(Fonts pane)* |
| ~~**Recognise**~~ | ~~OCR…~~ | ★ **MOVED to File ▸ Recognise, 2026-08-14 — `file.ocr`.** This section specified Tools, and Tools is **not in Read's tab list** (`["file", "view"]`). The operator's instruction was that OCR *be available in Read*, so shipping it here would have satisfied this document and broken the instruction. **This is the third time the same pattern has decided a tab**, and it is now a rule rather than a coincidence: *a command refused in a mode where the operator plainly needs it is evidence that the command's tab is wrong, not that the mode gate needs an exception.* It moved `edit.form_fill` → `view.panel_forms` (filling is not authoring), `edit.copy_*` → `file.copy_*` (copying is not authoring), and now `tools.ocr` → `file.ocr`. The alternative — giving Read the **Tools** tab — was refused because it would hand a reading stance batch merge, batch split and font embedding. **File over View** because OCR's product is a new file, which is what the File tab is for. |
| **Validate** | PDF/A validate & convert… | **N** — *see `DEFECTS.md` §2* |
| | Optimise… | **N** |
| **Diagnostics** | Render diagnostics | **G** *(status bar; belongs here)* |

Tools is now the tab for things that either operate on files other than
the open one, or are configured once and rarely touched. Redact moved to
Edit ▸ Protect, where a user editing a document will actually look for
it.

---

### 5.8 Format — *contextual*

Appears only while something is selected; disappears on deselect.
Contents vary by selection type:

| Selection | Groups |
|---|---|
| Markup | Colour · Fill · Line width · Line style · Opacity · Arrowheads · Note text · Delete |
| ce dimension | Group · Scale · Precision · Units · Standard · Witness lines · Delete |
| Image | Size · Position · Crop · Opacity · Replace · Delete |
| Vector object | Stroke · Fill · Winding rule · Node tools · Delete |
| Text run | Font · Size · **Bold · Italic** · Colour · ~~Spacing · Alignment~~ · Delete — ★ **BUILT 2026-08-27, see the amendment below** |
| Pages (rail) | Rotate · Delete · Extract · Move |

This tab is the single largest usability change proposed here. It is
also what makes selection *mean* something — right now, selecting an
object gives you an object-tree row and no way to act on it.

### Both surfaces, not one — operator decision, 2026-08-12

The contextual tab and a **persistent properties panel** both ship. They
are not redundant; they answer different questions.

| | Format tab | Properties panel |
|---|---|---|
| Lives | in the ribbon, appears on selection | right dock, always available |
| Holds | the edits reached for mid-gesture | the complete property set |
| Survives a tab switch | no | yes |
| Costs | nothing when nothing is selected | ~200 px of width |
| Discoverable | high — it appears, which is itself the affordance | medium |

The division of labour: the **tab** carries what a user changes *while
working* — colour, width, style, align, delete. The **panel** carries
everything, including the read-only facts (winding rule, node count,
embedded-font status, exact geometry) that belong beside the Objects
panel's inventory rather than in a ribbon band.

The panel is also where the **editable geometry** lives — X, Y, W, H as
typed values. That is the surface through which `/Rect` move-and-resize
becomes reachable without a drag, which matters for drafting work where
the number is known and the mouse is imprecise.

Build order: **panel first, tab second.** The panel is the harder half
and the tab's contents are a subset of it, so building the tab first
would mean writing the property editors twice.

### ★★★ Amendment, 2026-08-27 — the Font group, and three changes to the row above

**Operator instruction:** O37, *"We should also have all the font tools
available that Word does"*, followed by *"make the text tool discoverable,
then the Format tab's Font group."* This section is settled and is not
improvised around; the amendment is recorded here because the operator
directed it, and it is written up rather than applied silently.

The **Text run** row shipped as one captioned band called **Font**, placed
**first on the tab**, ahead of Selection. Three departures from the row as it
was written, each with its reason:

**1. Bold and Italic were added.** The row did not name them and Word's Font
group is unusable without them; they are also, literally, the buttons the
operator pressed. They pass the build-order test — the Properties panel's
*This text* section already had them — so the tab's contents remain a *subset*
of the panel's, which is the constraint the build order exists to protect.

**2. Spacing and Alignment were dropped, and it is not a scheduling deferral.**
`EditSession` has no verb for either. `format_text` sets face, size, weight and
fill and nothing else, so nothing writes `Tc`, `Tw` or `TL` for an existing
run. Alignment is worse than missing: it is not a property a PDF text run
*has*, it is a consequence of where each show operator was positioned, so
re-aligning existing text means re-laying it out. Both stay in
`shell::manifest::PLANNED` with those reasons written in, replacing the earlier
"panel first" note that read as a scheduling fact and was not one.

**3. Grow and Shrink were considered and refused**, though Word has them, for
the build order's own reason: they exist in no panel section, so adding them
here would make the tab a **superset** of the panel — which is exactly the
"writing the editors twice" that the build order is written to prevent.

**Order within the group** is Word's: face, size, a rule, then Bold, Italic,
colour. The rule separates *which typeface* from *how it is set*.

**The group is first on the tab** because every row of the table above ends in
Delete, so reading left to right goes "change how this looks", then "describe
it", then "destroy it" — increasing commitment, which is the ordering rule the
Selection group already follows internally.

**Two conditions carry it**, and the split is R9 rather than convenience:
`mode.edit_content` is each item's `visible_when`, so the whole band is
**absent** in Read and Review, which cannot change page content at all;
`selection.text` is each command's `enabled_when`, so inside Edit the controls
**grey** until something is swept, with a tooltip that says how to sweep. That
greyed state is deliberate and is the answer to O37's own admission that
nothing on screen told an operator to press `T`.

**The tab's `visible_when` changed** from `selection.any` to
`selection.formattable` — the union of an object selection and a live text
selection. The tab now has two kinds of subject and neither operand is its
question.

A third surface, the **context menu**, carries the same commands again
for the user who right-clicks. That is not duplication in the P1 sense —
context menus are not tabs — and it is the path most users try after the
keyboard.

---

## 6. What deliberately does not go on the ribbon

**Quick Access Toolbar** — Open, Save a copy, Undo, Redo. Unchanged from
today, now understood as a P1a shortcut surface rather than the only
home for those commands.

**Status bar** — Find toggle, actual size, fit width, fit page, zoom
−/%/+, page ◀ n/N ▶, and a **new editable page-number box**. These are
the controls a user touches constantly; they belong where they never
disappear behind a tab change. The current render-diagnostics text moves
behind a disclosure (see `DEFECTS.md` §5).

**Context menus** — currently zero in the entire crate
(`grep context_menu` → no hits). Every selection type above needs one,
carrying the same commands as its Format tab section plus Cut/Copy/
Paste/Delete. This is not a ribbon question, but it is the other half of
making selection meaningful, and no amount of ribbon design substitutes
for it.

**Tool Options pane** — per P2, every armed tool's parameters. Unchanged
in principle; the pane needs the layout work described in the roadmap.

### ★★★ Amendment, 2026-09-04 — a fourth surface: the TRAILING region

`OPERATOR_REQUESTS.md` **O122**, the operator: *"beside our read-review-edit
buttons at the top there should be an open in acrobat button."*

This document specified three surfaces — the ribbon, the QAT and the status bar
— and the tab-strip row had three regions: QAT on the left, tabs in the middle,
the mode selector on the right. There was no way to put a control **past** the
mode selector, which is where he asked for one. So there is now a fourth
region, and it is a first-class part of the shell rather than a special case:

```text
[QAT] │ File View Pages ⏷ 2 more   ( Read │ Review │ Edit )   [Acrobat]
                                                              └ trailing ┘
```

**What may go here, and it is a narrow set.** The QAT is *"the handful of
controls that must never sit behind a tab switch"* — verbs used continuously,
on the left, where reading starts. This is the opposite end of the row, read
last, and the rule that follows is:

> A trailing control **ends the current activity** rather than advancing it.

*Open in Acrobat* qualifies exactly: it closes the document and hands the file
to another program. A control an operator reaches for during their work does
not belong here and belongs on a tab, or on the QAT if it is used all day.

**P1 is not weakened.** A trailing control may appear here **and** on a tab, on
the same amendment P1a already grants the QAT and the status bar: *a shortcut
to a known home is not a second place to hunt.* `Shell::validate`'s
one-command-one-tab rule walks tabs only, so nothing here relaxes it.

★ `file.open_in_acrobat` is nonetheless on **no** tab, which is the first
command in this manifest of which that is true. Its home is a fixed position in
chrome that is visible in every mode, which is a *stronger* discoverability
guarantee than a tab gives — and putting it on File as well would give two
places to press for one act, one of which comes and goes with what is installed.

★★ **P3 applies here with full force and is the reason the region is data
rather than a callback.** The control is absent on a machine with no Acrobat —
not greyed — through `visible_when` on the manifest item, which is the shell's
own R9 mechanism. The region holds `Item`s, resolved through the command
registry, precisely so that nothing can draw a control here that has no command
behind it.

★ It is also the one reserved region on that row that may be **squeezed out**.
The QAT, the mode selector and the two overflow affordances are promises the
interface has already made; a trailing control's absence is a state the
application already handles, so on a very narrow window it is dropped whole
(never a sliver) and the drop is disclosed as `ribbon-trailing-dropped`.

---

## 7. Migration map

Every command that exists in the GUI today, and where it goes. Nothing
is dropped.

| Today | Proposed |
|---|---|
| QAT: Open, Save a copy, Undo, Redo | QAT *(unchanged)* + File ▸ File/Save |
| File ▸ Document ▸ Properties | File ▸ Document |
| File ▸ Clipboard ▸ Copy page/document text | **File ▸ Export** — *corrected 2026-08-25; this row said Edit ▸ Clipboard, which has not been true since 2026-08-14. See §5.1* |
| File ▸ Export ▸ Export DXF | File ▸ Export |
| File ▸ Print ▸ Print | File ▸ Print |
| File ▸ Layout ▸ Reset layout | **View ▸ Window** |
| File ▸ Settings ▸ Settings | File ▸ pdfcer |
| File ▸ Help ▸ Keyboard shortcuts | File ▸ pdfcer |
| Edit ▸ Pages ▸ Rotate left/right | **Pages ▸ Transform** |
| Edit ▸ ContentTools ▸ Editing on | Edit ▸ Mode |
| Edit ▸ ContentTools ▸ Aa / I⁺ Aa / Obj | Edit ▸ Content *(relabelled)* |
| Edit ▸ Forms ▸ Fill Form | **View ▸ Panels** — *corrected 2026-08-25; see §5.4* |
| Edit ▸ BuildForm ▸ Create Field | Edit ▸ Forms |
| Review ▸ Markup ▸ Rectangle/Ellipse/Arrow line/Highlight band + Colour | **Markup ▸ Shapes / Text markup / Style** |
| Review ▸ Comments ▸ Comments | Markup ▸ Comments |
| Review ▸ Notes ▸ Text box / Sticky note / Draft stamp | Markup ▸ Notes |
| Measure ▸ Measure ▸ Linear / Radius-Diameter | Measure ▸ ce dimension |
| Measure ▸ Measure ▸ Set Group Scale / Manage ce-dimension Groups | Measure ▸ Scale |
| Tools ▸ Protect ▸ Redact | **Edit ▸ Protect** |
| Tools ▸ Across files ▸ Tools *(batch pane)* | **Pages ▸ Insert/Organise** + Tools ▸ Batch |
| Tools ▸ Font folders ▸ Font folders | Tools ▸ Fonts |
| View ▸ Panels ▸ Sidebar / Bookmarks / Layers / Signatures / Objects | View ▸ Panels |
| View ▸ Panels ▸ Fonts | **File ▸ Document** |
| View ▸ Show ▸ annotations / points | View ▸ Display |

Three commands change tab in a way a returning user will notice:
`Copy text` (File → Edit), `Redact` (Tools → Edit), and `Rotate page`
(Edit → Pages). All three are moving *toward* where a first-time user
would look, which is the trade this proposal accepts.

---

## 8. Questions — answered and outstanding

### Answered 2026-08-12

**`Editing on` master toggle** → *"make it work the same way other
programs do."* Removed entirely. See §5.4.

**Format tab vs. properties panel** → **both**, panel first. See §5.8.

**Page display** → single page stays the **default**; continuous,
facing and facing-continuous are modes chosen on the View tab, persisted
per document. See §5.2.

**Rendering** → whole-page raster stays the default because it measured
better in use; tiled progressive becomes an opt-in, with raster scale
and settle delay exposed beside it. New **View ▸ Render** group. See
§5.2.

### Still open

1. **Save semantics** — is autosave + true in-place `Save` wanted in the
   next year, or is `Save a copy` the permanent model? The File tab
   layout differs.
2. **Compare** — worth building, or out of scope? It is the one absence
   an AEC reviewer will name first, and it is a large build.
3. **Multi-run text editing** (`GUI_ROADMAP.md` Phase 5d) — is "edit one
   run at a time, clearly disclosed" an acceptable resting state for
   another year, or is this the thing that has to be right?
