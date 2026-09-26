# `canvas::input` — reading one frame's pointer: what it landed on, what it is panning, and where the gesture is kept

## Why this is a module rather than four functions at the bottom of [`super`]

Rule R2's 1,500-line ceiling forced a split when the rulers landed, and
this is the seam it forced — the same way it produced [`super::trace`] when
Phase 4 added the strip, and [`super::strip`] alongside it. Both of those
headers record that the forced seam turned out to be a real one, and so
does this.

Everything here answers **"what is the pointer doing this frame?"**, and
every one of them is a question with a single, local answer:

| function | question |
|---|---|
| [`probe`] | what a click landed on, at every rung of the selection ladder at once |
| [`pan_delta`] | whether *either* of the two panning gestures is in flight, and how far it moved |
| [`load_gesture`] / [`store_gesture`] | where the in-flight press lives between frames |

What is left behind in [`super`] answers a different question — *how is the
frame composed?* — and it is a question about layout, the scroll area, the
strip and the order the overlay is painted in. Nothing here needs any of
that: [`probe`] needs a provider and a mapping, [`pan_delta`] needs an
input state and a rect, and the two `Memory` accessors need a `Context`.

## The one thing that is still in `egui::Memory`, and why

[`GESTURE_MEMORY_KEY`]. The selection moved off `Memory` and onto
`OpenDoc` at stage S4 because it is **document-scoped** state and `Memory`
outlives documents; the argument, and the address-as-identity hazard that
came with the workaround, are in [`crate::app::state::OpenDoc::selection`].

A gesture is the opposite case and it is worth being explicit about why.
The drag that is happening *right now* is genuinely frame-local UI state.
It has no meaning across a document, and a gesture that survived one would
be a drag continuing over a file it did not start on. Keying it in `Memory`
means it cannot: `Memory` is per-`Context`, and every document change
starts the next frame with no press in flight — by construction, with
nothing to compare and nothing to forget.

## Item notes

### `const GESTURE_MEMORY_KEY`

**The one thing that stayed in `Memory` when the selection left**, and
the distinction is the point rather than an omission — see this module's
header.

### `fn allowed_candidates`

# The engine's precision is not uniform across object kinds

`pdfcer_core`'s `object_hit` scores each kind differently, and the shell
inherits the asymmetry whole:

- **Path** — tested against its ink: bounding-box reject, then fill interior
  under the object's own winding rule, then stroke/outline proximity.
- **Text** — tested against each run's bounds.
- **Image** (inline image or image XObject) — tested as its page bounding
  box inflated by the tolerance. No alpha, no `/SMask`, no clip, no
  geometry.

So a mostly transparent picture takes every click inside its rectangle, and
it does so at both depths, because page objects and form leaves are scored
by the same function. Paint order is back-to-front, so such an image masks
everything beneath it.

A **form XObject** is never a candidate: the deep pick skips
`ImageSource::Form` because a form's `/BBox` is an extent declaration rather
than ink, and only its leaves compete.

# Why this is not `hit_test` with a predicate bolted on

[`CanvasTargetProvider::hit_test`] is defined as the head of
[`CanvasTargetProvider::hit_test_all`], and that definition is load-bearing:
it is what makes *"what does a plain click select?"* and *"what does
cycling step through?"* structurally the same answer rather than a
convention two implementations have to keep in step.

A filter must not break that. So it walks the same depth-ordered list and
takes the first **allowed** entry, which keeps the two answers derived from
one query — and, as a side effect, is exactly the traversal a future
"select the object underneath" needs.

# A provider that cannot classify lets everything through

[`CanvasTargetProvider::object_class`] returns `None` for a target it does
not know, and the default implementation returns `None` for every target.
`None` means *"I cannot say"* and is treated as ALLOWED.

Getting that default backwards would be quiet and severe: every test double
in the crate uses the default, so treating `None` as forbidden would make
every object unselectable in every harness — and the failure would look
like a broken hit test rather than a filter, because nothing would name the
filter in the output.

### `fn direct_hits_first`

# The defect this closes, measured on his own drawing

`OPERATOR_REQUESTS.md` O198, first sentence:

> *"I really need you to focus on finding ways to make all text editable on
> the sw drawing that is in pdftests folder. Find a way to make it happen."*


`crates/pdfcer-gui/tests/ken_sw41177_pick.rs` then asked the engine the same
question at four tolerances, and the answer is unambiguous:

| tolerance | frontmost candidate is text |
|---|---:|
| 0.0 pt | **9 of 9** |
| 8.0 pt | 4 of 9 |

The path was not on top of his text. It was winning on **slack**.
`pdfcer_core::vector` hits a path on *fill interior, or stroke proximity
within half the scaled line width **plus the tolerance***, and hits a text
object on *its bounding box inflated by the tolerance*. A title-block label
sits a few points from the rules of its own cell, and a tolerance derived
from a handful of screen pixels is several points of PDF user space on a
1,584 pt sheet. So the press was inside the text and beside a line, and the
line, being painted later, won.

**Every font control, Properties field and restyle verb in this program
is reached through a text selection.** A hit test that cannot produce one
makes all of them unreachable at once — which is precisely how a capability
that is present, registered and green on a fixture reaches the operator as
*"that entire area is always greyed out in the menu"*. Claims 1, 3 and 4 of
O198 are one defect seen from three surfaces.

# Why this is the conventional rule and not an invention

Every vector editor in the class behaves this way, and it is why none of
them needs a modifier to click a label on a busy drawing. Slack exists to
make thin things clickable; it is not a claim that a thin thing outranks the
thing the pointer is inside. Stated in full, the rule this function enacts:

> Slack is a tie-breaker of last resort. It may promote a candidate over
> **nothing**. It may never promote one over a candidate that needed no
> slack at all.

# What it deliberately does NOT do

* **It does not know what a text object is.** The partition is *exact versus
  inexact*. Text comes out in front on his drawing as a consequence of where
  he clicked, not because text is special — and a rule that named text
  would fix his title block while leaving a click on a hatch beside a leader
  line behaving exactly as it does today.
* **It never adds or removes a candidate.** Both groups are subsets of the
  list that was already going to be returned, concatenated. The count the
  status strip reports and the length an `Alt`-cycle wraps on are unchanged,
  so nothing that was reachable before this function existed becomes
  unreachable after it. That is the same property
  [`crate::canvas::pick::PickFilter`] states subtractively one layer up.
* **It leaves a genuine overlap alone.** If the press is dead-on both the
  line and the label, both are exact, the partition is a no-op and paint
  order decides — which is right. You clicked the line.

# The cost

One extra `hit_test_all` at tolerance zero, and only when two or more things
survive the filter under the pointer. The engine bounds that at one linear
pass over the page's objects, which is the same pass the first query already
made.

### `fn name`

Spelled out rather than `{:?}`-formatted. A `Debug` rendering of a target
has already made a driven check in this project report the opposite of the
truth while quoting the truth in its own message, and the two variants index
two different lists in the same document — so which list a number belongs
to has to be on the line, not inferred from it.

### `fn nth_allowed`

# The defect this closes

The operator, 2026-08-26: *"when I click on one of the objects all I get is
the page selected."*

The engine computes the **whole** front-to-back list of what is under a
point — `hit_test_all` — and this module called `.find()` on it and threw
the tail away. So the front-most candidate was the only reachable one, at
every point, for ever. On a page carrying anything page-sized, that one
candidate is the answer to every click anywhere.

The root cause of his complaint is one level below this — the engine does
not enter form XObjects, so the objects he is pointing at are not in the
list at all, and that is filed as an engine request. **This is the other
half**, and it is the half that is ours: even for the objects that ARE in
the list, anything underneath anything was unreachable.

# `depth` and its wrap


Wrapping rather than clamping because a cycle the operator can walk out the
far side of is a cycle they can get lost in: with no visible list, a control
that stops responding is indistinguishable from one that has broken. Coming
back round says *"that was all of them"* without a word of copy.

### `mod tests`

These exercise [`allowed_candidates`] rather than [`probe`] because the
rule is about the ORDER of a list, and `probe` returns only its head. A
test that could see the head alone could not tell *"the near miss was
demoted"* from *"the near miss vanished"*, and the second would be a
selection defect this function is explicitly promising not to introduce.

# What is NOT exercised here, stated rather than implied

The `direct.is_empty()` branch — every exact hit removed by the operator's
pick filter, so nothing may be promoted. [`crate::canvas::target::StubTargets`]
does not implement `object_class`, so it returns `None` for every target
and the filter lets everything through; there is no way to reach that arm
through this double. Naming the gap is the point: a reader counting four
tests against four branches would otherwise conclude the arm is covered.
The live cover for it is the driven check on his own drawing, where the
filter is real.


The call in [`allowed_candidates`] was gated behind `false`, the four tests
re-run, and the source restored from a file copy. Exactly one went red:
`a_direct_hit_outranks_a_near_miss`. The other three stayed green — which is
what they are FOR. They assert that the rule changed nothing else, so a
plant that turned them red as well would mean they were measuring the
re-rank rather than its blast radius, and a suite in which every test fails
together cannot tell a regression from a rewrite.

### `fn label_beside_a_rule`

Object 0 is the text run — a small box the pointer lands INSIDE.
Object 1 is the cell rule beside it: a thin strip the pointer lands
four units short of, painted afterwards, which is what makes it
front-most in [`StubTargets::hit_test_all`]'s reversed scan.

That is the measured shape of the defect, not an invented one: on page
1 of `SW41177.pdf` the winner was object 5,899 of 5,903 — very nearly
the last thing painted — at eight of nine aims.

### `fn a_direct_hit_outranks_a_near_miss`

This is O198's first sentence as an assertion. Without the re-rank the
head here is `Object(1)`, which is what the operator was selecting
every time he clicked a label on that drawing.

### `fn nothing_dead_on_leaves_paint_order_alone`

A press in the gap between the two — outside the text, outside the
rule, within four of each. Every candidate is a near miss, so promoting
one over another would be inventing a ranking rather than applying one.

### `fn a_genuine_overlap_is_still_decided_by_paint_order`

The pointer inside a second text box that genuinely crosses the rule.
Both are exact, the partition is a no-op, and the later-painted one
wins — because you clicked the line.

### `fn the_rerank_is_a_permutation_at_every_point_on_a_grid`

Asserted as a multiset over every point on a grid crossing both
objects and the space around them, rather than at one chosen point.
The status strip's *"3 objects here"* count and the wrap length of an
`Alt`-cycle are both this list's length, so a rule that dropped a
candidate would present as a control that had stopped responding
rather than as a hit-test defect.
