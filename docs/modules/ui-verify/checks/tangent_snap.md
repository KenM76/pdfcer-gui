# `the_linear_tool_snaps_tangent_to_a_circle`

**Asks:** O461. The circle a radius or diameter ce dimension draws offers a
tangent snap, as the reference CAD class's tangent arc condition does.

**Drives:** on `hole-in-a-big-object.pdf`:

1. Measure the r=30 hole with the radius/diameter tool and place the label.
2. With the linear tool, pick (420, 521.2), off the circle.
3. Hover the tangent point from it, (318.9, 472.9), pressing Tab until the
   shown snap is `Tangent`, at most eight times, then click.

**Passes when** a `measure-snap-marker kind=Tangent` is shown there.

The engine computes a tangent only from a previous pick, which the shell
passes with the snap query (`SnapConfig::with_from`). Without it the circle
offers its rim and quadrants and never a tangent; that was the falsification.
