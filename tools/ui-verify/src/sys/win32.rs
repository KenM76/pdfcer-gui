//! The Windows implementation of the platform API.
//!
//! Everything here is a thin, documented wrapper over one Win32 call. The
//! interesting decisions are recorded at the function that embodies them; the
//! cross-cutting ones are here.
//!
//! ## Why `GetDC(NULL)` + `BitBlt` and not `PrintWindow`
//!
//! [`capture_screen`] photographs the **composited desktop**, not the window's
//! own device context. The window's DC is the obvious choice and it is wrong
//! for this application: eframe renders through glow/wgpu, and a
//! GPU-composited surface frequently comes back **blank** from a
//! `PrintWindow`/`BitBlt` of the window DC. A blank capture is the worst
//! possible failure here, because it is indistinguishable from a real one at
//! the call site — the file exists, the call succeeded, and only a human
//! looking at the PNG can tell it is not evidence. This project's predecessor
//! recorded exactly that, twice, and recorded a plausible-but-invented cause
//! being attached to it before the real one was found.
//!
//! The consequence of reading the desktop is that whatever is *in front of*
//! the window is what gets photographed. Hence [`raise_window`], and hence the
//! near-uniformity guard in [`crate::pixels::region_not_uniform`], which is
//! the mechanical version of "a human looked at it".
//!
//! ## Why the window search is by process id
//!
//! Not by title, and not by class. Titles change with the open document; class
//! names are winit's business and not a contract. The process id is the one
//! thing the harness knows for certain, because it launched the process. It
//! also guarantees the harness can never drive a window belonging to an
//! instance the operator opened for their own work — a hazard the predecessor
//! scripts hit hard enough to write four paragraphs about.

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
    KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, keybd_event, mouse_event,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GA_ROOT, GetAncestor, GetClassNameW, GetClientRect,
    GetCursorPos, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    SW_MAXIMIZE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetCursorPos, SetForegroundWindow,
    SetWindowPos, ShowWindow, WindowFromPoint,
};

use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::geom::PixRect;

/// An opaque handle to a top-level window.
///
/// Wrapped rather than passed as a raw `HWND` so the rest of the crate never
/// has a pointer in its types, and so the `unsupported` build can offer the
/// same shape.
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
///
/// ★★ Written 2026-08-21, when thirteen dialogs became real OS windows and six
/// driven checks began clicking hundreds of pixels from the control they named.
/// A process used to have exactly one window; it now has one per open dialog,
/// and a harness that knows only the first cannot raise the one it is aiming
/// at — so it raises the main window instead, which puts the dialog BEHIND it,
/// and the click lands on the application.
///
/// Order is `EnumWindows`' own, which is **z-order, front to back**. Callers
/// that want a specific window must identify it by geometry rather than by
/// position in this list: z-order is what the raise is about to change.
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
///
/// The **client** area, not the window rect. The two differ by the title bar
/// and the border, and every logical coordinate the application traces is
/// relative to the client origin. Using the window rect would put a constant
/// offset — around 30 px vertically, and DPI-dependent — into every single
/// conversion, which is small enough to still hit *something* and therefore
/// exactly the kind of error that gets diagnosed as a hit-test bug.
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
///
/// Best-effort by Windows' own rules — a process without foreground rights may
/// be refused, so the boolean result is deliberately not turned into an error.
/// The consequence of a refusal is a screenshot of whatever is in front, which
/// is why the uniformity guard exists downstream rather than here.
///
/// pdfcer's predecessor script added this after a capture returned a
/// pixel-perfect screenshot of a completely different application: the target
/// had started, run its whole script and traced correctly, but its window was
/// created behind an already-maximised window, so the capture photographed
/// whoever owned those pixels.
pub fn raise_window(w: WindowHandle) {
    // ★★★ `AttachThreadInput` AROUND THE RAISE, and it is not defensive
    // decoration — it is the documented way this call is allowed to succeed.
    //
    // Windows refuses `SetForegroundWindow` to a process that does not already
    // own the foreground. This harness is exactly that process: it launches the
    // application and then asks, from the outside, for it to come forward. The
    // sanctioned workaround is to attach this thread's input queue to the
    // thread that currently owns the foreground, which makes the two count as
    // one input context for the duration, and detach again immediately.
    //
    // ★★ Found on 2026-09-02, and the symptom is why it is worth the unsafe
    // block. A sweep stalled with every input check reporting *"the foreground
    // is held by BluetoothNotificationAreaIconWindowClass"* — an **invisible**
    // 136 x 39 explorer tray helper at the origin. The harness's own advice was
    // *"dismiss it and run again"*, which is unactionable for a window with no
    // pixels, and its one retry did not help because the condition is not a
    // race: a bare `SetForegroundWindow` from a background process is simply
    // refused, and whatever holds the foreground keeps it.
    //
    // ★ This is also the likely root of the older measurement recorded in
    // `Driver::raise_and_confirm_at`: a full sweep reporting 45 of 127 checks
    // skipped on *"could not be brought to the front"*, each passing when
    // re-run alone. That was patched with a single retry, which treats a
    // permissions rule as a timing one. The retry stays — it costs nothing and
    // covers the genuine churn case — but this is the mechanism.
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

        // ★★★ THE ALT NUDGE, and it is the only thing that actually recovered a
        // stuck foreground — measured 2026-09-02, after `AttachThreadInput`
        // alone did not.
        //
        // Windows grants `SetForegroundWindow` to a process that has received
        // recent user input. Synthesising a bare Alt press-and-release is the
        // long-standing way to satisfy that rule from a harness: it is input,
        // it targets nothing, and Alt on its own opens no menu.
        //
        // ★★ The condition that forced it: an **invisible** 136 x 39 explorer
        // tray helper (`BluetoothNotificationAreaIconWindowClass`) took the
        // foreground and would not yield — not to a retry, not to
        // `AttachThreadInput`, and not to an explicit `SetForegroundWindow`
        // from an elevated shell. The harness's own advice was *"dismiss it and
        // run again"*, which is unactionable for a window with no pixels. It
        // recurred within a minute of being cleared by hand, so a sweep could
        // not be run at all without this.
        //
        // ★ Guarded on failure, so the ordinary path never synthesises input.
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
///
/// # ★ Why a harness needs this, and what it stops being a false failure
///
/// A ribbon **overflows** when it is wider than its window: groups past the fold
/// move into an overflow menu, and their controls stop publishing a rect. That
/// is correct application behaviour and it is indistinguishable, from a check's
/// point of view, from *"the control does not exist"*.
///
/// It was found the way these things are always found: `settings_theme` clicked
/// the File tab, asked for `ribbon.item.file.settings`, and was told the tab
/// published ten controls of which that was not one — because at the default
/// window size the File tab's last two groups were in the overflow. The check
/// would have reported a shipped feature as missing.
///
/// Maximising is the right fix rather than *"open the overflow menu"* for two
/// reasons. A menu's contents are not published as regions, so a check could
/// not aim at them anyway; and a maximised window is the state an operator
/// running a drawing tool on a desktop is overwhelmingly in, so it is also the
/// state most worth verifying.
///
/// What it costs is real and belongs in the record: a check that maximises is
/// **not** testing the narrow-window layout. Whether every control survives a
/// small window is a separate question and needs its own check.
pub fn maximize_window(w: WindowHandle) {
    // SAFETY: `w` is a handle this module produced. `ShowWindow` is
    // side-effect-only and tolerates a stale handle by returning false.
    unsafe {
        ShowWindow(w.hwnd(), SW_MAXIMIZE);
    }
}

/// The pointer's current desktop position.
///
/// Read before a run so it can be put back afterwards. Driving the real cursor
/// is the cost of testing the real input path (see [`crate::input`]); moving
/// the operator's pointer and *leaving* it moved is not part of that bargain.
pub fn cursor_position() -> Result<(i32, i32)> {
    let mut p = POINT { x: 0, y: 0 };
    // SAFETY: `p` is a live local.
    if unsafe { GetCursorPos(&raw mut p) } == 0 {
        return Err(Error::new("GetCursorPos failed"));
    }
    Ok((p.x, p.y))
}

/// Move the pointer.
///
/// # ★★ Why it tries twice, measured 2026-08-31
///
/// `SetCursorPos` failed once, at a coordinate that was demonstrably on screen
/// and inside the target window, and succeeded at that same coordinate on the
/// very next run of the same check. The cause is a **transient**: the previous
/// session’s window had just been killed, and for a few milliseconds after a
/// process holding a pointer capture dies, the platform declines to move the
/// cursor at all.
///
/// ⇒ The cost of not retrying is a check reporting SKIP — *"unable to
/// begin"* — for a reason that has nothing to do with the application. This
/// harness exists to turn "told you nothing" into something, and a suite whose
/// members randomly do not run is that same failure wearing another colour.
///
/// ★ **One retry, not a loop.** A genuinely bad coordinate — off every
/// monitor, which is an arithmetic error in the calling check — must still
/// fail, and fail quickly, with the message that names it. A loop would turn a
/// check’s own mistake into a slow timeout.
pub fn set_cursor_position(x: i32, y: i32) -> Result<()> {
    // SAFETY: no pointers involved; fails by returning false.
    if unsafe { SetCursorPos(x, y) } != 0 {
        return Ok(());
    }
    // ★ One retry, 120 ms later. See the doc comment.
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
///
/// `mouse_event` rather than `SendInput`: for a plain button at the current
/// position they are equivalent, and `mouse_event`'s signature has no
/// variable-length array to get wrong. The one thing `SendInput` would buy —
/// atomic multi-event batches — is not wanted here, because a real user's
/// click is not atomic either and the point of this harness is to exercise the
/// real path.
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
///
/// ★★★ Added 2026-08-28, and its absence until then is worth recording: this
/// harness had driven 92 checks and **had never once opened a context menu**.
/// pdfcer has had canvas right-click menus since Phase 1 — `canvas.object` and
/// `canvas.empty` — and every assertion about them is a unit test over
/// `MenuHost::would_open`, which is a question about the manifest rather than
/// about the running program.
///
/// ⇒ A whole gesture class was outside R1's reach for the life of the project,
/// and nothing said so, because a missing capability in a harness leaves no
/// failing test behind. It surfaced only when a fourth menu was added and
/// somebody went looking for the driver to exercise it with.
pub fn mouse_button_secondary(down: bool) {
    let flags = if down {
        MOUSEEVENTF_RIGHTDOWN
    } else {
        MOUSEEVENTF_RIGHTUP
    };
    // SAFETY: no pointers; the extra-info argument is unused (0).
    unsafe { mouse_event(flags, 0, 0, 0, 0) };
}

/// Press and release a virtual key.
///
/// Goes to the **foreground window**, whichever that is — which is why callers
/// raise the target first and why the input driver refuses to type when the
/// foreground window is not the one under test. A keystroke sent to the wrong
/// window is not a failed keystroke; it is a keystroke into the operator's
/// editor.
/// Turn the mouse wheel at the pointer's current position.
///
/// `notches` is in wheel detents — positive scrolls **up** (away from the
/// operator), negative down, which is the sign convention `WM_MOUSEWHEEL`
/// itself uses.
///
/// # ★ Why the harness needs this at all
///
/// Because a dock panel is a few hundred points tall and a real document's
/// content is not. A check that can only click what is on screen at launch can
/// only ever verify the top of every list — and it reports everything below the
/// fold as *"the control is drawn and inert"*, which is a **confident, wrong
/// defect report about a control that works**. That failure was produced three
/// times on 2026-08-19 before this existed.
///
/// `mouse_event` rather than `SendInput` for the same reason the button press
/// uses it: no variable-length array to get the size of, and at the current
/// pointer position the two are equivalent. `WHEEL_DELTA` is 120, the constant
/// §the API defines one detent as.
pub fn wheel(notches: i32) {
    const WHEEL_DELTA: i32 = 120;
    unsafe { mouse_event(MOUSEEVENTF_WHEEL, 0, 0, notches * WHEEL_DELTA, 0) };
}

pub fn key_stroke(vk: u16) {
    // SAFETY: no pointers; the scan-code argument is 0, which tells Windows to
    // derive it from the virtual key.
    unsafe {
        keybd_event(vk as u8, 0, 0, 0);
        keybd_event(vk as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}

/// Whether `w` is the window that will receive keystrokes right now.
///
/// # Why this is asked rather than assumed
///
/// [`raise_window`] is **best-effort by Windows' own rules**: a process
/// without foreground rights is refused, and the refusal is silent — it
/// returns a boolean nobody was obliged to read. So "we called
/// `SetForegroundWindow`" is not "the window is in front", and the gap between
/// those two is where a keystroke lands in the operator's editor.
///
/// `key_stroke`'s own doc comment has said since it was written that "the
/// input driver refuses to type when the foreground window is not the one
/// under test". That was a description of an intent, not of the code: the
/// driver checked only that a target *existed*. This is the function that
/// makes the sentence true.
/// **Which top-level window owns this screen point.**
///
/// ★★ Added 2026-08-20, after an afternoon of confident, specific and entirely
/// wrong defect reports.
///
/// `SetForegroundWindow` succeeding says the target has focus. It says
/// **nothing about what is drawn over it**, and an always-on-top window — the
/// Windows on-screen keyboard is the one that bit us — sits above a focused
/// window and swallows every click aimed at the region it covers.
///
/// The failure that produces is the worst shape available: `markup_rectangle`
/// and `insert_image` both reported the ribbon as unresponsive, intermittently,
/// over a build in which it works. The oracle that settled it was a screenshot
/// (`D:/dev/rag/egui/` — *a layout or reachability defect has exactly one
/// oracle*), which showed `osk.exe` lying across the ribbon's tab row.
///
/// So a click now asks who owns the point first, and refuses rather than
/// missing. `WindowFromPoint` returns the deepest child; the ancestor walk is
/// what makes the answer comparable to a top-level handle.
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

/// **Move the window to a known position**, without resizing it.
///
/// A harness that lets Windows choose gets a *cascade*: every launch steps down
/// and right from the last, so a long session marches its windows towards the
/// edge of the desktop and eventually off it. A known position also makes a
/// failure reproducible, which a cascading one is not.
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
///
/// ★★★ Added 2026-08-28 for `OPERATOR_REQUESTS.md` **O55**, whose whole
/// subject is *"if the canvas window is resized the pdf should resize to
/// match"*. Until then this harness could move a window and could not resize
/// one — so **no check had ever exercised a resize**, and a fit's behaviour
/// across one was outside R1's reach entirely.
///
/// ⇒ The second gesture class found missing today, after the secondary click.
/// The pattern is worth naming: a harness grows a primitive when a feature
/// needs it, so the primitives it has are a map of the features somebody
/// already had to prove — and the ones it lacks are where nothing has been
/// proved at all.
///
/// `SWP_NOMOVE | SWP_NOZORDER` keeps the position and the stacking exactly as
/// they were; the size is in **physical pixels**, which is what
/// `SetWindowPos` takes and what `describe_window`'s client rect reports.
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

pub fn is_foreground(w: WindowHandle) -> bool {
    // SAFETY: no pointers, no ownership; returns a handle or null.
    unsafe { GetForegroundWindow() == w.hwnd() }
}

/// **Whatever window currently has the foreground**, whoever owns it.
///
/// ★ Distinct from [`is_foreground`] in the way that matters: that answers
/// *"is THIS window in front"*, and the question a harness needs once the
/// application has several windows is *"which of them is"*. A dialog that just
/// opened has the foreground and was never clicked, so no record of a click can
/// answer it.
#[must_use]
pub fn foreground_window() -> Option<WindowHandle> {
    // SAFETY: no pointers, no ownership; returns a handle or null.
    let hwnd = unsafe { GetForegroundWindow() };
    WindowHandle::from_raw(hwnd)
}

/// **Name whatever currently holds the foreground**, for a refusal message.
///
/// # ★★★ Why a refused raise must name the window that refused it
///
/// `SetForegroundWindow` fails for exactly one reported reason — *"this
/// process does not have foreground rights"* — and that sentence is true of
/// two completely different situations which need opposite responses:
///
/// | what is really happening | what to do |
/// |---|---|
/// | the harness is a background process and Windows' foreground lock is doing its job | nothing; retry, or run the check when the desktop is free |
/// | **another window is holding the foreground and will not yield it** | dismiss that window — no amount of retrying will help |
///
/// On 2026-08-25 the second one cost forty minutes. Nine driven checks
/// reported SKIP with the foreground-rights sentence; three raise strategies
/// were probed against a running build; the harness itself came under
/// suspicion. The actual cause was a stray **`OpenWith.exe` "Open With"
/// dialog** sitting on the desktop from some earlier action, holding the
/// foreground the way a system modal does and yielding it to nothing. One
/// `taskkill` fixed all nine.
///
/// The diagnosis was a `GetForegroundWindow` followed by `GetClassNameW` —
/// two calls the harness could have made itself, at the moment of failure,
/// when it already knew something was wrong. **A check that reports a refusal
/// without naming the refuser has withheld the only fact that distinguishes
/// "wait" from "act".** That is the same shape as the `osk.exe` finding
/// recorded against [`window_at`]: an unrelated always-on-top window silently
/// eating the harness's input, diagnosed only by looking at what was actually
/// on the screen. This function makes the harness look, so a human does not
/// have to.
///
/// Returns something printable in every case, including no foreground window
/// at all — which is itself a distinct and diagnosable state (a locked
/// workstation, or a switch to the secure desktop).
#[must_use]
pub fn describe_foreground() -> String {
    let Some(w) = foreground_window() else {
        return "nothing at all (no foreground window), which usually means the workstation is locked or the secure desktop is up"
            .to_string();
    };
    describe_window(w)
}

/// Name **any** window, the way [`describe_foreground`] names the front one.
///
/// # ★★ Added 2026-08-27, because the same lesson had been applied to one
/// guard and not to its neighbour
///
/// [`describe_foreground`]'s own doc records what it cost to learn: *"a check
/// that reports a refusal without naming the refuser has withheld the only fact
/// that distinguishes 'wait' from 'act'."* That was written after a stray
/// `OpenWith.exe` dialog held the foreground and made nine checks SKIP, and the
/// diagnosis — a `GetForegroundWindow` plus a `GetClassNameW` — was two calls
/// the harness could have made itself.
///
/// The **cover** guard (`Driver::confirm_uncovered`) refuses for the same class
/// of reason and, until today, named nothing: *"the point (1627, 895) belongs to
/// another window"*, and then a paragraph guessing that it might be `osk.exe`.
/// On 2026-08-27 it was not `osk.exe` — the on-screen keyboard was not running —
/// and the SKIP was therefore unactionable in exactly the way the foreground
/// message had been before it was fixed.
///
/// ⇒ **When a guard learns to name its subject, check every other guard that
/// refuses for the same kind of reason.** A lesson applied at one call site and
/// not at its sibling is a lesson half-learned, and the sibling is where it will
/// be paid for again.
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
///
/// `modifiers` are virtual-key codes (`VK_CONTROL` 0x11, `VK_SHIFT` 0x10,
/// `VK_MENU` 0x12) held down for the duration of the stroke.
///
/// # ★ Every modifier is released, on every path, in reverse order
///
/// A modifier left down is not a failed keystroke — it is a **stuck key on
/// the operator's real keyboard**, applied to whatever they do next, until
/// they happen to press and release that key themselves. Ctrl left down turns
/// their next `s` into a save and their next `w` into a close-window. This
/// harness already refuses to type when the target window is not in front for
/// the same class of reason; leaking a modifier is the same hazard with a
/// longer tail, because the wrong window is at least visible and a stuck Ctrl
/// is not.
///
/// So the releases are unconditional and there is no early return between the
/// press and the release. Reverse order because that is what a human hand
/// does, and because a shell watching for a chord may key on the release
/// sequence.
/// # ★ The pauses are load-bearing, and their absence is why chords silently
/// did nothing
///
/// Added 2026-08-17. Without them this function posted four or more
/// `keybd_event` calls in the same microsecond, and the result was that **no
/// chord this harness sent ever reached the application** — while a plain
/// [`key_stroke`] of the very same key worked. `HANDOFF.md` §8 recorded that
/// asymmetry as *"synthetic keyboard input does not reach the target window"*
/// and blocked several checks on it; the truth is narrower and is about
/// ordering, not delivery.
///
/// `keybd_event` posts into the system input queue **asynchronously**. The
/// target reads that queue on its own schedule — for an `egui`/`winit`
/// application, once per frame. A modifier-down and a key-down that arrive in
/// the same batch give the application no frame in which the modifier is held
/// and the key is not yet pressed, so its notion of "current modifiers" at the
/// moment it processes the key can still be empty. The key is then delivered
/// **unmodified**: `Ctrl+2` arrives as a bare `2`, which this shell's keymap
/// binds to nothing, so nothing is traced and the check reports silence.
///
/// That is also exactly why a plain keystroke was unaffected and why the
/// earlier investigation (which confirmed foreground rights, then tried a
/// prior click) found nothing: neither had anything to do with it.
///
/// 12 ms is one frame at 60 Hz plus margin — enough that the application sees
/// at least one frame with the modifier down and the key not yet pressed, and
/// small enough that a chord still completes in well under a tenth of a
/// second.
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
        // ★ The releases stay unconditional and un-gated by any early return,
        // for the reason above: a leaked modifier is a stuck key on the
        // operator's real keyboard. The sleeps are between the posts, never
        // around the loop, so no path can skip a release.
        for m in modifiers.iter().rev() {
            keybd_event(*m as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }
}

/// Grab a desktop region as BGRA pixels, top row first.
///
/// Returns `region.w * region.h * 4` bytes. See the module docs for why this
/// reads the desktop rather than the window.
///
/// The GDI object dance is the standard one and every handle is released on
/// every path, including the error paths: a harness that leaks a DC per run
/// will exhaust the desktop heap during a long CI session, and the failure
/// looks like an unrelated rendering bug in whatever runs next.
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
///
/// # ★ Why this takes a closure instead of exposing down/up
///
/// Because a leaked modifier is a stuck key on the operator's real keyboard,
/// and this harness runs on the operator's real desktop. `key_stroke_with`'s own
/// comment makes the point about early returns; a pair of public `modifier_down`
/// / `modifier_up` functions would move the obligation to every caller, and the
/// first caller to add a `?` between them would leave Shift held down system
/// wide until the operator noticed their typing had gone into capitals.
///
/// A closure makes the release structural. `body` may panic and the modifiers
/// still come up, because the loop below is after the call in a function that
/// does not unwind past it — and every caller in this harness returns `Result`
/// rather than panicking anyway.
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
// ★★ WHY THE HARNESS HAS TO READ THE CLIPBOARD ITSELF
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
// ★ A NOTE ON THE DEPENDENCY POSTURE. This adds two `windows-sys` FEATURES,
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
///
/// Deliberately not `CF_TEXT`: that is code-page-dependent, and a check that
/// compared a round-tripped ANSI string would fail or pass depending on the
/// machine's locale rather than on the application's behaviour.
const CF_UNICODETEXT: u32 = 13;

/// How many times to retry opening the clipboard, and how long to wait between.
///
/// ★ The clipboard is a **global, singly-owned** resource: `OpenClipboard`
/// fails outright while any other process holds it, and on a live desktop
/// something always might — a clipboard manager, an editor polling for
/// changes, the shell itself. A single attempt would make this check flake in a
/// way indistinguishable from the defect it exists to catch, which is the worst
/// possible failure mode for a harness.
const OPEN_ATTEMPTS: u32 = 20;
const OPEN_RETRY_MS: u64 = 25;

/// Take ownership of the clipboard, run `body`, and always release it.
///
/// The release is unconditional — including on the `body` panicking, which is
/// why `body`'s result is captured rather than returned directly through the
/// `?`. A harness that left the clipboard open would wedge every other program
/// on the operator's desktop, and it is his desktop.
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
///
/// `None` covers three genuinely different situations and the caller must treat
/// them as one, because Windows does not distinguish them either: the clipboard
/// is empty, it holds something that is not text (a bitmap, a file list), or
/// another process would not let go of it. All three mean *"the operator would
/// not get text if they pasted"*, which is the question a check is asking.
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
///
/// ★★ **A check that asserts on the clipboard MUST call this first**, and the
/// reason is the whole shape of defect O18. The failing build left a marker
/// sentence on the clipboard; a check that copied, read, and found that marker
/// would fail correctly — but a check that copied *nothing at all* and found a
/// marker left by an earlier run would fail identically, and one that found
/// yesterday's correct text would **pass while the application did nothing**.
///
/// Clearing first is what makes the read afterwards a statement about this run.
pub fn clear_clipboard() -> bool {
    with_clipboard(|| {
        // SAFETY: the clipboard is open and owned by this task.
        unsafe { EmptyClipboard() != 0 }
    })
    .unwrap_or(false)
}

/// ★★★ **Every format on the clipboard, IN PLACEMENT ORDER, with its name.**
///
/// The oracle `checks::copy_as_vector` needs and the one
/// [`clipboard_text`] cannot supply. `OPERATOR_REQUESTS.md` O120's whole design
/// is an *order*: a pasting application "typically retrieves … the first format
/// it recognizes", so what makes a Word paste an editable graphic rather than a
/// flat picture is not *whether* the SVG is there — it is whether the SVG is
/// there **first**. A check that asked "is `image/svg+xml` available?" would
/// pass on a build that placed it last, which is the build that fails in Word.
///
/// `EnumClipboardFormats` answers exactly that question: it walks the formats
/// *in the order they were placed*, which is also the priority order a reader
/// sees. Anything Windows synthesised (it makes `CF_DIB` and `CF_BITMAP` out of
/// a `CF_DIBV5`) comes after the ones that were really placed, so a caller can
/// assert on a prefix and ignore the tail.
///
/// Each entry is `(id, name)`. The name comes from
/// `GetClipboardFormatNameW` for a registered format and is empty for a
/// predefined `CF_*` one — Windows has no name for those — so a caller matches
/// predefined formats by id and registered ones by name, which is exactly how
/// they are placed.
///
/// `None` means the clipboard could not be opened at all, which is a different
/// fact from "the clipboard is empty" and must not be collapsed into it: the
/// first is a flake, the second is a defect.
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
