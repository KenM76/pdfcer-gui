//! # shell::commands — every verb pdfcer can perform
//!
//! [`register`] populates an `egui_shell::CommandRegistry` with the
//! **hundred and twenty-one** commands this build has, which fall into three
//! groups:
//!
//! | group | count | how the operator reaches it |
//! |---|---|---|
//! | on a tab, the QAT or the keymap | 116 | a control [`super::manifest::built_in`] names by id |
//! | drawn by a **custom item** | 4 | `file.recent` and the three Format ▸ Font controls — see [`super::manifest::CUSTOM_BACKED`] |
//! | drawn on the **status bar** | 1 | `edit.find` — `RIBBON_IA.md` §6 |
//!
//! ★★★ **And it drifted a fifth and a sixth time, both found on 2026-08-27,
//! and the sixth is the one worth reading.** This table said *hundred and one*
//! / 99 / 1 / 1 while
//! [`tests::registration_succeeds_and_registers_every_command`] asserted
//! **115** and passed — nineteen commands out, which is not a rounding error,
//! it is a table describing a different program. The pinned number moved
//! nineteen times and the sentence above it moved never, because a test cannot
//! read prose.
//!
//! The sixth is the same defect one level down: the *custom-item* row said 1
//! for as long as `file.recent` was the only such command, and the Format ▸
//! Font group added three more. That row is the one an engineer would trust
//! when asking *"can a command live off the manifest?"*, and it would have
//! answered "yes, once, exceptionally" about a build where the answer is "yes,
//! four times, and there is a register listing them."
//!
//! ⇒ **Do not trust a count in this file without re-reading the assertion it
//! is supposed to mirror.** That instruction has now been earned six times and
//! the running tally is left in place — see the two paragraphs below — because
//! the tally is the argument.
//!
//! (This header said *eighty-one* and *79* until 2026-08-14, while
//! [`tests::registration_succeeds_and_registers_every_command`] asserted 88
//! and passed. Prose drifting from a number a test pins is a defect this
//! project has now had three times; the test is the fact, and the table has
//! been corrected to it rather than the other way round.)
//!
//! ★ **And it happened a fourth time, in the four hours before `file.new` was
//! written.** This table read *ninety-nine* and *97* while the assertion below
//! held **100**: `file.ocr` had been registered and the two sentences above it
//! were not moved. It is recorded rather than quietly repaired for the reason
//! the paragraph above it was: the count is the one thing here that has drifted
//! every single time, and the running tally is the argument for why nobody
//! should trust a number in this file without re-reading the assertion. Both
//! were corrected together when `file.new` took the total to 101.
//!
//! The last two are the interesting ones, because both are reachable by an
//! operator and neither is a button on a tab. They are kept honest by
//! different mechanisms and the difference is worth knowing: a `Custom` item
//! carries no command id, so `Shell::command_references()` cannot see
//! `file.recent` at all and [`super::manifest::CUSTOM_BACKED`] is the
//! register that says why that is allowed; whereas the status bar is simply
//! not part of the manifest, and `edit.find` needs no exemption because the
//! keymap's `Ctrl+F` binding **is** a reference site — so the orphan check
//! still guards it against a rename.
//!
//! A command carries five things:
//!
//! | Field | Comes from | Why here rather than the manifest |
//! |---|---|---|
//! | `id` | this file | the manifest may only *reference* it |
//! | `label`, `tooltip` | [`crate::text::commands`] | copy is a design surface with one owner |
//! | `icon` | this file, as a key | the icon *set* is the application's; the shell only needs to know which one |
//! | `enable` | this file, as a condition name | *"predicates are safety, not decoration"* |
//! | `handler` | this file, as an opaque `u64` | the shell never interprets it |
//!
//! `SHELL_FRAMEWORK.md` §5 turns that split into the customization
//! contract. An operator may reorder tabs, rename them, hide them, move a
//! command between groups, create tabs, rebind keys, and define new modes.
//! An operator may **not** invent a command, change what a command does,
//! or bypass an enable predicate. Every one of those prohibitions is a
//! consequence of this half being code and the other half being data.
//!
//! # Handler tokens are opaque, and this file does not implement behaviour
//!
//! An `egui_shell::HandlerToken` is a `u64` the shell stores and hands back
//! when the command is invoked. The application dispatches on it at **one
//! choke point**, which is where a confirmation gate, an undo entry or a
//! trace belongs; a registry of closures would scatter that across as many
//! sites as there are commands, and would force the shell to name pdfcer's
//! state type, which would end its reusability.
//!
//! The numbers are assigned here in blocks of one hundred, one block per
//! tab, and they are **stable**: a token is never reused for a different
//! command, because a persisted or traced token that silently changed
//! meaning between builds is a defect with no symptom at the site that
//! caused it. Gaps in the numbering are fine and expected — a command
//! removed leaves its number unused.
//!
//! # Enable conditions
//!
//! `Enable::When("doc.open")` names a condition the application publishes
//! once per frame in an `egui_shell::commands::ConditionSet`. Data rather
//! than a closure, because a name is serializable, testable headlessly,
//! and cannot capture state that makes a command's availability depend on
//! *when* it was registered.
//!
//! Seven conditions are used, and the whole vocabulary is listed here
//! because every one of them is a promise the application has to keep:
//!
//! | Condition | True when | Used by |
//! |---|---|---|
//! | *(none)* | always | commands with no precondition: Open, Settings, the batch tools, the window and render settings |
//! | `docs.multiple` | **more than one document is open** | Next / Previous document |
//! | `panels.floating` | **some panel is in a window of its own** | Dock all panels |
//! | `doc.open` | a document is open | document-level commands — close, save a copy, properties, print |
//! | `doc.pages` | …and it has at least one page | everything that acts on a page |
//! | `undo.available` / `redo.available` | the corresponding stack is non-empty | Undo, Redo |
//! | `selection.any` | something is selected | the contextual Format tab and its Delete |
//! | `selection.bounds` | …and it still resolves to a box on the page shown | Zoom to selection |
//!
//! `docs.multiple` is deliberately **not** a refinement of `doc.open`, and it
//! is published outside that arm. A tab whose file failed to open is still a
//! tab (`crate::app::documents` §2), so an operator can be sitting on a
//! damaged file — `doc.open` false — with three good documents behind it, and
//! that is the moment they most need a way back to one of them. Nesting the
//! condition would grey the only route out of a failed open.
//!
//! `selection.bounds` is separate from `selection.any` for the same shape
//! of reason, one level down. A selection here is an **identity** — page,
//! object, subpath, node — and an identity can outlive the box it once
//! described: it may name an object on a page that is not shown, or one an
//! edit has renumbered. Zoom to selection is the command where that gap is
//! visible, because framing nothing is not a no-op; it is a jump to the
//! origin that looks exactly like a bug.
//!
//! `doc.pages` is separate from `doc.open` because **a PDF with `/Count 0`
//! is a legal document**. pdfcer opens it, shows "This document has no
//! pages", and must not offer to rotate one. Collapsing the two would make
//! that file arm tools that cannot run — the exact class of failure the
//! removal of the `Editing on` master toggle was meant to end.
//!
//! Greying is what a false predicate produces, and P3 permits it only for
//! *temporarily* unavailable, *always explained on hover*. Every command
//! here has a tooltip; [`crate::text::commands`] has no way to express a
//! command without one.
//!
//! # Where the list itself lives
//!
//! [`catalog::all`] — its own file since 2026-08-14, when wiring
//! `file.save_copy` took this one past the 1,500-line gate (**R2**). The seam
//! is the one this header already draws: everything above is the **contract**
//! (what a command is, what a token means, what conditions the application
//! promises to publish), and everything in [`catalog`] is the **list** (which
//! commands exist, with which glyph and which predicate, and why each of those
//! was chosen). The icon-coverage argument moved with the registrations it
//! justifies, because an argument belongs beside the thing it argues for.
//!
//! The list is still **one flat function in one file** — see [`catalog`]'s
//! header for why splitting *that* would have been the cheaper edit and the
//! wrong one.

pub mod catalog;
pub mod mapping;

/// ★ **The sixth obligation, and the only one the five above cannot express:
/// a registered command must be REACHABLE by some arm of `app::dispatch`.**
///
/// `HANDOFF.md` §5's five obligations are all about the *registration* being
/// consistent — a count, a group count, a `PLANNED` removal, a RON
/// regeneration, a `KNOWN` condition name. Every one of them was satisfied by
/// `file.save_copy` on the day it was drawn on the quick-access toolbar, bound
/// to `Ctrl+S`, printed "(Ctrl+S)" in its own tooltip, and **did nothing**,
/// because no dispatch arm existed. [`reach`] is the assertion that closes
/// that: every id in this registry is routed by a literal arm, claimed by a
/// guard arm, or listed in [`reach::SCAFFOLDED`] with a written reason.
///
/// `#[cfg(test)]` because the reader parses `app/dispatch.rs` with `syn`, a
/// **dev**-dependency — see this crate's `Cargo.toml` for why a real parser and
/// not a grep, and [`reach`]'s own header for the two mechanisms that lost.
/// Nothing here is compiled into `pdfcer-gui.exe`.
#[cfg(test)]
mod reach;

/// ★ Re-exported flat, so every caller still writes
/// `shell::commands::measure_command` and nothing outside `shell/` learns that
/// the module was split.
///
/// A `pub use` rather than moving the callers, deliberately: the split is an
/// **R2** consequence, not a change to what the shell offers, and a file-size
/// rule that rewrote fifteen call sites in `app/` would be a rule that makes
/// unrelated diffs. See `mapping`'s own header for what the seam is.
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

/// The properties every registration in this catalogue must hold — the command
/// count, the icon-coverage split, the handler-token blocks, the condition
/// vocabulary and the with-nothing-open enabled set, each carrying the running
/// ledger of why its literal is the number it is. Split out under **R2** on
/// 2026-09-06; see that module's header for the seam.
#[cfg(test)]
mod tests;
