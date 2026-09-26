# `render::offpage` — proving the engine can rasterize past the page edge

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
