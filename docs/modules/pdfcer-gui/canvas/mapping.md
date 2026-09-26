# `canvas::mapping` — the ONE screen↔page conversion, the PDF↔canvas
projection, and the tolerance

## Why this file exists at all

`GUI_ROADMAP.md` Phase 1 names three ways a selection model loses the
*"selection survives navigation"* invariant. The first is **selection
stored in screen coordinates**, and it has a twin that is easier to miss:

> *"Every hit-test and snap `tolerance` is a PAGE-space radius, and
> nothing checks it. Pass raw screen pixels and it compiles, runs, and
> merely drifts with zoom"* (`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`,
> describing `pdfcer_core::vector::hit::hit_test_point`'s `tolerance` — and
> `hit_test_point_all`, `_deep`, `hit_test_text_runs`, `hit_test_subpaths`
> and `_of` take the same unchecked `f64`).

Both failures are the same mistake — *a screen number used where a page
number was meant* — and both are silent. So this module is the **single
boundary**: everything crossing it in one direction is screen space,
everything crossing it in the other is page space, and there is no second
place in `canvas/` that divides by `zoom`.

Concretely: [`PageMapping`] holds the frame's page rect, extent and zoom,
and every conversion the selection layer needs is a method on it. A caller
that has a `PageMapping` cannot accidentally convert a point with this
frame's zoom and a tolerance with last frame's, because there is one zoom
and it is inside the mapping.

## "Page space" here means CANVAS space, and that is deliberate

Three frames are in play and conflating any two of them is the classic
silent defect (`viewer`'s own header sets out the first two):

| frame | Y | origin | who speaks it |
|---|---|---|---|
| **screen** | down | window top-left | egui, the pointer, the painter |
| **canvas** | down | page top-left, `/Rotate` applied | this module, [`crate::panels::objects::provider::ObjectModelProvider`]'s public surface, the raster |
| **PDF user** | **up** | un-rotated CropBox lower-left | the object model's *internals*, every `pdfcer-core` authoring verb |

[`PageMapping`] converts **screen ⟷ canvas** and stops there. The
canvas → PDF-user hop is the provider's own business
([`crate::viewer::canvas_to_pdf_space`] is the per-point sibling), and it
is left there on purpose: it needs the page's device transform, it is
already implemented once by inverting the *renderer's* own transform, and
a second implementation here would be a second chance to get the Y-flip
backwards. **PDF user space is y-UP; canvas and screen are y-DOWN.** The
failure is silent — the page looks perfect until someone selects a line
and gets a different one.

## Why the tolerance is a distance and not a rect

Canvas space at zoom 1.0 is *distance-preserving* with respect to PDF user
space: `page_device_geometry(page, 1.0)`'s transform is a rotation, a
Y-flip and a translation, none of which change lengths. So a radius of
*n* canvas units **is** a radius of *n* PDF units, and one number can
serve both — which is exactly why
[`crate::panels::objects::provider::FALLBACK_SELECT_TOLERANCE`] can be
documented as "canvas space, and in effect page space" without a
conversion. If the canvas ever gained a non-uniform scale that would stop
being true, and this paragraph is where it would have to be revisited.

## Item notes

### `fn a_rectangle_with_no_area_produces_no_box`

The corner-order half matters on its own: §7.9.5 permits `/Rect` either
way round, and a hit test against an un-normalised rectangle would
silently never match. `widget_rects` normalises and
`Annotation::rect` does not, so this function must cope with either.

### `fn an_annot_box_lands_at_the_top_of_an_unrotated_page_and_moves_when_it_turns`

The half of the geometry a unit test can actually hold. `/Rect` is
y-**up** from the CropBox's lower-left and canvas space is y-**down**
from the page's top-left, so a field near the TOP of the page in PDF
terms (a large Y) must land near the top in canvas terms (a small Y) —
and the failure when it does not is silent, because the page looks
perfect and only the click is wrong.

Under a quarter-turn the box moves to the corresponding edge rather
than staying put, which is the assertion that would fail on a build
that projected the rect without the page's transform.

### `fn screen_tolerance_keeps_the_on_screen_catch_radius_constant`

`panels::objects::provider`'s salvage note §4 records that this test
could not come across with the provider, because asserting it there
would have meant re-declaring the constant — putting the tolerance in
two places, which is the *cause* of the defect it guards. It lands
here, with the constant and the conversion it is about.

The property: the canvas-space tolerance a click supplies scales as
`1 / zoom`, so the SCREEN-space catch radius is the same number of
pixels at every zoom level. Assert the *outcome* (the on-screen
radius) rather than the intermediate (the page-space number), so this
checks the law and not merely that the code agrees with itself.

### `fn a_canvas_point_survives_every_zoom_and_scroll_position`

The arithmetic half of the "selection survives navigation" invariant:
the *same object point* has the same canvas coordinate at every zoom
and every scroll position, which is exactly why a selection held in
canvas/identity terms survives navigation and one held in screen
terms cannot.

Modelled by taking a fixed canvas point, projecting it to screen at
each zoom (with the page rect moving as the scroll area would move
it), and converting back.

### `fn each_page_of_a_strip_has_its_own_mapping`

The failure this pins is the one Phase 4 was most likely to ship
silently: under a continuous mode the Find wash has to be painted for
several pages at once, and painting them all through the *acting*
page's mapping would stack every page's highlights onto one page. The
hits would still be found, the wash would still be drawn, and it would
be drawn in the wrong place — which reads as a highlight bug rather
than a mapping one, and is exactly the class `canvas/mod.rs`'s own
centring comment records the old GUI shipping.

Asserted as the *outcome*: the same canvas point — the top-left corner
of a page — projects to each page's own screen origin through that
page's mapping, and to somewhere else entirely through its neighbour's.
The second half is what makes this a test rather than a tautology; a
build in which the two mappings were accidentally equal would pass the
first half.

### `fn a_page_displacement_converts_without_the_origin`

`DEFECTS.md` D18's root: two quantities in two spaces, both `Vec2`, and
nothing to notice. The rect above starts at (316, 580) precisely so a
conversion written as `to_screen(a)` — the point form — fails this.

### `fn the_two_directions_are_inverses`

The number that matters: at 29.55 % a 60 px drag is 203 page units, and
handing those 203 to a function expecting 60 is what inflated every
resize factor by `1/zoom`.

### `fn a_degenerate_zoom_answers_zero_rather_than_nan`

Reachable: a page drawn at zero size for one frame. A NaN displacement
reaching a content stream is a corrupted file; a zero one is a gesture
that did nothing, and only one of those is recoverable.

### `fn an_anchor_is_caught_more_easily_than_an_object_and_as_easily_as_a_handle`

Both halves are asserted because both are the argument. The first says
the widening happened; the second says which number was chosen and why
— an anchor that was harder to hit than the Bézier handle hanging off
it was the concrete absurdity the row is about.

### `fn the_object_catch_radius_is_unchanged`

The assertion that pins the scoping. On a sheet this project has
measured at 129,758 objects a larger catch radius means more candidates
under every press and a different answer to "what did I click?" — so
the two constants must stay two constants.

### `fn the_node_radius_scales_as_one_over_zoom`

The sibling of `screen_tolerance_keeps_the_on_screen_catch_radius_constant`,
asserted for the new one because a radius that did not scale would be
eight page points at every zoom — a catch radius the size of a sheet
when zoomed out, and invisible when zoomed in.

### `const SELECT_SCREEN_TOLERANCE_PX`

# Why 6, and why a screen number rather than a page number

Salvaged verbatim from the old shell, with its reasoning, because the
reasoning is the valuable part. The behaviour it replaced was a fixed
`3.0` **canvas-space** value, which is `3.0 × zoom` pixels on screen: 3 px
at 100%, 1.5 px at 50%, 0.75 px at 25%. Objects were effectively
unclickable at exactly the zoom an operator uses to see a whole drawing.

Deliberately a *sibling* of the snap radius rather than the same constant:
snapping and selection answer different questions and are allowed to drift
apart. Selection is set **tighter** because a snap that grabs a nearby
vertex is a helpful correction the operator can see and cycle through,
whereas a selection that grabs a neighbouring object is a silent wrong
answer. The failure modes are not symmetric, so the tolerances should not
be either.

# This constant lives HERE and nowhere else

`panels::objects::provider`'s salvage note records that one test did not
come across —
`screen_tolerance_keeps_the_on_screen_catch_radius_constant` — because
*"re-declaring those constants here to keep a test green would put the
tolerance in two places, which is the cause of the defect the test guards,
not a way to guard it."* This module is where the constant landed, and
[`tests::screen_tolerance_keeps_the_on_screen_catch_radius_constant`] is
that test, restored.

### `const NODE_SCREEN_TOLERANCE_PX`

Eight rather than six, and both halves of that are borrowed rather than
invented:

* it is the number [`crate::canvas::handledrag::GRAB_PX`] already gives a
  **Bézier control point**, whose own doc says *"a target that requires
  hitting its exact pixels is a target an operator misses"* — so an anchor
  stops being harder to hit than the handle that hangs off it, which it
  was;
* it is Inkscape's *grab sensitivity* default, which is the operator's
  stated tie-breaker for this family of decisions.

# A SIBLING of [`SELECT_SCREEN_TOLERANCE_PX`], not a change to it

Widening the shared constant would have widened **object picking** too, on
a sheet this project has measured at 129,758 objects — where a larger catch
radius means more candidates under every press and a different answer to
*"what did I click?"*. The two answer different questions and are allowed
to drift, which is the same argument that constant's own header makes about
the snap radius.

It reaches exactly one call: `canvas::input::probe`'s `nearest_node`, and
only from the **Node tool**'s click. The general click path still passes
[`SELECT_SCREEN_TOLERANCE_PX`], so its behaviour is byte-identical.

### `fn screen_tolerance_to_page`

This is the exact `1 / zoom` distance law
[`crate::viewer::screen_to_page`] uses, proven zoom-invariant by that
module's `screen_to_page_distance_scales_as_one_over_zoom` test. A
constant on-screen catch radius therefore maps to a *shrinking*
page-space tolerance as the operator zooms in, which is what keeps the
click target feeling identical at every zoom.

# Degenerate inputs yield `0.0`, and that is not a silent failure

A non-finite or non-positive `zoom` (reachable: the page's drawn size is
zero for one frame after an open, and a fit scale on a degenerate CropBox
can go non-finite) returns `0.0` rather than a NaN or an infinity. `0.0`
is then recognised by
[`crate::panels::objects::provider`]'s `resolve` as degenerate and
replaced with the fixed canvas-space fallback — so a bad zoom makes
selection *fussy for one frame*, never *broken*. Returning NaN instead
would make every comparison in the hit test false and every query a miss,
with nothing to say why.

### `struct PageMapping`

Constructed once per frame in [`crate::canvas::show`], immediately after
the scroll area has settled and the page's true drawn rect is known, and
then handed to everything that needs a coordinate. **Nothing downstream
of it sees a screen coordinate again**, except the overlay, which converts
back through [`Self::to_screen`] at the moment of painting.

`Copy` because it is three small values and passing it by value removes
any question of it being stale: a mapping is a fact about one frame, and a
borrow that outlived the frame would be a mapping for a page rect that has
since moved.

### `fn image_rect`

Deliberately the only way *out* of this type other than a conversion.
There is no `zoom()` accessor: the zoom's whole job here is to be
divided by, and exposing it would be an invitation to divide by it at
a call site — which is the defect this module exists to make
unavailable. Anything that needs a page-space distance asks
[`Self::tolerance`].

### `fn page_vec_to_screen`

# Why a vector needs its own conversion, and what it cost not to have one

A point conversion carries the page's origin on screen; a displacement
must not. `to_screen(a) - to_screen(b)` is correct and says the origin
twice; this says it none.

**`DEFECTS.md` D18 is what its absence cost.** The gesture machine works
in **page** space by design — `canvas::interact` builds its
`PointerFrame` with `pos: screen_pos.map(|p| map.to_page(p))` — so
`GestureOutcome::Resize.delta` is a page-space displacement. It was
handed straight to `resizing::Frame::delta`, whose doc comment says *"in
screen points"*, and divided against a `bounds` that genuinely is screen
space. Every resize factor's distance from unity came out inflated by
**`1/zoom`**: at the operator's fitted 29.55 % a corner dragged 60 px
committed a 5.94× stretch where the contract says 2.46×, and the shape
shot 143 px past the cursor on both axes.

⇒ Two quantities in two spaces, one of them undocumented at its call
site, and **nothing in the type system to notice**: both are `Vec2`.
This method is the place the multiply lives, for the reason this
module's header already gives about the divide — *"there is one zoom and
one place in `canvas/` that divides by it."* The same must be true of
multiplying, or the two drift.

It is deliberately **not** called `to_screen_vec`. `to_screen` and
`to_page` are a matched pair over positions, and a name one character
away from them is how a caller reaches for the wrong one; this one says
*page vector* in its name so the space is at the call site rather than
in its documentation.

### `fn screen_vec_to_page`

A non-finite or non-positive zoom answers `Vec2::ZERO` rather than a
NaN, on the same argument [`screen_tolerance_to_page`] makes: a
degenerate zoom is reachable (a page drawn at zero size for one frame),
and a NaN displacement reaching a content stream is a corrupted file,
while a zero one is a gesture that did nothing.

### `fn rect_to_page`

Normalised with [`Rect::from_two_pos`] rather than assembled from
`min`/`max`, because a rubber-band is dragged in any of four
directions and its "min" corner is wherever the press happened to be.
A non-normalised rect has negative width and every containment test
against it silently answers `false`.

### `fn rect_to_screen`

Normalised for the same reason [`Self::rect_to_page`] is: the
screen↔canvas map is a pure scale and translate with no flip, but the
rects it is handed come from the provider, which built them by
bounding a *mapped quad* under a transform that may rotate. Assuming
corner order here would produce an inside-out rect that paints nothing.

### `fn tolerance`

The one call every hit test makes. Passing
[`SELECT_SCREEN_TOLERANCE_PX`] straight to a provider query — which
compiles, and runs, and merely drifts with zoom — is the defect this
method exists to make unavailable.

### `fn node_tolerance`

[`Self::tolerance`]'s sibling, wider by two pixels, and it exists here
rather than at the call site for this module's standing reason: no
other file in `canvas/` divides by zoom, and keeping that true is what
stops a second conversion drifting from this one.

See [`NODE_SCREEN_TOLERANCE_PX`] for why it is a separate constant
rather than a change to the shared one.

### `fn snap_tolerance`

[`Self::tolerance`]'s sibling, and it sits here for the identical
reason: this module's header states that there is no second place in
`canvas/` that divides by `zoom`, and a snap query that reached for
`SNAP_SCREEN_TOLERANCE_PX` and did the division at its call site would
be exactly that second place.

# Why it is a different number from the selection radius

They are both screen-pixel radii converted the same way, but they
answer different questions and the salvaged constant is the wider of
the two (10 px against the selection's smaller catch). A snap is an
*offer* — it proposes a point and shows an indicator saying which, and
the operator can refuse it by holding Alt — so casting a wide net costs
them nothing. A selection is a *commitment* made silently on release,
so its radius is deliberately tighter: `canvas::forms`' header makes the
same argument one step further, refusing any tolerance at all for form
widgets because fields sit a point apart.

Keeping them separate is what lets each be tuned without the other
moving. Collapsing them would mean widening the selection catch every
time snapping wanted a longer reach.

### `fn annot_canvas_rect`

# Why this lives here rather than with the forms code that wrote it


It moved when annotation **selection** arrived and needed the same answer
for stamps, notes and ce dimensions. Calling it in place would have made
the selection layer depend on the forms layer for pure geometry, and the
name would have lied at every one of those call sites. Both are how a
module graph stops being readable.

# The array shape, and the corner order

`[f64; 4]` is `EditSession::widget_rects`' own shape, taken verbatim rather
than re-wrapped in a `page_tree::Rect`, because converting through a second
rectangle type would be a place for the normalisation to be undone.

**Corner order is not assumed.** §7.9.5 permits `/Rect` either way round.
`widget_rects` normalises; `pdfcer_core::annot::Annotation::rect` reports
what the file says. So a caller can hand this either, and the four-corner
bound below is what makes both work — a two-corner version would produce an
inside-out rectangle for one of them and silently never hit anything.


**All four corners, then bound them.** Two corners are enough while the
transform is a scale, a flip and a quarter-turn, which is every case a
conforming `/Rotate` can produce. Four is what makes that not have to be
true: a transform that shears or rotates by anything else still produces a
box that contains the widget, where a two-corner version would produce one
that is inside-out and contains nothing.

### `fn oriented_canvas_quad`

# Why this cannot reuse [`annot_canvas_rect`], which does the same
projection

Because that function **bounds** its four mapped corners into an upright
`Rect`, and bounding is precisely what destroys the answer here. It is the
right thing for a `/Rect` — §12.5.2 requires that upright, so the bound *is*
the value — and the wrong thing for a quadrilateral whose whole content is
the angle its corners sit at.

⇒ Two functions rather than a flag, because they answer different questions
and a caller that passed the wrong flag would get a plausible rectangle with
no way to tell it had lost the orientation.

# The page's own `/Rotate` is applied here, and that is the point

`viewer::pdf_space_to_canvas` folds in the page rotation, so a 30° mark on a
`/Rotate 90` sheet comes out at the angle the *operator sees*, not the angle
stored in the file. Anything that mapped the artwork's angle and the page's
rotation separately would be a second implementation of the projection and
would disagree on rotated sheets only — which is the shape of defect that
survives every test written on an unrotated fixture.

Returns `None` when any corner falls outside the projection's domain, for
the same reason [`annot_canvas_rect`] does: three good corners and one
missing is not three quarters of an outline, it is a wrong one.
