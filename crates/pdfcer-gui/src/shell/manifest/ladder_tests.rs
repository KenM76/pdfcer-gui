//! Tests for `crate::shell::manifest::ladder`, kept in the gui because they reach gui modules.

use crate::shell::manifest::ladder::*;

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
