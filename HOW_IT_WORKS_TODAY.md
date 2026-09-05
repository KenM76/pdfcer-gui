# HOW IT WORKS TODAY

An accurate, unflattering manual for pdfcer-gui exactly as it behaves in the
current build. This is a description, not a defence. Nothing here proposes a
fix.

Paths written bare (`canvas/input.rs:144`) are relative to
`D:/Dev/pdfcer-gui/crates/pdfcer-gui/src/`. Engine paths are written in full from
`D:/Dev/pdfcer/`.

Behaviour marked **[driven]** was observed by launching
`D:/Dev/pdfcer-gui/target/release/pdfcer-gui.exe` against
`D:\Dev\temp\pdfcer\the conformance suite’s composite page\the conformance suite’s composite page`;
traces are in `D:/Dev/pdfcer-gui/evidence/audit/`. Behaviour marked
**[UNVERIFIED]** was not confirmed.

---

## 0. If you just want to select an object, here is literally what you must do today

Cold start, default install, the the conformance suite’s composite page test file open. To select one visible
thing on page 1 — say the magenta title text, or a piece of the drawing — the
complete sequence is:

1. **Look at the mode selector at the top right. It says Read.** It says Read
   every launch, on every document, forever. The active mode is not persisted;
   `assemble` activates `modes.first()`, which is `"read"`
   (`app/modes/mod.rs:594-598`, `app/modes/mod.rs:187`; pinned by the test at
   `app/modes/mod.rs:783`, whose comment is *"the session opens in Read"*).
   In Read, `caps.edit_content` is false, so `textsel::takes_the_press` returns
   true for the Select tool (`canvas/textsel/gate.rs:272-274`) and the click is
   consumed as a **text sweep** at `canvas/clicking.rs:348` before content
   selection is ever reached. **[driven]** A click on a coloured patch produced
   `canvas-text-selection via=clear page=0 chars=0 quads=0` and no
   `canvas-selection` line at all. Nothing on screen explains this.
2. **Click "Edit" in the mode selector.** The ribbon does *not* follow you —
   it stays on whatever tab was open (File). **[driven]**
3. **Confirm the Select tool is armed.** It is the default
   (`canvas/tool/mod.rs:191`), but any armed tool — text, pen, polygon, sticky,
   form field, measure — sits above content selection in the click ladder
   (`canvas/clicking.rs:141`, arms 1–8) and will take the press instead.
   Holding Space borrows the Hand and blanks the pointer frame entirely
   (`canvas/interact.rs:370-374`).
4. **Click the object.** On this file you now get a blue rectangle with eight
   handles around **the entire sheet**. **[driven]** The outline is
   `[[609.8 183.4] - [1150.2 901.3]]`; the page rect published the same frame
   is `[[609.5 183.0] - [1150.5 901.0]]`. That is not the page. It is object
   #26, a form XObject whose `/BBox` (0.362, −0.4331 → 595.638, 790.433) is
   slightly *larger* than the 595.276 × 790.866 media box, and which is painted
   second-to-last so it wins every hit test on every pixel of every page.
5. **Do not double-click.** It will not get you inside. It descends
   `Object → Part` (`canvas/selection/mod.rs:814-857`), the outline does not
   change, and the handles disappear (§3). **[driven]** Four further
   double-clicks just oscillate Object → Part → Object → Part.
6. **Find the status bar at the bottom right and click the button labelled
   "Select".** (`app/status.rs:861-863`.) A popup opens headed *"Selectable"*.
7. **Find the row labelled "Blocks" and switch it off.** "Blocks" is the
   operator-facing label for `PickClass::FormXObject` (`text/pick.rs:164`).
   Its tooltip is the only text in the application that describes the problem:
   *"Off: clicks pass through blocks — title blocks, borders, and anything else
   stored as one nested drawing"* (`text/pick.rs:193-196`). Nothing tells you
   that the thing you just selected **is** a block, or that 27 objects are
   underneath it.
8. **Close the popup and click the object again.** Now it selects.
   **[driven]** The same pixel that selected the whole-page form with Blocks on
   selects the title text object with Blocks off — outline
   `[[758.3 191.7] - [1001.1 215.8]]`.

That is eight steps, two of which require knowing a diagnosis the software
never states.

**And on this file, step 8 still fails for most of the page.** The four form
XObjects on page 1 contain the entire visible body — every test patch, every
swatch, both Overprint panels, the manta-ray images, the right-hand the conformance suite’s composite page logo.
The decomposer does not recurse into a form
(`panels/objects/decompose.rs:400-404` in the engine tree:
*"9a does NOT recurse into the form's own content"*). Those marks are not in the
object model, not in the Objects panel, not hit-testable, not selectable, not
editable. With Blocks off, clicking them selects **nothing**. **[driven]**
A click on blank-looking paper at pdf (79.87, 585.96) gave
`canvas-selection via=click mod=false sel=0`.

The objects you *can* reach on page 1 with Blocks off are the top-level ones: a
footer text run, a magenta title, three text blocks, and twenty-one path
objects that make up the left logo. A marquee over the header band selected 24
of them at once. **[driven]**

---

## 1. The selection model as implemented

### Three rungs, one ladder

`SelectionState` has a `level`, which is one of `Object`, `Part`, `Node`
(`canvas/selection/mod.rs`). A click lands at `Object`. A double-click descends
one rung. Escape climbs back out. There is no other way to change rungs.

A `Selection` entry is four integers: page, object (a paint-order index into
the page's decomposition), subpath, node. It cannot name an annotation and it
cannot name a form field; those are separate, parallel selection states
(`doc.selection.annot()`, `doc.selected_field`).

### What "an object" is

The engine decomposes a page into a flat `Vec<VectorObject>` in paint order.
Three variants only: `Path`, `Text`, `Image`
(`D:/Dev/pdfcer/crates/pdfcer-core/src/vector/hit.rs:433-439`). `Image` covers
inline images, image XObjects **and form XObjects** alike. A form XObject is
therefore modelled as a single opaque picture, and everything drawn inside it
is invisible to the whole selection system.

### The hit test is not uniform, and the non-uniformity is the bug the operator hit

```rust
fn object_hit(obj: &VectorObject, point: Point, tolerance: f64) -> bool {
    match obj {
        VectorObject::Path(p) => path_hit(p, point, tolerance),
        VectorObject::Text(t) => text_hit(t, point, tolerance),
        VectorObject::Image(i) => i.page_bbox.inflate(tolerance).contains(point),
    }
}
```
(`D:/Dev/pdfcer/crates/pdfcer-core/src/vector/hit.rs:433-439`)

- **Paths are ink-accurate.** Bbox reject, then fill-interior under the object's
  own winding rule, then stroke/outline proximity; curves flattened to 16
  chords (`hit.rs:443-468`, `hit.rs:502-513`, `hit.rs:78`).
- **Text is per-run**, not per-object: `t.runs[i].bounds`, falling back to the
  object box only when there are no runs (`hit.rs:~225`).
- **Images and form XObjects are a bare inflated rectangle.** One line. No
  geometry, no alpha, no clip, no ink (`hit.rs:437`).

So the one object kind that most often spans a whole CAD sheet is also the one
kind tested as a solid opaque rectangle. That is the mechanism.

### Only the front-most hit is ever considered

`topmost_allowed` (`canvas/input.rs:144-158`) calls the engine's full
front-to-back candidate list and then:

```rust
targets.hit_test_all(page_index, point, tolerance)
    .into_iter()
    .find(|target| match targets.object_class(page_index, *target) {
        Some(class) => filter.allows(class),
        None => true,
    })
```

`.find()` — first allowed entry, stop. The rest of the list is discarded. There
is no second query, no cursor, no cycle. `hit_test_all` has exactly one
non-test consumer in the GUI and this is it.

**The codebase already knows this fails.** `canvas/pick.rs:201-209`, the doc
comment on `PickClass::FormXObject`: *"this is the single most common cause of
'why is the selection box so big?' on a CAD sheet: a title block or a border
that is one object holding a hundred visible marks."* The shipped mitigation is
a global class toggle, not a per-click escape.

### Tolerance

`SELECT_SCREEN_TOLERANCE_PX = 6.0` screen pixels ÷ zoom
(`canvas/mapping.rs:94, 231-233`).

### Selection outline

`SelectionState::outline_rect` (`canvas/selection/mod.rs:720-726`) →
`targets.bounds` → `page_bbox()` mapped to canvas
(`panels/objects/provider.rs:892-898`). The outline is always the object's
bounding box, at every rung, for every kind. Descending to Part or Node does not
shrink it unless a real subpath was entered.

### The `paint=none` hypothesis — tested and cleared

Page 1 also carries a full-page path with `paint=none` (the `n` operator). It is
**not** the culprit. `path_hit` gives an unfilled, unstroked path a proximity
band of `tolerance` alone, so it is selectable only within 6 px of its outline —
i.e. at the page edges (`hit.rs:453-467`; intent stated at `hit.rs:23-25`).
Confirmed empirically: it appears in the candidate list at (0.5, 0.5) and
(595, 790) and is absent at the page centre. **[driven]** With Blocks off,
clicks on blank paper select nothing rather than selecting it.

### The measured page

`pdfcer object-list --page 1` on the the conformance suite’s composite page file: **28 objects (0–27)**, not
29. `objects=28 paths=21 text=3 images=0 forms=4`. Index 1 and index 26 both
span the page; index 26 is painted later and wins.

```
at=297.6,395.4  winner index=26 kind=form   candidates: [26, 1]
at=100,762      winner index=26 kind=form   candidates: [26, 19(path), 1]
at=50,22        winner index=26 kind=form   candidates: [26, 1, 0(text)]
at=0.5,0.5      winner index=26 kind=form   candidates: [26, 2(paint=none), 1]
```

Object 26 wins at **every point tested**, including directly over visible
linework and directly over the footer text. Pages 2–6 have the identical
structure. This is not a corner case; it is total.

---

## 2. Selecting

### Single click

Ladder B (`canvas/clicking.rs:141`; precedence stated at `clicking.rs:17-32`).
The first arm that answers consumes the click. **A click is exactly one thing,
never two.**

| # | arm | guard | line |
|---|---|---|---|
| 1 | Node tool direct pick | `active_tool.is_node() && caps.edit_content` | `clicking.rs:258` |
| 2 | text caret | text tool, or `is_text() && caps.edit_content` | `clicking.rs:254-257, 272` |
| 3 | annotation under pointer | Select tool && `caps.author_markup` | `clicking.rs:189-194, 334` |
| 4 | **text sweep** | `textsel::takes_the_press(tool, caps)` | `clicking.rs:348` |
| 5 | vertex markup | PolyLine/Polygon armed | `clicking.rs:400` |
| 6 | form field placement | `CanvasTool::Form(_)` | `clicking.rs:411` |
| 7 | sticky note | `CanvasTool::TextAnnot(_)` | `clicking.rs:445` |
| 8 | measure pick | `active_tool.measure_kind()` | `clicking.rs:476` |
| 9 | **content selection** | everything above declined | `clicking.rs:496-501` |

Rung 9 is the only route to selecting a path, text, image or form object.
**Arm 4 sits above it and is unconditionally true in Read and Review mode.**

### Double click

Only reaches anything if `PressMeaning.click` is true, i.e.
`caps.edit_content || textsel::takes_the_press(...)`
(`canvas/gesture/meaning.rs:769-772`, swallowed otherwise at
`canvas/gesture/mod.rs:301-311`). egui fires `clicked` on the first release and
`double_clicked` on the second, so an Edit-mode double-click is **two** trips
through the ladder: select, then descend.

| tool | Read | Review | Edit |
|---|---|---|---|
| Select | takes a **word** (text sweep, `clicking.rs:348-383`) | annotation if hit, else word | **descend one rung** |
| Node (`A`) | inert | inert | `click_direct` — **ignores `double` entirely**; a double is two identical picks |
| Text / caret | `double` is not passed through at all (`clicking.rs:272-294`) | same | same |
| PolyLine / Polygon | n/a | double **ends the shape** | same |
| Measure radius/diameter | n/a | double **ends the measurement** | same |
| Sticky / form field | n/a | second placement | second placement |
| Hand / Space | nothing | nothing | nothing |

`descend` (`canvas/selection/mod.rs:814-857`): miss → clear; at Object → Part;
at Part on the same object → Node; at Node → `return` (*"Nothing is below a
point."*).

**On an image or form XObject, descending is destructive and silent.**
`part_kind` returns `None` for those (`panels/objects/provider.rs:343-349`), so
`hit.part` is always `None`, so the Part entry names no part. The outline does
not change — it falls back to the object bbox — but §3's handles vanish and
§3's move refuses.

### Shift+click

Toggles membership of the same topmost object
(`canvas/selection/mod.rs:733-740`). It is not a depth gesture. `shift` is read
live every frame at `canvas/interact.rs:362`.

### Alt+click, Ctrl+click

**Do not exist.** The only modifier read anywhere on the selection path is
Shift. There is no `modifiers.alt` in `canvas/`. Ctrl appears in the canvas only
inside text editing and text-selection clipboard handling
(`canvas/textedit/keys.rs:247`, `canvas/textsel/clipboard.rs:138`).

★ **A documentation claim that is false.** `pdfcer object-list --help`
describes `--all-hits` as printing *"the same list the GUI's Alt+click cycling
steps through."* There is no Alt+click cycling.
`D:/Dev/pdfcer/crates/pdfcer-core/src/vector/hit.rs:41-48` and `target.rs:85-88`
repeat the same claim.

### Repeated clicking at one point

Does not cycle. `click_at_object_rung` (`canvas/selection/mod.rs:729-744`)
replaces the selection with the same topmost object every time. No per-point
cursor is stored anywhere.

### Right-click

Selects the same topmost object, or leaves the selection alone
(`canvas/menus.rs:172-199`). The menu offers `view.zoom_selection`,
`format.properties`, `format.delete` (`shell/menus.rs:305`, `:1035`). There is
no "select behind".

### Marquee

Press-drag on **anything not already selected** is a marquee, not a drag of that
object:

```rust
(None, None) => Some(DragKind::Marquee(MarqueeIntent::Select)),
```
(`canvas/gesture/meaning.rs:736`)

On release it **replaces** the selection with whatever the band **fully
enclosed** — enclosed, not touched (`canvas/interact.rs:676-687`,
`MarqueeMode::Enclosed` at `panels/objects/provider.rs:872-883`).

### Clicking empty paper to deselect

Works in principle (`canvas/selection/mod.rs:387-390`) and is **unreachable on
this file**, because there is no empty paper as far as the hit test is
concerned. After a double-click the rung is Part, every subsequent click finds
object 26 again, `click_inside` (`canvas/selection/mod.rs:754-780`) falls
through to `click_at_object_rung`, and object 26 is re-selected. Only **Escape**
clears it (`canvas/selection/mod.rs:607-637`).

### The Objects panel is not a selection route

`panels/objects/mod.rs:129-141`, verbatim: *"Clicking an object row points the
`properties` panel at it, and that is all it does — no canvas highlight, no
multi-select, no Shift+click, no scroll-to-reveal, because there is no
selection model to reveal into."* Part and point rows are **not clickable at
all**. **[driven]** Clicking row `#25 Text` filled the Properties panel and
produced no `canvas-selection` event and no outline on the canvas.

Conversely, selecting on the canvas does **not** highlight the row. The Objects
tree highlight is `focused == Some(index)` (`panels/objects/mod.rs:371`), and
`focus` is written only by a row click.

---

## 3. Manipulating

### The handles

Eight square resize grips (compass-named), plus an implicit `Grip::Move` body
target with no drawn square, plus a rotate handle drawn as a small circle on a
stem 20 pt above the top edge (`canvas/handles.rs:122-181`, `:335`,
`canvas/overlay.rs:507-541`).

- Grip square is `GRIP_SIZE_PX = 8.0`, fixed at every zoom
  (`canvas/handles.rs:100`); grab slack `2.0` (`:109`).
- Mid-edge grips are dropped on an axis shorter than 24 screen points
  (`canvas/handles.rs:117`, `:363-380`).
- The box is the **union** of every selected entry's outline
  (`canvas/selection/mod.rs:306-311`).

**They are painted on exactly one condition:**

```rust
if selection.level() == SelectionLevel::Object {
    draw_grips(painter, visuals, box_);
}
```
(`canvas/overlay.rs:221`)

The hit test agrees by construction: `grip_at(bounds, p, at_object_rung && ...)`
(`canvas/pressing.rs:141-153`).

| selection | grips? |
|---|---|
| path, text, image, form XObject at Object rung | **yes**, 8 + rotate — the predicate is the rung, never the kind |
| multi-select | **yes**, one set around the union |
| **anything at Part or Node rung (i.e. after a double-click)** | **no** |
| **any annotation** (stamp, note, ce dimension, markup) | **no** — `draw_selection` strokes the `/Rect` and returns before the grip block (`canvas/overlay.rs:184-189`) |
| **form field** | **no** — form fields are not in the content selection at all |

### Move

`canvas/moving.rs:375-472`. Dispatch is by rung, then by whether every selected
object is a path:

| rung | condition | verb |
|---|---|---|
| Object | all paths | `move_objects` — rewrites coordinate operands (`moving.rs:423`) |
| Object | **any** non-path (text, image, form) | `transform_objects` with a translate matrix (`moving.rs:424`) |
| Part | entered part is a subpath | `move_subpath` |
| Node | 1 anchor / ≥2 anchors | `move_node` / `move_nodes` |

So **text and images do move.** **[driven]** Dragging inside the whole-page
form selection moved it: `canvas-move page=0 level=Object dx=121.18 dy=-66.10
action=Vector(TransformObjects { objects: [26], … })`. The entire body of the
page slid off the sheet edge in one casual drag on what looked like blank paper.

### Resize

`canvas/resizing.rs:285-352`. The function **asks the provider nothing about the
object kind** — it needs only a provider and a non-empty selection
(`resizing.rs:302-308`). Paths, text runs, images, form XObjects and
multi-selections are all resized through one `TransformObjects` command
(`app/actions/vector.rs:532-556` → `D:/Dev/pdfcer/crates/pdfcer-core/src/edit.rs:7607`).

Only three refusals exist: `NothingSelected`, `NoObjectModel`, `Degenerate`
(factor ≤ 0.001, or a zero-extent grip box) (`canvas/resizing.rs:154-172`,
`:230-236`, `:268-272`). The old `NotAPath` refusal ("pdfcer cannot resize text
or pictures") was deleted on 2026-08-20 (`text/resizing.rs:41-68`).

**[driven]** Dragging the SE grip of the title text object gave
`resize-commit grip=SouthEast sx=1.4129 sy=3.8771` and the text visibly
stretched. Dragging the East grip of the whole-page form gave
`resize-commit grip=East sx=0.8206 sy=1.0000` and squashed the entire nested
drawing to 82% width.

**Stroke width is not scaled** — a stated CAD decision, disclosed rather than
fixed (`text/resizing.rs:78-104`).

**A grip is offered for objects that can never be transformed.**
`canvas/resizing.rs:174-205` says so plainly: the engine's `transform_preview`
exists and is not called, so an object with a singular CTM gets eight live-
looking grips and a drag the engine refuses on release.

### Rotate

`canvas/rotating.rs:206-291`. Same kind-agnosticism. Pivot is the selection
centre (`canvas/handles.rs:317`). Minimum travel 0.1° (`rotating.rs:91`).
**There is no `Refusal` enum for rotate at all** — it returns `None` and writes
nothing for no bounds, no mapping, degenerate ray, under-travel, empty
selection, or no page (`rotating.rs:220-256`).

### Select-and-drag in one gesture: does not exist

`grip` is derived from the **current** selection's cached outlines, evaluated at
press time before any click is applied (`canvas/pressing.rs:129-138` →
`canvas/overlay.rs:124`). The gesture machine makes click and drag mutually
exclusive on one interaction (`canvas/gesture/mod.rs:29-43`, `:300-311`).

**So pressing on an unselected object and dragging draws a rubber band.** On
release the band replaces the selection with whatever it fully enclosed —
usually nothing, because the band started inside the object. The visible result
is *"I dragged the picture and it did not move or resize, and it deselected."*
**This is the most likely reading of "dragging doesn't resize."** Two gestures
are always required: click, then drag.

The cursor mirrors the asymmetry: over a **selected** object's body you get
`CursorIcon::Move`; over an **unselected** object, the default arrow
(`canvas/interact.rs:429`, `canvas/tool/arm.rs:156`).

### Modifiers during a drag

| modifier | move | resize | rotate | Bézier handle | dimension vertex | marquee |
|---|---|---|---|---|---|---|
| **Shift** | axis lock, off-axis set to exactly 0 (`canvas/constrain.rs:277-285`) | preserve aspect (`constrain.rs:235`, applied `resizing.rs:452-456`) | snap total turn to 15° (`rotating.rs:83, 129, 156-161`) | lock to the anchor's axis | lock to axis from press | extends/toggles selection |
| **Alt** | nothing | nothing | nothing | nothing | **suspends snapping** (`interact.rs:1083`) | nothing |
| **Ctrl / Cmd** | nothing | nothing | nothing | nothing | nothing | nothing |

Alt-scale-about-centre, 45° translate lock, and Ctrl-disables-snapping are named
as unbuilt at `canvas/constrain.rs:64-76`. Bézier symmetry break/restore is
recorded as a gap at `canvas/handledrag.rs:81-83`.

**Escape** cancels an in-flight drag before anything is written
(`canvas/gesture/mod.rs:249-251`).

### Arrow-key nudge

**Does not exist.** `canvas/keys.rs:334-338` reads exactly three keys:
`Escape`, `Delete`/`Backspace`, `Tab`. Arrow keys are handled only by the text
caret (`canvas/textedit/keys.rs:330, 402-406`). Grepping `nudge` across the
crate returns prose in doc comments only — no code. There is no step size
because there is no nudge.

### Delete

The only keyboard verb on a selected object, and it works **only at the Object
rung**:

```rust
pub fn deletable_objects_on(&self, page: usize) -> Vec<usize> {
    if self.level != SelectionLevel::Object { return Vec::new(); }
    self.object_indices_on(page)
}
```
(`canvas/selection/mod.rs:370-375`)

So **Delete stops working after a double-click**, and so does `format.delete`.
The refusal goes to a trace line with `reason=no-verb-for-rung`
(`canvas/keys.rs:655-664`) and nothing else.

### After a double-click on an image or form: the complete state

Outlined, looks selected, and:
- no handles painted (`canvas/overlay.rs:221`);
- no handles hit-testable (`canvas/pressing.rs:152`);
- body drag returns `Err(Refusal::NoPartEntered)` (`canvas/moving.rs:429`) →
  trace only;
- Delete does nothing;
- nothing on screen says any of this.

Escape returns to the Object rung.

---

## 4. Properties and panels

### The Properties panel has five sections and two independent notions of "the thing I am looking at"

`panels/properties/mod.rs:257-294`:

```
markup::section        (271)  ← doc.selection.annot(), kind == Markup
dimension::section     (272)  ← doc.selection.annot(), kind == CeDimension
formfield::section     (284)  ← doc.selected_field
geometry::section      (285)  ← canvas selection, single object, must be a Path
object_section         (286)  ← PanelsState::focus()  — NOT the canvas selection
info::section          (293)  ← always
```

`focus` is written by exactly one thing outside tests: a click on an
Objects-panel tree row (`panels/objects/mod.rs:376-377`, `:450-451`). It is
explicitly not a selection (`panels/mod.rs:715-741`), and its own docs promise
it will be *"deleted in the commit that makes `properties` read the canvas's
selection, and not before"*, pinned by a test named
`the_panel_focus_has_not_quietly_become_a_selection`. The canvas selection model
has since landed (`app/conditions.rs:105`) and the field was not deleted.

**Net effect: clicking on the page and clicking in the Objects panel set two
different variables, and nothing syncs them in either direction.**

| Selection | What Properties shows |
|---|---|
| nothing | *"Pick a row in the Objects panel to see what it is made of."* then document metadata |
| **text object on canvas** | **nothing about the text** — document metadata only |
| **image on canvas** | **nothing about the image** — document metadata only |
| path on canvas | X / Y / W / H fields and an Apply button |
| markup annotation | Colour, Line width, Opacity |
| form field | Name, Type, Page, Value, Flags (read-only) + rename + delete |
| ce dimension | Group picker, measured value, display toggle, per-dimension overrides — the richest surface in the app |
| an Objects-panel **row** | twelve read-only fields, headed *"Nothing here can be changed in this build."* |

### Why text and images get no geometry fields

`geometry::section` has four `return false` gates
(`panels/properties/geometry.rs:337-352`). The decisive one is the fourth:

```rust
let points = provider.object_node_points(object);
let Some(bounds) = Bounds::of(&points) else { return false; };
```

and `object_node_points` is path-only:

```rust
let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
    return Vec::new();
};
```
(`panels/objects/provider.rs:677-680`)

So the typed X/Y/W/H — the fallback for when dragging fails — is unavailable for
**exactly the two kinds the operator named**: text, and an inserted image. The
section is not drawn at all: no heading, no greyed field, no explanation.

For a path, the fields are `DragValue`s editing a **draft** at 0.5 pt per pixel
of scrub; nothing commits until **Apply**, which emits at most a `MoveSelection`
then a `TransformObjects` (`geometry.rs:435-473`, `:495-520`). Units are PDF
points only, no picker (`geometry.rs:74-81`). There is **no rotation field**
(`geometry.rs:71-75`).

### The Tool panel does not read the selection. At all.

`panels/tool/mod.rs:175-211`. Its whole input is the armed `CanvasTool` plus a
disclosure slot. The word `selection` does not appear in its three source files
except in prose. Its header states the boundary: *"this panel is about the next
gesture"* (`panels/tool/armed.rs:52-55`).

It draws: **The pointer** (two lines, one of two sentences chosen by
`edit_content`), then either a static list of six tool buttons (when nothing is
armed) or an identity/stage/options block for the armed tool, then **What pdfcer
worked out** (the last edit disclosure).

**Options exist for exactly one tool** — Add Text, which offers Font / Size /
Colour for the *next* text you type (`panels/tool/armed.rs:96-103`). Every other
armed tool draws no options. The markup pen's colour and width are deliberately
absent because `Panel::show` cannot reach the pen (`panels/tool/mod.rs:113-126`).

**[driven]** With the magenta title text selected, the Tool panel read, verbatim
and unchanged from its empty state:

> **The pointer** — Drag marquees objects on the page; click selects one. Hold
> Space to move the paper.
> **Tools** — `[Select]` Pick a shape, then drag it… `[Points]` Show a shape's
> points…

### There are two things called "the Tool tab", and neither reads the selection

| Surface | Caption | Where | Driven by |
|---|---|---|---|
| the **Tool** dock panel | `"Tool"` (`text/commands/view.rs:365-371`) | right dock, Edit and Review | the armed tool |
| the **Tools** ribbon tab | `"Tools"` (`text/ribbon.rs:93-95`) | ribbon, Edit only | nothing — static batch/font/diagnostic commands |

They differ by one letter and sit on the same screen.

**Verdict on "the Tool tab doesn't switch to giving me the editable stuff":
not implemented.** There is no code path by which either surface could respond
to a selection.

### The contextual Format tab exists, appears last, does not activate, and holds two commands

```rust
pub(super) const VISIBLE_WHEN: &str = "selection.any";

Tab::new("format", ribbon::tab_format())
    .with_visible_when(VISIBLE_WHEN)
    .with_groups([group("selection", ribbon::group_format_selection(),
        [command("format.properties"), command("format.delete")])])
```
(`shell/manifest/format.rs:122-131`)

- It is appended **last** in the visible-tab list
  (`crates/egui-shell/src/ribbon/tabs.rs:247-253`) — far right, after Tools.
- It **does not auto-activate**; `resolve_active` keeps the current tab
  (`tabs.rs:283-291`). **[driven]** Selecting an object published
  `ui-rect name=ribbon.tab.format` while the band stayed on File.
- `format.properties` does not open an editor. It pushes
  `Action::Command("file.properties")` (`app/dispatch.rs:819-824`) — i.e. it
  re-shows the same read-only right-hand panel. **[driven]**
  `ribbon-command-invoked id=format.properties` →
  `panel-shown id=file.properties mounted=already`.

RIBBON_IA.md §5.8 specifies six per-selection column sets. All ~24 of the
implied commands are in `manifest::PLANNED` as unbuilt
(`shell/manifest/mod.rs:1043-1137`), **including `format.font`,
`format.font_size`, `format.spacing`, `format.alignment`** — precisely the set
for the operator's text case.

### What selecting a text object actually changes on screen

An outline and grips on the canvas; a greyed Delete becomes live; one new tab
appears at the far end of the ribbon carrying two commands. **Nothing in either
panel the operator is looking at.**

**[driven]** Across the whole of run 2 the trace carries 113
`properties-panel` lines and **every one says `object=25`** — the row clicked in
the panel — through five different canvas selections and two deselections.

---

## 5. Tools vs objects

> ### ⚠ SUPERSEDED 2026-09-04 — `OPERATOR_REQUESTS.md` O123
>
> **There is no Tool panel.** `Panel::Tool` and `view.panel_tool` were
> retired. What is on screen instead:
>
> * **A one-line strip the right dock reserves above its columns**
>   (`crate::app::toolstatus`, `egui_shell::dock::banner`): the armed tool's
>   name, one sentence, and *Put this tool down* — the last of those absent
>   while `Select` is armed, because Select is the resting state.
> * **Its live controls are in Properties**
>   (`crate::panels::properties::tool`): the text pen's face, size and colour,
>   the circular measure's pick list, the three resize switches.
> * **Its disclosure block is in Properties**
>   (`crate::panels::properties::disclose`), at the top of the body.
>
> The three-way dispatch split described below was already closed on
> 2026-08-26 — a row click raises `Action::SelectObject` and Properties reads
> the same canvas selection — and the paragraphs are left standing because the
> *shape* of the gap they describe is still the thing to design against.

The Tool panel was the only panel whose subject was the operator rather than the
document. It was built to answer an earlier complaint — *"no side bar area
showing what tool is active and its options"*. It answered that question and only
that question.

Object properties live in the **Properties** panel, and only for a path
(geometry), an annotation, a form field, a ce dimension, or an Objects-panel
row (read-only facts). There is no surface anywhere that shows a selected text
object's font, size, colour, spacing or alignment, and no command that would
change them.

The dispatch is therefore split three ways with no bridge:

- **armed tool** → Tool panel
- **`focus`** (Objects-panel row click) → Properties ▸ Object properties
- **`doc.selection`** (canvas click) → grips, Delete, Format tab, and geometry
  fields *if it is a path*

---

## 6. Modes

Three modes, declared at `shell/manifest/mod.rs:149-161`. Capability is derived
from the mode's **tab list**, not its id (`app/modes/capability.rs:47-63`):
`edit_content` is granted iff the mode contains the `edit` tab.

| mode | tabs | `edit_content` | `author_markup` | `author_measure` |
|---|---|:-:|:-:|:-:|
| `read` | file, view | ✗ | ✗ | ✗ |
| `review` | file, view, pages, markup, measure | ✗ | ✓ | ✓ |
| `edit` | file, view, pages, edit, markup, measure, tools | ✓ | ✓ | ✓ |

**`edit_content` gates the selection of page content itself**, not merely the
verbs on it (`app/modes/capability.rs:116-136`). In Read and Review the
identical press is routed to the text sweep by
`canvas::textsel::takes_the_press` (`canvas/textsel/gate.rs:272-274`).

**The application always starts in Read** (`app/modes/mod.rs:594-598`; test at
`:783`). Layout is persisted per mode; the active mode is not.

Panel arrangements (`app/modes/defaults.rs:299-470`):

- **Read** — left: Pages, Bookmarks; right: Forms. **No Objects, no
  Properties.**
- **Review** — left: Pages, Bookmarks; right: one stack — [Comments,
  Properties, Forms, Dimension groups]. **Properties is mounted but Objects is
  not.**
- **Edit** — left: **ONE stack of five tabs** — Pages, Bookmarks, Layers,
  Signatures, Fonts; right: **one column of two stacks** — [Objects] over
  [Properties, Comments, Forms, Redact, Dimension groups, Attachments], with
  the dock's own draggable splitter between them. Right-side default width
  **360 pt**, against 320 in the two reading stances.
  **Properties is the first tab of a six-tab stack**; only the active tab of a
  stack draws. If Forms or Comments was left active, selecting an object
  changes nothing visible in that slot at all.

★ Both arrangements changed on 2026-09-04 — `OPERATOR_REQUESTS.md` O123. The
Tool panel's stack is gone from both sides that had one, and its room went to
the Objects/Properties pair.

---

## 7. OCR

### Starting it

One command, `file.ocr`, labelled **"Recognise text…"**
(`text/commands/mod.rs:435-441`), registered at
`shell/commands/catalog.rs:396` with `.enabled_when("doc.pages")`. Drawn as a
Large button in its own one-item band, **File ▸ Recognise**
(`shell/manifest/file.rs:157-161`). It has no icon, deliberately
(`catalog.rs:382-395`), so it draws as a word-only large control. Available in
every mode. **No keyboard chord** — `file.ocr` appears in no keymap.

Second entry point: the Find bar offers it when a search returns zero hits and
the current page has no extractable text (`find/bar.rs:585`, `:605-648`).

It is **not** on the Tools tab, though RIBBON_IA.md §5.7 specifies it there. The
move to File is argued at `shell/manifest/tools.rs:48-91`: Read mode's tab list
is `["file","view"]`, so a Tools placement is unreachable in the mode the
operator asked for OCR in.

### Scope: exactly one page, with no control over which

The page is captured at dialog-open from the current view:

```rust
pub(super) fn open(doc: &OpenDoc) -> Self {
    Self { page_index: doc.view.page_index, source: doc.path.clone(), … }
}
```
(`dialogs/ocr.rs:194-202`)

The request carries a single `usize` (`ocr/mod.rs:463-472`). The worker
rasterizes that one page and calls the writer with that one index
(`ocr/mod.rs:558-575`, `:631-636`).

**There is no page-range control, no "all pages" checkbox, no multi-select, and
no loop anywhere in the codebase.** The button says so literally: **"Recognise
this page"** (`text/ocr.rs:91-93`), tooltip *"Runs the recogniser over the page
you are looking at."* (`text/ocr.rs:102-104`).

In continuous scroll, which page that is depends on scroll position at the
moment the dialog opened, **and the dialog never displays a page number
anywhere** (`dialogs/ocr.rs:293-340`).

The shell already owns a working page-range parser — `parse_page_range("5,1-2",
10) -> Some(vec![4,0,1])` at `dialogs/print/tabs.rs:161`, with tests at
`:537-589`. OCR does not use it.

### It produces a file, not an edit

`add_ocr_layer` reads the **base revision** of the session and returns a
complete PDF as `Vec<u8>` — the original plus one appended incremental revision
(`ocr/mod.rs:631-647`;
`D:/Dev/pdfcer/crates/pdfcer-core/src/ocr/layer.rs:411-418`).

**The open document is not touched.** `edit_epoch` does not move, no `Action` is
pushed, nothing enters the undo stack (`dialogs/ocr.rs:79-87`).

Saving is a raw filesystem write to a path the operator picks
(`dialogs/ocr.rs:433-443`), with a suggested name that is **never** the source:
`survey.pdf` → `survey-recognised.pdf` (`dialogs/ocr.rs:543-552`).

**After saving, the new file is not opened and the open document is not
repointed at it.** `Phase::Saved(path)` renders one label, *"Saved to {path}"*
(`dialogs/ocr.rs:328-330`). The operator is left looking at the un-OCR'd
original with a dialog telling them a different file exists somewhere.

To see the result: close the dialog, File ▸ Open the new file. To have it at the
original path: do it themselves in Explorer.

**Why Save-as and not Save**, per the code's own comments:
- the operator's standing rule of 2026-08-14, quoted at `dialogs/ocr.rs:37-40`:
  *"if in read mode ocr should still be available, but it will prompt to save
  changes as save as instead of save"*;
- disclose-before-write ordering (`dialogs/ocr.rs:16-34`);
- a regression guard that hashes the source before and after
  (`tools/ui-verify/src/checks/ocr.rs:24-38`).

The shell **does** have in-place save (`app/save.rs:329-377`, temp-file plus
atomic rename). OCR cannot use it because the recognised bytes never become part
of the open session — there is no verb that would put them there.

### ★ The preflight trap

```rust
if doc.edit_epoch != 0 { return Some(Refusal::UnsavedEdits); }
```
(`dialogs/ocr.rs:374-376`)

`edit_epoch` is monotonic and **is never reset — not even by a successful
save** (`app/save.rs:82-98`, pinned by the test at `:828-856`). It is `0` only
at open.

**So: make any edit, save it, and OCR refuses for the rest of that document's
life with "This document has unsaved changes"** (`text/ocr.rs:246-248`) — a
sentence that is false at that moment. The only recovery is to close and reopen
the file.

### Options: there are none

The dialog has three controls in total, across all states: "Recognise this
page", "Save recognised copy as…", "Close" (`dialogs/ocr.rs:293-340`).

| Option | Present? | Decided instead by |
|---|---|---|
| page / range | no | `doc.view.page_index` at dialog-open |
| DPI | no | `fitted_dpi`, targeting 8.4 Mpx, clamped 50–300 (`ocr/mod.rs:218-235, 335-351`) |
| language | no | `ocrs` takes none — a genuine engine gap |
| skip pages that already have text | no | nothing checks |
| output font | no | shell always passes `OcrLayerOptions::new()` (`ocr/mod.rs:635`); the engine exposes `.with_font(Std14)` |
| model directory | no | two hard-coded locations (`dialogs/ocr.rs:564-566`) |
| deskew / preprocessing | no | greyscale only (`ocr/mod.rs:377-394`) |

There are **no OCR keys in settings at all**. The effective DPI *is* computed
and *is* reported — to the trace line only (`dialogs/ocr.rs:268-278`), never to
the dialog.

The **CLI**, against the same engine, exposes `--dpi`, `--model-dir` and
`--words` (`D:/Dev/pdfcer/crates/pdfcer/src/main.rs:2374-2450`). The GUI
offers none of the three.

### Progress and cancellation

**Progress: an indeterminate spinner.** The job's only channel carries the
finished result (`ocr/mod.rs:479-482`); `poll()` returns a value or nothing. The
UI is `ui.spinner()` beside the word **"Recognising…"**
(`dialogs/ocr.rs:302-307`, `text/ocr.rs:108-110`). No page number, no elapsed
time, no percentage.

**Cancellation: none, deliberately.** `ocr/mod.rs:50-54`: *"No cancellation
token. … Nothing makes a recognition unwanted halfway through."* Closing the
dialog does not stop the work: `Job::spawn` detaches the thread
(`ocr/mod.rs:493-509`), the send fails harmlessly, and the thread runs to
completion burning CPU on a result nobody will see. There is no "Cancel" or
"Stop" control anywhere in `dialogs/ocr.rs`.

**A stale sentence the operator will read.** The ribbon tooltip still says *"It
takes a few seconds and the window will not respond while it does"*
(`text/ocr.rs:102-104`). That is false in two directions: OCR moved to a worker
thread, and since 2026-08-21 the dialog is its own OS-level viewport
(`dialogs/ocr.rs:212-226`).

### Where the ceiling actually is

The engine's verb takes one page index
(`D:/Dev/pdfcer/crates/pdfcer-core/src/ocr/layer.rs:620-625`) and there is **no
OCR verb on `EditSession` at all** — `grep -c "ocr"` over
`pdfcer-core/src/edit.rs` returns **0** against a 26,000-line file holding every
other editing verb.

But single-page is a shape, not a wall: `Document::from_bytes` is public
(`pdfcer-core/src/document.rs:392`), `OcrsEngine::recognize` takes `&self` and is
reusable, and the engine's own docs assume batch callers — *"a bad install fails
here rather than on page 340 of a batch"*
(`D:/Dev/pdfcer/docs/core-api/03-capabilities.md:1531-1533`). The shell currently
violates that guidance anyway: `OcrsEngine::from_model_dir` is called inside
`recognise_image` on every run (`ocr/mod.rs:717-718`), so a naive N-page loop
would reload ~12 MB of weights N times.

**[UNVERIFIED]** Whether `add_ocr_layer` behaves correctly when handed a
`Document` that is *already* an incrementally-saved OCR output. Nothing does
this today; the chained-revision approach is architecturally available and
untested.

**So, of the four OCR complaints:** multi-page, page ranges, and every missing
option are shell limitations in `ocr/mod.rs` and `dialogs/ocr.rs`, needing no
engine change. Only *"save from there"* in its strong reading — OCR as an edit
to the open document, undoable and savable in place — requires an engine verb
that does not exist.

---

## 8. Cursors and status

### Hover feedback for page content: none

`canvas/interact.rs:1367-1376` gathers four facts and calls
`tool::cursor_for(tool, gesture, hovered_grip, pointer_down, over_canvas)`.
Precedence (`canvas/tool/arm.rs:44-53`): armed tool's cursor → in-flight
gesture's cursor → hovered resize grip → nothing.

The pointer changes for:
- an armed tool — crosshair, I-beam, hand (`canvas/tool/arm.rs:70-75`);
- a resize/rotate grip of an **already-selected** object
  (`canvas/handles.rs:203-210`);
- a form-field widget (`canvas/forms.rs:1030-1032, 1118`);
- a ruler guide (`canvas/guides.rs:856-857, 924-925`);
- the I-beam's tilt over rotated text, but only when the icon is already Text
  (`canvas/interact.rs:1428-1454`).

**There is no hover highlight, no outline, no tint, no cursor change, and no
status readout for the object under the pointer.** The refusal is deliberate
and documented as a cost decision: `canvas/tool/arm.rs:139-143` — *"a hit test
against the page's extraction, paid on canvases nobody is selecting on, which is
most of them."* The same argument gates building the object model at all
(`canvas/interact.rs:465-501, 545-594`): the decomposition is not built on a
hover frame, only when a click, drag, right-click or armed measure tool needs it.

**Consequence: the first feedback about what a click will select is the outline
that appears after the commit.** On the the conformance suite’s composite page file that outline is the whole
sheet.

### How a refusal reaches the operator

Three different answers, two of which are "it doesn't".

**Move refusals — trace only.** All ten variants, including `NoPartEntered` and
`NoVerbForPart(Run)` (the two a double-click produces), go through:

```rust
fn decline(selection: &SelectionState, reason: Refusal) {
    crate::diag::trace(|| { /* ui-text-exempt: … never displayed in the UI */ });
}
```
(`canvas/moving.rs:733-742`)

The operator sees a drag that tracked, an outline that never ghosted, and
silence.

**Rotate refusals — nothing at all.** No enum, no trace, no note.

**Resize refusals — a real sentence, on a channel that is almost always
closed.** `canvas/resizing.rs:530-546` records the note at **epoch zero**,
deliberately and uniquely. Both surfaces that render the slot filter on the
document's **live** epoch (`app/status/disclosure.rs:155-164`,
`panels/tool/mod.rs:235`, filter at `app/actions/disclosure.rs:110-116`), and
`edit_epoch` starts at 0 and rises with every edit (`app/state.rs:874`).

**So a resize refusal is visible only on a document that has not yet been edited
this session. After the operator's first successful edit, the sentence is
written to a slot nothing will ever read.** The comment at
`canvas/resizing.rs:535-541` intends the opposite — *"a refusal must retire on
the operator's NEXT act"* — and the mechanism does not deliver it.

Engine-side refusals (from `transform_objects`) do reach the status bar,
through `vector_edit`'s disclosure list keyed on the new epoch
(`app/actions/vector.rs:537-549`).

**Nothing is greyed out and nothing changes shape** to signal "this cannot be
resized". The grips are painted on the same predicate for every kind.

### Active constraints

Announced as a sentence on the status row via a one-frame `egui::Memory` slot
(`canvas/constrain.rs:334-370, 373-381`).

### The Select filter, and what it does not tell you

A permanently visible button labelled **Select** at the left of the status bar's
fixed cluster (`app/status.rs:861-863`), with a standing note when everything is
off (`app/status.rs:790`). Default is everything on except `Link`
(`canvas/pick.rs:345-357`). The setting is persisted to
`target/release/userdata/select-filter.txt` (`app/pickstore.rs:116-160`) —
including the state where everything is off, which yields a canvas where no
click selects anything.

★ **Only 3 of the 11 rows are read by anything.** `.allows(` across the crate
resolves to `canvas/input.rs:102` (Part), `:108` (Node), `:155` (object class),
plus the popup's own render. **`Markup`, `CeDimension`, `FormField`, `Link` and
`Characters` are consulted by nothing** — `clicking.rs:191` calls
`annot::under_pointer` with no filter argument, and
`textsel::takes_the_press` takes only `(tool, caps)`. `canvas/pick.rs:105-109`
warns against exactly this: *"a row the operator can switch off but which
nothing consults is a lie told once per session."*

---

## 9. Known dead ends

Complete, working, wired features that the operator cannot reach; and gestures
that look like they should work and do not.

1. **Everything inside a form XObject.** The decomposer does not recurse
   (`decompose.rs:400-404`). Not in the object model, not in the Objects panel
   (the two Form rows carry no `>` expander while Text and Path rows do
   **[driven]**), not hit-testable, not selectable, not editable, at any zoom,
   with any modifier, at any double-click depth. On the the conformance suite’s composite page file this is the
   entire visible body of every page.

2. **Select-the-object-behind.** The engine computes the full front-to-back
   candidate list and the GUI throws away everything but the head
   (`canvas/input.rs:151-158`). No Alt+click, no repeat-click cycling, no list,
   no menu item. The only route is the global "Blocks" toggle, which is
   all-or-nothing and requires guessing the diagnosis first.

3. **`hit_test_all`.** Present, correct, on the trait, and consumed by exactly
   one caller that discards its tail.

4. **Alt+click cycling as documented.** The CLI help, `hit.rs:41-48` and
   `target.rs:85-88` all describe a GUI gesture that does not exist.

5. **The Objects panel as a way to select.** It sets `focus`, not
   `doc.selection` (`panels/objects/mod.rs:129-141`). Part and point rows are
   not clickable. Nothing highlights when the canvas selection changes.

6. **The Properties panel's object description as a way to edit.** Headed
   *"Nothing here can be changed in this build."* (`text/panels/properties.rs:334-335`).
   Twelve read-only fields.

7. **Typed X/Y/W/H for text and images.** Gated out by a path-only helper
   (`panels/objects/provider.rs:677-680` → `geometry.rs:350`). The section is
   not drawn — no heading, no greyed field, no explanation.

8. **The Format tab's promise.** Twenty-four planned per-selection commands,
   all in `PLANNED` (`shell/manifest/mod.rs:1043-1137`), including every font
   and paragraph control. What ships is Properties (which re-opens a read-only
   panel) and Delete.

9. **Delete after a double-click.** Silently a no-op at Part and Node rungs
   (`canvas/selection/mod.rs:370-375`).

10. **Handles after a double-click on an image or form.** Painted away, hit-
    tested away, and the body drag refuses to a trace line
    (`canvas/overlay.rs:221`, `canvas/pressing.rs:152`, `canvas/moving.rs:429`).

11. **Press-and-drag on an unselected object.** Always a marquee
    (`canvas/gesture/meaning.rs:736`). There is no select-and-move in one
    gesture anywhere in this build.

12. **Handles drawn over an object with a singular CTM.** Explicitly
    acknowledged at `canvas/resizing.rs:174-205`: `transform_preview` exists in
    the engine and is not called.

13. **Arrow-key nudge.** Does not exist.

14. **Rotate refusals.** No enum, no trace, no note — six distinct silent
    return-`None` paths (`canvas/rotating.rs:220-256`).

15. **Resize refusal messages after the first edit.** Written to epoch 0, read
    at the live epoch (`canvas/resizing.rs:530-546` vs
    `app/actions/disclosure.rs:110-116`).

16. **Eight of eleven Select-filter rows.** Switchable and consulted by nothing.

17. **OCR after any edit.** Permanently refused with a false message until the
    file is closed and reopened (`dialogs/ocr.rs:374-376`, `app/save.rs:82-98`).

18. **The saved OCR output.** Written, then neither opened nor pointed at
    (`dialogs/ocr.rs:433-443`).

19. **`parse_page_range`.** A tested page-range parser one directory away from
    the OCR dialog (`dialogs/print/tabs.rs:161`), unused by it.

20. **The Properties panel in Review mode.** Mounted without the Objects panel,
    so its empty state instructs the operator to click a row in a panel that
    mode does not have (`app/modes/defaults.rs:361-406`).

21. **The Format tab's discoverability.** Appended last in the tab strip and
    never auto-activated (`egui-shell/src/ribbon/tabs.rs:247-253, 283-291`).

22. **The Properties panel in Edit mode.** First tab of a five-tab stack; if
    the operator left Forms or Comments active, selecting an object changes
    nothing visible in that slot.

23. **A grip in the scrollbar strip.** **[driven]** Dragging the whole-page
    selection's bottom-right grip at window (1150, 901) — 8 points above the
    canvas viewport's bottom edge — produced no `resize-*` line and scrolled the
    view instead. **[UNVERIFIED]** that the scrollbar intercepted the press; only
    the outcome was established.

24. **"Dragging an inserted image doesn't resize."** **[UNVERIFIED — not
    reproduced, not disproved.]** One PNG, page 1, default placement, fresh
    session: the image was clickable (it is painted last, so it outranks the
    forms), a corner-grip drag resized it (`resize-commit grip=SouthEast
    sx=0.6810 sy=0.5899`), and a body drag moved it. The likeliest explanation
    of the operator's experience is dead end 11 — a press-drag on the freshly
    inserted, *unselected* image draws a marquee, which on release selects
    nothing, so the image appears to reject the drag and to deselect.
    `Action::InsertImage` never sets `doc.selection`
    (`app/actions/apply.rs:409-444`), so a newly placed image is never
    pre-selected. Placement itself is a separate OS window asking for millimetre
    coordinates; there is no draw-a-box placement. **[driven]**
