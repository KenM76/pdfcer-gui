//! Reads a window's title-bar mode back from the desktop window manager.
//!
//! Contract: `Some(true)` is a dark caption, `Some(false)` a light one, and
//! `None` a window the manager would not answer for.

use std::ffi::c_void;

use super::win32::WindowHandle;

/// `DWMWA_USE_IMMERSIVE_DARK_MODE`.
const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;

#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmGetWindowAttribute(
        hwnd: *mut c_void,
        attribute: u32,
        value: *mut c_void,
        size: u32,
    ) -> i32;
}

/// Whether `w`'s title bar is in dark mode.
#[must_use]
pub fn dark_caption(w: WindowHandle) -> Option<bool> {
    let mut value: i32 = 0;
    let size = u32::try_from(std::mem::size_of::<i32>()).ok()?;
    // SAFETY: `value` is a live `BOOL` and `size` says so; a stale handle
    // fails cleanly with a non-zero status.
    let status = unsafe {
        DwmGetWindowAttribute(
            w.hwnd(),
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&raw mut value).cast(),
            size,
        )
    };
    (status == 0).then_some(value != 0)
}
