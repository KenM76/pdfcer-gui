//! # `panels::dimension_groups::identity` — renaming a group, and removing one
//!
//! ## What this closes
//!
//! The two controls this surface shipped **without**, on 2026-08-18, with a
//! sentence where they should have been:
//!
//! > *"A group cannot yet be renamed or removed — pdfcer's editing engine has no
//! > command for either … Both are requested."*
//!
//! They were requested that day and shipped the next
//! (`EditSession::rename_dimension_group`, `delete_dimension_group_with`), so
//! the sentence is gone and these are the controls that replace it.
//!
//! ## ★ Deleting a populated group is the ORPHAN question, and it is asked
//!
//! `pdfcer-core` refuses a populated group by default and puts the **count** in
//! the refusal — `EditError::DimensionGroupNotEmpty { id, members }` — and its
//! reply says exactly why the count is there:
//!
//! > *"this group is not empty"* and *"this group holds forty dimensions"*
//! > prompt different decisions from an operator, and only you can put that
//! > question in front of them.
//!
//! So the panel does. Pressing Delete on a populated group does not delete it
//! and does not refuse it: it **asks where the members should go**, and only
//! then offers a button that will succeed.
//!
//! Two facts travel with that question, and both are stated before the press
//! rather than discovered afterwards:
//!
//! - **The members are re-measured.** A ce dimension's label is derived from
//!   its group's scale, unit and format, so moving them changes the numbers
//!   they print. The engine's measured example is `70.6 mm` becoming `2.00 m`
//!   for the same geometry.
//! - **pdfcer will not delete the dimensions with the group**, and that is the
//!   engine's decision with a reason rather than a gap: doing it inside the
//!   group verb would be a second implementation of `delete_dimension`'s
//!   `/Annots` removal, and looping the existing verb would make undo take one
//!   press per member and be able to stop halfway.
//!
//! ## ★ Why the rename draft carries its own `GroupId`
//!
//! A half-typed name must not follow the operator to a different row. Holding
//! the id **with** the text makes a stale pair detectable, so selecting another
//! group re-seeds the field from the group actually on screen rather than
//! offering to rename *it* to a name meant for the last one.
//!
//! That is the same hazard `dialogs::scale` names for its own captured group —
//! *"a group picker that moved underneath an open dialog would let them type a
//! number for one group and commit it to another"* — one control smaller.

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
    ///
    /// ★ The selection move is not tidiness. `body` falls back to the default
    /// group when the selected one has gone, which is correct and arrives **one
    /// frame late** — for that frame the lower half of the panel would draw
    /// against a group the document no longer has. Moving it here means the
    /// operator never sees the flicker, and the fallback stays as the guard it
    /// is for every path that is not this one.
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
    ///
    /// Stale means *"held for a different group"* — see the module header. It
    /// is also what makes the field follow an **undo**: a rename undone bumps
    /// the epoch and changes `group.name`, and the next frame's draft is
    /// re-seeded because... it is not, and this is the honest limitation.
    ///
    /// ★ **The draft does NOT follow the document while it is being typed**,
    /// deliberately, and that differs from `panels::docprops`'s
    /// epoch-reseed. The difference is what the two fields are: a metadata box
    /// commits on focus loss and is otherwise idle, so re-seeding it costs
    /// nothing; a rename box is typed into and then committed by a button, and
    /// an epoch bump from an unrelated edit — placing a dimension, moving a
    /// page — would wipe a half-typed name mid-keystroke.
    ///
    /// The narrow cost is that undoing a rename leaves the old name in the box
    /// until the operator selects another group and comes back. The button
    /// re-appears, because the draft now differs from the document, so the
    /// state is legible rather than wrong.
    fn rename_draft_for(&self, group: &Group) -> String {
        match &self.rename {
            Some((id, text)) if *id == group.id => text.clone(),
            _ => group.name.clone(),
        }
    }
}
