# `ui-verify/checks/markup_band`

`the_format_tab_restyles_a_selected_mark` — the Format ▸ Markup band, driven
from a drawn shape all the way to a thicker line on the page.

# The surface


Six controls now sit there. This check drives **one** of them end to end and
asserts the presence and the *absence* of the rest.

# ★★★ Seven links, and no test in the workspace observes two of them joined

| # | link | why a unit test cannot see it |
|---|---|---|
| 1 | a drawn `/Square` **selects** on a click | `selection::annot` hit-tests a `/Rect` through two coordinate spaces, against a canvas only the running program has laid out |
| 2 | the selection publishes `selection.markup_restylable` | a `ConditionSet` is recomputed per frame from live state; `manifest::format`'s test asserts the item **carries** the condition, never that anything satisfies it |
| 3 | the contextual **Format tab appears** | the shell decides tab visibility from the condition set; the application never asks |
| 4 | the six items **draw**, and the ones that do not apply draw *nothing* | `markupband::draw` returns `None` for an unknown kind and `endings` returns `false` for a subtype with no `/LE` — the difference between a control that is absent and one that is greyed is **pixels**, and R9 says which it must be |
| 5 | a drag on the width field commits **on release** | `DragValue::drag_stopped`, a property of a real pointer gesture across a real widget |
| 6 | the commit reaches `EditSession::set_markup_style` | `app::actions::apply`'s routing, over a parked operand the renderer put down |
| 7 | the regenerated `/AP` is **repainted** | the page raster's invalidation, then `pdfcer-render`, then the compositor |

★★ **Link 4 is the one with no other oracle at all.** `visible_when` in a
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

★★★ **Step 6 is two assertions because they fail separately.** A build whose
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
