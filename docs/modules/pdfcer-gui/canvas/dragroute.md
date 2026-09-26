# `canvas::dragroute` — which of THREE move verbs one drag reaches


## The seam is a subject, and it is the one this project got wrong

One gesture — press inside the thing, drag it — reaches three different
engine verbs, and **which one is decided entirely by what is selected**:

| selection | verb | why it is not the others |
|---|---|---|
| page content | `move_objects` / `move_nodes` | paint-order indices into a content stream |
| a **ce dimension** | `place_dimension` | moves where it is DRAWN and cannot alter the number it prints |
| ordinary **markup** | `move_annotation` | a stable `ObjId`, and two halves of geometry to write |

**A fork whose branches can all answer "not mine" eats the gesture**,
and that is exactly what this one did for ten days. The annotation branch
held only `dimdrag`, which answers `None` for anything that is not a ce
dimension; the content branch was in the `else`, unreachable behind an
annotation selection by construction. An operator pressed inside a stamp,
dragged it across the sheet, released, and nothing happened and nothing
declined.

⇒ The failure is worse than a missing feature, because a missing feature
usually refuses. Gathering the three into one function is what makes the
exhaustiveness visible: they are now adjacent, and a fourth kind arriving
has one place to be added rather than a fork to be noticed.

## Ordered, not exclusive-by-guard

`dimdrag` is asked first because it is the **narrower** claim — it answers
only for `AnnotKind::CeDimension` — and `annotdrag` re-checks the kind for
itself rather than reading *"dimdrag said no"*. A module that decided what
it handles from another module's refusal would be one rename away from
claiming everything.

## Shift is applied ONCE, above the fork

`ui-conventions/drag-moves.md` D5. All three verbs receive the same
constrained delta from one filter, because two copies of *"what does Shift
mean"* is how two drags in one program come to disagree about it.

## Item notes

### `struct Previews`

Three fields rather than an `enum`, matching the seven preview slots
`interact` already carries and for their stated reason: the painter reads
each one independently, and folding them together would put a branch in the
paint loop for a value that is `None` on every frame nobody is dragging.

### `struct Frame`

A struct because the argument list reached nine and clippy is right that
nine positional parameters is a call nobody can read — five of the six here
are borrows of similar-looking things, and transposing two would compile.
`dimdrag::Frame` and `annotdrag::Frame` take the same shape for the same
reason, so this is the local convention rather than an accommodation.
