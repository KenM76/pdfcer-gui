//! # `app::actions::deleteparts` — parts of placed drawings deleted, with page objects beside them, as one step
//!
//! Commits [`VectorAction::DeleteParts`](super::VectorAction::DeleteParts): a
//! Delete or Cut whose selection holds parts of placed drawings, alone or
//! beside page objects. The engine deletes many page objects in one command
//! but only the parts of one form per command, so this is one
//! `delete_objects` (when page objects are selected), then one
//! `delete_objects_in_form` per form, folded into one undo step.
//!
//! A part inside a selected drawing goes with the drawing and is not deleted
//! twice. Each form's parts are named by the form, the placement they were
//! selected at and their in-form index before anything is deleted, and found
//! again before each form's command: a delete renumbers the page's leaf list,
//! so a leaf index read before the first command is stale at the second. A
//! form selected at two placements is edited once.
//!
//! A refusal does not roll the others back: what was deleted stays deleted,
//! folded into one undo step, and the status line says the rest was refused.

use pdfcer_core::edit::{CommandKind, EditError, EditSession};
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::{FormLeaf, Matrix};

use super::foldeach::{OnRefusal, fold_each};
use crate::app::state::OpenDoc;
use crate::text::deleting as t;

/// The selected parts of one form, addressed so a renumbering cannot move them.
struct FormParts {
    form: ObjId,
    /// Where on the page the parts were selected; a form drawn twice has the
    /// same parts at each placement, and the engine edits one placement's.
    placement: Matrix,
    /// Indices into the form's own object list, unique.
    indices: Vec<usize>,
}

/// One engine command of the delete.
enum Step {
    Objects(Vec<usize>),
    Form(FormParts),
}

/// What [`commit`] managed, for the trace.
#[derive(Default)]
struct Report {
    objects: usize,
    parts: usize,
    forms: usize,
    of_forms: usize,
    carried: usize,
    folded: bool,
}

/// Apply one [`VectorAction::DeleteParts`](super::VectorAction::DeleteParts).
/// `label` names the funnel's trace line.
pub(super) fn apply(
    doc: &mut OpenDoc,
    label: &str,
    page: usize,
    objects: &[usize],
    leaves: &[usize],
) {
    if objects.is_empty() && leaves.is_empty() {
        return;
    }
    if !objects.is_empty() {
        hold_erased(doc, objects);
    }
    let mut report = Report::default();
    super::apply::vector_edit_on_page(doc, label, page, objects.len() + leaves.len(), |s| {
        let (said, done) = commit(s, page, objects, leaves)?;
        report = done;
        Ok::<_, EditError>(said)
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "{label}-applied page={page} objects={}/{} parts={}/{} forms={}/{} carried={} \
             one_step={}",
            report.objects,
            objects.len(),
            report.parts,
            leaves.len() - report.carried,
            report.forms,
            report.of_forms,
            report.carried,
            report.folded,
        )
    });
}

/// Keep the deleted objects' footprint on screen as erased until the page
/// redraws, so a slow redraw does not read as a delete that did nothing.
///
/// Both halves are ordered against the commit. The footprint is read first,
/// because the page model is keyed on the edit epoch the commit bumps. The
/// hold is taken first, because it stamps the epoch as it is when called and
/// reads "epoch unchanged" past 250 ms as "the engine refused": a hold taken
/// after the bump expires in a quarter of a second, and a refused delete
/// correctly drops it.
pub(super) fn hold_erased(doc: &mut OpenDoc, objects: &[usize]) {
    let preview = doc
        .page_objects()
        .and_then(|provider| crate::canvas::shapes::erased(&provider, objects));
    if let Some(preview) = preview {
        doc.hold_preview(preview);
    }
}

/// Run the commands and fold them. `Err` only when nothing was deleted.
fn commit(
    session: &mut EditSession,
    page: usize,
    objects: &[usize],
    leaves: &[usize],
) -> Result<(Vec<String>, Report), EditError> {
    let model = session.page_objects(page)?;
    let (kept, carried) =
        crate::panels::align::leftout::uncarried(&model, objects, leaves.to_vec());
    let forms = forms_of(kept.iter().map(|&i| model.leaves.get(i).and_then(key)));
    let mut report = Report {
        carried,
        of_forms: forms.len(),
        ..Report::default()
    };
    let mut steps = Vec::with_capacity(forms.len() + 1);
    if !objects.is_empty() {
        steps.push(Step::Objects(objects.to_vec()));
    }
    steps.extend(forms.into_iter().map(Step::Form));
    let done = fold_each(
        session,
        steps.into_iter(),
        CommandKind::DeleteObject,
        OnRefusal::Continue,
        |s, step| match step {
            Step::Objects(objects) => {
                let said = s.delete_objects(page, &objects)?;
                report.objects = objects.len();
                Ok(Some(said))
            }
            Step::Form(parts) => {
                let leaves = find(s.page_objects(page)?.leaves.iter().map(key), &parts);
                if leaves.is_empty() {
                    return Ok(None);
                }
                let outcome = s.delete_objects_in_form(page, &leaves)?;
                report.parts += leaves.len();
                report.forms += 1;
                Ok(Some(outcome.disclosures))
            }
        },
    )?;
    let mut run = done.run;
    if run.pushed == 0
        && let Some(e) = run.failure.take()
    {
        return Err(e);
    }
    report.folded = done.folded;
    let mut said = run.told;
    if let Some(e) = &run.failure {
        said.push(t::partly_deleted(&e.to_string()));
    }
    if !done.folded {
        said.push(crate::text::markup::deleted_not_one_undo().to_owned());
    }
    Ok((said, report))
}

/// A leaf's address that survives renumbering: the form it is directly
/// inside, where that form is placed, and its index in the form's own list.
type Key = (ObjId, Matrix, usize);

fn key(leaf: &FormLeaf) -> Option<Key> {
    let form = *leaf.containment.last()?;
    Some((form, leaf.placement, leaf.form_object_index))
}

/// The selected leaves' keys grouped by form, in first-seen order, at the
/// placement the first of each form's was selected at.
fn forms_of(selected: impl Iterator<Item = Option<Key>>) -> Vec<FormParts> {
    let mut forms: Vec<FormParts> = Vec::new();
    for (form, placement, index) in selected.flatten() {
        match forms.iter_mut().find(|f| f.form == form) {
            Some(f) if !f.indices.contains(&index) => f.indices.push(index),
            Some(_) => {}
            None => forms.push(FormParts {
                form,
                placement,
                indices: vec![index],
            }),
        }
    }
    forms
}

/// Where `parts` now are in the page's leaf list, ascending.
fn find(leaves: impl Iterator<Item = Option<Key>>, parts: &FormParts) -> Vec<usize> {
    leaves
        .enumerate()
        .filter(|(_, k)| {
            k.is_some_and(|(form, placement, index)| {
                form == parts.form && placement == parts.placement && parts.indices.contains(&index)
            })
        })
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests;
