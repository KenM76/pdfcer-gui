# `pdfcer-gui/canvas/gesture/outcome`

## Item notes

### `enum Phase`

Both matter and they mean different things: an in-flight drag draws a
rubber-band or a ghost outline (a pre-commit affordance — the cursor
describing what is about to happen), while a completed one changes the
selection or raises an action. Collapsing them into one signal is how a
marquee ends up committing on every frame it is dragged across.
