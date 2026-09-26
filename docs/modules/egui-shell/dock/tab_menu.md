# `egui-shell/dock/tab_menu`

## Item notes

### `fn fmt`

[`egui::Response`]'s own `Debug` is large and includes an
`egui::Context` handle; a `TabMenu` printed in a test failure
should say *which tab* and *what was asked of it*, which is the
whole of the useful information.

### `fn with_tab`

A synthetic `Response` cannot be constructed from outside `egui`
(its fields are private and `Response::new` is not public), which is
correct — these tests should exercise the same value the dock hands
out, not a stand-in that could diverge from it.

### `fn a_fresh_tab_menu_has_requested_nothing`

The flag's default matters: it is read unconditionally by the dock
after **every** handler call, on every tab, on every frame. A
default of `true` would close the whole dock on the first frame a
handler was supplied, which is the kind of defect that is obvious
once and invisible in review.

### `fn requesting_a_close_twice_is_the_same_as_requesting_it_once`

A handler assembled from several independent pieces — a menu, a
keyboard shortcut, a diagnostic — may reach the same conclusion
more than once in one frame, and two `Intent::Close`s for one panel
would make [`super::super::DockFrameReport::closed`] report a close
that removed nothing on the second pass.
