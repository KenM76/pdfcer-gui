# `ui-verify/sys/mod`

Every `unsafe` line in the crate, behind one safe API.

## Why the platform code is quarantined here

Two reasons.

**Review surface.** Everything else in this crate is ordinary safe Rust
that a reviewer can read quickly. Interleaving `mouse_event` calls with
check logic would mean every future check has to be reviewed as if it
contained `unsafe`, which is how `unsafe` spreads.

**Portability without pretence.** The harness only works on Windows — it
drives a Windows window through the Windows input API. But the *workspace*
must build elsewhere, or the crate gets dropped from the members list the
first time somebody runs `cargo check` on a laptop. So there are two
implementations of one API: [`win32`] on Windows, [`unsupported`]
everywhere else, which compiles and returns a clear error naming the
platform. The harness reports SKIPPED, not a build failure and not a pass.

## The API

| Function | Job |
|---|---|
| [`find_window_for_pid`] | which top-level window belongs to the process we launched |
| [`window_frame`] | where its client area is, and at what DPI scale |
| [`raise_window`] | bring it to the foreground before driving or capturing it |
| [`cursor_position`] / [`set_cursor_position`] | the pointer |
| [`mouse_button`] | primary button down/up |
| [`key_stroke`] | a virtual key press and release |
| [`capture_screen`] | a desktop region as BGRA pixels |

Note what is absent: there is no `send_message`, and there never will be.
See [`crate::input`] for why posting messages to the window was tried in
this project's predecessor and does not work.
