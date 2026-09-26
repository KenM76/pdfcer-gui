# `egui-shell/dock/float`

## Item notes

### `fn float_then_dock_puts_the_panel_back_where_it_was`

The single most important property in this module, and the one an
operator will test within ten seconds of finding the command. A
round trip that returned the panel to a *different* stack would be
a command that quietly edited an arrangement nobody asked it to
touch.

### `fn docking_back_brings_the_panel_to_the_front_of_its_stack`

Without this, docking a window into a three-tab stack whose front
tab is something else is a command whose only visible effect is
that a window vanished.

### `fn a_home_that_no_longer_exists_still_docks_the_panel`

The operator floats a panel out of the second column, then closes
everything else in that column so it is pruned, then docks the
float back. The recorded address names a column that no longer
exists, and [`DockLayout::dock_back`] rebuilds it rather than
refusing — it inserts the missing column and stack and clamps only
what is genuinely past the end.

### `fn closing_a_floating_panel_removes_its_float_entry`

A leaked entry would draw a window every frame for a panel the
operator closed, and no command would offer to close it again
because every "is it open" query would say no.

### `fn a_floating_panel_cannot_be_mounted_a_second_time`

The View ▸ Panels group calls `mount` for a panel that is not
showing. If `contains` ignored floats, choosing a floating panel
there would put it in the dock *and* leave it in its window —
two surfaces drawing one panel from two `Ui`s with the same widget
ids.

### `fn collapsing_the_home_side_does_not_hide_a_float`

The float has no side, so collapsing the dock it came from cannot
hide it. Worth pinning because `is_on_screen`'s docked branch
consults `side.visible`, and an implementation that checked the
*home* side would get this wrong in a way nobody would notice until
they collapsed a dock.

### `fn normalize_drops_a_float_whose_panel_is_also_docked`

Not reachable through any verb here; reachable through a
hand-edited `layout.ron`, which is a supported way to configure
this application.

### `fn a_float_survives_serialization`

This is the "remembers, per mode, the way docked width already
does" claim, asserted against the actual serializer rather than
against the intent.

### `fn a_layout_with_no_floating_key_loads_with_no_floats`

The backward-compatibility claim, asserted rather than asserted
*about*. A `layout.ron` with no `floating` key is what every
existing installation has.

### `const DEFAULT_SIZE_PTS`

Narrow and tall, because every panel in a dock was laid out in a
column: a float that opened square would reflow content that has only
ever been measured at dock width, and the first thing the operator
would see is a layout they did not ask for. 320 pt is a little wider
than [`super::model::SideLayout`]'s 280 pt default so a panel that
exactly fitted its dock is not immediately scrolling.

### `const MIN_SIZE_PTS`

A floor, not a preference. A resizable window with no floor can be
dragged down to a title bar and a scrollbar; here the way back exists
(dock it), but the operator has to be able to **read the control that
offers it**, and a 40 pt window cannot show one.

### `const MAX_SIZE_PTS`

Not a limit on what the operator may drag the window to — the window
manager owns that — but a limit on what is written to disk and read
back. A stored 90,000 pt window is not a preference, it is a corrupt
file or an arithmetic accident, and restoring it faithfully would open
a window with no visible edge to grab.

### `const CASCADE_PTS`

Without it, floating three panels in a row stacks three windows at
exactly one point and the operator sees one window and two commands
that appeared to do nothing.

### `const OPEN_INSET_PTS`

The same constant `dialogs::host::placement::OPEN_INSET_PT` uses, and
deliberately the same *number* rather than a shared one: this crate
may not depend on the application, and a shell whose windows opened at
a different inset from the application's dialogs would look like two
programs.

### `const OFFSCREEN_REACH_MONITORS`

See [`honour_position`] for the whole argument. Three, because a
three-monitor row is an arrangement people genuinely have and a
four-monitor row in which the application sits at one end is not one
this heuristic needs to serve — it only has to be **generous enough
that a legitimate second-monitor float is never dropped**, and tight
enough that a position left over from a monitor that no longer exists
usually is.

### `const OFFSCREEN_REACH_FALLBACK_PTS`

`egui 0.35`'s `ViewportInfo::monitor_size` is `Option`, and it is
`None` before the first frame and on platforms that do not report it.
4,000 pt is about two 4K monitors side by side at 100 % scaling, which
keeps the fallback on the generous side of the trade in
[`honour_position`].

### `struct DockHome`

The four indices of a [`super::model::PanelAddress`], **owned and
serialized**. It is a separate type rather than a reuse of
`PanelAddress` for one reason that is worth the duplication:
`PanelAddress` is explicitly a runtime-only answer to "where is this
panel *now*", derived fresh by [`super::DockLayout::find`] every time
it is asked. This is a **stored claim about the past**, it goes to
disk, and it can be stale — three properties `PanelAddress` does not
have and must not grow, because everything that consumes a
`PanelAddress` is entitled to assume it is current.

### `fn origin`

Used only when a float entry is constructed for a panel the layout
does not contain — which [`DockLayout::float`] refuses, so in
practice this is reached only by a hand-written or repaired layout
file.

### `struct FloatingPanel`

Serialized as part of [`super::DockLayout`], so every field here is
part of the on-disk schema and every one of them has to survive a
hostile file — see [`normalize_floats`].

### `fn float`

Returns `false` and changes nothing when the panel is not docked —
which covers both "it is already floating" and "it is not in this
layout at all". Neither is an error for the same reason
[`super::DockLayout::activate`]'s miss is not: the caller's honest
response is to do nothing, and a `Result` would make three call
sites each invent a way of ignoring it.

The order of the two mutations matters and is not
interchangeable. The address is read **before** the removal,
because `close` calls `normalize`, and `normalize` prunes the stack
and the column the panel just left — so an address read afterwards
would name whatever slid into their place.

### `fn float_at`

[`Self::float`] plus the placement, which is the whole difference
between the two routes to this verb: a drag out of the dock ended
somewhere deliberately and a menu row did not. Returns what
[`Self::float`] returns, and places nothing when it refused.

`at` is in **desktop** points, the space
[`FloatingPanel::pos_pts`] is stored in — see its docs for why that
is not the application window's space. Nothing here judges whether
the position is reachable; [`honour_position`] does that on the way
out, on every frame, and a position that was sane when the drag
ended can stop being sane before the window is next opened.

### `fn dock_back`

Returns `false` when the panel is not floating.

# Why this does not simply call [`DockLayout::mount`]

The case here looks exactly like the one the permissive clamp was
designed for and is the opposite of it.

[`DockLayout::mount`] clamps an out-of-range address into the
nearest existing container — *somewhere sensible after the
operator's own arrangement has moved on*, which is right when the
arrangement really has moved on. But **floating a panel that was
alone in its stack prunes that stack**, so the home address is out
of range *because of the float itself*, on the very next frame,
with nothing having moved on at all. Clamping then drops the panel
into its neighbouring stack — and a round trip that merges two
compartments into one without saying so is a command that edited an
arrangement nobody asked it to touch.
[`tests::float_then_dock_puts_the_panel_back_where_it_was`] is the
guard.

⇒ So this **rebuilds** the address rather than clamping into it:
a missing column at `home.column` is inserted, a missing stack at
`home.stack` is inserted, and only the parts that are genuinely
beyond the end are clamped. The panel gets its compartment back.

`MODES_AND_PANELS.md` failure mode #6's rule is the general form —
**restore, do not recompute** — and it applies to structure, not
only to sizes.

# The tab index IS honoured

Appending is the tempting alternative, on the reasoning that a
recorded tab index describes a stack that has since changed. The
reasoning is true and the conclusion does not follow: float the
*first* of three tabbed panels and append it back, and the
operator's tab order has been reordered by a command that promised
to put something back. Inserting at `home.tab`, clamped to the
stack's current length, restores the order when the stack is
unchanged (the common case) and degrades to appending when it is
not.

# It activates the panel afterwards

Docking a window into a stack whose front tab is something else,
and leaving it behind that tab, is a command whose entire visible
effect is that a window disappeared.

### `fn dock_all_floating`

This is the recovery route, and it is the reason it is a public
verb rather than a loop written at one call site. A window that is
off-screen because the monitor it was on has been unplugged cannot
be reached with a pointer, cannot be closed, and — since it is
still in the layout — cannot be re-floated. The operator's only
remaining lever is a command that acts on *all* of them without
having to name one, and it must be reachable from the application
window, which is the one surface guaranteed to be on a monitor that
exists.

Reset-layout is the other route, and it is the stronger one because
it also restores the arrangement — see [`crate::layout::reset`],
which drops the floats in its scope. This one is the *cheap* route:
it costs the operator nothing they arranged.

### `fn set_float_geometry`

Returns whether anything actually changed, so a caller writing this
every frame from the live window geometry does not mark the layout
dirty — and therefore does not trigger a save — sixty times a
second for a window nobody is touching. **That return value is the
whole reason this is a method and not a field assignment.**

Sizes are clamped on the way in rather than on the way out, so the
clamp is applied once at the boundary and everything downstream —
including the serializer — sees a value that is already sane.

### `fn clamp_size`

`NaN` is handled by the `is_finite` test and not by `clamp`, because
`f32::clamp` **panics** on a `NaN` bound and propagates a `NaN` value —
so a corrupt file would either abort the process or store a size no
arithmetic afterwards can recover from.

### `fn normalize_floats`

Called from [`super::DockLayout::normalize`], which is the one place
structural invariants are repaired.

## Which copy wins when a panel is both docked and floating

**The docked one.** The float entry is dropped.

The state cannot be produced by any verb in this module —
[`DockLayout::float`] removes the panel from the tree first and
[`DockLayout::dock_back`] removes the float entry first — so reaching it means a hand-edited file, a
truncated write, or a future merge of two layouts. In every one of
those the tree is the part with more information in it (a column, a
stack, a share, a neighbour) and the float entry is four numbers, so
dropping the float loses less.

It is also the safer half to lose in the way that matters: a panel left
**docked** is visible in the window the operator is already looking at,
whereas a panel left **floating** is visible only wherever the stored
position happens to point — which, if the file was corrupt enough to
produce this state, is not a coordinate to trust.

### `fn honour_position`

Returns the position to open at, or `None` for *"place it as if it had
never been placed"*.

# The problem, stated honestly, including the part that is a guess

A float window's position is stored in desktop coordinates. Desktop
coordinates are only meaningful relative to a monitor arrangement, and
**nothing persists the monitor arrangement** — not this crate, and not
`egui 0.35`, which exposes `ViewportInfo::monitor_size`, a *size with
no origin*, for the monitor the asking viewport is on, and no
enumeration of any others.

So the question *"is `[2400, 300]` on a monitor?"* is **not answerable**
from anything available here. What is answerable is a weaker question:
*"is `[2400, 300]` plausibly on a monitor adjacent to the one the
application window is on?"* — because the application window's outer
rectangle **is** in desktop coordinates and it **is** on a monitor that
exists, by the fact that it is being drawn.

⇒ This function answers the weaker question, and the docs say so
rather than implying the stronger one. A position within
[`OFFSCREEN_REACH_MONITORS`] monitor-widths of the application window
is honoured; one further away is dropped.

# Why the trade is deliberately generous

The two failure directions are not symmetric.

* **Too tight** — a legitimate second-monitor float is dropped and the
  window opens over the application. The operator drags it back to the
  monitor they wanted it on, and the position is re-remembered. Cost:
  one drag, once.
* **Too loose** — a stale position is honoured and the window opens
  where nobody can see it. Cost: a panel that has vanished, a command
  that appears to do nothing, and no way to tell that from a crash.

The second is much worse, but it is also **fully recovered** by
[`DockLayout::dock_all_floating`] and by a layout reset, both of which
are commands on the application window. The first has no such backstop
— nothing can tell that a window is on the *wrong* monitor. So the
bound is set generous, and the deterministic recovery is what carries
the case this heuristic misses.

# Arguments

* `remembered` — the stored position, in desktop points.
* `size` — the window's size, so a window whose *far* edge is beyond
  reach is judged on where it actually ends rather than where it
  starts.
* `app_outer` — the application window's outer rectangle, desktop
  points.
* `monitor` — the size of the monitor the application window is on, if
  the platform said. `None` uses [`OFFSCREEN_REACH_FALLBACK_PTS`].

### `fn opening_position`

Inset from the application window's corner and cascaded, so a run of
float commands produces a run of visibly distinct windows rather than
one window and a mystery.

Deliberately **not** clamped onto the application window, unlike
`dialogs::host::placement::opening`'s chosen-position path. A dialog
belongs to the window that raised it and must stay on it; a float is
the operator asking for a surface that is *not* confined to that
window, and a cascade that stopped at the application's right edge
would pile the fourth window on the first. The cascade is bounded by
[`CASCADE_MAX`] so it cannot run off the desktop either.

### `const CASCADE_MAX`

Eight windows at 28 pt is 224 pt of offset, which is inside any window
big enough to have a dock. Beyond that the cascade stops rather than
walking the ninth window off the screen — the ninth lands on the
eighth, which is an overlap the operator can drag apart, rather than a
window at the bottom-right corner of the desktop.
