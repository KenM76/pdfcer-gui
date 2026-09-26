# `egui-shell/ribbon/overflow`

## Item notes

### `fn region`

The RIGHT arrow's name is `ribbon.overflow`, which names the question
rather than the glyph. Region names are a **cross-repo stability
contract** with `tools/ui-verify`, and what those checks ask of this
control is: is the affordance on screen at every width, is it
hit-testable under real metrics, does any visible group overlap it.
None of that is a claim about the mechanism, so naming the region after
the mechanism would put an implementation detail into a cross-repo
contract.

The left arrow answers a question nothing asked before, so it carries a
name of its own.

### `fn arrow_width`

Deliberately a function of the theme rather than a constant: the arrow sits
in a row of controls and an arrow that did not scale with them would be a
misalignment at every scale but one.

### `fn clamp`

# Why this is a pure function and why it runs every frame

A remembered scroll position is an input to layout. Widen the window and a
position that was correct becomes one that leaves blank space at the right
of the band — the group list ends before the viewport does. The operator did
nothing wrong and there is nothing for them to press.

The tempting fix is to notice the blank space after drawing and pull the
band back. That is a measurement feeding the size that produced it — the
feedback loop R128 forbids. So instead: compute, from
the offered width and the group widths alone, the largest `first` at which
the remaining groups still reach the right edge — and clamp to it before
anything is drawn.

Walks from the end backwards, accumulating until the budget is exceeded.
The last index that fitted is the answer.

### `fn arrow`

`rect` is computed by the caller from the band's own edge **before any group
is laid out**, for the reason `plan`'s header gives: the affordance must
not be the thing that gets squeezed out when the band is short of room,
because it is the only way back.

Returns the `Response`, not a bare `clicked()`. The band publishes its `Id`
as `BandOutcome::overflow_id`, which is how a driven check finds the control
to click without knowing where it is; two tests fail loudly if it stops
being honoured.
