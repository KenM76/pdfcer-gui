# `fieldset` — the form fields selected beside the primary one

Holds the extra fields of a multi-field selection (Shift/Ctrl-click, marquee),
so Align and Distribute can move several fields as one undo step.

## Why a side set and not a `Vec` in place of `selected_field`

`OpenDoc::selected_field` has nine writers and many single-field readers
(properties panel, grips, Delete, Copy, rename). Turning it into a list would
touch every one of them for no behaviour change. The set instead records the
primary it was built against (`anchor`) and is honoured only while that still
equals `selected_field`. Any writer that replaces the primary therefore drops
the multi-selection without knowing the set exists — the same outcome a plain
click on another field should have.

## Order

`all()` returns the others in the order they were added, then the primary.
Removing the primary promotes the most recently added other, so the properties
panel always shows a field that is still selected.

## Not saved

Like every selection, it is view state: no epoch bump, nothing written.
