# `egui-shell/dock/railhide_tests`

## Item notes

### `fn frame`

`reachable` is the set the application claims the rail can raise —
deliberately a parameter rather than "everything", because the `all` vs
`any` distinction below is the whole safety argument and a helper that
could only express "everything" would make it untestable.

### `fn a_stack_whose_panels_are_all_on_the_rail_draws_no_tab_strip`

Two assertions rather than one, and the second is the one that matters. The
count says the dock *decided* to suppress; the absence of every
`dock.tab.*` region says it actually did. A build that incremented the
counter and drew the strip anyway would pass the first alone — and the
counter is the thing a later refactor is most likely to keep while moving
the drawing.

### `fn a_panel_the_rail_cannot_raise_keeps_the_strip_for_the_whole_stack`

The plausible wrong implementation suppresses when the rail covers the
**active** panel, or when it covers *most* of them. Either leaves the
uncovered panel with no switch of any kind: it is not on the rail, and the
tab that was its only other route has just been taken away. That panel is
then reachable by nothing at all — the defect this whole surface was
allowed to be built only because it could be refused mechanically.

The fixture makes the uncovered panel the **active** one deliberately, so a
build that checked only the active tab would also be caught.

### `fn a_side_too_narrow_for_the_rail_keeps_its_tab_strip_at_every_width`

[`rail::resolve_width`] returns zero when reserving 52 pt would leave the
panel body under [`super::plan::MIN_COLUMN_WIDTH`] — *absent rather than
squeezed*. So on a narrow side there is no rail, and a tab strip suppressed
there would leave the stack with no switch at all.

The assertion is therefore an **implication**, checked at every width in a
fine series: *suppressed ⇒ a rail was drawn*. A build that asked "is a rail
configured for this side" instead of "was one drawn" passes at 320 pt and
fails somewhere below it, which is precisely why a two-endpoint test would
have been worthless — the interesting widths are in the middle.

### `fn the_panel_beside_a_hiding_rail_is_the_same_width_revealed_and_hidden`

This is R128 for the rail, and it is the property that makes auto-hide
usable rather than nauseating: the operator's pointer is travelling towards
a control in the panel, and the panel must not move as the pointer passes
the rail's edge. It holds because [`rail::PEEK_WIDTH_PTS`] is reserved from
the SETTING, before the reveal is resolved, and the revealed strip is
painted into an `Area` that allocates nothing.

Read as body rectangles rather than as a claim about the code, at several
side widths, because the arithmetic is per-side and a build that reclaimed
the sliver would differ by exactly ten points — a difference invisible in a
screenshot and obvious in a number.

### `fn reveal`

Two, because [`crate::peek::Peek`] answers from the pointer position **and**
last frame's state: the first frame is the one that reveals, and the second
is the one that draws the revealed strip and reports the geometry.

### `fn a_hiding_rail_always_publishes_a_trigger_wide_enough_to_hit`

The trigger region is published on every frame the side is drawn — hidden or
not — and is never thinner than [`crate::peek::Peek::MIN_TRIGGER_PTS`]. That
is the entire reason it is safe to suppress a tab strip beside a rail that
can hide: the rail is never *gone*, only narrow.

Asserted against the **width of the published rectangle**, not against the
constant. A build that reserved ten points and then published a rectangle
clipped to nothing would satisfy a constants-only test and would strand
every panel on the side.
