# `egui-shell/dock/drag_tests`

## Item notes

### `fn the_dock_publishes_where_it_drew_every_compartment`

The nesting assertions are the ones that matter: a record whose tab rects
were correct but whose stack rect named a different compartment would hit-
test a drop into the wrong stack, and every rect in it would still look
plausible read one at a time.

### `fn a_press_that_does_not_move_activates_the_tab_and_reorders_nothing`

It is also what proves the event pump reaches the tabs at all: every other
test in this file would pass vacuously against a dock whose tabs were never
under the pointer.

### `fn dragging_a_tab_past_the_last_one_moves_it_to_the_end`

Three assertions, because three different builds pass any two of them: one
that previewed and never applied, one that applied and never previewed, and
one that moved the tab but left [`Stack::active`] pointing at whatever is
now at the old index — which switches the panel on screen as a side effect
of rearranging tabs.

### `fn a_drag_pulled_off_the_strip_ends_and_reorders_nothing`

Two failures in one test, because they are the same mistake seen from
either side.

The first is a caret nobody can get rid of: a release read from the tab's
own `Response` never arrives when the pointer left the widget, so the drag
survives into the next frame, and the next, with the strip still painting
its caret and the cursor still `Grabbing`.

The second is a reorder the operator did not ask for. A boundary is
resolved from x alone, and x is perfectly well defined over the middle of
the document — so a drag pulled down into the page would quietly rearrange
the strip it left. What that gesture means is [`super::tear`]'s subject and
is asserted there; what it must never mean is a permutation of the strip the
pointer has left, and that is what this measures.

### `fn a_drag_that_wanders_a_little_below_the_strip_is_still_a_reorder`

The failure it refuses is a caret that blinks in and out along a gesture
that never left the tab bar to the eye — which is what bounding the reorder
by the strip rectangle exactly produces.

### `fn the_caret_marks_the_boundary_the_release_will_use_and_then_goes`

The operator's wording for this feature is *"clear markers of where it is
going to move to"*, and a marker drawn at the wrong boundary is worse than
none: it is a promise the release does not keep. So this asserts the
position, not merely that something was drawn — which the report's own
`gap` field would already have said.

It also asserts the caret **goes**. A region that is published on every
frame is not a marker, it is furniture, and a harness reading it as a
change would then see a drag in flight forever.

### `fn the_caret_at_the_first_boundary_is_drawn_whole`

Boundary zero is the left edge of the first tab, which is the strip's own
left edge, so a caret centred on it has half its width outside the `Ui` that
clips it. The operator then sees the one marker in the strip drawn at half
the weight of every other, at the end where a faint marker is hardest to
tell from none.

The containment has no tolerance, and that is the point: an epsilon here
passes on the build this test refuses.

### `fn the_caret_dims_at_the_two_boundaries_against_the_dragged_tabs_own_edges`

Stated as its own test because the painting reads one predicate and the
reorder reads another — `gap` against `from.tab` here, `gap > from`'s
off-by-one in the model — and the pair a reader most wants proved is that
"dimmed" and "the order did not change" name the same boundaries.
