# `egui-shell/dock/banner`

## Item notes

### `type BannerHandler`

Called at most once per side per frame, with a [`egui::Ui`] whose
`max_rect` **and clip rectangle** are the strip. The clip is the load
bearing half: a caller that draws two rows into a one-row strip gets the
second row clipped away rather than pushing the columns down. Content that
drives the height of the region containing it is a feedback loop this crate
is arranged to make unwritable.

### `const MIN_HEIGHT`

One row of ordinary text plus its padding. Below this the strip cannot
carry a sentence, and a strip that cannot carry a sentence is a coloured
band — see the module header on why that is worse than nothing.

### `const MAX_FRACTION`

Chrome that could take a quarter of the dock is already too much; this is
a backstop against a caller passing a nonsense height, not a design
target. A realistic banner is one row.

### `fn resolve_height`

Returns `0.0` when the request cannot be honoured at all, and a caller
seeing zero draws **nothing** — no rectangle is published and the columns
get the whole side back.

# Why zero rather than [`MIN_HEIGHT`] when the side is short

Because the alternative is a strip that exists in the trace and not on the
screen. A banner squeezed into a side too short for both it and a panel
publishes a region, satisfies every reachability check, and shows the
operator a sliver. A surface with a healthy rectangle and nothing legible
in it is indistinguishable from a working one everywhere except the screen,
so it is refused outright.

### `fn draw`

Returns `area` unchanged when there is no banner for this side or the
height resolved to zero — so the no-banner path costs one comparison and
changes no geometry, which is what keeps every existing layout test valid.

# The region is published against the side's `Ui`, not the child's

[`report::Reporter::report`] owns the rule: a region reported against a
clip derived from itself can only ever measure fully visible, which is a
tautology rather than a measurement. The question asked of
`dock.<side>.banner` is *can the operator see this strip*, and only the
side's clip can answer it — in a window narrower than
[`super::plan::MIN_SIDE_WIDTH`] the side is drawn at the floor and clipped,
and the banner goes off screen with an entirely ordinary rectangle.
