# `rasteroffpage` — proving the engine can rasterize past the page edge

## Why this module is nothing but tests

`OPERATOR_REQUESTS.md` **O23**, second half:

> *"also objects should still be reachable even if they are off the page."*


`pdfcer_render::render_page_region` takes an arbitrary page-space rectangle
and — this is the part the feature depends on — **never clamps or intersects
it with the crop box**.

## The caveat that produced this file

That last claim is true *by construction*: there is no code in the region
path that could reject an off-page rectangle. It is also, in the engine's own
test suite, **entirely unproven**. Its region tests cover sub-rectangles,
quadrant tiling and a stroke-mitre band — every one of them **inside** the
page.

Correct-by-construction and covered-by-a-test are different things, and this
project has spent a day on the difference. A shell feature built on an
unexercised engine path is a feature whose first failure will look like a
shell defect.

So: before any of O23's second half is built, these run.

## What this is NOT

Not a test of `pdfcer-render`. That crate is
[read-only to this project](../../../../PROJECT_PLAN.md) and its coverage is
its own business. This is **pdfcer-gui asserting the properties it is about to
rely on**, in this repository, so that if a future engine bump changes them
the failure lands here — on the consumer, with a message naming the feature
that cared — rather than as a mysterious blank canvas.

That is the same posture `app::settings`' funnel test takes toward
`pdfcer-core`'s option structs: assert the contract you depend on, where you
depend on it.

## Item notes

### `fn a_region_entirely_off_the_page_still_rasterizes`

The property O23's second half stands on. If this ever fails, the engine
has started clamping the region against the crop box and *"objects off
the page are reachable"* is no longer buildable in this shell without an
engine change — which is exactly the finding that would otherwise be
made by an operator staring at a blank rectangle.

### `fn the_pixmap_matches_the_region_asked_for_not_its_overlap_with_the_page`

The distinction that matters for a canvas. A build that quietly
intersected the region with the crop box would still return `Ok` and a
non-empty pixmap for a region that merely *touches* the page — and the
canvas would then draw a raster smaller than the rectangle it asked
for, which presents as content sliding rather than as an error.

Asserted with a tolerance of one pixel per axis: `region_device_geometry`
floors the origin and ceils the extent, so an exact equality would be
pinning rounding rather than behaviour.

### `fn a_region_containing_the_whole_page_and_a_margin_works`

The shape a pasteboard actually asks for: the page plus a margin all
round, in one raster. Separate from the two above because it is the case
where the region *contains* the crop box rather than missing or
straddling it, and a clamp would be invisible in the other two if it
only triggered on containment.

### `fn the_content_union_is_available_and_non_empty`

Asserted here rather than taken from the engine's documentation because
O23's plan uses it as the source of *"what must I be able to scroll to in
order to reach everything"*. If the decomposer ever started culling
against the page box, this union would silently shrink to the page and
the shell would stop being able to reach the very objects the feature
exists for — with nothing failing.

The fixture has no off-page content, so this asserts the weaker, stable
property: the union is non-empty and is not *narrower* than the ink it
contains. It is the guard rail, not the demonstration — a fixture with
deliberate off-page geometry is worth adding when the feature is built.
