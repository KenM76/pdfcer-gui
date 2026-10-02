//! Key messages posted to one window, so a check can press a chord in the app
//! it launched without touching the operator's keyboard.
//!
//! Contract: a posted key message reaches only that window's procedure. It
//! does not move the thread's key state, so the app sees a posted modifier
//! only if it reads the key messages themselves.

use super::win32::WindowHandle;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MAPVK_VK_TO_VSC, MapVirtualKeyW};
use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_KEYDOWN, WM_KEYUP};

/// Post a press (`down`) or release of virtual key `vk` to `w`; `false` if
/// the window's queue refused it.
pub fn post_key(w: WindowHandle, vk: u16, down: bool) -> bool {
    // SAFETY: no pointers; an out-of-range code maps to 0.
    let scan = unsafe { MapVirtualKeyW(u32::from(vk), MAPVK_VK_TO_VSC) };
    // Repeat count 1 and the scan code; a release also sets the previous-state
    // and transition bits, as a typed release does.
    let mut lparam = 1 | (scan as isize & 0xFF) << 16;
    if !down {
        lparam |= 0xC000_0000;
    }
    let msg = if down { WM_KEYDOWN } else { WM_KEYUP };
    // SAFETY: posting to a handle that may have closed fails cleanly.
    unsafe { PostMessageW(w.hwnd(), msg, usize::from(vk), lparam) != 0 }
}

/// Post a press and release of `vk` with `modifiers` held around it.
pub fn post_chord(w: WindowHandle, modifiers: &[u16], vk: u16) -> bool {
    let mut ok = true;
    for &m in modifiers {
        ok &= post_key(w, m, true);
    }
    ok &= post_key(w, vk, true) && post_key(w, vk, false);
    for &m in modifiers.iter().rev() {
        ok &= post_key(w, m, false);
    }
    ok
}
