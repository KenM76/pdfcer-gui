# `ui-verify/checks/tab_order_drag`

`tab_order_drag_moves_a_field_and_shows_where` — dragging a row in the
Tab-order list draws an insertion caret, and releasing commits the new
`/Annots` order.

# The gap this closes

The operator asked for this by name and named the reference himself:

> *"the tab order list is supposed to be able to be reordered by dragging
> and dropping rows around like we can with pages in the page preview, and
> have **clear markers** of where the field is going to move to."*


# Why this cannot be a unit test, and the shape of the failure it catches

[`crate`]'s standing lesson, in its sharpest form. The permutation
arithmetic — `panels::forms::tab_order::drag::reordered` — has seven unit
tests, one of them exhaustive over every `from`/`gap` pair on a list with
interleaved non-widget entries. **Not one of them can fail on a build where
the row does not sense a drag**, because the arithmetic is reached only by a
gesture, and the gesture is three frame-level edges the harness alone can
produce:

1. `Response::drag_started()` on a row — which requires the row to have been
   built with `Sense::drag()` rather than as a plain `ui.label`, and a plain
   label is exactly what it was for the whole of its previous life;
2. a resolved drop target, which exists only inside the layout pass because
   a *gap* has no position until the rows are placed;
3. a release read from raw pointer input, because a drag begun on a row ends
   anywhere.

This is the same shape as the two founding defects: a green suite over a
feature that does nothing when a human touches it.

# The assertion that is the point of the feature

**The caret was drawn.** He did not ask to be able to drag rows; he asked
for *"clear markers of where the field is going to move to"*. A drag that
reorders correctly and shows nothing while it is in flight has answered the
wrong half of the request — and it is the half that is invisible to every
other kind of test, because the caret exists only while the pointer is down.

It is observable after the fact because `crate::diag::ui_rect_visible` is a
**change log**: the region is emitted the first frame the caret is drawn and
a matching `ui-rect-gone` when it stops. So a completed drag leaves both
lines behind, and their presence is proof the marker existed during a
gesture no screenshot could have caught mid-flight. Hence
[`declared_since`](crate::checks::driving::declared_since) and never
`declared`, which asks the present tense of a thing whose nature is to be
gone.

# What a passing run does NOT prove

* That the caret was in the right *place*. This reads its rectangle and
  asserts a non-zero **width** and an overlap with the row it was dropped
  on, which rules out the two failures that publish a region and draw
  nothing an operator could see. Width, not height: this caret is a
  horizontal line, where the page rail's is vertical.
* That the file on disk is correct. The check stops at the engine's own
  applied line and its `moved=` count. Whether the bytes are right is
  `pdfcer-core`'s twenty-odd tests for the verb, not this harness's job.
* That `/Tabs` was left alone. Nothing here observes the page dictionary,
  and it deliberately does not try: the sourced reason `/Tabs` must not be
  written by a drag lives in `panels::forms::tab_order::drag`'s header, and
  the place to hold the engine to it is `pdfcer-core`'s own dirty-set test,
  which asserts the reorder touches exactly one object.

What it DOES prove about correctness, beyond the gesture, is one thing, and
it is step 6's last assertion: **`non_widgets=0`**. `/Annots` order is paint
order, so a permutation that moved a `/Link` would change what is drawn over
what — a visible change to the page, produced by a gesture whose whole
subject was tab sequence. That the count is zero is the one correctness
claim this harness is in a position to make.
