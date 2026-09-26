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
///
/// A group absent from this table **never collapses**, which is the safe
/// default and the reason absence rather than a sentinel means "never": a tab
/// added later without a ladder entry behaves exactly as it did before this
/// feature existed.
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
///
/// Called once, at the end of `built_in`, so the tab modules stay lists of
/// commands and this file stays the only place a ranking is stated.
///
/// Silently ignores an entry naming a group that does not exist — the test
/// below is what makes that safe, and it is the right split: a typo should
/// fail the build, not the running application, and a *layer* that removed a
/// group at runtime should not panic the ribbon.
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

    /// ★★ **Every entry names a group that exists.**
    ///
    /// The guard that pays for keeping the ranking away from the definitions.
    /// A renamed group would otherwise lose its rung silently: the ribbon
    /// would still work, still collapse, and simply never collapse *that*
    /// group — a defect with no symptom until an operator's band overflows at
    /// a width where it used not to.
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
    ///
    /// Named separately from the test above because they fail for opposite
    /// reasons: that one catches a stale table, this one catches an `apply`
    /// that stopped being called — which would leave every group unrankable
    /// and every band collapsing nothing, a state that looks exactly like the
    /// feature having never been built.
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

    /// ★★★ **Every tab keeps at least one group off the ladder** — except the
    /// one where that is a deliberate decision, which is named here so the
    /// exception cannot be acquired by accident.
    ///
    /// This is the invariant that stops the ladder from being tuned into
    /// uselessness. A tab whose every group may collapse can reach a width at
    /// which it is a row of identical chevron buttons and nothing else: the
    /// operator can still reach every command, and the band has stopped
    /// telling them anything. Word never does this — Clipboard is expanded at
    /// 460 pt, the narrowest width measured.
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
