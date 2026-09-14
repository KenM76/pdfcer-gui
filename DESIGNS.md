# DESIGNS.md — drafted, argued, and not yet built

**What this file is.** One section per `OPERATOR_REQUESTS.md` row that has a
complete design and no code. Each section states what was measured, what the
design is, which alternatives were rejected and why, and what the work owes in
the way of a driven check.

★★★ **Why it exists, and it is not tidiness.** Every design in here was
written into a session scratchpad under `C:/Users/Ken/AppData/Local/Temp/`,
which is deleted when the session ends. A design that lives only in a
conversation is a design that will be **done twice** — and the second time it
will very likely come out differently, because the expensive half of each of
these is not the code. It is the measurement, and the list of approaches that
were tried on paper and rejected. Re-deriving the code is a day; re-deriving
the reason a plausible approach is wrong is how a defect ships.

**The deletion rule, and it is the same rule `DEFECTS.md` and
`SWEEP_REPAIRS.md` use.** A section is deleted when its design has been
**built and driven**, in the same commit — not when the code lands. Until a
driven check has agreed with it, the section is the only record of what the
correct behaviour was supposed to look like, and an implementation that
reintroduces the symptom is otherwise indistinguishable from one that fixed
it.

⚠ **These are designs, not decisions already taken by the operator.** Where a
section says a question needs his ruling, that is load-bearing: do not build
past it. `OPERATOR_REQUESTS.md` is where the rulings are recorded, and **only
Ken closes a row.**

⚠ **Re-measure before building.** Every `file:line`, every constant and every
trace slot quoted below was true when the section was written. This repository
has spent eight corrections on prose drifting from a count, and the engine pin
is a branch pin that moves without a `cargo update`. Treat each citation as a
claim with a shelf life of hours — see `DOC_DRIFT.md` S11 for what happens to
a hard-coded line number in a file that grows at the head.

---

## O181 — installed fonts in Add text, and the Format ribbon that does nothing

Design, written 2026-09-12 from a read-only survey of the whole font pipeline.
Every `file:line` below was measured, not remembered.

The report is two complaints in one sentence, and they have **nothing in
common mechanically**. They are written up separately so neither can hide
behind the other.

---

### Half A — the Format ribbon is not dead. It is asking for a gesture he has no reason to know about.

#### What is actually true

The five font controls (`format.font`, `format.font_size`, `format.bold`,
`format.italic`, `format.font_colour`) are registered, drawn, predicated and
fully wired through to `Action::TextStyle`. They are **not** stubs and their
commands are **not** no-ops — `app/dispatch/format.rs:521-583` builds a real
`StyleChange` and pushes a real action. The driven check
`the_format_tab_offers_font_controls_for_swept_text` exercises the whole
route and passes.

Two predicates gate them:

| predicate | where | effect |
|---|---|---|
| `shown_when("mode.edit_content")` | `shell/manifest/format.rs:221`, set at `app/conditions/mod.rs:760` | the group does not exist at all in Read or Review |
| `enabled_when("selection.text")` | `shell/commands/catalog/format.rs:255-275` | greyed unless a **text** selection is live |

and `selection.text` is set only by `app/conditions/mod.rs:480-486`, from
`doc.text_selection`.

#### The trap, in one line

`canvas/textsel/gate.rs:272-274`:

```rust
pub fn takes_the_press(tool: CanvasTool, caps: Capabilities) -> bool {
    tool.is_text() || (matches!(tool, CanvasTool::Select) && !caps.edit_content)
}
```

★ **In Edit mode `caps.edit_content` is true, so the Select tool does *not*
sweep text.** Edit mode is the only mode where the Font group is visible.
So the operator, in the one mode that shows font controls, clicks on a piece
of text with the tool that is armed by default, gets an *object* selection —
and every font control greys. The disclosure exists (the disabled-hover
sentence names the route), but a greyed control is the last thing anyone
hovers; they read "greyed" as "not implemented" and stop. That is exactly the
words he used.

This is a **discoverability report against shipped work**, which by the
standing rule means: fix the route that failed him, do not merely point at the
route that works.

#### The fix — the conventional gesture, not a new one

Every application in this product class — PowerPoint, Illustrator, InDesign,
Acrobat, Word's text boxes — resolves this the same way:

> **Double-click a text object with the arrow/select tool ⇒ you are now
> editing that text, with the whole block swept.**

So: **double-clicking a text object while `CanvasTool::Select` is armed in
Edit mode arms `CanvasTool::Text` and produces a text selection spanning that
object's runs.** Nothing new is invented, no new widget appears, the ribbon
group lights up the instant the operator does the thing he already does in
every other program.

Second, cheaper half of the same fix, to be built **only if the runs for an
object selection are cheaply derivable**: a *single* click selecting exactly
one text object also satisfies `selection.text` over that object's whole span,
so Bold on a clicked text object works without the double-click. Establish
first whether an object selection carries enough to name the runs; if it does
not, the double-click alone is the deliverable and that is enough.

⚠ What must **not** change: `takes_the_press` itself. Making Select sweep text
in Edit mode would destroy object selection, move, resize and the `/Rect`
handles — the entire reason Edit mode's Select exists.

#### The disclosure that must change with it

`fontband.rs:170-176`'s disabled-hover sentence currently names "arm the Text
tool" as *the* route. Once double-click works, that sentence is incomplete and
must name the double-click first, because it is the one the reader already
knows.

#### Driven check owed

`font_group.rs` has a phase 1 that asserts the route sentence appears without
`T`. Add a phase 0 **before** it: with `CanvasTool::Select` armed, double-click
the text object and require the five `ribbon.item.format.*` controls to be
**live**, and a `text-style-applied` after pressing Bold. Falsify it against a
build with the double-click removed.

---

### Half B — installed fonts in Add Text

#### What is actually true

`canvas/textedit/pen.rs:182` — `pub const FACES: &[Std14]`, fourteen entries
written out by hand, and `TextPen::face` is **typed** `Std14` (`pen.rs:95`), so
an installed font is *not expressible*. The module header says so outright
(`pen.rs:58-65`): *"A donor font … is deliberately NOT here."*

The rest of the chain is already built, on both sides of the crate boundary:

| piece | where |
|---|---|
| walks `%WINDIR%\Fonts` and the per-user font dir | `app/prefs/fonts.rs:109` `os_font_dirs()` |
| parses every file, takes every advertised face name | `app/fonts.rs:340` via `FontProgram::parse` + `.face_names()` |
| holds name → path | `app/fonts.rs:223` `paths: BTreeMap<String, PathBuf>` |
| engine: subset a donor for the characters typed | `pdfcer-render` `font::subset::plan_subset(donor, face_index, chars, base_name, subset_tag)` |
| engine: the Add-Text seam that takes the result | `pdfcer-core` `NewTextFace::Embedded(Box<FontEmbedPlan>)`, `AddTextRequest::with_embedded_face(plan)` |

★ **Nothing is missing on the engine side.** No request needs to be filed. The
two hard stops are both in this repository: the `Std14` type on the pen, and
the fact that `Library` exposes no way to *list* what it scanned (its public
surface is `scan`, `scan_with`, `donor_for`, `len`, `is_empty` — a resolver
keyed by a `/BaseFont` the document already names, never an offer list).

#### The build

1. **`Library::names() -> Vec<&str>`** — sorted, from the private `paths` map.
   The engine's own `FontEnvironment::named_faces()` exists for this and is
   called nowhere in this workspace; use whichever reads cleaner, but expose
   exactly one.
2. **Widen the pen's face**: `enum PenFace { Standard(Std14), Installed(String) }`.
   `pen::FACES` stays as the first group.
3. **The combo grows a second group**, and it must be **the same shape as the
   restyle chooser already ships** — `panels/properties/face.rs:249`, which
   offers page faces first and "faces pdfcer would add to the document"
   second, with the disclosure drawn in the popup *before* any addable row.
   The driven check `the_face_chooser_offers_a_face_the_document_does_not_contain`
   asserts that disclosure is on screen before the click, and it passes today.
   ★ Copy that shape exactly rather than inventing a second vocabulary for the
   same idea.
4. **Commit**: on an `Installed` face, read the bytes, `plan_subset` over the
   characters actually typed, `with_embedded_face(plan)`.

#### Three things that will bite, all knowable now

- **CFF donors are refused by name** (`pdfcer-core` `font_embed.rs:45-53` —
  TrueType `glyf` only). A great many installed `.otf` files are CFF. The scan
  already parses each file, so the kind is known *at scan time*: decide there,
  and if a face is filtered out, the count of what was filtered belongs in the
  popup's disclosure. A silent short list is the defect wearing the other coat.
- **`use_os_fonts` defaults to `false`** (`app/prefs/mod.rs:867`), so the second
  group would be **empty** for him until he finds a Settings checkbox.
  ⇒ **Default it to `true`.** This is the identical ruling to O180's trimming
  tick: defaulting it off fixes the report only for people who go looking in
  Settings, which is nobody.
- **No driven check enumerates what the Add Text font list contains.**
  `properties_tool.rs:115` asserts the region is drawn and never opens the
  combo. That is the coverage hole behind this whole half, and the new check
  must open the popup and read the rows.

#### Driven check owed

`add_text_offers_a_font_the_machine_has` — arm Add Text, open the pen's font
combo, require a second group with at least one row that is not one of the
fourteen, pick it, type, and require the saved document to carry an embedded
program for it. SKIP (not fail) with a named reason when the machine's font
folders yield no embeddable TrueType face at all.

---

## O188 — moving and deleting one text block inside a group

**Design survey, 2026-09-12. Read-only.** Every `file:line` below was grepped or
read during this survey, against the working tree as it stands. Where a
published document's line number disagrees with what I read, I say so and the
measurement wins.

> *"In text that is grouped together or whatever it is called, such as in my
> title blocks, I would like a way to move the individual text blocks within it
> around, and have the ability to delete them like I can when I add text using
> our Add text tool."*
> — filed at `D:\Dev\pdfcer-gui\OPERATOR_REQUESTS.md:487`

---

### 0. The verdict first, because it is not the expected one

**O188 is two requests wearing one sentence, and they are in completely
different states.**

| half | state | what it needs |
|---|---|---|
| **delete one piece** | ✅ **ALREADY SHIPS, wired and driven since 2026-09-05** | nothing built. It is a **discoverability** problem — the gesture is the Points tool, not the double-click he will have tried. |
| **move one piece** | ❌ **blocked on a missing engine verb** | `pdfcer-core` has no `move_text_run` and no mechanism that can be retargeted to one. The shell's routing, hit test, rung, ghost and refusal are all already built and already waiting. |

So the classification asked for — (a) shell-only, (b) blocked on the engine, (c)
partly available — is **(c) and (b) together**, and the (c) half is the one worth
acting on first because it costs nothing and answers half his sentence today.

★ **The most valuable single fact in this document:** he can delete a label out
of his title block right now. He has almost certainly tried it by
double-clicking the text, which since O70 opens the **caret** and never descends
the selection ladder at all (`canvas/clicking.rs:871-910`, the `double` arm that
calls `textedit::click` with `TextEditKind::Edit`). The route that works is
pressing **`A`** — the *Points* tool — and then clicking the label.
`A` is bound to `view.tool_node` at
`crates/pdfcer-gui/src/shell/manifest/mod.rs:483`.

---

### 1. What the engine's unit of selection actually is — MEASURED

#### 1.1 The model: one text object, N runs, one run ≈ one show operator

`pdfcer_core::vector::decompose` produces `VectorObject::Text(TextObject)`. The
addressable sub-unit is **`TextRun`**, defined at
`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\decompose.rs:866-882`:

```rust
pub struct TextRun {
    pub bounds: Bounds,            // page-space, laid-out extent of ONE show operator
    pub tokens: TokenRange,        // its operands through its operator
    pub bytes: ByteSpan,           // "This is what a per-run edit rewrites, and the
                                   //  reason this struct exists."
    pub positioned_by: RunPositioning,
    // …plus a byte range into TextObject::preview for the decoded text
}
```

Its own doc header, `decompose.rs:857-865`, answers the question O188 asks
directly:

> *"One show operator inside a `BT`…`ET`: where it drew, which bytes drew it,
> and whether it owns its own position."*
> *"**A `TJ` array is ONE run, deliberately.** Its numeric elements are kerning
> *within* a single positioned string, not separate placements, so splitting on
> them would fragment a word into per-glyph boxes for no gain — and would make
> 'delete this run' mean something no operator asked for."*

**So the answer to "is the addressable unit a `BT..ET` block, a `Tj`/`TJ`, a run,
or a whole content-stream object" is: the `BT..ET` is the OBJECT, and one
`Tj`/`TJ` show operator is the PART.** `TextObject::runs: Vec<TextRun>` is the
part list (`decompose.rs:690`, per `docs/core-api/01-reading-and-model.md:1695`).

#### 1.2 ⚠ The trap: a `TextRun` here is NOT `text_extract`'s run

There are two things called a run in this engine and they are different objects.
`docs/core-api/01-reading-and-model.md:1107-1160` (§8.4.0, `Pass 145.0`) is a
whole section about it, and it measures the divergence:

| measured on a SOLIDWORKS export | |
|---|---|
| distinct `operator_span` groups | 29,246 |
| **`text_extract` runs carrying glyphs from MORE THAN ONE show operator** | **2,420 (13 %)** |

That divergence belongs to `text_extract::TextRun` (the reading/extraction
model). **`vector::decompose::TextRun` — the one O188 is about — is one show
operator by construction**, because `close_text_run`
(`vector/decompose.rs:3361`) closes on the show operator. The two must not be
conflated when writing the engine request; naming the wrong one would get the
wrong verb built.

#### 1.3 `RunPositioning` — the field that governs everything downstream

`vector/decompose.rs:842-855`:

```rust
pub enum RunPositioning {
    /// A positioning operator set this run's origin after the previous run
    /// ended, so the run stands on its own coordinates. Deleting anything
    /// before it cannot move it.
    /// The FIRST run of a `BT`…`ET` is always this: `BT` resets both the text
    /// and line matrices to the identity (§9.4.1).
    Explicit,
    /// The run inherits its origin from the previous run's advance. It has no
    /// coordinates anywhere in the file, and deleting its predecessor moves it.
    Inherited,
}
```

**This one enum is the whole difficulty of O188's move half.** An `Inherited`
run has *no coordinates anywhere in the file*. There is no operand to rewrite.

#### 1.4 The verb that exists — `delete_text_run`

`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:13951`:

```rust
pub fn delete_text_run(
    &mut self,
    page_index: usize,
    object_index: usize,
    run_index: usize,
) -> Result<Vec<String>, EditError>
```

⚠ `docs/core-api/02-editing-and-saving.md:1134` publishes this at `edit.rs:4825`.
**That line number is stale** — the doc's own front matter says its figures were
derived at `e194b46`, 2026-08-18. Measured today it is `13951`. Same for
`move_subpath`: published `4875`, measured `edit.rs:14001`. Use the measured
ones.

Its doc (`edit.rs:13933-13935`) states the index contract:

> *"`run_index` is into `TextObject::runs` in content order — the same numbering
> the hit test returns."*

The planner is `vector::plan_delete_text_run`, `vector/edit.rs:1412-1457`. Two
behaviours worth knowing, both read from the source:

1. **`count == 1` deletes the whole object** (`vector/edit.rs:1432-1441`) —
   *"a text object that shows nothing is not an object"*. So on a single-label
   text object the Part rung and the Object rung do the same thing.
2. **The §9.4.2 guard** (`vector/edit.rs:1445-1453`): if `runs[run_index + 1]`
   has `positioned_by == RunPositioning::Inherited`, the call refuses with
   `VectorEditError::DeleteWouldMoveNextRun`. The refusal names a remedy that
   always works: **delete the later run first.**
3. **Disclosures are always empty** — `Vec::new()` on both arms
   (`vector/edit.rs:1437`, `:1456`). Measured, not assumed.

#### 1.5 The verb that does NOT exist — and the engine already knows it

Grepped across the whole of `pdfcer-core/src`:

```
edit.rs:12257  pub fn move_object(
edit.rs:12307  pub fn delete_object(
edit.rs:13951  pub fn delete_text_run(
edit.rs:14001  pub fn move_subpath(
edit.rs:14776  pub fn move_subpath_in_form(
```

**There is no `move_text_run` and no `plan_move_text_run`.** `vector/mod.rs:87`
re-exports `plan_delete_text_run` beside `plan_move`, `plan_move_handle`,
`plan_move_many`, `plan_move_node` — and no text-run mover.

The full sub-object verb table, from
`docs/core-api/02-editing-and-saving.md:1128-1142`, shows the asymmetry as a
hole you can point at:

| part kind | move | delete |
|---|---|---|
| whole object | `move_object` / `move_objects` | `delete_object` / `delete_objects` |
| subpath | `move_subpath` ✅ | `delete_subpath` ✅ |
| anchor node | `move_node` / `move_nodes` ✅ | `delete_node` ✅ |
| Bézier handle | `move_handle` ✅ | — |
| **text run** | **✗ NOTHING** | **`delete_text_run` ✅** |

Every other part kind has both twins. Text has only the delete.

#### 1.6 ★★ Why `transform_objects` cannot be borrowed for this — the decisive measurement

The obvious escape is `transform_objects`
(`edit.rs:12434`, planner `vector/edit.rs:922`). Its doc argues it is
**kind-agnostic by construction** (`vector/edit.rs:872-878`):

> *"Wrapping in `q…cm…Q` has none of those problems because it never looks at an
> operand. It is therefore kind-agnostic by construction — a path, a text
> object, an image XObject, a form XObject and an inline image are all just a
> byte span with a CTM."*

**That argument does not survive one level down, and the engine has already
written the reason four separate times:**

- `text_edit/format.rs:102` — *"`q … Q` wrap is **illegal inside a text object**
  (§8.2 Table 51 / …)"*
- `text_edit/format.rs:4594` — *"`q`/`Q`, which is not admitted inside a text
  object (§8.2 Table 51)"*
- `text_edit/format.rs:5108-5110` — *"…are not admitted inside a text object
  (§8.2 Table 51 / Figure 9), and splitting the `BT … ET` to use them outside
  would discard `Tm` (§9.4.1) and force absolute re-positioning of…"*
- `text_edit/reflow_apply.rs:609-611` — same finding, independently stated.

And `plan_transform_many` addresses objects, not parts: it maps over
`objs: &[&VectorObject]` taking `o.bytes()` — the **whole** object span
(`vector/edit.rs:955-960`). Its own balance argument
(`vector/edit.rs:905-912`) is that *"a decomposed object is a complete graphics
object, so the inserted pair encloses a balanced region"* — which is exactly
what a run inside a `BT..ET` is **not**.

⇒ **A `move_text_run` must be operand rewriting, the `move_subpath` shape, not
the `transform_objects` shape.** That is the single most useful sentence to put
in the engine request, because it forecloses the reviewer's first idea.

---

### 2. What the shell currently selects — MEASURED, and it is further along than expected

#### 2.1 The identity already expresses "piece N of object M"

`canvas/selection/identity.rs:99-129`:

```rust
pub struct Selection {
    pub page: usize,
    pub object: TargetId,
    pub subpath: Option<usize>,   // "a path's subpath OR a text object's show-operator run"
    pub node: Option<usize>,
}
```

`SelectionLevel` (`canvas/selection/identity.rs:154-175`) is the three-rung
ladder: `Object` / `Part` / `Node`, with `Part` documented as *"Inside one
object, selecting its parts — a path's subpaths or a text object's runs."*

`TargetId` (`panels/objects/provider/mod.rs:174-199`) is `Object(u64)` (a
paint-order index into `PageObjects::objects`) or `Leaf(u64)` (an index into
`PageObjects::leaves`, inside a form XObject).

**So the answer to "can the selection express 'piece N of object M'" is: yes,
today, for text, with no change at all.** `Selection { object: Object(m),
subpath: Some(n), node: None }` at `SelectionLevel::Part` *is* "run n of text
object m". The `subpath` field name is the only thing that lies about it.

#### 2.2 The hit test is wired to the engine's own run picker

`panels/objects/provider/mod.rs:534-540` dispatches on kind:

```rust
match self.part_kind(object) {
    Some(PartKind::Subpath) => self.subpath_hits(object, point, tolerance),
    Some(PartKind::Run)     => self.text_run_hits(object, point, tolerance),
    None => Vec::new(),
}
```

`text_run_hits` (`panels/objects/provider/mod.rs:585-590`) calls
`pdfcer_core::vector::hit_test_text_runs` directly (line 589). That engine
function is at `D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:530`, and its
header was written for precisely his file
(`vector/hit.rs:501-505`):

> *"A producer may put every label on a sheet inside one `BT`…`ET`. Measured on
> a real SolidWorks export: **one text object holding all 237 dimension
> labels**. `hit_test_point` already tests per-run boxes so such an object is not
> a page-wide hit — but it answers *whether*, not *which*, and a shell that wants
> to delete 'this label' needs the index."*

`PartKind` is decided in one place — `panels/objects/provider/geometry.rs:100-105`:
`VectorObject::Path(_) => PartKind::Subpath`, `VectorObject::Text(_) =>
PartKind::Run`.

Supporting readers, all measured:
- `text_run_count` — `panels/objects/provider/mod.rs:596-601`, reads `t.runs.len()`
- `text_run_bounds_canvas` — `panels/objects/provider/mod.rs:612` (the outline
  drawn round ONE label; its doc at `:604-610` explains the object's own bbox
  *"would draw a rectangle around every label on the sheet"*)
- `text_run_delete_would_move_next` — the §9.4.2 pre-check

#### 2.3 The form-XObject hole, stated by the code itself

`canvas/target.rs:565-571`:

```rust
(Some(PartKind::Run), Some(object)) => self.text_run_hits(object, point, tolerance),
// ★ A text run INSIDE a form has no leaf-indexed hit test yet —
// `text_run_hits` indexes the page's own list, and answering from it
// would return another object's runs entirely. Empty is the honest
// answer, and it is the next thing to build rather than an oversight.
(Some(PartKind::Run), None) | (None, _) => Vec::new(),
```

⚠ **This matters for O188 specifically, and is the sharpest risk in the whole
request.** His title block is very often a **form XObject drawn on every sheet**
— `ENGINE_BACKLOG.md:270` records exactly that: *"on a SolidWorks set, where one
title block is a single form drawn on thirty-six sheets"*. If the text he is
pointing at is a form leaf, the Part rung answers **empty** and neither the
existing delete nor any future move will reach it. That must be established on
his actual file before anything is built.

---

### 3. The comparison he named — what Add Text actually gives him

Measured: **Add Text is not special in any way.** There is no session-added flag,
no tracked id, no separate list.

- `Action::CommitAddText` is defined at `app/actions/action.rs:979-1010`, whose
  doc says the result *"becomes the page's own content, exactly like the text
  already there."*
- The call site is `app/actions/addtext.rs:118` — `session.add_text(&req)` with
  `pdfcer_core::text_edit::AddTextRequest`. Routed from
  `app/actions/apply.rs:920-931`.
- `commit()` (`app/actions/addtext.rs:112-125`) reads only `report.disclosures`.
  It stores no index, no `ObjId`, nothing.
- Grepping the shell for `session-added` / `just_added` / `last_added` /
  `added_object` / `newly_added` / `recently_added` returns **zero hits**.

So what he can do to his own added text is what he can do to any whole object:

| gesture | verb | shell call site |
|---|---|---|
| drag it | `move_objects` | `app/actions/vector.rs:756` |
| Delete | `delete_objects` | `app/actions/vector.rs:744` |

⇒ **And that is the crux of his sentence.** `add_text` appends a *new content
stream* (`docs/core-api/02-editing-and-saving.md:394`: *"Appends a new content
stream; originals stay byte-verbatim"*), so **each line he types is its own page
object**. One line = one object = the Object rung = `move_objects` works. His
title block's twelve strings are twelve *runs of one object*, which is the Part
rung, where move has no verb.

**He is not comparing two features. He is comparing two granularities, and he
has correctly noticed that the shell is generous at one and silent at the
other.** That framing belongs in the reply to him.

⚠ **Would the same verbs reach a piece of an existing text object?** No, and the
engine says so by name: `move_objects` *"refuses them with `NotAPath`"*
(`vector/edit.rs:869-870`) for text, and it addresses whole objects anyway.
`delete_objects` on his title block would remove every string in it at once —
which is the exact defect `delete_text_run` was built to end
(`tools/ui-verify/src/checks/deeper_rung_delete.rs:19-35`).

---

### 4. The delete half already ships — the measured record

`EDITABLE_SURFACES.md:795` is the authoritative row, and it is emphatic:

> ✅ **WIRED 2026-09-05, and DRIVEN — one label off a sheet whose 237 share a
> text object.**

The chain, each link measured:

| link | file:line |
|---|---|
| `DeleteSubject::TextRun` variant | `canvas/deleting.rs:127-130` |
| routed by `canvas::deleting::subject` | `canvas/deleting.rs:302-310` |
| §9.4.2 pre-check ahead of the press | `canvas/deleting.rs:305` |
| → `VectorAction::DeleteTextRun` | `canvas/deleting.rs:394-396` |
| variant defined | `app/actions/vector.rs:232` |
| applied — the only call site in the shell | `app/actions/vector.rs:810`, `session.delete_text_run(page, object, run)` |
| census either side | `app/actions/vector.rs:808`, `:812` (`p.text_run_count(o)`) |
| driven check | `tools/ui-verify/src/checks/deeper_rung_delete.rs:392-394`, `deleting_a_label_leaves_the_other_labels_alone` |
| its fixture | `fixtures/paragraph.pdf` at `0,120,704` — one `BT..ET` with six `Tj` at 12 pt (`deeper_rung_delete.rs:357-362`, `:376-380`) |

The check's own header states the assertion discipline worth repeating
(`deeper_rung_delete.rs:21-35`): *"a check that asserted 'something was deleted'
would pass on the exact bug this feature was built to prevent"*, so the verdict
is a **pair** — `objects_after == objects_before` **and**
`parts_after == parts_before - 1`.

★ And the gesture caveat, from the same `EDITABLE_SURFACES.md:795` row:

> **The operator reaches that rung with the Points tool (`A`), not with a
> double-click** — O70 gave the double-click on text to the caret, so
> `canvas::clicking` opens a caret and returns before the ladder is touched; the
> node-tool branch is the one route that descends on text.

Measured in the source: `canvas/clicking.rs:389` is
`if active_tool.is_node() && caps.edit_content` — the branch that descends; and
`canvas/clicking.rs:871-910` is the `double` branch that arms the caret and
returns. `A` → `view.tool_node` at `shell/manifest/mod.rs:483`; the command is
labelled **Points** (`shell/commands/catalog/view.rs:255`).

---

### 5. The move half — where it dies, and how loudly

`canvas/moving/eligible` (`canvas/moving/mod.rs:476`) is the one pure function
both the drag and the keyboard ask. On a text run it returns:

```rust
Some(other) => Err(Refusal::NoVerbForPart(other)),   // canvas/moving/mod.rs:591
```

`Refusal::NoVerbForPart` is at `canvas/moving/mod.rs:401`, and its doc is a
complete statement of the gap:

> *"The entered part has no move verb: a text object's show operator is a
> 'part', but `move_subpath` translates path construction operands and there is
> nothing for it to translate."*

The shell has a **unit test asserting the decline** —
`canvas/moving/tests.rs:509-534`,
`a_text_run_at_the_part_rung_declines_rather_than_moving_the_object`, with the
message *"moving the enclosing object because a run was selected is the wrong
action, not a lenient one"*.

⚠ **And the decline is SILENT.** `canvas/moving/mod.rs:1074-1109` — `decline()`
raises an operator-facing action for exactly one refusal, `InsideForm`, and its
comment says why the others are left mute:

> *"The rest describe states the operator put themselves in and can see: nothing
> selected, a rung with no verb, a drag that travelled zero distance. A sentence
> in the status bar for any of those would be a bar that narrates the obvious."*

That argument is sound for `NothingSelected` and `NoTravel`. **It is wrong for
`NoVerbForPart(Run)`, and O188 is the evidence.** The operator has a box drawn
round one label, presses inside it, drags, and nothing happens with no sentence
anywhere — which is precisely the *"from where they sit, dragging is broken"*
state the same comment grants to `InsideForm` two paragraphs later. This is a
one-line shell fix that is worth doing whether or not the engine ever ships the
verb.

#### 5.1 The old GUI reached the same conclusion, twice

Not new knowledge — `D:\Dev\pdfce\crates\pdfce-gui\src` recorded it a Pass ago:

- `main.rs:21785-21790` — *"a text object's parts are its RUNS, and the path
  wording describes two things a run does not have — a drag-to-move verb (**no
  core `move_text_run` exists at all**) and a Point rung beneath it (a run has no
  anchors). Saying either would assert an affordance that is not there."*
- `ui_text.rs:4613-4621` — *"the path sentence offers 'drag it to move just this
  part' … and **a run has neither** — no `move_text_run` verb exists anywhere in
  core."*

⇒ Two independent code bases have written this sentence and **neither filed the
engine request.** That is the actionable finding: the gap has been *described*
three times and *asked for* zero times. `EDITABLE_SURFACES.md:755-760` names this
exact failure mode — *"considered verbs whose consideration was filed where the
register could not see it … writing them down felt like discharging them."*

---

### 6. The build

#### 6.1 Ship today, no engine dependency — two shell changes

**(A) Tell him the delete gesture.** The capability exists and is driven; he does
not know the route. This is a reply and possibly a one-line status sentence, not
a build.

**(B) Make the move decline audible.** `canvas/moving/mod.rs:1074` — add
`NoVerbForPart(PartKind::Run)` to the refusals that raise an operator-facing
action, with a sentence that says what IS available rather than what is not:
*"This is one line inside a block of text. It can be removed, but pdfcer cannot
yet move one line on its own — press Escape to select the whole block and drag
that."* R83: a refusal that names its remedy.

⚠ Both are shell-only, both are inside the frozen-source constraint of this
session, so **neither is done here.**

#### 6.2 The engine request — the move verb

Drafted as §7 below. **Not filed** — this document is a survey.

#### 6.3 When the verb lands — the shell work is small and already shaped

Because the rung, the identity, the hit test, the outline and the routing all
exist, wiring a `move_text_run` is the same three edits `move_subpath` needed:

1. `canvas/moving/mod.rs:591` — `Some(PartKind::Run) => Ok(MoveSubject::TextRun
   { page, object, run })` beside the existing `Subpath` arm at `:582`.
2. A `MoveSubject::TextRun` variant and its `VectorAction::MoveTextRun` twin
   (`app/actions/vector.rs`, beside `MoveSubpath`).
3. The apply arm calling `session.move_text_run(...)`, beside
   `app/actions/vector.rs:810`'s delete.

★ **The ghost, the preview outline and the drag-hold need nothing** —
`moving::drag` (`canvas/moving/mod.rs:942-990`) builds them from
`part_bounds_of`, which already answers for a run via `text_run_bounds_canvas`
(`panels/objects/provider/mod.rs:550`).

★★ **And index stability is already settled in our favour.**
`docs/core-api/02-editing-and-saving.md:1088-1096` measures it against
`crates/pdfcer-core/tests/object_identity_across_edits.rs`: the **move** family
*"rewrites operator operands in place"* and does **NOT** renumber; the **delete**
family excises byte spans and **DOES**. A `move_text_run` built as operand
rewriting inherits the safe half of that table — the selection survives the drag
unchanged, exactly as it does for `move_subpath`.

---

### 7. Draft engine feature request — DO NOT FILE FROM THIS DOCUMENT

> ### Request: `move_text_run` — the missing twin of `delete_text_run`
>
> **What we called.** `canvas::moving::eligible` resolves a drag at
> `SelectionLevel::Part` on a `VectorObject::Text`. For a path it reaches
> `EditSession::move_subpath(page, object, subpath, dx, dy)`
> (`pdfcer-core/src/edit.rs:14001`). For text there is no call to make.
>
> **What we expected.** The twin `delete_text_run` implies. Every other part kind
> in the crate has both: subpath has `move_subpath` + `delete_subpath`; anchor has
> `move_node`/`move_nodes` + `delete_node`. Text has `delete_text_run`
> (`edit.rs:13951`) and nothing that moves one.
>
> **What happened.** `canvas/moving/mod.rs:591` returns
> `Refusal::NoVerbForPart(PartKind::Run)` and the drag does nothing, silently.
> We have a unit test asserting that decline
> (`canvas/moving/tests.rs:509-534`) because declining is currently correct.
>
> **What the operator asked for**, verbatim: *"In text that is grouped together
> or whatever it is called, such as in my title blocks, I would like a way to
> move the individual text blocks within it around, and have the ability to
> delete them…"* One `BT..ET` on his SolidWorks title block holds every string in
> it; `hit_test_text_runs`' own header measures the sibling case at **237
> dimension labels in one text object**.
>
> **★ What we checked before asking, so this is not a request to re-derive
> something.**
>
> 1. **`transform_objects` cannot be retargeted at a run.** Its mechanism is a
>    `q … cm … Q` wrap (`vector/edit.rs:955-980`), and `q`/`Q` are **not
>    admitted inside a text object** — §8.2 Table 51 / Figure 9, as this crate
>    itself states at `text_edit/format.rs:102`, `:4594`, `:5108-5110` and
>    `text_edit/reflow_apply.rs:609-611`. Splitting the `BT..ET` to wrap outside
>    would discard `Tm` (§9.4.1), which `format.rs:5109-5110` has already
>    rejected. So the mechanism must be **operand rewriting**, the `move_subpath`
>    shape, not the `transform_objects` shape.
> 2. **The hard case is `RunPositioning::Inherited`** (`vector/decompose.rs:851-854`)
>    — *"It has no coordinates anywhere in the file."* There is no operand to
>    rewrite. We would rather meet a **named refusal with a remedy** than a
>    materialised `Td` we did not ask for — but if materialising one is the right
>    answer, `move_subpath`'s own precedent covers it: its doc
>    (`edit.rs:14012-14020`) already discloses *"materializing the `m` an
>    implicitly-started subpath never had"* as a form change the caller must
>    surface. We will surface it.
> 3. **The successor problem is the mirror of a guard you already ship.** Moving
>    run N via its own `Tm`/`Td` moves run N+1 too if N+1 is `Inherited` — the
>    same fact `plan_delete_text_run` refuses on with
>    `DeleteWouldMoveNextRun` (`vector/edit.rs:1445-1453`). ⇒ **We are not
>    asking you to guess.** Either policy is usable by us; we would rather have
>    the one you can promise:
>    - **refuse**, mirroring the delete guard, with the same remedy wording; or
>    - **compensate**, re-spacing the following `Td`/`Tm` by the net delta —
>      which `edit_text` already does for a different reason (`Pass 256.0`:
>      *"the following `Td`/`Tm` steps on the line are re-spaced by the net
>      advance"*), so the machinery may already exist.
>    Per `R206`, both as options with your best default is also a fine answer.
> 4. **Coordinates.** We will pass page-space `dx, dy`, exactly as we do to
>    `move_subpath` and `move_node`. The composition through the CTM *and* the
>    text matrix (and, for `Td`, the line matrix) is yours — `move_node`'s doc
>    already sets that precedent (*"mapped from page space to the object's user
>    space by its CTM's affine inverse"*). We will not pre-convert.
> 5. **Index stability.** We are relying on
>    `docs/core-api/02-editing-and-saving.md:1088-1096` — operand rewriting does
>    not renumber. If `move_text_run` renumbers for any reason, say so and extend
>    that table; a verb that renumbers without saying so re-opens the hazard that
>    section exists to close.
>
> **Signature we would consume unchanged:**
> ```rust
> pub fn move_text_run(
>     &mut self, page_index: usize, object_index: usize,
>     run_index: usize, dx: f64, dy: f64,
> ) -> Result<Vec<String>, EditError>
> ```
> `run_index` into `TextObject::runs` in content order — the same numbering
> `delete_text_run` and `hit_test_text_runs` already use, so the shell needs no
> new index space.
>
> **A second, smaller row while you are in here**, filed separately if you
> prefer: there is no `delete_text_run_in_form` (nor `delete_subpath_in_form` /
> `delete_node_in_form`). `EDITABLE_SURFACES.md:786-791` records the consequence
> — the Part and Node rungs inside a form XObject cannot delete at all — and on a
> SolidWorks set the title block **is** a form drawn on every sheet, which is
> exactly the container O188 is about. A `move_text_run_in_form` would be owed on
> the same argument.

---

### 8. Prior art to copy, not invent

The standing rule is **use the conventional interaction, never invent one**. The
good news is that nothing needs inventing: the gesture is already built, already
conventional, and already used elsewhere in this shell.

#### 8.1 What the industry does for "one piece inside a text object"

| application | gesture | what it addresses |
|---|---|---|
| **Adobe Illustrator** | **Direct Selection (`A`)** — click descends into the object and selects one component; **Group Selection** clicks descend one nesting level per click | anchor points and path segments; for *type*, `A` gives you the baseline/anchor, and the **Type tool double-click** gives you a caret |
| **Adobe Acrobat — Edit PDF** | single click selects the recognised **text block**; drag its bounding box to move it | Acrobat's unit is a *reconstructed* block, not the `Tj` — it merges adjacent show operators into an editable box, which is why Acrobat can move "a line" that has no coordinates of its own |
| **Inkscape** | double-click to enter a group, then the object inside is selected directly | group descent |
| **Figma** | ⌘/Ctrl-click selects the deepest child under the pointer | one-gesture deep select |

★ **The convergent convention is "`A` / Direct Selection descends one level and
then the ordinary drag applies to what you descended to."** That is Illustrator's
`A` — and pdfcer's Points tool is already bound to **`A`**
(`shell/manifest/mod.rs:483`). This is not a coincidence to preserve by accident;
it is the thing to point at when explaining the gesture to him.

⚠ **The honest caveat about Acrobat**, and it should go in the reply: Acrobat can
move "a line" of a title block because it does not move the `Tj` — it
re-recognises a block and re-lays it out. pdfcer's contract is byte-verbatim
surgery on page content he must not have silently altered (R8b Rule 15 — these
are **pdf dimensions**). So pdfcer will do less than Acrobat here, on purpose,
and the refusals he meets (a run with no coordinates of its own) are the price
of not re-flowing his drawing behind his back.

#### 8.2 The nearest in-house neighbour — and it is exact, not analogous

**`move_subpath` at the Part rung is the same gesture on the same rung reached by
the same tool.** The prior art the request asks for is not something to find in
another application; it is sitting one match arm away in
`canvas/moving/mod.rs:582`.

The sub-path/anchor work it refers to is real and measured. `hit_test_subpaths`'
header (`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:559-570`) records the
original report:

> *"Measured on a real SolidWorks export: object 5870 of page 1 is a single
> stroked path with **1194 subpaths and 6681 anchors** covering a 550×500 pt
> isometric view — every visible line of that view is one object as far as
> `hit_test_point` is concerned. Clicking any line selects all of them, and the
> operator's report was the direct consequence: 'how do I click on individual
> lines and nodes to move or delete them?'"*

**O188 is that same report, in text.** `OPERATOR_REQUESTS.md:494-498` already says
so: *"This is the **second** report of the same underlying fact … the shell's
unit of selection is not the operator's."*

⇒ **The design decision is therefore already made, and made well.** Do not design
a new gesture. Do not add a modifier. Do not add a context-menu item. Press `A`,
click the label, drag — and make the drag reach a verb.

---

### 9. What will bite

Ordered by how likely each is to cost a day.

1. **★★★ His title block may be a form XObject, in which case none of this
   reaches it.** `canvas/target.rs:566-570` answers **empty** for a run inside a
   form, by design and with the comment saying so. `ENGINE_BACKLOG.md:270`
   records that a SolidWorks title block is commonly *one form drawn on
   thirty-six sheets*. **Establish this on his file before anything else** —
   if it is a form, O188 needs `text_run_hits` on a leaf index AND
   `move_text_run_in_form` AND `delete_text_run_in_form`, and *"delete works
   today"* is false for him even though it is true in general.

2. **★★ The `Inherited` run has no coordinates and cannot be moved at all.**
   Some of his labels will move and some will refuse, with **nothing visible to
   distinguish them** — the same asymmetry `delete_text_run`'s §9.4.2 guard
   already produces, which the shell pre-empts at `canvas/deleting.rs:305`. The
   move path will owe the identical pre-check, or he will file *"sometimes it
   works"*.

3. **★★ Moving one run can move its successor.** If run N+1 is `Inherited` and
   run N's `Tm` is rewritten, N+1 slides. On a title block that is a whole
   column of labels marching off. Whichever policy the engine picks, the shell
   must disclose it — and a *silent* slide is the worst possible outcome on page
   content pdfcer must not alter (R8b Rule 15).

4. **★ Editing inside a shared form changes every sheet.** Decision 076 is
   settled — edit in place, disclose, `unshare_form` as the *option* not the
   precondition (`docs/core-api/02-editing-and-saving.md:1204-1210`). But the
   operator dragging a label on sheet 3 of 36 must be told before the release,
   not after. `ENGINE_BACKLOG.md:270` names this exactly: *"the operator wants to
   know the blast radius **before** picking up the tool"*.

5. **★ `TextObject::approximate` is always `true`** (`vector/decompose.rs:698`,
   per `01-reading-and-model.md:2092`). The outline drawn round a label is an
   approximation. It affects what he sees, not what is edited — but a ghost that
   does not sit exactly on the glyphs will read as a bug during the drive.

6. **The `Selection::subpath` field is named for paths and carries runs.** It is
   correct and documented (`canvas/selection/identity.rs:110-128`) and it is a
   standing trap for the next reader. Do not rename it inside this work; note it.

7. **Doc line numbers in `docs/core-api/` are stale by roughly 9,000 lines** for
   `edit.rs` (`delete_text_run` published at 4825, measured at 13951). Cite
   measured lines in the engine request or the reviewer will look in the wrong
   place and conclude the verb was moved.

8. **A single-run text object's Part delete is a whole-object delete**
   (`vector/edit.rs:1432-1441`). Harmless, but any driven check that asserts
   `objects_after == objects_before` will fail on such a fixture for a correct
   build — which is why `deeper_rung_delete`'s fixture has **six** runs.

---

### 10. The driven checks owed

House convention, verified against the directory: `tools/ui-verify/src/checks/`
holds **210** files; names are lowercase words joined by underscores with **no
digits** (e.g. `deleting_a_label_leaves_the_other_labels_alone`,
`fit_places_the_view`, `deeper_rung_delete`).

#### 10.1 Owed for the shell-only work (§6.1) — buildable now

**`a_label_that_cannot_be_moved_says_so`** — new file
`tools/ui-verify/src/checks/label_move_refusal.rs`.

- **Defect it guards:** *"Dragging one label inside a block of text does nothing
  and says nothing, so from where the operator sits the program is broken — the
  same state `Refusal::InsideForm` was given a sentence for, left mute for the
  one refusal an operator meets without having made a mistake."*
- **Drive:** open `fixtures/paragraph.pdf`; press `A` (Points); click
  `0,120,704` to enter the Part rung on run 0; press and drag 20 pt; release.
- **Oracle:** the decline reaches a surface. Assert the status row carries the
  sentence AND that the trace shows no `move-*-applied` line — a build that
  moved the whole paragraph instead would be the wrong kind of "something
  happened".
- **Falsify by:** removing the new arm from `decline()` and confirming the check
  goes red; and by routing the arm to `MoveSubject::Objects` and confirming it
  goes red for the *other* reason.

#### 10.2 Owed when the engine verb lands (§6.3)

**`dragging_a_label_moves_only_that_label`** — add to
`tools/ui-verify/src/checks/deeper_rung_delete.rs`'s sibling, or a new
`label_move.rs`.

★ **The assertion that matters is the same pair `deeper_rung_delete` teaches**
(`deeper_rung_delete.rs:21-35`): *"something moved"* passes on the exact bug this
is built to prevent — a build that routed the Part rung to `move_objects` slides
**every label on the sheet** and traces a perfectly healthy move.

| must hold | what a wrong build does |
|---|---|
| the dragged run's page-space bounds change by the drag delta | unchanged (nothing happened) |
| **every other run's bounds are byte-identical** | all 237 slide together |
| `objects_after == objects_before` | unchanged either way — this half proves nothing alone |
| the trace carries BOTH `move-text-run-applied …` (the shell's census) and the funnel's `move-text-run page=… n=1 epoch=…` (the engine accepted it) | a build that computed the census and never reached `EditSession` writes only the first |

⚠ The `-applied` suffix is load-bearing — `check-trace-names.py` exists because a
module line sharing its first token with a funnel label is the one `Trace::last`
returns (`deeper_rung_delete.rs:52-56`, three recorded instances).

**`a_label_whose_neighbour_would_slide_refuses_before_the_drag`** — the move twin
of the §9.4.2 pre-check the delete path already makes at
`canvas/deleting.rs:305`. Needs a fixture whose run N+1 is
`RunPositioning::Inherited`; `crates/pdfcer-core/tests/text_run_delete.rs` is
cited by `canvas/deleting/tests.rs:86` as having built one — reuse its shape.

**`a_label_inside_a_title_block_form_can_be_reached`** — only if §9.1 turns out
to apply to his file. Fixture: `fixtures/a1-titleblock.pdf`, which is present in
the tree and which `OPERATOR_REQUESTS.md:4390` describes as *"a real CAD title
block"*.

#### 10.3 Already green, needs nothing

`deleting_a_label_leaves_the_other_labels_alone`
(`tools/ui-verify/src/checks/deeper_rung_delete.rs:392`) already covers the
delete half of O188 end to end. **It should be named in the reply to him** as the
evidence that the half he asked for exists.

---

### 11. What I could not measure

Stated plainly rather than guessed, because an honest gap is useful and a
confident wrong answer costs a day.

1. **Whether HIS title block's text is a page object or a form leaf.** Decisive
   for §9.1 and I have no access to his file. `fixtures/a1-titleblock.pdf` is in
   the tree but I did not open or decompose it (no cargo, and I did not want to
   spend the survey on a fixture that may not match his producer).

2. **Whether his title block's runs are `Explicit` or `Inherited`.** This decides
   whether the move verb would refuse on most of them. Measurable with
   `pdfcer extract-text --json --spans` or a decompose dump; not run here.

3. **Whether the engine could re-space a following `Inherited` run.** I read that
   `edit_text` does something of this shape (`Pass 256.0`, per
   `docs/core-api/02-editing-and-saving.md:387`) but I did **not** read that code
   path in `text_edit/` to confirm the machinery is reusable for a geometric move
   rather than a text replacement. The engine request asks rather than asserts.

4. **The exact runtime cost of `hit_test_text_runs` on a 237-run object.** It is
   a linear scan with a `contains` per run (`vector/hit.rs:538-551`), so it is
   almost certainly fine, but `OPERATOR_REQUESTS.md:9534` has a table comparing
   *"ordinary A1 title block"* against *"dense CAD site plan"* per call that I did
   not read.

5. **Whether `moving::drag`'s ghost renders correctly for a run.** I measured
   that `part_bounds_of` answers for a run via `text_run_bounds_canvas`
   (`panels/objects/provider/mod.rs:550`) on the page-object path, but
   `canvas/target.rs:574-578`'s `part_bounds_of` override calls only
   `subpath_bounds_canvas_of` — so the **form-leaf** path may draw no run
   outline. Not measured whether that matters before the verb exists.

6. **Whether any status sentence in the NEW shell tells him a run cannot be
   moved.** The OLD gui had one (`ui_text.rs:4630`, `entered_object_readout` with
   an `is_run` flag). I grepped the new shell's `text/` for an equivalent and
   found no `is_run` and no `no_verb_for_part` wording — but a negative grep is
   weaker evidence than a positive one, and the sentence may exist under a name I
   did not guess.

7. **Whether O188 has an engine-request file already drafted somewhere.** I found
   none under the repo (`find -name "request_*.md"` returned nothing outside
   `target/`), and `OPERATOR_REQUESTS.md:12778` references
   `request_restyle_an_existing_text_run.md` as a file that exists — so requests
   live somewhere I did not locate. **Find that location before filing §7.**

8. **Cargo, tests, and the binary were not run**, per the session constraint. Every
   claim above is from reading source and committed documents. No claim here has
   been driven by me.

---

## O189 — bookmarks do not transfer when a page is dragged between documents

Survey + design. Read-only pass; nothing was edited, no `cargo` was run.
Every `file:line` below was measured in this session with `grep -n` / `sed -n`
against the working tree as it stands now. Where I could not measure something
it is listed in §7 rather than guessed at.

---

### 0. The answer in one line

**The operator is right, the engine already has the rule written down for a
different verb, and the rule should be the same one.** `pdfcer-core` decided in
`pageops/outline.rs` that an *extract* carries every outline entry whose
destination lands on an extracted page, keeps the ancestor headings above it as
destination-less containers, and counts what it dropped. A cross-document page
drag **is** an extract that happens to land in an already-open document, so it
should carry the same thing by the same rule — and the one part that is
genuinely new (what happens to the *source* when the gesture was a move) is the
part that needs the operator's approval, not the carry itself.

---

### 1. Measured facts — the existing drag path

#### 1.1 The gesture, end to end

| stage | where | measured |
|---|---|---|
| drag state that survives a document switch | `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\pagedrag.rs` (525 lines) | `wc -l` |
| the drop resolves a gap and raises the action | `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\canvas\pagedrop.rs:190` (`pagedrag::end`), `:221` (`wants_move`), `:235` (`insert_position`) | `grep -n` |
| the cross-document arm | `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\app\actions\crossdoc.rs:126` — `PdfcerApp::apply_insert_from_open_document(&mut self, source_slot, pages: &[usize], position, take: bool)` | `grep -n` |
| the transfer itself | `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\app\actions\crossdoc.rs:196` — `super::pages::insert_from_view(target, &view, pages, position)` | `grep -n` |
| the shared insert half | `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\app\actions\pages.rs:812` — `pub(super) fn insert_from_view(doc, view: &DocumentView, pages: &[usize], position) -> usize` | `grep -n` |
| **the engine verb** | `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\app\actions\pages.rs:851` — `session.insert_pages(view, pages, position)` | `grep -n` |
| the move's second half | `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\app\actions\crossdoc.rs:225` — `fn take_pages_from(..)`, which at `:279` runs `vector_edit(source, "page-move-take", ..)` around `super::pages::delete(session, pages, separations)` | `grep -n` |

So: **one engine verb performs the transfer —
`pdfcer_core::edit::EditSession::insert_pages`, at
`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:39906`** — and a move is that
verb followed by `delete_pages_with` on the source
(`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:38538`), each on its own undo
stack. `crossdoc.rs` §2 (lines 22–41) carries the reasoning for why the default
is a copy at all.

#### 1.2 Why nothing comes across today, stated by the engine itself

`insert_pages` copies **outward from the pages**. `/Outlines` is a **catalog**
entry, unreachable from any page, so the copier never sees it. That sentence is
not mine — it is in the shell's own text module at
`D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\text\pages.rs:599-604`:

> `/Outlines` is a **catalog** entry, unreachable from any page, so a copy that
> walks outward from the pages never sees it. They are not lost in transit —
> they were never in the set of objects being copied. Which is why carrying
> them means replaying the source outline through `add_outline_item`, i.e.
> exactly what the Bookmarks panel now does by hand.

That last clause is the design, already written down by someone who was not
building this. It is the route §4 takes.

#### 1.3 What the operator is told today

`insert_pages` returns `InsertOutcome`
(`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:3465`) whose
`source_outline_dropped` field (`:3548`) is computed at `:40007` as, verbatim:

```rust
let source_outline_dropped = source
    .graph()
    .catalog_dict()
    .is_some_and(|c| c.contains_key(b"Outlines"));
```

**It is a test for "the source had an `/Outlines` key at all", not for "any
bookmark pointed at a page you dragged".** The shell turns it into one clause,
at `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\text\pages.rs:685`:

```rust
if structures.outline_dropped {
    line.push_str(" That file's bookmarks did not come across.");
}
```

Measured consequence: **the operator dragging one page out of a bookmarked
manual is already being told the bookmarks did not come across.** The report is
honest and it is already off-canvas. What O189 asks for is not disclosure — it
is the capability the disclosure is apologising for.

---

### 2. Measured facts — what the engine offers for outlines

`D:\Dev\pdfcer\docs\core-api\index.md` contains **no** occurrence of `outline`
or `bookmark` (`grep -rn "outline\|bookmark\|Outline\|Bookmark"` → no output).
The core-api index does not document this surface at all; every signature below
came from the source.

#### 2.1 Reading document A's outline with destinations resolved to page indices — **YES**

```rust
// D:\Dev\pdfcer\crates\pdfcer-core\src\outline.rs:1547
pub fn read_outline<G: ObjectGraph + ?Sized>(graph: &G) -> Outline
```

- **Infallible** by contract (`outline.rs:1505`): malformed input yields a
  partial tree plus populated `OutlineDiagnostics`, never a panic, never an
  unbounded walk. Bounded three ways — `MAX_OUTLINE_ITEMS`, `MAX_OUTLINE_DEPTH`
  = 32 (`outline.rs:218`), and no object visited twice.
- It builds a `page_slots`-derived `ObjId → 0-based index` map
  (`outline.rs:1553-1560`) and resolves destinations through it, so items come
  back as `Destination::Page { page_index, view }` — **already in A's page
  index space** (`outline.rs:423`).
- `DestView` (`outline.rs:640`) carries the full Table 151 fit style:
  `Xyz { left, top, zoom }`, `Fit`, `FitH`, `FitV`, `FitR`, `FitB`, `FitBH`,
  `FitBV`. So **an explicit XYZ destination survives reading with its zoom and
  point intact.**
- Unresolvable destinations are *kept*, as the most specific variant the file
  supports: `Destination::UnmappedPage`, `Destination::Named`,
  `Destination::Remote`, `NonNavigation`.

**Critically for us: it takes any `&dyn ObjectGraph`, and
`pdfcer_core::view::DocumentView` implements `ObjectGraph`
(`D:\Dev\pdfcer\crates\pdfcer-core\src\view.rs:387`) and exposes
`graph()` at `view.rs:339`.** The parked source document in `crossdoc.rs:194`
is already a `DocumentView`. So **A's outline is readable from exactly the
borrow the cross-document drag already holds**, with no new borrow shape and no
`&mut` on the parked session.

#### 2.2 Inserting an outline entry into B at a given parent/position — **YES**

```rust
// D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:41729
pub fn add_outline_item(
    &mut self,
    parent: Option<ObjId>,
    title: &str,
    destination: Option<crate::outline::Destination>,
) -> Result<ObjId, EditError>
```

`parent: None` means top level. It maintains `/Count` up the ancestor chain and
opens a parent that was a leaf (`edit.rs:41700-41709`). It **refuses by name**
rather than silently dropping: `EditError::UnsupportedDestination` for
`Remote` / `UnmappedPage` / `NonNavigation`, `NamedDestinationNotFound` for a
`Named` key nothing defines, `PageOutOfRange` for a page B does not have
(`edit.rs:41713-41728`). The refusal happens **before anything is allocated**,
so a refused add leaves the session byte-for-byte untouched (`edit.rs:41741`).

Placement anchors are `OutlinePlacement` (`edit.rs:1290`) —
`FirstChild { parent: Option<ObjId> }`, `LastChild { parent }`,
`Before { sibling }`, `After { sibling }`.

#### 2.3 An existing "import outline subtree" verb — **YES, TWO OF THEM**

**(a) The clipboard pair.** This is the closest existing thing to O189 and it
already works *between two documents*:

```rust
// D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:46254
pub fn copy_outline_item(&self, item_id: ObjId)
    -> Result<crate::outline::OutlineClip, EditError>

// D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:46276
pub fn cut_outline_item(&mut self, item_id: ObjId)
    -> Result<crate::outline::OutlineClip, EditError>

// D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:46307
pub fn paste_outline_item(&mut self, clip: &OutlineClip, placement: OutlinePlacement)
    -> Result<OutlinePasteOutcome, EditError>
```

- `OutlineClip` / `OutlineClipItem` at `outline.rs:1177` / `:1186`: the
  **logical** subtree — `title: String`, `destination: Option<Destination>`,
  `open: bool`, `color: Option<[f64;3]>`, `style_flags: Option<i64>`,
  `children: Vec<OutlineClipItem>`. Built by `outline_item_to_clip`
  (`edit.rs:46442`), a field-for-field copy of `OutlineItem`, so
  `Destination::Page { page_index, view }` — **including the XYZ/FitR
  parameters** — is carried.
- `OutlineClip::deepest_page()` (`outline.rs:1240`) lets a shell say, *before*
  the press, how many destinations will not survive in the target.
- `OutlineClip::to_bytes` / `from_bytes` (`:1260`, `:1288`) with magic
  `PDFCEBKM\x00\x00\x00\x01` (`:1307`) — it survives leaving the process. <!-- old-name-exempt: the ENGINE's clipboard magic bytes, quoted verbatim. It is an on-the-wire constant a reader must match byte for byte; 'correcting' the spelling here would make the citation wrong. -->
- `paste_outline_item` is **ONE undo entry** however many bookmarks arrive
  (`edit.rs:46316`, and the `coalesce_last` at `:46349` — with a measured-not-
  counted note at `:46325` recording that counting intended commands instead of
  committed ones once silently produced three undo entries).
- Its per-item worker `paste_outline_subtree` (`edit.rs:46355`) **drops** a
  destination the target cannot honour rather than clamping it:
  ```rust
  Some(Destination::Page { page_index, view }) if *page_index < pages => …keep…
  Some(_) => { outcome.destinations_dropped += 1; None }
  ```
  reported as `OutlinePasteOutcome { items_pasted, destinations_dropped }`
  (`edit.rs:19401`).

**★ The gap, stated precisely: `paste_outline_item` does NOT remap page
indices.** It takes `page_index` from A's page space and tests it against B's
page *count*. For O189 the pages have landed at a known offset in B, so every
carried `Destination::Page` needs `page_index` rewritten from A's index to B's
before the paste. That rewrite does not exist anywhere in the engine today.

The GUI already drives this pair: `panels/bookmarks/clip.rs:114`
(`doc.session.copy_outline_item(selected.id)`), `:192`
(`OutlinePlacement::LastChild`), and `app/actions/bookmarks.rs:508`
(`session.paste_outline_item(clip, to)`). Acrobat, per the engine's own note at
`edit.rs:46241`, *cannot* copy bookmarks between files at all — this is already
an exceed.

**(b) The merge path, which carries outlines wholesale.**

```rust
// D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:39035
pub fn merge_document(&mut self, source: &DocumentView<'_>, position: InsertPosition)
    -> Result<MergeOutcome, EditError>
```

`MergeOutcome::outline_items_carried` at `edit.rs:3652`, set at `:39096` from
`merge_outline` (`edit.rs:39270`). `merge_outline` imports the source's
top-level `/First`/`/Next` chain through `import_object` on the **same
`PageSplice::mapping`** the pages used (`edit.rs:8762-8783`), so `/Dest` page
references re-point automatically — and then splices the arrivals onto the end
of B's top-level chain via `append_outline_items` (`edit.rs:39351`), creating
`/Outlines` when B has none.

**This works only because a merge takes every page.** The mapping contains
every source page, so no `/Dest` can point outside it. For a page *subset* the
same code would be actively dangerous: `import_object` following a `/Dest`
whose page is not in the subset would copy that page object into B as a loose
object. That is the hazard named in §5.

The shell already reports it: `text/pages.rs:986` —
`"{n} bookmark(s) came across, added after this document's own."`

**(c) The policy that already answers O189's hard question.** The engine has
written this ruling down — for extract and split, not for insert — in
`D:\Dev\pdfcer\crates\pdfcer-core\src\pageops\outline.rs:1-63` and
`D:\Dev\pdfcer\crates\pdfcer-core\src\pageops\assemble.rs:168`:

```rust
pub enum OutlinePolicy {
    /// Carry every outline entry whose destination lands on a copied
    /// page, rewritten to point at the copy; drop the rest, counted.
    Subset,
    /// One top-level entry per source document, that source's entries nested.
    PerSource,
    /// No outline at all.
    Drop,
}
```

And the header at `pageops/outline.rs:33-38` says, in terms, why *insert* got
neither:

> Insert takes **neither**: it carries the target's outline … and does not
> import the source's, which matches `core_ops__insert_pages.md`'s recommended
> default — *"No bookmark carryover on plain Insert … bookmark-carrying insert
> should be a distinct, explicitly-named mode if ever added."*

**A cross-document page drag is the "distinct, explicitly-named mode" that
document anticipated.** It is not plain Insert-from-file: the operator picked
specific sheets out of a document that is open in front of them, with its
bookmark tree visible in a panel.

The `Subset` implementation's two working parts are worth naming because §4
reimplements them at the logical level:

- `Item::is_kept` (`pageops/outline.rs:124`) — an entry is kept if its own
  destination is a kept page **or any descendant's is**. A container whose only
  value is holding kept children survives; dropping it would reparent its
  children to the root and destroy the hierarchy the operator can see.
- `prune` (`pageops/outline.rs:516`) — a kept container whose *own* destination
  left keeps its title and its children and simply has **no** destination.
  Verbatim from the source: *"Clicking it does nothing, expanding it works —
  which is what a chapter heading whose title page was not extracted should
  do."* Dropped subtrees are counted whole (`count_all`, `:548`), because
  reporting "1 bookmark dropped" for a 40-entry chapter *"would be true and
  useless."*

That is the answer to the question in the row note — *"moving one page out of a
five-page chapter cannot carry the chapter heading without deciding what happens
to the other four"* — and **pdfcer already decided it, in writing, for extract.**

---

### 3. Measured facts — what happens today to a destination that pointed at a moved page

#### 3.1 In A (the source), after a *move*

**A keeps a dangling entry. It is not fixed up, and that is deliberate and
documented.**

`delete_pages_with` (`edit.rs:38538`) runs `census_dangling` **before** the
splice (`edit.rs:38584`) and returns `DeleteOutcome.dangling: DanglingReport`
(`D:\Dev\pdfcer\crates\pdfcer-core\src\pageops\references.rs:335`):

| field | meaning |
|---|---|
| `outline_items` | outline items whose destination is a removed page |
| `links` | link annotations **on surviving pages** pointing at a removed page |
| `non_link_annotations` | non-link annots on surviving pages whose `/A /GoTo` names a removed page |
| `named_destinations` | named destinations resolving to a removed page |
| `page_labels_stale` | bool — `/PageLabels` now numerically stale |

Nothing repairs the outline. `delete_pages`' own doc says so explicitly at
`edit.rs:38505-38511`, contrasting it with the one class pdfcer *does* repair:
*"see `DeleteOutcome::separations` for why this one class of broken reference
is repaired when bookmarks are not."*

**And it is already reported on the drag path.** The move's second half at
`crossdoc.rs:279` captures the source's notes and re-files them under the
*target's* epoch (`crossdoc.rs:262-268` explains why: `app::status` draws only
the active document's disclosure, so a note filed against the parked source
would be *recorded, correct, and invisible*). The sentence itself is
`D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\text\pages.rs:400`, pushed at
`app/actions/pages.rs:621`:

> `"{count} bookmarks now point at pages that are no longer in this document."`

So today, a Shift-drag of a bookmarked page out of A produces, on B's status
row: *"N sheets were removed from A…"* + *"N bookmarks now point at pages that
are no longer in this document."* — **both true, and the second one is about A
while the operator is looking at B.** That is a live wording hazard, listed in
§5.

#### 3.2 In B (the target), after either a copy or a move

Nothing arrives. B's own outline is untouched; the operator gets
`"That file's bookmarks did not come across."` (`text/pages.rs:685`) whenever A
had an `/Outlines` key, *whether or not any bookmark pointed at a dragged page*.

---

### 4. The rule — alternatives, argued, then a recommendation

Four candidates. For each: what happens to A, to a split chapter heading, to an
explicit XYZ destination, and to a B with no outline at all.

#### Candidate 1 — flatten to B's top level, in page order

Carry only entries whose own destination is one of the moved pages; discard
ancestry; append to B's top level sorted by landing page.

- **A:** unchanged on a copy; on a move, dangling entries left and counted
  (unchanged from today).
- **Split chapter heading:** *lost.* Dragging pages 3–4 of a five-page chapter
  gives B two bookmarks named after the two sub-headings and no clue which
  chapter they were in. On a drawing set where every sheet's bookmark is
  *"Sheet 12"*, B gains twelve identically-shaped entries with no discipline
  heading above them.
- **XYZ destination:** survives — `DestView` is carried whole by
  `outline_item_to_clip`, only `page_index` is rewritten.
- **B has no outline:** `add_outline_item` creates `/Outlines`
  (`append_outline_items`, `edit.rs:39351`, does the equivalent for merge).

Cheapest to build, and the only one that loses information the operator can
see on screen in the Bookmarks panel at the moment they drag. **Rejected.**

#### Candidate 2 — preserve ancestry, synthesising the ancestor headings in B

Carry every entry whose destination is a moved page, **plus every ancestor of
such an entry**, with each surviving ancestor emitted as a destination-less
container carrying its original title.

- **A:** identical to candidate 1 — unchanged on copy, dangling-and-counted on
  move.
- **Split chapter heading:** the heading comes across as a **container**, with
  only the moved children under it. The other four pages' entries do not come;
  they stay in A, where their pages still are. Clicking the heading in B does
  nothing; expanding it works. This is `prune`'s documented behaviour at
  `pageops/outline.rs:527-531`, word for word, and it is the behaviour extract
  already ships.
  - The one wrinkle: if the chapter heading's *own* destination happened to be
    one of the moved pages (a title page), it is kept **with** its destination,
    remapped. `prune`'s `.filter(|(page,_)| kept_pages.contains(page))` gives
    exactly that.
- **XYZ destination:** survives whole, as candidate 1.
- **B has no outline:** created.
- **Where in B:** appended at top level, after B's own entries, in the order the
  carried roots appeared in A. This matches `merge_outline`/`append_outline_items`
  and matches `text/pages.rs:986`'s existing sentence *"added after this
  document's own"*, so a second placement rule is not introduced.

**This is the recommendation.** Reasons, in order of weight:

1. **It is already pdfcer's ruling.** `OutlinePolicy::Subset` is exactly this,
   written down in `pageops/outline.rs:14-24` as a *documented pdfcer decision*
   sourced from the RAG's own recommendation, applied to extract and split.
   Deciding the opposite for a drag would mean the same operator, on the same
   five-page chapter, gets a chapter heading when they use Extract and no
   chapter heading when they drag — a divergence with no defence.
2. **The information exists and is free.** A's outline is readable from the
   `DocumentView` the arm already holds (`view.rs:387`), the destinations come
   back already resolved to page indices (`outline.rs:1547`), and the target
   offset is already computed as `landing` at `app/actions/pages.rs:834-845`.
3. **It is the only candidate that answers the row note's objection honestly.**
   A chapter heading whose children were split is not a contradiction; it is a
   container in both documents, describing whichever pages each document
   actually holds.
4. **The failure mode is benign.** A destination-less heading is a legal
   §12.3.3 shape (`paste_outline_item`'s doc at `edit.rs:46296` relies on
   exactly that), and the worst an operator suffers is a click that does
   nothing on a node that expands.

#### Candidate 3 — carry nothing, report off-canvas what was left behind

- **A:** unchanged.
- **Split heading / XYZ / empty B:** all moot.
- This is **what ships today** (`text/pages.rs:685`), and it is a *correct*
  rule, not a bug. The operator's complaint is not that they were misled — the
  disclosure is accurate and prominent. Their complaint is that it apologises
  rather than doing the work.
- Keep it as the fallback when the carry **cannot** run (see §5), never as the
  default. **Rejected as the rule; retained as the decline path.**

#### Candidate 4 (the better one the brief invited) — carry by rule, and offer the mirror on the source

Candidate 2, **plus**: when the gesture was a **move** (Shift), also *remove*
from A the entries that were carried and whose destination left with the page —
the same set, by the same `is_kept` test — so A is not left with dangling
bookmarks the operator now has to clean by hand.

The argument for it: a move that carries a bookmark to B and leaves a broken
copy of it in A has produced a **third state**, which is the exact failure
`crossdoc.rs:70-76` already names for pages (*"the pages are now in both
documents"*). Applying the same reasoning to bookmarks is consistent.

The arguments against it, which is why this is the operator's call and not
mine:

- **It is a third command on A's undo stack.** `crossdoc.rs` §2 spent its
  entire header establishing that a cross-document move is already two edits on
  two stacks with no single Ctrl+Z. Adding an outline delete makes it three,
  and `cut_outline_item` (`edit.rs:46276`) coalesces only *its own* pair.
- **It deletes something the operator did not point at.** A container heading
  whose destination left is still a heading for the four pages that stayed.
  Deleting it would take a working bookmark out of A.
- A narrower form exists: delete only **leaf** entries whose destination was a
  moved page, and leave every container alone. That is defensible and small.

**Recommendation: ship candidate 2 now; put candidate 4 in front of the
operator as a follow-on question, defaulted OFF.** Rationale: candidate 2 is
purely additive — it cannot make any document worse than it is today — whereas
candidate 4 mutates a document the operator is not looking at, on a gesture
whose undo story is already the weakest in the application.

#### The recommended rule, stated for the operator to approve

> **When pages are dragged from one open document into another, the bookmarks
> that point at those pages come with them.**
>
> A bookmark comes across when it points at one of the pages you dragged. Its
> parent headings come too, so the pages arrive under the chapter they were in
> — but a heading whose own page stayed behind arrives as a heading only: you
> can expand it, clicking it goes nowhere. Bookmarks pointing at pages you did
> not drag do not come.
>
> The arriving bookmarks are added after the target document's own, and they
> keep their zoom and position. If the target had no bookmarks, it gets a
> bookmark list.
>
> Bookmarks that pdfcer could not carry — a bookmark that opens another file,
> or one whose destination this file never defined — are left behind and
> counted, on the status row, at the moment it happens.
>
> **The source document is not changed.** On a Shift-drag (a move), the pages
> leave and its bookmarks that pointed at them stay, now pointing at nothing —
> which pdfcer already tells you. Cleaning those up is a separate decision.

---

### 5. The build

Three edits, one new text function, one new check. No engine change is
*required* — everything needed is public — but §5.4 argues for one.

#### 5.1 A new shell-side function: collect A's carryable subtree, remapped

New in `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\app\actions\crossdoc.rs` (it is
the only arm that reads two documents, per its own header at `:5-9`), or in a
sibling `crossdoc/outline.rs` if the arm gets long:

```
fn carried_outline(
    view: &DocumentView<'_>,   // the SOURCE, already held at crossdoc.rs:194
    moved: &[usize],           // source page indices, as the action carries them
    landing: usize,            // B index where the first sheet lands
) -> (OutlineClip, CarryReport)
```

Algorithm, mirroring `pageops::outline::prune` at the logical level:

1. `let outline = pdfcer_core::outline::read_outline(view.graph());`
   (`outline.rs:1547`). Check `outline.diagnostics.is_faithful()` — a truncated
   tree must not be carried as if it were the document's own; see §6/§7.
2. Build `remap: HashMap<usize, usize>` from `moved`: A's page index → B's page
   index, i.e. `moved` sorted ascending, the *k*-th entry mapping to
   `landing + k`. **The insert places the dragged pages contiguously in source
   order at `landing`**, which is what `insert_from_view`'s `landing`
   computation (`app/actions/pages.rs:834-845`) and `copy_and_splice` together
   establish — verify against the driven check (§7: I did not measure
   `copy_and_splice`'s ordering).
3. Depth-first over `outline.items`, recursion capped at `MAX_OUTLINE_DEPTH`
   (32, `outline.rs:218`):
   - `is_kept(item)` = `item.destination` is a `Destination::Page` whose
     `page_index` is in `remap`, **or** any descendant is kept.
   - a kept item emits an `OutlineClipItem` with `title`, `open`, `color`,
     `style_flags` copied verbatim (as `outline_item_to_clip`, `edit.rs:46442`),
     `children` = the kept children, and `destination` =
     `Some(Destination::Page { page_index: remap[old], view: view.clone() })`
     when its own destination is a moved page, else `None`.
   - a dropped subtree increments `report.dropped` by `1 + count_all(children)`
     (`pageops/outline.rs:539-546`'s reasoning: reporting 1 for a 40-entry
     chapter is true and useless).
   - a kept item whose destination is a `Named`, `Remote`, `UnmappedPage` or
     `NonNavigation` variant emits with `destination: None` and increments a
     **separate** counter (`report.unresolvable`), because the remedy differs —
     see §6.
4. Return the clip and the report.

**Why rebuild the clip rather than call `copy_outline_item`:** that verb takes
one `item_id` and copies a whole subtree unfiltered; there is no per-entry
filter and no remap, and it needs `&self` on the *source session* (which
`crossdoc.rs` does not hold — it holds a `DocumentView`). `read_outline` on the
view is the right borrow and the right granularity.

#### 5.2 Splice it into the arm

In `apply_insert_from_open_document` (`crossdoc.rs:126`), **after**
`insert_from_view` returns a non-zero count (`crossdoc.rs:196`) and while the
target is still borrowed:

```
let inserted = super::pages::insert_from_view(target, &view, pages, position);
if inserted == 0 { return; }                     // unchanged ordering discipline
let (clip, carry) = carried_outline(&view, pages, landing);
if !clip.is_empty() {
    // one undo entry, folded by paste_outline_item itself (edit.rs:46349)
    target.session.paste_outline_item(&clip, OutlinePlacement::LastChild { parent: None })
}
```

through the existing `super::apply::vector_edit` funnel, so the epoch bump,
render-worker cancel and `pages::resync` all happen as they do for every other
edit (`crossdoc.rs:270-278` explains why that funnel is not optional even for a
parked document).

**Ordering is insert-then-outline and cannot be reversed**, for the same reason
`crossdoc.rs:61-67` gives for insert-then-delete: `paste_outline_item` tests
`page_index < pages` against B's *current* page count
(`edit.rs:46370, 46396`), so pasting first would drop every destination.

**`landing` must be lifted out of `insert_from_view`.** Today it is computed
locally at `app/actions/pages.rs:834-845` and never returned; the function
returns only a `usize` count (`pages.rs:812`). Change its return to a small
struct `{ inserted: usize, landing: usize }` — `insert_from_view` has exactly
two callers (`pages.rs:785`, `crossdoc.rs:196`), both measured.

#### 5.3 Two undo entries, and say so or fold them

`insert_pages` commits one `CommandKind::InsertPages` (`edit.rs:40016`);
`paste_outline_item` coalesces its own into one `CommandKind::PasteOutlineItem`
(`edit.rs:46349`). They are **on the same stack (B's)**, so unlike the
insert/delete pair this one *is* foldable — but only if the shell calls
`coalesce_last(2, …)`, and I could not find a public `coalesce_last` (§7).

Two honest options:
- **(a)** leave two entries and say nothing — Ctrl+Z removes the bookmarks,
  Ctrl+Z again removes the pages. Defensible, slightly surprising.
- **(b)** ask the engine for `insert_pages_with_outline(view, pages, position,
  OutlinePolicy)` returning an extended `InsertOutcome`, folding both into one
  command and doing the remap inside, where `PageSplice::mapping`
  (`edit.rs:8766`) makes it exact rather than positional.

**(b) is the better engineering** and is the one engine change worth asking
for — see §5.4 — but it is a `D:\Dev\pdfcer\` change and this survey is
read-only there. **(a) ships without touching the engine.**

#### 5.4 The engine change worth asking for (not required, not made here)

`insert_pages` at `edit.rs:39906` already computes the mapping and then throws
it away — literally, at `edit.rs:39922`:

```rust
let _ = &mapping;
```

That line is the hook. An `insert_pages_with(…, OutlinePolicy)` that reuses
`pageops::outline`'s `Item`/`is_kept`/`prune` against `PageSplice::mapping`
would (i) remap by object identity instead of by position, removing the
assumption in §5.1 step 2 entirely; (ii) fold to one undo entry; (iii) put the
rule in the one place `pageops/outline.rs` already documents it, rather than in
a second shell-side implementation that will drift. Filing it as a request is
the right move; building around it with (a) meanwhile is also right.

#### 5.5 The text

New function in `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\text\pages.rs`,
beside `inserted` (`:672`) and following `deleted_dangling_bookmarks`'
singular/plural discipline (`:400` — spelled out, never `1 bookmark(s)`):

```
pub fn bookmarks_carried(carried: usize, dropped: usize, unresolvable: usize) -> Option<String>
```

- `carried > 0` → *"N bookmarks came across with those pages, added after this
  document's own."* (mirrors `text/pages.rs:986-990`, which already says
  exactly this for merge — reuse the phrasing verbatim so two verbs do not
  describe one outcome two ways).
- `dropped > 0` → *"M bookmarks pointed at pages you did not drag and stayed
  in <source>."*
- `unresolvable > 0` → *"K bookmarks opened another file or a destination this
  file does not define, and could not be carried."* — separate clause because
  the remedy is separate (nothing the operator can do here vs. drag the other
  pages too).
- each clause dropped when its count is zero, per the `Structures` ruling at
  `text/pages.rs:586-596` (*"it used to be 'always', and that made it a
  disclaimer rather than a disclosure"*).

And **`Structures::outline_dropped` must stop firing when the carry ran.** Today
it is set from a bare `/Outlines` key test (`edit.rs:40007`) and the shell
prints *"That file's bookmarks did not come across"* (`text/pages.rs:685`). With
the carry in place that sentence would be false. Under build (a) the shell must
suppress it on the cross-document path whenever `carried > 0`; under build (b)
the engine sets it correctly.

---

### 6. Rule 4 — what gets reported, and where

The carry performs **three inferences the operator cannot see**, and each owes
a report:

| inference | report | surface |
|---|---|---|
| which bookmarks matched the dragged pages | *"N bookmarks came across…"* | status row disclosure |
| which bookmarks were left behind because their pages stayed | *"M bookmarks pointed at pages you did not drag and stayed in `<source>`."* | same line, second clause |
| which bookmarks could not be expressed at all (remote / named-undefined / unmapped) | *"K bookmarks opened another file or a destination this file does not define, and could not be carried."* | same line, third clause |
| a heading arrived **without** its destination because its own page stayed | *"L headings came across without their own page — they group the sheets that arrived; clicking them goes nowhere."* | same line, fourth clause, dropped at zero |
| A's outline tree was **truncated** by `read_outline`'s guard rails | *"That document's bookmark list is larger than pdfcer reads, so this carried only the part it can see."* | same line — `OutlineDiagnostics::is_faithful()` (`outline.rs:~938`) is the single question, and `outline.rs:1533-1538` says a truncated tree must be *presented* as truncated |

**Where:** the **status-row edit disclosure**, via
`super::record_edit_disclosure(Some(EditDisclosure { epoch, notes }))` —
`D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\app\actions\disclosure.rs:126`, as
`crossdoc.rs:315` already does for the move. `app/status.rs:48` names this
exact lane: *"Edit disclosure | **rule 4** — what a move or a delete had to
change…"*. **Not a badge on a page, not a modal, not a Bookmarks-panel
decoration.**

**Epoch:** the **target's** (`crossdoc.rs:203-206` establishes why —
`app::status` draws only the active document's disclosure, and the target is
the document on screen).

**Order:** *what happened* before *what it cost* — the carry's own sentence
first, then the leftovers, then the source's dangling report on a move. That is
the ordering `crossdoc.rs:310-313` already states as the application-wide shape.

**One live wording defect this exposes, and it is the sharpest thing in the
survey.** On a Shift-drag today the operator sees, on **B's** status row:

> *"3 bookmarks now point at pages that are no longer in this document."*
> (`text/pages.rs:400`, pushed at `app/actions/pages.rs:621`, re-filed under
> B's epoch at `crossdoc.rs:262-268`)

**"this document" means A, and the operator is looking at B.** With the carry
shipped, B will *also* be gaining bookmarks in the same breath, and the two
sentences will read as contradicting each other. The sentence needs a
source-naming variant for the move path — *"…now point at pages that are no
longer in `<source>`."* — and that variant is required by this change, not
optional.

---

### 7. What will bite

1. **The remap in §5.1 step 2 is positional, and I did not measure that the
   engine places the pages contiguously in `moved`-sorted order at `landing`.**
   `copy_and_splice` (`edit.rs:39682`) was not read. If it splices in the order
   `source_pages` was given rather than sorted, or non-contiguously, every
   carried destination is wrong by an offset — and a wrong bookmark looks
   exactly like a right one until it is clicked. **This must be asserted by the
   driven check before the feature is called done**, and it is the argument for
   build (b) in §5.4, which uses object identity and cannot be wrong.
2. **`paste_outline_item` refuses the whole paste above `MAX_UNDO_DEPTH`/2
   items** (`edit.rs:46318-46323`, `SelectionTooLargeForOneUndo`). Dragging
   fifty pages out of a densely bookmarked manual can trip it. The decline path
   is candidate 3 — carry nothing, say why, in words that name the limit.
3. **`add_outline_item` refuses on an encrypted or certified target**
   (`edit.rs:41735-41739`), and so does `paste_outline_item` through it. But
   `insert_pages` has *already committed* by then. The pages land and the
   bookmarks do not — so the decline must be caught and reported, never allowed
   to unwind the insert.
4. **`Destination::Named` is a trap.** A carried `Named` destination would be
   refused by `add_outline_item` with `NamedDestinationNotFound`
   (`edit.rs:41720-41724`) unless the key is defined in B first. Since
   `insert_pages` does **not** carry `/Dests` (only `merge_document` does, via
   `merge_named_destinations`, `edit.rs:39140`), **every `Named` destination is
   unresolvable on this path.** §5.1 step 3 routes them to
   `destination: None` + `report.unresolvable` rather than letting the paste
   refuse — do not skip that.
5. **`OutlineDiagnostics` truncation.** A hostile or merely enormous `/Next`
   chain is capped at `MAX_OUTLINE_ITEMS`. Carrying a truncated tree silently
   is precisely the rule-4 violation `outline.rs:1533` warns about.
6. **Do not reach for `merge_outline` (`edit.rs:39270`) for a page subset.** It
   imports the outline dictionaries *verbatim* through `import_object`, which
   follows `/Dest` references. On a subset, a bookmark pointing at an
   un-dragged page would drag that page's object into B as a loose object. It
   is safe for merge only because merge takes every page.
7. **Two undo entries under build (a)** — see §5.3. If shipped that way it is a
   rule-4 disclosure of its own, not a silent behaviour.
8. **The `is_modified` guarantee for A must not change.** `crossdoc.rs:78-89`
   promises that on a copy *"nothing is written to it and nothing is read out of
   it destructively"* — the whole reason the gesture is safe to try. §5.1 reads
   A through `&DocumentView` only. Candidate 4 would break this promise on the
   move path, which is a second reason it is the operator's call.
9. **`/Count` arithmetic in B.** `add_outline_item` maintains `/Count` up the
   chain and opens a leaf parent that gains a child (`edit.rs:41700-41709`).
   Pasting at `LastChild { parent: None }` touches only the root, which is the
   least surprising place for it — but a B whose root `/Count` was already
   wrong in the file will now be wrong by a different amount. Not a regression;
   worth knowing.

---

### 8. The driven check owed

`D:\Dev\pdfcer-gui\tools\ui-verify\src\checks\` holds **213** files (measured
2026-09-13, `ls | wc -l`; it was 210 when this section was written, and the
three added since are the O186 work's own). ★ **Date a file count or it
becomes a claim about a directory that grows every week.**. The existing cross-document drag check is
`checks\page_drag_between_documents.rs`, registered at
`checks\mod.rs:845` and `checks\roster.rs:771-772` as two instances,
`PageDraggedBetweenDocuments::COPY` and `::MOVE`, whose `fn name` at
`page_drag_between_documents.rs:129-135` returns
`"a_page_dragged_between_documents_is_copied"` /
`"a_shift_drag_between_documents_moves_the_pages"`.

**New file:** `tools\ui-verify\src\checks\page_drag_carries_bookmarks.rs`
**Check names** (lowercase words + underscores, **no digits**):

- `a_page_dragged_between_documents_brings_its_bookmark`
- `a_bookmark_for_a_page_left_behind_stays_behind`
- `a_chapter_heading_whose_page_stayed_arrives_as_a_heading`

Registered as three `const` instances on one struct, following
`PageDraggedBetweenDocuments`' `COPY`/`MOVE` pattern, added to `checks\mod.rs`
and `checks\roster.rs`.

**Fixture** (`tools\ui-verify\src\fixture.rs` builds these — not measured in
detail, §9): document A with five pages and a three-level outline —
`Chapter One` (dest = page 1) → `Section A` (page 2), `Section B` (page 3),
`Section C` (page 4) — and `Appendix` (page 5). Document B with two pages and
either no outline (one variant) or one bookmark of its own (another).

**What each asserts, driven against the running binary:**

| check | gesture | assertion |
|---|---|---|
| `a_page_dragged_between_documents_brings_its_bookmark` | drag A page 3 into B between its two sheets | B's Bookmarks panel gains `Chapter One` → `Section B`; `Section B`'s destination resolves to B's **page 2** (the landing index), not page 3; the status row carries the carried-count clause |
| `a_bookmark_for_a_page_left_behind_stays_behind` | same gesture | B's panel does **not** contain `Section A`, `Section C` or `Appendix`; the status row states how many stayed behind and names A |
| `a_chapter_heading_whose_page_stayed_arrives_as_a_heading` | same gesture | `Chapter One` is present in B and **expands**, and clicking it does not navigate (its own page 1 stayed in A); the status row carries the heading-without-its-page clause |

Each also asserts **A's page count and A's bookmark count unchanged** on the
unmodified (copy) drag — the §7.8 guarantee — and, on a Shift variant, that the
status row's dangling-bookmark sentence **names A** rather than saying *"this
document"* (§6's wording defect).

The existing check's header (`page_drag_between_documents.rs:16-31`) documents
the four mechanisms no unit test can reach; the new file should cite it rather
than restate it, and add the fifth: **the outline carry reads a document that
is not on screen, through a borrow that exists only for the duration of the
drop.**

---

### 9. What I could not measure

- `EditSession::copy_and_splice` (`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:39682`)
  — **not read.** So the ordering and contiguity of inserted pages relative to
  `source_pages` and `landing` is **not measured**. §5.1 step 2 depends on it
  and §7.1 is the consequence.
- Whether `EditSession::coalesce_last` is public or crate-private — **not
  measured**; it appears only at internal call sites I read
  (`edit.rs:46283`, `:46349`).
- `OutlineDiagnostics::is_faithful()` — the type is at `outline.rs:791` and
  `impl` at `:938`, and `read_outline`'s doc at `outline.rs:1531` names the
  method, but I did **not** read the method body or confirm its exact
  signature.
- `D:\Dev\pdfcer-gui\tools\ui-verify\src\fixture.rs` — **not read.** Whether it
  can author an outlined fixture PDF, and how, is not measured; §8's fixture may
  need building.
- `text::doctabs::drag_landing_other` / `drag_landing_move`
  (`text\doctabs.rs:309`, `:345`) — line numbers measured, **bodies not read**.
  The pre-release caption may also need a bookmark clause; not established.
- Whether any `pdfcer` doc under `docs/decisions/` already rules on
  bookmark-carrying insert — I grepped `docs/core-api/index.md` (no outline
  hits) and read `pageops/outline.rs`'s header, but did **not** sweep
  `docs/decisions/`.
- `MAX_UNDO_DEPTH`'s value — **not measured**; only its use at
  `edit.rs:46318`.
- The GUI's Bookmarks panel refresh path after an external outline write — I
  read `panels/bookmarks/clip.rs:114, :192` and `app/actions/bookmarks.rs:508`
  but did **not** establish whether a paste raised from `crossdoc` (rather than
  from the panel) refreshes the panel's own cached tree.
- `pdfcer-core`'s tests for `OutlinePolicy::Subset` — **not located**; the rule
  is documented and implemented but I did not confirm it is covered.

---

### 10. What needs the operator's decision

1. **Approve the rule as stated in §4** ("The recommended rule, stated for the
   operator to approve"). This is candidate 2 — carry matching entries, keep
   ancestor headings as destination-less containers, drop the rest and count
   them. It is the rule pdfcer already applies to Extract and Split.
2. **Candidate 4 — should a Shift-drag (move) also clean up A's now-dangling
   bookmarks?** Recommendation: **no, not in this change.** It adds a third
   undo entry to a gesture whose undo story is already the weakest in the
   application, and it deletes from a document the operator is not looking at.
   If yes, the narrow form (leaf entries only, containers untouched) is the one
   to take.
3. **Where the carried bookmarks land in B** — recommendation: **appended after
   B's own top-level bookmarks**, matching merge (`text/pages.rs:986`,
   `edit.rs:39351`). The alternative — inserting them at the position
   corresponding to where the pages landed — reads better on paper and requires
   deciding what "corresponding" means when B's outline order and B's page
   order disagree, which they frequently do.
4. **Two undo entries, or wait for an engine verb?** Build (a) ships now with
   Ctrl+Z removing bookmarks then pages; build (b) is one entry but needs a
   change in `D:\Dev\pdfcer\`, which is read-only until fold-in. Recommendation:
   **ship (a), file (b)** — and disclose the two-step undo in words.
5. **The dangling-bookmark sentence must start naming the source document**
   (§6, last block). This is a wording fix the change forces; it needs sign-off
   because it alters an existing shipped sentence.

---

## O183 — the nine-part ce-dimension request, surveyed part by part

**Read-only survey. Nothing was edited, no `cargo` was run, `D:\Dev\pdfcer\` was
read and never written.** This file is the only write.

**Rule 15 governs every line below.** A **ce dimension** is one *pdfcer authors*
— a `/Line` (or `/Polygon`/`/PolyLine`) annotation with `/IT /LineDimension`, a
baked `/AP`, and a `/PieceInfo /pdfcer` sidecar carrying its measurement model.
A **pdf dimension** is CAD-exported page content pdfcer *reads* and must not
silently alter. O183 is entirely about **ce dimensions**. No fix proposed here
touches page content; anything that did would be a defect, not a feature.

**Method.** Every `file:line` below was measured during this survey by reading
the file. Where a line is quoted from a document rather than from source, the
document is named and the source is treated as authoritative over it — which
matters, because two engine documents are demonstrably stale on exactly this
subject (see §13, *What I could not measure*).

---

### 0. The one-paragraph verdict

**Six of the nine are already built.** The dominant failure in O183 is not
missing capability — it is that a click on a ce dimension opens the **comment
window** instead of leading the operator to the Properties panel where all of
this already lives. Requirement **#7 is the keystone**: it is a small addition at
one measured line, and closing it makes **#2, #4, #8a and #9 stop being
complaints** without writing a single new control. **#1** is half-true and its
real defect is narrower and different from how it was reported. **#3** is true
and is a *recorded decision*, not a gap. **#5** is a genuine scope question with
a partial answer already in the engine. **#6** is half-shipped — the live
preview exists on both create and move; the text-versus-line disclosure does
not. **#8** is half-shipped — radius-versus-diameter ships; leader location,
leader style and centre mark are **absent from the engine** and need a hand-off.

---

### 1. "Units cannot be changed after creation"

#### Verdict: **PARTLY FALSE as reported — and there is a real defect underneath it, a different one.**

**Unit *is* changeable on an existing ce-dimension group.** The control is a
combo box in the dimension-groups panel:

- `D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\panels\dimension_groups\mod.rs:609-641`
  — `ui.horizontal_wrapped` → `egui::ComboBox::from_id_salt("dimension-group-unit")`,
  populated from `Unit::all()`, raising
  `Action::Dimension(DimensionAction::SetGroupScale { group, scale, format })`
  when the selection changes.
- The group's **scale is carried through unchanged** (`mod.rs:625`, `scale: group.scale`),
  which the code comment at `mod.rs:604-608` explains deliberately: `set_group_scale`
  takes both, and passing anything but the group's own scale would recalibrate the
  drawing while the operator was only changing a unit.
- The action reaches the engine: `app\actions\dimensions.rs:876-885` →
  `session.set_group_scale(group, scale, format)`.

So the *unit* half of #1 is shipped and reachable.

#### The real defect: the **fraction mode** is reset, and is not settable on an existing group

`mod.rs:638` passes `format: unit.default_format()`. The comment above it
(`mod.rs:627-637`) argues the case: a `NumberFormat` is a unit *and* how its
fractional part is written, and carrying "eighths" across a change to
millimetres would produce a format nobody's drawing uses. **That reasoning is
sound and the consequence is still the defect Ken is reporting**: an operator
who has set inches-in-sixteenths and then changes the unit loses the sixteenths,
and **there is no fraction or precision control anywhere on an existing group**
to put it back.

Measured: the only `fraction_combo` in the shell is in the **Set-scale dialog** —
`dialogs\scale.rs:576` (`fn fraction_combo`), fed by `const FRACTIONS` at
`dialogs\scale.rs:549` (Decimal 0/1/2/3, Fraction 8/16/32), committed at
`dialogs\scale.rs:470` (`fn commit`) → `DimensionAction::SetGroupScale`. The
dimension-groups panel itself has **no fraction row** — its rows are unit, scale,
drafting standard (`panels\dimension_groups\mod.rs:645-666`), rename/delete
(`panels\dimension_groups\identity.rs`), and draw-into.

The engine has no objection: `set_group_scale` takes a whole `NumberFormat`
(`D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\units.rs:178`,
`NumberFormat { unit, fraction, decimal_marker }`), so a fraction picker on an
existing group is purely a shell addition.

The engine's own feature table corroborates the shape:
`D:\Dev\pdfcer\docs\FEATURES.md:241` — *"The **unit** is writable only as a side
effect of `set_group_scale`, which is the sanctioned route rather than a hole."*

#### Where it lives, and what class it is

**(c) already shipped and merely undiscoverable**, for the unit half — the combo
is in a panel the operator may not have opened.
**(a) present in engine but unexposed in shell**, for the fraction half — the
engine takes the whole `NumberFormat`; the shell only offers the fraction inside
the Set-scale dialog.

#### Fix, in shell only

1. Add a fraction/precision row to `panels\dimension_groups\mod.rs` beside the
   unit combo at 609-641, reusing `dialogs\scale.rs`'s `FRACTIONS` list rather
   than inventing a second one.
2. When the unit combo changes, keep the operator's fraction mode **if it is
   coherent with the new unit**, and otherwise fall to `default_format()` **and
   say so in the panel** — rule 4: the substitution must be visible.
3. Per-ce-dimension precision already exists and overrides this
   (`panels\properties\dimension\overrides.rs:132-139`), so the group row is the
   default, not the only control.

**Proposed driven checks:** `an_existing_group_offers_a_fraction_control`,
`changing_a_group_unit_discloses_a_fraction_it_could_not_keep`.

---

### 2. "The per-dimension override does nothing"

#### Verdict: **FALSE as a capability claim. The control exists, is complete, and reaches the engine. It is being masked — see #7.**

**All eleven cascade properties are drawn** on the per-ce-dimension surface:

`D:\Dev\pdfcer-gui\crates\pdfcer-gui\src\panels\properties\dimension\overrides.rs`
— `pub fn show(ui, group, overrides) -> bool` at `:107`, and the rows:

| property | line |
|---|---|
| unit | `overrides.rs:113-129` (`ComboBox::from_id_salt("dimension-override-unit")`, `Unit::all()`) |
| fraction / precision | `overrides.rs:132-139` |
| decimal marker | `overrides.rs:143-157` |
| drafting standard | `overrides.rs:161-175` |
| text height | `overrides.rs:179-191` |
| line width | `overrides.rs:196-206` |
| arrow length | `overrides.rs:212-222` |
| arrow form | `overrides.rs:228-247` |
| colour | `overrides.rs:250-262` |
| **tolerance** | `overrides.rs:263-274` |
| **tolerance precision** | `overrides.rs:276-299` |

A test asserts the set is complete — `const DRAWN: [&str; 11]` at
`overrides.rs:448-460` and `fn no_property_of_the_cascade_is_left_without_a_row`
at `overrides.rs:480`, checked against the engine's
`StyleProvenance::each() -> [(&'static str, StyleSource); 11]`
(`D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\style.rs:423`).

**The action path is whole, measured end to end:**

- `panels\properties\dimension\mod.rs` — reads `record.style`, calls
  `overrides::show`, and on a validated change raises
  `Action::Dimension(DimensionAction::SetStyle { dimension: record.id, style: next })`.
- `app\actions\dimensions.rs:1085-1091` —
  `DimensionAction::SetStyle { dimension, style } => … session.set_dimension_style(dimension, style)`,
  wrapped in `super::apply::vector_edit(doc, "set-dimension-style", 0, 1, …)`.
- Engine verb `set_dimension_style` — `edit.rs:16115` per
  `D:\Dev\pdfcer\docs\core-api\02-editing-and-saving.md` §1.22 (line number from
  the doc, not measured against `edit.rs`).

**The panel is actually drawn**, not dead code: `panels\properties\mod.rs:452`
— `let drew_dimension = dimension::section(ui, doc, actions);`, and its return
feeds `something_drew` at `panels\properties\mod.rs:524`.

**Engine's own record agrees this shipped:** `D:\Dev\pdfcer\docs\FEATURES.md:246`
— *"**Reachable in `pdfcer-gui`** — the Properties panel shows all eleven
bottom-tier properties with an override checkbox each and a sentence naming
which tier supplied the value in force."*

#### So why does Ken see nothing?

Two candidates, and both are addressed by other items in this survey rather than
by building anything for #2:

1. **#7 masks it.** Clicking the ce dimension pops the comment window. The click
   *does* also select (see §7 — the popup consumes nothing), so the Properties
   section *is* populated on the same click; the operator's attention is taken by
   the window that appeared on top of the canvas. The panel he needs is the one
   he was not looking at.
2. **Provenance can read as "nothing happened."** `StyleSource::follows_group()`
   returns `true` for **both** `Factory` and `Group`
   (`D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\style.rs:347` ff, and the
   shell's own warning at `overrides.rs:35-38`: *"`StyleSource::follows_group()`
   is `true` for `Factory`, which is the easy thing to get wrong"*). A row whose
   checkbox is ticked but whose resolved value happens to equal the inherited one
   draws identically. **Not a defect found — a hypothesis I could not rule out**
   without driving the binary, which this survey is forbidden from doing.

**One structural note, measured, that reads as a bug and is not one:**
`overrides.rs:39-47` — `unit`, `fraction`, `decimal_marker` and `standard` have
**concrete fields** on `Group` rather than `Option`s
(`D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\group.rs:50`, fields `format:59`,
`standard:83`, `style:95`), so their provenance can only ever be `Group` or
`Dimension`, never `Factory`. The panel never prints a factory sentence for those
four. That is deliberate — the engine's words, quoted in the shell header:
saying "factory" for them *"would be a lie an operator could act on."*

#### Class

**(c) already shipped and merely undiscoverable.** A discoverability fix is
still owed, and the cheapest one is #7.

**Proposed driven check:**
`a_unit_override_on_one_ce_dimension_changes_what_is_drawn`.

---

### 3. "Tolerance is missing from the group entirely"

#### Verdict: **TRUE today — and it is a written decision, not an omission. The engine supports it.**

**Engine has it.** `D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\style.rs:235`
`pub struct GroupStyle` carries seven `Option` fields, of which two are
`tolerance` (`:251`) and `tolerance_places` (`:254`).

**Shell deliberately does not draw them.** `panels\dimension_groups\style.rs`
draws only the other five, and carries an explicit section titled
**"★ Why tolerance is not drawn here"** at `panels\dimension_groups\style.rs:108-122`.
Its argument, verbatim:

> `GroupStyle` carries `tolerance` and `tolerance_places`, so a group *can*
> default them — and a group-level tolerance is the rarer half of the feature.
> A tolerance is a statement about **one manufactured feature**: two holes on the
> same drawing routinely carry different ones even though they share the
> drawing's units and precision, which is the ui-spec's own reasoning (§C.11.1)
> and the reference tool's. So tolerance belongs on the **per-ce-dimension**
> surface, where it is drawn, and putting a second control for it here would
> invite an operator to set a group default that almost every member then
> overrides — the shape that makes the moving-count read *"no change on screen"*
> on nearly every press. It is reachable from the CLI for the drawing that
> genuinely wants one, and this paragraph is the record of that being a decision
> rather than an omission.

The moving-count it refers to is real: `panels\dimension_groups\style.rs:320`
`fn will_move` computes how many members a group-tier change visibly moves, and
the panel shows it per press (rows built by `fn property_row` at `:272`,
`pub fn show` at `:123`).

#### Class

**(a) present in engine, deliberately unexposed in shell.** This is an
**operator's-decision item**, not a bug and not a hand-off. If Ken overrules the
decision, the work is small — a tolerance row and a tolerance-precision row in
`panels\dimension_groups\style.rs`, using the same
`panels\properties\dimension\tolerance.rs` editor (`pub fn show` at `:104`), and
routed through `DimensionAction::SetGroupStyle`
(`app\actions\dimensions.rs:1068-1072` → `session.set_group_style`).

**Note the counter-argument is answerable.** SolidWorks *does* carry a
document-level tolerance default, and Ken's standing position is that SolidWorks
is the floor, not the ceiling. The shell's argument is not that a group default
is wrong, but that it reads as a no-op press. That is a **disclosure** problem,
and the panel already solves disclosure problems the same way everywhere else:
show the count of members that will change, including when it is zero, and say
*why* it is zero ("every member overrides this").

**Proposed driven check:** `a_group_tolerance_default_reports_how_many_members_take_it`.

---

### 4. "Tolerance does not work on the override either"

#### Verdict: **FALSE as a capability claim. It is shipped, validated, and reaches the engine. Same mask as #2.**

- `panels\properties\dimension\overrides.rs:263-274` — the tolerance row,
  `valid &= super::tolerance::show(ui, value, unit)`.
- `panels\properties\dimension\overrides.rs:276-299` — the tolerance-precision row.
- `panels\properties\dimension\tolerance.rs:104` — `pub fn show(ui, value, unit) -> bool`;
  `const FORMS: [Tolerance; 7]` at `tolerance.rs:77`; drag speed
  `SPEED = 0.005` at `tolerance.rs:70`; `fn same_form` `:183`; `fn reshape` `:211`.
  It renders **fields, not a specimen** — the label is never previewed by
  concatenation, because a **limit tolerance suppresses the nominal** and only
  `author_dimension` knows the exact baked string.
- **It raises nothing until it validates.** `overrides::show` returns `valid`, and
  `panels\properties\dimension\mod.rs` only pushes `SetStyle` when
  `valid && next != record.style`. An inverted limit pair is refused with the
  engine's own sentence — `text\panels\dimension.rs:438`
  `pub fn tolerance_refused(reason: &str)`, with the unit note at
  `text\panels\dimension.rs:423` `pub fn tolerance_unit_note(unit: Unit)`
  (tolerance values are in the **displayed unit**, not points —
  `D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\tolerance.rs:1-55`).
- Engine record: `D:\Dev\pdfcer\docs\FEATURES.md:247` — *"**Reachable in
  `pdfcer-gui`** — tolerance and tolerance decimals are two of the eleven
  overridable properties in the same Properties-panel editor as the style row
  above; an inverted limit pair is refused with the engine's own sentence, shown
  verbatim, and no action is raised until it validates."*

**Remember `Some(Tolerance::None)` ≠ `None`** — the first is an override saying
*this ce dimension carries no tolerance*, the second is *inherit*. The panel's
checkbox is the difference, and both are legitimate states.

#### Class

**(c) already shipped and merely undiscoverable.** Closing #7 closes this.

**Proposed driven check:** `a_symmetric_tolerance_typed_on_one_ce_dimension_is_drawn`.

---

### 5. "Tolerance control is thinner than SolidWorks'" — ★ SCOPE QUESTION

#### ★ Flagged explicitly: do not promise anything here until SolidWorks is measured.

**What exists.** `D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\tolerance.rs:1-55`
implements **7 of `swTolType_e`'s 13** forms: none, basic (boxed), bilateral /
deviation, limit, symmetric, min, max.

**What is omitted, and the engine's stated reason for each** (from the same header):

| SolidWorks form | Status | Reason recorded in `tolerance.rs:1-55` |
|---|---|---|
| `swTolFIT`, `swTolFITWITHTOL`, `swTolFITTOLONLY` | **not implemented** | ISO 286 fit classes. The reference RAG's class list is flagged `UNVERIFIED` (`D:\Dev\Rag-Specialized\SolidWorks_Dimensions\`, §A.3). *"A wrong `H7/g6` deviation is a manufacturing defect."* |
| `swTolBLOCK`, `swTolGeneral` | **not implemented** | Need a general-tolerance table pdfcer does not have. |
| `swTolMETRIC` | **not implemented** | Shares enum value 7 with `swTolFIT`; cannot be distinguished. |

Corroborated at `D:\Dev\pdfcer\docs\FEATURES.md:499` —
*"ce-dimension tolerance, the ISO 286 fit forms — fit, fit-with-tolerance,
fit-tolerance-only, plus block and general tolerance. Every other tolerance form
is built; these six need a sourced class/table lookup pdfcer does not have."*

#### Ken's specific asks in #5, item by item

| Ken's word | Measured answer |
|---|---|
| bilateral | **shipped** (deviation form) |
| limit | **shipped** — and note it **suppresses the nominal**; never preview a limit label by concatenation |
| symmetric | **shipped** |
| min / max | **shipped** |
| fit | **NOT built.** Blocked on an unverified ISO 286 table. This is the scope question. |
| **precision per side** | **NOT built, and it is a data-model change, not a control.** There is exactly **one** `tolerance_places` — `style.rs:254` on `GroupStyle`, `style.rs:309` on `StyleOverrides`. SolidWorks carries an upper and a lower precision independently. |

#### What must be measured in SolidWorks before any promise

1. Does SolidWorks in practice let the *upper* and *lower* tolerance carry
   different decimal places on the same dimension, or is that only true across
   the two fields of a *limit* display? (Determines whether "precision per side"
   is one new field or a second cascade property with all that implies —
   `StyleOverrides` is currently eleven fields and a test counts them.)
2. Which fit-class table does the shop actually use — ISO 286-2, ANSI B4.1, or a
   house table? A **sourced** table turns three tolerance forms from "blocked" to
   "a day of typing".
3. Is block/general tolerance a per-dimension property in his workflow, or a
   title-block note?

**Until those three are answered this item has no estimate.** It is the only
item in O183 with no tranche row, and the engine's refusal to guess is the right
refusal.

---

### 6. "No live preview when creating or moving, and no way to tell TEXT from LINE"

#### Verdict: **FIRST HALF IS FALSE — the live preview ships, on create AND on move. SECOND HALF IS TRUE and is the real request.**

#### The preview exists, and is drawn by the same function that draws the committed article

The rule the module holds to, stated at `canvas\dimdrag.rs:1117` — *"The
segments come from `measure::pick::dimension_preview_segments` — **the same
function a committed dimension is drawn from**"*.

**On create:**
- `canvas\measure\pick.rs:249` — `pub fn placing_preview(&self, p: Point) -> Option<DimensionKind>`.
- `canvas\measure\pick.rs:589` — `pub fn dimension_preview_segments(kind: &DimensionKind) -> Vec<(Point, Point)>`.
- Wired at `canvas\measure\mod.rs:1180-1182` (scale/line tool), `:1197-1199`
  (linear), `:1215`, `:1233`.
- `canvas\measure\mod.rs:930-1010` — the `Preview` struct and `pub(super) fn preview`.
- Test: `canvas\measure\pick.rs:778`
  `the_placing_preview_is_exactly_what_the_placing_click_authors`.

**On move / drag:**
- `canvas\dimdrag.rs:478` — `Some(super::measure::pick::dimension_preview_segments(&moved))`,
  returned every frame that is not the committing frame. The comment at
  `canvas\dimdrag.rs:471-476` explains the one-frame gap: *"Nothing is previewed
  on the frame that commits: the annotation is about to be regenerated and drawn
  for real, and a preview left on screen over it would be a second copy of the
  same line, one frame stale."*
- On a vertex drag: `canvas\dimdrag.rs:866-876`, same function, plus a snap marker
  (`canvas\dimdrag.rs:601` `pub fn drag_vertex`).
- `canvas\dimdrag.rs:1135` `pub fn annot_shapes` — the shapes for a *selected*
  ce dimension, same segment source.

**Painted:** `canvas\painting.rs:117`
`pub dimension_preview: Option<&'a [(Point, Point)]>`, fed at
`canvas\interact.rs:1295` (`dimension_preview: pv.dimension.as_deref()`), stored
at `canvas\previews.rs:91` and `canvas\dragroute.rs:86`.

**Correction to an existing record:** `D:\Dev\pdfcer-gui\OPERATOR_REQUESTS.md:9503`
says *"`painting.rs:437-450` draws page-space segments for the ce-dimension
placement preview"*. **That line reference is stale** — measured today the field
is at `canvas\painting.rs:117`.

#### The second half is real: nothing says whether you are dragging the TEXT or the LINE

- **There is exactly one grab box, and it is the whole `/Rect`.**
  `canvas\dimdrag.rs:318` — `pub fn grab_box(...)` returns
  `map.rect_to_screen(annot.outline)`. One box, covering label and lines together.
- **One drag moves both at once.** `canvas\dimdrag.rs:346` `pub fn placed`, and
  the commit at `canvas\dimdrag.rs:466-470` pushes
  `DimensionAction::Place { dimension, offset, text_along }` — `offset` is the
  standoff perpendicular to the measured axis, `text_along` is where the label
  sits along the dimension line. **Both are set from one gesture.**
  (This is `place_dimension`, the value-preserving verb — distinct from
  `move_dimension`, which translates the measured points.)
- **The label is deliberately excluded from the selection shapes.**
  `canvas\dimdrag.rs:1130`, the doc comment on `annot_shapes`:
  *"The label is deliberately NOT included … a dimension is selected by its
  lines … If that proves too strict in use, the fix is to ask the engine for the
  label's box rather than to guess one here."* **That sentence is the hand-off
  this requirement needs, already written by the module's author.**
- **No distinct cursor exists.** `canvas\cursor.rs:634-642` gives custom shapes
  only to Crosshair and Text/Ibeam. `canvas\tool\arm.rs:67-167` maps `DragKind`
  → `CursorIcon`: `Move`→`Grabbing` (`:109`), `Rotate`→`Grabbing` (`:117`),
  `MarkupVertex`→`Grabbing` (`:132`), `TextSelect`→`Text` (`:167`). **No
  ce-dimension-label case.**
- The module header also records that the operator asked for this before, and
  that the label drag itself *"failed twice"* before being fixed 2026-08-20
  (`canvas\dimdrag.rs` header, D1 live-preview row ~`:107`) — so this surface has
  a history of being harder than it looks.

#### Class, and the one engine question

**(a)/(b) split.**

- **Shell-only, and cheap:** two cursors and/or two hover highlights — the
  dimension line lit one way, the label anchor another. This is exactly the
  pre-commit affordance Rule 4 welcomes; it changes no document state.
- **Engine-dependent, and the module says so:** the shell **does not know the
  label's box**. `annot.outline` is the whole `/Rect`. To *highlight* the label
  region the shell must ask the engine for its rectangle, or guess — and
  `dimdrag.rs:1130` forbids guessing.

**The intermediate that needs nothing from the engine: two grips rather than two
highlights.** Drag the dimension line to set `offset`; drag a small handle at the
label anchor to set `text_along`. The anchor position is derivable from
`text_along` and the measured axis, both of which the shell already holds. Per
R9/R83, if the label handle cannot be honoured for a kind (a circular ce
dimension has no axis to slide along — `place_dimension` refuses it by name), the
handle is **not drawn**, never greyed.

**Proposed driven checks:**
`hovering_a_ce_dimension_line_says_it_is_the_line`,
`hovering_a_ce_dimension_label_says_it_is_the_label`,
`the_label_handle_moves_the_label_and_not_the_line`.

---

### 7. "Clicking a ce dimension pops the comment box" — ★ THE KEYSTONE

#### Verdict: **TRUE, root cause measured, and it is a small addition at one line — not a feature.**

#### The chain, measured end to end

1. **Every ce dimension is authored with a non-empty `/Contents`.**
   `D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\author.rs:727-730`:
   ```rust
   annot.insert(
       Name::from(b"Contents"),
       Object::String(label.as_bytes().to_vec()),
   );
   ```
   The `label` is the printed measurement. It is never empty. The dict shape is
   documented at `author.rs:255` — `/Type /Annot /Subtype /Line /IT /LineDimension
   /Rect /L /C /Contents` — and `author.rs:240` lists `Contents` among the keys
   the authoring path writes.

2. **The popup's "has this got anything to say" gate therefore always passes.**
   `canvas\notepopup\model.rs:457-467`:
   ```rust
   pub fn has_something_to_read(note: &NoteView) -> bool {
       if matches!(note.subtype.as_str(), "Text" | "FreeText") { return true; }
       note.contents.as_deref().is_some_and(|c| !c.trim().is_empty())
   }
   ```
   A ce dimension is `/Line`, so it falls to the second clause — and its
   `/Contents` is the label, so the clause is `true`. **Always.**

3. **`clicked_on` toggles the window on that gate and nothing else.**
   `canvas\notepopup\mod.rs:982-1018` —
   `pub fn clicked_on(ctx, doc, page_index, point, map) -> Option<ObjId>`,
   with the gate at **`canvas\notepopup\mod.rs:1008`**:
   ```rust
   if !model::has_something_to_read(note) {
       return None;
   }
   ```
   and the toggle immediately after at `:1014-1017` (*"★ TOGGLE, not open"*).

4. **The popup sits above the click ladder and consumes nothing.**
   `canvas\clicking.rs:304` is the single call site; the comment at
   `canvas\clicking.rs:282-303` states it *"sits above the ladder and consumes
   nothing"*. **This is the good news:** selection still happens on the same
   click, so the Properties panel **is** populated. The popup merely competes for
   attention. Nothing has to be re-plumbed.
   (`clicking.rs:276-281` `annot_hit` is separately gated on `caps.author_markup`;
   `clicked_on`'s own header at `:963-972` explains it deliberately repeats the
   hit test so Read mode still opens notes.)

5. **The popup already knows about ce dimensions** and was given bespoke wording
   for them — `canvas\notepopup\mod.rs:440` (`is_ce_dimension: bool` on the frame
   context, *"Decided once per frame in `show` … never per pop-up"*),
   populated at `canvas\notepopup\mod.rs:382` from
   `crate::panels::comments::model::ce_dimension_annots(&doc.session)`
   (`panels\comments\model.rs:294`, which walks `dimension_model().dimensions()`
   and collects each `d.annot`), with ce-dimension heading and caption at
   `canvas\notepopup\mod.rs:713-714` and `:778-790`.
   The module header at `canvas\notepopup\model.rs:86-97` is explicit:
   *"★★ Rule 15: a ce dimension is a `/Line` and it is NOT excluded here."*

6. **History:** `canvas\notepopup\mod.rs:203-216` records that the *empty*-popup
   case was closed 2026-09-05 (O133) by adding exactly the gate at `:1008`. The
   ce-dimension case was seen and left in on purpose. O183 is the operator
   overruling that.

#### The fix — small, surgical, shell-only

`clicked_on` already takes `doc`, so it can reach the sidecar set the same way
`show` does. **Change at `canvas\notepopup\mod.rs:1008`**:

- compute `panels::comments::model::ce_dimension_annots(&doc.session)` once per
  *click* (not per frame — `clicked_on` runs on press only, and its own cost note
  at `:976-981` already budgets one `/Annots` walk per click);
- if the note is a ce dimension **and** the preference says "do not pop," return
  `None`, so the press falls through to selection alone.

**Cost caveat, measured:** `ce_dimension_annots` deserializes the `/PieceInfo`
sidecar; `panels\comments\model.rs:285-293` documents it as *"bounded by the
number of ce dimensions … Called **once** per frame by `panels::comments::body`,
never per row."* One extra deserialization **per click** is acceptable; per frame
from here would not be. A cheaper test would read the `/IT /LineDimension` marker
off the `NoteView` directly — **but I did not measure whether `NoteView` carries
`/IT`** (see §13).

#### The preference Ken asked for ("a setting to suppress it and edit from Properties")

**No such setting exists today.** Measured:

- `dialogs\settings\measuring.rs` exposes exactly one function —
  `pub fn parallel(ui, draft)` at `dialogs\settings\measuring.rs:83`. Its header
  (`:1-32`) says the group *"is a group rather than a move into Pages and
  printing because dimensioning is a growing subject with an obvious next tenant
  … Those belong beside this, and a group that exists now is one they can arrive
  into rather than a second reorganisation later."* **That is the home for this
  preference, by the module's own invitation.** The same header also applies
  Rule 15 to the operator-facing copy, saying *"new dimensions you draw"* rather
  than the bare word — the new setting's wording must do the same.
- `app\prefs\` contains `cache, chrome, file, fonts, mod, offpage, opening,
  pastechords, printing, quality, tests, wheel` — **no popup key**.
- `dialogs\settings\` contains `acrobat, appearance, colour, comments, defaultapp,
  display, fonts, images, measuring, mod, pages, preset, saving, signatures, text,
  widgets`. It could live under `comments`, but `measuring` is the symptom-driven
  home — *"my dimension opens a comment box"* is a dimensioning symptom.

#### Which way should the default go?

**Recommend: default OFF (do not pop on a ce dimension).** The popup's caption for
a ce dimension is descriptive text about a measurement; the operator's actual
editing controls are in Properties, which the same click already populates. The
setting then exists for whoever wants the old behaviour back — which is the
direction that does not need a second complaint to discover.

#### Class

**(c) already shipped and merely undiscoverable — plus a small deletion.** The
highest-return, lowest-cost item in O183, and the one that makes #2, #4, #8a and
##9 stop being complaints.

**Proposed driven checks:**
`clicking_a_ce_dimension_does_not_open_the_comment_window`,
`clicking_a_cloud_with_a_comment_still_opens_it`,
`clicking_a_ce_dimension_selects_it_and_fills_the_properties_panel`,
`the_comment_window_on_a_ce_dimension_can_be_turned_back_on`.

---

### 8. "Radius ce dimensions have no controls at all"

Ken named four things. **They split two and two.**

#### 8a. Radius versus diameter — **SHIPPED**

- `panels\properties\dimension\mod.rs` — `display_toggle`, a radio pair, raising
  `DimensionAction::SetDisplay { dimension, show_diameter }`. Region constant
  `REGION_DISPLAY = "properties.dimension.display"`. The section's kind gate is
  at `panels\properties\dimension\mod.rs:114`; the measured readout comes from
  `model.display(record.id)`, the one producer, and `NO_SCALE_DISCLOSURE` is
  carried verbatim.
- `app\actions\dimensions.rs:1092-1101` →
  `session.set_dimension_display(dimension, show_diameter)`.
- `D:\Dev\pdfcer\docs\FEATURES.md:242` — *"Switch a placed circular dimension
  between radius and diameter"*, `core [x] cli [x] gui [x]`.

**Class: (c) undiscoverable.** Same mask as #2/#4 — it is in Properties, and the
click opened a comment box.

#### 8b. Leader location, leader style, centre mark — **ABSENT FROM THE ENGINE**

Measured in `D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\`:

- `author.rs:920` — `fn draw_circular` authors: a fitted circle outline (four
  kappa cubics), **one** radius leader from centre to rim, and **one** arrowhead
  at the rim. There is no parameter for which side the leader leaves on, no enum
  for leader style, and no centre mark drawn at all.
- `author.rs:761-763` — `fn leader_endpoints`, the only leader geometry in the
  module. (`author.rs:965` `fn draw_angular` is its angular sibling.)
- A grep of the whole `dimension/` directory for
  `center_mark|centre_mark|CenterMark|Leader` returns **no type, no field, no
  variant**. There is nothing to expose.
- `D:\Dev\pdfcer-gui\ENGINE_BACKLOG.md` (729 lines) has **no row** for centre
  marks or leader style — grep for `centre mark|center mark|leader` finds only
  unrelated rows. **This is a new hand-off, not an existing one.**

**Class: (b) absent from the engine.** Hand-off drafted in §11 (A). **Not filed.**

**Proposed driven checks** (for after the engine ships it):
`a_radius_ce_dimension_offers_a_leader_side`,
`a_radius_ce_dimension_offers_a_centre_mark`.

---

### 9. "Arrow display is not controllable"

#### Verdict: **FALSE. It is controllable on BOTH tiers.**

- **Engine:** `D:\Dev\pdfcer\crates\pdfcer-core\src\dimension\style.rs:108` —
  `pub enum ArrowForm { Filled, Open, Slash, Dot, None }`, which the file header
  maps to `swCLOSED` / `swOPEN` / `swSLASH` / `swDOT` / `swNO_ARROWHEAD`. Arrow
  **length** is a separate property: `GroupStyle::arrow_length` (`style.rs:241`),
  `StyleOverrides::arrow_form` (`style.rs:296`) and `::arrow_length` (`style.rs:294`).
- **Per-ce-dimension:** `panels\properties\dimension\overrides.rs:228-247`
  (arrow form, `ComboBox`, named via
  `crate::text::dimension_groups::arrow_form_name`) and `:212-222` (arrow length).
- **Per-group:** `panels\dimension_groups\style.rs:193-215` (arrow form) — one of
  the seven `GroupStyle` properties the panel draws, with the members-will-move
  count from `fn will_move` at `:320`.
- Engine record: `D:\Dev\pdfcer\docs\FEATURES.md:246` lists arrow form and arrow
  length among the nine style properties and marks the row
  **"Reachable in `pdfcer-gui`"**.

#### Class

**(c) already shipped and merely undiscoverable.** Closing #7 closes this.

**If "arrow display" meant something narrower** — first-arrowhead versus
second-arrowhead independently, or arrows-inside versus arrows-outside, both of
which SolidWorks carries — **that is not built at any level.** Nothing in
`style.rs` or `author.rs` distinguishes the two ends. That would be a third
engine hand-off, not a shell row. See §12 item 4.

**Proposed driven check:** `changing_the_arrow_form_on_a_group_redraws_its_members`.

---

### 10. Tranches, ordered by operator-visible return per unit of work

| # | Tranche | Contents | Size | Why it is here |
|---|---|---|---|---|
| **T1** | **Stop the comment box** | **#7** — the gate at `notepopup\mod.rs:1008`, plus the preference in `dialogs\settings\measuring.rs`, plus one key in `app\prefs\`. | **Half a day** | The single highest return in O183. It is the change that makes **#2, #4, #8a and #9 stop being complaints**, because it stops hiding the panel they all live in. Four of nine items, closed by one gate. |
| **T2** | **Say where the controls are** | Pure discoverability. On selecting a ce dimension, make the Properties section visibly *the* place: expand it by default, and if the panel is closed surface a one-line affordance pointing at it. No new control, no engine call. | **Half a day** | T1 removes the distraction; T2 supplies the direction. Without it, an operator who has just lost his familiar popup has been given nothing in exchange. **T1 and T2 ship together** — T1 alone is a subtraction. |
| **T3** | **Fractions on an existing group** | **#1's real defect** — a fraction/precision row in `panels\dimension_groups\mod.rs` beside the unit combo at `:609-641`, reusing `dialogs\scale.rs:549` `FRACTIONS`; and disclose a fraction mode that could not survive a unit change. | **One day** | Genuinely missing, cheap, and it is the half of #1 that is actually true. Shell only. |
| **T4** | **Two grips, not one** | **#6's second half, the free part** — a distinct grip and cursor for the label anchor versus the dimension line, so `text_along` and `offset` are separately draggable and the cursor says which. No engine dependency. | **Two to three days** | The disclosure Rule 4 wants, built from geometry the shell already holds. Does not need hand-off B. |
| **T5** | **Group-tier tolerance** | **#3**, *if the operator overrules the recorded decision* — two rows in `panels\dimension_groups\style.rs` through the existing `tolerance.rs` editor and `SetGroupStyle`, with the members-will-move count saying when it is a no-op and why. | **One day** | Small because both the engine field and the editor already exist. Gated on a decision, not on work. |
| **T6** | **Label box from the engine** | **#6's engine half** — hand-off B: a read verb returning the ce dimension's label rectangle, so the label can be *highlighted* rather than merely *gripped*. | **A week, mostly waiting on the engine** | Only worth doing if T4 proves insufficient in use. `dimdrag.rs:1130` names this as the sanctioned fix and forbids the alternative. |
| **T7** | **Radius leaders and centre marks** | **#8b** — hand-off A: leader side/location, a leader-style enum matching SolidWorks' glyphs, and a centre-mark model, in the engine first, then a shell section. | **A week minimum in the engine, then two to three days in the shell** | Genuinely new capability, no engine substrate at all. The largest single piece of real work in O183 and the only one that is class (b). |
| **—** | **#5 has no tranche** | Blocked on a SolidWorks measurement. | **No estimate** | Promising anything here before the measurement would be promising an unverified ISO 286 table into a manufacturing drawing. |

**If only one thing ships: T1 + T2, together, in one day.** They close or unmask
five of the nine.

**Which are a day and which are a week, stated plainly:**
T1, T2, T3 and T5 are **day-scale**. T4 is **a few days**. T6 and T7 are
**week-scale**, and both are week-scale *because of the engine*, not the shell.

---

### 11. What needs an engine change — **DRAFTED, NOT FILED**

> Written in the shape `ENGINE_BACKLOG.md` uses. **Not added to it.** Filing is
> the operator's call.

#### Hand-off A — radius/diameter ce dimensions: leader control and centre marks

**Wanted.** Three properties `draw_circular` has no substrate for:

1. **Leader location / side.** `author.rs:920` `fn draw_circular` draws exactly
   one leader, centre → rim, on a side the caller cannot choose. SolidWorks lets
   the leader leave on any side, and lets the text sit inside the circle, outside
   it, or beyond a broken/shouldered leader. **Needs:** a parameter (an angle, or
   a side/placement enum) on the circular kind, persisted in the `/PieceInfo`
   sidecar, honoured by `draw_circular`, and readable back.
2. **Leader style.** There is no enum. SolidWorks carries a set of leader glyphs
   (straight, bent/jogged, underlined/shouldered). **Needs:** a `LeaderForm`
   enum alongside the existing `ArrowForm` (`style.rs:108`), most naturally as a
   **twelfth cascade property** so it inherits factory → group → ce dimension
   like every other one. That means extending `StyleOverrides` (`style.rs:280`,
   currently eleven fields, `count()` at `:325`), `GroupStyle` (`style.rs:235`),
   and `StyleProvenance::each()` (`style.rs:423`, currently `[…; 11]`).
3. **Centre mark.** Nothing in `dimension/` mentions one. **Needs:** a
   `CentreMark { shown, form, size }` model and drawing code in `draw_circular`.
   SolidWorks' forms are cross, cross-with-centrelines, and dashed centrelines;
   size is a document property there, which argues for the group tier.

**Why the shell cannot do it.** All three are *drawn geometry inside the baked
`/AP`*. The shell never authors a ce dimension's appearance — the engine does,
and `author_dimension` returning the exact baked article is the invariant that
keeps preview and commit identical. A shell-side centre mark would be a fourth
thing on screen that the document does not contain.

**Cascade note.** If leader style becomes the twelfth property, the shell change
is mechanical: `overrides.rs`'s test
`no_property_of_the_cascade_is_left_without_a_row` (`overrides.rs:480`, against
`DRAWN: [&str; 11]` at `:448-460`) will fail until the row is added — exactly the
right failure.

**Rule 15 check.** All of this authors **ce dimensions**. Nothing here reads,
rewrites or re-bakes a **pdf dimension** from a CAD export.

#### Hand-off B — read a ce dimension's LABEL rectangle

**Wanted.** A read verb returning the rectangle the **label** occupies, in page
space, for a given ce dimension id — distinct from the annotation `/Rect`.

**Why.** `canvas\dimdrag.rs:318` `grab_box` returns the whole `/Rect`, which
covers the label *and* the lines. To hover-highlight the label separately, or to
hit-test it, the shell needs its box. The module's own comment at
`canvas\dimdrag.rs:1130` states the sanctioned fix in the engine's favour:
*"If that proves too strict in use, the fix is to ask the engine for the label's
box rather than to guess one here."*

**Shape.** Read-only, no document mutation, alongside `dimension_rects`. It must
be derived from the same placement the `/AP` was baked from, so preview and
reality cannot disagree.

**Priority.** Lower than A. T4 (two grips) delivers most of #6's value without it.

---

### 12. What needs the operator's decision

1. **#3 — overrule the recorded no-group-tolerance decision, or leave it?**
   `panels\dimension_groups\style.rs:108-122` argues a group default would read
   as a no-op press on nearly every drawing. SolidWorks *does* carry a
   document-level default, and SolidWorks is the floor. **If parity is wanted,
   T5 is a day.** If the real objection is the no-op disclosure, the panel's
   existing members-will-move count already answers it. **This is the only
   "missing" item in O183 that is a decision rather than work.**

2. **#5 — the SolidWorks measurement, three questions.** (a) Is precision
   genuinely per-side, or per-field-of-a-limit? (b) Which fit-class table does
   the shop use — ISO 286-2, ANSI B4.1, or a house table? A **sourced** table
   converts three tolerance forms from blocked to routine. (c) Is block/general
   tolerance a per-dimension property in his workflow, or a title-block note?
   **No estimate exists until these are answered.**

3. **#7 — which way does the default go?** Recommended: a ce-dimension click does
   **not** open the comment window by default, with a setting to restore it. The
   alternative (default on, setting to suppress) means the next operator meets
   the same complaint.

4. **#9 — did "arrow display" mean more than form and length?** Form
   (`Filled/Open/Slash/Dot/None`) and length are controllable on both tiers today.
   **Independent first/second arrowhead**, and **arrows-inside vs
   arrows-outside**, are SolidWorks features pdfcer does **not** have at any
   level. If those are what he meant, they are a third engine hand-off.

5. **#2, #4, #8a, #9 — is "undiscoverable" an acceptable close?** These are
   shipped. The survey's position is that a discoverability fix is still owed
   (T1+T2) and that closing them as "already works" without it would be filing a
   record over a real complaint.

---

### 13. What I could not measure

1. **Whether the eleven override rows actually change the drawn article on Ken's
   file.** This survey may not run the binary or `cargo`. The source path is whole
   from control → action → engine verb, and the engine's own FEATURES rows say
   "Reachable in `pdfcer-gui`" — but **"reaches the verb" is not "the operator
   sees the number change."** The `follows_group()` `Factory`-vs-`Group` trap
   (`overrides.rs:35-38`) is a live candidate for a row that looks inert.
   **Drive it before closing #2 and #4.**

2. **Whether `NoteView` exposes `/IT`.** The cheap form of the #7 fix would test
   the subtype marker directly instead of deserializing the sidecar per click. I
   did not read `NoteView`'s field list. Not measured.

3. **Whether the Properties panel is open by default**, and what it takes for an
   operator to find it. `panels\properties\mod.rs:452` shows `dimension::section`
   is called; whether the *panel* is visible at the moment of the click is a
   layout/session-state question I did not trace. **This is load-bearing for T2.**

4. **The ui-spec the overrides panel was built against.** `overrides.rs:9-15`
   quotes it — *"ce-dimension style AND tolerance in the GUI — one panel covering
   both"* and *"should have a default dimensioning and tolerance style that can be
   set"* — and `panels\dimension_groups\style.rs:114` cites **"§C.11.1"** of it.
   **There is no `docs\ui_specs\` directory in `D:\Dev\pdfcer-gui\`** (measured:
   `docs\` has no such subdirectory, and `tool-options-dock-and-ce-dimension-properties.md`
   was not found). The spec is quoted but absent where the code points. **That
   second quote reads as direct support for #3**, and I could not read its
   surrounding paragraph to be sure. **Find the spec before settling §12 item 1.**

5. **Exact `edit.rs` line numbers for the engine verbs.** Those cited here
   (`set_dimension_style` 16115, `set_dimension_display` 15921,
   `set_group_scale` 15549, `set_group_style` 16052, `dimension_rects` 15645,
   `place_dimension` 15804) are read from
   `D:\Dev\pdfcer\docs\core-api\02-editing-and-saving.md` §1.22, **not measured
   against `edit.rs`**. That doc's own preamble warns source wins when they
   disagree. Treat them as approximate. The *verb names* and the *shell call
   sites* in this survey were measured.

6. **`D:\Dev\pdfcer\docs\core-api\03-capabilities.md:109-553`** states `gui [ ]`
   for the style cascade and tolerance (citing `FEATURES.md:103-104`). **That is
   stale** — contradicted by the current `D:\Dev\pdfcer\docs\FEATURES.md:246-247`
   and by the shipped panel measured in §2 and §4. I did not determine when it
   went stale, only that it has. `index.md` in the same directory warns of exactly
   this ("Source is authoritative when these disagree with it").

7. **Whether any of this behaves differently on a rotated page or inside a
   form-wrapped CAD sheet.** Not in scope for the nine, not measured.

---

### 14. Proposed driven-check names, collected

Lowercase, underscores, no digits.

```
clicking_a_ce_dimension_does_not_open_the_comment_window
clicking_a_cloud_with_a_comment_still_opens_it
clicking_a_ce_dimension_selects_it_and_fills_the_properties_panel
the_comment_window_on_a_ce_dimension_can_be_turned_back_on
an_existing_group_offers_a_fraction_control
changing_a_group_unit_discloses_a_fraction_it_could_not_keep
a_unit_override_on_one_ce_dimension_changes_what_is_drawn
a_symmetric_tolerance_typed_on_one_ce_dimension_is_drawn
a_group_tolerance_default_reports_how_many_members_take_it
changing_the_arrow_form_on_a_group_redraws_its_members
hovering_a_ce_dimension_line_says_it_is_the_line
hovering_a_ce_dimension_label_says_it_is_the_label
the_label_handle_moves_the_label_and_not_the_line
a_radius_ce_dimension_offers_a_leader_side
a_radius_ce_dimension_offers_a_centre_mark
```

---

### 15. Corrections owed to existing records

1. **`D:\Dev\pdfcer-gui\OPERATOR_REQUESTS.md:9503`** — *"`painting.rs:437-450`
   draws page-space segments for the ce-dimension placement preview"*. Stale.
   Measured today: `canvas\painting.rs:117`, paint loop at `canvas\painting.rs:579`.
2. **`D:\Dev\pdfcer\docs\core-api\03-capabilities.md`** — `gui [ ]` for the
   ce-dimension style cascade and tolerance. Stale; both shipped.
   `FEATURES.md:246-247` is current and says so. **Read-only tree — record the
   correction here and hand it over; do not edit it.**
3. **`D:\Dev\pdfcer-gui\OPERATOR_REQUESTS.md:277-326`** (O183 as filed,
   2026-09-12) — no correction to the filing, which quotes Ken verbatim and is
   right to. But the survey's findings should be appended: **#1 is half-true with
   a different defect underneath, and #2, #4, #8a and #9 are shipped.**

---
