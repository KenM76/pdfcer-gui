# `egui-shell/menu/shortcut`

## Item notes

### `fn the_chosen_chord_does_not_depend_on_insertion_order`

The failure this prevents is nasty and would never be reported as a
bug in this file: an operator adds an unrelated binding, the
`BTreeMap`'s iteration order shifts, and *Delete* silently starts
advertising a different key. The rule in [`prefer`] is total, so
the answer is a function of the set of chords and of nothing else —
which is what this asserts, by building the same set several ways.

### `struct Shortcuts`

Built once per frame (or once per menu open) from a
[`Keymap`], because the keymap is indexed the other way round. See the
module header.

Ordered (`BTreeMap`) so enumerating it — in a trace, in a failing test
message — is deterministic.

### `fn none`

What a menu gets when the manifest has no keymap: every row draws
its label alone, which is a correct menu rather than a degraded
one.

### `fn of`

The convenience the renderer actually uses: a [`crate::Shell`] with
no `keymap` key yields [`Self::none`] rather than requiring every
call site to unwrap an `Option`.

### `fn prefer`

Fewest modifiers, then shortest, then lexicographic. See the module
header for why each step is there. Total and antisymmetric, so the
result is a function of the two strings and of nothing else.

### `fn modifier_count`

`"Ctrl+Shift+P"` → 2, `"F11"` → 0, `"Ctrl+E"` → 1.

Degenerate inputs degrade to 0 rather than to a panic or a negative:
`"+"` names no pieces at all once empty segments are dropped, and
`""` names none either. A chord this strange is an operator's typo,
and the correct penalty is that it sorts first among equals — not that
the menu refuses to draw.
