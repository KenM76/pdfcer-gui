# `pdfcer-gui/canvas/offset`

## Item notes

### `fn won`

Every `return` in [`decide`] goes through this, which is what keeps the
name and the number impossible to separate: there is no way to add an
arm that produces an offset without also naming it.

### `struct Frame`

A struct rather than nine parameters because the ranking is what a reader
comes here for, and nine positional arguments at the call site would be the
thing they had to read first. Every field is `Copy` and small.

### `const SEED_FRAME`

`OpenDoc::canvas_frames` counts the canvas frames this document has had and
is incremented at the end of [`super::viewpos::position`], *after* this
chain has run — so during the chain it reads as the zero-based index of the
frame being decided. The seed fires on index `1`, the **second** frame; see
the open-seed arm for the four bisecting runs that argued against index `0`.

### `struct Decision`

# Why the winner is returned and not merely the number


That cost a full session of diagnosis. A regression placed a freshly
opened multi-page document off the bottom-right corner, the published
offset was `[0.0 0.0]`, and **three different arms of this chain can
produce exactly `(0.0, 0.0)`** — the deep-tier arm returns it as a literal,
[`geometry::strip_offset`]'s lower clamp produces it from any sufficiently
negative page-local solve, and a strip-space page scroll to the very top of
the content produces it honestly. With only the number in hand, those are
indistinguishable, and so is a fourth case: no arm firing at all. Naming
the winner collapses four hypotheses into one measurement.

The name is a short stable token, never a sentence, because its readers are
a `grep` in a trace file and a `ui-verify` assertion rather than a person
reading prose.

### `fn decide`

See the module header for the ranking. The body below is that table in
code, in the same order, and the comments on each arm are the ones that
were written when each source was added.
