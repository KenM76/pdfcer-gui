# `ui-verify/checks/markup_node_edit`

`a_markup_shapes_nodes_can_be_edited` — the operator's *"I also can't edit
or delete nodes of a markup shape once it is drawn"*, for the half that is
a **markup shape** rather than a ce dimension.

## Why this check and not the sixteen unit tests beside the feature

`canvas::annotnodes::tests` asserts the arithmetic, the subtype table and
the engine's own floor, against a real `EditSession`, and **every one of
them would pass on a build where the operator can do none of this.** This
project once had eight green tests while the feature performed 1 of 14
steps; R1 is the rule that came out of it, and this file is what discharges
it here.

Seven links stand between `annotnodes::resolved` and a hand on a mouse, and
not one is observable in process:

| # | link | why a unit test cannot see it |
|---|---|---|
| 1 | the **Polygon markup tool** arms in Review | `Capabilities::for_mode` reads the real manifest, and the mode is entered by clicking a segment |
| 2 | four clicks become a four-vertex `/Polygon` in the file | `markup::vertex` accumulates in `egui::Memory` and commits on a double-click |
| 3 | a click **selects** the shape it just drew | `selection::annot` hit-tests a `/Rect` resolved through two coordinate spaces |
| 4 | `canvas.markup-node.N` is **painted where the hit test looks** | the anchor's position is the end of a page → canvas → screen conversion; only the running application knows it |
| 5 | the press classifies as `DragKind::MarkupVertex` and **outranks** the body and the eight resize grips | `gesture::press_kind`'s precedence, over flags `canvas::pressing` resolves from a screen position |
| 6 | `Ctrl`+`Shift` survives the OS → winit → egui path **for the whole drag** | `Driver::press_held`'s note: a modifier applied and undone inside one frame's event batch |
| 7 | the engine accepts it, the `/AP` is re-baked, and **the pixels change** | only a real edit followed by a real render can show this |

## What it asserts that a trace alone cannot: the PIXELS

Step 7 is the one this check was written for. `move-annotation-vertex` in
the trace says the verb ran; it does not say the **drawing changed**. A
reshape rewrites three things — the `/Vertices` array, the `/Rect`, and the
baked `/AP` stream — and the engine's own note is that a shell writing only
some of them *"looks right in your canvas, right in a screenshot, right in
pdfcer, and is reconstructed in the old place by the next viewer"*. The
inverse failure is just as reachable here: an edit that lands in the model
and never reaches the raster leaves the operator dragging a node and
watching nothing move.

So the shape is dragged **out of its own bounding box**, into paper that was
blank before, and the ink there is counted before and after.

**The selection is CLEARED before the "after" capture, and that is the
whole honesty of the pixel assertion.** A selected shape draws its outline
and its node anchors, which are ink, at exactly the place the drag ended —
so a build that moved nothing but drew an anchor at the pointer would put
ink in the sampled box and pass. Deselecting first leaves only what
`pdfcer-render` drew from the annotation's own appearance stream.

## The fixture is PINNED and `--pdf` is ignored

`fixtures/four-pages.pdf`, and the reason is the pixel assertion above: it
needs a region of page that is **blank before the drag**, and "blank" is not
a property a sweep's `--pdf` can promise. On the operator's own CAD sheet
the destination box would already be full of ink and the assertion could
neither pass nor fail honestly. The check says so in its notes when a
`--pdf` was supplied and thrown away, because a sweep that silently ignored
a flag is indistinguishable from one that honoured it.

## The shape, and why a square

Four nodes. A `/Polygon`'s floor is **three**, so four is one above it —
which is what lets a single traced shape exercise the whole sequence and
reach the boundary:

```text
4 nodes  → drag node 1                → 4 nodes, moved   (and the pixels change)
4 nodes  → Ctrl+Shift-drag node 1     → 3 nodes          (a node is DELETED)
3 nodes  → Ctrl+Shift-drag node 1     → REFUSED, and the refusal is SHOWN
```

The third step is the operator's actual complaint in its purest form: a
gesture that cannot be honoured. It passes only if the status bar's `⊗` slot
is on screen — `status-group:decline` is published as a `ui-rect` on the
frame it draws, and its **absence is the failure**, because a refusal
nobody is told about is the founding defect of this project.

## Item notes

### `const MODE`

Driving this in Edit would exercise the same code with the interesting
half of every gate short-circuited: `press_kind`'s markup-node rung is
gated on `author_markup` precisely so it fires where `edit_content` does
not, and Edit has both.

### `const DRAW_EVENT`

Note how close this is to the node-editing lines below, and that the
closeness is exactly why they are spelled `markup-node-*`. This event
belongs to `canvas::markup::vertex` and has since polygons became
authorable; a node-move line under the same first token would make
`Trace::last` return whichever came later, and a check asserting a node
MOVED would read a line about a node being PLACED.

### `const ENGINE_MOVE`

Distinct from [`SHELL_MOVE`] deliberately, and asserting both is the
point: one says the gesture was understood, the other says the document
changed. A check that read only the first could not tell a shell that never
asked from an engine that refused.

### `const DESTINATION`

**Upper right, and the first run is why it is not lower right.** The
first draft aimed at `(0.80, 0.15)` and the "before" box came back holding
423 ink pixels of 1,406 — because `four-pages.pdf`'s page 1 carries a
coloured **title block** in exactly that corner. The assertion still passed,
on a delta of 28 pixels against a floor of 423, which is a measurement one
stray antialiased edge could have made either way. ⇒ **Read the run's own
capture before believing a pixel assertion.** The title block was plain in
it; only the number hid it.

### `const INK_DELTA_FLOOR`

Four pixels, and the reasoning is `InkReport::is_text`'s: one or two
pixels either way is antialiasing on an edge that did not move, while a
2 pt stroke crossing a box contributes a run. A strict `>` on a raw count
would let noise decide the verdict, and this project's standing rule is
that when a measurement runs out you read something else rather than
widening a tolerance — so the fix here is a floor with a stated reason plus
a SECOND, opposite measurement below, not a looser comparison.

### `const SAMPLE_HALF_FRACTION`

A fraction and not a constant in points, and the first run is why: 22 pt
on this fixture at fit-page zoom is an **8 x 9 pixel** window, which is too
few pixels for `ink_run_into` to say anything with. Small enough that the
square's original edges are nowhere near it — the nearest is a quarter of
the page away — and large enough to contain the corner two edges now meet
at, even after a snap has nudged it.

### `fn drag_holding`

**Held across the press, the walk and the release, and that is not
politeness.** `Driver::press_held`'s own note records the finding: a
modifier that goes down and up inside one frame's event batch can be applied
and undone before the event it was meant to carry is dispatched, because
modifier state reaches egui through winit's `ModifiersChanged`. A harness
that pressed Ctrl just before the button would produce a plain drag and
report *"the node was moved, not deleted"* about a perfectly working build.

It also matches what `canvas::dimdrag::intent` actually does: it reads the
modifiers **live on every frame**, so a Ctrl released half way through turns
the gesture back into a move and the preview says so on that very frame.
Holding it throughout is the only way to drive the gesture the operator's
hand makes.

### `fn ink_pair`

**Two boxes from ONE capture, and the pair is the assertion.** A single
"ink arrived at the destination" reading is satisfied by anything that puts
dark pixels there, including a build that drew a stray anchor. A single "ink
left the origin" reading is satisfied by a build that simply stopped drawing
the shape. Requiring **both, in opposite directions, in the same frame** is
what makes the pair describe a node that MOVED rather than one that appeared
or vanished.
