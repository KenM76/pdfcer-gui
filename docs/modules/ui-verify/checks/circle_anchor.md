# `a_dimension_follows_a_circle_and_measures_to_its_far_edge`

**Asks:** O461. A ce dimension can be placed to the circle a radius or diameter
ce dimension draws, follows it, and can measure to its centre, near edge or far
edge, as an arc condition does in the reference CAD class.

**Drives:** on `hole-in-a-big-object.pdf`:

1. Measure the r=30 hole with the radius/diameter tool and place the label.
2. With the linear tool, pick the hole's rim at 45° (clear of its quadrant
   nodes, so the pick is the ce dimension's rim rather than the drawing's
   path), a point level with it, and a placing point below.
3. Select the linear ce dimension by its label.
4. In Properties, press *Measure to: Far edge*, then *Detach*.

**Passes when:**

- The linear commit's `anchors=` carries `0:0:rim-min`: point 0 follows the
  near edge of dimension 0.
- *Far edge* traces `dimension-anchor-arc ... condition=rim-max` followed by a
  `set-dimension-anchor` commit.
- *Detach* traces `dimension-anchor-detach` followed by a commit.

The *Far edge* region is drawn only for a point the engine reports anchored to
a rim (`EditSession::dimension_anchors`), so reaching it is itself the witness
that the anchor was stored.
