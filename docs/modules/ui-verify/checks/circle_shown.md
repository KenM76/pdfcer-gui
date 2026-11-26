# `a_dimensions_circle_can_be_hidden`

**Asks:** O461 — the circle a radius or diameter ce dimension draws is a part
of its own, which can be hidden.

**Drives:** on `hole-in-a-big-object.pdf`, the radius/diameter tool measures
the r=30 hole from three rim picks and places the label. Escape leaves the
tool; a click on the label selects the ce dimension. The Properties panel's
*Draw the circle* switch (`properties.dimension.circle`) is pressed twice.

**Passes when** the first press traces `dimension-circle shown=0` and the
engine commits `set-circle-shown`, and the second traces `shown=1` and commits
again.

**Why two presses.** The switch draws the stored `show_circle` each frame. Had
the first edit not been kept, the second press would start from *shown* again
and trace `shown=0` a second time, so the pair tells a stored change from a
traced intention.

**Not measured:** pixels. The fixture's own hole lies under the dimension's
circle, so the rim's ink is the same with the circle hidden or shown.
