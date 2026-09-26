# `canvas::annotnodes` tests — the shell's half, and the engine's ruling

## What these can and cannot prove, stated first

**They cannot prove the operator can edit a node.** Every test here calls a
function directly, and the whole point of R1 is that a passing unit test is
not a report of working software: this project once had eight green tests
while the feature performed 1 of 14 steps. Six process boundaries stand
between [`super::resolved`] and a hand on a mouse — the tool arming, the
chord reaching dispatch, the modifier surviving winit, the press
classifying, the anchor being painted where the hit test looks, and the
engine accepting the write — and **not one of them is observable in
process**. `tools/ui-verify`'s `a_markup_shapes_nodes_can_be_edited` is the
instrument for that, and its own header enumerates the six.

What these DO prove, and what makes them worth the second they cost:

1. **The shell's subtype table is right** — which shapes show anchors. That
   is a painting decision this shell owns, so nothing else can check it.
2. **The engine's ruling is asked rather than restated.** Every count test
   below authors a real annotation into a real fixture and drives the real
   `reshape_annotation_preview`. A test that faked the annotation would get
   `AnnotationNotFound` for every case *while looking exactly like a test
   that passed for the right reason* — `dimdrag::tests` records that trap in
   the same words.
3. **A refusal is a sentence.** Half of each refusal test asserts the
   decline was raised, because a build that merely dropped the gesture would
   pass the other half and would be the operator's original complaint.

## The shapes are chosen so the boundary is exercised

A test on a five-sided polygon proves nothing about the floor. The shapes
here are:

| shape | nodes | floor | what removing one does |
|---|---|---|---|
| `/Polygon` triangle | 3 | 3 | **refuses** — at the floor |
| `/Polygon` square | 4 | 3 | succeeds, leaving a triangle |
| `/PolyLine`, three points | 3 | 2 | succeeds, leaving a straight line |
| `/Line` | 2 | — | **refuses by name** — a Line is two ends by definition |
| `/Ink`, strokes of 3 and 2 points | 5 anchors | 2 **per stroke** | succeeds on the first stroke, **refuses** on the second — the same mark, two answers |

The triangle and the three-point polyline are the same three points and give
**opposite** answers, which is what proves the refusal measures the shape's
own floor rather than blanket-refusing three-node shapes. The two-stroke ink
(`Pass 278.0`) proves the same thing one level down: the floor is the
**stroke's**, not the mark's, and a flat anchor index that was converted to
the wrong stroke would get the opposite answer from the engine.

## The ink tests sit at the stroke boundary, on purpose

Every ink assertion below is made at **index 2 or 3** of a five-anchor mark
whose first stroke holds three points — the last point of stroke 0 and the
first point of stroke 1. That is the one place a flat-list implementation
goes wrong in a way that looks right: an off-by-one in the stroke table
addresses the neighbouring stroke, a naive preview draws a segment across
the gap, and a naive insert-after puts the new point into stroke 1. A test
on the middle of a long stroke would pass all three defects.
