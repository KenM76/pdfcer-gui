//! # shell::commands — every verb pdfcer can perform
//!
//! [`register`] populates an `egui_shell::CommandRegistry` with every command
//! this build has. They reach the operator by three routes:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/mod.md`.

pub mod catalog;
pub mod mapping;

/// **A registered command must be REACHABLE by some arm of `app::dispatch`.**
///
/// Every other obligation this catalogue carries is about the *registration*
/// being self-consistent — a count, a group count, a `PLANNED` removal, a RON
/// regeneration, a `KNOWN` condition name — and a command can satisfy all of
/// them while doing nothing at all: drawn on the quick-access toolbar, bound to
/// a chord, printing that chord in its own tooltip, with no dispatch arm behind
/// it. [`reach`] is the assertion that closes that gap: every id in this
/// registry is routed by a literal arm, claimed by a guard arm, or listed in
/// [`reach::SCAFFOLDED`] with a written reason.
///
/// `#[cfg(test)]` because the reader parses `app/dispatch.rs` with `syn`, a
/// **dev**-dependency — see this crate's `Cargo.toml` for why a real parser and
/// not a grep, and [`reach`]'s own header for what a grep cannot see.
/// Nothing here is compiled into `pdfcer-gui.exe`.
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
///
/// A constant rather than a literal because this id is used in four places
/// that must agree and two of them are not obvious: the registration below,
/// the `CUSTOM_BACKED` entry that records why it is on no tab, the registry
/// lookup in [`crate::app::PdfcerApp::ribbon_band`] that turns the operator's
/// menu choice back into this command's token, and the dispatch arm. A typo
/// in any of them produces silence — a menu that draws and reports nothing —
/// rather than an error.
///
/// The other command ids stay literals at their (single) use sites, which is
/// this file's existing convention; this one earns a name by being spelled in
/// two modules.
pub const FILE_RECENT: &str = "file.recent"; // ui-text-exempt: a command id, never displayed

/// **Register every command the built-in manifest names.**
///
/// # Panics
///
/// If two commands claim one id. That is a programming error in
/// [`catalog::all`] and not a condition any input can produce, so it fails
/// loudly at
/// start-up rather than being swallowed: the registry refuses a duplicate
/// precisely so that behaviour cannot come to depend on the order of
/// start-up code, and catching the error here to ignore it would give back
/// exactly the defect the refusal prevents.
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
///
/// Mostly commentary against a handful of assertions, which is the point rather
/// than an accident: an integer records nothing, and what a reader needs when
/// one of them fails is whether the change that moved it was supposed to.
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
