//! # `ribbonladder` — which groups give up their rows first
//!
//! **The editorial half of S3.** `egui-shell`'s
//! [`egui_shell::ribbon::plan::collapse`] knows *how* to collapse a group; it
//! deliberately does not know *which*, because that answer is a judgment about
//! this application's commands and the shell is forbidden to hold one (R7).
//! This file is where pdfcer answers.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ribbonladder.md`.

use egui_shell::manifest::Shell;

/// `(tab id, group id, priority)` — lower collapses first.
#[doc(hidden)]
pub const LADDER: &[(&str, &str, u32)] = &[
    // FILE — the tab is open/save/print. Everything else is occasional.
    ("file", "pdfcer", 1),    // About, Settings — visited once a month
    ("file", "document", 2),  // Properties, Fonts — inspection, not action
    ("file", "export", 3),    // DXF, form data, text — deliberate errands
    ("file", "recognise", 4), // OCR — a real verb, but a rare one
    // VIEW — navigating and zooming are the tab. The rest is chrome.
    ("view", "window", 1),
    ("view", "display", 2),
    ("view", "panels", 3),
    ("view", "page_display", 4),
    // PAGES — Organise is the reason; insert and transform support it.
    ("pages", "transform", 1),
    ("pages", "insert", 2),
    // EDIT — Content is the tab. Protect is the least-reached.
    ("edit", "protect", 1),
    ("edit", "forms", 2),
    ("edit", "clipboard", 3),
    ("edit", "insert", 4),
    // MARKUP — Shapes is the pen. Style configures it, so it goes late.
    ("markup", "comments", 1),
    ("markup", "notes", 2),
    ("markup", "text_markup", 3),
    ("markup", "style", 4),
    // MEASURE — Dimension is the tab; Scale is set once per drawing.
    ("measure", "scale", 1),
    // TOOLS — no group here outranks another, so all three may collapse.
    ("tools", "diagnostics", 1),
    ("tools", "fonts", 2),
    ("tools", "batch", 3),
];

/// Apply [`LADDER`] to a built shell.
pub fn apply(shell: &mut Shell) {
    for (tab_id, group_id, priority) in LADDER {
        if let Some(group) = shell
            .tabs
            .iter_mut()
            .flatten()
            .chain(shell.contextual_tabs.iter_mut().flatten())
            .filter(|t| t.id == *tab_id)
            .flat_map(|t| t.groups.iter_mut().flatten())
            .find(|g| g.id == *group_id)
        {
            group.collapse = Some(*priority);
        }
    }
}
