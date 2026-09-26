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
