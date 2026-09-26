# `egui-shell/dock/overlay_tests`

## Item notes

### `fn two_columns`

`layers` alone in the second left column is the load-bearing part: taking it
out prunes that column, so a drop of `layers` anywhere re-lays the whole left
side. That is the case a preview read off the target's *current* rect gets
wrong, and [`the_outcome_is_the_layout_after_the_take_not_the_target_as_it_stands`]
is what measures it.

### `fn body`

Read through [`super::compass::body_of`] rather than subtracted here,
because a fixture with its own spelling of "the compartment less its strip"
would be aiming at zones a strip's height away from the ones the release is
resolved against, and would report that as a defect in the overlay.

### `fn a_drag_along_its_own_strip_is_not_a_drop`

Two things at once, and they are the same fact from either side: the pump
reaches the tabs at all — without which every absence asserted below is
satisfied by a build that never sensed a drag — and the two affordances do
not both claim one gesture. An overlay that offered a five-zone compass over
the compartment a reorder is passing across would put a wash of colour under
the caret the operator is aiming with.

### `fn dragging_a_tab_onto_another_compartments_centre_joins_that_group`

Four assertions, because four different builds pass any three: one that
offered and never applied, one that applied and never offered, one that left
the panel in both stacks, and one that reported the move as a reorder — which
is a real distinction, because an application telling the operator "moved"
for a rearranged tab bar is reporting a structural change that did not
happen.

### `fn a_release_over_another_stacks_strip_is_a_caret_between_its_tabs`

The compass is not offered there, because a tab strip already answers a
better question than "which of five": it answers *where among these tabs*,
and the operator aiming at a strip is aiming between two labels.

### `fn the_armed_zone_and_the_outcome_are_published_and_then_go`

The visible form of this affordance is a wash of colour over a
quadrilateral: precise to look at, and nothing a harness can assert on. So
the armed zone and the outcome are named, for the reason
`crate::dock::report`'s header gives — and a region published on every frame
would be furniture rather than a marker, which is why the absence after the
release is asserted too.

### `fn the_zones_divide_the_compartment_less_its_strip`

The tab strip is not part of the compass — it answers a better question, and
[`super::compass::body_of`] subtracts it before dividing. A compass laid over
the compartment *including* its strip is drawn one strip height above the
zones the release is resolved against: the operator aims at a painted "top"
band and the pointer is over the tab bar, so the release inserts a tab where
a split was offered. R8b's failure mode #2, in paint.

The assertion is the armed band's **top edge**, because that is what the
wrong mechanism cannot produce: it would start the band at the compartment's
top, a strip's height higher. Containment inside the compartment is satisfied
by both, which is why it is not the measurement.

### `fn the_outcome_is_the_layout_after_the_take_not_the_target_as_it_stands`

`layers` is alone in the second left column, so removing it prunes that
column and the first one grows to the whole side — *before* the panel
arrives. A preview that looked the destination up in the current geometry
would outline half the side and then deliver the whole of it, which is the
disclosure failure this project names failure mode #2, in the commonest drag
there is.

The width comparison is what the wrong mechanism cannot produce: it would
return the destination's rect unchanged, and that rect is measured here
before the gesture starts. The equality after the release is the other half —
the promise was kept, not merely different.

### `fn a_release_that_would_change_nothing_is_offered_dimmed`

Dropping a panel back into the middle of the group it already leads is
legal and permutes nothing. Refusing it would be a lie about the grammar;
promising a move that will not happen is the other half of the same lie. So
the offer stands, `lands` is false — which is what knocks the ink back — and
the outcome outlined is the compartment the panel is already in.

### `fn a_release_over_a_splitter_docks_nothing`

A splitter is recorded in no compartment's rect, so
[`super::DockLayout::resolve_drop`] answers `None` — the same answer as the
canvas. Asserted because the failure mode is the opposite of a refusal: a
resolution that snapped to the nearest compartment would dock a panel the
operator released into a gap on purpose.

### `fn a_release_over_the_document_docks_nothing_and_ends_the_drag`

The canvas is where [`super::tear`] attaches, and what happens there is
asserted in its own file. What must not happen is anything from *this* one:
no compass over a document, no `moved`, and no compartment on the right-hand
side quietly gaining a tab because the nearest stack won by default. The
gesture must also end — a drag that survived its release would carry the
compass into the next frame, and the one after, with no button held to get
rid of it.
