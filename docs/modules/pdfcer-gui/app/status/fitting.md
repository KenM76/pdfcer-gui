# `status::fitting` — what the bar can afford to show when the window is narrow

## The defect this exists for


```text
status-group:page    457.4 .. 603.1     ok
status-group:zoom    326.7 .. 435.4     ok
status-group:fit       6.6 .. 304.7     ok, and 298 pt wide — half the bar
status-group:find    -54.2 ..  -15.4    off the left edge
status-group:filter -127.5 ..  -76.2    off the left edge
```

**Find and the selection filter were unreachable**, and `status-group:notes`
sat at 8 .. 114, underneath the fit group, so the left-hand notes and the
right-hand cluster were drawn on top of each other.

It is the redaction-apply defect's shape — a control declared outside the
body it lives in — reached by *scaling* rather than by adding copy, which is
why no layout test saw it. Every test in this crate measures at
`ui_scale = 1.0`.

## Why the existing argument did not hold, and it is worth reading

`status.rs` already argued the case, correctly, for the mechanism it chose:

> *"A right-to-left layout cannot get that wrong; it simply runs out of
> room, and the notes on the left are what yields."*

Right-to-left **is** the right layout, and the alternative it rejected —
`right − width`, which goes negative the moment the bar is narrower than its
content — is genuinely worse. What the argument missed is that *running out
of room* is not a graceful state. egui does not clip a `Layout` to its
parent; a right-to-left run whose content exceeds the available width simply
continues past the left edge into negative coordinates. The notes do not
yield, they are **overdrawn**.

So the layout was never wrong. What was missing was anybody asking *does
this fit* before adding the next thing.

## The rule, and where it comes from

**Shed from the low-priority end until the rest fits.** That is what a
status bar does everywhere it is done well — Word drops items as the window
narrows, VS Code hides them by declared priority, and browsers do the same
with their own. Nothing announces it, and nothing should: a status bar is a
summary surface, and an overflow chevron on one is a control about a control.

**Relative order survives any subset**, which is what makes shedding by
priority safe rather than disruptive. A right-to-left run draws whatever it
is given in the order it is given; removing an item from the middle closes
the gap without moving anything past anything else. So `Filter · Zoom ·
Page` reads left to right exactly as `Filter · Find · Fit · Zoom · Page`
does, minus two. `status::fit`'s warning about not reordering controls the
operator has learned is about the four fit buttons **among themselves**; it
does not bind here.

## THE CLAUSE THAT DECIDES EVERYTHING: nothing may shed its only home

This is what makes the design legitimate rather than convenient, and it is
**enforced, not asserted** — [`SHED_ORDER`] names each group this module may
drop beside a command that still reaches it, and
[`tests::nothing_sheddable_loses_its_last_route`] resolves every one through
the real command registry.

**That test immediately refused the first design, and it was right.**
The obvious rule — shed from the low-priority end, which in this layout is
the left — would have dropped the **selection filter** first, since it is
leftmost. The filter has **no ribbon command, no menu entry and no
shortcut**: it exists only on this bar. Shedding it makes it unreachable,
which is the very defect being fixed, moved from 1.80 scale to 1.60.

Checking the rest against the registry left exactly two groups that may go:

| group | pt | may be shed? | why |
|---|---:|---|---|
| Fit | 298.1 | **yes** | all four buttons are View ▸ Zoom commands |
| Find | 38.8 | **yes** | `edit.find`, and `Ctrl+F` |
| Zoom | 108.7 | no | `+`/`−` have no command; the readout has no other home |
| Filter | 51.3 | no | **no other home at all** |
| Page | 145.7 | no | the only answer to *which page am I on* |

And it is enough: at the 611 pt width that found the defect, dropping the
fit group alone takes the cluster from 666 pt to 362 pt.

Two of the five being unsheddable is a finding about the **ribbon**, not
about this module, and it is recorded rather than worked around: the
selection filter and the zoom stepper are status-bar-only capabilities. If
either acquires a home, it can join the list — one line, and the test will
confirm it.

## Measuring: last frame's rect, not a recomputed estimate

[`Widths`] remembers what each group actually occupied on the previous frame,
taken from the same rect the group already publishes for the harness. The
alternative — a `min_width()` per group that re-measures its own labels —
is a second implementation of egui's layout that would drift from the first
the day a separator's padding changed, and it would drift *silently*, in the
direction of a bar that thinks it fits and does not.

The cost is one frame: a window resized in a single step shows the old
decision once before correcting. During a drag that is invisible, and on the
first frame after start-up the bar simply shows everything, which is the
behaviour it had before this module existed.

## Item notes

### `const SEPARATOR_PTS`

egui's `ui.separator()` is a fixed spacing plus a hairline and does not vary
with content, so unlike the groups themselves it is a constant rather than a
measurement. Six points is `Spacing::item_spacing.x` (4) plus the rule (2)
at this crate's theme; it is deliberately a slight **over**-estimate, which
biases the decision towards shedding one group too early rather than one too
late. Too early costs a control that is reachable elsewhere; too late puts
it at negative x, which is the defect.

### `fn measured_width`

A group with no remembered width contributes **nothing**, which is what
makes the bootstrap work: on the first frame nothing has been measured, the
total is zero, and everything is shown — so everything gets measured. See
[`Widths`].

### `fn measured`

Taken from the trace of the failing run rather than invented, so the
thresholds below are the real ones: page 145.7, zoom 108.7, fit 298.1,
find 38.8, filter 51.3 — measured at `ui_scale = 1.80` in a 611 pt bar.

### `fn at_the_scale_that_found_the_defect_the_cluster_sheds_rather_than_overflows`

611 points is the client width the failing run reported. The whole
cluster needs 145.7 + 108.7 + 298.1 + 38.8 + 51.3 = 642.6 plus four
separators = 666.6 — so it cannot fit, and before this module it did not
fit *and was drawn anyway*, at negative x.

Dropping the fit group alone takes it to 362.5, which is what the
assertion below checks: the biggest, least essential group goes, and
nothing else has to.

### `fn every_width_keeps_the_groups_in_order`

The property that keeps the bar's controls in the positions the operator
learned. Shedding is by priority, not by position, so the result is not
a prefix — but whatever survives must still read in the same order.
Swept across every width from nothing to generous, because the
interesting failures are at the boundaries and picking two points either
side of a transition looks exactly like no transition at all.

### `fn a_group_with_no_other_home_survives_every_width`

The clause the whole design rests on, swept rather than sampled. The
selection filter has no ribbon command, no menu entry and no shortcut —
it exists only on this bar — and the zoom stepper's `+`/`−` have no
command either. Dropping either would move the very defect this module
fixes from 1.80 scale to a narrower one, which is not a fix.

### `fn a_narrower_bar_never_shows_more`

A monotonicity property, and the one a hand-written threshold ladder
would break first: as the bar narrows, the set shown must only ever
shrink. A bar that dropped Find at 600 pt and showed it again at 590
would flicker as the window is dragged.

### `fn nothing_is_shed_before_it_has_ever_been_measured`

Without this the first frame would shed every group whose width is
unknown — which is all of them — and they would never be drawn, so they
would never be measured, and the bar would be permanently empty. A
bootstrap that cannot bootstrap.

### `fn nothing_sheddable_loses_its_last_route`

The clause that makes shedding legitimate, checked against the real
command registry rather than against a comment. A group whose ribbon
home was renamed or deleted would fail here — loudly, in a unit test —
rather than quietly becoming unreachable at a UI scale nobody on this
project runs at.

### `enum Group`

A plain enum rather than the region-name strings, so a caller cannot ask
about a group that does not exist and the exhaustiveness of [`SHED_ORDER`] is
the compiler's problem rather than a reviewer's.

### `fn region`

Deliberately the **published** name rather than a private key: the width
is remembered from the rect the harness reads, so one name for both
makes it impossible for the two to describe different widgets.

### `const SHED_ORDER`

Ordered by *what dropping it buys against what it costs* — the fit group is
298 points, nearly half the bar at the width that found the defect, and its
four buttons are all View ▸ Zoom commands; Find is 39 points and is a
keystroke away. Everything not in this list is undroppable, and the module
header's table says why for each.

The second field is a command id and it is **checked against the real
registry** by [`tests::nothing_sheddable_loses_its_last_route`], which is
what stopped the first version of this list shedding a control that has no
other home anywhere in the program.

### `struct Widths`

A `BTreeMap` over five keys rather than a struct of five `f32`s: the map is
**partial**, and that is the point. A group that has never been drawn — no
document open, or the first frame after start-up — has no entry, and
[`affordable`] treats an unknown width as *"show it and find out"*, which is
exactly right for a bar that has not yet been measured.

### `fn record`

Non-finite and negative widths are dropped rather than stored: egui can
report a degenerate rect for a widget laid out in a zero-width parent,
and a `NaN` in here would poison every comparison in [`affordable`] into
answering `false` — which would shed the whole cluster permanently, from
one bad frame.

### `fn affordable`

Returns them in [`Group::ORDER`] — the order `status::bar` must add them —
with the undroppable ones always present and the droppable ones removed, in
[`SHED_ORDER`], until the rest fits.

# It can return more than fits, and that is deliberate

If every droppable group is gone and the remainder still overflows, the
remainder is returned anyway. There is nothing left this function is allowed
to drop — each of the three is the operator's only route to what it says —
and a window that narrow is past what any shedding rule can rescue. A bar
that overflows by a little is a better outcome than one that has thrown away
the page number, and the alternative is to start making a capability
unreachable, which is the defect this module exists to prevent.

### `fn still_reachable_at`

The read side of [`SHED_ORDER`], used by [`trace_shed`] so a diagnostic
naming a dropped control also names where it went. `None` for a group that
is never shed.

### `fn trace_shed`

Emitted on change only, from `status::bar`, once the decision is made.

# Why this is traced when nothing else about the bar's layout is

Because **absence is not evidence**, and a driven check has nothing else to
read. A shed group publishes no `ui_rect` — but neither does a group that is
merely scrolled out of view, nor one whose widget failed to build, nor one
the mode does not offer. Four causes, one symptom, and a harness reading
only the region list cannot tell them apart.

So the bar states its own decision. `status-shed groups=none` is the
ordinary case and says the window is wide enough; anything else names what
went and where an operator can still reach it, which is the fact the
shedding rule's legitimacy rests on.
