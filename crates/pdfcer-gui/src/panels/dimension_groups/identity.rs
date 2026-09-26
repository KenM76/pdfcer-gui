//! # `panels::dimension_groups::identity` — renaming a group, and removing one
//!
//! ## What this closes
//!
//!
//! > *"A group cannot yet be renamed or removed — pdfcer's editing engine has no
//! > command for either … Both are requested."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/dimension_groups/identity.md`.

use egui::Ui;
use pdfcer_core::dimension::{DEFAULT_GROUP_ID, DimensionModel, Group, GroupId};
use pdfcer_core::edit::GroupDeletion;

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::text::dimension_groups as t;

/// The region the rename field publishes.
pub const REGION_RENAME: &str = "dimension-groups.rename"; // ui-text-exempt: trace region name, never displayed
/// The region the Delete button publishes.
pub const REGION_DELETE: &str = "dimension-groups.delete"; // ui-text-exempt: trace region name, never displayed

impl super::DimensionGroupsUi {
    /// The selected group's name and its removal.
    pub(super) fn identity(
        &mut self,
        ui: &mut Ui,
        model: &DimensionModel,
        group: &Group,
        actions: &mut Vec<Action>,
    ) {
        self.rename_row(ui, group, actions);
        self.delete_row(ui, model, group, actions);
    }

    /// The name field and its Rename button.
    fn rename_row(&mut self, ui: &mut Ui, group: &Group, actions: &mut Vec<Action>) {
        let draft = self.rename_draft_for(group);
        let mut typed = draft.clone();
        let mut commit = false;
        ui.horizontal_wrapped(|ui| {
            ui.label(t::rename_label());
            // escape-disposition: keeps-draft — written back below whatever
            // happened, so the key leaves the box and leaves the words in it.
            let response = ui.add(egui::TextEdit::singleline(&mut typed).desired_width(160.0));
            crate::diag::ui_rect(REGION_RENAME, response.rect);

            let trimmed = typed.trim();
            // Absent when there is nothing to do, rather than greyed: a Rename
            // button beside a field holding the group's current name is a
            // control whose only possible effect is an undo entry the operator
            // did not earn. The field alone reads as "this is the name", which
            // is true.
            if !trimmed.is_empty() && trimmed != group.name {
                commit = ui.button(t::rename_button()).clicked()
                    // Enter commits too, because a name is a thing people type
                    // and then press Enter on. `lost_focus` alone would make
                    // that keystroke do nothing and the operator reach for the
                    // mouse they had just stopped using.
                    || (response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            }
        });
        // Written back whatever happened, so the next frame redraws what the
        // operator sees rather than what they last committed.
        self.rename = Some((group.id, typed.clone()));
        if commit {
            let name = typed.trim().to_owned();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed. LENGTH,
                // not the text: a group name is the operator's own words about
                // their drawing, and the trace is a file a harness keeps.
                format!(
                    "dimension-group-rename id={} chars={}",
                    group.id.0,
                    name.len()
                )
            });
            actions.push(Action::Dimension(DimensionAction::RenameGroup {
                group: group.id,
                name,
            }));
            // Cleared so the next frame re-seeds from the document. Without
            // this the field would hold the typed name against a group that now
            // has it, and the Rename button would correctly vanish — which is
            // the right end state reached by luck rather than by design.
            self.rename = None;
        }
    }

    /// The Delete button, and the question a populated group has to answer.
    fn delete_row(
        &mut self,
        ui: &mut Ui,
        model: &DimensionModel,
        group: &Group,
        actions: &mut Vec<Action>,
    ) {
        if group.id == DEFAULT_GROUP_ID {
            // R9: the engine refuses, so the control is ABSENT rather than
            // offered and declined — the same treatment its layer switch gets
            // two sections down, for the same reason.
            ui.weak(t::delete_default_group());
            return;
        }

        let members = model.member_count(group.id);
        if members == 0 {
            let response = ui.button(t::delete_button());
            crate::diag::ui_rect(REGION_DELETE, response.rect);
            if response.clicked() {
                self.raise_delete(group.id, GroupDeletion::Refuse, 0, actions);
            }
            return;
        }

        // --- populated: ask, then offer a button that will succeed ---------
        ui.label(t::delete_needs_a_home(members));
        ui.weak(t::delete_move_changes_labels());
        ui.weak(t::delete_cannot_remove_members());

        // Every other group is a candidate. `DEFAULT_GROUP_ID` is included and
        // is the seed, because it is the one group guaranteed to exist and the
        // one an operator with no better idea means by "somewhere".
        let mut destination = self.delete_destination.unwrap_or(DEFAULT_GROUP_ID);
        if destination == group.id {
            destination = DEFAULT_GROUP_ID;
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(t::delete_move_to());
            egui::ComboBox::from_id_salt("dimension-group-delete-destination")
                .selected_text(
                    model
                        .group(destination)
                        .map_or_else(String::new, |g| g.name.clone()),
                )
                .show_ui(ui, |ui| {
                    for other in model.groups() {
                        if other.id == group.id {
                            continue;
                        }
                        ui.selectable_value(&mut destination, other.id, &other.name);
                    }
                });
            let response = ui.button(t::delete_button());
            crate::diag::ui_rect(REGION_DELETE, response.rect);
            if response.clicked() {
                self.raise_delete(
                    group.id,
                    GroupDeletion::Reassign(destination),
                    members,
                    actions,
                );
            }
        });
        self.delete_destination = Some(destination);
    }

    /// Raise the deletion, and put the selection somewhere that will still
    /// exist.
    fn raise_delete(
        &mut self,
        group: GroupId,
        policy: GroupDeletion,
        members: usize,
        actions: &mut Vec<Action>,
    ) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "dimension-group-delete id={} members={members} policy={policy:?}",
                group.0
            )
        });
        actions.push(Action::Dimension(DimensionAction::DeleteGroup {
            group,
            policy,
        }));
        self.selected = Some(DEFAULT_GROUP_ID);
        self.rename = None;
        self.delete_destination = None;
    }

    /// The rename draft for `group`, seeded from the document when it is stale.
    fn rename_draft_for(&self, group: &Group) -> String {
        match &self.rename {
            Some((id, text)) if *id == group.id => text.clone(),
            _ => group.name.clone(),
        }
    }
}
