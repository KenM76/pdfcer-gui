# `egui-shell/dock/preview`

## Item notes

### `fn walk`

The offset steps over a splitter after every span, including the last —
where nothing reads it, which is why this is the same walk the draw path
performs with the step inside an `if`.

### `fn layout`

The lone stack is the one the tests drag, because taking it out
**removes its column** — so a preview that merely echoed the target's
current rect would be wrong by half the side's width.

### `fn the_walk_leaves_exactly_one_splitter_between_compartments_and_no_slack`

It compares the walk against a frame that was drawn *by the walk*, so
an error in the step — dropping the splitter, double-counting it —
moves both together and every rect still matches. The step is
therefore asserted directly: compartments sit one splitter apart and
together fill the area they divide, with nothing left over at either
end.

### `fn the_frame_draws_every_compartment_where_the_walk_says_it_does`

Everything below asks the walk where a compartment *would* be. This
asks whether the walk agrees with where the dock *did* draw one — so
if the draw path ever stops calling [`columns_across`] and
[`stacks_down`], the divergence is red here rather than shown to an
operator mid-drag.
