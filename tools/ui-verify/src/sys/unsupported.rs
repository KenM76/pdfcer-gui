//! The non-Windows implementation: the same API, every call refusing clearly.
//!
//! ## Why this file exists at all
//!
//! The harness drives a Windows window with the Windows input API and cannot
//! work anywhere else. It would be easy to say so with a `compile_error!` —
//! and that would break `cargo check --workspace` on any non-Windows machine,
//! which is how a crate gets quietly removed from the workspace members list.
//! A verification harness that has been removed from the build verifies
//! nothing.
//!
//! So the crate compiles everywhere, every platform call fails with a sentence
//! naming the platform, and the checks resolve to SKIPPED with that sentence
//! as the reason. That is the honest outcome: on a Linux box the harness has
//! learned nothing about the application, and "learned nothing" must never be
//! printed as a pass.
//!
//! ## Keeping it honest
//!
//! Every function here mirrors [`super::win32`]'s signature exactly. If the
//! Windows API grows a function and this one does not, the non-Windows build
//! breaks immediately and loudly, which is the intended failure mode — much
//! better than a stub that silently returns a plausible zero.

use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::geom::PixRect;

/// Stands in for the Windows handle so the rest of the crate type-checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowHandle(isize);

fn refuse<T>(what: &str) -> Result<T> {
    Err(Error::new(format!(
        "{what} is not available on this platform ({}). ui-verify drives a Windows \
         window through the Windows input API; on any other platform every check \
         reports SKIPPED rather than a result.",
        std::env::consts::OS
    )))
}

/// Always `None`: there is no window to find.
#[must_use]
pub fn find_window_for_pid(_pid: u32) -> Option<WindowHandle> {
    None
}

/// Always refuses.
pub fn window_frame(_w: WindowHandle) -> Result<WindowFrame> {
    refuse("measuring a window's client area")
}

/// No-op: there is nothing to raise.
pub fn raise_window(_w: WindowHandle) {}

/// Always `None`: there are no windows to ask about.
#[must_use]
pub const fn window_at(_x: i32, _y: i32) -> Option<WindowHandle> {
    None
}

/// No-op: there is nothing to move.
pub const fn move_window(_w: WindowHandle, _x: i32, _y: i32) {}

/// Does nothing. See [`raise_window`] for why the no-op stubs are silent while
/// the reading ones refuse: a check that cannot maximise still runs, it just
/// runs against whatever size the window happened to open at.
pub fn maximize_window(_w: WindowHandle) {}

/// Always refuses.
pub fn cursor_position() -> Result<(i32, i32)> {
    refuse("reading the pointer position")
}

/// Always refuses.
pub fn set_cursor_position(_x: i32, _y: i32) -> Result<()> {
    refuse("moving the pointer")
}

/// No-op: there is no pointer to click with.
pub fn mouse_button(_down: bool) {}

/// No-op: the secondary button, off Windows. See the win32 twin.
pub fn mouse_button_secondary(_down: bool) {}

/// No-op: there is no window server here to resize. See the win32 twin.
pub fn resize_window(_w: WindowHandle, _width: i32, _height: i32) {}

/// No-op: there is no window to type into.
/// Turn the mouse wheel. No-op off Windows.
pub fn wheel(_notches: i32) {}

pub fn key_stroke(_vk: u16) {}

/// Does nothing.
pub fn key_stroke_with(_modifiers: &[u16], _vk: u16) {}

/// Always `false` — there is no window server here, so nothing can be
/// confirmed to be in front. `false` rather than `true` so a caller that
/// gates typing on it refuses rather than types blind.
pub fn windows_for_pid(_pid: u32) -> Vec<WindowHandle> {
    Vec::new()
}

/// Always `None` — there is no window server here.
/// Name a window. Unsupported here; see the win32 twin.
#[must_use]
pub fn describe_window(_w: WindowHandle) -> String {
    "an unidentified window (this platform has no window inspection)".to_string()
}

pub fn pid_of_window(_w: WindowHandle) -> Option<u32> {
    None
}

/// Always `false` — there is no window server here, so nothing can be
/// confirmed to be in front. `false` rather than `true` so a caller that
/// gates typing on it refuses rather than types blind.
pub fn is_foreground(_w: WindowHandle) -> bool {
    false
}

/// Always `None` — there is no window server here.
pub fn foreground_window() -> Option<WindowHandle> {
    None
}

/// Always refuses.
pub fn capture_screen(_region: PixRect) -> Result<Vec<u8>> {
    refuse("capturing the screen")
}

/// See the win32 implementation. Runs `body` with no modifiers held.
pub fn with_modifiers<T>(_modifiers: &[u16], body: impl FnOnce() -> T) -> T {
    body()
}

/// Always `None` — there is no clipboard here.
///
/// A check that asserts on the clipboard therefore reports SKIPPED on this
/// platform, which is the honest answer. Returning `Some(String::new())` would
/// let a comparison against an expected string fail and be read as a defect in
/// the application.
#[must_use]
pub fn clipboard_text() -> Option<String> {
    None
}

/// Always `false` — nothing was cleared, because there is nothing to clear.
///
/// `false` rather than `true` so a caller that gates on "did the clear work"
/// refuses rather than proceeding to assert against a clipboard it never
/// controlled.
pub fn clear_clipboard() -> bool {
    false
}

/// Always `None` — there is no clipboard here, so there are no formats on it.
///
/// `None` rather than `Some(vec![])` for [`clipboard_text`]'s reason one step
/// on: an empty list is a real and *different* answer, and a caller that read
/// one here would report "the application placed nothing" about a platform that
/// has no clipboard to place onto.
#[must_use]
pub fn clipboard_formats() -> Option<Vec<(u32, String)>> {
    None
}

/// Always `false` off Windows: there is no keyboard to have a latch on, and
/// [`key_stroke_with`] here is a no-op. See the Windows implementation for what
/// this is for.
#[must_use]
pub const fn caps_lock_is_on() -> bool {
    false
}
