# `egui-shell/dock/drop`

## Item notes

### `fn insert_at`

Total rather than panicking on a bad index, but not a second gate:
[`DockLayout::move_panel`] has already asked
[`DockLayout::accepts_drop`], and a `false` from here means those two
answers disagree.

### `enum DropTarget`

Every variant names a **boundary**, not a destination index: `0` is before
the first item and `len` is after the last. The three variants are the three
things a dock can do with a panel — join a group, split a column, or start a
column.

### `fn accepts_drop`

Purely a question about the target's indices: whether the compartment it
names exists, and whether the boundary is within range. It says nothing
about which panel is being dropped, so an overlay can ask it once per
candidate zone while laying the zones out.

### `fn move_panel`

The one verb behind every drop: a tab dragged onto another tab bar, a
panel dropped against a compartment's edge to split it, and a floating
panel dragged back over the dock all arrive here. It is total — an
unknown panel or an out-of-range target is declined, and **a declined
move mutates nothing**, which is what makes it safe to call with a target
that was resolved against a snapshot a frame old.

# What a drop does besides moving the panel

- **The panel becomes the active tab of the stack it joins.** Dropping a
  panel into a group and leaving it behind a sibling's tab is
  indistinguishable, on screen, from the drop having done nothing.
- **The destination side is made visible**, for the same reason
  [`DockLayout::activate`] does it: a panel moved somewhere that draws
  nothing has been hidden, not moved.
- **A new stack or column is created at the default share**, so the
  compartments around it keep their proportions and the newcomer takes an
  even split. The share the panel's old compartment had is not carried
  across, because a share is a weight against *its own* parent's siblings
  and means nothing under a different parent.

A move within a single stack is a reorder, delegated to
[`DockLayout::reorder_tab`] — which preserves the visible panel by
identity, so rearranging a tab bar does not change what is on screen.
That is also the one case where the boundary is counted in a list the
removal shortens; see the module header.

### `fn take_panel`

The half of a close that is not the pruning, shared so the rule for what
becomes active afterwards exists once: removing the active tab selects
the **previous** one, which keeps a run of closes moving leftwards along
the bar instead of marching through tabs the operator has not touched.

Returns where the panel was, or `None` if it was not docked. The layout
is left un-normalized deliberately — [`DockLayout::close`] normalizes
immediately after calling this, and [`DockLayout::move_panel`] needs the
hole to survive until it has inserted.
