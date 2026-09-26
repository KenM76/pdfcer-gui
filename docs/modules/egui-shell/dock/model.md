# `egui-shell/dock/model`

## Item notes

### `fn a_gap_is_a_boundary_so_a_rightward_move_lands_one_short_of_it`

Dropping tab 0 at gap 3 lands it at index 2, not 3, because removing it
shifts every tab to its right down by one. Testing both directions in
one test is deliberate: the conversion is `if gap > from { gap - 1 }`,
and a build that omitted it entirely passes every leftward case.

### `fn a_tab_dropped_against_either_of_its_own_edges_does_not_move`

Both sides matter: gap `n` is the tab's own left edge and gap `n + 1`
is its right, and a release anywhere over the dragged tab produces one
or the other.

### `fn reordering_keeps_the_same_panel_on_screen`

The bug this refuses is silent and looks like the dock switching panels
on its own: leave `Stack::active` as an integer across a reorder and
whatever tab lands on that index becomes the one on screen. Two cases,
because each alone is passed by a plausible wrong build — one where
active follows the dragged tab always, one where it never moves.

### `fn an_out_of_range_reorder_reports_failure_rather_than_panicking`

A gesture resolves its address from the previous frame's rectangles, so
a stack emptied or a side collapsed between the press and the release
is reachable, not hypothetical.

### `fn a_backgrounded_panel_can_be_brought_forward`

Kept even when the shipped default arrangement happens to have no
backgrounded tab: without it the function is effectively untested
and an edit can break it with every test still green.

### `fn a_panel_mounted_twice_keeps_only_its_first_mount`

Two live copies of one surface each have their own scroll position
and their own idea of which tab is active, and `activate` raises
whichever it finds first. The model **repairs** it rather than a
test merely forbidding it, because the input that causes it is a
hand-edited file that no test of the defaults can reach.
