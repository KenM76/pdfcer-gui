//! # shell::commands — every verb pdfcer can perform
//!
//! [`register`] populates an `egui_shell::CommandRegistry` with every command
//! this build has. They reach the operator by three routes:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/mod.md`.

pub mod catalog;
pub mod mapping;

/// **A registered command must be REACHABLE by some arm of `app::dispatch`.**
#[cfg(test)]
mod reach;

/// Re-exported flat, so every caller writes `shell::commands::measure_command`
/// and nothing outside `shell/` needs to know which file the function lives in.
///
/// A `pub use` rather than moving the callers, deliberately: where a function
/// sits inside `shell/` is not a change to what the shell offers, and moving it
/// must not rewrite fifteen call sites in `app/`. See `mapping`'s own header
/// for what the seam is.
pub use mapping::{
    chrome_command, chrome_for_command, form_for_command, markup_command, markup_for_command,
    measure_command, measure_for_command, page_display_command, page_display_for_command,
    text_mark_command, text_mark_for_command,
};

use egui_shell::CommandRegistry;

/// **Open a document from the recent list.**
pub const FILE_RECENT: &str = "file.recent"; // ui-text-exempt: a command id, never displayed

/// **Register every command the built-in manifest names.**
pub fn register(reg: &mut CommandRegistry) {
    reg.register_all(catalog::all())
        // ui-text-exempt: a panic message, read by whoever is looking at
        // the stack trace. Never rendered to an operator — the process
        // does not reach a window if this fires.
        .expect("two shell commands claim the same id");
}

/// **The two counters, and why each is the number it is** — the command count
/// and the icon-coverage split, each literal carrying the reasoning that fixes
/// it.
#[cfg(test)]
mod ledger;

/// The properties every registration in this catalogue must hold — the
/// handler-token blocks, the condition vocabulary, the with-nothing-open
/// enabled set, the tooltip rule and the icon-key rules, each carrying the
/// reasoning that fixes its literal. See that module's header for the seam.
///
/// The two *counts* live in [`ledger`], not here.
#[cfg(test)]
mod tests;
