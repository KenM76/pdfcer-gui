//! # `shell::manifest::ladder` — which groups give up their rows first
//!
//! **The editorial half of S3.** `egui-shell`'s
//! [`egui_shell::ribbon::plan::collapse`] knows *how* to collapse a group; it
//! deliberately does not know *which*, because that answer is a judgment about
//! this application's commands and the shell is forbidden to hold one (R7).
//! This file is where pdfcer answers.
//!
//! ## ★★★ Why one table and not a `.collapses_at(n)` on each group
//!
//! Because a collapse priority is not a property of a group. It is a
//! **ranking of groups against each other**, and the only way to review a
//! ranking is to see all of it at once. Scattered across the eight tab
//! modules, the question *"is Export really less important than Document?"*
//! could not be answered without opening two files and holding a third in your
//! head; here it is two adjacent lines.
//!
//! The cost is that the priority sits away from the group's definition. That
//! is a real cost and it is paid deliberately, with a guard: every id in this
//! table is checked against the built manifest by
//! [`tests::every_ladder_entry_names_a_real_group`], so a group that is
//! renamed or removed fails the build rather than silently losing its rung.
//!
//! ## The rule the ranking follows
//!
//! Word's, measured from `evidence/word-ribbon/`: **the group that never
//! collapses is the one carrying the verb the operator came to the tab for**,
//! not the smallest one. Clipboard is wider than Editing and outlives it at
//! every width from 1900 down to 460, because Paste is why you are on the Home
//! tab.
//!
//! So each tab here keeps one or two groups off the ladder entirely, and ranks
//! the rest by how far they are from that tab's reason for existing:
//!
//! | tab | never collapses | why |
//! |---|---|---|
//! | File | File, Save, Print | open it, keep it, print it — the whole tab |
//! | View | Navigate, Zoom | moving around the document IS the View tab |
//! | Pages | Organise | reordering sheets is the reason to be here |
//! | Edit | Content | the edit verbs themselves |
//! | Markup | Shapes | the pen — everything else configures it |
//! | Measure | Dimension | the measuring tools |
//! | Tools | — | every group here is a utility; none outranks the others |
//! | Format | Selection | one group; collapsing it would gain nothing |
//!
//! ★ **Lower collapses first**, and the numbers are deliberately sparse (1, 2,
//! 3, 4) rather than dense, so a group can be inserted between two existing
//! rungs later without renumbering the tab.

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
