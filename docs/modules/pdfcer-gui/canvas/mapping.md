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
