//! # `shell::commands::catalog` — the list itself, and the argument for every
//! entry on it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/catalog.md`.

use crate::text::commands::CommandText;
use egui_shell::{Command, HandlerToken};

/// One command, with its label and tooltip taken from the catalog.
pub(super) fn command(id: &str, text: CommandText, handler: u64) -> Command {
    Command::new(id, text.label, HandlerToken::new(handler)).with_tooltip(text.tooltip)
}

/// Every command, in manifest order.
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
    out.extend(arrange::edit_band());
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

/// the Markup and Edit tabs' Arrange groups — what is drawn on top
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
