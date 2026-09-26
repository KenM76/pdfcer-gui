//! # `shell::commands::catalog` — the list itself, and the argument for every
//! entry on it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/catalog.md`.

use crate::text::commands::CommandText;
use egui_shell::{Command, HandlerToken};

/// One command, with its label and tooltip taken from the catalog.
///
/// The two are always fetched together, from one catalog entry, so a
/// command cannot end up with one command's label and another's tooltip —
/// which is not a hypothetical: the salvage source's two adjacent Content
/// buttons both read `Aa`, and only their tooltips distinguished them.
pub(super) fn command(id: &str, text: CommandText, handler: u64) -> Command {
    Command::new(id, text.label, HandlerToken::new(handler)).with_tooltip(text.tooltip)
}

/// Every command, in manifest order.
///
/// # Why the registry is assembled from one band per tab
///
/// The registry is a flat namespace and this ordering mirrors the ribbon, so
/// the two can be read against `RIBBON_IA.md` §5 side by side — and the
/// concatenation below preserves both: one namespace, ribbon order.
///
/// ⚠ **A per-tab split does NOT hide a handler-token collision**, which is the
/// argument most likely to be raised against it.
/// [`super::tests::every_handler_token_is_unique`] sweeps the whole registry
/// and [`super::tests::every_handler_token_is_in_its_tabs_block`] asserts each
/// token sits inside its own tab's hundred. A collision is a red test in either
/// arrangement, so it is not a reason to keep every command and its prose in
/// one file.
pub(super) fn all() -> Vec<Command> {
    // One band per tab, concatenated in ribbon order.
    //
    // The order is the ribbon's own and it is load-bearing for exactly one
    // reason: `egui_shell` renders a group's items in the order the manifest
    // names them, not in registry order, so this sequence decides nothing about
    // the ribbon — but it decides what a reader of `--all` sees, and a
    // catalogue that listed Format before File would be a second ordering for
    // somebody to reconcile against §5.
    let mut out = Vec::new();
    out.extend(file::band());
    out.extend(view::band());
    out.extend(pages::band());
    out.extend(edit::band());
    out.extend(markup::band());
    // The Markup tab's second band. A group rather than a tab — the ids stay
    // `markup.*`; see that file's header for why the file is named for the group
    // and the ids for the tab.
    out.extend(arrange::band());
    out.extend(measure::band());
    out.extend(tools::band());
    out.extend(format::band());
    out.extend(modes::band());
    out
}

/// the Markup tab's Arrange group — which mark is drawn on top
mod arrange;
/// the Edit tab — changing content that is already there
mod edit;
/// the File tab — opening, saving, exporting, printing, and pdfcer itself
mod file;
/// the Format contextual tab — what changes about the selection
mod format;
/// the Markup tab — what is added for somebody else to read
mod markup;
/// the Measure tab — ce dimensions and the scale they are read at
mod measure;
/// the mode selector — Read, Review, Edit
mod modes;
/// the Pages tab — what happens to the set of sheets
mod pages;
/// the Tools tab — what runs across files, or is configured once
mod tools;
/// the View tab — what is on screen and how the page is laid out
mod view;
