//! Every `unsafe` line in the crate, behind one safe API.
//!
//! Design and rationale: `docs/modules/ui-verify/sys/mod.md`.

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::*;

#[cfg(not(windows))]
mod unsupported;
#[cfg(not(windows))]
pub use unsupported::*;

/// Virtual-key codes the harness needs, named so that call sites read as
/// keystrokes rather than as magic numbers.
pub mod vk {
    /// `Delete`. The key D1 is about.
    pub const DELETE: u16 = 0x2E;
    /// `Escape` — closes a dialog, cancels a tool.
    pub const ESCAPE: u16 = 0x1B;
    /// `Backspace`. Bound to the same action as Delete in this application,
    /// and suppressed by the same guard, so a check that presses one should
    /// usually be able to press the other.
    pub const BACKSPACE: u16 = 0x08;
    /// `Enter` — steps to the next Find hit, commits the page box.
    pub const ENTER: u16 = 0x0D;
    /// `Tab` — O204's key: the one that walked into the ribbon.
    ///
    /// Pressed with [`LSHIFT`] for the backward direction, never with
    /// [`SHIFT`]: the shell decides the direction from winit's modifier
    /// state, which is derived from key EVENTS, and `VK_SHIFT` is a key no
    /// real keyboard ever sends.
    pub const TAB: u16 = 0x09;

    /// `Ctrl`, as a **modifier** for [`super::key_stroke_with`].
    pub const CONTROL: u16 = 0x11;
    /// `Shift`, as a modifier. `Ctrl+Shift+…` is two entries in the slice.
    pub const SHIFT: u16 = 0x10;
    /// `VK_LSHIFT` — the LEFT shift specifically.
    pub const LSHIFT: u16 = 0xA0;

    /// `F` — the letter, for `Ctrl+F`.
    pub const F: u16 = 0x46;

    /// `H`, for `Ctrl+H` — the read-mode toggle, and the only way back out of
    /// read mode once the chrome it hides has taken the ribbon with it.
    pub const H: u16 = 0x48;
    /// `Alt`, as a modifier. `Alt+Down` is one entry in the slice.
    pub const ALT: u16 = 0x12;
    /// `F4`, for **`Alt+F4`** — the only way this harness can ask the
    /// application to close **gracefully**.
    pub const F4: u16 = 0x73;

    /// `Z`, for `Ctrl+Z` and `Ctrl+Shift+Z` — undo and redo.
    ///
    pub const S: u16 = 0x53;
    pub const Z: u16 = 0x5A;
    /// `VK_NUMPAD5`, for Inkscape's `Ctrl+Alt+keypad 5` (centre on both axes).
    pub const NUMPAD5: u16 = 0x65;
    /// `Y`, for `Ctrl+Y` — redo's other spelling.
    pub const Y: u16 = 0x59;
    /// `E`, for `Ctrl+E` and `Ctrl+Shift+E` — edit text and add text.
    pub const E: u16 = 0x45;
    /// `[` (`VK_OEM_4`), for the bare-character `pages.rotate_left` binding.
    pub const OPEN_BRACKET: u16 = 0xDB;
    /// `Down` (`VK_DOWN`), for the `Alt+Down` page-move binding — the Alt
    /// modifier family, which nothing else here presses.
    pub const ARROW_DOWN: u16 = 0x28;
    /// Up. Added 2026-08-21 with the block-navigation check.
    pub const ARROW_UP: u16 = 0x26;
    /// `VK_RIGHT`. One character to the right, or one more selected when Shift
    /// is held with it.
    pub const ARROW_RIGHT: u16 = 0x27;
    /// `VK_HOME`. Pressed to put the caret at a KNOWN end before a check
    /// counts what a shifted arrow selects.
    pub const HOME: u16 = 0x24;
    /// `VK_END`. Pressed to prove that End reaches the end of the page's LINE
    /// rather than of the show operator the caret happens to sit in.
    pub const END: u16 = 0x23;
    /// `VK_NEXT` -- the key every keyboard prints as **Page Down**.
    pub const PAGE_DOWN: u16 = 0x22;

    /// `D`, `T`, `A`, `I` and `L` — the five letters that spell **DETAIL**.
    pub const D: u16 = 0x44;
    /// See [`D`].
    pub const T: u16 = 0x54;
    /// See [`D`].
    pub const A: u16 = 0x41;
    /// Copy. Added 2026-08-20 with the object clipboard's driven check; see the
    /// note above about why these are added one at a time rather than as a
    /// block.
    pub const C: u16 = 0x43;
    /// `V` — the select tool's chord, and the way a driven check puts an armed
    /// tool down. With a measure or markup tool armed, a click on the page is a
    /// PICK rather than a selection, so any check that needs to select
    /// something it just authored has to disarm first.
    pub const V: u16 = 0x56;
    /// `X`, for `Ctrl+X` — cut.
    ///
    pub const X: u16 = 0x58;
    /// See [`D`].
    pub const I: u16 = 0x49;
    /// See [`D`].
    pub const L: u16 = 0x4C;

    /// `Space` — the bar, pressed as a **character** rather than as a
    /// command.
    pub const SPACE: u16 = 0x20;

    /// `2` — the digit, for the `Ctrl+2` mode chord.
    pub const DIGIT_2: u16 = 0x32;
}
