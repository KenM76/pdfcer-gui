# `ui-verify/checks/escape_finishes_a_run`

Escape during a multi-point run finishes it with the points placed so far,
rather than discarding them.

- `escape_finishes_a_markup_run` — Markup ▸ Polygon, three corners, Escape.
- `escape_finishes_a_measure_run` — Measure ▸ Length, three picks, Escape.

# What it drives

`dimension-scaled.pdf`, with the command list `mode.review,markup.polygon` or
`mode.review,measure.length`. Three clicks at `CORNERS`, which are clear of the
fixture's ce dimension so no pick snaps onto it. The trace mark is taken after
the clicks and before Escape, so everything asserted was caused by the press.

# What it asserts

Markup:

| | oracle (after the mark) | what the discarding build does |
|---|---|---|
| 1 | `canvas-escape outcome=FinishedMarkupVertexRun` | `AbandonedMarkupVertexRun` |
| 2 | `markup-finish via=escape`, `markup-commit kind=Polygon vertices=3`, and the engine's `add-markup` | none of them: nothing reaches the engine |
| 3 | no `canvas-tool` line | a press that finished the run *and* put the pen down breaks the ladder's one press, one effect rule |

Measure:

| | oracle (after the mark) | what the discarding build does |
|---|---|---|
| 1 | `canvas-escape outcome=FinishedMeasureRun` | `AbandonedMeasurePick` |
| 2 | `measure-finish via=escape kind=PathLength` and the engine's `add-dimension` | neither |

Assertion 1 alone could be satisfied by a rung that traced the right outcome
and committed nothing, so 2 requires the engine verb.

# What it does not cover

- Polyline, Cloud, Perimeter, Area and Circular share the same Escape path and
  are unit-tested in `canvas::markup::vertex`, `canvas::measure` and
  `canvas::keys::tests::escape_ladder`.
- The fallback, where Escape with too few points abandons the run, is
  unit-tested only.
- Linear, Scale and TwoLine picks, and band and ink drags, still discard on
  Escape; `escape_still_discards_a_linear_pick` holds that.
