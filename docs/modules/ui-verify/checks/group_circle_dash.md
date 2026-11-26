# `a_groups_circle_line_type_reaches_its_members`

**Asks:** O461. The circle a radius or diameter ce dimension draws takes its
line type apart from the dimension, and a group sets it for its members.

**Drives:** in the dimension-groups panel, on the default group:

1. Tick the circle's *Set by group* dash box (`dimension-groups.circle.dash.set`).
2. Open its chooser.
3. Pick Phantom.

**Passes when** every member's newest `dimension-member-style` line after the
pick carries all four of:

- `circle_dash=group`: the value came from the group tier.
- `circle_line_type=phantom`
- `circle_solid=false`: the resolved circle stroke is dashed.
- `solid=true`: the member's own lines are untouched.

**Why `solid=true` is asserted.** Without it, a row that wrote the group's own
`dash` would pass. That wrong row would turn every member's lines Phantom
along with its circle.

The fixture's members are linear. The engine resolves a circle stroke for
every kind, so the cascade is measured on them even though none draws a
circle.
