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
