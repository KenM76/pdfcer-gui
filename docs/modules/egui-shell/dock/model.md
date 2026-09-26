# `egui-shell/dock/model`

## Item notes

### `fn a_gap_is_a_boundary_so_a_rightward_move_lands_one_short_of_it`

Dropping tab 0 at gap 3 lands it at index 2, not 3, because removing it
shifts every tab to its right down by one. Testing both directions in
one test is deliberate: the conversion is `if gap > from { gap - 1 }`,
and a build that omitted it entirely passes every leftward case.

### `fn a_tab_dropped_against_either_of_its_own_edges_does_not_move`

Both sides matter: gap `n` is the tab's own left edge and gap `n + 1`
is its right, and a release anywhere over the dragged tab produces one
or the other.

### `fn reordering_keeps_the_same_panel_on_screen`

The bug this refuses is silent and looks like the dock switching panels
on its own: leave `Stack::active` as an integer across a reorder and
whatever tab lands on that index becomes the one on screen. Two cases,
because each alone is passed by a plausible wrong build — one where
active follows the dragged tab always, one where it never moves.

### `fn an_out_of_range_reorder_reports_failure_rather_than_panicking`

A gesture resolves its address from the previous frame's rectangles, so
a stack emptied or a side collapsed between the press and the release
is reachable, not hypothetical.

### `fn a_backgrounded_panel_can_be_brought_forward`

Kept even when the shipped default arrangement happens to have no
backgrounded tab: without it the function is effectively untested
and an edit can break it with every test still green.

### `fn a_panel_mounted_twice_keeps_only_its_first_mount`

Two live copies of one surface each have their own scroll position
and their own idea of which tab is active, and `activate` raises
whichever it finds first. The model **repairs** it rather than a
test merely forbidding it, because the input that causes it is a
hand-edited file that no test of the defaults can reach.

### `struct PanelId`

The shell stores it, compares it, serializes it and hands it back to
the application's body callback. It never interprets it. See this
module's header on why that is a hard rule rather than a style
preference.

Ordering and hashing are derived so a layout can be diffed and a set
of panels can be addressed cheaply; the ordering is lexicographic on
the id and carries no meaning of its own.

### `enum DockSide`

Two sides, deliberately, and not four. `MODES_AND_PANELS.md`'s peer
table shows one product with four edges and every other with two; a
top or bottom dock competes for space with the ribbon above and the
status bar below, and the shell already owns both of those. Adding
them later is a variant plus two match arms, and the serialized form
names sides by keyword rather than by index precisely so that adding
one does not renumber the others.

### `fn is_empty`

A side with no columns draws **nothing at all** — not an empty
panel with a border. An empty container that still takes space is
how an application ends up with a permanent grey stripe nobody can
remove, and it is the same defect as a ribbon group with no items
still drawing its caption.

### `struct PanelAddress`

Returned by [`DockLayout::find`]. Positional rather than by handle,
because a handle would be exactly the kind of identifier this model
exists not to have — see the module header.

### `fn empty`

Not [`Default`], which gives two *visible* empty sides — the
difference matters because `Default` is what a deserializer
reaches for when a field is missing, and a file that omits the
right dock should get a right dock that draws nothing rather than
one that draws a grey stripe.

### `fn panels`

The floats are included, and that is the whole reason this is
worth a doc comment. Three consumers depend on it and all three
would be wrong without it:

* [`Self::contains`], and through it [`Self::mount`] — so choosing
  a floating panel from a View ▸ Panels menu cannot mount a second
  copy of it into the dock while the first is still in a window.
* [`Self::unregistered_panels`] — so a float naming a panel this
  build cannot draw is reported, rather than being a window that
  opens with nothing in it.
* A caller asking *"what does this layout hold"* before saving a
  workspace's `known_panels`.

[`Self::docked_panels`] is the narrower question, for the callers
that genuinely mean *in a stack*.

### `fn is_active`

The honest answer to "is the operator looking at this panel", and
the query a command like "show Properties" must consult rather than
keeping a boolean of its own. A separate `properties_open` flag is a
second copy of one fact and can disagree with what is on screen, and
a control whose selected state is a stale copy of the truth is worse
than one with no state at all.

Note the deliberate limit of the claim: it does not consider
whether the side is visible, because that is a second, separately
meaningful fact. [`Self::is_on_screen`] answers the conjunction.

### `fn activate`

Returns `false` if the panel is not mounted, which **must not be
an error**: the caller's fallback is to mount it or to restore a
default arrangement, not to refuse. That is a statement about the
caller's options rather than about the tree, so it holds whatever
the tree is made of.

Also makes the side visible, because "show me the Layers panel"
meaning "select its tab inside a dock you cannot see" is a command
that from the operator's side did nothing at all.

### `fn close`

Returns `false` if it was not mounted. Removing the tab and deciding
what becomes active afterwards is [`DockLayout::take_panel`], which
carries that rule; this verb is that plus the pruning.

### `fn reorder_tab`

Returns whether the tab order actually changed.

# `gap` is a boundary, and the conversion to an index is the whole
body

`0` is before the first tab; `tabs.len()` is after the last. The tab is
removed before it is re-inserted, so every boundary to the **right** of
`from` is one larger than the index the panel ends up at, and every
boundary to the left is the index itself. Getting that wrong is an
off-by-one in one direction only — a tab dragged leftwards lands
correctly and a tab dragged rightwards stops one short — which reads as
a sticky drag rather than as a bug in arithmetic.

# The active tab is preserved by IDENTITY, not by index

[`Stack::active`] is an index, so reordering moves it under the panel it
names. Remembering the active `PanelId` across the move and looking it up
again afterwards is the only spelling that cannot silently switch which
panel is on screen — and switching the visible panel as a side effect of
rearranging tabs is a change the operator did not ask for and nothing
announced.

Out-of-range arguments are a no-op returning `false`, not a panic: the
caller is a gesture resolved against a snapshot, and the layout it names
may have been edited by a command in the same frame.

### `fn mount`

The permissive shape is deliberate: this is what an application
calls when it wants a panel *somewhere sensible* after the
operator's own arrangement has moved on. A version that refused an
out-of-range address would push that fallback logic into every
caller, where it would be written five times and differently.

### `fn unregistered_panels`

The question *"is anything mounted that nothing can draw?"*, asked
from the side that actually knows the answer. A shell with a closed
panel enum would sweep its own variants; this one has no list of its
own to sweep, because it has no list.

### `fn normalize`

See the module header for the table of what is repaired and why.
Returns nothing: a caller that wants to know *what* was repaired
uses [`crate::layout`]'s loader, which performs the same repairs
item by item and reports each one as a
[`crate::layout::LayoutSkip`]. This method is the silent form, for
the paths where there is no operator to tell — an application's
own programmatic edit, or a close that emptied a column.

### `fn is_normalized`

Exists so a test can assert *"normalize is idempotent"* and so an
application can assert its own built-in default is already clean
rather than relying on a repair pass to make it so — the same
posture `manifest`'s merge takes towards the built-in layer, and
for the same reason: a defect in a compiled-in constant should
fail a test, not be quietly patched on every machine that runs it.

### `fn drawn_side_width`

Applies the presentation clamp described on
[`super::plan::MAX_SIDE_FRACTION`]. Deliberately a pure function
taking `&self`: it cannot write the clamped value back even by
accident, which is the property failure mode #6 turns on.

### `trait PanelCatalog`

The exact shape of [`crate::manifest::CommandCatalog`], and for the
same reason: it lets a layout be loaded, validated and diffed by a
tool that has no application at all — a schema linter, a diff viewer,
a harness inspecting a saved workspace without linking the binary.

### `struct AnyPanel`

For tests and for tooling that has no registry. Using it in
production would disable the check that turns a stale panel id into a
disclosed skip instead of an empty compartment, which is why it is a
named type at a call site rather than a default.

### `struct PanelInfo`

The shell needs three strings to draw a tab: a label, a tooltip, and
the id it already has. It needs nothing else, and asking for nothing
else is what keeps [`PanelId`] opaque.

### `struct PanelRegistry`

Populated at runtime, exactly like [`crate::commands::CommandRegistry`]
— and for the reason `SHELL_FRAMEWORK.md` §7 gives: *a capability's
presence is expressed by registering it, and by nothing else.* A panel
belonging to a feature that was compiled out is simply not registered,
its saved mount is dropped with a disclosed reason, and no `#[cfg]`
appears anywhere in this crate.

### `fn register`

Replacement rather than rejection, unlike the command registry: a
command's handler is behaviour and two of them is a genuine
conflict, whereas a panel entry is three strings and the last
caller wins harmlessly. An application that wants strictness can
check [`Self::get`] first.

### `fn thin_tooltips`

A helper the *application* asserts on, because the application owns
the strings. The rule it encodes: a tooltip states **when to reach
for** a surface, and a tooltip that restates the label has spent a
disclosure opportunity on nothing.

The threshold is twenty characters beyond the label. It is a
heuristic and it is deliberately generous; its job is to catch
`tooltip: "Pages"`, not to grade prose.
