# `shortcutsdialog` — every keyboard chord, derived from the keymap that
dispatches them

## This window has **no list in it**

Nothing in this module holds the reference. Every row is derived, on the
frame that draws it, from the keymap and the registry. That is the whole
design, and it is why the old shell's hand-written `shortcuts_reference()`
is not carried across at all rather than carried across and corrected: a
correct copy is still a copy.

## D5 is not fixed here; it is made unrepresentable

`DEFECTS.md` D5 is a *hand-maintained reference* disagreeing with the actual
bindings, and the reason it happened is the reason it would happen again:
two places state the same fact, one of them dispatches and the other is
prose, and only the first is exercised by using the program.

Every row below is produced by iterating **the same `Keymap` that
`app::keyboard::commands` resolves a keystroke against**. A binding that
exists is listed because listing is a fold over the bindings; a listing that
is wrong is not a thing this window can produce. There is no second copy to
drift.

That is the difference between fixing a defect and closing the class of it.
[`crate::canvas::snap`]'s tolerance and [`super::scale`]'s unit list are the
same move against the same hazard: one statement of a fact, derived wherever
it is needed.

## An unregistered command is DROPPED, not shown greyed — R8

A chord whose command is not in the registry names a capability this build
does not have — the strippable-capability convention, where a feature's
absence is expressed by its command not being registered. Listing it would
promise a key that does nothing, which is the placeholder rule applied to
prose.

**The count of dropped chords is disclosed** rather than silently absorbed,
because *"this build has fewer shortcuts than the manifest declares"* is a
true and surprising fact about a stripped build, and an operator comparing
two installations needs it. That is `SHELL_FRAMEWORK.md` §5b's
`CapabilityAbsent` posture, arriving in a window rather than in a log.

## Why it is application-scoped

A keyboard reference is meaningful with nothing open — it is one of the two
things a new operator reaches for before opening a file, the other being
About. It therefore sits beside [`super::about`] in the group
[`super::DialogsState`] does not close when a document closes.

## Item notes

### `struct Row`

**Chords are plural**, and that is not a nicety: `edit.redo` is bound to
both `Ctrl+Y` and `Ctrl+Shift+Z`, deliberately, and a reference showing one
of them would be a reference that is *incomplete in exactly the way D5 was*
— quietly, on the binding an operator's other application taught them.

### `fn rows_from`

## Grouped by command, not by chord

A keymap is `chord → id`, and rendering it directly would give `Ctrl+Y` and
`Ctrl+Shift+Z` two rows saying the same thing — which reads as two features
rather than as one with two keys. Inverting it is what makes the *plural*
case legible, and the plural case is the one D5 got wrong.

## Why the order is the command id's

`BTreeMap` over the id, so `edit.*` sorts together, `file.*` together, and
the list is stable across runs and machines. Sorting by chord would
interleave every tab's bindings and put `[` next to `]` next to `Alt+Down`,
which is alphabetical and useless — an operator looking for *"the shortcut
for rotating"* is thinking about the verb, not the key.

It is deliberately **not** the ribbon's tab order, which would be truer to
the operator's mental model and would require this window to know the
manifest's tab list. A window that reads the keymap and the registry and
nothing else is a window that cannot disagree with either.

### `fn every_bound_chord_appears`

D5 is a hand-maintained reference disagreeing with the keymap that
dispatches. The listing here is a fold over the bindings, so the
property is structural — and this test is what says so out loud,
because a reader looking at a window full of shortcuts has no way to
tell a derived list from a copied one.

### `fn a_command_with_two_chords_is_one_row_naming_both`

`edit.redo` really is bound twice, deliberately, and a reference showing
one of them would be incomplete in exactly D5's way — quietly, on the
binding an operator's other application taught them.

### `fn a_chord_for_a_missing_command_is_dropped_and_counted`

R8: a command that is not registered is a capability this build does not
have, so listing its key would promise a keystroke that does nothing.
The count is what stops the omission being silent — a stripped build
genuinely has fewer shortcuts, and that is worth a sentence rather than
a shrug.

### `fn the_label_is_the_registrys_own`

The second half of the same argument: a hand-written label would drift
from the ribbon's the day one of them was reworded, and an operator
reading two different names for one command has to work out that they
are one command.

### `struct ShortcutsDialog`

**It holds nothing.** Every row is derived from the keymap and the
registry on each frame, which is the whole point — see the module header.
A cached list would be a second copy, and a second copy is D5.

The unit struct exists because [`super::DialogsState`]'s idiom is one
`Option<T>` per dialog, whose `Some` *is* the open state. A `bool` would
work and would be the one dialog here shaped differently.
