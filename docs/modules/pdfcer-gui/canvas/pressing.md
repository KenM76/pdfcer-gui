# `canvas::pressing` — **what a press would land on, and what it would mean**

## Why this is its own file

Everything here answers one question — *if the primary button went down at
this point, right now, what would happen?* — and nothing here changes
anything. `canvas::interact`'s remaining sections advance a gesture, route a
click and paint; this one only looks.

## The precedence, in one place

Four different things can be under the pointer at once, and the order they
are asked in is the whole behaviour. **The most specific thing under the
pointer wins, and specificity is depth down the selection ladder:**

1. a **Bézier handle** of a selected anchor — it sits *inside* the
   selection box, so anything asked before it swallows every press on one;
2. an **anchor**, reached through the inflated move box — an anchor sitting
   on the bounding box's edge is half outside it, so the box is inflated
   rather than the grips suppressed a second time;
3. a **resize grip**, Object rung only — it scales the whole object, which
   is the wrong subject at an inner rung, and left unconfined it covers the
   corner **anchors**, making the end point of every path undraggable while
   the middle ones work;
4. the **selection body** → move — the least specific claim, so it answers
   last.

⚠ Every one of those failures is silent. `grip_at` answering `Move` for a
press on a **handle** moves the whole object instead of shaping the curve,
and that is entirely plausible from a chair, because the object *did* move.

## Everything here reads `press_origin`, not the current pointer

`egui` does not call an interaction a drag until the pointer has travelled a
threshold, so by the frame it says so the pointer is **already that far from
where it went down** — measured at 94 PDF points of error on an A1 sheet at
0.21× zoom. A grip is an 8 pt square and a handle mark is 7 pt across.
Reading the current position misses both, and the miss is silent: the
gesture becomes a marquee, which *clears the selection the operator was
trying to resize*.

## Item notes

### `const STICKY_SUBTYPE`

Compared against the engine's own string rather than mapped through an
enum here: `AnnotSelection::subtype` carries `/Subtype` verbatim, and a
local enum would be a second vocabulary that has to be kept in step with a
standard that keeps adding to it.

### `fn pick_for_body`

**Everything, deliberately, and not the operator's filter.** The question
is *"is the press on the thing I have selected?"*, and the answer must not
depend on whether that kind of object is currently pickable — an operator
who selects an image, then switches images off in the selection filter, has
not thereby asked for the image to become undraggable. The filter governs
what a press may *acquire*; this asks about what is already held.

### `const PRESS_SLOT`

Its own slot rather than sharing one: `trace_changed` keys on the slot, so
two unrelated lines sharing one would suppress each other, and the
suppression looks exactly like the event never happening.

### `fn the_outline_flag_is_not_the_content_flag_nor_the_grip_set`

Asserted as `Grabbable` literals rather than by driving `grabbable`,
which needs a `Context` and an `OpenDoc`. What is under test is the
*relationship between the three flags*, which is where a mistake would
hide: the whole point of `outline` is that it is not expressible as
either of the other two.

### `fn a_sticky_note_gets_no_resize_grips_and_every_other_markup_still_does`

`/Text`'s `/Rect` says **where** the marker is, not **how big**: a
conforming reader draws it as though `NoZoom/NoRotate` were set and
anchors it at the rect's upper-left corner, so `resize_annotation`
declines a corner drag by name. Eight squares round it would be the
*visible control, silently inert* failure this shell exists to remove.

⚠ **Three assertions, and the third is the one that keeps this honest.**
A build that answered `move_only` for *everything* satisfies the first
two and silently removes the resize from rectangles, clouds and arrows —
the kinds that do resize, and did before this change.

### `fn freetext_is_not_caught_by_the_sticky_rule`

`/Text` is a sticky note; `/FreeText` is a text box and resizes. A
`contains` or a case-insensitive compare catches the second with the
first and silently removes the resize from every text box.

### `struct Grabbable`

ONE value, read by the hit test, the painter and the drag — which is
rule **H7**, and the reason this is a struct rather than two functions.

Two surfaces deriving the answer separately diverge in one direction or the
other, and both directions are failures. A handle painted and not
hit-tested is the "visible control, silently inert" failure — grips
**visible and untouchable in the very mode that authors them**. One
hit-tested and not painted is worse: an invisible target that steals the
press aimed at what is under it.

### `fn markup_grips`

Its own function so it can be asserted without an `OpenDoc`, and the seam
this file's own header argues for: *one value, one decision, two
consumers*. The painter and the hit test both receive whatever this
returns.

`""` for a selection that is not an annotation answers the full set,
which is correct by construction: the only caller reaches this line inside
the markup arm, so the empty string is unreachable rather than a default
standing in for a real subtype.

### `fn grabbable`

The chain is ordered by **narrowness** and every link answers `None`
unless its own kind is selected, so it is exhaustive rather than a precedence
anybody has to remember:

- a **ce dimension** → its `/Rect`, and the **rotate handle alone**:
  `rotate_dimension` turns one, no verb scales one, and none ever will.
- a **markup** annotation → its `/Rect`, and everything:
  `resize_annotation` scales it and `rotate_annotation` turns it.
- a **form field's** box → the widget's rect, and the eight scale grips,
  through `edit_widget(… with_rect)`; a widget's rotation is `/MK /R` and
  is not built.
- page **content** → the selection's union, everything, and only at the
  Object rung.

## The markup and the widget are separate arms, and the separation
is the wiring

The two offer different grip sets, for different reasons, so they cannot
share a branch: `rotate_annotation` turns a markup and nothing rotates a
widget. A `grab_box(…).or_else(widgetdrag::grab_box)` answering
`scale_only()` for both would leave a markup with no rotate handle for the
same reason a widget has none — **which is not the same reason at all**,
and the code would say it was.

⇒ The dimension goes the other way again: a box, and the ninth grip only.
See [`handles::GripSet`]'s own header for the table of verbs behind them.

### `fn body_under`

The predicate [`look`] uses to decide whether a `Grip::Move` over a page
**content** selection is genuine, and the one `crate::canvas::presspick`
uses to decide whether the current selection already claims a point. One
function, two callers, because the two must not be able to disagree — see
the block comment at [`look`]'s grip downgrade for the incident that rule
comes from.

# What it asks

Whatever is topmost under `point`, and whether that thing is a member of
the selection. Deliberately the **same** hit test the canvas uses for
picking (`input::topmost`), so "the press landed on the object" here means
exactly what it means everywhere else, including its tolerance.

# Why `false` when there is no decomposition

A page whose objects have not decomposed cannot answer the question, and
the honest answer to *"is the press on the selection?"* is then no. That
direction is deliberate: `false` yields a marquee, which selects things;
`true` would yield a move of a selection the operator may not have pressed
on, which changes the document. When the question cannot be answered, the
gesture that cannot damage anything is the right default.

# At the chunk rung the OBJECT is not the subject

`Grabbable::bounds` at the Part rung is
[`crate::canvas::selection::SelectionState::outline_union`] of the selected
chunks, so a set built from the first and fifth lines of a note spans the
three unselected lines between them. Asking only *"is the topmost hit one of
my objects?"* answers yes for a press on any of those three — the whole
block is one object — so the press is claimed as a move of the set and
[`crate::canvas::presspick`] never gets to re-pick the line the operator
actually aimed at. O215 ask 1 read through a multi-chunk selection.

So where the chunk boxes are drawn and the rung is Part, membership is asked
of the **chunk** under the point. `None` — inside the block's box but on
no line of it, which is most of a CAD note — keeps the object-rung answer,
which leaves the press in the white between two lines a move exactly as it
has always been.

Eight parameters, for [`look`]'s reason and not a weaker one: each is a
borrow both callers already hold, and the two of them must pass the same
values or the predicate stops being one predicate.

### `struct Press`

Returned as one struct rather than a tuple because four of the five members
are `Option`s of similar-looking types, and a caller that transposed two of
them would compile. Named fields make that a spelling error instead.

### `fn look`

Changes nothing. See the module header for the precedence and for why every
hit test reads `press_origin`.

# Why eight arguments rather than a `Frame` struct

Because every one of them is a borrow the caller already holds and none is a
*product* of anything here — the same call `canvas::painting::draw` makes
when it takes `ui` and `doc` alongside its `Frame`. A struct assembled at the
one call site, passed once, and destructured immediately would be a grouping
that groups nothing.
