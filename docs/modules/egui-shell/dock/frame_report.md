# `egui-shell/dock/frame_report`

## Item notes

### `struct DockFrameReport`

Returned by [`Dock::show`] and also kept on [`DockState`], because two
different callers want it: the frame's own caller, and a diagnostic
surface that runs later in the same frame and has no access to the
return value.
