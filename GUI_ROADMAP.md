# pdfcer GUI — roadmap

What is not yet built in this shell, the order it gets built in and the reason
for that order, and the scope questions that need the operator's ruling; read by
whoever picks up the next piece of work.

---

## How the order is chosen

Order by what the operator reaches for, never by what most recently arrived in
the engine channel. The engine's replies are an input to *how* a thing gets
built; they are never an input to *which* thing gets built next. Among items the
operator weights equally, cheapest first — a shipped small thing is worth more
than a scheduled large one.

Two consequences decide sequencing more often than cost does:

- **A capability's presence is expressed by registering its command.** A ribbon
  item naming an unregistered command is dropped from the merged manifest with a
  `CapabilityAbsent` skip; it is never drawn inert. "Build the surface now and
  wire it later" is therefore not an available ordering.
- **An unavailable capability renders nothing.** Greying is reserved for a
  *temporarily* unavailable capability and is always explained on hover. Several
  items below are blocked on a **consumer** rather than on a condition: flipping
  a manifest condition to reveal a control over a code path that does not exist
  is exactly what that rule prevents.

Three registers divide the work between them, and duplicating a row across two
of them is how they drift apart:

| Register | Holds |
|---|---|
| `RESUME.md` | which operator request is next, in the operator's order |
| `FEATURES.md` | row-level status for every shipped and planned capability |
| this document | what remains, grouped by what it waits for, and the open scope questions |

---

## A standing rule this investigation earned

**Every GUI change needs a verification that drives the running binary,
not only a test that passes.**

`tools/ui-verify/src/lib.rs` cites this section by name, because the rule is the
reason that crate exists. Two defects were invisible to a fully green test suite
and obvious within thirty seconds of using the application:

- **The Delete key.** The only test of the keyboard collector built a bare
  `egui::Context` with no widgets, so the focused-widget condition that breaks
  the real application could not occur in the harness.
- **Invisible headings.** Two theme tests sat adjacent to the bug and neither
  measured a rendered foreground/background pair. A screenshot audit separately
  found **two ribbon groups rendering with no caption at all** — caught by a
  screenshot, not by a test.

Those are the two findings that justified building the harness. It launches the
release binary, opens a fixture, drives a scripted sequence through the OS,
captures the window, and asserts on both the `PDFCER_DIAG` trace and the pixels.
The smallest useful set it started from, and which `tools/ui-verify/src/checks`
still names as its founding three:

1. Click an object on the canvas, assert selection non-empty, press Delete,
   assert the object count dropped — through the real event loop.
2. Screenshot each ribbon tab and assert every group has a caption whose
   rendered contrast against its background exceeds a threshold.
3. Screenshot the Settings dialog and assert the same for every section
   heading.

The suite is far larger now; `tools/ui-verify/src/checks/mod.rs` is the list and
the additions are this same rule applied to each new surface as it lands. A
capability with no check that drives it is not done.

---

## The phase numbers the source cites

Module headers across both crates cite this document by phase number, and
several quote a row's own words. The work is built; the numbers are live
identifiers, so they are kept here with what they named.

| Cited as | What it named | State |
|---|---|---|
| Phase 1 | Make selection mean something — context menus first, then the selection model below | built |
| 1.2 | *"move and resize anything carrying a `/Rect`"* (`FEATURES.md:208`) — markup, form widgets, redaction marks, links and ce dimensions unblocked by one row | built, and extended to content-stream objects |
| 1.3 | *"Eight handles plus move, per the convention every drawing tool shares."* | built |
| Phase 2 | Rebuild the ribbon — seven tabs, specified in `RIBBON_IA.md`. Its rules stand: P1a (the quick-access toolbar and status bar are shortcut surfaces), P2 (the ribbon picks the activity, the sidebar holds its controls), **P3 (no placeholders)**, P4 (mandatory group captions), P5 (nothing floats over the canvas) | built |
| Phase 3 | Viewer conventions — cursor-anchored zoom, **3.2** a hand tool and space-to-pan, **3.3** an editable page-number box (*"Reaching page 37 of 42 currently means the thumbnail rail or 36 keystrokes."*), zoom to selection and to a marquee region, a recent-files list, a persisted dock layout, a thumbnail rail that reflows to a grid | built |
| Phase 4 | Page display modes — single stays the default; continuous, facing and facing-continuous are chosen on View and remembered per document. **4.3** is scroll-driven current-page tracking: greatest visible area wins, lowest index breaks a tie, and a press outranks the scroll | built |
| Phase 5 | Text editing — see *Text editing* below for what is left of it | partly built |

---

## Rules that decide what gets written next

These govern how the program behaves. They are here because an engineer who does
not know them writes something that compiles, passes, and is wrong.

### Index stability across edits

Two families of edit, two behaviours. Mixing them up corrupts a document
silently, because the next gesture edits a different object than the one on
screen.

| Family | Members | What it does to indices |
|---|---|---|
| `move_*` | `move_object`, `move_objects`, `move_subpath`, `move_node`, `move_nodes`, `move_handle` | Rewrites operator **operands** in place. Nothing renumbers, so a held index still names the same object afterwards. |
| `delete_*` | `delete_object`, `delete_objects`, `delete_subpath`, `delete_node`, `delete_text_run` | Excises byte **spans**. Everything after the excision renumbers, so every held index is stale the moment the call returns. |

After a delete, remap held indices through
`pdfcer_core::vector::remap_index_after_delete`. It answers `None` for *it is
gone* and never a different object's index, and it handles unsorted and
duplicated input. Carrying an index across a delete without it is the defect.

### Selection is identity, not position

The operator's requirement, in force: *"if I select a node or something for a
tool, I should be able to pan and zoom out without losing my first selection."*

> **Navigation is not an edit. Panning, zooming, changing fit mode, rotating the
> view, switching page-display mode and changing ribbon tab must never alter the
> selection.**

Selection is held as page, object index, sub-path and node — never as a point or
a rectangle. From that:

- **Navigation does not clear it.** A page change, a zoom, a scroll and a
  re-render all leave it intact.
- **A completed click on empty space clears it; a press does not.** A press that
  becomes a drag is a marquee or a move, and clearing on press makes both
  gestures begin by destroying their own operand.
- **Identity is re-resolved, not discarded, when the page is decomposed again**
  after an edit. Discarding it is what makes a selection appear to evaporate for
  no reason the operator can see.

### Coordinate spaces

PDF user space is y-**up**. Image space and screen space are y-**down**. Every
conversion between them is a place a sign error hides, and a sign error in a hit
test reads as *the click missed*.

Hit-test and snap `tolerance` is a radius in **page** space, not in pixels. It
has to be derived from a pixel slop at the current zoom, or deep zoom gets a
tolerance the size of a room.

### Whole-page rendering is the default, and tiling is refused

The canvas rasterises a whole page. A generation counter plus a settle debounce
abandons superseded renders mid-flight, so a rapid run of zoom steps starts a
generation per step and completes exactly one — the last — while the operator
sees the previous texture linearly scaled. That is why zoom and pan feel the way
they do; it is cheaper than a tile cache rather than a compromise against one.

Tiling is **cancelled**, on measurement rather than on taste: the cost of a
region render is almost entirely area-independent, so a one-point region costs
about what a hundred-thousand-pixel region costs, and a three-by-three ring is a
ninefold regression. Deep zoom is served instead by **one region per viewport**
— `pdfcer_render::render_page_region`, called from
`crates/pdfcer-gui/src/render/worker.rs` — with a whole-page-to-region
crossover. Tiling survives only as a way to bound memory on an enormous
viewport, never to save time.

The interpretation floor cannot be parallelised: a content stream is a state
machine where the save and restore operators, the transform, colour and clip all
accumulate, so an operator cannot be interpreted without the state every
operator before it left behind. Driving several region calls across cores
multiplies that floor instead of dividing it.

### Reflow stays operator-invoked

Automatic re-wrap on edit invents line breaks the file never stated. Reflow is
made *reachable*, never silent.

### Applied content renders exactly as saved content will render

Fuzzy, never sneaky. Nothing provisional is painted on the canvas; disclosure
lives off-canvas — the status line, a results panel, a report, a properties
field. A pre-commit affordance that is *the cursor* is welcome and is not an
exception to this.

### Never write a bare "dimension"

A *ce dimension* is one pdfcer authors. A *pdf dimension* is CAD-exported page
content pdfcer reads and must not silently alter. A sentence that says only
"dimension" is ambiguous in the one place the ambiguity is expensive.

### `egui-shell` never learns what a PDF is

The shell crate owns the ribbon, the dock, modes, layout persistence, the theme
and the command registry, and knows nothing about documents. Anything that has
to know what a page is belongs in `pdfcer-gui`.

---

## Text editing — the largest remaining block

Fixtures first, then the shell-side work, then the engine-blocked items: the
engine-blocked items cannot start, and the fixtures decide whether any of the
rest can be believed.

### Fixtures, before anything else here

The generated text fixtures are single-run by construction: one show operator
per line, real space glyphs, uniform font and uniform size
(`tools/gen-reflow-fixture.py`, `tools/gen-textedit-fixtures.py`). Every
condition that fails in the field is absent. Add a multi-run paragraph, mixed
sizes within one block, rotated text, and words separated by positioning rather
than by space glyphs — first, so the work below has something to fail against.

### Live layout while typing — blocked on the engine

The draft is drawn as ghost text in a proportional font over a translucent mask,
and real layout runs once on commit. You type at the wrong widths and it snaps
to reality on Accept. Re-measuring per keystroke is the fix, and the measurement
of whether it can be afforded already exists:

```sh
cargo test -p pdfcer-gui --lib canvas::textedit::cost -- --ignored --nocapture
```

That test is `#[ignore]`d by design and reports a per-keystroke cost for a small
synthetic block, a real drawing sheet and the dense benchmark drawing. Roughly
16 ms is one frame and roughly 50 ms is felt as lag; read the figures from the
run, never from prose.

**What blocks it:** the engine computes the advance delta inside its edit
planner before any write, and neither the planner nor its plan type is public. A
public *measure this edit* entry point — or making the planner and its plan
public — is the whole of what this waits on.

**A cheap approximation is refused.** Summing `ExtractedGlyph::advance` over the
draft works only for characters the page already shows; a character it does not
show has no width there, so the approximation is silently wrong for exactly the
input that motivates the feature. **Debouncing is refused too**: a re-layout
appearing a moment after you stop typing is a second, later surprise rather than
a fix for the first one.

### Reflow reachability

Reflow re-wraps a block to the block's own extent, at an alignment inferred from
glyph positions, with leading taken from the block's own median baseline gap.
The engine's reflow request carries a wrap width, an alignment and a leading
override, and this shell passes none of the three — it supplies only the page
crop box, which is what buys the *your paragraph now ends below the bottom of
the sheet* disclosure. Exposing any of the three is shell-only work; whether to
is an open question below.

Two refusals remain, and both are the engine's:

| Refusal | What it takes to lift |
|---|---|
| A word boundary realised as a derived word space rather than as a real space glyph is not a break opportunity, so CAD output that positions words with text-positioning offsets presents one unbreakable word and nothing wraps | tokenisation in the engine's reflow planner |
| The font resource and the text matrix must be uniform across the block | relaxing the uniformity checks in the engine's reflow applier |

The shell's half is already done: every engine-side refusal is worded **after**
the attempt, from the engine's own discriminant, never forecast. A shell-side
pre-flight guard over the same model is refused on principle — two predicates
over one model is the shape that ships silent disagreement, and no test of
either can see them disagree.

### Cross-run editing — blocked on the engine

An edit request pins to one show-text operator, and a text array is one
operator, so a paragraph split across several runs must be edited run by run.
**This is not silently disabled**: a caret landing where two runs meet refuses
*in a sentence*, on the status bar, naming what to do instead.

Lifting it needs a multi-run edit request in the engine that groups runs into a
line or a block and re-emits them as a set. It is the correct end state and the
most expensive item in this document. How much of it to build is an open
question below.

---

## Measure — the takeoff set

The dimension-group model — named groups carrying a shared scale and a drafting
standard — is the differentiator here, and nothing below should dilute it.

| Item | What it waits for |
|---|---|
| **Angular** | Shell only. The engine's dimension kinds carry an angular variant and author it; this shell registers no command and arms no tool. The absent-command register still records this as needing an engine verb, which is stale. |
| **Area** | An engine verb. The dimension kinds are linear, circular, angular and perimeter; there is no area kind to author. |
| **Count tool and takeoff schedule** | An engine verb plus a schedule surface. The larger of the two remaining takeoff items. |
| **Aligned** | Nothing. It is a *constraint* on a linear pick rather than a tool, so it belongs on a property control and not as another armable kind. |

---

## Shell surfaces with no route to a capability that exists

Each is small, independently shippable, and can fill a gap in any sprint.

| Capability | What it waits for |
|---|---|
| Imposition in the print dialog | Sheet composition lifted into `pdfcer-print` so both shells share one implementation, including the mutual-exclusion guard that is CLI-local today. A control before that is an affordance for something that cannot happen. |
| Insert blank page | A size-and-count dialog. The engine inserts blank pages already. |
| Unencrypted-wrapper warning | A surface. The engine can tell that an otherwise unencrypted document carries encrypted embedded files; nothing on screen says so. |

Two capabilities are **built and undriven** — they exist, they are registered,
and no check in `tools/ui-verify` drives them. Until one does, a UI change is
not done:

- Page export to PNG, JPEG, SVG and EMF.
- Sheet resize, which changes the **paper** and does not move, scale or reflow
  one byte of page content — an A1 drawing put on A4 is cropped, not shrunk, and
  the dialog states that rule and measures the overhang before anything is
  committed.

List what the harness actually registers rather than trusting this paragraph,
and rebuild it first, because a stale harness prints nothing and an empty
listing reads as an empty roster:

```sh
cargo run --release -q -p ui-verify -- --list
```

---

## Rendering — what remains, in priority order

1. **A display list, built once and replayed at any scale** — engine side. A
   zoom change currently re-walks every operator on the page although only the
   transform moved; replaying a display list turns a zoom re-render into fill
   alone. It is a larger win than more cores, it applies at every zoom, and it
   composes with parallel fill.
2. **A reusable parsed handle** — engine side, and **deep zoom via per-viewport
   regions should not start before it is scheduled**. Without it, second and
   subsequent renders of a page pay full parse cost, and switching to regions
   trades the smooth pan the operator values for a gesture-length stall.
3. **A thread pool for thumbnails and adjacent-page prerender** — shell side,
   and it needs no interpreter change. Pages are independent; the render worker
   is single-slot by design and a pool is a different structure beside it.
   Adjacent-page prerender is cheap, independent of render strategy, and makes
   paging through a sheet set feel instant.
4. **Find off the dispatch path** — shell side. Find cancels the render worker
   and then runs the whole-document scan synchronously in dispatch
   (`crates/pdfcer-gui/src/find/mod.rs`). On a large document that is a visible
   stall, and it has nothing to do with rasterisation.
5. **A maximum zoom derived from measured performance** — not from `f32`
   numerics, which hold sub-pixel accuracy three orders of magnitude past any
   plausible viewing zoom, and not from the pixmap allocation guard.
6. **Anchor decimation for ink annotations** — every point of every stroke is an
   anchor today, so a dense stroke makes node-grab the hot spot. It resurfaces
   as selection gets richer.

Two controls that were once proposed here now exist as preferences: a
raster-scale quality multiplier, and the zoom settle delay. Three others do not
exist and are not merely unbuilt — a render-strategy radio has no tiled path to
select, an antialias control has no engine knob to turn, and a floating-panel
setting has no floating mode in the dock.

Processes are not on this list. They buy crash isolation, not speed: this is one
binary, threads share memory for free, and processes would ship a
multi-megabyte pixmap across a pipe per render.

---

## Explicitly not on this roadmap

Named so the omissions read as decisions rather than oversights.

| Not doing | Why |
|---|---|
| **A Home tab** | It would mirror commands across tabs. The quick-access toolbar and the status bar carry the shortcut role instead, which is where undo and redo live. |
| **Automatic reflow on edit** | Reflow would invent line breaks the file never stated. Make it reachable, not silent. |
| **Tiled rendering** | Cancelled on measurement: region cost is area-independent, so a tile ring is a regression rather than a win. |
| **JavaScript execution** | Standing refusal. Detection is a different thing and ships: a document that would reach outside itself says so on open, off-canvas. |
| **Provisional styling painted on the canvas** | The canvas shows what the saved file will show. Disclosure goes off-canvas. |
| **i18n and CJK chrome coverage** | English-only, no locale detection. A real ceiling on adoption, not a defect, and not this year's work. |

---

## Open questions

Each needs an operator ruling. Each names what it blocks.

**Document comparison — build it, or rule it out of scope?** It is the feature
an AEC reviewer asks for first and it is a large build. *Blocks:* a phase of its
own and probably a view mode of its own. Nothing else waits on it, which is why
it can sit here unanswered without stalling anything.

**How much of cross-run text editing?** Is *edit one run at a time, refused in a
sentence when the caret lands on a seam* an acceptable resting state for another
year, or is this the thing that has to be right? *Blocks:* whether a multi-run
edit request is filed with the engine and scheduled at all, and therefore
everything else about multi-run editing.

**Does the operator get to choose a re-wrap width, alignment and leading?** The
engine's reflow request carries all three as overrides and this shell passes
none of them, so a re-wrap uses the block's own extent, an inferred alignment
and a measured leading. Those are defensible defaults, not choices the operator
made. *Blocks:* whether Format grows three reflow controls, and whether the
inferred alignment needs a visible readout.

**Is area takeoff worth an engine request?** There is no area dimension kind to
author, so unlike angular this cannot be reached from the shell. The alternative
is for the shell to author a polygon with a measured caption, which is a second
measurement model beside ce dimensions and dilutes the dimension-group model.
*Blocks:* the whole Measure quantity group, which is absent as a group because
every command in it is absent.

**Which scale does a ce-dimension tolerance apply to?** A dimension group carries
a shared scale and a drafting standard; a tolerance stated in drawing units and
one stated in page units differ by that scale, and the wrong choice prints a
plausible number. *Blocks:* tolerance authoring in the dimension-group editor.
