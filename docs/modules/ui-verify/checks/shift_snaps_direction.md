# `ui-verify/checks/shift_snaps_direction`

`shift_snaps_a_drawn_direction` — holding Shift while placing a polygon
corner puts it on the nearest 15° direction from the previous corner, and the
lock is announced.

# What it drives

`dimension-scaled.pdf`, `mode.review,markup.polygon` armed. One plain click at
`START`, then two Shift-clicks aimed `REACH` points away at 7° and 52° from the
corner before. Both bearings are 7° off a step, so an unsnapped vertex is far
outside `TOLERANCE_DEG` and cannot pass.

# What it asserts

| | oracle | what the wrong build does |
|---|---|---|
| 1 | exactly `i + 2` `markup-vertex` lines after the `i`-th Shift-click | a click that placed nothing leaves the previous corner last, whose bearing from itself reads 0° and would pass 2 |
| 2 | the bearing of each landed `x, y` from the previous landed corner is within 0.2° of a multiple of 15° | lands at the aimed 7° / 52° |
| 3 | a `constrain lock=Direction` line after the mark | the snap happened and the status row was never told |

Bearings are measured in PDF user space from the trace; a 15° step survives
the canvas mapping's uniform scale and Y flip, so the space does not change
the verdict. Each segment is measured from the previous *landed* corner,
because that is the anchor `markup::vertex::aim` snaps from.

# What it does not cover

The Arrow drag and measure picks share `constrain::direction` but are not
driven here; they are unit-tested (`canvas::constrain_tests`,
`measure::resolve`). On measure picks an object snap outranks the direction
snap, so a driven check there needs a point with no snap candidate, or Alt.
