# `canvas::handles` — eight grips plus move, and the cursor over each

`GUI_ROADMAP.md` Phase 1.3: *"Eight handles plus move, per the convention
every drawing tool shares. Cursor changes over a handle, over a movable
object, over the canvas."*

## Rule 4 says these are welcome, and says exactly why

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, fourth clause of
the disclosure rule:

> **A pre-commit affordance is not content marking.** A snap indicator, a
> hover highlight, a rubber-band, a selection handle — these are the
> *cursor*; they describe what is about to happen and they are welcome.
> What is forbidden is styling content that has **already been applied**
> as though it were pending.

So the grips are drawn, and nothing else is. No badge, no tint, no dashed
"provisional" layer over content, nothing that would make a screenshot of
the editing canvas differ from a screenshot of the same document saved and
reopened. The grips vanish with the selection because they are the
cursor's statement about the selection, not a property of the page.

## These are SCREEN-space rects, deliberately, and it is the one place

Everything else in `canvas/` past [`crate::canvas::mapping`] is page
space. A grip is the exception and must be: it is a **fixed number of
screen pixels**, because it is something the operator has to hit with a
mouse, and a grip sized in page units would be a 3-pixel speck at fit-page
and a slab the size of the object at 800%. It sits on the *output* side of
the boundary — the selection's bounds are converted to screen once, by
[`crate::canvas::mapping::PageMapping::rect_to_screen`], and the grips are
laid out on the result.

## What a grip drag does today, stated so it is not mistaken for an oversight

[`Grip::Move`] is live: a drag on the selection's body moves it, through
`EditSession::move_objects`.

The **eight resize grips change the cursor and consume the drag, and
perform no edit yet.** That is not a placeholder left in by accident, and
the reason is worth writing down rather than rediscovering: `pdfcer-core`
has `move_object`, `move_objects`, `move_subpath`, `move_node`,
`move_nodes` and `move_handle` — and **no scale or resize verb for a
vector object at all**. `GUI_ROADMAP.md` 1.2 (*"move and resize anything
carrying a `/Rect`"*, `FEATURES.md:208`) is the row that gives them one,
and it covers annotations, form widgets, redaction marks, links and ce
dimensions — objects whose size is a rectangle in the file rather than a
consequence of their path data.

Consuming the drag is the deliberate part. Without it, a drag that started
on a grip would fall through and become a **marquee**, so aiming at a
resize handle would silently replace the selection the operator was trying
to resize. Swallowing the gesture is the honest behaviour until the verb
exists.

## Item notes

### `fn pushed`

# The defect it inherits, and why it could not simply be skipped

[`grip_bounds`]'s own header records it: a 0.85 pt cell's grips land on
top of each other and on its body, so there is nothing to aim at. That
is not a property of *upright* selections — a 0.85 pt mark turned 30° is
exactly as unreachable — so a turned frame that skipped the push would
have re-opened the defect for precisely the annotations this change is
about.

The push is along the frame's **own** axes, not the page's, which is
what keeps a pushed turned frame a rectangle. Expanding an axis-aligned
bound instead would move the corners off the mark's diagonal and the
grips would no longer sit on the outline they are drawn against.

A degenerate frame (coincident corners) has no axes to push along;
`normalized` answers zero there and the frame is returned unchanged,
rather than becoming a NaN box that paints nothing anywhere.

### `fn a_narrow_box_drops_its_mid_edge_grips_and_keeps_its_corners`

22 keeps the property this test is actually about: below
`MIN_MID_GRIP_EXTENT_PX` (24) on the narrow axis, so North and South are
still dropped for piling onto the corners; above `MIN_BODY_STRIP_PX`
(20) across it, so a body survives and East and West are legitimately
offered. **Two rules, two thresholds, and a fixture between them is what
tests either one in isolation.**

### `fn every_resize_grip_pivots_about_the_opposite_point`

The property the whole resize rests on, asserted as a relation rather
than against a table of corners — a table would pass for a build whose
`pivot` returned `anchor` unchanged if somebody wrote the table from the
same wrong function.

The failure it forbids is specific and looks plausible: scaling about
the grip being dragged makes the shape grow *away* from the operator's
hand instead of towards it. It resizes; it is wrong; and it is the kind
of wrong that survives a screenshot.

### `fn an_inner_rung_offers_move_and_no_scale_handles`

The regression test for a defect found by driving: an anchor mark is
centred on its point, so an anchor at a corner of the object's bounding
box is half outside it — and the corner grip, with two points of grab
slack, covers exactly that spot. A drag from a selected corner anchor
raised no move at all, because the press had been claimed by the
north-west grip.

The operator's version is *"I can drag the middle nodes and not the end
ones"*, which reads as a broken hit test rather than as two features
competing for one pixel.

### `fn a_rotate_only_set_offers_the_handle_and_none_of_the_eight`

The combination [`GripSet`] grew a second field for, asserted rather
than assumed, because it is the one a build can get wrong in two
directions and look plausible in both:


The middle row — a press at a **corner** answering `Move` rather than
`NorthWest` — is the load-bearing one. `grip_at` gates the eight
separately from the ninth, and a build that gated them together would
pass the first and third assertions and fail only this one.

### `const GRIP_SIZE_PX`

Large enough to hit with a mouse without a steady hand, small enough that
eight of them around a modest selection do not obscure it. It is also the
*drawn* size — grip and target are the same square, which is what makes
"aim at the thing you can see" true rather than approximately true.

### `const GRIP_GRAB_SLACK_PX`

Small and asymmetric with the selection catch radius on purpose: a grip is
a visible target the operator is aiming at, so it needs far less
forgiveness than an invisible hairline does, and every point of slack here
is a point stolen from the body-drag region just inside it.

### `const MIN_MID_GRIP_EXTENT_PX`

Below three grip-widths the mid-edge grip would sit on top of its two
corner neighbours, producing an unaimable pile that looks like a rendering
fault. Corner grips are always offered — they are the ones that survive a
small box — so nothing is unreachable, there is simply less on screen.

### `const MIN_BODY_STRIP_PX`

# The defect this closes, measured

[`MIN_MID_GRIP_EXTENT_PX`] gates a mid-edge grip on its **own** axis, so it
cannot pile onto its corner neighbours. Nothing gated it on the
**perpendicular** one, and that is the axis a mid-edge grip eats into.

A form field of 160 × 20 pt at the operator's fitted 29.55 % is **47.3 × 5.9
px**. The box is wide enough for North and South, so both are offered — and
each reaches `GRIP_SIZE_PX / 2 + GRIP_GRAB_SLACK_PX` = **6 px** into a box
that is 5.9 px tall. **Dead centre of the field is inside its own North
grip.** An operator dragging a short field from the middle to move it gets a
degenerate resize instead, which the engine then refuses by name:
`resize-widget-commit … grip=North sy=-42.5314` → `edit-widget-refused …
rectangle has no area`. Found by a driven check whose own press landed there.

# Why this number

Two opposing mid-edge grips consume `2 × 6 = 12 px` of the crossing axis, and
a body worth aiming at needs at least a grip's width of its own. Below
**20 px** across, the mid-edge pair is withheld and the corners — which are
the grips that survive a small box, and the ones a resize actually wants —
are all that is offered.


This constant keeps its job: it is the width of body a box must have before
the grips can sit on its own edges, and [`grip_bounds`] is the function that
guarantees it by construction rather than by a filter.

### `enum Grip`

Named by compass point rather than by index, because an index would have
to be read against a table to know which corner it meant, and the cursor
mapping below is exactly such a table — written once, here.

### `fn cursor`

The diagonal cursors are *shared between opposite corners* — NW and SE
both read as `ResizeNwSe` — because that is what the cursor is
describing: the **axis of the resize**, not which corner is under the
hand. Every platform's own resize cursors work this way, and giving
each corner its own arrow would be a private convention the operator
has to learn.

### `fn is_resize`

This was `self != Self::Move`, and leaving it that way when
[`Self::Rotate`] arrived would have made the rotate handle **the ninth
resize grip**: `gesture::meaning` asks exactly this question to decide
between `DragKind::Resize` and everything else, so a press on the handle
would have scaled the object about a corner. It would have looked like a
deliberate feature and nothing in the suite asked about it.

The enumeration is deliberate rather than a negation for that reason: a
tenth affordance added later has to be classified rather than defaulting
into the resize family.

### `fn anchor`

[`Self::Move`] answers with the box's centre. It has no drawn square,
so the value is only meaningful as "the middle of the thing" — used by
nothing that paints, and defined rather than left as an `Option` so
every arm of the enum has an answer and a future caller cannot be
surprised by a `None`.

### `fn opposite`

[`Self::anchor`] answers where the grip *is*; this answers what it pivots
about, and the two are opposite corners. Dragging the south-east grip moves
the south-east corner and leaves the north-west one still — which is what
every drawing application does, and what the standing *"behave the way the
tools they already use behave"* tie-breaker asks for.

# Why it is a method here and not arithmetic in `canvas::resizing`

Because it is the same fact as `anchor`, mirrored, and the two must agree:
the ghost is drawn about this point and the commit is computed about it, so
a second spelling would be a preview and an edit that disagreed about which
corner stayed still — an object that jumps on release by exactly the box's
size.

A mid-edge grip pivots about the **opposite edge**, keeping the axis it
does not scale centred. `East` returns the west edge at the same y, so the
unscaled axis's factor of 1.0 leaves every point on it unmoved whatever y
this returns — but returning the mid-point rather than a corner keeps the
value meaningful if a future edit ever scales both.

[`Self::Move`] pivots about itself: it does not resize, and a caller that
reached here for it has already gone wrong. Returning the centre is the
harmless answer — a scale about the centre with factors of 1.0 is the
identity — rather than a panic in a frame that is trying to draw.
The grip **diagonally opposite** this one — the one whose anchor is this
grip's pivot.

Factored out of [`Self::pivot`] rather than duplicated in
[`GripFrame::pivot`], because "which grip stays still" is a fact about
the *enum*, not about the frame it is laid out in. Spelling it twice is
how a turned frame and an upright one end up disagreeing about which
corner a drag anchors to — an object that jumps on release by exactly
the box's size, which is the failure `pivot`'s own doc comment names.

[`Self::Move`] and [`Self::Rotate`] are their own opposite: neither has a
corner that must not move, and both already answer with the centre.

### `const ROTATE_STEM_PX`

Far enough that its grab area (the handle plus [`GRIP_GRAB_SLACK_PX`])
cannot overlap the north grip's, or the two would fight for the same press
and which one won would depend on the order they are checked in — the
failure `handles.md` H5's corollary is about. With a 7 pt handle and 2 pt of
slack on each, 20 pt clears both by a comfortable margin.

Screen-space, like every other number here (H3), so the handle sits the same
distance from the box at 20 % as at 400 %.

### `fn rotate_rect`

Separate from [`grip_rects`] rather than an entry in it, because every
consumer of that list treats its members as resize grips: the painter draws
them as squares and the hit test routes them to `DragKind::Resize`. Adding a
ninth entry would have made the rotate handle a square that resizes — the
same collision `Grip::is_resize`'s own note describes, arriving through the
list instead of through the predicate.

Drawn as a circle at this rect's centre; see [`Grip::Rotate`].

### `fn rotate_rect_in`

In a turned frame the stem points out along the frame's own outward normal,
so the handle stays above the mark's own top edge at every angle rather than
above the page's. A handle that stayed at the page's top would end up
*inside* the mark for any turn past 90°.

### `enum GripFrame`

# Why a type rather than an `Option<[Pos2; 4]>` parameter everywhere


# The upright case is bit-for-bit what it always was

[`GripFrame::Upright`] carries the same `Rect` the old signature took and
every arm below reduces to the old arithmetic for it. That is deliberate and
it is what makes this change unable to regress the overwhelming majority of
selections — nothing that is not *turned* takes a new code path.

### `fn corners_for_trace`

Rounded, and that is the point rather than tidiness: a trace is
compared by a driven check, and a float printed with full precision
differs between a debug and a release build on the last digit. Whole
screen points are the resolution the assertion is about anyway.

### `fn anchor`

The turned arm is the upright arm's arithmetic written in terms of the
frame's own corners rather than the page's axes — a mid-edge grip is the
midpoint of the two corners it lies between, and the rotate handle is
[`ROTATE_STEM_PX`] out along the frame's **own** outward normal from the
top edge, so the stem still points away from the mark at every angle.

### `fn pivot`

⚠ **[`Grip::Move`] and [`Grip::Rotate`] are handled before
[`Grip::opposite`] is consulted, and getting that wrong is a real trap.**
Both are their own opposite, so routing them through `anchor` would
answer with the *rotate handle's position on its stem* — a point outside
the box — where the correct answer for a rotation is the centre. The
upright arm has never had this hazard because [`Grip::pivot`] answers
both arms directly.

### `fn grip_rects`

Mid-edge grips are omitted on an axis shorter than
[`MIN_MID_GRIP_EXTENT_PX`] — see that constant for why. The corners are
always present, so a selection is never left with nothing to grab.

`bounds` must already be the **visible** box, i.e. after
[`crate::canvas::overlay::visible_outline_rect`] has grown a degenerate
one. A zero-height rule would otherwise get eight grips stacked along a
line, which is both unaimable and a fair description of nothing.

### `fn grip_rects_in`

# The mid-edge omission test is taken on the frame's OWN edges

A turned box's screen width says nothing about whether its grips pile up:
a 200 × 10 pt bar turned 45° has a screen bound of ~148 × 148, which would
pass a width test while its two mid-edge grips sat 7 px apart on the short
edge. Measuring the frame's own edge lengths is the same question the
upright arm asks, asked correctly.

### `struct GripSet`

The single `offer_resize` bool this replaces was correct while exactly one
kind of thing could be resized. It cannot express *"eight grips, no rotate
handle"*, and the alternative — painting a rotate handle that does nothing —
is the **visible control, silently inert** failure this project spends its
time removing.

**AND IT EARNED THE SECOND FLAG THE SAME DAY, FROM THE OTHER SIDE.**

`Pass 155.0` gave the engine `rotate_annotation` and `Pass 159.0` gave it
`rotate_dimension`, so within hours of this struct being written the two
flags stopped moving together in *both* directions:

| selection | resize | rotate | why |
|---|---|---|---|
| page content, Object rung | ✓ | ✓ | `transform_objects` does both |
| a **markup** annotation | ✓ | ✓ | `resize_annotation` **and** `rotate_annotation` |
| a **ce dimension** | ✗ | ✓ | its extent IS its measurement, so there is no scale verb and there will not be one — but a rotation is an isometry, so it can turn |
| a **form field's** box | ✓ | ✗ | a widget's rotation is `/MK /R`, a quantised 0/90/180/270 declaration, and it is not built |

⇒ The ce-dimension row is the one a single bool could never have expressed,
and it is the row that proves the split was not premature: **rotate without
resize** is a real, shipping combination, not a hypothetical. The engine
declined a dimension scale by name and will keep declining it — *"either the
displayed value stays fixed while the geometry grows, so the dimension lies
about the drawing; or both change, so nothing was measured"* — so this is a
permanent asymmetry rather than a gap waiting to close.

### `fn all`

The annotation reached this set from `scale_only` when
`rotate_annotation` shipped, and the two callers now name the identical
value for two different reasons. That is fine and is not a merge waiting
to happen: content rotates through `transform_objects`, an annotation
through `rotate_annotation`, and the day either verb is withdrawn only
one of the two callers changes.

### `fn scale_only`

**A widget is the one thing on this canvas that scales and cannot
turn**, and the asymmetry is the PDF standard's rather than a gap in
pdfcer. `edit_widget(… with_rect)` rebuilds a field's appearance into a
new box, so a resize is expressible; a widget's *rotation* is `/MK /R`
(§12.5.6.19 Table 189), **a quantised 0/90/180/270 declaration the
appearance generator reads** rather than a free-angle transform. There
is no verb for it, and `rotate_annotation` refuses a widget by name and
points at one that is not built.

⇒ So no rotate handle is painted over a form field and none is
hit-tested there. **R9**: rendering nothing is the honest answer for a
capability that does not exist. A ninth handle that declined on release
would be the *"visible control, silently inert"* failure wearing the
costume of a fix.

Until 2026-08-28 this said *"an annotation or a form field's box"*.
The annotation moved to [`Self::all`] when `rotate_annotation` shipped;
the widget stayed, and it is the only member left.

### `fn move_only`

# The one kind on this canvas whose box does not describe it


⇒ So it is offered no grips. **The outline is still drawn** — it has to
be, or a selected sticky is indistinguishable from an unselected one —
and the body still drags, because moving one is exactly what its `/Rect`
is for.

This is R9 in the form this project keeps meeting it: *an unavailable
capability renders nothing.* Eight squares round a marker that cannot
take a resize is the "visible control, silently inert" failure, and
until today it was shipped — the operator grabbed a corner, dragged, and
got a decline whose stated reason was **false** ("pdfcer did not draw
it", about a marker pdfcer had drawn).

⚠ **Not `rotate_only`**, which would be the natural guess from the ce
dimension's row: `NoRotate` is in the same sentence as `NoZoom`. A
sticky does not turn either.

### `fn rotate_only`

The combination that could not be spelled before this struct had two
fields, and the one that proves they had to be two.

A ce dimension has **no scale verb and is never going to have one**.
`pdfcer-core` declined it outright rather than leaving it unbuilt, and
the argument is worth carrying here because it is the reason this
constructor is not a temporary shape:

> It has no honest reading. Either the displayed value stays fixed while
> the geometry grows, so the dimension **lies about the drawing**; or
> both change, so **nothing was measured** and the operator has *drawn*
> a number rather than *taken* one.

A **rotation** has no such problem, because a rotation is an isometry:
every distance is preserved, so the measured value is identical either
side of it *by construction*. That is what makes turning a dimension a
legitimate drafting operation while scaling one is not.

If an operator wants a dimension to read a different number for the
same drawn line, the operation they want is `set_group_scale` — points
per unit — which already ships and lives on the Measure surface. This
canvas deliberately offers no handle for it: a scale that is a property
of a *measurement group* has no grip on one member of that group.

### `fn grip_at_in`

# The body test stays on the UPRIGHT bound, and that is not an oversight

The eight squares and the rotate handle move into the turned frame, because
they are affordances the operator aims at and they must be where they are
drawn. [`Grip::Move`] is different: it means *"the press landed on the
object"*, and narrowing it to the turned quad would make a turned mark
**harder to grab than an upright one**, on exactly the marks whose extent is
least obvious.

That is also the rule `selection::annot::Candidate` already follows for
selection itself, and it comes from the engine's own argument: `/Rect`
contains the pen half-width, so it is the geometry *plus* the tolerance a
hit test would otherwise have to add. Selection stays generous; only the
drawing gets more honest.
