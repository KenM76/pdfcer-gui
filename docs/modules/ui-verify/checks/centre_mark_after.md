# `a_placed_dimension_takes_and_drops_a_centre_mark`

**Asks:** O461. A centre mark can be added to or removed from a radius or
diameter ce dimension after it is placed, and its extension lines turned on.

**Drives:** on `hole-in-a-big-object.pdf`:

1. Measure the r=30 hole with the radius/diameter tool and place the label.
2. Select the placed ce dimension.
3. In Properties, press *Centre mark*. If the tool placed a mark already, this
   press removes it, so it is pressed once more.
4. Press *Extension lines*.
5. Drag *Angle* 30 px to the right.
6. Press *Centre mark* again to remove the mark.

**Passes when:**

- Each press traces `dimension-centre-mark`, in turn `on=1 extension=0`, then
  `on=1 extension=1`, then `on=0 extension=0`, and each is followed by a
  `set-centre-mark` commit.
- `properties.dimension.centre-mark.extension` is drawn after the mark is
  added.
- The angle drag raises exactly one `dimension-centre-mark` request, with a
  non-zero `angle=`, and exactly one commit: a drag is one undo entry, not one
  per frame.

**Why the second condition.** The extension switch is drawn only while the
stored kind carries a mark, so it shows the stored state rather than the press.
A panel that traced the press but sent no mark fails on it; that was the
falsification.
