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

### `type TabMenuHandler`

Spelled as a type alias for the same reason [`super::RectSink`] is: the
`dyn` form appears in [`super::Dock`]'s private field and in
[`super::ctx::Ctx`], and writing it out twice is two places for the
lifetimes to drift apart.

### `struct TabMenu`

Constructed by the dock immediately after the tab is drawn and dropped
as soon as the handler returns — it borrows the `Response` that was
just produced, so it cannot outlive the frame and cannot be stored.

See the module header for the seam this sits on and for what happens to
the dock's built-in "Close".

### `fn panel`

The application's own id — whatever string it registered with
[`super::PanelInfo`]. A handler attaching a context menu usually
needs this to decide *which* menu to attach, or to carry alongside
the chosen command so the dispatcher knows what the operator
right-clicked.

### `fn response`

Senses clicks, so [`crate::menu::Menu::attach`] and
[`egui::Response::context_menu`] both work on it directly. Its
`WidgetInfo` is already published (see the module header's
accessibility section); a handler that publishes another one would
overwrite the panel's purpose with whatever it supplies, which is
allowed but is almost always a mistake.

**One popup per response.** Attaching two context menus to it is
the id collision the module header describes; attach one.

### `fn request_close`

The seam's route to the dock's own close path: it becomes an
[`super::ctx::Intent::Close`] like any other, applied after the
frame, reported in [`super::DockFrameReport::closed`], and counted
towards [`super::DockFrameReport::layout_changed`] so an
application that persists on that flag persists this.

**Nothing happens during this call.** The panel is still drawn for
the rest of this frame, its body included; the layout is not
mutable while it is being drawn and this method does not make it
so. Calling it twice is the same as calling it once, and calling it
on a panel that is not mounted is a no-op — see
[`super::DockLayout::close`].

### `fn close_requested`

Rarely needed by an application — it knows what it asked for — but
it makes the flag readable by a handler composed of several
independent pieces, and it is what the dock itself reads.

### `fn request_float`

The verb this module's own header names as the canonical example
of something *"the shell cannot know"* — and it still cannot: the
application owns the row, its label, its icon, its position in the
menu and its keyboard chord. What arrives here is the **act**, and
it goes through the dock's queue exactly as a close does, so it
appears in [`super::DockFrameReport::floated`], counts towards
[`super::DockFrameReport::layout_changed`], and is therefore
persisted by an application that saves on that flag.

A no-op on a panel that is already floating or is not mounted —
see [`super::DockLayout::float`]. **Nothing happens during this
call**; the tab and its body are drawn for the rest of this frame.

### `fn request_dock`

The mirror of [`Self::request_float`], and the verb a floating
panel's header strip offers. A no-op on a panel that is not
floating.

It is offered on a **tab** as well as on a header strip, and
deliberately: one handler serves both surfaces, so an application
writes one menu and gets the right rows in both places by making
the rows conditional rather than by writing the menu twice.
