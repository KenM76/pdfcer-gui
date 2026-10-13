//! # `app::actions::movemixed` — content and form-field widgets moved as one step
//!
//! Commits [`VectorAction::MoveMixed`](super::VectorAction::MoveMixed): what
//! Align and Distribute raise when the selection holds form fields. The engine
//! moves several objects in one command but a widget only one at a time, so
//! this is one `move_objects_each` (when there is content) plus one
//! `move_widget` per widget, then `coalesce_last` over every command pushed.
//! That workaround is filed as G189; a batch widget move retires it.
//!
//! ## A refused widget does not roll the others back
//!
//! The only public way to take a command back is `EditSession::undo`, which
//! leaves it on the Redo stack, where Redo would re-apply a partial move the
//! operator never saw. So what was applied stays applied, is still folded into
//! one undo step, and the status line says how many fields moved and why the
//! rest did not.

use pdfcer_core::edit::{CommandKind, EditError, EditSession};
use pdfcer_gui_base::docidentity::SelectedField;

use crate::app::state::OpenDoc;
use crate::text::panels::align as t;

/// Apply one [`VectorAction::MoveMixed`](super::VectorAction::MoveMixed).
pub(super) fn apply(
    doc: &mut OpenDoc,
    page: usize,
    objects: &[(usize, f64, f64)],
    in_form: bool,
    widgets: &[(SelectedField, f64, f64)],
    gesture: &'static str,
) {
    let n = objects.len() + widgets.len();
    let mut moved = (0_usize, 0_usize);
    let mut folded = true;
    super::apply::vector_edit_on_page(doc, "move-mixed", page, n, |session| {
        let (said, report) = commit(session, page, objects, in_form, widgets)?;
        moved = (report.objects, report.widgets);
        folded = report.folded;
        Ok::<_, EditError>(said)
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        let asked: Vec<String> = widgets
            .iter()
            .map(|(f, dx, dy)| format!("{}#{}:{dx:.1},{dy:.1}", f.field, f.widget))
            .collect();
        format!(
            "move-mixed-applied gesture={gesture} page={page} objects={}/{} widgets={}/{} \
             one_step={folded} moves={}",
            moved.0,
            objects.len(),
            moved.1,
            widgets.len(),
            asked.join(";")
        )
    });
}

/// What [`commit`] managed.
struct Report {
    objects: usize,
    widgets: usize,
    folded: bool,
}

/// Run the verbs and fold them. `Err` only when nothing at all was applied.
fn commit(
    session: &mut EditSession,
    page: usize,
    objects: &[(usize, f64, f64)],
    in_form: bool,
    widgets: &[(SelectedField, f64, f64)],
) -> Result<(Vec<String>, Report), EditError> {
    let mut said = Vec::new();
    let mut pushed = 0_usize;
    if !objects.is_empty() {
        if in_form {
            said.extend(
                session
                    .move_objects_each_in_form(page, objects)?
                    .disclosures,
            );
        } else {
            said.extend(session.move_objects_each(page, objects)?);
        }
        pushed += 1;
    }
    let mut moved_widgets = 0_usize;
    let mut refusal: Option<EditError> = None;
    for (field, dx, dy) in widgets {
        match session.move_widget(&field.field, field.widget, *dx, *dy) {
            Ok(outcome) => {
                moved_widgets += 1;
                pushed += 1;
                if outcome.siblings_left_behind > 0 {
                    said.push(crate::text::forms::widget_siblings_unmoved(
                        outcome.siblings_left_behind,
                    ));
                }
            }
            Err(e) => {
                refusal.get_or_insert(e);
            }
        }
    }
    if pushed == 0 {
        // Nothing applied: report the first refusal as the edit's own.
        if let Some(e) = refusal {
            return Err(e);
        }
    }
    let kind = if objects.is_empty() {
        CommandKind::MoveWidget
    } else {
        CommandKind::MoveObject
    };
    let folded = session.coalesce_last(pushed, kind);
    if let Some(e) = refusal {
        said.push(t::fields_partly_moved(
            moved_widgets,
            widgets.len(),
            &e.to_string(),
        ));
    }
    if !folded {
        said.push(t::not_one_undo().to_owned());
    }
    Ok((
        said,
        Report {
            objects: if objects.is_empty() { 0 } else { objects.len() },
            widgets: moved_widgets,
            folded,
        },
    ))
}
