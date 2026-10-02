//! Title bars in the application's light or dark mode, through the documented
//! `DWMWA_USE_IMMERSIVE_DARK_MODE` window attribute.
//!
//! The toolkit's own route (`ViewportCommand::SetTheme`, winit's `set_theme`)
//! writes an undocumented composition attribute that `DwmGetWindowAttribute`
//! cannot read back, and winit re-derives it from the system on every settings
//! change, so a Dark preset on a light system would lose its caption.

use std::ffi::c_void;

type Hwnd = *mut c_void;

/// `DWMWA_USE_IMMERSIVE_DARK_MODE` (Windows 10 20H1 and later).
const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;

#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmSetWindowAttribute(hwnd: Hwnd, attribute: u32, value: *const c_void, size: u32) -> i32;
}

unsafe extern "system" {
    fn EnumWindows(
        callback: Option<unsafe extern "system" fn(Hwnd, isize) -> i32>,
        lparam: isize,
    ) -> i32;
    fn GetWindowThreadProcessId(hwnd: Hwnd, pid: *mut u32) -> u32;
    fn GetCurrentProcessId() -> u32;
}

/// The enumeration's state: this process's id and the windows found.
struct Ours {
    pid: u32,
    found: Vec<Hwnd>,
}

unsafe extern "system" fn collect(hwnd: Hwnd, lparam: isize) -> i32 {
    // SAFETY: `lparam` is the `&mut Ours` `set_dark_captions` passed to the
    // synchronous `EnumWindows`, so the borrow is live for the whole call.
    let ours = unsafe { &mut *(lparam as *mut Ours) };
    let mut pid: u32 = 0;
    // SAFETY: `hwnd` comes from the enumerator; `pid` is a live local.
    unsafe { GetWindowThreadProcessId(hwnd, &raw mut pid) };
    if pid == ours.pid {
        ours.found.push(hwnd);
    }
    1
}

/// Sets every top-level window of this process to a dark (`true`) or light
/// title bar, and answers how many windows took it. Hidden helper windows are
/// included: the attribute is harmless on them and a window shown later keeps
/// it.
#[must_use]
pub fn set_dark_captions(dark: bool) -> usize {
    let mut ours = Ours {
        // SAFETY: no arguments.
        pid: unsafe { GetCurrentProcessId() },
        found: Vec::new(),
    };
    // SAFETY: `collect` matches `WNDENUMPROC`; the pointer is to a local that
    // outlives this synchronous call.
    unsafe { EnumWindows(Some(collect), (&raw mut ours) as isize) };
    let value = i32::from(dark);
    let size = u32::try_from(std::mem::size_of::<i32>()).unwrap_or(4);
    ours.found
        .into_iter()
        .filter(|&hwnd| {
            // SAFETY: `hwnd` is a window of this process from the enumeration;
            // `value` is a live `BOOL` and `size` says so.
            let status = unsafe {
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE,
                    (&raw const value).cast(),
                    size,
                )
            };
            status == 0
        })
        .count()
}
