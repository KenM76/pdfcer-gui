# `egui-shell/dock/tear_tests`

## Item notes

### `fn a_drag_along_its_own_strip_offers_no_window`

Two facts at once, and they are the same one from either side: the pump
reaches the tabs, and the tear does not claim a gesture the strip has
already claimed. An outline appearing under a reorder caret would be
offering to make a window out of a tab the operator is nudging one place
along.

### `fn a_drag_carried_out_of_the_dock_offers_a_window`

The outline is the disclosure: a release that made a window with no warning
would be the whole gesture happening after the fact. It is drawn *around*
the pointer rather than beside it, so the thing about to be made is under
the hand making it.

The control is the first gesture: over another compartment the compass has
it and the tear does not, so the outline appearing a frame later is a fact
about where the pointer went and not about a build that offers one always.

### `fn a_release_outside_the_dock_makes_the_window_the_outline_promised`

The stored position is asserted against the *reported* one rather than
against a coordinate computed here: the two are in different spaces — the
outline is in this frame's screen points and the window is placed in desktop
points — and the claim worth making is that the offer and the outcome are
the same quantity, not that a headless frame happens to put the origin at
zero.

### `fn the_torn_panel_remembers_the_compartment_it_came_from`

The half of [`super::float`]'s central decision that a drag has to honour as
much as the command does: docking it back puts it where it came from, and a
tear that recorded no home — or recorded the address *after* the removal
pruned its column — would put it somewhere else.

### `fn the_dock_edge_handle_is_not_a_tear_zone`

A side's width handle lies outside the rectangle the geometry records for
that side — it sits on the edge facing the document — so a predicate asking
only *"is the pointer inside a side"* would lay a few points of tear zone
down the whole height of the dock's inner edge, which is the strip every
drag crossing from the dock to the document passes through.

### `fn the_outline_is_published_and_then_goes`

An affordance whose whole form is an outline drawn during a gesture cannot
be captured after the gesture, so the report is the checkable half — and a
report surviving the release would be a promise of a window the operator has
already been given.

### `fn a_drag_carried_back_onto_the_dock_docks_it`

The end-to-end claim the other tests each hold one end of: an offer made on
one frame is not a commitment, and the affordance that answers the release
is the one live on the frame the button came up.

### `fn a_drag_over_another_strip_offers_no_window`

A strip that is not the drag's own is the compass's stripless case — an
insertion caret between two of that stack's tabs — and it lies at the very
top of the compartment, a few points inside the dock's outer edge. A
predicate testing the pointer against each compartment's *body* rather than
against the side would offer a window along every tab bar in the dock.
