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

### `struct Preview`

Returned by [`drag`] only while the pointer is down, and only when the
release would commit — the same "the preview describes something that will
actually happen" contract [`crate::canvas::moving::drag`] honours with its
ghost.

### `fn endpoints`

# Why two point conversions and no arithmetic of our own

[`viewer::canvas_to_pdf_space`] applies the renderer's own page transform —
the crop-box origin, the `/Rotate`, and the Y flip. Writing any part of that
out here would be a second derivation of the page transform, which is the
precise failure `viewer`'s header warns about: *"PDF user space is y-UP;
canvas and screen are y-DOWN. The failure is silent — the page looks perfect
until someone selects a line and gets a different one."* For a markup the
symptom is worse than a mis-selection, because it is written to the file: a
rectangle dragged over the title block lands mirrored about the page's
horizontal centre line, and the operator finds out after saving.

Unlike [`crate::canvas::moving::page_delta`] this maps **positions**, not a
displacement, so the transform's translation is *not* cancelled — which is
the whole point. A markup has an absolute place on the page.

Returns `None` for a page whose device transform cannot be inverted, which
is the same condition under which both halves of the `viewer` bridge
decline.

### `fn drag`

The **only** function here that touches the frame. It does one of two
things:

* [`Phase::InFlight`] — returns the band for [`draw_preview`] and changes
  nothing. Nothing is decomposed and nothing is re-rasterized: a markup drag
  hit-tests nothing at all, which is why `canvas::interact` deliberately
  leaves it out of the set of outcomes that need an object model. A preview
  over a 129,758-object drawing costs one stroke.
* [`Phase::Complete`] — converts both endpoints to page space and pushes
  exactly one [`Action::CommitMarkup`].

Returns `Some` only when a band should be drawn, and — as with the move
ghost — only when the release would actually commit. A drag with no page
under it draws nothing rather than a band that promises an annotation the
frame cannot author.

**The first line is a guard on the family**, and it is not defensive
clutter: `canvas::interact` routes a freehand drag to [`super::ink`] before
reaching here, so a non-band kind arriving means the routing changed and this
function's two-point assumption stopped holding. Drawing nothing is the only
honest answer available — a band drawn between an ink stroke's first and
latest point is a rectangle the operator never asked for and the release
would author it.

# Why the refusal is traced only on release

An in-flight drag is re-evaluated 60 times a second, and the `canvas-pointer`
lesson — fifty identical lines in nine seconds from a stationary pointer —
is what a per-frame refusal trace would reproduce. The release is one event,
and it is the one a harness reading the trace is asking about.

### `fn draw_preview`

# Why this is not `draw_marquee` with a different colour

Because a marquee and a markup band answer different questions. A marquee
asks *"what does this rectangle enclose?"* and is therefore always a
rectangle whatever it is about to select. A markup band asks *"is this the
shape you meant?"*, and the only way it can answer is by **being that
shape**: an ellipse drawn as its bounding box misstates the geometry by the
difference between a box and the ellipse inside it — 21 % of the area — and
an arrow drawn as a plain segment says nothing about which end the head is
on, which is the single most reversible property of the thing being
committed.

⚠ The near miss is `circle_stroke` at the *smaller* of the two
half-extents — the inscribed **circle** rather than the ellipse about to be
authored. On a wide drag the two differ by the whole aspect ratio, and the
operator releases expecting the circle they were shown.

# The colours are document colours, and that is why they are literals

Everything painted here is the pen — the colour and width that are about to
be written into the file. Reading it from [`egui::Visuals`] would be wrong in
the way `check-theme-colors.sh`'s own header describes: restyling the
application would change the colour of markup about to be committed, and the
change would only become visible after saving.

### `fn draw_text_marks`

`OPERATOR_REQUESTS.md` **O54**. The sibling of [`draw_preview`] for the
gesture that found text under it.

It uses `highlight_wash` — the identical colour the area band draws —
because they are one feature reached by one tool. A preview that changed
colour depending on whether the pointer had found text would tell the
operator they had switched tools when they had not.

Rectangles rather than quads: the quads a text sweep produces are already
axis-aligned per line in canvas space, which is what `TextSelection::
highlights` hands back. A rotated run is drawn by the committed appearance
stream, not by this — the same division `draw_preview` makes.
