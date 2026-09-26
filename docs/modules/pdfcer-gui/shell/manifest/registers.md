# `pdfcer-gui/shell/manifest/registers`

## Item notes

### `const PLANNED`

`(id, reason)`. The reason is the entry's whole value: it is what lets
a later stage tell a **C** row — engine written and tested, shell
missing, a day's work — from an **N** row that is a month, without
re-deriving the analysis from the specification each time.

# Why this exists rather than a comment

P3 says an unavailable capability renders nothing. Applied literally
and alone, that turns `RIBBON_IA.md`'s specification into a much smaller
manifest with no record of the difference, and the next person to read
this module cannot tell a command that was *considered and deferred*
from one that was *never noticed*. Those are very different facts and
only one of them is a plan. The sizes of the two sets are not restated
here: the tables are the only copy, and a count in prose beside a table
is a second copy that drifts.

So the omissions are data:

- **tested**, in both directions — `planned_commands_are_genuinely_absent`
  asserts nothing here is referenced by the manifest *and* nothing here
  is registered, so an entry that gets built and not removed fails the
  suite rather than becoming a stale comment;
- **enumerable**, so a diagnostic surface or a roadmap tool can list
  the gap;
- **greppable by id**, so the search that finds `measure.two_line` in
  the manifest also finds the note saying where it went.

# Ordering

By tab, in the tab order of [`built_in`], then in the order
`RIBBON_IA.md` §5 lists them within their group. Not sorted
alphabetically: this list is read against the specification, and a
reader checking §5.3 against it wants the Pages entries together and in
the document's order.

### `const DIRECTED`

`(id, why)`. It exists as a list rather than as prose because otherwise
this manifest would look like it applied P3 everywhere except in one
place, for no stated reason.

⚠ **The bar is a command the specification describes in enough detail to
be a decision rather than a wish**, even though no status mark says so.
A row that names a verb in every selection type it applies to has
specified something; a row that names a capability has not.

The tension this register holds, stated rather than smoothed over: P3
exists so an operator is never shown a control that does nothing, and an
entry here is a control P3 would otherwise have suppressed. It earns its
place by being **real** — `format.delete` is wired through
`PdfcerApp::dispatch_token` to `SelectionState::deletable_objects_on`,
the same rule the Delete key reads — not by being specified.

⇒ **A settings knob does not belong here.** A knob whose value is a
compiled-in constant reads as `partial G` and is tempting, but a
preference surface is where a setting is discovered and changed; this
register is for commands. `app::prefs` and `dialogs::settings::display`
are where such knobs live, and `shell::commands::reach` records what
moving them cost. Keeping the list short is what makes deleting a row
from it cheaper than re-deriving which entries were deliberate.

### `const TAB_SCOPED`

# Why this register exists, and what it must NOT become

Two tests state one rule from two sides:
`shell::tests::no_registered_command_is_orphaned` and
`shell::menus::tests::every_menu_command_is_also_reachable_from_the_ribbon`.
The rule is right and it is worth restating in its own words:

> *A command reachable only by right-clicking one particular surface is
> a command nobody can find: a context menu is discovered by an operator
> who already suspects something is there, which is exactly the state a
> command with no other home cannot put them in.*

⇒ **The bar for an entry here is [`CUSTOM_BACKED`]'s bar, unchanged:
the command needs an OPERAND a ribbon control cannot ask for.** Not
*"a button would be redundant"*, not *"the menu is the natural place"* —
the ribbon control must be impossible to make *correct*.

`CUSTOM_BACKED` answers that by drawing a non-button control on the
ribbon that asks for the operand (a recent-files menu, a font-face
chooser). This register answers it for the case where **even that is
impossible**, because the operand is *the surface the operator
gestured at*. There is no ribbon control that can ask "which of the
twelve panels?" and get the answer "the one you just right-clicked",
because at the moment a ribbon control is pressed the operator is not
pointing at a panel.

# How discoverability is answered instead

The rule's reason is discoverability, and a register creates none.

The **capability** is on the ribbon even though the per-panel verbs are
not. View ▸ Window carries `view.dock_all_panels` — *"Bring every
floating panel back into the dock"* — and `view.reset_layout`. An
operator reading that group learns that panels can float and that there
is a way back, which is the fact worth discovering; where the verb that
floats *this* panel lives is then the universal idiom, on the tab.

That is a weaker answer than a ribbon button and it is stated as such
rather than dressed up. The day a panel tab grows a visible affordance
— a close cross, a chevron — these entries come out, because then there
is a control on the surface itself and the menu is a second route rather
than the only one.

# What an entry buys and what it does not

It buys the two tests above. It does **not** buy the rename check:
`every_command_every_menu_names_is_registered` still runs, so an entry
naming a command that no longer exists, or a command here that is not
in any menu, fails [`tests::every_tab_scoped_entry_is_real`] in both
directions.
