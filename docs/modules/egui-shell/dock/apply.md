# `egui-shell/dock/apply`

## Item notes

### `fn apply`

The **only** function in this module that takes `&mut DockLayout`.
Returns whether anything changed, which the application uses to decide
whether the layout is worth persisting.

Splitter drags are applied by resolving the *current* spans, moving
one boundary with [`plan::drag_boundary`], and converting back — which
is the one place [`plan::spans_to_shares`] may be called, and its
documentation says why.
