# `ui-verify/sys/unsupported`

## Item notes

### `fn clipboard_text`

A check that asserts on the clipboard therefore reports SKIPPED on this
platform, which is the honest answer. Returning `Some(String::new())` would
let a comparison against an expected string fail and be read as a defect in
the application.

### `fn clear_clipboard`

`false` rather than `true` so a caller that gates on "did the clear work"
refuses rather than proceeding to assert against a clipboard it never
controlled.

### `fn clipboard_formats`

`None` rather than `Some(vec![])` for [`clipboard_text`]'s reason one step
on: an empty list is a real and *different* answer, and a caller that read
one here would report "the application placed nothing" about a platform that
has no clipboard to place onto.

### `fn desktop_bounds`

See the Windows implementation for what this is for: a guard against a
pointer coordinate that lies off the screen and would be silently clamped.
Off Windows nothing drives a pointer at all, so there is nothing to guard.
