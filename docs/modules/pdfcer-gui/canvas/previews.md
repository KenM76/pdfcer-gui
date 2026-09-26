# `canvas::previews` — **the fourteen things one frame might be showing you**

Every *pre-commit affordance* the canvas can draw, in one place, each with
the argument for why it is its own value rather than a variant of another.

## Why this is a module and not a paragraph in `interact`


The gate's own message says *"split the module along its seams — one subject
per file — rather than raising the limit"*, and this is a seam rather than a
convenient cut: `interact` answers *"what does this frame's pointer mean?"*
and these answer *"what is drawn while the answer is still provisional?"*.
The painter reads them; `interact` only fills them in.

## ★★★ The one argument every field here shares, stated once

**They are separate values and not one `enum`.** The reasoning was repeated
nine times in the source this was extracted from, and it is worth keeping
once rather than nine times:

> The painter reads each independently. Folding them together would put a
> branch inside the paint loop for a value that is `None` on every frame
> nobody is dragging — and it would make *"which kind of thing is this
> rectangle about?"* answerable only by looking at the selection.

Several really are the same Rust type — [`Slots::marquee`],
[`Slots::annot_ghost`] and [`Slots::zoom_region`] are all `Option<Rect>` —
and they are still separate, deliberately. **Two names is one fewer place to
be wrong.**

## Rule 4, which every field here is on the right side of

All of this is **the cursor**, not the document. A pre-commit affordance —
a rubber band, a snap marker, a ghost, the O63 shape preview — is explicitly
permitted; what is forbidden is styling content *already applied* as though
it were provisional. Every value here describes something that has not
happened yet and disappears when it does.
