# `canvas::target` — the seam a hit-testable content model plugs into

The canvas selects *things*. It does not know what a thing is, how it was
decomposed, or what coordinate frame its geometry was authored in. All it
needs is: **what is under this point, what is inside this rect, and where
is the thing I already have?** That question set is
[`CanvasTargetProvider`], and everything in `canvas/` is written against
it rather than against `pdfcer-core`.

## Why a trait rather than a direct call into the provider

Three reasons, in order of how much they cost if ignored.

1. **The selection layer becomes headlessly testable.** Every invariant
   this stage is accountable for — *selection survives navigation* above
   all — is a property of the selection layer's *logic*, not of PDF
   decomposition. A test that had to build a `Document` to prove that
   zooming does not clear a selection would be a slow test of the wrong
   thing. [`StubTargets`] lets those tests state a page's contents in
   three lines.
2. **The old shell already drew this line, and the provider was salvaged
   expecting it.** `panels::objects::provider`'s header, §2 of "What
   changed at salvage": *"The `CanvasTargetProvider` trait impl became
   inherent methods. The trait lives in `canvas/` and does not exist yet.
   The three methods keep their names and their exact semantics …
   Re-attaching the trait at S4 is a one-line `impl` block over methods
   that already have the right signatures."* This module is that
   re-attachment, and it is exactly that: [`impl CanvasTargetProvider for
   ObjectModelProvider`] delegates and adds nothing.
3. **`GUI_ROADMAP.md` Phase 4** (continuous page display) changes *which
   pages* a provider answers for. A canvas written against the concrete
   single-page provider would have that assumption spread through it; a
   canvas written against a trait that takes `page_index` on every query
   already asks the right question.

## Every geometric argument here is CANVAS space

Points, rects and tolerances crossing this trait are in canvas space —
Y-**down**, origin at the page's top-left, `/Rotate` already resolved. The
provider owns the hop into PDF user space (Y-**up**), because it owns the
page transform and inverts the *renderer's own* map to get there, so the
selection geometry and the raster agree by construction. See
[`crate::canvas::mapping`] for the full three-frame table and why
conflating any two of them is silent.

## The tolerance is a parameter, never a provider constant

Stated on [`CanvasTargetProvider::hit_test`] and worth stating here too:
the only honest source for a hit tolerance is the live zoom, and the live
zoom belongs to the frame, not to the model. A provider that baked its own
tolerance would be a provider whose catch radius shrank as the operator
zoomed out — which is the defect
[`crate::canvas::mapping::SELECT_SCREEN_TOLERANCE_PX`] exists to close.
