# INTERACTION GAP

**The comparison, and the work list.**
Written 2026-08-26 as the third document of the interaction-design audit opened
by the operator's report of the same day.

Reads `HOW_IT_SHOULD_WORK.md` (the target) against `HOW_IT_WORKS_TODAY.md` (the
measured present) and says what the difference is, what kind of difference it
is, and what order to close it in.

Bare paths (`canvas/input.rs:151`) are relative to
`D:/Dev/pdfcer-gui/crates/pdfcer-gui/src/`. Engine paths are written in full from
`D:/Dev/pdfcer/`. Every claim about pdfcer-gui behaviour below was either cited by
one of the two source documents and re-checked against the source tree while
writing this one, or is marked **[from HOW_IT_WORKS_TODAY, driven]** where the
evidence is a launch trace in `D:/Dev/pdfcer-gui/evidence/audit/`.

**This document proposes no code. It describes, classifies, and orders.**

---

## Contents

1. [The headline](#1-the-headline)
2. [The gap table](#2-the-gap-table)
3. [The work list](#3-the-work-list)
4. [The things that are already right](#4-the-things-that-are-already-right)
5. [The one change](#5-the-one-change)
6. [Appendix — the operator's seven complaints, adjudicated](#appendix--the-operators-seven-complaints-adjudicated)

---

## 1. THE HEADLINE

**On the operator's own files the object he is clicking does not exist in the
program's object model at all** — the engine decomposes a page into a flat list
in paint order and stops at the door of a form XObject
(`D:/Dev/pdfcer/crates/pdfcer-core/src/vector/decompose.rs:2666-2672`, which emits
the form as one `ImageSource::Form` and never enters it), and then hit-tests
that form as a bare inflated rectangle
(`D:/Dev/pdfcer/crates/pdfcer-core/src/vector/hit.rs:437`), so on
`the conformance suite’s composite page` the page-sized form at index 26 wins
every hit test at every point on every page while the entire visible body of the
sheet — every patch, swatch, overprint panel and image — is invisible to
selection, permanently, at every zoom, with every modifier.

**Everything else he complained about is a binding that was designed, argued for
in-tree, and then not wired up**: the engine computes the full front-to-back
candidate list and the GUI calls `.find()` and throws the tail away
(`canvas/input.rs:151-158`); the Properties panel is fed by an Objects-panel
`focus` variable rather than by the canvas selection, with a test pinning the
severance in place (`panels/mod.rs:736`); press-and-drag on an unselected object
is *defined* as a marquee so select-and-move can never be one gesture
(`canvas/gesture/meaning.rs:736`); "Properties" next to a selection dispatches
the **document** properties command (`app/dispatch.rs:819-824`); and a tested
page-range parser sits one directory away from the OCR dialog that has no page
range (`dialogs/print/tabs.rs:161`).

**So he is right that the system is convoluted, but not in the way he thinks:
it is not over-built, it is under-wired** — three parallel notions of "the thing
I am working on" (the armed tool, the panel `focus`, the canvas selection) with
no bridge between them and no one of them authoritative, which is a governance
failure rather than a complexity failure, and it means that of his seven
complaints five are one binding each in the GUI and only two need pdfcer-core.

**A correction to the brief that opened this audit.** The anchoring hypothesis —
that the full-page `paint=none` path is swallowing clicks — is **false and was
tested**. `path_hit` gives an unfilled, unstroked path a proximity band of the
tolerance alone, so it is selectable only within 6 px of its outline
(`D:/Dev/pdfcer/crates/pdfcer-core/src/vector/hit.rs:453-467`, intent stated at
`:22-25`); it appears in the candidate list at (0.5, 0.5) and is absent at the
page centre. **[from HOW_IT_WORKS_TODAY, driven]** That part of the engine is
correct. The culprit is the *form*, which is the one object kind that most often
spans a whole CAD sheet and also the one kind tested as a solid opaque
rectangle. Also corrected: the page has **28** objects (0–27), not 29
(`pdfcer object-list --page 1`: `objects=28 paths=21 text=3 images=0
forms=4`).

---

## 2. THE GAP TABLE

**Class legend — the four-way split, which matters more than the list:**

| Class | Meaning | What it costs to close |
|---|---|---|
| **(a) not implemented** | No code path exists. | Build it. |
| **(b) implemented but unreachable** | The capability is complete and correct, and its only consumer discards it, or no consumer exists. | Wire it. Usually the cheapest row in this table. |
| **(c) implemented but gated behind something unexpected** | It works, once you satisfy a precondition nothing states — a mode, a rung, a filter row, an epoch. | Move or announce the gate. |
| **(d) working; the operator could not find it** | Shipped, correct, and either invisible or masked by a different failure. | Nothing, or one sentence of feedback. |

**Tally across the 63 rows below: (a) 32 · (b) 9 · (c) 12 · (d) 10.** Read that
as: half the gap is genuinely unbuilt, a third is built-and-mis-gated or
built-and-unwired, and a sixth is working software the operator never got to
see because something upstream failed first.

### 2.1 Selecting

| Behaviour | Should (`HOW_IT_SHOULD_WORK` §) | Today (`file:line`) | Class |
|---|---|---|---|
| A click selects the smallest visible thing that painted the mark | §1, §2.2 | Returns the topmost bbox hit; on the the conformance suite’s composite page file that is the page-sized form #26 at every point tested | **(a)** |
| Marks painted *inside* a form XObject are selectable | §2.6 | They are not in the object model: `pdfcer-core/src/vector/decompose.rs:2666-2672` emits the form and returns | **(a)** — engine |
| Form XObjects are never a first-click target | §2.6, §9.2 | A form is `VectorObject::Image` and wins by paint order | **(a)** — engine |
| Images and forms hit-tested against ink, not a box | §2.2, §9.3 | `pdfcer-core/src/vector/hit.rs:437` — one line, `page_bbox.inflate(tol).contains(point)` | **(a)** — engine |
| Paths hit-tested against ink, winding-correct, curves flattened | §2.2 | Exactly this (`hit.rs:443-468`, `:502-513`) | **(d)** |
| `paint = none` path hittable only near its geometry | §2.2 | Correct (`hit.rs:453-467`) | **(d)** |
| Clicking blank paper deselects | §2.5 | Implemented (`canvas/selection/mod.rs:387-390`); unreachable on this file because the hit test says there *is* no blank paper | **(b)** |
| `Alt`+click cycles down the candidate list | §2.7, §9.13 | `hit_test_all` is complete and correct in the engine and has exactly one GUI consumer, which keeps the head: `canvas/input.rs:151-158` | **(b)** |
| `Shift`+`Alt`+click adds the next one down | §2.7 | No `modifiers.alt` exists anywhere in `canvas/` | **(a)** |
| `Ctrl`+click selects the owning container | §2.8 | No `Ctrl` on the selection path at all | **(a)** |
| `Ctrl`+`Up`/`Down` walks the containment chain | §2.8 | No containment chain exists to walk | **(a)** |
| `Shift`+click adds / toggles | §2.9 | Works (`canvas/selection/mod.rs:733-740`); invisible on this file because the topmost is always the same object | **(d)** |
| Marquee, enclose by default | §2.9 | Works; `MarqueeMode::Enclosed` (`panels/objects/provider.rs:872-883`), release at `canvas/interact.rs:676-687`; selected 24 objects at once **[driven]** | **(d)** |
| `Alt` during a marquee switches to touch | §2.9 | Not built; `Alt` is inert on marquee | **(a)** |
| A marquee never starts on top of a selectable object | §2.9 | Inverted: press-drag on an unselected object is *always* a marquee (`canvas/gesture/meaning.rs:736`) | **(a)** |
| Right-click an unselected object selects it | §2.10 | Works (`canvas/menus.rs:172-199`) | **(d)** |
| Right-click ▸ Select ▸ lists every object under the pointer | §2.10, §9.13 | No such menu; the menu carries zoom, properties, delete (`shell/menus.rs:305`, `:1035`) | **(a)** |
| Hover outlines the prospective target | §2.4 | None. Refused on a cost argument at `canvas/tool/arm.rs:139-143` and `canvas/interact.rs:1412-1427`; the machinery exists and is mounted for measure tools only (`canvas/measure/hover.rs`) | **(a)** |
| Hover names the target on the status bar | §2.4, §8.3 | None | **(a)** |
| The nothing-selected tutorial string | §8.3 | None | **(a)** |
| The pick filter's 11 classes, subtractive, one click away | §2.11 | Built and persisted (`canvas/pick.rs:345-357`, `app/pickstore.rs:116-160`) | **(d)** |
| `PickClass::FormXObject` off = the escape hatch | §2.11 | It **is** the only escape today, and it requires the operator to already know the diagnosis; its tooltip is the only text in the program that describes the problem (`text/pick.rs:193-196`) | **(c)** |
| A refused click says why | §2.11.3, §3.7 | Nothing is said, ever | **(a)** |
| Filter rows are all consulted | §2.11 | **Only 3 of 11 are read by anything.** `Markup`, `CeDimension`, `FormField`, `Link`, `Characters` have no consumer — and `canvas/pick.rs:105-109` warns against exactly this | **(a)** |
| The filter never goes off-screen | §2.11.2, §9.19 | Measured off-screen at `ui_scale = 1.80` in 1100×800 (`app/status.rs:806-820`) | **(a)** |
| The Objects panel row selects on the canvas | §4.4 | It sets `focus` and nothing else, deliberately and documented (`panels/objects/mod.rs:129-141`); part and point rows are not clickable at all | **(a)** — pinned by `the_panel_focus_has_not_quietly_become_a_selection` (`panels/mod.rs:736`) |
| Canvas selection scrolls and highlights the panel row | §4.4 | Highlight is `focused == Some(index)`, written only by a row click (`panels/objects/mod.rs:371`) | **(a)** |
| Per-object eye and lock toggles in the list | §4.4 | No visibility model, no lock model | **(a)** |
| Double-click descends a rung | §2.8 | Implemented (`canvas/selection/mod.rs:814-857`) and on a form or image it descends into a Part that names no part, because `part_kind` returns `None` (`panels/objects/provider.rs:343-349`) | **(c)** |

### 2.2 Manipulating

| Behaviour | Should | Today | Class |
|---|---|---|---|
| Press on an unselected object selects **and** moves it, one gesture (7 of 7) | §3.2 | Impossible by construction: `grip` is read from the *current* selection at press time (`canvas/pressing.rs:129-138`) and the gesture machine makes click and drag mutually exclusive (`canvas/gesture/mod.rs:29-43`). Press-drag draws a rubber band | **(a)** |
| Move works on paths, text, images, forms | §3.2 | Works (`canvas/moving.rs:375-472`, dispatch by rung then by all-paths) | **(d)** |
| Resize works on every kind | §3.3 | Works, kind-agnostically, through one `TransformObjects` (`canvas/resizing.rs:285-352`); the old `NotAPath` refusal was deleted 2026-08-20 | **(d)** |
| Rotate works on every kind | §3.4 | Works (`canvas/rotating.rs:206-291`), pivot at selection centre | **(d)** |
| 8 grips + a drawn rotate handle, screen-sized, mid-edge dropped under 24 pt | §3.1 | Exactly this (`canvas/handles.rs:100`, `:109`, `:117`, `:335`) | **(d)** |
| `Shift` constrains move / aspect / 15° rotate, and says so | §3.2–3.4 | All three built (`canvas/constrain.rs:277-285`, `:235`, `canvas/rotating.rs:83`), caption on the status bar | **(d)** |
| `Alt` scales about the centre | §3.3 | Named as unbuilt at `canvas/constrain.rs:64-76` | **(a)** |
| `Alt`+drag duplicates | §3.2 | Not built | **(a)** |
| Arrow / `Shift`+arrow / `Ctrl`+arrow nudge, repeating, coalesced undo | §3.5 | **Does not exist.** `canvas/keys.rs:334-338` reads three keys: Escape, Delete/Backspace, Tab | **(a)** |
| A selection always has handles (§9.5) | §3.1 | Gated on the rung, not the kind: `if selection.level() == Object` (`canvas/overlay.rs:221`), hit test agreeing at `canvas/pressing.rs:152`. After a double-click on an image or form: outlined, no handles, body drag refuses to a trace line, Delete does nothing, nothing on screen says any of it | **(c)** |
| Delete works on a selection | §3.5 | Object rung only (`canvas/selection/mod.rs:370-375`); silently a no-op after a double-click, refusal goes to `reason=no-verb-for-rung` in a trace | **(c)** |
| Annotations and form fields get handles | §3.1 | They do not — `draw_selection` strokes the `/Rect` and returns (`canvas/overlay.rs:184-189`); form fields are not in the content selection at all | **(a)** |
| A newly placed / pasted object arrives selected (§9.7) | §5.3 | `Action::InsertImage` never sets `doc.selection` (`app/actions/apply.rs:409-444`) | **(a)** |
| Dragging a handle never does nothing (§9.6) | §3.7 | Grips are painted over objects with a singular CTM; the engine's `transform_preview` exists and is not called, acknowledged in-tree at `canvas/resizing.rs:174-205` | **(b)** |
| A locked object selects, shows padlocks, prints a sentence | §3.7 | No lock model, no padlock handles, no sentence | **(a)** |
| Move refusals reach the operator | §3.7, §9.4 | All ten variants go to a trace line marked `ui-text-exempt` (`canvas/moving.rs:733-742`) | **(a)** |
| Rotate refusals reach the operator | §3.7 | **No refusal enum at all** — six silent `return None` paths (`canvas/rotating.rs:220-256`) | **(a)** |
| Resize refusals reach the operator | §3.7 | A real sentence, recorded at **epoch zero** (`canvas/resizing.rs:530-546`) and read at the **live** epoch (`app/actions/disclosure.rs:110-116`). Visible only on a document not yet edited this session | **(b)** |
| Stroke widths scale with the object | §3.3 | Deliberately not (`text/resizing.rs:78-104`), disclosed rather than fixed — a stated CAD decision that §3.3 reverses | **(c)** |
| `Esc` cancels an in-flight drag before anything is written | §2.5 | Works (`canvas/gesture/mod.rs:249-251`) | **(d)** |

### 2.3 Properties, panels, ribbon

| Behaviour | Should | Today | Class |
|---|---|---|---|
| Selecting anything repopulates a property surface, same frame (7 of 7, §9.8) | §4.1 | No surface responds to `doc.selection` except grips, a greyed Delete, and one tab appearing at the far right of the ribbon | **(a)** |
| The Properties panel reads the canvas selection | §4.2, §9.10 | It reads `PanelsState::focus`, written only by an Objects-panel row click (`panels/properties/mod.rs:286` ← `panels/objects/mod.rs:376-377`). Across an entire driven run, 113 `properties-panel` lines **all said `object=25`** through five different canvas selections **[driven]** | **(a)** — and actively pinned |
| X/Y/W/H for a selected path | §4.3 | Exists, behind an **Apply** button, points only, no units picker, no rotation field (`panels/properties/geometry.rs:435-473`, `:74-81`) | **(c)** |
| X/Y/W/H for a selected **text object or image** | §4.3 | Not drawn at all — no heading, no greyed field, no explanation. Gated out by a path-only helper (`panels/objects/provider.rs:677-680` → `geometry.rs:350`) | **(a)** |
| Font / size / colour / spacing / alignment for selected text | §4.3 | No surface anywhere shows them and no command would change them; `format.font`, `format.font_size`, `format.spacing`, `format.alignment` are all in `manifest::PLANNED` (`shell/manifest/mod.rs:1043-1137`) | **(a)** |
| Fields accept units and arithmetic, commit on `Enter`/`Tab` | §4.5 | Draft + Apply, points only | **(a)** |
| Values update live during a drag | §4.5 | No | **(a)** |
| The Tool panel is about the armed tool, never the selection | §4.1, §9.11 | Exactly that, and it says so (`panels/tool/mod.rs:23-40`, `panels/tool/armed.rs:52-55`) | **(d)** — correct, and it is what the operator was looking at |
| Two surfaces one letter apart | — | The **Tool** dock panel and the **Tools** ribbon tab, on the same screen (`text/commands/view.rs:365-371`, `text/ribbon.rs:93-95`) | **(a)** — a naming defect that manufactures the misunderstanding |
| The Format tab carries the property band | §4.2, §9.12 | Two commands (`shell/manifest/format.rs:126-131`) | **(a)** |
| The Format tab appears on selection without stealing focus | §4.6 | Appears — appended **last**, far right (`egui-shell/src/ribbon/tabs.rs:247-253`) | **(d)** on appearing, **(a)** on placement |
| The Format tab becomes active on double-click and on insert (Microsoft's MUST) | §4.6 | Never auto-activates (`tabs.rs:283-291`) **[driven]** | **(a)** |
| `format.properties` shows the **object** (§9.9) | §4.6 | It pushes `Action::Command("file.properties")` — the document dialog (`app/dispatch.rs:819-824`). The exact inverse of universal practice | **(c)** |
| Nothing-selected shows document/page properties, never a blank panel | §4.3 | *"Pick a row in the Objects panel to see what it is made of."* | **(a)** |
| Properties is visible when a selection happens | §4.2 | First tab of a five-tab stack in Edit; if Forms or Comments was left active, selecting an object changes nothing visible in that slot (`app/modes/defaults.rs:299-470`) | **(c)** |
| Properties in Review mode has a route in | §4.4 | Mounted **without** the Objects panel, so `focus` can never be set and the empty state instructs the operator to click a row in a panel that mode does not have | **(c)** |

### 2.4 Modes

| Behaviour | Should | Today | Class |
|---|---|---|---|
| Selection is never gated by mode; mutation always is | §6.2 | `edit_content` gates selection itself (`app/modes/capability.rs:116-136`); in Read and Review the press is routed to the text sweep by `textsel::takes_the_press` (`canvas/textsel/gate.rs:272-274`) | **(c)** |
| Read inspects with a read-only Properties panel | §6.2 | Read mounts **no Tool, no Objects, no Properties** panel at all | **(a)** |
| The session opens where the operator left it | (implied by §6) | Always opens in Read, every launch, every document; the active mode is not persisted (`app/modes/mod.rs:594-598`, pinned at `:783`). So the operator's first clicks of every session select nothing and nothing explains it | **(c)** |
| The ribbon follows a mode change | (implied) | It does not — it stays on whatever tab was open **[driven]** | **(a)** |

### 2.5 OCR

| Behaviour | Should | Today | Class |
|---|---|---|---|
| Dialog opens with **All pages** selected (6 of 6 tools) | §7.2 | One page, captured from `doc.view.page_index` at dialog-open (`dialogs/ocr.rs:194-202`); button reads "Recognise this page" | **(a)** |
| A `Pages n to m` field accepting `2,3,5-7` | §7.2 | None — **and a tested parser sits one directory away**: `parse_page_range("5,1-2", 10) -> Some(vec![4,0,1])` (`dialogs/print/tabs.rs:161`, tests at `:537-589`) | **(b)** |
| Multi-select thumbnails ▸ "Recognise selected pages…" | §7.2 | No route; this is the literal answer to *"Where is the option to select more than one page?"* | **(a)** |
| The dialog states which page it is about to do | §7.2 | It never displays a page number anywhere (`dialogs/ocr.rs:293-340`) | **(a)** |
| Skip / redo / force existing-text policy | §7.3 | Nothing checks for existing text | **(a)** |
| Language, downsample, deskew options in the dialog | §7.4 | None. The **CLI**, against the same engine, exposes `--dpi`, `--model-dir`, `--words` (`D:/Dev/pdfcer/crates/pdfcer/src/main.rs:2374-2450`) | **(b)** |
| Effective DPI shown | §7.4 | Computed and reported — to the trace line only (`dialogs/ocr.rs:268-278`) | **(b)** |
| Progress: page *k* of *n*, per-page list | §7.5 | An indeterminate spinner and the word "Recognising…" | **(a)** |
| Cancel, keeping completed pages, writing nothing | §7.5 | No cancel control; closing the dialog detaches the thread, which runs to completion burning CPU (`ocr/mod.rs:493-509`) | **(a)** |
| In Edit, OCR applies in place and `Ctrl+S` writes over the original | §7.6 | Removed program-wide by a rule sited on the *feature*: *"no second save command, no in-place path, no `Save`-labelled control anywhere"* (`dialogs/ocr.rs:64`) — which is a **Read**-mode rule (§9.17) | **(c)**, with an engine-blocked core |
| The recognised file is opened afterwards | §7.6 | Written, then neither opened nor pointed at; one label says *"Saved to {path}"* (`dialogs/ocr.rs:433-443`, `:328-330`) | **(a)** |
| The window stays responsive | §7.5, §9.18 | It does — the worker is on its own thread and the dialog is its own viewport since 2026-08-21 — but the tooltip still promises *"the window will not respond while it does"* (`text/ocr.rs:102-104`) | **(d)** working, **(a)** the string is false |
| OCR remains available after an edit | — | **The preflight trap.** `if doc.edit_epoch != 0 { return Some(Refusal::UnsavedEdits) }` (`dialogs/ocr.rs:374-376`), and `edit_epoch` is monotonic and **never reset, not even by a successful save** (`app/save.rs:82-98`). Make one edit, save it, and OCR refuses for the rest of that document's life with a message that is false at that moment. Only close-and-reopen recovers | **(c)** |

### 2.6 Truth in shipped text

Not a behaviour gap; a credibility one, and it belongs in the same table
because it is why an operator stops believing the interface.

| Claim | Where | Status |
|---|---|---|
| *"the same list the GUI's Alt+click cycling steps through"* | `pdfcer object-list --help`; `pdfcer-core/src/vector/hit.rs:41-48`; `target.rs:85-88` | **False.** There is no Alt+click cycling. |
| *"It takes a few seconds and the window will not respond while it does"* | `text/ocr.rs:102-104` | **False in two directions** — worker thread, own viewport. |
| *"This document has unsaved changes"* | `text/ocr.rs:246-248` | **False whenever it appears after a successful save.** |
| *"Nothing here can be changed in this build."* | `text/panels/properties.rs:334-335` | True, and it is the honest one. |

---

## 3. THE WORK LIST

Ordered by **operator-visible return per unit of work**, not by architectural
interest. Size is a rough band: **XS** = a constant or a line; **S** = under a
day; **M** = a few days; **L** = a week or more; **XL** = engine plus shell.

Items **1–9 are GUI-only and unblocked.** Items 10–12 need pdfcer-core.

---

### 1. A newly placed or pasted object arrives selected — **XS**

**Changes:** `Action::InsertImage` (and paste, and place-text) set
`doc.selection` to the new object at the Object rung
(`app/actions/apply.rs:409-444`).

**Unblocks:** the whole of complaint 4. A driven test already proved that once
an image *is* selected, a corner-grip drag resizes it
(`resize-commit grip=SouthEast sx=0.6810 sy=0.5899`) and a body drag moves it.
The only reason the operator saw neither is that the image arrived unselected
and his first drag became a marquee.

**Could break:** a paste that lands off-screen now steals the selection from
whatever he was working on; the Format tab appearing on insert changes the
ribbon's visible tab set mid-gesture. Both are the intended behaviour in §5.3
and §4.6, so the risk is that a test pinning "insert does not change the
selection" exists and must be retired deliberately, not silently.

---

### 2. `format.properties` stops answering a question about the file — **XS**

**Changes:** one match arm (`app/dispatch.rs:819-824`) stops pushing
`file.properties` and instead opens/focuses the Properties panel on the
selection.

**Unblocks:** nothing on its own until item 5 lands — but it removes the single
most misleading control in the application. Today the button whose stated
question is *"tell me about the thing I just clicked"* answers a question about
the document, which is the inverse of universal practice in all eight surveyed
applications (§9.9).

**Could break:** anyone who learned to reach document properties through the
Format tab. Nobody has; `file.properties` keeps its own command and its own
place on the File tab.

---

### 3. Say what is selected, on the status bar — **S**

**Changes:** a readout region at the far left of the status row (§8.2, §8.3):
what is selected, its size and position, and — once item 11 lands — its
containment path. Plus the nothing-selected tutorial string. **No hover hit
testing in this item**; this is the *selected* half only, which costs nothing
because the selection is already resolved.

**Unblocks:** the operator's core experience — *"all I get is the page
selected"* becomes *"Selected: Form — 214 objects · 595.6 × 790.9 pt"*, which is
a diagnosis instead of a mystery. It also makes items 4 and 11 legible when they
land, and it is the surface every refusal sentence in §3.7 will be printed on.

**Could break:** status-bar space at high `ui_scale` — the cluster already
measured off-screen at 1.80 in an 1100×800 window (`app/status.rs:806-820`), and
adding a region on the left makes that worse for the fixed cluster on the right.
The readout must be the thing that elides, never the filter (§9.19).

---

### 4. `Alt`+click cycling — **S**

**Changes:** `topmost_allowed` (`canvas/input.rs:144-158`) stops calling
`.find()` and returns the filtered list; a per-point cursor lives in
`SelectionState`, reset on a 3 px pointer move; `modifiers.alt` is read on the
selection path for the first time. Each landing names itself through item 3.

**Unblocks:** every object that is underneath something, which on the the conformance suite’s composite page file
today is 27 of 28 objects on the page. It is the difference between the
"Blocks" filter toggle (global, all-or-nothing, requires knowing the diagnosis)
and a per-click escape.

**Could break:** `Alt` is the platform menu-activation key on Windows and egui's
handling of it must be checked; and `Alt`+drag is reserved for duplicate-drag in
§3.2, so the travel-based split (no travel = cycle, travel = duplicate) must be
built in the same commit or the second one becomes hard to add later.

**Note:** this also retires a false documentation claim that ships in three
places today (§2.6 above).

---

### 5. Delete `focus`; the Properties panel reads the canvas selection — **M**

**Changes:** `PanelsState::focus` is deleted; `panels/properties/mod.rs:286`'s
`object_section` reads `doc.selection`; the Objects panel row click writes
`doc.selection` instead; canvas selection scrolls and highlights the row; the
test `the_panel_focus_has_not_quietly_become_a_selection` (`panels/mod.rs:736`)
is retired **with its reasoning replaced, not deleted**, since it was right for
the world it was written in.

**Unblocks:** complaint 3 — *"when I have an object selected like text the Tool
tab doesn't switch to giving me the editable stuff"* — for everything the
current panel can already render, plus the Objects panel as the guaranteed route
to an object the pointer cannot reach (§4.4), which matters enormously while
item 11 is still outstanding.

**Could break:** the Review-mode arrangement, which mounts Properties without
Objects (`app/modes/defaults.rs:361-406`) — its empty state must stop naming a
panel that mode does not have. And the panel's own header documents a
deliberate refusal to render editable geometry on the grounds that *"there is
nothing to edit"* (`panels/properties/mod.rs`); that argument dies with this
item and the header must be rewritten, not left contradicting the code.

---

### 6. Press-and-drag on an unselected object selects and moves it — **M**

**Changes:** `canvas/gesture/meaning.rs:736`'s `(None, None) => Marquee` arm
grows a hit test; `canvas/pressing.rs:129-138` must resolve the grip against the
*prospective* selection rather than the cached one; the gesture machine's
click/drag mutual exclusion (`canvas/gesture/mod.rs:29-43`) has to admit a
gesture that is both.

**Unblocks:** the second half of complaint 4, and the convention whose absence
most reliably reads as "the program is broken" — 7 of 7, zero variation.

**Could break:** this is the most invasive of the unblocked items and it touches
the machinery every other gesture routes through. A press on empty paper must
still marquee; a press on a *selected* object must still move without
re-selecting; region-zoom arming must still outrank it
(`canvas/gesture/meaning.rs:738`). Expect the marquee-from-over-an-object case
(§2.9, `Shift`+`Alt`) to need building at the same time or the capability is
lost.

---

### 7. OCR scope: all pages, current page, a range, selected thumbnails — **M**

**Changes:** `ocr::Request` carries `Vec<usize>` instead of `page_index: usize`
(`ocr/mod.rs:463-472`); the dialog grows the four-radio group of §7.2 with
**All pages** first and default; the range field calls the existing
`parse_page_range` (`dialogs/print/tabs.rs:161`); the worker loops. **The engine
must be constructed once, not per page** — `OcrsEngine::from_model_dir` is
currently called inside `recognise_image` on every run
(`ocr/mod.rs:717-718`), so a naive N-page loop reloads ~12 MB of weights N
times. A progress channel replaces the single-result channel
(`ocr/mod.rs:479-482`) so §7.5's counter and Cancel become possible.

**Unblocks:** three of the operator's four OCR complaints — *"how do I OCR more
than one page"*, *"why does the tool stop at one"*, *"where is the option to
select more than one page"*. **No engine change is required for any of it.**

**Could break:** `add_ocr_layer` writes an incremental revision per call; a
multi-page run means chaining revisions, and whether `add_ocr_layer` behaves
correctly when handed a `Document` that is already an incrementally-saved OCR
output is **[UNVERIFIED]** — nothing does this today. That is the one
measurement this item needs before it starts. The alternative — accumulate
recognised pages and write one revision — needs a shape the engine does not
currently offer.

---

### 8. The OCR preflight trap — **S**

**Changes:** `dialogs/ocr.rs:374-376` stops comparing `edit_epoch` to `0` and
compares it to the epoch at last save; or `edit_epoch` gains a companion
"clean" marker that a successful save updates (`app/save.rs:82-98`, currently
pinned as never-resetting by a test at `:828-856`).

**Unblocks:** OCR at all, for any document the operator has edited and saved.
Today the feature dies for the life of the session after the first save and
tells him something false on its way out.

**Could break:** the guarantee the trap exists to protect is real —
`add_ocr_layer` reads the session's **base** revision, so a recognised copy
taken after unsaved edits would silently omit them. The fix must preserve the
refusal for genuinely-unsaved edits and only lift it for saved ones, which
means the base document the OCR reads must be re-pointed at the saved bytes.
Get this wrong and the failure is silent data loss, which is worse than the
trap.

---

### 9. Selection in Read and Review; and open where he left off — **S–M**

**Changes:** `Capabilities::edit_content` splits into `select_content` (true in
all three modes) and `edit_content` (Edit only)
(`app/modes/capability.rs:116-136`); `textsel::takes_the_press`
(`canvas/textsel/gate.rs:272-274`) stops claiming the press for the Select tool
merely because content editing is off; the active mode is persisted alongside
the per-mode layout that already is (`app/modes/mod.rs:594-598`); Read mounts a
read-only Properties panel.

**Unblocks:** the invisible first failure of every session. Today the program
opens in Read on every launch, and in Read a click on a coloured patch produces
`canvas-text-selection via=clear page=0 chars=0 quads=0` and no selection event
at all **[driven]** — nothing on screen explains it. It also removes the
ribbon-does-not-follow-the-mode surprise, which is the same class of problem.

**Could break:** the argument for the current gating is written down and half of
it is good — every Format-tab verb takes the selection as its operand, so gating
twice is redundant. Splitting the capability means auditing every consumer of
`edit_content` to decide which half it meant. And the text-sweep gate is
load-bearing for copy-out-of-a-PDF, which is the most common thing done in Read;
losing it to a selection change would be a regression the operator would notice
within a minute.

---

### 10. **[BLOCKED — pdfcer-core]** OCR as an edit to the open document — **L**

**The engine request:** an OCR verb on `EditSession`. Today
`add_ocr_layer(doc: &Document, page_index, …)`
(`D:/Dev/pdfcer/crates/pdfcer-core/src/ocr/layer.rs:620-625`) takes an immutable
document and returns a complete new PDF as `Vec<u8>`, and `grep -c "ocr"` over
`pdfcer-core/src/edit.rs` returns **0** against a 26,000-line file holding every
other editing verb. What the shell needs is: *apply a recognised text layer to
the live session as an undoable edit, one entry for the whole run, so that
`edit_epoch` moves and the existing in-place save path
(`app/save.rs:329-377`, temp file plus atomic rename) writes it back over the
original.*

**Unblocks:** *"Why do I have to save a copy instead of just go back into my pdf
and save over it?"* — in its strong reading. Zero of six surveyed OCR tools
force a Save-As on the open-document path.

**Could break:** the operator's own standing rule of 2026-08-14 —
*"if in read mode ocr should still be available, but it will prompt to save
changes as save as instead of save"* (`dialogs/ocr.rs:37-40`). It is not
reversed by this item; it is **re-sited from the feature to the mode** (§9.17).
Read still cannot modify the open document, because nothing in Read can. Also
at risk: the regression guard that hashes the source before and after
(`tools/ui-verify/src/checks/ocr.rs:24-38`), which asserts the current
guarantee and must become mode-aware rather than being deleted.

**A weaker version is unblocked and worth pricing separately:** after a
successful save-a-copy, open the new file and repoint the session at it
(`dialogs/ocr.rs:433-443`). That is **XS**, it is not what he asked for, and it
turns *"a dialog telling me a different file exists somewhere"* into *"I am now
looking at the recognised document"*.

---

### 11. **[BLOCKED — pdfcer-core]** Decompose into form XObjects — **XL**

**The engine request, in three parts, stated as §2.6 states them:**

1. **The decomposer recurses.** `decompose.rs:2666-2672` currently handles `Do`
   on a form by emitting `ImageSource::Form` with the form's `/BBox` corners and
   returning. It must instead push the form's matrix onto the CTM, decompose the
   form's content stream, and emit the leaves — each with its own `TargetId`.
2. **Each leaf carries its containment path** — the chain of enclosing form
   XObjects — so the status readout can say `inside Title block (form)` and
   `Ctrl`+click can select the container.
3. **A form's own bbox is retired from the first-click candidate list.** The
   bbox test at `hit.rs:437` may stay for raster images, whose quad genuinely
   *is* their ink; it must not answer for forms.

**Unblocks:** complaint 1 and complaint 2 — *"when I click on one of the objects
all I get is the page selected"* and *"when I double click on an object it
doesn't select"* — completely, and on his real work. On the the conformance suite’s composite page file the four
forms on page 1 contain **the entire visible body**: every test patch, every
swatch, both Overprint panels, the manta-ray images, the right-hand logo. On a
SolidWorks drawing the same structure holds per view. Without this item, every
other item on this list improves how well he can select *the twenty-five
objects that are not the drawing*.

**Could break:** everything downstream of object indices. `Selection` is four
integers, the second of which is *"a paint-order index into the page's
decomposition"* — recursion renumbers every object on every page, so anything
persisted, cached or asserted against those indices moves. Object counts
explode: a 28-object page could become 4,000, so the Objects panel must expand
containers lazily rather than materialising the tree, and any hover budget
(§2.4's 4 ms) applies to the recursed model, not the flat one. Editing verbs are
the sharpest risk: `TransformObjects` rewrites operands in a content stream, and
a leaf inside a shared form is drawn everywhere that form is invoked — moving it
moves it in all of them, or requires cloning the form, and the engine must
decide which and say so. **That decision is the actual content of this
request**, more than the recursion is.

---

### 12. Everything else, honestly ranked below the line

Real, in §§3.5–3.7, 4.5, 8.1, and each worth doing; none of them changes whether
he can select the thing he is pointing at.

| Item | Size | Note |
|---|---|---|
| Arrow-key nudge, three tiers, repeating, coalesced undo | **S** | Genuinely absent (`canvas/keys.rs:334-338`). High value the day selection works; zero value before that |
| Handles and Delete stop vanishing at the Part rung on kinds with no parts | **XS** | `canvas/overlay.rs:221` + `canvas/selection/mod.rs:370-375`. Cheap, and it removes the "my double-click bricked the selection" state entirely |
| Refusal sentences for move and rotate | **S** | `canvas/moving.rs:733-742` has ten trace-only variants; rotate has no enum at all. Depends on item 3 |
| The resize refusal is written at the epoch it will be read at | **XS** | `canvas/resizing.rs:530-546` vs `app/actions/disclosure.rs:110-116` |
| `transform_preview` is called before grips are drawn | **S** | The engine capability exists and is not called (`canvas/resizing.rs:174-205`) |
| Right-click ▸ Select ▸ candidate list | **S** | The cheapest safety net in the design (§2.10); one menu over an already-computed list. Depends on item 4's list plumbing |
| Hover outline and hover readout | **M** | Needs the 4 ms budget, the memoisation and the two worst-case measurements of §2.4. The 355 ms figure that killed it (`canvas/interact.rs:1412-1427`) is the *text extraction*, not the vector hit test |
| `Alt`-from-centre, `Alt`-duplicate, `Alt`-touch-marquee | **S** each | Named as unbuilt at `canvas/constrain.rs:64-76` |
| Locks, padlock handles, per-object eye toggle | **M** | No model exists for either today |
| The eight unconsulted pick-filter rows | **S** | Either wire them or remove them — *"a row the operator can switch off but which nothing consults is a lie told once per session"* (`canvas/pick.rs:105-109`) |
| The three false strings of §2.6 | **XS** | Fix the text or fix the behaviour; do not ship either as it stands |
| Rename **Tool** or **Tools** | **XS** | Two surfaces one letter apart on the same screen is what made complaint 3 expressible in the words he used |
| Format tab activation rules (§4.6), tab placement | **S** | Microsoft's five MUST/SHOULD rules; the tab already appears, it just appears last and inert |
| Typed X/Y/W/H with units and arithmetic, no Apply button | **M** | Depends on item 5 |
| The Format tab's ~24 planned per-selection commands | **L** | `shell/manifest/mod.rs:1043-1137`. This is the *large* half of complaint 3 and it is genuinely a build, not a wire-up |

---

## 4. THE THINGS THAT ARE ALREADY RIGHT

**A rewrite that discards these is the failure mode this audit is most likely to
cause.** The operator's frustration reads as "tear it out and start again"; most
of what he touched is either correct or was correct and never reached him. Each
item below is named so that a later commit that removes it has to argue with
this list first.

**The Tool panel is right, and it is the panel he was looking at.** It is about
the armed tool and it says so (`panels/tool/mod.rs:23-40`,
`panels/tool/armed.rs:52-55`). It was built to answer an earlier complaint of
his — *"no side bar area showing what tool is active and its options"*
(`panels/tool/mod.rs:9-21`) — and it answers it. §9.11 is explicit: the
complaint is not that the Tool panel is wrong, it is that the panel that *should*
have answered did not exist. **Fixing the Tool panel would break a good one.**

**Manipulation is kind-agnostic and it works.** Move, resize and rotate all
operate on paths, text, images, form XObjects and multi-selections through one
`TransformObjects` path (`canvas/resizing.rs:285-352`,
`canvas/moving.rs:375-472`, `canvas/rotating.rs:206-291`). The old `NotAPath`
refusal — *"pdfcer cannot resize text or pictures"* — was already deleted on
2026-08-20. A driven run stretched the title text and squashed the whole-page
form to 82 % width. **The verbs are not the problem and must not be rewritten
while fixing selection.**

**The handle geometry is finished work.** Eight grips plus a drawn rotate
handle, screen-sized so they do not scale with zoom, 2 pt of grab slack,
mid-edge grips dropped below 24 pt on an axis (`canvas/handles.rs:100`, `:109`,
`:117`, `:335`). §3.1 reviews all of it and changes none of it. The drawn rotate
handle in particular is the right side of a three-way industry split — the
invisible hot-zone designs are the ones users file bugs about.

**`Shift` means one thing everywhere.** Axis lock on move, aspect on resize, 15°
on rotate (`canvas/constrain.rs:277-285`, `:235`, `canvas/rotating.rs:83`), and
the constraint announces itself on the status bar while it is active
(`app/status.rs:709-713`). A single constrain key across all three is worth more
than matching any one vendor exactly.

**Marquee-by-enclosure, and the marquee itself.** `MarqueeMode::Enclosed`
(`panels/objects/provider.rs:872-883`), decided on the record. It selected 24
objects at once on the header band **[driven]** — on this file it is the *only*
gesture that reached a useful set of objects, and it did so without ceremony.

**Path hit testing is the good half of the engine.** Bbox reject, fill interior
under the object's own winding rule, stroke and outline proximity, curves
flattened to 16 chords (`hit.rs:443-468`, `:502-513`). The `paint=none`
treatment is exactly right and was wrongly accused (§1). The 6 px tolerance and
its deliberate tightness against the 10 px snap radius — *a snap is an offer and
a selection is a commitment* — survives §2.2 untouched.

**`hit_test_point_all` is correct.** The engine already computes the full
front-to-back candidate list with a doc comment that describes precisely the
feature that has not been built on top of it
(`pdfcer-core/src/vector/hit.rs:126-135`). Item 4 is a binding, not a feature.

**The pick filter's design.** Eleven switchable classes, subtractive, ANDed with
mode capability, one click from anywhere, persisted
(`canvas/pick.rs`, `app/pickstore.rs:116-160`). It is more granular than
Illustrator's "Object Selection by Path Only" or CorelDRAW's "Treat all objects
as filled". It needs consumers for eight of its rows and a guarantee it stays
on-screen; it does not need redesigning.

**The Node tool.** `A` arms direct selection, anchors appear on the object,
clicking an anchor selects it (`canvas/tool/mod.rs:195-222`) — Illustrator's
Direct Selection, Inkscape's Node tool, CorelDRAW's Shape tool, and the header
carries the operator's own rule about not inventing where a convention exists.

**`Esc` cancels an in-flight drag before anything is written**
(`canvas/gesture/mod.rs:249-251`). Small, and the reason a mis-aimed drag on a
CAD sheet is not a disaster.

**The cursor work.** The custom two-tone crosshair, built because the stock
white one was invisible on white paper; the I-beam rotated to match text
orientation; the `Grab`/`Grabbing` pair so a pan that has run out of scroll is
distinguishable from a pan that is not working (`canvas/cursor.rs`,
`canvas/tool/mod.rs:517-560`). §8.1 preserves all of it, including the rule that
`CanvasTool::Select` returns `None` so grip cursors underneath are not
overwritten.

**The disclosure and status machinery, and its priority reasoning.**
`constrain::caption` and `filter::empty_note` are the exact shape every refusal
sentence in §3.7 needs, and the reasoning at `app/status.rs:788-791` — *"a
decline explains why one gesture did nothing, while [the empty filter note]
explains why EVERY gesture will"* — is already correct and extends without
change.

**The no-nagging rule.** No modal dialogs, no red badges, no toasts for
refusals (`panels/tool/mod.rs:63-78`). It is his own standing instruction and
§3.7 is written inside it.

**The OCR dialog's honesty.** The intro sentence, the confidence sentence
(`no_confidence()`), the disclosure list, the deliberate absence of an icon, the
disclose-before-write ordering (`dialogs/ocr.rs:16-34`), and the regression
guard that hashes the source before and after
(`tools/ui-verify/src/checks/ocr.rs:24-38`). §7.7 keeps every one of them and
says this dialog is more honest than any of the six products surveyed. **The OCR
work is an expansion of scope and options around a good core, not a
replacement.**

**And the thing that made this audit possible in a day:** the code carries its
own reasons. `canvas/pick.rs:201-209` diagnosed the whole-page-selection problem
before the operator hit it. `canvas/resizing.rs:174-205` documents its own
inert-grip defect. `panels/objects/mod.rs:129-141` states in plain words that
the panel cannot select. `canvas/interact.rs:1412-1427` records the measurement
that killed hover. Nothing in this document is a discovery; it is a collation of
things the codebase already knew about itself. **That discipline is the asset —
do not let a rewrite trade it for speed.**

---

## 5. THE ONE CHANGE

> **Recurse into form XObjects, and stop a form from being the answer to a
> click.** (Work item 11 — `pdfcer-core`.)

**Why this one and not a cheaper one.** Every other item on the list improves
how well the operator can select **the objects that are not his drawing**. On
`the conformance suite’s composite page` the four form XObjects on page 1 contain
the entire visible body of the sheet; the twenty-four objects reachable outside
them are a footer, a title, three text blocks and the left logo. On a SolidWorks
export the same structure holds per view — the direct observation in
`HOW_IT_SHOULD_WORK` §2.6 is that Acrobat, clicking a horizontal line in an
orthographic view, selects **that one path**, because Acrobat's editable-item
model cannot return a form wrapper at all.

A perfect selection-fed Properties panel, `Alt`+click cycling, a status readout
and select-and-move in one gesture, all shipped together, would still leave him
clicking on his drawing and getting object #26. **This is the only item on the
list that cannot be worked around in the GUI**, because the thing he is pointing
at is not in the object model to be selected. Items 1–9 make the program
coherent; item 11 makes it *applicable to his files*.

**What it does not fix, so the choice is made with eyes open:** it does not give
him properties for a selected text object (item 5 and the Format band), it does
not make dragging an unselected object move it (item 6), and it does nothing at
all for OCR (items 7, 8, 10). It also carries the largest blast radius on the
list — object indices renumber, counts explode, and the shared-form editing
question has to be answered. If the decision is instead to buy visible progress
this week, the honest alternative is **items 1, 2, 3, 4 and 8 as one batch**:
five small changes, no engine work, and by the end of them a click says what it
hit, `Alt`+click reaches past it, a placed image is selected and resizable, the
Properties button stops answering the wrong question, and OCR stops refusing
forever after the first save.

But if it is one thing: **it is the recursion.** Everything else is a binding.
This is the missing model.

---

## Appendix — the operator's seven complaints, adjudicated

| # | His words | Verdict |
|---|---|---|
| 1 | *"when I click on one of the objects all I get is the page selected"* | **Correct, and worse than he thinks.** It is not the page — it is a form XObject whose `/BBox` is slightly *larger* than the media box, painted second-to-last, winning every hit test at every point. And the objects he is aiming at are not in the model at all. Not a misunderstanding in any part. |
| 2 | *"When I double click on an object it doesn't select"* | **Correct.** Double-click descends `Object → Part` on an object that has no parts (`panels/objects/provider.rs:343-349`), so the outline does not change, the handles vanish (`canvas/overlay.rs:221`), Delete stops working, a body drag refuses to a trace line, and nothing on screen says any of it. He found a genuinely dead state. |
| 3 | *"the Tool tab doesn't switch to giving me the editable stuff for that object"* | **The complaint is right; the panel he named is not the culprit.** The Tool panel is correctly about the armed tool (§9.11) and must not change. What is missing is the surface that should have answered — and the program made the confusion: two surfaces one letter apart (**Tool** dock, **Tools** ribbon tab), a Format tab that appears last and never activates, and a "Properties" button that opens the *document's* properties. **This is a misunderstanding manufactured by the interface, not by the operator.** |
| 4 | *"if I add an image I Expect to click on it to resize but dragging doesn't resize"* | **Correct, via two causes.** The image arrives unselected (`app/actions/apply.rs:409-444`), and a press-drag on an unselected object is a marquee (`canvas/gesture/meaning.rs:736`) which on release selects nothing — so the image appears to reject the drag *and* to deselect. Resizing itself works: a driven corner-grip drag resized the image. Placement also asks for millimetre coordinates in a separate OS window; there is no draw-a-box placement. |
| 5 | *"how do I OCR more than one page? Why does the tool stop at one?"* | **Correct, and it is a shell limitation only.** `Request` carries one `usize`; there is no loop anywhere. No engine change is needed to fix it. The single-page design may have been reasoned from Acrobat's *lazy per-page OCR inside Edit PDF*, which is a different feature from Acrobat's document-scoped `Scan & OCR ▸ Recognize Text` (§7.2). |
| 6 | *"Why do I have to save a copy instead of just go back into my pdf and save over it?"* | **Working as designed, and the design is wrong — and it was derived from his own instruction.** His rule of 2026-08-14 governs **Read** mode; the code applies it to the whole program in every mode (`dialogs/ocr.rs:64`), which is a stronger rule than he asked for and is the one he is now complaining about. §9.17: rules belong to their owner. The strong reading needs an engine verb that does not exist. |
| 7 | *"Where is the option to select more than one page?"* | **Correct.** There is no page-range control, and a tested page-range parser sits one directory away, unused (`dialogs/print/tabs.rs:161`). The thumbnail panel he already uses for reordering is the natural second route and offers nothing. |
| — | *"Somehow we have the most convoluted system"* | **Half right, and worth being precise about.** The code is not convoluted; each subsystem is coherent and argues its own case in-tree. The convolution is at the seams: three parallel notions of "the thing I am working on", a capability flag that gates selection as well as mutation, a rung that silently removes verbs, a filter row that is the only cure for a disease nothing names, and several complete features whose only consumer throws their result away. **Each was defensible alone. Nobody owned the joins.** The most telling artifact is a test whose name pins the severance in place — `the_panel_focus_has_not_quietly_become_a_selection` — guarding a binding that was promised *"in the commit that makes properties read the canvas's selection, and not before"*, for a commit that never came. |
