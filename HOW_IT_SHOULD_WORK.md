# HOW IT SHOULD WORK

**The target interaction model for pdfcer-gui.**
Written 2026-08-26, as part of the interaction-design audit opened by the
operator's report of 2026-08-26 ("The interface for this has gotten so wonky…").

This document is a **manual**, not a design sketch. It is written for two
readers at once:

- **The operator**, who should be able to read any section and know exactly
  what will happen when he presses the button.
- **The engineer**, who should be able to build the described behaviour from
  this file alone, without inventing a single rule.

Every non-obvious rule carries its **why**, and where the rule is inherited
from another application, that application is **named**. Where the reference
applications disagree, the disagreement is stated and one side is chosen with
the argument attached.

Where the text says *"today"*, it is describing shipped pdfcer-gui behaviour and
cites `file:line`. Everything else is the **target**. Nothing in this file has
been implemented by writing it.

---

## Contents

0. [Vocabulary](#0-vocabulary)
1. [The one-sentence model](#1-the-one-sentence-model)
2. [Selecting](#2-selecting)
3. [Manipulating](#3-manipulating)
4. [Seeing and editing properties](#4-seeing-and-editing-properties)
5. [Tools versus objects](#5-tools-versus-objects)
6. [Modes](#6-modes)
7. [OCR](#7-ocr)
8. [Cursors and status](#8-cursors-and-status)
9. [What must never happen](#9-what-must-never-happen)
10. [Conformance checklist](#10-conformance-checklist)

---

## 0. Vocabulary

These words are used precisely throughout. Where the operator's word and the
implementation's word differ, the operator's word is the one the interface
must show him.

| Term | Meaning |
|---|---|
| **Mark** | Ink that is actually painted on the page: a filled region, a stroked line, a glyph, a raster image's pixels. What the operator can see. |
| **Object** | One entry in the page's content model — a text object, a path, an image, a form XObject. May or may not paint a mark. |
| **Leaf** | An object that paints marks itself and contains no other objects. |
| **Container** | An object whose visible marks are produced by *other* objects inside it. In PDF this is almost always a **form XObject**. |
| **Part** | A subpath of a path, or one show-operator run inside a text object. The middle rung of the selection ladder. |
| **Node / anchor / point** | One coordinate on a subpath. The operator's *"points"*. The bottom rung. |
| **Hit** | An object the pointer is geometrically over, per §2.2. |
| **Candidate list** | Every hit under the pointer, deepest-painted last, i.e. topmost first. `hit_test_point_all` already computes this (`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:126-135`). |
| **Selection** | The set of objects, parts or nodes the next verb will act on. Exactly one such set exists per document. |
| **Focus** | *Not a thing this design has.* The Objects panel's separate `focus` field (`crates/pdfcer-gui/src/panels/objects/mod.rs:158-161`) is retired by §4.4. |
| **Grip / handle** | A drawn square or disc on the selection's bounding box that a drag transforms through. |
| **Armed tool** | The tool that decides what a press on empty page does. Today `CanvasTool` (`crates/pdfcer-gui/src/canvas/tool/mod.rs`). |

---

## 1. The one-sentence model

> **Click a mark and you select the smallest visible thing that painted it;
> the selection is always drawn, always named, always has handles, and always
> owns the property panel.**

Four clauses, each of which is a promise that everything below is obliged to
keep.

1. **"a mark"** — hit testing is against ink, not against boxes. §2.2.
2. **"the smallest visible thing that painted it"** — containers and packaging
   are not the answer to a click. §2.6.
3. **"always drawn, always named, always has handles"** — there is no
   selection state in this program that shows nothing, says nothing, or cannot
   be dragged. §3, §8.
4. **"owns the property panel"** — selection is the *only* input to the
   property surface, and it retargets on the selection event with no further
   gesture. §4.

**Why this sentence and not another.** It is the sentence that all seven
surveyed editors (Illustrator, Inkscape, Figma, Affinity Designer, CorelDRAW,
PowerPoint, Acrobat) would each answer "yes" to, and it is the sentence
pdfcer-gui currently answers "no" to on all four clauses. The operator's
complaint is not a request for a feature; it is a report that this sentence is
false here and true everywhere else he has worked.

---

## 2. Selecting

### 2.1 The resting state

The **Select tool** (`V`) is the resting state of the canvas. It is already
`#[derive(Default)]` (`crates/pdfcer-gui/src/canvas/tool/mod.rs:188-192`) and
already bound to `V` (`crates/pdfcer-gui/src/shell/manifest/mod.rs:331`). That
stays.

- `Esc` with no selection and no gesture in flight **returns to the Select
  tool** from any other tool.
- Holding `Ctrl` while any authoring tool is armed **temporarily** switches to
  Select for as long as the key is held. *(Illustrator does this with
  `Ctrl`/`Cmd`; CorelDRAW does it with `Space`. Illustrator's is chosen
  because `Space` is already the pan modifier here and in every browser.)*

**Why a home state at all:** six of seven surveyed applications return to the
arrow automatically or on one keystroke, and PowerPoint has no modal tools at
all. Acrobat is the sole outlier — object selection is behind an explicit mode
change — and Acrobat is the program being replaced.

### 2.2 What a click hits, and what it does not

**The rule: a click hits an object if the pointer is within `6.0` screen
pixels of ink that object actually paints.**

The tolerance constant already exists and does not change:
`SELECT_SCREEN_TOLERANCE_PX = 6.0` (`crates/pdfcer-gui/src/canvas/mapping.rs:94`).
It is converted to page units once per frame at the current zoom
(`mapping.rs:231-233`). It is deliberately tighter than the snap radius of
`10.0` (`crates/pdfcer-gui/src/canvas/snap.rs:136`) because *a snap is an offer
and a selection is a commitment*.

Per object kind:

| Kind | Interior of its bounding box | Its painted region | Its outline |
|---|---|---|---|
| Path with a **fill** (`f`, `B`, `f*`, `B*`) | not a target as such | **hit** anywhere inside the filled region | hit within 6 px |
| Path with **stroke only** (`S`, `s`) | **not a target** | — | hit within 6 px of the stroke centreline, plus half the stroke width |
| Path with `paint = none` (`n`) | **not a target** | — | hit within 6 px of the geometry. *Already correct in core* (`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:22-23`). |
| Text object | **hit** anywhere inside the union of its glyph boxes | — | — |
| Raster image | **hit** anywhere inside its placed quad | — | — |
| Form XObject | **never a first-click target at all** — see §2.6 | — | — |

**Why ink and not bounding boxes.** This is the single strongest agreement in
the entire reference survey: **7 of 7** applications hit-test against painted
geometry. Two of them ship a *preference* to make the rule even stricter for
line-art work — Illustrator's **Preferences ▸ Selection & Anchor Display ▸
"Object Selection by Path Only"**, and CorelDRAW's **"Treat all objects as
filled"** toggle on the property bar, which Corel's own documentation says to
turn off "when you work with line drawings and often need to select objects
that appear beneath other objects." That is Ken's exact working case, and both
vendors moved their default *toward* him, not away.

**Consequence the operator relies on:** clicking the empty interior of an
unfilled rectangle reaches whatever is behind it — usually nothing, i.e. a
deselect. This is convention row **C2** in `D:\dev\rag\ui-conventions\click-selects.md`
and it is what makes a dense CAD sheet workable at all.

**Two PDF-specific rules the reference apps have no opinion on, decided here:**

- **Clipped-away ink is not a target.** If an object's geometry is outside the
  clip path in effect when it was painted, the clipped-away part paints
  nothing and is therefore not hittable. *Why: the rule is "ink", and clipped
  geometry is not ink. Without this rule a CAD sheet's off-sheet construction
  geometry becomes a click-swallower exactly the way a full-page form does.*
- **OCR-invisible text (`Tr 3`) is not an object-selection target**, but it
  **is** a target for the character sweep (`PickClass::Characters`). *Why: the
  operator asked for an invisible layer that makes Find and copy work
  (`crates/pdfcer-gui/src/text/ocr.rs:80-87`); a layer that also starts
  swallowing object clicks would be the anchor fact happening a second time,
  and this time we would have caused it.*

### 2.3 Priority — what wins when several things are under the pointer

A press resolves in this order. The **first** rule that produces a target
wins; nothing below it is consulted.

1. **Rotate handle** of the current selection (§3.1).
2. **Resize grip** of the current selection (8 of them, §3.1).
3. **Node/anchor**, if the Node tool (`A`) is armed or the selection is
   already at the Node rung.
4. **Annotation** — markup, ce dimension, form field — subject to the pick
   filter.
5. **Page content object**, topmost painted first.
6. **Nothing** → §2.5.

Within rule 5, ties are broken by **paint order: topmost wins** (convention
**C3**). There is no area-based tiebreaker and there must not be one; "the
small thing on top of the big thing wins" already falls out of paint order,
because the small thing was painted later.

**The amendment C3 needs, and this document adds:** *topmost-wins is only
honest if there is a way to reach what is underneath.* §2.7.

### 2.4 Hover — the user is told what will be selected before committing

**Every pointer move over the canvas with the Select or Node tool armed
produces two pieces of feedback, before any click:**

1. **A hover outline** — the prospective target's *painted geometry* traced at
   1 px in the selection colour at 60 % alpha. Not its bounding box: the
   outline must be the same shape the selection outline will be, or the hover
   is lying about what the click will do.
2. **A status-bar readout** naming it, in the operator's words, with its
   containment path:
   `Path — 4 segments · inside Title block (form)`
   `Text — "WEIGHT: 683.33LBS"`
   `Image — 1200 × 900 px`

**Why both.** The survey splits: Illustrator (Smart Guides object
highlighting, on by default, `Ctrl+U`), Figma ("Highlight layers on hover",
a named preference people go looking for how to turn *off*) and Acrobat
(a dotted box on hover, plus every editable block outlined permanently on
entering Edit) draw an outline. Inkscape and CorelDRAW describe the thing in
the **status bar** instead. Only PowerPoint does neither for shapes, and
compensates with a permanently-mounted Selection Pane. We do both because we
have a harder problem than any of them: a CAD sheet has thousands of hairlines
and no layer names, so the outline says *which* and the readout says *what*.

**This is not a new capability, it is an unmounted one.** The measure tools
already draw exactly this — `crates/pdfcer-gui/src/canvas/measure/hover.rs`,
painted at `crates/pdfcer-gui/src/canvas/measure/mod.rs:1020-1039` — and its
header quotes the operator asking for it: *"I should be able to hover over
a[n object]…"*. He has asked once, been given it for one tool, and not for
selection.

**It is not blocked by the disclosure rule.** `crates/pdfcer-gui/src/canvas/overlay.rs:277`
and `:402` record that the rule "explicitly welcomes *pre-commit affordances*:
snap indicators, **hover highlights**, rubber-bands and selection handles."
The forbidden thing is content marking. A hover highlight is not that.

**The performance contract, because this is the objection that killed it
before.** `crates/pdfcer-gui/src/canvas/interact.rs:1412-1427` refuses hover
hit-testing on the grounds of a measured **355 ms** cost. That figure is the
**text extraction**, not the vector hit test, which `hit.rs:141-148` describes
as "one linear pass over the page's objects." The target rule is therefore:

- Hover hit-testing runs **only on pointer-move**, never on a still pointer,
  never during a drag, and never on a frame where the document changed.
- The result is memoised on `(page_index, pointer rounded to 2 px, zoom
  bucket, filter)`.
- **Budget: 4 ms.** If a page's hover probe exceeds 4 ms three frames running,
  the outline is dropped for that page and only the status readout remains,
  and the status bar says so once: `This page is too dense to preview
  selections; the pointer still selects normally.`
- The budget must be verified against the two known worst cases: the
  6,681-anchor / 1,194-subpath CAD export named in
  `crates/pdfcer-gui/src/canvas/pick.rs:60-66`, and the benchmark sheet
  in `BENCHMARK.md`.

### 2.5 Clicking on nothing

**A press on empty page that releases without travel deselects everything.**
A press that travels starts a marquee. The two are distinguished by travel
alone — the threshold is the platform drag threshold, and the marquee only
becomes visible once it is exceeded.

This is **7 of 7** across the survey, and it is already implemented twice
(`crates/pdfcer-gui/src/canvas/clicking.rs:59-66`, convention **C6**, which
clears both the content selection and the separate annotation selection).

`Esc` also deselects. `Esc` a second time returns the tool to Select.

**The reason C6 cannot fire on the the conformance suite’s composite page page today is the whole audit in one
sentence:** with a full-page form XObject hit-tested as a solid rectangle
(`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:24-26`), *there is no empty
page*. Every pixel is a hit, so the gesture `clicking.rs` calls "the gesture
every operator tries first" re-selects the same page-sized object, which reads
as stuck. §2.6 removes the cause.

**One honest exception.** A page that genuinely contains a full-page **painted**
object — a white background rectangle, a full-page scan — has no empty space,
legitimately, and clicking it legitimately selects it. That case is handled by
telling the truth (the hover outline showed the whole page before the click,
and the status readout named it) and by §2.7 (Alt-click gets past it). It is
*not* handled by inventing a special case, because a full-page white rectangle
is a real object the operator may genuinely need to delete.

### 2.6 Containers — the form XObject rule

**A form XObject is never the target of a first click. Clicking a mark that a
form paints selects the leaf object inside the form that painted it.**

This is the single most important rule in this document, and it is the one
place where pdfcer-gui deliberately **departs from the graphics-editor
convention**. The departure needs its argument stated in full.

**What the convention says.** In Illustrator, Inkscape, Figma, Affinity,
CorelDRAW and PowerPoint, a single click selects the **outermost group**, and a
modifier or a double-click descends. That is 6 of 6 of the apps that have
groups, and it is a strong convention.

**Why it does not transfer.** In every one of those applications, a group is
something **the user made, on purpose, and can see in a layer tree with a name
they gave it.** Respecting the group boundary respects the user's own
statement of intent.

A PDF form XObject is **none of those things**:

- The operator did not create it. SolidWorks, or the the conformance suite’s composite page test generator, or
  Word did.
- He cannot see it. It has no visible boundary, no name, no colour.
- It carries no editorial meaning. A producer may emit one per page, one per
  title block, one per glyph run, or none at all, for reasons that are entirely
  about its own output pipeline.
- Two files that look identical may differ entirely in their form structure.

Treating producer packaging as user intent means the same click on the same
drawn line means different things depending on which CAD package exported it.
That is not a selection model; it is a lottery.

**Acrobat, the program being replaced, agrees — and this is the one place
Acrobat gets it right.** Acrobat's Edit model synthesises a layout-level DOM of
`PDFEditableItem`s clustered from the page tag tree (RTTI evidence in
`C:\Program Files\Adobe\Acrobat DC\Acrobat\plug_ins\TouchUp.api`); a form
XObject wrapper never enters that set, so Acrobat *could not* return one from a
click even if it tried. Observed on a real SolidWorks drawing: a single click
on a horizontal line in an orthographic view selected **that one path**, with a
degenerate zero-height selection box, not the enclosing view and not the page.

**Three things this rule requires of the engine:**

1. **The decomposer must recurse into form XObjects.** Today it does not:
   `D:\Dev\pdfcer\crates\pdfcer-core\src\vector\decompose.rs:2666-2672` handles a
   `Do` on a form by calling `emit_image(ImageSource::Form, …)` with the form's
   `/BBox` corners and returning. The contents are never decomposed, never
   counted, never given a `TargetId`. **For content inside a form there is
   currently no object to win a click.** That is why double-click descends into
   nothing — a form has zero parts
   (`crates/pdfcer-gui/src/panels/objects/provider.rs:359-365`,
   `:385-391`) — and it is the direct cause of the operator's *"When I double
   click on an object it doesn't select — it still only has the whole page
   selected."*
2. **Each recursed leaf carries its containment path** — the chain of form
   XObjects it lives inside — so the status bar can say `inside Title block`
   and §2.8's select-parent can work.
3. **A form's own bounding-box hit test is retired** for first-click purposes.
   `hit.rs:24-26` may keep the bbox test for images (an image's quad *is* its
   ink) but a form must be removed from the first-click candidate list
   entirely.

**The form is still reachable**, three ways, all in §2.8.

**Cost note, stated honestly:** recursion multiplies object counts. A page whose
29 objects become 4,000 after recursion is a real possibility on a CAD sheet.
The Objects panel must therefore lazily expand containers rather than
materialising the whole tree, and the hover budget of §2.4 applies to the
recursed model.

### 2.7 Select-behind — reaching what is underneath

**`Alt`+click selects the topmost object under the pointer. Each further
`Alt`+click at the same point selects the next one down the candidate list.
At the bottom it wraps to the top.**

- The status bar **names each object as you land on it**, so the cycle is
  legible rather than guessed. *(CorelDRAW does exactly this: "The status bar
  displays a description of each hidden object as you select it.")*
- The hover outline updates to the newly selected object, so the cycle is also
  visible on the page.
- The point is captured on the first `Alt`+click of a cycle. Moving the pointer
  more than 3 px resets the cycle.
- `Shift`+`Alt`+click **adds** the next object down to the current selection
  rather than replacing it. *(Inkscape.)*

**Why `Alt` and not `Ctrl`.** The survey splits: `Alt` in Inkscape, Affinity
Designer and CorelDRAW; `Ctrl`/`Cmd` in Illustrator. Three to one, and the
three include the two most CAD-adjacent members of the set. `Ctrl` is also
already spoken for here — it is the temporary-Select modifier (§2.1) and the
multi-select modifier in Office — so `Alt` is both the majority and the free
key. Illustrator's own implementation carries a documented wart worth avoiding:
select-behind "works when clicking an object's fill, but not its path."
pdfcer-gui's cycle works on any hit, fill or stroke.

**The machinery already exists and is discarded.** `hit_test_point_all`'s doc
comment says in terms that it "exists so a GUI can offer click-through cycling:
repeated clicks at one point step down the returned list, which is the only way
an object completely covered by another can ever be selected"
(`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:126-135`), and
`topmost_allowed` (`crates/pdfcer-gui/src/canvas/input.rs:144-155`) walks that
exact list and throws the tail away. This is a binding, not a feature.

**The escape hatch when the cycle is the wrong tool: right-click.** §2.10.

### 2.8 Going up and down the ladder

There are three rungs — **object**, **part**, **node** — plus the containment
chain above the object rung. Four ways to move between them, and no ritual:

| Gesture | Effect |
|---|---|
| `A` (Node tool) | Arms direct selection. Click an object and every anchor appears immediately; click an anchor to select it. *This already exists and is correct* (`crates/pdfcer-gui/src/canvas/tool/mod.rs:195-222`), modelled on Illustrator's Direct Selection (`A`), Inkscape's Node tool, CorelDRAW's Shape tool. |
| Double-click on the selection | Descends one rung: object → part → node. Still supported, still tested, no longer the only route. |
| `Ctrl`+click on a mark | Selects the **container** that owns it — the form XObject, as one object, with one bounding box and one set of handles. Repeat to go up another level of nesting. |
| `Ctrl`+`Up` / `Ctrl`+`Down` | Selects the parent container / re-descends to the previously selected child, without moving the pointer. |

**Why `Ctrl`+click means *up* here when it means *down* in Figma and
CorelDRAW.** Because our default is already down. Figma's plain click selects
the outermost frame, so its `Cmd`/`Ctrl`+click "deep select" has to dig
downward. Ours selects the leaf (§2.6), so the only direction left to offer is
upward. The *concept* — one modifier that jumps the hierarchy in the direction
the default does not go — is Figma's and CorelDRAW's; only the sign is
inverted, and it is inverted because the default is.

**The containment path is always on the status bar** while something is
selected: `Title block (form) ▸ Path`. Illustrator's isolation-mode breadcrumb
is the precedent — Adobe's argument, which holds here, is that *you must never
be at a depth you did not choose, and the depth must be stated in words while
you are there.*

### 2.9 Marquee

**A drag that starts on empty page draws a rubber band. On release, every
object entirely enclosed by the band is selected.**

- **Default: enclose.** `MarqueeMode::Enclosed` already
  (`crates/pdfcer-gui/src/panels/objects/provider.rs:879`), decided in
  `hit.rs:83-86` decision 011.
- **`Alt` held during the drag switches to touch** — anything the band crosses
  is selected. The band's appearance changes when it does, so the mode is
  visible while it is active.
- **`Shift` held adds** the marquee's result to the existing selection instead
  of replacing it. 7 of 7.
- A marquee never starts on top of a selectable object; a press there moves
  that object instead (§3.2). 7 of 7. To touch-marquee starting from over an
  object, hold `Shift`+`Alt`. *(Inkscape's own resolution of the same
  collision.)*

**Why enclose, when the survey is split.** Illustrator and Figma default to
touch; Inkscape, CorelDRAW, Affinity and PowerPoint default to enclose — 4 to
2. More decisively, the two that default to touch are both tools for making
small numbers of large objects, and the operator's documents are the opposite:
thousands of hairlines where a touch marquee sweeping a title block would
select the border, the sheet frame and every zone letter it crossed. Inkscape
and CorelDRAW — the two with the densest documents — both default to enclose
*and* both use `Alt` to loosen to touch, which is the only real point of
agreement on the modifier and is therefore what we copy.

**A CAD note flagged rather than adopted:** AutoCAD distinguishes
crossing-versus-window by **drag direction** (right-to-left crosses,
left-to-right windows). No application in the graphics survey does this, and
adopting it would mean the same drag means different things depending on which
corner you started from, with nothing on screen saying so until you release. It
is not adopted. If the operator asks for it later, it is a preference, not a
default.

### 2.10 Right-click

**Right-clicking an unselected object selects it first, then opens a menu
scoped to it.** 7 of 7. No left-click is required first. Right-clicking empty
page opens the page/document menu instead.

The object menu carries, at minimum:

- Cut / Copy / Delete
- Arrange ▸ Bring to Front / Bring Forward / Send Backward / Send to Back
- Align ▸ (six)
- Rotate 90° left / right, Flip horizontal / vertical
- **Select ▸** — the disambiguation list, below
- Properties (the *object's*, §4.2)

**The Select submenu is the recovery path, and it is the cheapest safety net in
this whole document.** It lists **every object under the pointer**, topmost
first, each named the way the hover readout names it, each highlighting on the
page as the menu row is hovered. Clicking one selects it.

**Why:** three of seven applications ship exactly this as the documented
recovery for "the click hit the wrong thing" — Figma's **Select layer**
submenu, Illustrator's **Select ▸ Next Object Below / First Object Above**,
Inkscape's **Select Same ▸**. It costs one menu build over an already-computed
candidate list, and it means that no matter how badly hit-testing goes wrong on
some file we have never seen, **there is always a way to select the thing the
operator is pointing at.**

### 2.11 The pick filter stays, and gets a voice

`PickFilter` / `PickClass` (`crates/pdfcer-gui/src/canvas/pick.rs`) is correct
and stays exactly as designed — eleven switchable classes on the status bar,
subtractive, `AND`ed with mode capability, one click from anywhere. It is
pdfcer-gui's version of Illustrator's "Object Selection by Path Only" and
CorelDRAW's "Treat all objects as filled", and it is more granular than either.

Three amendments:

1. **`PickClass::FormXObject` changes meaning.** With §2.6 in force it no
   longer means "may a click land on a form"; it means **"may `Ctrl`+click and
   the Objects panel select a form as one object"**. Its documented purpose —
   relief from *"why is the selection box so big?"* (`pick.rs:203-209`) — is
   met by §2.6 by default, so the row's default becomes irrelevant to the
   complaint rather than being the fix for it.
2. **The filter must never be off-screen.** `crates/pdfcer-gui/src/app/status.rs:806-820`
   records the measured case where at `ui_scale = 1.80` in an 1100 × 800 window
   the fixed status cluster needed 666 pt of 611 pt available and the filter
   landed at x = −127. A control that explains why nothing is selectable must
   not be the control that disappears when the window is small. It collapses to
   an icon before it sheds.
3. **A refused click says so.** If a press produced a hit that the filter
   excluded, and no other candidate survived, the status bar states it (§8.3,
   wording in §3.7).

---

## 3. Manipulating

### 3.1 What appears around a selection

**Nine targets: eight resize grips and one rotate handle.**

| Target | Count | Drawn | Existing constant |
|---|---|---|---|
| Corner grips (NW, NE, SE, SW) | 4 | 8 pt filled square, screen-sized, does not scale with zoom | `GRIP_SIZE_PX = 8.0` (`crates/pdfcer-gui/src/canvas/handles.rs:100`) |
| Mid-edge grips (N, E, S, W) | 4 | same | dropped on an axis shorter than `MIN_MID_GRIP_EXTENT_PX = 24.0` (`handles.rs:117`) |
| Rotate handle | 1 | disc on a 20 pt stem above top-centre | `ROTATE_STEM_PX = 20.0` (`handles.rs:335`) |
| Grab slack | — | 2 pt beyond the drawn square | `GRIP_GRAB_SLACK_PX = 2.0` (`handles.rs:109`) |

All of this is already built and already correct. It is listed here because
this document has to be complete, not because it needs changing.

**Corner grips are never dropped**, only mid-edge ones, and only when the box
is under three grip-widths on that axis — otherwise the mid-edge grip would sit
on top of its neighbours. Illustrator does the same thing and its users log it
as a bug ("The bounding box for lines and narrow objects doesn't have center
handles"); it is not a bug, it is the only way a degenerate box stays aimable.

**Why a drawn rotate handle rather than a hot zone outside the corner.** The
survey splits three ways: hot region outside a corner (Illustrator, Figma,
Acrobat), a dedicated visible handle (PowerPoint, Affinity), a second-click
mode toggle (Inkscape, CorelDRAW). The hot-region design is the one users
complain about — Illustrator's own bug tracker carries "Objects rotating when
trying to transform or move" and "Cannot grab corner handles on bounding box" —
because the zone is invisible and sits within a few pixels of the resize grip.
Nobody complains about PowerPoint's rotation handle. pdfcer-gui has already taken
the visible-handle route and it is the right one.

**A selection always has handles.** There is no selection state in this program
that draws an outline and nothing else. Illustrator has one — `View ▸ Show
Bounding Box` off — and it produces a steady stream of "my handles disappeared"
reports. We do not ship that toggle.

### 3.2 Moving

**Press on an unselected object selects it, and the same drag moves it.** One
gesture. No click-first ritual. **7 of 7**, zero variation, and it is the
convention whose absence most reliably reads as "the program is broken."

- Dragging the **body** of the selection moves it.
- **`Shift` constrains** the move to horizontal or vertical.
  *(Shift is the Adobe/Microsoft/Serif answer; Inkscape and CorelDRAW use
  `Ctrl`. Shift is chosen because it is already this program's constrain key
  everywhere else, and because a single constrain key across move, resize and
  rotate is worth more than matching either family exactly.)*
- The constraint is **announced while it is active** on the status bar —
  `constrain::caption` already does this
  (`crates/pdfcer-gui/src/app/status.rs:709-713`, convention **D5**).
- **`Alt`+drag duplicates**, leaving the original in place. *(Illustrator,
  Figma, Affinity. CorelDRAW uses right-button-release, PowerPoint uses
  `Ctrl`+drag; `Alt` is the majority and `Ctrl` is taken.)* Note the collision
  with `Alt`+click select-behind, resolved by travel: no travel = cycle,
  travel = duplicate-drag, exactly as click and marquee are resolved.
- A live geometry readout follows the drag in the Properties panel (§4.5).

### 3.3 Resizing

| Gesture | Effect |
|---|---|
| Drag a **corner** grip | Scales both axes, anchored at the opposite corner |
| Drag a **mid-edge** grip | Scales one axis, anchored at the opposite edge |
| `Shift` + corner drag | Constrains aspect ratio |
| `Alt` + any grip drag | Scales about the selection's **centre** instead of the opposite corner/edge |
| `Shift`+`Alt` + corner drag | Proportional, about the centre |

**Why these assignments.** Both behaviours exist in all seven applications;
only the keys differ. Shift-constrains-aspect is Illustrator, Figma, Affinity,
PowerPoint and Acrobat (5 of 7 — Inkscape uses `Ctrl`, CorelDRAW makes corner
drags proportional by default). Alt-from-centre is Illustrator, Figma and
PowerPoint's `Ctrl` equivalent. We take the Adobe assignment because Acrobat is
the program being replaced and its users' fingers already know it, and because
it keeps `Shift` meaning "constrain" across move, resize and rotate.

**Scale-strokes question, which must be answered and not dodged.** Every app in
the survey treats "resize the frame" and "scale everything inside it" as
distinguishable operations — Illustrator's `Scale Strokes & Effects`, Figma's
separate Scale tool (`K`), Affinity's and CorelDRAW's toggles. pdfcer-gui's
answer:

- **Stroke widths, text size and dash patterns scale with the object by
  default.** *Why: on an engineering drawing, a resized detail whose line
  weights stayed put looks wrong immediately, and the operator resizing a
  placed logo expects it to scale as a picture does.*
- A **`Scale line weights` checkbox** lives in the Properties panel's Transform
  section, remembered per session.

### 3.4 Rotating

- Drag the rotate handle. The pivot is the selection's bounding-box centre.
- **`Shift` snaps to 15°.** *(5 of 7: Inkscape, Figma, Affinity, CorelDRAW,
  PowerPoint. Illustrator's bbox rotate snaps to 45°, which is too coarse for
  drawing work.)*
- The live angle is shown beside the pointer and in the Properties panel's
  rotation field while the drag is in flight.
- **A movable pivot is not shipped in the first cut.** Inkscape, CorelDRAW and
  Affinity have one; Illustrator has it only via the separate Rotate tool;
  Figma's is recent; PowerPoint and Acrobat have none. It is a real feature and
  it is deferred, and this sentence exists so that the deferral is a decision
  on the record rather than an omission.

### 3.5 Nudging

**Three tiers, all configurable, defaults:**

| Keys | Step | Precedent |
|---|---|---|
| Arrow | **1.0 pt** | Illustrator (1 pt), Figma (1), Affinity (1 pt), Acrobat form fields (1 pt) |
| `Shift`+arrow | **10.0 pt** (10×) | universal — 7 of 7 |
| `Ctrl`+arrow | **0.1 pt** (÷10) | the three-tier apps: Inkscape (`Alt` = 1 screen px), CorelDRAW (micro nudge), PowerPoint (`Ctrl` fine nudge) |

**Why three tiers.** Two tiers is universal; a *finer* third tier appears in
exactly the three most CAD-adjacent applications in the survey. Sub-step
keyboard positioning is table stakes in the tools drafters already use, and the
operator is a drafter.

**All three are user-configurable.** Nobody in the survey hard-codes them.

**The nudge must repeat.** Acrobat carries a long-standing defect — "Selecting
an object with the Object Edit tool and nudging with the arrow keys will only
nudge it once", the object must be re-selected for each nudge — which is on its
own UserVoice with years of votes. Holding an arrow key here repeats at the
platform key-repeat rate, and a run of nudges coalesces into **one** undo entry
if they are uninterrupted.

### 3.6 Multi-selection

**One union bounding box, one set of nine targets, transforms apply to the
whole set.** Members keep a faint individual outline so the operator can see
what is in the set. 5 of 5 among the apps that do it this way (Illustrator,
Inkscape, Figma, Affinity, CorelDRAW).

**PowerPoint's divergence is explicitly rejected.** PowerPoint draws handles
around each ungrouped shape separately, and typing a width into the ribbon sets
*every* selected shape to that width rather than scaling the set — which is why
"resize multiple ungrouped objects proportionally" is a perennial PowerPoint
question with no native answer. We do not reproduce that.

Properties for a mixed selection: §4.3.

### 3.7 Refusals — how "you cannot move this" is said

This is the **weakest** convention in the entire survey and the one place with
genuine headroom. Illustrator historically made locked objects unselectable
with no feedback at all; Figma's locked layers silently pass the click through
and there is a standing forum request for "a layer locked hint notification
[to] clarif[y] why I cannot click to select"; Inkscape gives nothing on canvas.
**No application in the survey changes the cursor to a "no" symbol, and none
shows a message on a refused drag.**

Two apps do it well, and both do it the same way: **by changing the handles
themselves**, which is the strongest available signal because it appears at the
exact place the user is aiming.

- **CorelDRAW**: a locked object still selects, and its eight control handles
  are drawn as **padlocks**.
- **PowerPoint 365**: a locked shape still selects — bounding box appears —
  but has **no sizing handles at all**.

**pdfcer-gui's rule, taking CorelDRAW's design and adding the sentence nobody
has:**

1. **A refused object still selects.** The operator gets a selection outline
   and a name. Never a silent nothing, never a fall-through to something else.
2. **Its handles are drawn as padlocks** rather than squares. The rotate handle
   is not drawn.
3. **The first refused drag prints one sentence on the status bar**, which
   clears on the next successful gesture.

**The exact wording. These strings are normative; do not paraphrase them.**

| Situation | Status-bar sentence |
|---|---|
| Locked object, drag attempted | `This object is locked. Right-click it to unlock.` |
| Read mode, drag attempted on content | `Read mode does not change the page. Press Ctrl+3 for Edit.` |
| Review mode, drag attempted on page content | `Review changes your markup, not the page. Press Ctrl+3 for Edit.` |
| Click landed only on a class switched off in the pick filter | `Nothing here is selectable: {class} is switched off in the filter, on the status bar.` |
| Every class switched off | *(existing)* `filter::empty_note` (`crates/pdfcer-gui/src/app/status/filter.rs:234`) |
| Resize attempted on an object whose transform is not invertible | `This object's placement can't be recalculated, so it can't be resized. It can still be moved.` |

**Why words at all, when nobody else uses them.** Because the failure the
operator actually reported is *not knowing why nothing happened*. Every app in
the survey leaves him to deduce it; two of them at least make the handles carry
the message. Carrying the message **and** saying it in one sentence costs one
status-bar line and closes the entire class of complaint. The status bar
already has the machinery — `constrain::caption` and `filter::empty_note` are
exactly this shape, and `status.rs:788-791` already reasons correctly about
their relative priority.

**What a refusal must never be:** a modal dialog, a red badge, or a toast. The
operator's standing complaint about "nagging and red flagging"
(`crates/pdfcer-gui/src/panels/tool/mod.rs:63-78`) governs here.

---

## 4. Seeing and editing properties

### 4.1 The rule

> **Selecting an object populates a property surface for that object, on the
> selection event, with no further gesture.**

**7 of 7.** There is no mainstream editor in which selecting an object leaves
the property surface unchanged. Illustrator swaps the Properties panel, Figma
the right sidebar, Affinity the Transform/Colour/Stroke studio panels,
CorelDRAW the Properties inspector, Inkscape the Fill & Stroke dialog and
selector bar, PowerPoint materialises a whole contextual ribbon tab **and**
retargets the Format pane, Acrobat swaps the right-hand Format/Objects panel.

pdfcer-gui has three surfaces and none of them does it:

- The **Format tab** appears on `selection.any`
  (`crates/pdfcer-gui/src/shell/manifest/format.rs:120`, `:126`) and contains
  exactly two commands, `format.properties` and `format.delete` (`:130`). No
  editors.
- `format.properties` dispatches `file.properties` — the **document**
  properties command (`crates/pdfcer-gui/src/app/dispatch.rs:819-825`). The
  button whose stated question is "tell me about the thing I just clicked"
  answers a question about the file.
- The **Properties panel** holds real editors but is fed by the Objects
  panel's `focus`, not by the canvas selection
  (`crates/pdfcer-gui/src/panels/properties/mod.rs:64-70`), and is entirely
  read-only (`:71-86`).
- The **Tool panel** — the thing the operator calls "the Tool tab" — is
  architecturally about *what is armed*, never about *what is selected*, and
  says so (`crates/pdfcer-gui/src/panels/tool/mod.rs:23-40`). **That panel is
  right and must not change.** No application in the survey has a panel that
  shows only tool state while an object is selected — and where a surface is
  shared (Illustrator, Inkscape, CorelDRAW's property bar), **the selection
  takes priority over the tool whenever a selection exists.**

### 4.2 Where properties live: two surfaces, one source

**Surface 1 — the Properties panel. Always mounted in Edit and Review, always
retargets on selection, never needs to be found.**

This is Figma's and Affinity's architecture, and it is the one to build,
because it is the architecture in which "the panel changed" is the *only* event
the user has to notice. It is fed by **the canvas selection** and by nothing
else.

**Surface 2 — the contextual Format tab.** Keeps its `selection.any` visibility
and gains real content: the same controls as the panel, laid out as a ribbon
band, so the operator who works from the ribbon is not forced into a panel.

**Why both, when Figma ships one.** Illustrator and CorelDRAW ship both
simultaneously (Properties panel + Control bar; Properties inspector + Property
bar) and neither is redundant: the panel is for sustained work on one object,
the bar is for a single change without leaving the ribbon. pdfcer-gui has a
ribbon, so it needs the band; it has docks, so it needs the panel.

**The one hard rule binding them: they read the same state and write the same
commands.** Two surfaces that can disagree is worse than one surface.

### 4.3 What appears, per object kind

Sections are **hidden** when they do not apply; individual controls within a
shown section are **greyed**. That granularity rule holds across all eight
surveyed applications, and Microsoft states the underlying principle: *"Remove
the control when there is no way for users to enable it or they don't expect it
to apply… Disable a control when users expect it to apply and they can easily
deduce why."*

**Always present, for every selection:**

- **Transform** — X, Y, W, H, rotation, with a units selector and a lock-ratio
  toggle, plus a 9-point reference-point proxy saying which point X/Y describe.
  *(Illustrator's proxy; every app has X/Y/W/H except Acrobat, whose absence of
  it for page content is the single loudest reason a precision user abandons
  Acrobat for placement work.)*
- **Arrange** — bring forward / send backward / to front / to back.
- **Worth knowing about this object** — the read-only facts the current
  Properties panel already writes well. They stay, below the editors, at
  ordinary weight, under that heading. They are facts about the document, not
  warnings about pdfcer.

**Per kind:**

| Selection | Sections shown |
|---|---|
| **Nothing** | Page background, page size, crop box, units, grid/guides, and the document's Quick Actions. **Never an empty panel and never the words "Nothing selected."** *(Figma and Illustrator both fill the no-selection state with document properties; the reason is that a panel that goes blank teaches the user to stop looking at it.)* |
| **Path** | Transform · Fill (colour, colour space, none) · Stroke (width, cap, join, dash, miter) · Opacity · Blend mode · Arrange · Facts |
| **Text object** | Transform · Character (font, size, char spacing, word spacing, horizontal scale, baseline offset) · Fill · Stroke · Arrange · Facts (embedded/subset, and the name-join caveat the current panel already discloses) |
| **Image** | Transform · Opacity · Resolution and colour space (read-only) · Replace image… · Crop · Arrange · Facts |
| **Form XObject** (selected via §2.8) | Transform · Contents: *N objects* with an **Enter** button (§2.8) · Arrange · Facts |
| **Part** (subpath / run) | Its own bounds · its parent's Fill and Stroke, greyed with the note that they belong to the whole object · Arrange (disabled) |
| **Node** | X, Y of the anchor · its two control handles · segment type |
| **Markup annotation** | Existing `panels::properties::markup` content, now selection-fed |
| **ce dimension** | Existing `panels::properties::dimension` content, now selection-fed |
| **Form field** | Existing `panels::properties::formfield` content, now selection-fed |
| **Mixed multi-selection** | The **intersection** of applicable sections. Fields whose values differ show the word **`Mixed`**; typing into a `Mixed` field applies to every member. *(Figma's convention, and the only one in the survey that handles this without lying.)* |

### 4.4 The severance is repaired

**The Objects panel's `focus` field is deleted. Clicking a row in the Objects
panel selects that object on the canvas. Selecting on the canvas scrolls the
Objects panel to that row and highlights it. Hovering a row draws the hover
outline on the page; hovering the page highlights the row.**

Today these are two disconnected stores, deliberately, defended by a test named
`the_panel_focus_has_not_quietly_become_a_selection`
(`crates/pdfcer-gui/src/panels/mod.rs:736`). That test was right for a world in
which panel focus and selection had different lifetimes and different
capabilities. It is wrong for this one: **the layer/object list is, in every
application surveyed, the guaranteed route to an object the pointer cannot
reach**, and a list that cannot select is not that route.

- Microsoft's documented answer to "select an object hidden behind a full-slide
  picture" is the **Selection Pane** (`Alt`+`F10`): click a row to select, and
  a per-object eye icon to hide the thing on top temporarily.
- Figma, Illustrator and Inkscape all bind list ↔ canvas in both directions,
  with hover highlight in both directions.

Two features come with the repair, both from PowerPoint's pane:

- **A per-object visibility (eye) toggle**, view-only, not written to the file.
  It is how you get at something underneath without moving anything.
- **A per-object lock toggle**, feeding §3.7's padlock handles.

Consequence for `panels::properties`: its empty state no longer has to name the
Objects panel as "the only route in"
(`crates/pdfcer-gui/src/panels/properties/mod.rs:64-70`), because it no longer
is.

### 4.5 Immediate-apply versus commit

**Read-out is always immediate. Writing is gated by input method.**

| Input | Behaviour |
|---|---|
| Dragging on the canvas | The panel's numbers update **live, every frame**, while the drag is in flight. The document is changed once, on release, as one undo entry. |
| Dragging a field's **label** horizontally (scrub) | Applies live, every frame; one undo entry for the whole scrub. *(Figma's label scrub. Worth building: it is the fastest way to nudge a value without doing arithmetic.)* |
| Typing into a field | Applies on **`Enter`** (commits, keeps focus) or **`Tab`** (commits, moves to the next field). `Esc` abandons the edit and restores the previous value. |
| Spinner arrows | Apply immediately per click. *(PowerPoint's model, which is the one non-designers have internalised.)* |

**Fields accept arithmetic and units.** `12.5mm`, `1 1/2in`, `100+3`,
`72*2`. *(Figma documents `- + * / ^ ()`; Illustrator, Inkscape, Affinity and
CorelDRAW all accept units.)* The document's display unit is settable and the
field always echoes back in it.

**There is no `Apply` button.** Inkscape's Transform dialog and CorelDRAW's
Transform docker have one, and in both cases it is a *secondary* surface beside
a primary one that commits on `Enter`. We ship only the primary.

### 4.6 The contextual-tab question, answered

Microsoft's ribbon rules, quoted from the Win32 UX guide and the Ribbon UI
licensing guidelines, are the only written specification anyone has for this:

- Contextual tabs **MUST** be selected when a **new object is inserted**.
- Contextual tabs **MUST NOT** be automatically selected when an **existing
  object is selected** — "Users MUST click the Contextual Tab for it to be
  selected."
- They **SHOULD** be selected when the user **double-clicks** an existing
  object.
- They **SHOULD** be re-selected if the user had one active, deselected, and
  immediately selected another object of the same type.
- They **MUST** disappear when the object is deselected, and when the
  disappearing tab was active, the ribbon **MUST** fall back to the first tab
  rather than leaving a blank one.

**pdfcer-gui adopts all five, and it is only allowed to adopt the MUST NOT
because §4.2 gives it the second surface Office has.** Office can afford not to
steal ribbon focus because the Format task pane retargets silently at the same
instant; the user always sees the property surface change *somewhere*. An
application with only the tab would be applying half of Microsoft's rule and
would produce exactly the operator's complaint — *"the Tool tab doesn't switch
to giving me the editable stuff."*

So, concretely:

| Event | Format tab | Properties panel |
|---|---|---|
| Object selected by click | **Appears**, colour-flagged at the right end of the tab row. Does not steal focus. | **Retargets immediately.** |
| Object double-clicked | **Appears and becomes active.** | Retargets. |
| Object inserted or pasted | **Appears and becomes active.** | Retargets, with the new object already selected and handled (§5.3). |
| Deselected | Disappears; if it was active, the ribbon returns to the previous tab. | Shows the document/page state (§4.3). |
| Multi-selection of two kinds | Both kinds' bands appear. *(Office shows all relevant contextual tab sets at once.)* | Shows the intersection with `Mixed`. |

**And `format.properties` stops dispatching `file.properties`.** In all eight
surveyed applications, "Properties" next to a selection means the *object*;
document-level facts live in a separate File/Document Properties dialog. The
current wiring (`crates/pdfcer-gui/src/app/dispatch.rs:819-825`) is the exact
inverse of universal practice. `format.properties` opens/focuses the Properties
panel on the selection; `file.properties` keeps the document dialog.

---

## 5. Tools versus objects

### 5.1 When a tool is needed

**Never, to select.** Selection is the resting state (§2.1). The only tools
that exist are tools that *author* something or that change what a drag means:

| Tool | Key | A press means |
|---|---|---|
| Select | `V` | select / marquee / drag the selection |
| Node (direct selection) | `A` | select an anchor, or an object and show all its anchors |
| Hand | `H`, or `Space` held | pan |
| Text (sweep) | `T` | sweep characters to copy |
| Text edit / Add text | `Ctrl+E` / `Ctrl+Shift+E` | put a caret in text / place a new text object |
| Markup (each kind) | ribbon | author that markup |
| Measure / dimension (each kind) | ribbon | place that dimension |
| Form field (each kind) | ribbon | place that field |

Bindings `V`, `A`, `T`, `H` already exist
(`crates/pdfcer-gui/src/shell/manifest/mod.rs:331-334`).

### 5.2 An armed authoring tool does not select

**With a markup, measure, form or text-placement tool armed, a press on an
existing object authors a new object; it does not select the existing one.**
This is 6 of 7 in the survey (Inkscape's shape tools are the exception, and
they only select objects *of their own shape type*). It is already how
`press_kind` behaves and it is correct.

The corresponding obligation: **the operator must always be able to see which
tool is armed.** The Tool panel does this well
(`crates/pdfcer-gui/src/panels/tool/mod.rs`), the ribbon button is pressed, and
the cursor is a crosshair (§8). Three channels; the survey's mature apps use
three or four.

### 5.3 Newly created, pasted or placed objects arrive selected

**7 of 7.** An image the operator just placed is already selected, already has
its nine targets drawn, and is already described in the Properties panel. He
does not have to find it and click it in order to resize it.

This is the direct answer to *"if I add an image I Expect to click on it to
resize but dragging doesn't resize."* Two things were wrong: the object was not
arriving selected, and the handles were not reachable. Both are covered — this
rule and §3.1.

The Format tab **becomes active** on insert (§4.6, Microsoft's MUST).

### 5.4 Double-click may re-tool; single click never does

- **Single click never changes the armed tool.** 8 of 8 in the survey.
- **Double-click on text** switches to the text-edit tool with the caret placed
  at the click point. *(Illustrator, Figma, Affinity, CorelDRAW, PowerPoint,
  Acrobat all do this.)*
- **Double-click on a path** descends one rung (object → part → node), leaving
  the tool alone. *(Inkscape switches to the Node tool here; we do not, because
  `A` already exists as the signposted route and switching tools under the
  operator is the kind of invention the tool-ladder rewrite was written to
  stop — see `crates/pdfcer-gui/src/canvas/tool/mod.rs:195-222`.)*
- **`Esc` leaves** whatever a double-click entered, one level per press.

---

## 6. Modes

### 6.1 The question

Today, `Capabilities::edit_content` gates *selection* as well as editing
(`crates/pdfcer-gui/src/app/modes/capability.rs:164-170`, `:358`), so **Read
mode cannot select anything at all**. The argument for that, written in the
same file's §5, is twofold: every Format-tab verb takes the selection as its
operand so gating twice would be redundant, and — the load-bearing half — *"A
selection with nothing to read it is not inspection"*, because Read mounts no
Properties panel.

**That second argument dissolves the moment §4 lands.** With a
selection-fed Properties panel that can render read-only facts, a selection in
Read is exactly inspection, which is the most common thing a person does with a
PDF they are not editing.

### 6.2 The rule

| Mode | Select | Inspect | Move / resize / rotate / delete content | Markup & dimensions | OCR |
|---|---|---|---|---|---|
| **Read** (`Ctrl+1`) | **yes** | yes — Properties panel mounts, **read-only** | no | no | yes, into a **new document** (§7.6) |
| **Review** (`Ctrl+2`) | yes | yes — read-only for content, editable for own markup | no, for page content | yes | yes, into a new document |
| **Edit** (`Ctrl+3`) | yes | yes — fully editable | yes | yes | **yes, in place** (§7.6) |

**Selection is never gated by mode. Mutation always is.**

**Why.** Three arguments, in order of weight:

1. **The operator's own rule for Read is about *changing*, not about
   *pointing*.** *"Read may produce a new document; it may not modify this
   one"* (`crates/pdfcer-gui/src/dialogs/ocr.rs:36-42`). Clicking a line to see
   how wide it is modifies nothing.
2. **Acrobat's modality is the thing being escaped.** Acrobat is the sole
   modal outlier in the survey — nothing on the page is selectable until you
   find *All tools ▸ Edit a PDF* — and the operator's expectation of "just
   click it" is therefore explicitly *not* coming from Acrobat. It is coming
   from everything else he has used, all of which let the pointer select from
   the moment the document opens.
3. **A mode that refuses to select cannot explain itself.** Today the refusal
   is invisible: the click just does nothing. With selection allowed and
   mutation refused, the refusal has a place to be said (§3.7's Read-mode
   sentence) and an object to say it about.

**What is preserved from the current design:** the mode still governs which
tabs exist and which panels mount, `Capabilities` remains the owner of *what
may be authored*, and the pick filter remains authoritative on top of it —
`pickable(class) = capability_allows(class, mode) && filter.allows(class)`,
unchanged (`crates/pdfcer-gui/src/canvas/pick.rs`). What changes is that
`edit_content` splits into `select_content` (true in all three modes) and
`edit_content` (Edit only).

---

## 7. OCR

### 7.1 What is wrong now, stated plainly

Six of six surveyed OCR tools (Acrobat Pro, ABBYY FineReader 16, Foxit PDF
Editor, Nitro PDF Pro, PDF-XChange Editor, OCRmyPDF) default their scope to the
**whole document**, offer a **page-range** control in the dialog that starts
the job, and apply the result **in place** so that ordinary Save writes it back.
Zero of six force a Save-As on the open-document path.

pdfcer-gui does the opposite on both axes, deliberately and with the reasoning
in-tree:

- `Request` carries a single `page_index: usize`
  (`crates/pdfcer-gui/src/ocr/mod.rs:463-469`); the button reads **"Recognise
  this page"** (`crates/pdfcer-gui/src/text/ocr.rs:92`).
- The only write control is **"Save recognised copy as…"**
  (`crates/pdfcer-gui/src/text/ocr.rs:156`), whose tooltip promises "The
  document you opened is not changed" (`:162`), under the module rule *"no
  second save command, no in-place path, no `Save`-labelled control anywhere"*
  (`crates/pdfcer-gui/src/dialogs/ocr.rs:64`).

The operator's four questions — *"how do I OCR more than one page? Why does the
tool stop at one? Why do I have to save a copy? Where is the option to select
more than one page?"* — are, one for one, the four ways this differs from every
other program.

### 7.2 Scope

**The OCR dialog opens with a three-way radio group, exactly as Acrobat's has
since roughly Acrobat 6.** The layout below is not a guess: it is Acrobat's
own ADM dialog script, extracted verbatim from
`C:\Program Files\Adobe\Acrobat DC\Acrobat\plug_ins\PaperCapture.api`, and
matched by Foxit, PDF-XChange, ABBYY and OCRmyPDF's `--pages`.

```
Pages
  (•) All pages                      ← default, listed first
  ( ) Current page
  ( ) Pages [    ] to [    ]
  ( ) Selected pages (7)             ← enabled only when the Pages panel has a multi-selection
```

- **`All pages` is the default and is listed first.** 5 of 6 tools default to
  the whole document; the sixth (OCRmyPDF) has no dialog and OCRs everything
  unless `--pages` says otherwise.
- The range field accepts `2,3,5-7` and `2-4,9`, sorted and de-duplicated.
  *(ABBYY and OCRmyPDF both do this; OCRmyPDF warns on duplicates.)*
- **The Pages panel is the second route.** `Ctrl`-click / `Shift`-click
  thumbnails, right-click, **"Recognise selected pages…"** — the dialog opens
  with the fourth radio pre-selected. *(ABBYY's "Recognize Selected Pages",
  Nitro's "OCR Pages…" on a Ctrl-selected set. This is the direct answer to
  "Where is the option to select more than one page?": it is the thumbnail
  panel he already uses for reordering.)*

**One trap to avoid inheriting.** Acrobat's *automatic* OCR inside Edit PDF
genuinely is per-page and lazy — Adobe: "By default, only the current page is
converted to editable text in one go." That is a lazy-editing convenience, not
the OCR feature; the explicit `Scan & OCR ▸ Recognize Text` path is
document-scoped. If pdfcer-gui's single-page design was reasoned from Acrobat,
this is where the confusion came from.

### 7.3 Pages that already have text

There are exactly three policies. OCRmyPDF names them and every GUI implements
some subset. pdfcer-gui ships all three, named in the operator's words, with the
consequences stated in the dialog:

| Control label | Behaviour | Equivalent |
|---|---|---|
| **Leave pages that already have text** *(default)* | Pages with real text are copied through untouched; the rest are recognised. | OCRmyPDF `--skip-text`; PDF-XChange "Ignore existing text on page"; ABBYY "Use only text from PDF" |
| **Replace text that was added by a recogniser** | Invisible OCR text is stripped, visible text is masked out of the rendered image, the remainder is recognised, results re-inserted. **Vectors, real text and form fields survive.** | OCRmyPDF `--redo-ocr`; ABBYY "Use OCR" |
| **Recognise everything, flattening the page to an image** | The page is rasterised, then recognised. | OCRmyPDF `--force-ocr`; Acrobat's documented TIFF round-trip workaround |

**The third option carries a permanent, non-dismissable line directly under
it**, because on an engineering drawing it is destructive and the destruction
is invisible until someone tries to measure the flattened drawing:

> `This turns the page into a picture. The line work, the real text and any
> form fields stop being objects and cannot be selected, measured or edited
> afterwards.`

**Why default to "leave":** it is the most conservative of the three, it is
ABBYY's documented default, and it is the only one that cannot lose anything.

**Why not simply refuse, as Acrobat does.** Acrobat hard-refuses per page —
*"Acrobat could not perform recognition (OCR) on this page because: This page
contains renderable text"* — and offers no in-product force or redo; the KB's
only remedy is to export every page to TIFF and re-import. That is a refusal
with no way forward, and it is one of the specific frustrations pdfcer exists to
remove.

### 7.4 Options

Inline in the dialog, because in every surveyed GUI the language is reachable
from the OCR dialog itself and never from preferences only:

- **Language** (the models present in the model directory).
- **What to keep** — the three-way policy of §7.3.
- **Downsample** — none / 600 / 300 / 150 dpi. *(Acrobat's list; its default is
  600.)*
- **Deskew page content** — off by default. *(PDF-XChange, Nitro, OCRmyPDF all
  offer it; off, because rotating a CAD sheet's content by a fraction of a
  degree is a change to the drawing.)*

The confidence sentence stays exactly as written
(`crates/pdfcer-gui/src/text/ocr.rs`, `no_confidence()`). It is the best string
in the current dialog and nothing here touches it.

### 7.5 Progress and cancellation

**A modal progress panel with a live page counter, a per-page list, and a
Cancel button.**

```
Recognising page 7 of 36…
  ✓ 1–6 recognised          ✓ 4 skipped (already had text)
  [ Cancel ]
```

**Cancel semantics, stated explicitly because this is the least-documented
corner of every product surveyed:**

- **Cancel stops after the page in flight and keeps every page already
  recognised.** The document is left dirty with those pages recognised, and the
  status bar says so: `Cancelled. Pages 1–7 were recognised; the rest were
  left alone.`
- **Nothing is written to disk by cancelling.** Undo removes the recognised
  layers in one entry.
- Acrobat's behaviour here is undocumented and reported to leave a partially
  OCR'd dirty document; OCRmyPDF's is all-or-nothing because output is only
  written on success. We take Acrobat's *shape* (keep the work) with
  OCRmyPDF's *guarantee* (nothing hits disk), which is the combination neither
  ships.

**The window must not freeze.** The current tooltip promises "the window will
not respond while it does" (`crates/pdfcer-gui/src/text/ocr.rs:103`). That was
honest about a single page; across 36 pages it is not acceptable. The worker
already runs on its own thread (`crates/pdfcer-gui/src/ocr/mod.rs`); the dialog
polls it and the canvas keeps painting.

### 7.6 In place, and saving

**In Edit mode, OCR applies to the open document. It becomes dirty. `Ctrl+S`
saves over the original.**

**In Read and Review, OCR produces a new document**, offered as "Save
recognised copy as…" exactly as today.

**Why this is not a reversal of the operator's own rule.** His rule
(`crates/pdfcer-gui/src/dialogs/ocr.rs:36-42`) is *"Read may produce a new
document; it may not modify this one."* That is a statement about **Read**. The
current design enforces it by removing the in-place path from the *whole
program*, in every mode, which is a stronger rule than he asked for and it is
the one he is now complaining about. Re-siting the enforcement from the
*feature* to the *mode* keeps his rule exactly and removes the complaint:

- Read still cannot modify the open document. Nothing in Read can.
- Edit can, because that is what Edit is for, and `file.save` /
  `Ctrl+S` already exist (`crates/pdfcer-gui/src/shell/manifest/mod.rs:298`)
  along with `file.save_copy` / `Ctrl+Shift+S` (`:299`).

**Against the survey:** zero of six tools force Save-As on the open-document
path. PDF-XChange is the only one that even offers "create a new document" as a
control, it is a **checkbox inside the OCR dialog next to the other output
options**, and it is **off by default**. That checkbox is what pdfcer-gui should
ship — and it should sit beside the scope radios, off by default in Edit, on
and locked in Read.

Acrobat, the incumbent: **Save (Ctrl+S) is a first-class, always-present
command that writes back over the open file**; "Save as…" is the secondary,
explicitly-named alternative. Observed on the installed build (25.001.20435),
hamburger menu.

### 7.7 The rest of the OCR dialog

Unchanged: the intro sentence, the confidence sentence, the disclosure list.
They are good and they are the reason this dialog is more honest than any of
the six.

One addition: **a per-page result line** in the disclosure list, so that a
36-page run does not collapse into one number —
`Page 12: 41 words · Page 13: no text found · Page 14: skipped, already had text`.

---

## 8. Cursors and status

### 8.1 The cursor table

Complete. Every state the pointer can be in over the canvas.

| Situation | Cursor | Source / note |
|---|---|---|
| Over empty page, Select armed | `Default` arrow | Illustrator, Figma, Inkscape, PowerPoint all do this |
| Over the **body of an unselected object**, Select armed | `Default` arrow — **the hover outline carries the message, not the cursor** | Illustrator and PowerPoint change nothing here either; only Inkscape adds an open hand. Deliberate: a cursor change over every hairline on a CAD sheet would flicker constantly |
| Over the **body of the current selection** | `Move` (four-way) | PowerPoint, Figma. Existing: `Grip::Move` → `CursorIcon::Move` (`crates/pdfcer-gui/src/canvas/handles.rs:203-218`) |
| Over a **corner grip** NW or SE | `ResizeNwSe` | existing, `handles.rs:203-218` |
| Over a **corner grip** NE or SW | `ResizeNeSw` | existing |
| Over a **mid-edge grip** N or S | `ResizeVertical` | existing |
| Over a **mid-edge grip** E or W | `ResizeHorizontal` | existing |
| Over the **rotate handle** | `Grab` today; **a custom rotate glyph is owed** | egui 0.35 has no rotate cursor. `handles.rs:203-218` records this as a compromise, not a choice, and convention **H6** asks the cursor to *name* the gesture. A texture and an atlas entry. |
| While **rotating** | the rotate glyph, held | |
| Over an **anchor**, Node tool armed | arrow with a small square glyph | Illustrator's Direct Selection |
| Hand tool armed | `Grab`; `Grabbing` while dragging | existing, `crates/pdfcer-gui/src/canvas/tool/mod.rs:517-560`. The pair matters: a pan that has run out of scroll must be distinguishable from a pan that is not working |
| `Space` held (temporary pan) | `Grab` / `Grabbing` | |
| Text sweep or text edit armed | `Text` I-beam, **rotated to match the text's orientation** | existing, `crates/pdfcer-gui/src/canvas/cursor.rs:201`, built because Acrobat does it |
| Any markup / measure / form / place-text tool armed | `Crosshair` — the custom two-tone one | existing, `crates/pdfcer-gui/src/canvas/cursor.rs`; built because the stock white crosshair was invisible on white paper |
| Dragging a ruler guide | existing guide cursors | `crates/pdfcer-gui/src/canvas/guides.rs:890-893` |
| Over a **locked** object's padlock handle | `NotAllowed` | the one place a "no" cursor is correct, because there is a specific thing being refused and a padlock drawn under the pointer explaining it |

**The rule that governs the whole table:** `CanvasTool::Select` returns `None`
(`crates/pdfcer-gui/src/canvas/tool/mod.rs:521`) so that the grip cursors
underneath are not overwritten. That reasoning is correct and is preserved; the
"there is something here you can click" signal is carried by the **hover
outline** (§2.4), not by a cursor. That is the split Illustrator and Figma both
use, and it is why neither needed a cursor for it.

### 8.2 The status bar, left to right

| Region | Contents |
|---|---|
| **Far left — the readout** | *(new)* What is under the pointer on hover; what is selected once something is. Includes the containment path. See §8.3. |
| Left — transient | Drag-constraint caption, page-drag caption, refusal sentences (§3.7), render notes behind a disclosure triangle |
| Right | Page number box · zoom · fit · Find · **pick Filter** — existing (`crates/pdfcer-gui/src/app/status.rs:625-880`) |

### 8.3 The readout — exact strings

**Hovering, nothing selected:**
```
Path — 4 segments · inside Title block (form)
Text — "WEIGHT: 683.33LBS" · Helvetica 8 pt
Image — 1200 × 900 px
Form — 214 objects
```

**Selected, one object:**
```
Selected: Path — 4 segments · 118.4 × 0.0 pt at (312.0, 455.5) · inside Title block (form)
```

**Selected, several:**
```
Selected: 7 objects (5 paths, 2 text) · 214.0 × 96.5 pt
```

**Nothing selected:**
```
Nothing selected. Click a mark to select it, drag on blank paper to select several, Alt+click to reach what is underneath.
```

**Why that last string exists and why it is that long.** Inkscape's equivalent
— *"No objects selected. Click, Shift+click, Alt+scroll mouse on top of
objects, or drag around objects to select."* — is the single best empty-state
string in the whole survey, because it is **a tutorial placed exactly where the
failure is felt**. An operator who has just clicked three times and got nothing
is reading the status bar; that is where the answer belongs. It replaces
nothing and costs nothing.

**Priority when several transient lines compete** — the existing reasoning at
`crates/pdfcer-gui/src/app/status.rs:788-791` holds and extends: *"a decline
explains why one gesture did nothing, while [the empty filter note] explains
why EVERY gesture will."* Order, highest first:

1. `filter::empty_note` — every gesture will fail
2. A refusal sentence (§3.7) — this gesture failed, and why
3. Drag-constraint caption — a gesture is in flight
4. The selection readout
5. The hover readout
6. The nothing-selected tutorial

---

## 9. What must never happen

These are the anti-patterns this design forbids. Each is stated as a
prohibition with the reason and, where it applies, the specific report or
reference case that produced it.

**9.1 A click must never select the page.**
There is no page object. Clicking blank paper in Acrobat selects nothing —
observed. If a click resolves to an object whose bounds are the whole page,
either it genuinely painted the whole page (in which case the hover outline
said so first, and `Alt`+click gets past it) or the hit test is wrong.

**9.2 A container must never be the answer to a first click.**
§2.6. A form XObject is producer packaging, not user intent. If a click returns
a form XObject as a leaf, the decomposer failed to recurse.

**9.3 A bounding box must never be a hit region for anything that is not a
raster image.**
`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:24-26` is the defect. 7 of 7
applications hit-test ink.

**9.4 A gesture must never fail silently.**
Every refused press produces either a selection with padlock handles or a
sentence on the status bar, and often both. This is the one place the industry
convention is weak and where we are deliberately going further than any of the
seven.

**9.5 A selection must never exist without handles.**
Illustrator ships `View ▸ Show Bounding Box` and reaps a permanent stream of
"my handles disappeared." We do not ship that toggle. A locked object's handles
are padlocks — still drawn, still nine, still saying what is going on.

**9.6 Dragging a handle must never do nothing.**
"Any editor where dragging a handle does nothing will read as broken, not as a
different design." If the transform cannot be applied, the handle is a padlock
(§3.7), never an inert square.

**9.7 A newly placed object must never arrive unselected.**
7 of 7. This is the operator's image-resize complaint in one line.

**9.8 The property surface must never stay unchanged when the selection
changes.**
7 of 7. There is no mainstream editor in which this is allowed.

**9.9 "Properties", next to a selection, must never mean the document.**
`crates/pdfcer-gui/src/app/dispatch.rs:819-825` is the inverse of universal
practice in all eight surveyed applications.

**9.10 A panel that shows object properties must never be fed by anything but
the canvas selection.**
The `focus`/selection severance (`crates/pdfcer-gui/src/panels/objects/mod.rs:158-161`)
is the defect. One selection, one truth.

**9.11 The Tool panel must never become an inspector.**
`crates/pdfcer-gui/src/panels/tool/mod.rs:23-40` is right. The complaint is not
that the Tool panel is wrong; it is that the panel that *should* have answered
did not exist. Fixing the wrong panel would break a good one.

**9.12 A contextual tab must never appear empty.**
`crates/pdfcer-gui/src/shell/manifest/format.rs:126-131` — a tab whose entire
content is two commands, one of which answers the wrong question. Either it
carries the property band (§4.2) or it does not appear.

**9.13 A hidden or covered object must never be unreachable.**
`hit_test_point_all` exists for exactly this
(`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:53-63`, `:126-135`) and is
currently truncated to its first element
(`crates/pdfcer-gui/src/canvas/input.rs:144-155`). Three routes are required:
`Alt`+click cycling, the right-click Select submenu, and the Objects panel.

**9.14 A gesture the operator must be told about must never be told about only
in documentation.**
`Alt`+click's existence is announced by the status bar while `Alt` is held —
Inkscape's design, and the reason Inkscape users know about `Alt`+click without
reading anything.

**9.15 OCR must never stop at one page.**
6 of 6 tools default to the whole document. `crates/pdfcer-gui/src/ocr/mod.rs:463-469`.

**9.16 A destructive OCR option must never be unlabelled.**
`--force-ocr` on an engineering drawing destroys the line work. The consequence
is stated under the control, permanently (§7.3).

**9.17 A feature must never enforce a mode rule the mode system should
enforce.**
The OCR dialog's "no `Save`-labelled control anywhere"
(`crates/pdfcer-gui/src/dialogs/ocr.rs:64`) applies a Read-mode rule to Edit
mode. Rules belong to their owner.

**9.18 A long operation must never freeze the window.**
`crates/pdfcer-gui/src/text/ocr.rs:103` promises exactly that. Acceptable for one
page, not for thirty-six.

**9.19 A status control that explains a refusal must never be the control that
disappears when the window is small.**
`crates/pdfcer-gui/src/app/status.rs:806-820` — the measured off-screen filter.

**9.20 The program must never invent an interaction where a convention exists.**
The operator's own verdict, on the record:
*"The selector should be predictable like other programs. It seems a lot of
ideas are getting invented instead of just using the … most common method
expected."* (`crates/pdfcer-gui/src/canvas/tool/mod.rs:195-222`.) Every rule in
this document either names the applications it comes from or states why the
convention does not transfer.

---

## 10. Conformance checklist

An implementation conforms to this document when all of the following are true
of the file
`the conformance suite’s composite page`, page 1 (29 objects, two of them
page-sized), and of a SolidWorks drawing export.

**Selecting**

- [ ] Hovering any drawn mark outlines that mark and names it on the status bar.
- [ ] Hovering blank paper outlines nothing and shows the nothing-selected line.
- [ ] Clicking a drawn line selects that line, not the page, not a form.
- [ ] Clicking blank paper deselects.
- [ ] The page-sized `paint=none` path is selectable only within 6 px of its edge.
- [ ] The page-sized form XObject is never returned by a plain click.
- [ ] `Ctrl`+click on a mark inside the form selects the form as one object.
- [ ] `Alt`+click cycles down the candidate list, naming each on the status bar, wrapping at the bottom.
- [ ] `Shift`+click adds and toggles off.
- [ ] Drag from blank paper draws a band; only fully enclosed objects are selected; `Alt` switches to touch mid-drag.
- [ ] Right-click on an unselected object selects it and opens a menu containing a Select submenu listing every object under the pointer.
- [ ] Clicking a row in the Objects panel selects it on the canvas; canvas selection scrolls and highlights the row.

**Manipulating**

- [ ] A selection has 8 grips and a rotate handle, screen-sized, mid-edge grips dropped below 24 pt.
- [ ] Press-and-drag on an unselected object selects and moves it in one gesture.
- [ ] Corner drag resizes both axes; `Shift` constrains; `Alt` scales from centre.
- [ ] `Shift` constrains a move to an axis and says so on the status bar.
- [ ] Rotate snaps to 15° with `Shift`.
- [ ] Arrow / `Shift`+arrow / `Ctrl`+arrow nudge by 1 / 10 / 0.1 pt and repeat on key-hold, coalescing into one undo entry.
- [ ] A locked object selects, shows padlock handles, and prints `This object is locked. Right-click it to unlock.` on a refused drag.
- [ ] A placed image arrives selected with handles drawn.

**Properties**

- [ ] Selecting anything repopulates the Properties panel in the same frame.
- [ ] The Format tab appears on selection without stealing focus, and becomes active on double-click and on insert.
- [ ] X/Y/W/H/rotation read out live during a drag and accept typed values with units and arithmetic, committing on `Enter`/`Tab`.
- [ ] `format.properties` shows the object, not the document.
- [ ] A mixed selection shows `Mixed` in differing fields and applies typed values to all members.
- [ ] Nothing-selected shows document and page properties, never a blank panel.

**Modes**

- [ ] Read selects and inspects and refuses to mutate, with a sentence.
- [ ] Edit does everything.

**OCR**

- [ ] The dialog opens with `All pages` selected.
- [ ] `Pages n to m` accepts `2,3,5-7`.
- [ ] Multi-selecting thumbnails and right-clicking offers "Recognise selected pages…".
- [ ] The default existing-text policy leaves text pages untouched, and the flattening option carries its warning.
- [ ] Progress shows page k of n with a Cancel that keeps completed pages and writes nothing.
- [ ] In Edit, the result applies in place and `Ctrl+S` writes over the original.
- [ ] In Read, the result is offered only as a new document.
- [ ] The window stays responsive for a 36-page run.

---

## Appendix A — the modifier key table, complete

| Gesture | Modifier | Effect |
|---|---|---|
| Click | — | select the leaf under the pointer |
| Click | `Shift` | add to / toggle out of selection |
| Click | `Alt` | select-behind; repeat to cycle down; wraps |
| Click | `Shift`+`Alt` | add the next object down to the selection |
| Click | `Ctrl` | select the container that owns the mark |
| Drag from blank paper | — | marquee, enclose |
| Drag from blank paper | `Alt` | marquee, touch |
| Drag from blank paper | `Shift` | marquee adds to the selection |
| Drag from over an object | — | move it |
| Drag from over an object | `Shift` | move constrained to an axis |
| Drag from over an object | `Alt` | duplicate and move |
| Drag from over an object | `Shift`+`Alt` | touch-marquee starting over an object |
| Drag a corner grip | — | scale both axes from the opposite corner |
| Drag a corner grip | `Shift` | scale proportionally |
| Drag any grip | `Alt` | scale about the centre |
| Drag the rotate handle | — | rotate about the centre |
| Drag the rotate handle | `Shift` | rotate in 15° steps |
| Arrow | — | nudge 1 pt |
| Arrow | `Shift` | nudge 10 pt |
| Arrow | `Ctrl` | nudge 0.1 pt |
| Any authoring tool armed | `Ctrl` held | temporarily become the Select tool |
| Any tool | `Space` held | temporarily become the Hand tool |
| Any state | `Esc` | leave one level: cancel a drag, then leave a descent, then deselect, then return to Select |
| Selection | `Ctrl`+`Up` / `Ctrl`+`Down` | select the parent container / re-descend |
| Selection | `Delete` / `Backspace` | delete, in Edit only |
| — | `V` / `A` / `T` / `H` | Select / Node / Text sweep / Hand |
| — | `Ctrl`+`1` / `2` / `3` | Read / Review / Edit |
| — | `Ctrl`+`S` / `Ctrl`+`Shift`+`S` | Save / Save a copy |

## Appendix B — where the reference behaviours come from

| Rule in this document | Taken from | Confidence |
|---|---|---|
| Hit test by painted ink | all seven surveyed editors | documented in Illustrator, Inkscape, Figma, CorelDRAW; observed in Acrobat |
| Unfilled interiors pass clicks through | Illustrator, Inkscape, Figma, CorelDRAW, PowerPoint | documented except PowerPoint (widely reported) |
| Containers are not first-click targets | Acrobat's `PDFEditableItem` model | inferred from RTTI symbols in `TouchUp.api` **plus** direct observation of a SolidWorks drawing |
| Hover outline before the click | Illustrator (Smart Guides), Figma (Highlight on hover), Acrobat (dotted box) | documented / observed |
| Status-bar description on hover | Inkscape, CorelDRAW | documented |
| Empty-state tutorial string | Inkscape | documented |
| Marquee = enclose, `Alt` = touch | Inkscape, CorelDRAW | documented, both |
| `Shift` adds and toggles | all seven | documented |
| `Alt` cycles down the z-stack | Inkscape, Affinity, CorelDRAW | documented |
| Right-click Select-layer list | Figma, Illustrator, Inkscape | documented |
| 8 grips + drawn rotate handle | PowerPoint, Affinity | documented |
| `Shift` constrains aspect, `Alt` from centre | Illustrator, Figma, Acrobat | documented |
| 15° rotation snap | Inkscape, Figma, Affinity, CorelDRAW, PowerPoint | documented |
| Three-tier nudge | Inkscape, CorelDRAW, PowerPoint | documented |
| Padlock handles on a locked object | CorelDRAW | documented |
| Selection-fed property surface | all seven | documented |
| Contextual tab MUST/MUST NOT/SHOULD rules | Microsoft Win32 UX guide + Ribbon UI licensing guidelines | quoted verbatim |
| `Mixed` for differing values | Figma | documented |
| List ↔ canvas both-ways binding, eye and lock toggles | PowerPoint Selection Pane, Figma Layers | documented |
| OCR three-radio page scope | Acrobat's own ADM dialog script in `PaperCapture.api` | extracted from the shipped binary |
| OCR defaults to all pages | Acrobat, ABBYY, Foxit, Nitro, PDF-XChange, OCRmyPDF | documented, 6 of 6 |
| Thumbnail multi-select as OCR scope | ABBYY, Nitro | documented |
| skip / redo / force policy vocabulary | OCRmyPDF | documented |
| In-place OCR + ordinary Save | Acrobat, ABBYY, Foxit, Nitro, PDF-XChange | documented / observed |
| "Create a new document" as an off-by-default checkbox | PDF-XChange | documented |
| `Ctrl+S` overwrites the open file | Acrobat 25.001.20435 | observed on this machine |

**Not verified, flagged:** PowerPoint's no-fill interiors being click-through
(widely reported, no Microsoft documentation found); Acrobat's marquee being
touch or enclose; Acrobat's OCR cancel semantics; Affinity Designer's hover
feedback; whether the the conformance suite’s composite page page's visible ink lives inside the full-page form
or beside it — that last one is the highest-value remaining measurement and it
decides whether the operator's content is currently *hard* to select or
*impossible* to select.
