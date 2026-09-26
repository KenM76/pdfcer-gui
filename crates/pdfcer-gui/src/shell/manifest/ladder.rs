//! # `shell::manifest::ladder` — which groups give up their rows first
//!
//! **The editorial half of S3.** `egui-shell`'s
//! [`egui_shell::ribbon::plan::collapse`] knows *how* to collapse a group; it
//! deliberately does not know *which*, because that answer is a judgment about
//! this application's commands and the shell is forbidden to hold one (R7).
//! This file is where pdfcer answers.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/manifest/ladder.md`.

use egui_shell::manifest::Shell;

/// `(tab id, group id, priority)` — lower collapses first.
const LADDER: &[(&str, &str, u32)] = &[
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
pub(super) fn apply(shell: &mut Shell) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every entry names a group that exists.**
    #[test]
    fn every_ladder_entry_names_a_real_group() {
        let shell = crate::shell::manifest::built_in();
        for (tab_id, group_id, _) in LADDER {
            let found = shell
                .tabs
                .iter()
                .flatten()
                .chain(shell.contextual_tabs.iter().flatten())
                .filter(|t| t.id == *tab_id)
                .flat_map(|t| t.groups.iter().flatten())
                .any(|g| g.id == *group_id);
            assert!(
                found,
                "the collapse ladder names {tab_id}/{group_id}, which is not a \
                 group in the built manifest — it was renamed or removed, and \
                 its rung went with it"
            );
        }
    }

    /// **The ladder is actually applied**, and to the right groups.
    #[test]
    fn the_built_manifest_carries_the_priorities() {
        let shell = crate::shell::manifest::built_in();
        let group = shell
            .tabs
            .iter()
            .flatten()
            .find(|t| t.id == "view")
            .and_then(|t| t.groups.as_ref())
            .and_then(|g| g.iter().find(|g| g.id == "window"))
            .expect("view/window must exist");
        assert_eq!(group.collapse, Some(1));
    }

    /// **Every tab keeps at least one group off the ladder** — except the
    /// one where that is a deliberate decision, which is named here so the
    /// exception cannot be acquired by accident.
    #[test]
    fn every_tab_keeps_something_expanded() {
        // Tools is the deliberate exception: every group on it is an
        // occasional utility and none outranks the others, so there is no
        // honest answer to "which one stays". Stated here rather than left to
        // be noticed.
        const MAY_FULLY_COLLAPSE: &[&str] = &["tools"];

        let shell = crate::shell::manifest::built_in();
        for tab in shell
            .tabs
            .iter()
            .flatten()
            .chain(shell.contextual_tabs.iter().flatten())
        {
            if MAY_FULLY_COLLAPSE.contains(&tab.id.as_str()) {
                continue;
            }
            let groups: Vec<_> = tab.groups.iter().flatten().collect();
            if groups.is_empty() {
                continue;
            }
            assert!(
                groups.iter().any(|g| g.collapse.is_none()),
                "every group on the {} tab may collapse, so at a narrow enough \
                 width the band is nothing but chevrons. Leave the group \
                 carrying the tab's own verb off the ladder, or add the tab to \
                 MAY_FULLY_COLLAPSE with a reason",
                tab.id
            );
        }
    }
}
