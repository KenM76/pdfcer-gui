//! The paste chord, seen by the window before the toolkit can drop it.
//!
//! `egui-winit` turns `Ctrl+V` into `Event::Paste` only when the clipboard
//! holds text, and otherwise emits nothing at all, so a picture copied in
//! another program could never be pasted with the keyboard. [`install`]
//! replaces the main window's procedure with one that notes each paste chord
//! and passes every message on unchanged; [`take`] hands the note to the
//! frame, once.
//!
//! Contract: one window per process is hooked (the first [`install`] wins and
//! later calls answer `false`); the hook never consumes a message, so the
//! toolkit sees exactly what it saw before.

use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, AtomicU8, Ordering};

type Hwnd = *mut c_void;
type WndProc = unsafe extern "system" fn(Hwnd, u32, usize, isize) -> isize;

const GWLP_WNDPROC: i32 = -4;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_KILLFOCUS: u32 = 0x0008;
const VK_SHIFT: usize = 0x10;
const VK_CONTROL: usize = 0x11;
const VK_INSERT: usize = 0x2D;
const VK_V: usize = 0x56;

/// The window procedure the hook replaced; 0 until [`install`] succeeds.
static PREVIOUS: AtomicIsize = AtomicIsize::new(0);
/// The pending chord: 0 none, 1 `Ctrl+V`, 2 `Ctrl+Shift+V`.
static PENDING: AtomicU8 = AtomicU8::new(0);
/// Ctrl (bit 0) and Shift (bit 1) as the key messages this window saw.
static HELD: AtomicU8 = AtomicU8::new(0);

unsafe extern "system" {
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    fn CallWindowProcW(previous: WndProc, hwnd: Hwnd, msg: u32, w: usize, l: isize) -> isize;
    fn GetKeyState(key: i32) -> i16;
    fn DefWindowProcW(hwnd: Hwnd, msg: u32, w: usize, l: isize) -> isize;
}

/// A paste chord the window saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteChord {
    /// `Ctrl+V` or `Shift+Insert`.
    Paste,
    /// `Ctrl+Shift+V`.
    PasteShifted,
}

/// Hook `hwnd`'s window procedure; `false` if a window is already hooked or
/// the platform refused.
#[must_use]
pub fn install(hwnd: isize) -> bool {
    if hwnd == 0 || PREVIOUS.load(Ordering::Acquire) != 0 {
        return false;
    }
    let hook: WndProc = procedure;
    // SAFETY: `hwnd` is the caller's own top-level window; the new procedure
    // forwards every message to the one returned here, stored before any
    // message can reach the hook because both run on the window's thread.
    let previous = unsafe { SetWindowLongPtrW(hwnd as Hwnd, GWLP_WNDPROC, hook as usize as isize) };
    if previous == 0 {
        return false;
    }
    PREVIOUS.store(previous, Ordering::Release);
    true
}

/// The chord seen since the last call, if any.
#[must_use]
pub fn take() -> Option<PasteChord> {
    match PENDING.swap(0, Ordering::AcqRel) {
        1 => Some(PasteChord::Paste),
        2 => Some(PasteChord::PasteShifted),
        _ => None,
    }
}

/// Whether `key` is down, by the thread's key state or by the key messages
/// this window saw; a posted key message moves only the second.
fn down(key: usize, bit: u8) -> bool {
    // SAFETY: no pointers; a virtual-key code in range.
    let state = unsafe { GetKeyState(key as i32) };
    state < 0 || HELD.load(Ordering::Acquire) & bit != 0
}

fn note(msg: u32, key: usize) {
    let bit = match key {
        VK_CONTROL => 1,
        VK_SHIFT => 2,
        _ => 0,
    };
    match msg {
        WM_KEYDOWN if bit != 0 => {
            HELD.fetch_or(bit, Ordering::AcqRel);
        }
        WM_KEYUP if bit != 0 => {
            HELD.fetch_and(!bit, Ordering::AcqRel);
        }
        WM_KILLFOCUS => HELD.store(0, Ordering::Release),
        WM_KEYDOWN => {
            let (ctrl, shift) = (down(VK_CONTROL, 1), down(VK_SHIFT, 2));
            let chord = match key {
                VK_V if ctrl && shift => 2,
                VK_V if ctrl => 1,
                VK_INSERT if shift && !ctrl => 1,
                _ => 0,
            };
            if chord != 0 {
                PENDING.store(chord, Ordering::Release);
            }
        }
        _ => {}
    }
}

unsafe extern "system" fn procedure(hwnd: Hwnd, msg: u32, w: usize, l: isize) -> isize {
    note(msg, w);
    let previous = PREVIOUS.load(Ordering::Acquire);
    if previous == 0 {
        // SAFETY: the arguments are the ones this procedure was called with.
        return unsafe { DefWindowProcW(hwnd, msg, w, l) };
    }
    // SAFETY: `previous` is the procedure `SetWindowLongPtrW` returned in
    // `install`, stored before this hook could run.
    unsafe {
        let previous: WndProc = std::mem::transmute::<isize, WndProc>(previous);
        CallWindowProcW(previous, hwnd, msg, w, l)
    }
}
