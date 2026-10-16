//! One-time reshapes of a remembered workspace, applied as it is restored.
//!
//! Each is recorded per workspace in the layout file, so it runs once: an
//! arrangement the operator makes afterwards — including the very shape an
//! upgrade removed — is never undone. A workspace born from this build's
//! default is marked without being touched, since the default already has
//! the shape.

use egui_shell::dock::{DockLayout, PanelId};
use egui_shell::layout::LayoutDocument;

use crate::panels::Panel;

/// O301: the right side's object list shares the inspector's tab bar.
const ONE_INSPECTOR_BAR: &str = "one-inspector-bar";

/// Apply to `layout`, restored for workspace `ws`, every upgrade it has not
/// had; `remembered` is false when `layout` is this build's default.
pub(super) fn apply(doc: &mut LayoutDocument, ws: &str, layout: &mut DockLayout, remembered: bool) {
    let id = format!("{ONE_INSPECTOR_BAR}:{ws}");
    if doc.upgrade_applied(&id) {
        return;
    }
    if remembered && merge_inspector(layout) {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| {
            format!("layout-upgraded workspace={ws} upgrade={ONE_INSPECTOR_BAR}")
        });
    }
    doc.mark_upgrade(&id);
}

/// Fold a right-side stack holding only Objects into the stack under it when
/// that one opens with Properties: Properties stays first and the active tab
/// is kept. `true` when a fold was made.
fn merge_inspector(layout: &mut DockLayout) -> bool {
    let objects = PanelId::new(Panel::Objects.command_id());
    let properties = PanelId::new(Panel::Properties.command_id());
    for column in &mut layout.right.columns {
        let Some(i) = column.stacks.windows(2).position(|pair| {
            pair[0].tabs == [objects.clone()] && pair[1].tabs.first() == Some(&properties)
        }) else {
            continue;
        };
        let master = column.stacks.remove(i);
        let detail = &mut column.stacks[i];
        detail.tabs.insert(1, objects);
        if detail.active >= 1 {
            detail.active += 1;
        }
        detail.share += master.share;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::dock::{Column, SideLayout, Stack};

    fn id(panel: Panel) -> PanelId {
        PanelId::new(panel.command_id())
    }

    /// The arrangement Edit shipped with before O301: Objects over a
    /// Properties-led stack, Forms active.
    fn two_bars() -> DockLayout {
        let detail =
            Stack::tabbed([id(Panel::Properties), id(Panel::Forms)]).with_active(&id(Panel::Forms));
        DockLayout::new(
            SideLayout::default(),
            SideLayout::new([Column::new([Stack::new(id(Panel::Objects)), detail])])
                .with_width(360.0),
        )
    }

    #[test]
    fn a_remembered_two_bar_inspector_becomes_one_bar_once() {
        let mut doc = LayoutDocument::default();
        let mut layout = two_bars();
        apply(&mut doc, "edit", &mut layout, true);
        let stacks = &layout.right.columns[0].stacks;
        assert_eq!(stacks.len(), 1);
        assert_eq!(
            stacks[0].tabs,
            [id(Panel::Properties), id(Panel::Objects), id(Panel::Forms)]
        );
        assert_eq!(
            stacks[0].active_panel(),
            Some(&id(Panel::Forms)),
            "the active tab is kept"
        );

        let mut resplit = two_bars();
        apply(&mut doc, "edit", &mut resplit, true);
        assert_eq!(
            resplit,
            two_bars(),
            "a split made after the upgrade is the operator's"
        );
    }

    #[test]
    fn a_default_layout_is_marked_and_never_folded_later() {
        let mut doc = LayoutDocument::default();
        let mut fresh = two_bars();
        apply(&mut doc, "edit", &mut fresh, false);
        assert_eq!(fresh, two_bars(), "a default is not reshaped");
        let mut later = two_bars();
        apply(&mut doc, "edit", &mut later, true);
        assert_eq!(
            later,
            two_bars(),
            "the operator's own split survives a relaunch"
        );
    }

    #[test]
    fn each_workspace_is_upgraded_on_its_own() {
        let mut doc = LayoutDocument::default();
        apply(&mut doc, "read", &mut DockLayout::default(), true);
        let mut edit = two_bars();
        apply(&mut doc, "edit", &mut edit, true);
        assert_eq!(edit.right.columns[0].stacks.len(), 1);
    }
}
