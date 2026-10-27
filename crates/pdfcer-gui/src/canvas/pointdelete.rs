//! # `canvas::pointdelete` — **Ctrl+Alt+click deletes the point under it** (O332)
//!
//! With the Points tool armed, one Ctrl+Alt+click on a shown point deletes it,
//! as Inkscape's node tool does. Two kinds of point answer:
//!
//! - **a markup shape's node** raises the plan *Remove this point* raises
//!   ([`annotnodes::menu::remove_node`]), so the engine's refusal of a shape
//!   that would fall below its minimum is the menu's refusal;
//! - **a page-content anchor** is selected and handed to the Delete key's
//!   ladder ([`deleting::subject`]), so every refusal and its sentence are the
//!   Delete key's.
//!
//! A Ctrl+Alt+click on no point is an ordinary click. Every outcome traces
//! `point-delete via=… outcome=…`.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{ClickHit, SelectionState};
use crate::canvas::{annotnodes, deleting};
use crate::panels::objects::provider::ObjectModelProvider;

/// `point-delete via=markup|anchor … outcome=raised|declined`.
pub const TRACE: &str = "point-delete"; // ui-text-exempt: diagnostic trace name

/// Delete the selected markup shape's node under `point` (canvas space).
/// `false` when no node of a selected shape is there.
pub fn markup(
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    point: egui::Pos2,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(index) = annotnodes::node_at(doc, map, selection, map.to_screen(point)) else {
        return false;
    };
    let action = annotnodes::menu::remove_node(doc, selection, index);
    let outcome = if action.is_some() {
        "raised"
    } else {
        "declined"
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!("{TRACE} via=markup node={index} outcome={outcome}")
    });
    actions.extend(action);
    true
}

/// Delete the page-content anchor `hit` names. `false` when the click hit no
/// anchor; the selection is then untouched.
pub fn anchor(
    selection: &mut SelectionState,
    page: usize,
    targets: Option<&ObjectModelProvider>,
    hit: ClickHit,
    epoch: u64,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(node) = hit.node else {
        return false;
    };
    selection.click_direct(page, hit, false);
    match deleting::subject(selection, page, targets) {
        Ok(subject) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                format!("{TRACE} via=anchor node={node} outcome=raised")
            });
            actions.push(deleting::action(subject).into());
        }
        Err(reason) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                format!("{TRACE} via=anchor node={node} outcome=declined")
            });
            deleting::decline(selection, reason, epoch, targets.is_some());
        }
    }
    true
}
