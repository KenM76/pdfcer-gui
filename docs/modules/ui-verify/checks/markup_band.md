# `ui-verify/checks/markup_band`

`the_format_tab_restyles_a_selected_mark` — the Format ▸ Markup band, driven
from a drawn shape all the way to a thicker line on the page.

# The surface


Six controls now sit there. This check drives **one** of them end to end and
asserts the presence and the *absence* of the rest.

# Seven links, and no test in the workspace observes two of them joined

| # | link | why a unit test cannot see it |
|---|---|---|
| 1 | a drawn `/Square` **selects** on a click | `selection::annot` hit-tests a `/Rect` through two coordinate spaces, against a canvas only the running program has laid out |
| 2 | the selection publishes `selection.markup_restylable` | a `ConditionSet` is recomputed per frame from live state; `manifest::format`'s test asserts the item **carries** the condition, never that anything satisfies it |
| 3 | the contextual **Format tab appears** | the shell decides tab visibility from the condition set; the application never asks |
| 4 | the six items **draw**, and the ones that do not apply draw *nothing* | `markupband::draw` returns `None` for an unknown kind and `endings` returns `false` for a subtype with no `/LE` — the difference between a control that is absent and one that is greyed is **pixels**, and R9 says which it must be |
| 5 | a drag on the width field commits **on release** | `DragValue::drag_stopped`, a property of a real pointer gesture across a real widget |
| 6 | the commit reaches `EditSession::set_markup_style` | `app::actions::apply`'s routing, over a parked operand the renderer put down |
| 7 | the regenerated `/AP` is **repainted** | the page raster's invalidation, then `pdfcer-render`, then the compositor |

**Link 4 is the one with no other oracle at all.** `visible_when` in a
*menu* did nothing for the whole of this project's life until 2026-09-06 —
`menu::plan::resolve` never read `Item::visible_condition()`, so every row
meant to vanish was **greyed** instead, R9 inverted, with prose at each site
describing behaviour that was not happening. The commit that found it says
why no test could: *"every one asked the model rather than the resolution."*
The arrowhead control here is the same shape one surface over — the manifest
deliberately gives it **no condition** and lets `markupband` decide its own
absence from the value it read — and the only way to tell an absent control
from a greyed one is to ask how much space it took.

# What it does, and the two oracles it ends on

1. Arm Review and the Rectangle tool through `PDFCER_DIAG_INVOKE`; draw a
   shape with one drag.
2. Photograph the strip along its top edge — **the thin line**.
3. Put the pen down, click the shape, confirm `annot-select`.
4. Click the Format tab. Assert the four controls a `/Square` has are drawn
   and **substantial**, and that the arrowhead chooser is **not**.
5. Drag the width field to its ceiling.
6. Assert `set-markup-style` reached the engine — *and* photograph the same
   strip again with nothing selected: **the line is thicker**.

**Step 6 is two assertions because they fail separately.** A build whose
parked operand never reaches `apply` traces nothing and paints nothing. A
build that restyles the dictionary and never re-bakes the appearance — or
bakes it and never invalidates the page raster — traces `set-markup-style`
**perfectly** and paints the old line. That second failure is this project's
signature shape, and `markup_node_edit` records the same pair for the same
reason: *"the engine's own note is that a shell writing some of the three
looks right in every renderer."*

# Calibration

```text
--pdf fixtures/a1-titleblock.pdf --doc-point 0,300,500
```

⚠ `--doc-point` is **0-based** and this check does not aim with it — it
places its shape in page fractions. Any single-page fixture with blank paper
across the middle third serves; the check asserts that emptiness and SKIPs
rather than measuring a fixture's own linework.

# Every way this reports SKIP

* no binary, no `--pdf`, `--no-input`;
* the canvas is not showing page 1;
* the fixture has its own content under the strip, so a thickness reading
  could not be attributed to this check's mark;
* the rectangle tool authored nothing, or the shape could not be selected —
  both are `dragging_a_markup_moves_it`'s subject and both are steps *before*
  the one under test;
* the width field was drawn but the drag did not change its value, so there
  was no restyle to observe.

## Item notes

### `const RESTYLE_EVENT`

The bare name and not `-applied`: `apply::vector_edit` names the edit
itself, and a refusal traces `set-markup-style-refused`, which the failure
message points at.

### `const EXPECTED`

A `/Square` and not "a markup", because which controls apply is a property
of the subtype: a highlight has no border to widen, an arrow has no interior
to fill. This check draws a rectangle, so this is the list for a rectangle.

### `const ABSENT_FOR_A_SQUARE`

`/LE` is meaningful for a `/Line` alone. `manifest::format` deliberately gives
this item no condition and lets `markupband::endings` decide its own absence
from the value it read — *"a control that decides its own absence from the
value it reads, in the one place that has read it"* — so the only way to
check that decision is to look at how much room it took.

### `const MIN_CONTROL_EXTENT`

# Why "declared" is not enough, in both directions

`markupband::draw` publishes its `ui-rect` from the response of an
`add_enabled_ui` **whether or not the closure drew anything**, so an absent
control still declares a rect — a degenerate one. Presence and absence are
therefore both statements about *area*, not about the name appearing in the
trace, and a check that only asked "is it declared" would pass on a build
where every one of the six drew nothing.

The number is a floor on the smaller of the two dimensions rather than on the
area, because the failure this guards against is a control laid out with **no
usable extent in one axis** — which is the redaction panel's apply button
shipped below the bottom of its own pane, a defect this project has already
had once. 10 logical px is under half the height of the smallest control in
the band and an order of magnitude over the ~0 an empty `add_enabled_ui`
allocates.

### `const WIDTH_DRAG_PX`

`DragValue::speed(0.1)` means 0.1 pt per pixel, so 200 px is +20 pt against a
field whose range is 0.25–12 pt. **Deliberately past the ceiling**: the
commit is compared against the range's own maximum rather than against an
arithmetic prediction, so this check does not have to track the speed
constant, and a clamp that stopped working would show up as a value over 12
rather than as a check that had to be re-tuned.

### `const GRAB_FRACTION`

A third. `markupband::FIELD_WIDTH` is 46 logical px and a two-digit
`DragValue` with a ` pt` suffix is comfortably wider than 15, so the left
third is inside the spinner on any theme this shell has had. See the drag's
note for what the centre cost.

### `const MAX_WIDTH_PT`

Spelled here rather than imported, like every other constant in this crate:
the harness must be able to fail against a binary built from a different
tree, and an import would make the check assert that the application agrees
with itself.

### `const MIN_THICKENING`

# Why 1.5 and not 6

The width goes from 2 pt to 12 pt, which is **six times** the line — and the
ink count does not go up sixfold, because the strip is a fixed box and a
thicker line fills more of its height but no more of its length. Measured on
this fixture at fit-page zoom (20 %), a 2 pt line covers about one row of the
strip and a 12 pt line about two: the honest expectation is a doubling, not a
sextupling.

⇒ **A floor derived from what the geometry can actually produce**, not from
the ratio of the numbers that were typed. A check demanding six times would
go red on a working build, and the session that met it would go looking for a
rendering defect. This project has already spent a morning on exactly that
mistake in the other direction (`markup_rectangle`'s note on three candidate
palettes).

1.5 is comfortably under the measured doubling and comfortably over the ±2
pixels an antialiased edge moves by, on a strip that starts with tens of ink
pixels.

### `const STRIP_HALF`

Four times `markup_palette`'s, and the difference is the subject: that
check reads a *colour* and wants as little paper in the box as possible, this
one reads a *thickness* and needs room above and below for the line to grow
into. A strip only as tall as the thin line would saturate at the first
widening and report nothing about the rest.

### `const PARK`

Blank paper in a corner, and `markup_palette`'s first run is why: a
pointer left on the shape pops the *"No note has been written on this
markup."* tooltip, a floating dark panel that lands on the very box being
measured and reads as ink. A driven check photographs the pointer as well as
the program.

### `const ELSEWHERE`

Far from the shape and far from the strip, so it can neither re-select the
mark nor add ink to what is about to be measured. Distinct from [`PARK`] on
purpose: the click lands here and the pointer then moves on, so the tooltip
question and the deselection question do not share an answer.

### `fn edge_ink`

The mapping is re-derived by the caller for every reading rather than cached:
a cached mapping is a stale coordinate, and a stale coordinate is
symptom-identical to a broken conversion — the confusion behind one
filed-then-retracted defect in this codebase.

### `fn left_part`

See the drag's own note: a band item's published rect is the widget **plus**
whatever padding the renderer allocated to make the row line up, and the
padding is always on the right.

### `fn an_empty_control_is_not_substantial`

The property the whole of link 4 rests on: `markupband::draw` publishes
a `ui-rect` for an item whose closure drew nothing, so *declared* and
*drawn* are different questions and this is what tells them apart.

### `fn the_strip_lies_on_the_top_edge_and_clear_of_the_corners`

A check aimed at the wrong box produces an articulate failure message
about nothing, which is this project's commonest wasted afternoon. The
arithmetic that decides where this one looks is therefore asserted rather
than eyeballed.
