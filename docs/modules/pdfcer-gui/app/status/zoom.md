# `pdfcer-gui/app/status/zoom`

## Item notes

### `fn readout_width`

# Why this is measured rather than declared

Because the thing it has to fit stopped being knowable at authoring time.
[`ZOOM_READOUT_WIDTH_PTS`] was written when [`crate::viewer::ZOOM_LADDER`]
topped out at 800 % and four characters covered every string the readout
could produce. O24 made the ceiling a **preference** whose top preset is
`1e12`, and `{:.0}` formatting turns that into `1000000000000%` — fourteen
characters. A constant cannot cover a range the operator chooses.

So the reserve is the galley width of the widest string this ceiling can
ask for, floored at the old constant.

# Why this is not the feedback loop that has bitten this project before

R128 and the fit-zoom defect were both *a measurement of laid-out content
fed back into the size of the thing that lays it out*, which oscillates.
This measures a **string that does not depend on the width** —
`zoom_percent(ceiling)` is the same text whatever the reserve turns out to
be — so there is no loop to close. The output is a pure function of
(ceiling, font), and both are stable across a frame.

The width changes only when the operator picks a different ceiling, which
is an explicit act in a popup, not something that happens under the pointer
while stepping.

`+ 2.0`: `Button::frame(false)` still lays out with the style's button
padding, and a galley measured to the pixel against a rect measured to the
pixel truncates on the last glyph under rounding. Two points is the
smallest allowance that is visibly never wrong, and it is stated here
rather than folded into the floor so that the floor keeps meaning
"the old reserve".

### `fn the_readout_can_be_asked_to_draw_far_more_than_four_characters`

`ZOOM_READOUT_WIDTH_PTS`' doc comment said 46 pt was *"wide enough for
four characters, which is the whole range `ZOOM_LADDER` can produce."*
This is that sentence turned into an assertion, and it fails: O24 made
the ceiling a preference topping out at `MAX_MAX_ZOOM_PERCENT`, and
`{:.0}` renders that as fourteen characters.

A test on the STRING rather than on the pixel width, deliberately: the
defect is not "46 pt is the wrong number", it is "the readout's content
outgrew what the number was chosen for". A width assertion would pin a
font metric and would have to be re-tuned whenever the face changed; the
character count is the durable statement.

### `fn group`

The readout is a label rather than a field: there is no action that sets
a zoom to a named value (see [`crate::text::status::zoom_percent`]), and
a text box in front of nothing is a placeholder. It is given a fixed
width so that stepping from `100%` to `75%` does not move the − button
out from under the operator's pointer.
