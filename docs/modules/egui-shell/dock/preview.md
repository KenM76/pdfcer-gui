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

### `fn preview_drop`

`None` when the panel is nowhere to be found, when the geometry has no
rect for the side it would land on, or when the layout is empty. A
*declined* move is not one of those: a target naming where the panel
already is returns that compartment, so the overlay shows the honest
no-op rather than blinking out.

### `fn drop_lands`

The overlay knocks its highlight back where the answer is `false` — the
releases that are legal and permute nothing, which are common: a panel
dropped back into the middle of the group it already leads, or against
the edge of a column it is already alone in.

Asked by applying the same verb the release will apply, to a clone, so
the dimming cannot disagree with the outcome. A predicate written out by
hand would be a second description of the drop grammar, and the
grammar's exceptions — a reorder inside one stack, an insertion into a
column the take has just emptied — are exactly what it would get wrong.
