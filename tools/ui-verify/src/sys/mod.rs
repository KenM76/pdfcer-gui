//! Every `unsafe` line in the crate, behind one safe API.
//!
//! ## Why the platform code is quarantined here
//!
//! Two reasons.
//!
//! **Review surface.** Everything else in this crate is ordinary safe Rust
//! that a reviewer can read quickly. Interleaving `mouse_event` calls with
//! check logic would mean every future check has to be reviewed as if it
//! contained `unsafe`, which is how `unsafe` spreads.
//!
//! **Portability without pretence.** The harness only works on Windows — it
//! drives a Windows window through the Windows input API. But the *workspace*
//! must build elsewhere, or the crate gets dropped from the members list the
//! first time somebody runs `cargo check` on a laptop. So there are two
//! implementations of one API: [`win32`] on Windows, [`unsupported`]
//! everywhere else, which compiles and returns a clear error naming the
//! platform. The harness reports SKIPPED, not a build failure and not a pass.
//!
//! ## The API
//!
//! | Function | Job |
//! |---|---|
//! | [`find_window_for_pid`] | which top-level window belongs to the process we launched |
//! | [`window_frame`] | where its client area is, and at what DPI scale |
//! | [`raise_window`] | bring it to the foreground before driving or capturing it |
//! | [`cursor_position`] / [`set_cursor_position`] | the pointer |
//! | [`mouse_button`] | primary button down/up |
//! | [`key_stroke`] | a virtual key press and release |
//! | [`capture_screen`] | a desktop region as BGRA pixels |
//!
//! Note what is absent: there is no `send_message`, and there never will be.
//! See [`crate::input`] for why posting messages to the window was tried in
//! this project's predecessor and does not work.

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
///
/// Deliberately a tiny closed list rather than a binding of the whole
/// `VIRTUAL_KEY` space: a harness that can press any key is a harness whose
/// scripts stop being readable.
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
    ///
    /// Named `CONTROL` rather than `CTRL` because that is what Windows calls
    /// it (`VK_CONTROL`), and a constant that renames a platform's own
    /// vocabulary makes the next person check twice.
    pub const CONTROL: u16 = 0x11;
    /// `Shift`, as a modifier. `Ctrl+Shift+…` is two entries in the slice.
    pub const SHIFT: u16 = 0x10;
    /// `VK_LSHIFT` — the LEFT shift specifically.
    ///
    /// ★ Not a synonym for [`SHIFT`] where synthesis is concerned. `VK_SHIFT`
    /// is the "either shift" virtual key that Windows reports in keyboard
    /// STATE; a real keyboard never sends it, and a toolkit that derives its
    /// modifier state from key events — winit does — may not recognise it.
    pub const LSHIFT: u16 = 0xA0;

    /// `F` — the letter, for `Ctrl+F`.
    ///
    /// Letters are their ASCII uppercase code point on Windows, which is why
    /// this is `0x46` and not something derived. Only the letters the harness
    /// actually presses are listed: the closed-list rule above applies to
    /// letters more than to anything else, because `pub const A..Z` would be
    /// exactly the "can press any key" the doc comment refuses.
    pub const F: u16 = 0x46;

    /// `H`, for `Ctrl+H` — the read-mode toggle, and the only way back out of
    /// read mode once the chrome it hides has taken the ribbon with it.
    pub const H: u16 = 0x48;
    /// `Alt`, as a modifier. `Alt+Down` is one entry in the slice.
    pub const ALT: u16 = 0x12;
    /// `F4`, for **`Alt+F4`** — the only way this harness can ask the
    /// application to close **gracefully**.
    ///
    ///
    /// A check that killed the window and then asserted the preference survived
    /// would be asserting that the 750 ms debounce had already expired --
    /// which is true on a slow run and false on a fast one, and is not the
    /// property anybody cares about. The property is *"I changed it and closed
    /// the program straight away"*, and only a real `WM_CLOSE` reproduces it.
    pub const F4: u16 = 0x73;

    /// `Z`, for `Ctrl+Z` and `Ctrl+Shift+Z` — undo and redo.
    ///
    pub const S: u16 = 0x53;
    pub const Z: u16 = 0x5A;
    /// `Y`, for `Ctrl+Y` — redo's other spelling.
    pub const Y: u16 = 0x59;
    /// `E`, for `Ctrl+E` and `Ctrl+Shift+E` — edit text and add text.
    pub const E: u16 = 0x45;
    /// `[` (`VK_OEM_4`), for the bare-character `pages.rotate_left` binding.
    ///
    /// A bare character is the class that has to YIELD to typing, so it is the
    /// one worth driving: a build where `[` fires while a caret is in flight
    /// rotates the drawing instead of inserting a bracket.
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
    ///
    ///
    /// Pressed rather than reached through the ribbon because `Action::NextPage`
    /// is what the operator's own gesture raises, and because the page-number
    /// box would make the run depend on a text field's focus rules. Note the
    /// Windows name is `VK_NEXT`, not `VK_PAGEDOWN`: the platform's own
    /// vocabulary is kept, per the note on `CONTROL` above, and the doc line
    /// is what tells a reader which key it is.
    pub const PAGE_DOWN: u16 = 0x22;

    /// `D`, `T`, `A`, `I` and `L` — the five letters that spell **DETAIL**.
    ///
    /// ★ The closed-list rule again, and this is the first entry that exists to
    /// **type a word** rather than to press a chord.
    /// `checks::dimension_groups` names a new dimension group, and the name is
    /// the one thing in that window a check must supply — the Add button is
    /// greyed with an empty field, deliberately, so a check that cannot type
    /// cannot reach the verb at all.
    ///
    /// "Detail" is chosen rather than a nonsense string because the check's
    /// failure text quotes it, and an operator reading *"no group called
    /// Detail appeared in the list"* is being told something about a drawing
    /// they recognise. Added 2026-08-18.
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
    ///
    ///
    /// A space rather than a tab, although the defect covers both: a tab in a
    /// single-line egui field is a focus-moving key in most toolkits and would
    /// risk measuring the focus handling instead of the search. The space is
    /// also the character the operator actually named.
    pub const SPACE: u16 = 0x20;

    /// `2` — the digit, for the `Ctrl+2` mode chord.
    ///
    /// Present only as a **control probe**: `Ctrl+2` is bound to
    /// `mode.review` and is a chord the application's key table has always
    /// been able to spell, so a check that gets nothing from it learns that
    /// the keystroke never arrived, rather than that the feature under test
    /// is broken.
    pub const DIGIT_2: u16 = 0x32;
}
