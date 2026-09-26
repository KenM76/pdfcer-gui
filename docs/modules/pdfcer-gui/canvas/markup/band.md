# `canvas::markup::band` — the two-point rubber band

Rectangle, Ellipse, Arrow and Highlight: **press, drag out a shape,
release.** One of the four gesture families [`super`]'s header tabulates;
[`super::vertex`] and [`super::ink`] own the other shapes.

## The seam against [`super`]

[`super`] holds *what a markup is* — the kinds, the geometry, `spec`,
`action`, the pen. This file holds *how this family is gestured*: the
canvas→page conversion for two points, the one function that touches the
frame, and the band that is drawn while the button is down.

**The seam is a subject and not a line count**, and the test is that the two
sides change for different reasons. A new markup *kind* changes [`super`] —
a variant, an `rgb` arm, a `spec` arm — and does not touch this file unless
it is band-shaped. A change to *how a band is drawn* — a snap, a modifier
that constrains the aspect ratio, a different preview — changes this file
and nothing in [`super`].

## The band draws the shape it is about to author, not a box round it

Rule 4's pre-commit affordance, applied literally. [`draw_preview`] carries
the argument, including what an ellipse previewed as its inscribed *circle*
costs.

## A click with no drag places nothing

[`super`]'s header carries that decision in full, including the two reasons
a 120 × 60 default box and a 4-point page-space threshold are both
deliberately absent. The mechanical half of it lives here: [`drag`] is
reached only from `GestureOutcome::Markup`, which only a real drag produces,
and a zero-extent drag is refused by [`super::action`].

## Item notes

### `const HEAD_LEN_PX`

Screen-space, deliberately: the head is part of the *cursor*, and a head that
shrank to nothing at 25 % would stop saying which end of the band is the
head — which is the one thing this preview exists to say. The committed
annotation's own `/LE` head is drawn by the appearance stream at whatever
size the engine chooses; this is not a promise about that size, it is a
statement about direction.

### `fn arrowhead`

Returns an empty array's worth of coincident points for a zero-length band,
which draws nothing — a head with no direction to point in must not be
invented from a normalised zero vector (which would be NaN).

### `fn the_markup_lands_where_the_drag_was_and_not_at_the_page_centre`

The operator: *"they just drop things into the center of the pdf
window."* It is written as a **magnitude** assertion against the dragged
corners and, separately, as a statement that the result is nowhere near
the media-box centre, because a test asserting a relation rather than a
magnitude is satisfied by any absurdity in the right direction. "The
shape is on the page" passes on the defective build; "the shape's
corners ARE the corners dragged" cannot.

### `fn the_same_drag_authors_the_same_page_coordinates_at_every_zoom`

This is the markup gesture's form of
`moving::a_drag_between_two_page_points_moves_the_same_distance_at_every_zoom`,
and it is the stronger of the two statements: a move only has to be the
same *displacement* at every zoom, while a markup has to land on the same
*absolute* page coordinates.

### `fn a_click_places_nothing_and_the_degenerate_drag_it_resembles_is_refused`

The module docs' decision, pinned from both ends: the gesture machine
raises `Click` (not a `DragKind`) for a press-and-release under egui's
threshold, and if a zero-extent drag does arrive it commits nothing.
Without the second half, "a click places nothing" would rest on egui's
behaviour alone.

### `fn a_non_band_kind_is_refused_by_the_band_gesture`

The guard on the first line of [`drag`], asserted because its absence is
silent: an ink drag routed here by mistake would draw a rectangle between
the stroke's first and latest points and would **author** one on release
— a perfectly valid `/Square` that the operator did not draw, over the
region their freehand stroke happened to span.

### `fn the_preview_arrowhead_sits_at_the_head_whichever_way_the_drag_went`

Asserted as a distance, not as a side: both barbs must be within a barb's
length of the head and nowhere near the tail. A "the head is drawn"
assertion would pass on an implementation that drew it at the wrong end.

### `fn the_highlight_wash_is_the_pen_colour_with_an_alpha`

Compared as the *whole* `Color32` against a value rebuilt from the pen,
rather than channel by channel, because `egui::Color32` stores
**premultiplied** components: `r()` on a translucent colour returns the
multiplied byte, so a per-channel comparison against the opaque pen
fails for a wash that is completely correct — a false failure against
working code, which is why the whole value is compared.
