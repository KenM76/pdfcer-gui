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
