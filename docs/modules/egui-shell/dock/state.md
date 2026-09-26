# `egui-shell/dock/state`

## Item notes

### `fn new`

The arrangement is normalized on the way in, so an application's
built-in default cannot ship a stack with no tabs or a panel
mounted twice. An application that wants to *know* whether its
default needed repair asserts [`DockLayout::is_normalized`] in its
own test suite — the same posture `manifest` takes towards its
built-in layer, and for the same reason: a defect in a compiled-in
constant should fail a test, not be quietly patched on every
machine that runs it.

### `fn set_float_drag`

[`super::Dock::show_floating`] calls this for its own header strips, so
an application that uses the shell's float windows needs nothing here.
It stays public for one that draws them another way, and because the
conversion needs two window origins a platform may not report — see
[`super::floatgrab`] for the arithmetic and [`super::floatdrag`] for
what the dock does with the answer.

**Consumed, not held.** The next [`super::Dock::show`] takes it, so a
caller that stops renewing the report ends the gesture with nothing
landed. That is what makes a window closed mid-drag, or a platform that
lost the pointer, safe by construction rather than by a cancel path
somebody has to remember to call.

### `fn rail_auto_hide`

See [`crate::peek`] for the model, and [`rail::PEEK_WIDTH_PTS`] for the
sliver that is reserved in its place — the strip never disappears
entirely, because it is the only route to some panels and a rail that
vanished would take them with it.

### `fn layout_mut`

The application's route to everything [`DockLayout`] can do —
mounting a panel, hiding a side, applying a workspace, resetting a
scope. Deliberately **not** normalized on the way out: a caller
making several edits should not pay a repair pass per edit. Call
[`Self::normalize`] when the edits are finished, or let the next
[`super::Dock::show`] do it.

### `fn geometry`

Rebuilt from nothing each frame, so it holds no rect for a compartment
that has stopped being drawn — but it is one frame old, and a rect one
frame old is indistinguishable from a current one by inspection. See
[`geometry`].
