# `shell::menus::tests` — the sweeps that keep the menu document honest


## The seam, and why it is a subject rather than a cut

[`super`] is a **document**: one function returning the menus pdfcer
defines, plus the prose arguing every row of every one of them. It changes
when a menu changes.

This is a **checker**. Every test here is a sweep over *whatever*
[`super::built_in`] happens to return — every command registered, every menu
non-empty in the state it is opened in, every id also reachable from the
ribbon, the whole thing round-tripping through RON. Not one of them names a
menu it was written for. They change when the *rules about menus* change,
which is a different rate and a different reason.

⇒ That is this project's own test for a seam, applied: two subjects, two
rates of change. It is the same cut [`crate::canvas::annotnodes`] and
[`crate::shell::commands::reach::guards`] already make, and it is why the
file that had to be split is the one that grew — the document grew a menu;
the checker did not grow anything.

## What did NOT move, and why

[`super::MenuHost`]. It is ~300 lines and looks like the other obvious cut,
and it stays because it is not a separate subject: `with_condition`'s
frame-ordering argument is about *when a menu is drawn*, which is the same
subject as *what is in it*, and a reader following a row from its
`visible_when` to the condition that corrects it would have to change files
mid-sentence. Splitting the tests off costs the reader nothing, because a
sweep is read on its own or not at all.

## Item notes

### `fn every_command_every_menu_names_is_registered`

The check nothing else performs. `Shell::validate_against` walks
`command_references()`, which covers tab groups, the QAT and the
keymap and deliberately **not** the menus — so a menu naming a
command this build does not have passes the manifest's own
validation, renders as one row fewer, and discloses the omission on
a channel nobody reads during development.

`Menus::validate_against` is the engine's own opt-in answer and it
checks both this and the structural rules (no empty context id, no
duplicate context, no command listed twice within one menu), so it
is asked rather than reimplemented. Its error names the menu **and**
the id, which is what makes a failure point at a line rather than at
a file.

### `fn the_shipped_shell_carries_the_menu_document`

The failure this catches is a one-line omission with no symptom: drop
the `menus` assignment from `manifest::built_in` and every test in
this file that builds `built_in()` directly still passes, while every
right-click in the running application does nothing.

### `fn the_catalog_defines_exactly_the_documented_contexts`

[`CONTEXTS`] is hand-written and every sweep below is only as
complete as it is, which is the classic way a test suite quietly
stops covering something. Checked in both directions.

### `fn the_built_in_menu_document_is_valid`

Distinct from the registry check and not implied by it: the built-in
layer is what every customization layer patches and what a reset
restores, so it has to stand up without an application present —
non-empty context ids, no duplicates, no command listed twice in one
menu.

### `fn no_menu_offers_a_command_this_build_does_not_have`

`every_command_every_menu_names_is_registered` proves the positive.
This proves the *specific* negative `RIBBON_IA.md` §6 asks for and
P3 forbids: §6 wants Cut/Copy/Paste on the selection menu, this build
has no object clipboard, and the honest answer is **absence**.

Asserted against `PLANNED` rather than against a hand-written list of
four ids, so a clipboard command that lands — and is therefore
removed from `PLANNED` — stops being forbidden here automatically
instead of failing a test that had gone stale.

### `fn every_menu_offers_something_when_a_document_is_open_and_selected`

The other half of the empty-menu rule, and the half that would
otherwise be satisfied by defining no menus at all. A menu that never
opens is indistinguishable from a right-click that is not wired, and
the operator draws the same conclusion from both.

`dock.tab` is included: it is not *attached* (see the module header),
but the day the `egui-shell` seam lands it must have something to
offer, and this is what says so.

### `fn the_field_menu_opens_with_a_field_selected_and_nothing_else`

The state the operator is actually in when they right-click a text box:
`doc.selected_field` is set and `SelectionState` is **empty**, because a
`/Widget` is deliberately not an annotation selection. Every other canvas
menu resolves nothing there.

⇒ This is the assertion that would have caught the bug this feature
shipped with for ten minutes: `format.delete` and `format.properties`
were gated on `selection.any`, which is **false** in exactly this state,
so both items resolved disabled, `offers_anything` was false, and the
menu never opened. A right-click on a form field would have done nothing
at all — `DEFECTS.md` D1's shape, arrived at through a new door.

`everything_open()` is deliberately not used: it sets both conditions
and would pass on a build where the two are confused. The whole point is
that only the wider one holds here.

### `fn a_menu_with_nothing_to_offer_does_not_open`

The engine's rule 2, asserted through the seam this application
actually uses rather than against the engine's own unit tests.
Three shapes, and all three are reachable:

1. **a context with no menu at all** — a right-click site whose id is
   misspelled, or one wired ahead of its menu;
2. **a menu whose every command is disabled** — `canvas.object` with
   nothing selected, which is what a right-click on paper would find
   if the canvas picked the wrong context id;
3. **a menu whose every command is unregistered** — the shape a
   build with a capability compiled out produces.

Shape 2 is the one that matters most in daily use, and it is the one
a naive wiring gets wrong: `format.delete` is registered, so a
`context_menu` closure written by hand would happily draw it greyed
and cost a click to dismiss.

### `fn correcting_the_selection_condition_is_what_opens_the_object_menu`

[`MenuHost::with_condition`] exists for one frame-ordering hazard,
and this is that hazard reduced to two assertions: with the stale
snapshot the selection menu does not open, and with the correction
the canvas just computed it does.

Without this the first right-click on an object silently does
nothing — the menu is decided before `egui` is asked for a popup, so
there is no later frame on which it can recover.

### `fn every_menu_command_is_also_reachable_from_the_ribbon`

`RIBBON_IA.md` §5.8: the context menu *"carries the same commands
again … that is not duplication in the P1 sense — context menus are
not tabs"*. Every id in this document is also on a ribbon tab, which
is the point and not an oversight; if a future edit extends the
one-command-one-tab rule over menus, this is the test that says no.

### `fn the_menu_document_round_trips_through_ron`

The whole value proposition of the shell-as-data design is that an
operator can edit this; `crate::shell::ron` asserts the same thing
for the manifest as a whole. Asserted here as well, on the menu
document alone, because a failure in the shared file says only that
*something* stopped round-tripping.

### `fn each_menu_holds_exactly_the_documented_items`

A change-detector, and deliberately one: the table in the header is
the specification, and a menu that quietly gains an item has a
specification that quietly became wrong. The failure message names
the menu, so the fix is one line in one of the two places.

### `fn the_two_node_rows_are_absent_greyed_and_live_in_the_three_states`

The markup menu's two node rows are the only place in this document where a
row can be *absent on one shape and greyed on another*, and the two halves
come from two different mechanisms — `visible_when` on the item, and the
command's own `enabled_when`. A build that wired one and not the other would
look correct in every screenshot of the working case.

The three cases below are the three the operator meets:

| conditions | what the row is | the shape it describes |
|---|---|---|
| neither offered | **absent** | a `/Square`, an `/Ink` stroke — no points, ever |
| offered, not enabled | **greyed** | a three-corner polygon, on a corner: the vertex floor |
| offered and enabled | **live** | a five-corner polygon, on a corner |

Falsified three ways, each independently: dropping `shown_when` from the
item makes case 1 fail (the row is drawn where it can never work); dropping
`enabled_when` from the command makes case 2 fail (a floor-breaching remove
is offered as pressable); and setting `enabled` without `offered` — which no
caller does, because `RowState::enabled` implies `RowState::shown` — would
leave case 3 asserting nothing, which is why case 3 sets both.

### `fn the_menu_surface_owns_no_copy_of_its_own`

`text::menus`' emptiness is a *consequence* of every menu item being a
command reference, and that consequence has a precise failure mode:
an `Item::Custom` row is drawn by the application, so its words come
from the application, and there is no other honest place for them
than `text::menus`. A separator has no words either, so it is allowed —
it is punctuation.

If this fails, the fix is **not** to delete the test. It is to write
the string into `text::menus` and hand it to whatever renders the
custom row, which is the sequence the whole catalog rule exists to
force.
