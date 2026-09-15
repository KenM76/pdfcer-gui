//! # `native-window` — the two or three things the window manager will only
//! # tell the operating system
//!
//! ## What this is for
//!
//! Almost nothing in this shell needs to know what a window manager is.
//! `eframe` opens the window, `egui` draws into it, and a viewport is a
//! platform-neutral idea. This module exists for the cases where that
//! abstraction has a **hole in it that costs the operator something**, and
//! where the toolkit exposes no way to say what needs saying.
//!
//! There are three, and the rest of this header is about the first: **a dialog
//! must be OWNED by the window it belongs to.** [`cursor_position`] and
//! [`clipboard`] carry their own arguments where they are declared.
//!
//! ## ★★★ Why ownership, and why it is not cosmetic
//!
//! `ui-conventions/dialogs.md` G3 states the rule, and its absence costs two
//! different things:
//!
//! 1. **The dialog can fall behind the application window**, which is the
//!    classic Windows bug — a program that appears to have frozen because the
//!    thing waiting for an answer is behind the thing it is blocking.
//! 2. **The dialog loses the keyboard a third of a second after it opens.**
//!    Measured, with both windows reporting their own focus:
//!
//!    ```text
//!    dialog-focus  focused=Some(true)     the note window is given the keyboard
//!    root-focus    focused=Some(false)
//!    …17 idle passes: no resize, no reposition, no input, nothing asked for…
//!    root-focus    focused=Some(true)     and Windows hands it BACK
//!    dialog-focus  focused=Some(false)
//!    ```
//!
//!    The operator's version: *drag out a note box, type without clicking the
//!    field first, and the words go nowhere.*
//!
//! ★ **Asking again does not work.** Half a second of
//! `ViewportCommand::Focus`, one per pass, straight through the moment of the
//! loss, and the root still takes the foreground back. Windows refuses the
//! foreground to a process that does not already hold it, silently, which is
//! the same rule `tools/ui-verify` documents at length about
//! `SetForegroundWindow`.
//!
//! **Ownership is not a request.** An owned window is *by definition* above its
//! owner in z-order, and activation follows the relationship rather than a call
//! that can be declined. It is the mechanism every native dialog on this
//! machine already uses, which is why none of them has this problem.
//!
//! ## ★ Why `eframe` cannot express it, and why this is not a workaround
//!
//! Not one of `ViewportBuilder`'s options in `egui 0.35` is an owner, and
//! `egui-winit` never passes down the parent relationship egui itself
//! tracks in `viewport_parents`. There is also no way to get the child window's
//! handle back out — `eframe::Frame` hands out the ROOT window's, once.
//!
//! So the handle is found the same way `tools/ui-verify` finds it: by asking
//! the operating system for **this process's** top-level windows and matching
//! on the title. That is a real lookup rather than a hack — the title is what
//! the application asked the platform to call the window, so it is the one name
//! both sides already agree on.
//!
//! ★★ **The process check is not optional.** `FindWindowExW` searches every
//! window on the desktop, so a title match alone could name another
//! application's window — and `SetWindowLongPtrW` on somebody else's window is
//! a real thing to do to somebody else's program. Every function here confirms
//! the window belongs to this process before touching it.
//!
//! ## What this module refuses to become
//!
//! A platform layer. There is no trait, no abstraction, and no second backend
//! beyond a no-op for every non-Windows target. When the toolkit grows an owner
//! option, this module is **deleted** rather than ported — which is why it is
//! one file with one public function and no state.

#![cfg_attr(not(windows), allow(unused))]

#[cfg(windows)]
mod win32;

/// **A picture other programs can paste** — `OPERATOR_REQUESTS.md` O71.
///
/// Its own file rather than a third function in [`win32`], because it is the
/// first thing in this crate that is not about a *window*: imported symbols of
/// its own, two clipboard formats and a `BITMAPINFOHEADER`, with an argument
/// about why the two payloads must be written in one transaction. Everything
/// the crate header says about hand-written declarations applies to it.
#[cfg(windows)]
pub mod clipboard;

/// Off Windows, the clipboard writer answers `false` — nothing was written.
///
/// A stub module rather than a `cfg` on every call site, matching how
/// [`own_window`] and [`cursor_position`] are handled: a caller writes one line
/// and reads one `bool` on every platform.
#[cfg(not(windows))]
pub mod clipboard {
    /// No-op off Windows. See the crate header.
    #[must_use]
    pub fn set_image_and_text(_rgba: &[u8], _width: u32, _height: u32, _text: &str) -> bool {
        false
    }
}

#[cfg(windows)]
pub use win32::{cursor_position, own_window};

/// **Where the pointer is, in physical desktop pixels.** `None` off Windows.
///
/// See [`win32::cursor_position`] for why this is asked of the operating
/// system rather than of the toolkit. Answering `None` rather than a position
/// is what makes a caller on another platform fall back to its
/// position-blind behaviour instead of acting on a fabricated point.
#[cfg(not(windows))]
#[must_use]
pub fn cursor_position() -> Option<(i32, i32)> {
    None
}

/// **Make the window titled `title` owned by `owner`.** No-op off Windows.
///
/// See the module header. `owner` is a raw window handle, as an application
/// obtains it from `raw_window_handle`.
#[cfg(not(windows))]
pub fn own_window(_owner: isize, _title: &str) -> bool {
    // Every other platform: dialogs are already handled correctly by their
    // window managers, or the port has not happened. Answering `false` rather
    // than `true` keeps a caller that logs the outcome honest.
    false
}
