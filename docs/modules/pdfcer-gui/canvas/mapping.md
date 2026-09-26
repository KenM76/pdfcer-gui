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
