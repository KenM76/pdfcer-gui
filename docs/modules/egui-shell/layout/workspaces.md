# `egui-shell/layout/workspaces`

## Item notes

### `fn one_bad_workspace_does_not_cost_the_others`

The per-item promise at the granularity an operator thinks in.
Here: an unnamed one, a duplicate one, one whose only panel this
build does not offer — and two good ones that come back intact.

### `fn a_fresh_document_ships_no_workspaces_at_all`

`MODES_AND_PANELS.md` makes a mode *a named workspace*, and
`SHELL_FRAMEWORK.md` makes Read/Review/Edit a configuration rather
than a built-in. A default store with three magic names would
quietly weld one application's modes into the framework, which is
the exact class of coupling the purity gate exists to prevent —
and that gate greps for the application crates' names, so it would
not catch this one.

### `fn an_unstamped_workspace_is_unknown_not_empty`

The single most important property here, because collapsing them is
the tempting simplification and it is the one that reintroduces a
worse bug than the one this fixes. `New(vec![])` says *every
registered panel was already known* — act on nothing. `Unknown` says
*this file predates the record* — there is no evidence, decide.

If `Unknown` were represented as an empty list, every workspace
written before this field existed would report "nothing is new",
and the upgrade case this whole mechanism exists for would silently
do nothing. If instead it were represented as "everything is new",
every panel the operator had deliberately closed would spring back
open — undoing a decision they actually made.

### `fn a_panel_the_operator_closed_stays_closed`

The behaviour the whole design is for, stated as a test rather than
left to follow from the definition: `known_panels` records what
EXISTED, not what was mounted, so a panel that was registered and
deliberately left out of the arrangement is known — and stays out.

### `fn a_file_written_before_this_field_still_loads`

The compatibility property the `#[serde(default)]` buys, asserted
against real serialized text rather than against a constructed
value — a `Default` impl cannot prove that a file written by an
older build parses.

### `enum Unseen`

Returned by [`LayoutDocument::unseen_panels`], which reports rather than
decides — see [`Workspace::known_panels`] for why the decision is the
application's.

### `fn save_workspace`

Returns whether an existing workspace was replaced, so an
application can offer "overwrite?" *after* the fact in a status
surface rather than asking before it — the same posture the rest
of this crate takes towards modal interruptions.

Replacing in place rather than appending is not cosmetic: a
workspace that moved to the end of the list every time it was
updated would reorder the operator's own menu behind their back,
and a menu whose order changes when you use it is a menu you
cannot build muscle memory for.

### `fn workspace`

Returns a reference; the caller clones it into
[`crate::dock::DockState::set_layout`]. Deliberately **not** a
method that applies it: the store does not own the live state, and
a function here that reached into a `DockState` would be a second
path by which the arrangement changes.

### `fn unseen_panels`

`registered` is every panel id the application has registered right
now — not the ids the layout mounts, and not the ids it *would*
mount by default. The comparison is against what EXISTED, which is
the only thing that separates "closed on purpose" from "did not
exist yet".

Returns [`Unseen::Unknown`] for a workspace saved before the record
existed, and for a name that is not in the store at all — in both
cases the honest answer is that this document cannot say. A caller
that wants to distinguish them can check
[`Self::workspace`] first.

# This reports; it does not act

Consistent with [`Self::workspace`] returning a reference rather
than applying it: the store does not own the live state, and a
method here that mounted a panel would be a second path by which
the arrangement changes. What to do with the answer — mount it,
mention it in a status line, ignore it — is the application's, and
it is a product decision rather than a framework one.

### `fn mark_panels_seen`

For the moment **after** an application has acted on an
[`Unseen`] answer: having decided what to do about the new panels,
it records that it has seen them, so the next launch reports
`New(vec![])` rather than offering the same ones again.

Separate from [`Self::save_workspace`] because the two happen at
different times and for different reasons — saving is the operator
rearranging something, stamping is the application acknowledging a
release. Folding them together would mean an application could only
record what it had seen by also rewriting a layout it had no reason
to touch.

Returns whether the workspace existed.

### `fn delete_workspace`

Returns whether one was removed. Deleting something that is not
there is not an error — a second click on a delete command, or a
stale menu, must not produce a failure the operator has to read.

### `fn rename_workspace`

Refusing rather than merging: a rename that silently absorbed
another workspace would destroy an arrangement the operator did
not mention.

### `fn sanitize_all`

**Per workspace**, which is the whole point: one saved arrangement
naming a panel this build does not offer loses that tab; one with no
name at all is dropped; a second one claiming a name already used is
dropped; and every other workspace in the file is untouched. That is
the per-item promise applied at the granularity an operator thinks in
— *"my Review layout came back and my Proofing one did not"* is a
sentence they can act on.
