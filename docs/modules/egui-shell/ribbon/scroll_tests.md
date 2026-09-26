# `egui-shell/ribbon/scroll_tests`

## Item notes

### `fn no_visible_group_overlaps_the_left_scroll_arrow`

The right-hand affordance's reservation is taken from the band's right edge
before any group is laid out. The left arrow needs the mirror of that: its
rect is `full.min .. full.left() + reserve`, it is drawn **after** the
groups, and unless `groups_rect` starts past it the two overlap exactly.

What that costs an operator is worse than a cosmetic overlap. The arrow
wins the hit test, so on a scrolled band **the leading control is
unreachable and clicking it scrolls the ribbon instead** — a control that
looks normal, is drawn normally, and does something entirely different from
what it says.

The band must be SCROLLED for the left arrow to exist at all, which is why
this test drives a click rather than merely rendering — and why no
width-sweep test can stand in for it: those render a fresh, unscrolled band,
and there is no width at which an unscrolled band draws a left arrow.
