# `provider::geometry` — **the same questions, asked of either index space**

`OPERATOR_REQUESTS.md` **O70**, 2026-09-01. Everything here answers *"what
is inside this thing?"* for a [`TargetId`] rather than for a page
paint-order index, which is what lets the Part and Node rungs be offered for
something painted inside a form XObject.

## Why a module rather than more methods in [`super`]

R2's gate, and it points at a real seam. `super` answers *"what is on this
page and where is it?"* — the decomposition, the hit tests, the canvas
projection, the panel's tree. This answers the narrower question of what one
already-identified object is made of, and it does so for **both** address
spaces, which is a distinction none of the rest of that file has to make.

## Why these are additions rather than changed signatures

The index-based methods in `super` have thirty-odd call sites and every one
is correct: `canvas::input`, `canvas::painting`, `canvas::shapes` and the
Objects panel all hold page indices for good reasons. Changing them in one
edit is a diff nobody can review against a hot path this project has already
broken twice by second-guessing.

⇒ So this is the general form, the index form delegates to it, and call
sites move over one at a time behind their own tests. The duplication is a
`TargetId::Object(i)` wrapper, not a second implementation — the distinction
`canvas::mapping`'s header draws between a shared helper and a second
opinion.

## The engine already published the leaf-friendly forms

`hit_test_subpaths_of(&PathObject, …)` takes the object rather than a model
and an index, and `PathObject::page_subpaths` is geometry in page space
whoever holds it. So none of this needed a request — the shell had been
asking the narrower question because the narrower question was all it had
ever needed.

Proven against a real file rather than a stub, in
`crates/pdfcer-gui/tests/leaf_geometry.rs`: a stub carries rectangles and
cannot model where a path's anchors are, so a unit test against one would
assert that the plumbing returns what the stub was given.
