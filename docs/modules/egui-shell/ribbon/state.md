# `egui-shell/ribbon/state`

## Item notes

### `fn new`

Both resolve on the first frame — the first visible tab and the
first mode in the manifest — so an application does not have to
know the manifest's contents to start.

### `fn set_auto_hide`

Called by the application when it restores the operator's preference at
start-up and when it dispatches the command that toggles it. Takes
effect on the next frame, like every other setting on this struct.

Turning it **off** always shows the band, immediately — that is the
way back, and it is why the command exists as well as the setting. See
[`crate::peek::Peek::set_mode`].

### `fn sync_auto_hide`

This exists because [`Self::set_auto_hide`] is *not* idempotent and
must not be: it clears the reveal, which is exactly right when the
operator changes the setting and exactly wrong when an application calls
it every frame to keep the shell in step with its own preferences store.
Called unconditionally in a frame loop, `set_auto_hide` would clear
`revealed` on every frame, so the band would be re-decided from the
pointer alone and the keyboard keep-term of [`crate::peek::Peek`] would
never hold. **The band would close under a keyboard user on the frame
after they reached it, and nothing would look wrong.**

So the frame-loop call is this one, and the difference between the two
is one comparison written down once rather than a `if state.auto_hide()
!= prefs.x` at every call site — the second of which is the one that
gets it wrong.

### `fn with_id_salt`

Needed only when two ribbons are drawn in one `egui` context — two
document windows in one viewport, say. See this module's header for
the symptom without one.

### `fn set_active_tab`

A tab set here is also **pinned into the strip**: whatever the
window width, [`super::plan::plan_tab_strip`] keeps the active tab
out of the overflow menu. So this is a complete way to drive the
ribbon from a keyboard binding or a restored session — the tab is
guaranteed to be visible, not merely selected.

### `fn set_mode`

This is what an application calls when the operator presses the
`Ctrl+1` its manifest bound to `mode.read` — the shell reports the
command's token, the application dispatches it, and dispatching it
means calling this. The mode selector is a *second* way to reach
the same state, not a separate one.

### `fn mode_segment_id`

Published so a harness can drive the selector by keyboard — focus a
segment, send an arrow — rather than by synthesising a click at a
guessed coordinate.
