//! # `text::settings` — every word the Settings window shows
//!
//! The catalog area for `pdfcer_gui::dialogs::settings`. Ported from the old
//! shell's `ui_text.rs`, where these strings occupied roughly 700 lines in the
//! middle of a 7,912-line file.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/settings/mod.md`.

pub mod bytes;
pub mod extract;
pub mod look;
/// The page list and search box.
pub mod nav;
/// The colour the recognised text is drawn in over a scan — O229. Its own
/// file because it is the copy for a *feature*, where this module's neighbours
/// are copy for answers to a silent standard.
pub mod ocrlayer;
pub mod overprint;
/// The two print-ready colour controls and the field wash. Its header says
/// which of the three is there for a weak reason and should move out first if
/// the module grows.
pub mod print_colour;
pub mod redaction;
/// Whether another program may drive the window.
pub mod remote;
/// The settings that change pdfcer's own window.
pub mod shell;

pub use bytes::*;
pub use extract::*;
pub use look::*;
pub use nav::*;
pub use ocrlayer::*;
pub use overprint::*;
pub use print_colour::*;
pub use redaction::*;
pub use remote::*;
pub use shell::*;

use pdfcer_core::settings::StoreKind;
use pdfcer_core::settings::StoreLocation;

// ===========================================================================
// Window chrome
// ===========================================================================

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Settings"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "The PDF standard leaves some things genuinely undefined, so different \
     programs can open the same file and be equally correct while showing you \
     different results. Where that happens, pdfcer asks you rather than deciding \
     quietly. Each choice below says what the standard does not settle, what \
     pdfcer ships as its answer and why, and what changing it affects."
}

/// Where the settings file lives, said in the operator's terms.
#[must_use]
pub fn store_location(store: &StoreLocation) -> String {
    match (store.kind, store.path.as_deref()) {
        (StoreKind::Portable, Some(path)) => format!(
            "Kept in {} — this folder is yours. When you update pdfcer by replacing \
             the program files, keep it.",
            path.display()
        ),
        (StoreKind::Portable, None) => "Your choices are kept beside the program.".to_owned(),
        (StoreKind::PlatformFallback, Some(path)) => format!(
            "Kept in {} because pdfcer's own folder is not writable. These choices \
             will NOT travel with the program folder if you move or copy it.",
            path.display()
        ),
        (StoreKind::PlatformFallback, None) => {
            "Kept in your system settings folder, because pdfcer's own folder is not writable."
                .to_owned()
        }
        _ => "No writable location was found, so anything you change here lasts only \
              until you close pdfcer."
            .to_owned(),
    }
}

// ===========================================================================
// Buttons
// ===========================================================================

/// The commit button.
#[must_use]
pub const fn save() -> &'static str {
    "Save"
}

/// Why Save is greyed.
#[must_use]
pub const fn save_disabled_tooltip() -> &'static str {
    "Nothing has changed yet."
}

/// The abort button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// What Cancel promises, said plainly and unconditionally.
#[must_use]
pub const fn cancel_tooltip() -> &'static str {
    "Close without changing anything. Nothing you have clicked here has taken \
     effect yet."
}

/// The reset control.
#[must_use]
pub const fn restore_defaults() -> &'static str {
    "Restore defaults"
}

/// Why *Restore defaults* is greyed.
#[must_use]
pub const fn restore_defaults_disabled_tooltip() -> &'static str {
    "Everything is already set to pdfcer's own answer."
}

/// What *Restore defaults* actually does, on hover when it is live.
#[must_use]
pub const fn restore_defaults_tooltip() -> &'static str {
    "Sets every choice below back to pdfcer's own answer. Nothing is written \
     until you press Save, and Cancel still puts everything back."
}

/// The status-bar line after a successful save.
#[must_use]
pub fn saved(path: &str) -> String {
    format!("Settings saved to {path}.")
}

/// The status-bar line after a failed save.
#[must_use]
pub fn save_failed(reason: &str) -> String {
    format!(
        "Settings could NOT be saved: {reason} — this session is using your \
         choices, but they will be gone when pdfcer restarts."
    )
}

// ===========================================================================
// Group headings
// ===========================================================================

/// Group 1.
#[must_use]
pub const fn group_appearance() -> &'static str {
    "Appearance"
}

/// Group 2 — the one that starts expanded.
#[must_use]
pub const fn group_colour() -> &'static str {
    "Colour"
}

/// The group holding the one control about the PERSON rather than the document
/// or the program.
#[must_use]
pub const fn group_comments() -> &'static str {
    "Comments"
}

/// The Forms group's caption.
#[must_use]
pub const fn group_forms() -> &'static str {
    "Forms"
}

/// Group 3.
#[must_use]
pub const fn group_images() -> &'static str {
    "Images and transparency"
}

/// The Fonts group's caption.
#[must_use]
pub const fn group_fonts() -> &'static str {
    "Fonts"
}

/// The folder list's label.
#[must_use]
pub const fn font_folders_label() -> &'static str {
    "Folders to take fonts from"
}

/// The hint states the **consequence of leaving it empty**, which is the
/// one fact an operator cannot discover from an empty list.
#[must_use]
pub const fn font_folders_hint() -> &'static str {
    "When a document names a font it does not carry, pdfcer looks here to embed it. It \
     never searches your system fonts on its own."
}

/// Shown in place of an empty list.
#[must_use]
pub const fn font_folders_none() -> &'static str {
    "No folders of your own yet."
}

/// The same empty state when the OS-fonts box is **not** ticked either.
#[must_use]
pub const fn font_folders_none_at_all() -> &'static str {
    "No folders yet and this computer's fonts are switched off, so embedding a missing \
     font has nowhere to take one from."
}

/// The checkbox the operator asked for, in his own words.
#[must_use]
pub const fn use_os_fonts_label() -> &'static str {
    "Use the fonts installed on this computer"
}

/// What ticking it means, including the part pdfcer cannot answer for them.
#[must_use]
pub const fn use_os_fonts_hint() -> &'static str {
    "Embedding puts a font's outlines inside a document you may send to somebody else, \
     and whether you may do that depends on the font. pdfcer leaves that to you."
}

/// The heading over the folders the checkbox resolves to.
#[must_use]
pub const fn use_os_fonts_folders() -> &'static str {
    "pdfcer will also search:"
}

/// Shown when the box is ticked and the machine reports no font folder at all.
#[must_use]
pub const fn use_os_fonts_none_found() -> &'static str {
    "pdfcer could not find a font folder on this computer."
}

/// The Add button.
#[must_use]
pub const fn font_folder_add() -> &'static str {
    "Add a folder…"
}

/// See [`font_folder_add`].
#[must_use]
pub const fn font_folder_add_hover() -> &'static str {
    "Folders are searched in the order they are listed, and the first one holding the face wins."
}

/// The per-row remove button.
#[must_use]
pub const fn font_folder_remove() -> &'static str {
    "Remove"
}

/// See [`font_folder_remove`].
#[must_use]
pub const fn font_folder_remove_hover() -> &'static str {
    "Stop searching this folder. Nothing on disk is touched."
}

/// The Add button when the list is at its cap.
#[must_use]
pub fn font_folders_full(cap: usize) -> String {
    format!("{cap} folders is the most pdfcer will search. Remove one to add another.")
}

/// The folder picker's title bar.
#[must_use]
pub const fn font_folder_dialog_title() -> &'static str {
    "Choose a folder pdfcer may take fonts from"
}

/// Group 4.
#[must_use]
pub const fn group_text() -> &'static str {
    "Copying and extracting text"
}

/// Group 5.
#[must_use]
pub const fn group_measuring() -> &'static str {
    "Measuring and dimensioning"
}

/// Group 6.
#[must_use]
pub const fn group_pages() -> &'static str {
    "Pages and printing"
}

/// Group 7.
#[must_use]
pub const fn group_saving() -> &'static str {
    "Saving files"
}

/// Group 8 — the only one that is not about the PDF standard.
#[must_use]
pub const fn group_display() -> &'static str {
    "Drawing the page"
}
