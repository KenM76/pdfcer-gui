# `app::actions::vector` — everything that changes page GEOMETRY

## Why this is its own file

**R2**, and the seam [`super::action::Action`] already draws twice:
[`super::dimensions`] is *what happens to the dimensioning model*,
[`super::pages`] is *what happens to a page*, [`super::annots`] is *what
happens to an annotation that already exists*. This is **what happens to the
marks on the page**.

It is a real subject rather than a size-driven cut, and the evidence is a
property every variant here shares and nothing elsewhere does: **each one
addresses paint-order indices into one page's content stream.** That makes
all of them subject to one rule nothing else in `actions` has to think
about — `docs/core-api/02-editing-and-saving.md` §1.10.1, *which verbs
RENUMBER* — and it is why the page travels on every variant rather than
being re-derived at apply time.

## The two verbs that both "move things", and why there are two

[`VectorAction::MoveSelection`] reaches `move_objects`, which rewrites
numeric **operands** in place. [`VectorAction::TransformObjects`] reaches
`transform_objects`, which wraps each object's operator run in `q <cm> … Q`
and never looks at an operand at all.

The second can express everything the first can and more — rotation, scale,
and *any object kind*, because text runs and images carry no coordinate
operands, which is exactly why `move_objects` is path-only by name. The
obvious tidy is therefore to route everything through the transform, and it
would be worse.

`move_objects` rewrites the numbers the producer wrote, so a moved path's
operands still say where its marks are. The transform leaves those operands
naming the old place and adds a `q <cm> … Q` the producer never wrote to
say how far. Both render identically — R8b's screenshot test cannot tell
them apart — so the whole of the difference is in the bytes a CAD re-import
reads, and pdf content is the one thing this program is not allowed to
alter casually.

⇒ The transform earns the gestures operand rewriting cannot express at
all: rotation and scale, which have no `re` spelling and no user-space
scalar for `line_width`, and every object kind, because text runs and
images have no coordinate operands and `move_objects` answers `NotAPath`
for both. It does not earn the gestures the operand rewrite already
expresses exactly.
