//! # `native_window::win32` — the Windows half
//!
//! See the crate docs for why this exists at all. This file is the Windows
//! side: hand-written declarations, two public entry points, no state.
//!
//! ## The second entry point, and why the toolkit cannot answer it
//!
//! [`cursor_position`] exists for the same reason [`own_window`] does — the
//! toolkit will not say something the operating system knows. **During an OLE
//! drag the application receives no mouse-move messages at all**, so
//! `egui`'s `pointer.latest_pos()` is stale from before the drag began, and
//! `winit` 0.30.13 discards the drop point it is handed:
//!
//! ```text
//! // winit/src/platform_impl/windows/drop_handler.rs
//! pub unsafe extern "system" fn Drop(
//!     this: *mut IDropTarget,
//!     pDataObj: *const IDataObject,
//!     _grfKeyState: u32,
//!     _pt: *const POINTL,        // ← the drop point, discarded
//!     _pdwEffect: *mut u32,
//! ) -> HRESULT
//! ```
//!
//! `DragOver` discards it too, so there is no route to it through the toolkit
//! at all — not for the drop, and not for the hover on the way in. Without
//! this, a dropped file is an event with no position, and *"drop it onto the
//! thumbnails"* cannot be distinguished from *"drop it anywhere"*.
//!
//! ## ★ Why the declarations are hand-written rather than a crate
//!
//! Because a handful of symbols is not worth a dependency, and
//! `tools/ui-verify` sets the precedent in this repository for that reason: it
//! declares its own `user32` externs rather than pulling in `windows-sys`,
//! whose feature surface is larger than this whole module.
//!
//! Each declaration below is copied from the SDK signature and each call site
//! carries a `SAFETY` note. Nothing here allocates, owns a handle, or frees
//! anything: every value is an integer the operating system gave us.

use std::ffi::c_void;

/// An opaque window handle.
type Hwnd = *mut c_void;

/// `GWLP_HWNDPARENT` — the window-long index that holds a window's **owner**.
///
/// ★ The name is a trap and the SDK documents it as one: this index sets the
/// window's *owner*, not its *parent*. A parent would make the dialog a child
/// control clipped inside the application's client area — the in-viewport
/// behaviour this whole module exists to leave behind. An owner is a peer
/// top-level window that stays above the one it belongs to.
const GWLP_HWNDPARENT: i32 = -8;

/// The `POINT` the SDK fills in, laid out exactly as `windef.h` declares it.
///
/// ★ Two `i32`s in declaration order and nothing else. `#[repr(C)]` is not
/// decoration here: the operating system writes through this pointer, and a
/// Rust-ordered struct would be a silent coordinate swap on some future
/// compiler rather than a compile error.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}

unsafe extern "system" {
    /// The cursor's position in **physical desktop pixels**.
    ///
    /// Fails — and returns zero — when the calling thread's desktop is not the
    /// input desktop, which is the locked-workstation and screensaver case.
    fn GetCursorPos(point: *mut Point) -> i32;
    /// Find a top-level window by class and/or title.
    ///
    /// Passing null for `parent` and `child_after` searches top-level windows
    /// from the beginning; passing null for `class` matches any class.
    fn FindWindowExW(parent: Hwnd, child_after: Hwnd, class: *const u16, title: *const u16)
    -> Hwnd;
    /// The process and thread a window belongs to.
    fn GetWindowThreadProcessId(hwnd: Hwnd, pid: *mut u32) -> u32;
    /// This process's id.
    fn GetCurrentProcessId() -> u32;
    /// Read or write one of a window's "long" values — here, its owner.
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    /// The same, reading.
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
}

/// **Make the window titled `title` owned by `owner`**, and report whether the
/// relationship now holds.
///
/// Idempotent: a window that is already owned by `owner` is left alone and
/// answers `true`. That matters because this is called from a per-frame path —
/// a dialog draws every frame and there is no "it just opened" event a caller
/// could rely on that is cheaper than simply checking.
///
/// # ★★ The three ways this answers `false`, and none of them is an error
///
/// 1. **The window does not exist yet.** A viewport is created by the toolkit
///    *during* the frame, so the first call after a dialog opens can run before
///    the platform has a window to find. The caller tries again next frame.
/// 2. **The title matched a window belonging to another process.** Refused
///    rather than acted on — see the module header. `SetWindowLongPtrW` on
///    somebody else's window is a real thing to do to somebody else's program.
/// 3. **The platform refused the write.** Reported rather than assumed.
///
/// A caller may treat all three as *"not yet"*, which is what makes the
/// per-frame retry correct rather than wasteful.
#[must_use]
pub fn own_window(owner: isize, title: &str) -> bool {
    if owner == 0 {
        return false;
    }
    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: both pointer arguments are null (search all top-level windows,
    // any class); `wide` is a NUL-terminated UTF-16 buffer that outlives the
    // call. The return is a handle or null and carries no ownership.
    let hwnd = unsafe {
        FindWindowExW(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null(),
            wide.as_ptr(),
        )
    };
    if hwnd.is_null() {
        return false;
    }
    // ★ It must be OURS. See the module header.
    let mut pid: u32 = 0;
    // SAFETY: `hwnd` came from `FindWindowExW`; the out-parameter points at a
    // live stack local. The call tolerates a stale handle by returning 0.
    unsafe { GetWindowThreadProcessId(hwnd, &raw mut pid) };
    // SAFETY: no arguments, no pointers.
    if pid == 0 || pid != unsafe { GetCurrentProcessId() } {
        return false;
    }
    // SAFETY: `hwnd` is a window of this process, confirmed above.
    let current = unsafe { GetWindowLongPtrW(hwnd, GWLP_HWNDPARENT) };
    if current == owner {
        return true;
    }
    // SAFETY: as above. Writing `GWLP_HWNDPARENT` sets the owner; the previous
    // value is returned and deliberately discarded — this is the only writer.
    unsafe { SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, owner) };
    // SAFETY: as above. Read back rather than trusting the write: the SDK
    // returns the PREVIOUS value on success and 0 on failure, and 0 is also a
    // perfectly valid previous value, so the return alone cannot distinguish
    // them.
    unsafe { GetWindowLongPtrW(hwnd, GWLP_HWNDPARENT) == owner }
}

/// **Where the pointer is, in physical desktop pixels**, or `None` if the
/// operating system declined to say.
///
/// # ★★ Why a caller wants this rather than the toolkit's pointer position
///
/// Only one caller should: an application's file-drag path, while a file drag
/// is in flight. In every other situation `egui`'s pointer position is better
/// in three ways —
/// it is in the toolkit's own coordinate space, it is the position as of the
/// frame being drawn rather than as of *now*, and it does not cross a syscall.
///
/// A file drag is the exception because there is no toolkit position to have:
/// see the module header. The conversion into `egui` coordinates lives with
/// the caller, because it needs that frame's `pixels_per_point` and the
/// window's own rectangle, neither of which this crate knows or should.
///
/// # ★ Why `None` rather than `(0, 0)`
///
/// Because `(0, 0)` is a real place — the top-left corner of the primary
/// monitor — and a caller that resolved a drop target against it would put a
/// dropped file wherever the top-left corner happens to point. The failure is
/// rare (a locked workstation cannot receive a drop) but the cost of
/// mistaking it for a position is a document edited at a coordinate nobody
/// chose.
#[must_use]
pub fn cursor_position() -> Option<(i32, i32)> {
    let mut point = Point::default();
    // SAFETY: the out-parameter points at a live stack local of exactly the
    // layout the SDK declares. The call writes two `i32`s and returns a
    // BOOL; it allocates nothing and takes no ownership.
    let ok = unsafe { GetCursorPos(&raw mut point) };
    (ok != 0).then_some((point.x, point.y))
}
