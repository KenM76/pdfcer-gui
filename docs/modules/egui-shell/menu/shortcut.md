# `egui-shell/menu/shortcut`

## Item notes

### `fn the_chosen_chord_does_not_depend_on_insertion_order`

The failure this prevents is nasty and would never be reported as a
bug in this file: an operator adds an unrelated binding, the
`BTreeMap`'s iteration order shifts, and *Delete* silently starts
advertising a different key. The rule in [`prefer`] is total, so
the answer is a function of the set of chords and of nothing else —
which is what this asserts, by building the same set several ways.
