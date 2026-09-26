//! The Windows implementation of the platform API.
//!
//! Design and rationale: `docs/modules/ui-verify/sys/win32.md`.

use std::ffi::c_void;

use windows_sys::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, ClientToScreen, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC,
    SRCCOPY, SelectObject,
};
use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, VK_CAPITAL, keybd_event, mouse_event,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GA_ROOT, GetAncestor, GetClassNameW, GetClientRect,
    GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    SW_MAXIMIZE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetCursorPos, SetForegroundWindow,
    SetWindowPos, ShowWindow, WindowFromPoint,
};

use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::geom::PixRect;

/// An opaque handle to a top-level window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowHandle(isize);

impl WindowHandle {
    fn hwnd(self) -> HWND {
        self.0 as HWND
    }

    /// Wrap a raw handle. Private to this module, for [`window_at`].
    fn from_raw(hwnd: HWND) -> Option<Self> {
        if hwnd.is_null() {
            None
        } else {
            Some(Self(hwnd as isize))
        }
    }
}

/// State for [`EnumWindows`]' callback: the pid to look for, and the first
/// visible window found for it.
struct Search {
    pid: u32,
    found: Option<isize>,
}

/// `EnumWindows` callback. Records the first **visible** top-level window
/// belonging to the target process and stops.
///
/// Visibility matters: a winit application creates helper windows, and an
/// invisible one has a nonsensical rect that would be used as the client area
/// for every subsequent conversion.
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    // SAFETY: `lparam` is the `&mut Search` this module passed to
    // `EnumWindows` on the line below; Windows passes it back unchanged, and
    // `EnumWindows` is synchronous so the borrow is live for the whole call.
    let search = unsafe { &mut *(lparam as *mut Search) };
    let mut pid: u32 = 0;
    // SAFETY: `hwnd` is supplied by the enumerator and `pid` is a live local.
    unsafe { GetWindowThreadProcessId(hwnd, &raw mut pid) };
    // SAFETY: `hwnd` is supplied by the enumerator.
    if pid == search.pid && unsafe { IsWindowVisible(hwnd) } != 0 {
        search.found = Some(hwnd as isize);
        return 0; // stop enumerating
    }
    1 // keep going
}

/// State for the all-windows enumerator.
struct SearchAll {
    pid: u32,
    found: Vec<isize>,
}

/// Collect EVERY visible top-level window belonging to a pid.
unsafe extern "system" fn enum_proc_all(hwnd: HWND, lparam: LPARAM) -> i32 {
    // SAFETY: as `enum_proc` — the pointer is the `&mut SearchAll` handed to
    // `EnumWindows`, which is synchronous.
    let search = unsafe { &mut *(lparam as *mut SearchAll) };
    let mut pid: u32 = 0;
    // SAFETY: `hwnd` is supplied by the enumerator.
    unsafe { GetWindowThreadProcessId(hwnd, &raw mut pid) };
    // SAFETY: `hwnd` is supplied by the enumerator.
    if pid == search.pid && unsafe { IsWindowVisible(hwnd) } != 0 {
        search.found.push(hwnd as isize);
    }
    1
}

/// **Every visible top-level window belonging to `pid`.**
#[must_use]
pub fn windows_for_pid(pid: u32) -> Vec<WindowHandle> {
    let mut search = SearchAll {
        pid,
        found: Vec::new(),
    };
    // SAFETY: `enum_proc_all` matches `WNDENUMPROC`, and the pointer is to a
    // stack local that outlives this synchronous call.
    unsafe {
        EnumWindows(Some(enum_proc_all), (&raw mut search) as LPARAM);
    }
    search.found.into_iter().map(WindowHandle).collect()
}

/// The process a window belongs to.
#[must_use]
pub fn pid_of_window(w: WindowHandle) -> Option<u32> {
    let mut pid: u32 = 0;
    // SAFETY: `w` is a handle this module produced; the call tolerates a stale
    // one by returning 0.
    unsafe { GetWindowThreadProcessId(w.hwnd(), &raw mut pid) };
    (pid != 0).then_some(pid)
}

/// The first visible top-level window belonging to `pid`, if it has one yet.
///
/// `None` is a normal early answer, not an error: a freshly launched
/// application has no window for several hundred milliseconds. Callers poll.
#[must_use]
pub fn find_window_for_pid(pid: u32) -> Option<WindowHandle> {
    let mut search = Search { pid, found: None };
    // SAFETY: `enum_proc` matches the `WNDENUMPROC` signature, and the pointer
    // handed across is to a stack local that outlives this synchronous call.
    unsafe {
        EnumWindows(Some(enum_proc), (&raw mut search) as LPARAM);
    }
    search.found.map(WindowHandle)
}

/// Where the window's client area is on the desktop, how big it is, and at
/// what DPI scale.
pub fn window_frame(w: WindowHandle) -> Result<WindowFrame> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    // SAFETY: `w` is a handle this module produced; `rect` is a live local.
    if unsafe { GetClientRect(w.hwnd(), &raw mut rect) } == 0 {
        return Err(Error::new("GetClientRect failed for the target window"));
    }
    let mut origin = POINT { x: 0, y: 0 };
    // SAFETY: as above. `ClientToScreen` maps the client-space point (0, 0) to
    // desktop coordinates, which is the client origin.
    if unsafe { ClientToScreen(w.hwnd(), &raw mut origin) } == 0 {
        return Err(Error::new("ClientToScreen failed for the target window"));
    }
    // SAFETY: as above. Returns 0 for an invalid window, handled below.
    let dpi = unsafe { GetDpiForWindow(w.hwnd()) };
    let scale = if dpi == 0 { 1.0 } else { dpi as f32 / 96.0 };

    Ok(WindowFrame {
        client_origin: (origin.x, origin.y),
        client_size: (
            (rect.right - rect.left).max(0) as u32,
            (rect.bottom - rect.top).max(0) as u32,
        ),
        scale,
    })
}

/// Bring the window to the front and show it.
pub fn raise_window(w: WindowHandle) {
    // `AttachThreadInput` AROUND THE RAISE, and it is not defensive
    // decoration — it is the documented way this call is allowed to succeed.
    //
    // Windows refuses `SetForegroundWindow` to a process that does not already
    // own the foreground. This harness is exactly that process: it launches the
    // application and then asks, from the outside, for it to come forward. The
    // sanctioned workaround is to attach this thread's input queue to the
    // thread that currently owns the foreground, which makes the two count as
    // one input context for the duration, and detach again immediately.
    //
    // The symptom is why it is worth the unsafe block. **The condition is a
    // permissions rule, not a race**, so a retry does not clear it: a bare
    // `SetForegroundWindow` from a background process is simply refused, and
    // whatever holds the foreground keeps it. Measured once as a whole sweep
    // stalling with every input check reporting *"the foreground is held by
    // BluetoothNotificationAreaIconWindowClass"* — an **invisible** 136 x 39
    // explorer tray helper at the origin, for which the harness's advice to
    // dismiss it is unactionable; and once as 45 of 127 checks skipping on
    // *"could not be brought to the front"*, each passing when re-run alone.
    // `Driver::raise_and_confirm_at`'s single retry stays because it costs
    // nothing and covers genuine churn, but this is the mechanism.
    //
    // SAFETY: every call is side-effect-only and tolerates a stale handle by
    // returning false. The attach is unconditionally undone on both paths, so
    // the input queues cannot be left joined.
    unsafe {
        let ours = GetCurrentThreadId();
        let fg = GetForegroundWindow();
        let theirs = if fg.is_null() {
            0
        } else {
            GetWindowThreadProcessId(fg, std::ptr::null_mut())
        };
        let attached = theirs != 0 && theirs != ours && AttachThreadInput(ours, theirs, 1) != 0;
        ShowWindow(w.hwnd(), SW_SHOW);
        BringWindowToTop(w.hwnd());
        SetForegroundWindow(w.hwnd());
        if attached {
            AttachThreadInput(ours, theirs, 0);
        }

        // THE ALT NUDGE, and it is the only thing that recovers a stuck
        // foreground; `AttachThreadInput` alone does not.
        //
        // Windows grants `SetForegroundWindow` to a process that has received
        // recent user input. Synthesising a bare Alt press-and-release is the
        // long-standing way to satisfy that rule from a harness: it is input,
        // it targets nothing, and Alt on its own opens no menu.
        //
        // The condition that forces it: an **invisible** 136 x 39 explorer
        // tray helper (`BluetoothNotificationAreaIconWindowClass`) takes the
        // foreground and yields to nothing — not a retry, not
        // `AttachThreadInput`, not an explicit `SetForegroundWindow` from an
        // elevated shell — and recurs within a minute of being cleared by
        // hand. Without this nudge a sweep cannot be run at all while it is up.
        //
        // Guarded on failure, so the ordinary path never synthesises input.
        // A harness that pressed Alt before every raise would be injecting a
        // keystroke into the application it is measuring, which is exactly the
        // kind of side effect that makes a check's result mean something else.
        if GetForegroundWindow() != w.hwnd() {
            const VK_MENU: u8 = 0x12;
            keybd_event(VK_MENU, 0, 0, 0);
            std::thread::sleep(std::time::Duration::from_millis(40));
            keybd_event(VK_MENU, 0, KEYEVENTF_KEYUP, 0);
            std::thread::sleep(std::time::Duration::from_millis(60));
            SetForegroundWindow(w.hwnd());
        }
    }
}

/// Maximise the window.
pub fn maximize_window(w: WindowHandle) {
    // SAFETY: `w` is a handle this module produced. `ShowWindow` is
    // side-effect-only and tolerates a stale handle by returning false.
    unsafe {
        ShowWindow(w.hwnd(), SW_MAXIMIZE);
    }
}

/// The pointer's current desktop position.
pub fn cursor_position() -> Result<(i32, i32)> {
    let mut p = POINT { x: 0, y: 0 };
    // SAFETY: `p` is a live local.
    if unsafe { GetCursorPos(&raw mut p) } == 0 {
        return Err(Error::new("GetCursorPos failed"));
    }
    Ok((p.x, p.y))
}

/// Move the pointer.
pub fn set_cursor_position(x: i32, y: i32) -> Result<()> {
    // SAFETY: no pointers involved; fails by returning false.
    if unsafe { SetCursorPos(x, y) } != 0 {
        return Ok(());
    }
    // One retry, 120 ms later. See the doc comment.
    std::thread::sleep(std::time::Duration::from_millis(120));
    // SAFETY: as above.
    if unsafe { SetCursorPos(x, y) } == 0 {
        return Err(Error::new(format!(
            "SetCursorPos({x}, {y}) failed — the coordinate may be off every monitor, or \
             another process may hold a pointer capture"
        )));
    }
    Ok(())
}

/// Press (`true`) or release (`false`) the primary mouse button, wherever the
/// pointer currently is.
pub fn mouse_button(down: bool) {
    let flags = if down {
        MOUSEEVENTF_LEFTDOWN
    } else {
        MOUSEEVENTF_LEFTUP
    };
    // SAFETY: no pointers; the extra-info argument is unused (0).
    unsafe { mouse_event(flags, 0, 0, 0, 0) };
}

/// Press (`true`) or release (`false`) the **secondary** mouse button.
pub fn mouse_button_secondary(down: bool) {
    let flags = if down {
        MOUSEEVENTF_RIGHTDOWN
    } else {
        MOUSEEVENTF_RIGHTUP
    };
    // SAFETY: no pointers; the extra-info argument is unused (0).
    unsafe { mouse_event(flags, 0, 0, 0, 0) };
}

/// Turn the mouse wheel at the pointer's current position.
pub fn wheel(notches: i32) {
    const WHEEL_DELTA: i32 = 120;
    unsafe { mouse_event(MOUSEEVENTF_WHEEL, 0, 0, notches * WHEEL_DELTA, 0) };
}

/// Press and release a virtual key.
pub fn key_stroke(vk: u16) {
    // SAFETY: no pointers; the scan-code argument is 0, which tells Windows to
    // derive it from the virtual key.
    unsafe {
        keybd_event(vk as u8, 0, 0, 0);
        keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}

/// **Which top-level window owns this screen point.**
pub fn window_at(x: i32, y: i32) -> Option<WindowHandle> {
    // SAFETY: both calls take plain values and tolerate any input, returning
    // null for a point on no window.
    unsafe {
        let hwnd = WindowFromPoint(POINT { x, y });
        if hwnd.is_null() {
            return None;
        }
        let root = GetAncestor(hwnd, GA_ROOT);
        WindowHandle::from_raw(if root.is_null() { hwnd } else { root })
    }
}

/// **The whole desktop, as `(x, y, width, height)` in physical pixels.**
#[must_use]
pub fn desktop_bounds() -> (i32, i32, i32, i32) {
    // SAFETY: `GetSystemMetrics` takes a plain index and returns a plain int.
    unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    }
}

/// **Move the window to a known position**, without resizing it.
pub fn move_window(w: WindowHandle, x: i32, y: i32) {
    // SAFETY: `w` is a handle this module produced; `SetWindowPos` tolerates a
    // stale handle by returning false. `SWP_NOSIZE | SWP_NOZORDER` keeps the
    // size and the stacking exactly as they were.
    unsafe {
        SetWindowPos(
            w.hwnd(),
            std::ptr::null_mut(),
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER,
        );
    }
}

/// **Resize the window**, keeping its position.
pub fn resize_window(w: WindowHandle, width: i32, height: i32) {
    // SAFETY: `w` is a handle this module produced; `SetWindowPos` tolerates a
    // stale handle by returning false.
    unsafe {
        SetWindowPos(
            w.hwnd(),
            std::ptr::null_mut(),
            0,
            0,
            width,
            height,
            SWP_NOMOVE | SWP_NOZORDER,
        );
    }
}

/// Whether `w` is the window that will receive keystrokes right now.
pub fn is_foreground(w: WindowHandle) -> bool {
    // SAFETY: no pointers, no ownership; returns a handle or null.
    unsafe { GetForegroundWindow() == w.hwnd() }
}

/// **Whatever window currently has the foreground**, whoever owns it.
#[must_use]
pub fn foreground_window() -> Option<WindowHandle> {
    // SAFETY: no pointers, no ownership; returns a handle or null.
    let hwnd = unsafe { GetForegroundWindow() };
    WindowHandle::from_raw(hwnd)
}

/// **Name whatever currently holds the foreground**, for a refusal message.
#[must_use]
pub fn describe_foreground() -> String {
    let Some(w) = foreground_window() else {
        return "nothing at all (no foreground window), which usually means the workstation is locked or the secure desktop is up"
            .to_string();
    };
    describe_window(w)
}

/// Name **any** window, the way [`describe_foreground`] names the front one.
#[must_use]
pub fn describe_window(w: WindowHandle) -> String {
    let class = window_class(w);
    let title = window_title(w);
    let pid = pid_of_window(w).unwrap_or(0);
    let named = if title.is_empty() {
        "untitled".to_string()
    } else {
        format!("\"{title}\"")
    };
    format!("{named} (window class `{class}`, pid {pid})")
}

/// The window's class name, or `?` if it cannot be read.
#[must_use]
fn window_class(w: WindowHandle) -> String {
    let mut buf = [0u16; 256];
    // SAFETY: `buf` is a real array and its length is passed honestly; the
    // call writes at most that many code units and tolerates a stale handle
    // by returning 0.
    let n = unsafe { GetClassNameW(w.hwnd(), buf.as_mut_ptr(), buf.len() as i32) };
    if n <= 0 {
        return "?".to_string();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

/// The window's title bar text, empty if it has none.
#[must_use]
fn window_title(w: WindowHandle) -> String {
    let mut buf = [0u16; 512];
    // SAFETY: as `window_class` above.
    let n = unsafe { GetWindowTextW(w.hwnd(), buf.as_mut_ptr(), buf.len() as i32) };
    if n <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

/// Press and release a virtual key **while modifiers are held**.
const CHORD_GAP: std::time::Duration = std::time::Duration::from_millis(12);

pub fn key_stroke_with(modifiers: &[u16], vk: u16) {
    // SAFETY: no pointers; the scan-code argument is 0, which tells Windows to
    // derive it from the virtual key. Same contract as `key_stroke`.
    unsafe {
        for m in modifiers {
            keybd_event(*m as u8, 0, 0, 0);
        }
        // Let the target see a frame with the modifiers held and the key not
        // yet pressed. See CHORD_GAP.
        std::thread::sleep(CHORD_GAP);
        keybd_event(vk as u8, 0, 0, 0);
        std::thread::sleep(CHORD_GAP);
        keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
        std::thread::sleep(CHORD_GAP);
        // The releases stay unconditional and un-gated by any early return,
        // for the reason above: a leaked modifier is a stuck key on the
        // operator's real keyboard. The sleeps are between the posts, never
        // around the loop, so no path can skip a release.
        for m in modifiers.iter().rev() {
            keybd_event(*m as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }
}

/// Grab a desktop region as BGRA pixels, top row first.
pub fn capture_screen(region: PixRect) -> Result<Vec<u8>> {
    if region.area() == 0 {
        return Err(Error::new("refusing to capture a zero-area region"));
    }
    let w = region.w as i32;
    let h = region.h as i32;

    // SAFETY: `GetDC(null)` returns a DC for the whole screen, released below.
    let screen_dc = unsafe { GetDC(std::ptr::null_mut()) };
    if screen_dc.is_null() {
        return Err(Error::new("GetDC(NULL) failed — no screen device context"));
    }

    // A closure so every early return releases the screen DC exactly once.
    let result = (|| -> Result<Vec<u8>> {
        // SAFETY: `screen_dc` is a valid DC obtained above.
        let mem_dc = unsafe { CreateCompatibleDC(screen_dc) };
        if mem_dc.is_null() {
            return Err(Error::new("CreateCompatibleDC failed"));
        }
        // SAFETY: as above.
        let bitmap = unsafe { CreateCompatibleBitmap(screen_dc, w, h) };
        if bitmap.is_null() {
            // SAFETY: `mem_dc` is valid and not yet deleted.
            unsafe { DeleteDC(mem_dc) };
            return Err(Error::new("CreateCompatibleBitmap failed"));
        }

        // SAFETY: both handles are valid; the previous object is restored
        // before the DC is deleted, as GDI requires.
        let old = unsafe { SelectObject(mem_dc, bitmap) };
        // SAFETY: valid DCs and an in-range source rectangle (clipped by GDI
        // if it extends past the desktop).
        let blitted = unsafe {
            BitBlt(
                mem_dc,
                0,
                0,
                w,
                h,
                screen_dc,
                region.x as i32,
                region.y as i32,
                SRCCOPY,
            )
        };

        let mut out = vec![0u8; (region.w as usize) * (region.h as usize) * 4];
        let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            // NEGATIVE height requests a TOP-DOWN DIB. Without it GDI hands
            // back a bottom-up bitmap and every row is mirrored — which is not
            // obviously wrong in a screenshot of a symmetric-looking window,
            // and would silently make every region lookup sample the wrong
            // part of the picture.
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        let copied = if blitted == 0 {
            0
        } else {
            // SAFETY: `out` is sized exactly `w * h * 4` to match the header,
            // and `info` is a live local.
            unsafe {
                GetDIBits(
                    mem_dc,
                    bitmap,
                    0,
                    region.h,
                    out.as_mut_ptr().cast::<c_void>(),
                    &raw mut info,
                    DIB_RGB_COLORS,
                )
            }
        };

        // SAFETY: restore then delete, in that order, exactly once each.
        unsafe {
            SelectObject(mem_dc, old);
            DeleteObject(bitmap);
            DeleteDC(mem_dc);
        }

        if blitted == 0 {
            return Err(Error::new(format!(
                "BitBlt of {}x{} at ({}, {}) failed",
                region.w, region.h, region.x, region.y
            )));
        }
        if copied == 0 {
            return Err(Error::new("GetDIBits copied no scanlines"));
        }
        Ok(out)
    })();

    // SAFETY: `screen_dc` came from `GetDC(NULL)` and is released once.
    unsafe { ReleaseDC(std::ptr::null_mut(), screen_dc) };
    result
}

/// Hold `modifiers` down, run `body`, and release them **on every path**.
pub fn with_modifiers<T>(modifiers: &[u16], body: impl FnOnce() -> T) -> T {
    // SAFETY: no pointers; scan code 0 tells Windows to derive it from the
    // virtual key. Same contract as `key_stroke`.
    unsafe {
        for m in modifiers {
            keybd_event(*m as u8, 0, 0, 0);
        }
    }
    // Let the target see a frame with the modifiers held before the click
    // arrives — the same reason `key_stroke_with` sleeps between its posts, and
    // it matters more here because the application reads modifier state on the
    // frame it processes the press, not on the frame the press was posted.
    std::thread::sleep(CHORD_GAP);
    let out = body();
    std::thread::sleep(CHORD_GAP);
    unsafe {
        for m in modifiers.iter().rev() {
            keybd_event(*m as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }
    out
}

// ===========================================================================
// The clipboard
// ===========================================================================
//
// WHY THE HARNESS HAS TO READ THE CLIPBOARD ITSELF
//
// Defect O18: the operator selected text, pressed Ctrl+C, pasted into Notepad
// and got "1 object copied from pdfcer" — because Ctrl+C reached the object
// clipboard instead of the text one. It had been broken since the day it was
// written, under 1,628 passing unit tests.
//
// **No test in the application's own suite can see that**, and no amount of
// tracing fixes it: the trace can say `text-copy source=selection`, and be
// telling the truth, while a later handler in the same frame overwrites the
// clipboard. The only oracle for "what does the operator get when they paste"
// is the operating system's clipboard, read from outside the process.
//
// So this is not harness convenience. It is the *only* place the assertion the
// defect needs can be made.
//
// A NOTE ON THE DEPENDENCY POSTURE. This adds two `windows-sys` FEATURES,
// not a dependency. This crate's manifest records that a new dependency which
// is not already in `D:\Dev\pdfcer`'s lockfile is an operator decision;
// `windows-sys 0.61` is already there and already linked, so enabling
// `Win32_System_DataExchange` and `Win32_System_Memory` changes nothing about
// what ships or what has to be reviewed for licensing.

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard,
};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};

/// `CF_UNICODETEXT`, the only clipboard format this harness reads.
const CF_UNICODETEXT: u32 = 13;

/// How many times to retry opening the clipboard, and how long to wait between.
const OPEN_ATTEMPTS: u32 = 20;
const OPEN_RETRY_MS: u64 = 25;

/// Take ownership of the clipboard, run `body`, and always release it.
fn with_clipboard<T>(body: impl FnOnce() -> T) -> Option<T> {
    for _ in 0..OPEN_ATTEMPTS {
        // SAFETY: a null window handle is documented as associating the
        // clipboard with the current task, which is what a harness wants.
        let opened = unsafe { OpenClipboard(std::ptr::null_mut()) };
        if opened != 0 {
            let out = body();
            // SAFETY: paired with the successful OpenClipboard above.
            unsafe {
                CloseClipboard();
            }
            return Some(out);
        }
        std::thread::sleep(std::time::Duration::from_millis(OPEN_RETRY_MS));
    }
    None
}

/// The clipboard's text, or `None`.
#[must_use]
pub fn clipboard_text() -> Option<String> {
    with_clipboard(|| {
        // SAFETY: the clipboard is open; a null return means "no such format",
        // which is not an error.
        let handle: HANDLE = unsafe { GetClipboardData(CF_UNICODETEXT) };
        if handle.is_null() {
            return None;
        }
        // SAFETY: `GlobalLock` on a clipboard handle yields a pointer valid
        // until the matching `GlobalUnlock`, and the block is a NUL-terminated
        // UTF-16 string by the definition of CF_UNICODETEXT.
        let ptr = unsafe { GlobalLock(handle) }.cast::<u16>();
        if ptr.is_null() {
            return None;
        }
        let mut len = 0usize;
        // SAFETY: walking to the NUL terminator the format guarantees.
        while unsafe { *ptr.add(len) } != 0 {
            len += 1;
            // A clipboard string is operator text, not a stream. This bound
            // stops a corrupt or unterminated block from hanging the harness
            // rather than failing it — 16 MB of UTF-16 is far beyond anything a
            // copy from a PDF can produce.
            if len > 8 * 1024 * 1024 {
                break;
            }
        }
        // SAFETY: `len` units were just walked and found in bounds.
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let text = String::from_utf16_lossy(slice);
        // SAFETY: paired with the GlobalLock above.
        unsafe {
            GlobalUnlock(handle);
        }
        Some(text)
    })
    .flatten()
}

/// Empty the clipboard, reporting whether it was actually emptied.
pub fn clear_clipboard() -> bool {
    with_clipboard(|| {
        // SAFETY: the clipboard is open and owned by this task.
        unsafe { EmptyClipboard() != 0 }
    })
    .unwrap_or(false)
}

/// **Every format on the clipboard, IN PLACEMENT ORDER, with its name.**
#[must_use]
pub fn clipboard_formats() -> Option<Vec<(u32, String)>> {
    use windows_sys::Win32::System::DataExchange::{EnumClipboardFormats, GetClipboardFormatNameW};
    with_clipboard(|| {
        let mut out: Vec<(u32, String)> = Vec::new();
        let mut id: u32 = 0;
        loop {
            // SAFETY: the clipboard is open and owned by this task, which is
            // `EnumClipboardFormats`' only precondition. Passing 0 asks for the
            // first format; passing the previous id asks for the next. A zero
            // return ends the walk (and also signals an error, which for a
            // read-only oracle is the same outcome: nothing more to report).
            id = unsafe { EnumClipboardFormats(id) };
            if id == 0 {
                break;
            }
            let mut buffer = [0u16; 256];
            // SAFETY: the buffer is a live array of exactly `len` `u16`s and
            // the call writes at most that many. A zero return means the format
            // has no name — every predefined `CF_*` is in that case — which is
            // reported as an empty string rather than as a failure.
            let written =
                unsafe { GetClipboardFormatNameW(id, buffer.as_mut_ptr(), buffer.len() as i32) };
            let name = if written > 0 {
                String::from_utf16_lossy(&buffer[..written as usize])
            } else {
                String::new()
            };
            out.push((id, name));
            // A clipboard with more entries than this is not one this
            // application produced; the bound stops a driver bug from hanging
            // the harness rather than failing it.
            if out.len() > 64 {
                break;
            }
        }
        out
    })
}

/// **Is CapsLock currently latched on?**
///
/// # The failure this exists for
///
/// `an_encrypted_document_can_be_opened_with_its_password` types the fixture's
/// documented user password, `userpw`. Under a latched CapsLock the
/// application answers:
///
/// ```text
/// password-submitted chars=6 non_ascii=0
/// password-rejected  attempt=2 reason=wrong
/// ```
///
/// Six characters, all ASCII, and wrong. The check's own failure message —
/// *"the fixture's user password is published in its PROVENANCE file; if it has
/// changed, this check is aimed at the wrong string"* — sends a reader to a
/// provenance file that is correct, against a fixture byte-identical to the
/// engine's copy. `Driver::type_ascii` spells a lowercase letter by pressing
/// that letter's virtual key with no Shift, which under a latched CapsLock
/// produces `USERPW`.
///
/// The damage is not confined to one check: every check that types letters
/// types the wrong case, and most of them compare what they typed against
/// nothing, so they pass. The one that DOES compare fails and blames the
/// document. **A machine state nobody set is the hardest kind of wrong answer
/// to see** — the same shape as a locked workstation making the foreground
/// unreachable.
///
/// Read the state and compensate, rather than clearing it: CapsLock belongs
/// to the operator, and a harness that toggles his keyboard's latches is a
/// harness that leaves his machine changed. See `Driver::type_ascii`.
#[must_use]
pub fn caps_lock_is_on() -> bool {
    // SAFETY: no pointers, no allocation. `GetKeyState` reads the calling
    // thread's view of the keyboard state and cannot fail; the low bit of the
    // return is the TOGGLE state, which is what a latch is.
    (unsafe { GetKeyState(VK_CAPITAL as i32) } & 1) != 0
}
