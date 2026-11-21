//! # `app::actions::forms::radiogroup` — putting a radio button into a group
//!
//! Properties ▸ *Radio group* moves the selected button into the named group,
//! or into a new one. (Another button pasted into its group is
//! `super::paste`'s, by `paste_field`.)
//!
//! Both are the engine's `move_widget_to_field`, which picks an on-state no
//! member uses (§12.7.3.2: a shared name is what makes radios exclusive) and
//! creates the target group when no field bears its name.

use pdfcer_core::forms::Field;

use crate::app::state::{OpenDoc, SelectedField};

/// Every export value the radio group `fqn` uses, empty when there is none.
#[must_use]
pub fn exports(doc: &OpenDoc, fqn: &str) -> Vec<String> {
    with_field(doc, fqn, |f| {
        f.widgets
            .iter()
            .flat_map(|w| w.on_states.iter())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect()
    })
    .unwrap_or_default()
}

/// **Move the button `from`'s widget `widget` into the radio group `to`**,
/// creating `to` when no field bears that name. One undo step.
///
/// It is one `move_widget_to_field`: the widget keeps its object and its
/// look, and the engine picks a free on-state.
pub(super) fn move_to(doc: &mut OpenDoc, from: &SelectedField, to: &str) {
    let to = to.trim();
    if to.is_empty() || to == from.field {
        return;
    }
    regroup(doc, from, to);
}

fn regroup(doc: &mut OpenDoc, from: &SelectedField, to: &str) {
    let page = from.page;
    let (field, widget) = (from.field.clone(), from.widget);
    let before = doc.edit_epoch;
    crate::app::actions::apply::vector_edit(doc, "radio-move", page, 1, |session| {
        let done = session.move_widget_to_field(&field, widget, to)?;
        let export = done.on_state.clone().unwrap_or_default();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "radio-move-applied group={to} export={export} page={page} \
                 engine=move_widget_to_field source_removed={} orphaned={} \
                 target_created={}",
                done.source_removed, done.action_targets_orphaned, done.target_created
            )
        });
        let mut lines = vec![crate::text::radiogroup::moved(to, &export)];
        if done.target_created {
            lines.push(crate::text::radiogroup::created(to));
        }
        lines.extend(done.disclosures);
        if done.action_targets_orphaned > 0 {
            lines.push(crate::text::forms::form_field_actions_orphaned(
                done.action_targets_orphaned,
            ));
        }
        Ok::<_, pdfcer_core::edit::EditError>(lines)
    });
    select_last(doc, before, to, page);
}

/// Leave the new member selected, if the edit happened.
fn select_last(doc: &mut OpenDoc, before: u64, group: &str, page: usize) {
    if doc.edit_epoch == before {
        return;
    }
    let widget = with_field(doc, group, |f| f.widgets.len().saturating_sub(1)).unwrap_or(0);
    doc.selected_field = Some(SelectedField {
        field: group.to_owned(),
        widget,
        page,
    });
}

fn with_field<T>(doc: &OpenDoc, fqn: &str, read: impl FnOnce(&Field) -> T) -> Option<T> {
    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view)?;
    form.field_by_name(fqn).map(read)
}
