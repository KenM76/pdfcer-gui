# `ui-verify/checks/model_measure`

`the_3d_viewer_measures_from_points_on_the_model` — **points picked on a 3D
model make a measurement the engine values, shown as it is measured, and one
kept is listed and drawn when the model is next opened.**

Off-screen, embedded, scripted pointer, no OS input. The fixture is
`model_views_saved`'s: one page whose PRC stream saves one view. Its parts are
flat and leave gaps, so picking walks a grid of spots and a miss only moves on.

Oracles, from the trace:

1. With one pick left, hovering the model traces `model-measure-preview`: the
   value shows before the last click.
2. `model-measure-result`'s value equals the distance between the two traced
   `model-measure-pick` points, to the trace's rounding.
3. *Keep on this view* moves `model-measure-saved` to 1, and a second from the
   same window to 2: the viewer's own write must not make it refuse the next.
4. Angle and Radius each finish from points on the model. A perpendicular
   between two points of this flat fixture has no length: the engine refuses
   it, and `model-measure-refused` is the oracle. The value of a perpendicular
   that has a length is pinned by a unit test.
5. Closed and opened again, `model-measure-drawn count=2`.
6. *Delete* on the options menu's Measure tab moves `model-measure-saved` to 1.

The pictures `model-measure.measured.png`, `.perpendicular.png`, `.angle.png`, `.radius.png`, `.reopened.png` and
`.tab.png` are kept for the drawing itself, which only a look can judge.
