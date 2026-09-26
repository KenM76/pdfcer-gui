# `app::dispatch::routes` — the commands that perform nothing and point
somewhere else

## The seam, and it is a subject rather than a size

Every arm here raises `Action::Command(other_id)` and does nothing else:

> `Action::Command` exists so a second route to an existing command cannot
> become a second implementation of it.

It is **not** every such arm in the shell. `format.properties` has the
same shape and stays in [`super::format`], because an id must have exactly
one claimant and moving it would have given it two. This file claims the
shape, not a monopoly on it.

The failure a route exists to prevent is the one this project has spent its
time removing everywhere else: two surfaces for one capability, drifting
apart, each with its own guards.

## Why a command exists at all when another command does the work

Because `egui-shell` enforces **one command, one tab**, and the placements
answer different questions. An operator hunting on the Tools tab for where
font folders live should find the entry there; that it opens the Settings
window is an implementation detail of *where the list is kept*, not a
reason to leave the Tools tab silent.

⚠ A command whose capability lives somewhere else is the kind whose recorded
blocker stops being true without anybody noticing, because nothing about it
changes when the other surface ships. Re-derive a route's blocker from the
target, never from the route's own entry.

## Item notes

### `fn target`

One function, so the mapping is stated once and [`handles`] cannot drift
from [`dispatch`] — the two would otherwise be a list and a match that agree
today.

### `fn every_route_points_at_a_registered_command_that_is_not_itself`

Two failures in one assertion, and both are silent. A route to an
unregistered id raises an `Action::Command` that `dispatch_command`
drops on the floor — a drawn control that does nothing. A route to
itself is an infinite loop through the action queue.

### `fn handles`

`pub(crate)` for `dispatch::format::handles`' reason: `shell::commands::reach`'s
reachability checker must be able to **evaluate** every guard arm it finds,
and a guard it cannot evaluate is a place commands could hide from the check
that exists to find them.

### `fn dispatch`

It does **not** re-check the target's guards. `dispatch_command` is the
choke point and the raised id goes through it exactly as a ribbon click
would — which is the entire point of routing rather than performing. A
guard applied here would be a second copy of the target's rule, in the file
whose purpose is to have none.
