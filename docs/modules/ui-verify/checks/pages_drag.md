# `ui-verify/checks/pages_drag`

`pages_drag_shows_where_it_lands` — dragging a page thumbnail draws an
insertion caret, and releasing puts the page there.

# The gap this closes

The Pages panel sensed `egui::Sense::click()` and nothing else, so
reordering was two ribbon buttons that move the operand set **one place at
a time**. Putting page 40 in front of page 3 was thirty-seven presses, and
the gesture every operator tries first was not sensed at all.

# Why this needs driving, and could not be unit-tested

`ops::drop_order` — the arithmetic — has twenty-one unit tests, including a
sweep asserting every operand set at every gap in a six-page document yields
a permutation. **None of them can fail on a build where the panel does not
sense a drag**, because the arithmetic is reached only by a gesture and the
gesture is three frame-level edges:

1. `Response::drag_started_by(Primary)` on a tile, which settles the
   operand set;
2. a resolved drop target, which exists only inside the layout pass because
   a *gap* has no position until the rows are placed;
3. a release read from raw pointer input, because a drag begun on a tile
   ends anywhere.

# The assertion that is the whole point of the feature

**The caret was drawn.** The operator's request was not "let me drag pages"
— it was *"make indicators to show when moving pages where they are going
to go to."* A drag that reorders correctly and shows nothing while it is in
flight has answered the wrong half.

It is observable after the fact because `crate::diag::ui_rect_visible` is a
**change log**: the caret's region is emitted the first frame it is drawn
and a matching `ui-rect-gone` is emitted when it stops being drawn. So a
completed drag leaves both lines in the trace, and their presence is proof
the indicator existed during a gesture no screenshot could have caught
mid-flight.

# What a passing run does NOT prove

That the caret was in the right place, or the right colour. The trace gives
its rectangle and this check reads it — it asserts the caret sat **inside
the grid** and had a non-zero height, which rules out the two failures that
produce a region and no visible mark. It does not assert which boundary it
sat on; that is what the `gap=` field on the release line is for, and the
two are cross-checked below.
