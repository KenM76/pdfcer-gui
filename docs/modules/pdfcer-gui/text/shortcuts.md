# `pdfcer-gui/text/shortcuts`

## Item notes

### `fn no_string_here_is_part_of_the_reference`

The rule this module exists to hold, asserted rather than trusted. A
hand-written `"Ctrl+S — Save a copy"` added here for convenience would
re-create D5 in the one file whose whole purpose is that D5 cannot
happen, and it would look perfectly reasonable in review.

The probe list is chord *shapes* rather than every chord, because the
point is to catch the habit, not to enumerate the keymap — enumerating
it here would itself be the second copy.

### `fn a_missing_table_and_an_empty_one_read_differently`

*"pdfcer could not read its table"* and *"nothing is bound"* are a fault
and a customization, and an operator seeing one message for both cannot
tell which they have.

### `fn intro`

Says the reference is **live** rather than written down, because that is a
fact an operator can act on: if a key is not here, it is not bound, and
there is no third possibility involving a list somebody forgot to update.

### `fn chord_separator`

A comma and a space rather than a slash or a pipe: `Ctrl+Y` and
`Ctrl+Shift+Z` are two *alternatives*, not a sequence, and a slash between
keys reads as "press these together" to anyone who has met `Ctrl+Alt+Del`.

### `fn derived_note`

The count is here **because it is checkable**. An operator who suspects a
key is missing can compare it against nothing useful — but a *future* build
whose count drops has told them something, and the number is the cheapest
form that fact can take.

### `fn dropped_note`

**Disclosed rather than absorbed.** R8's convention is that a capability's
absence is expressed by its command not being registered, and a customized
or stripped build can therefore carry a keymap naming commands that are not
there. Those keys do nothing, so they are not listed — and *"this build has
fewer shortcuts than its keymap declares"* is a true, surprising fact that
an operator comparing two installations needs.

Worded as a fact about **this build**, not as an error: a stripped build is
a supported thing to be, and the eventual exe-to-DLL move makes it the
ordinary case.

### `fn no_keymap`

Reachable when the manifest failed to load, in which case **no chord works
either** — so an empty list would be accurate and unhelpfully so. Saying
which of the two states this is turns a puzzling window into a diagnosis.

### `fn none_bound`

A different sentence from [`no_keymap`], because the two are different
situations and only one of them is a fault. A manifest that deliberately
binds nothing is a legitimate customization.
