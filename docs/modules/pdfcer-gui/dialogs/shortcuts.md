# `dialogs::shortcuts` — every keyboard chord, derived from the keymap that
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
