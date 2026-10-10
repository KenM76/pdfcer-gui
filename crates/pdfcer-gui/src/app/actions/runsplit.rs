//! Commit `VectorAction::SplitTextLines`: plan the line cuts, then cut, on a
//! page text object or a text object inside a placed drawing (`G158`).
//!
//! A page object's plan and cut run inside one funnel call, so the plan is
//! read from the same session state the cut applies to, and the plan's
//! inference disclosure (the file does not record where its lines are) goes to
//! the status line with the piece count. A form leaf has no engine plan: its
//! cuts come from `text_object_split_points` on the leaf's text, read from the
//! current object model before the cut, and the shell words the inference
//! itself. One undo entry, `CommandKind::SplitTextObject`.

use pdfcer_core::edit::EditError;
use pdfcer_core::vector::{SplitGranularity, VectorObject, text_object_split_points};

use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;
use crate::text::runsplit::{RunSplitRefusal, lines_inferred, split_into};

/// Split the text object `target` on `page` into one text object per line.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, target: TargetId) {
    match (target.page_object_index(), target.leaf_index()) {
        (Some(object), _) => page_object(doc, page, object),
        (None, Some(leaf)) => form_leaf(doc, page, target, leaf),
        (None, None) => {
            crate::app::status::decline::record_run_split(RunSplitRefusal::Stale);
        }
    }
}

fn refused(e: &EditError) {
    crate::app::status::decline::record_run_split(RunSplitRefusal::of_edit(e));
}

fn page_object(doc: &mut OpenDoc, page: usize, object: usize) {
    super::apply::vector_edit_on_page(doc, "split-text-lines", page, 1, |session| {
        let (cuts, mut said) = session
            .text_object_split_plan(page, object, SplitGranularity::Line)
            .inspect_err(refused)?;
        let inferred = said.len();
        said.extend(
            session
                .split_text_object(page, object, &cuts)
                .inspect_err(refused)?,
        );
        trace_applied(page, object, false, cuts.len(), inferred);
        said.push(split_into(cuts.len() + 1));
        Ok::<_, EditError>(said)
    });
}

fn form_leaf(doc: &mut OpenDoc, page: usize, target: TargetId, leaf: usize) {
    let planned = doc
        .page_objects()
        .and_then(|provider| match provider.object_for(target) {
            Some(VectorObject::Text(text)) => Some((
                text_object_split_points(text, SplitGranularity::Line),
                text.runs.len(),
            )),
            _ => None,
        });
    let Some((cuts, operators)) = planned else {
        crate::app::status::decline::record_run_split(RunSplitRefusal::Stale);
        return;
    };
    if cuts.is_empty() {
        crate::app::status::decline::record_run_split(RunSplitRefusal::NoLineBreaks);
        return;
    }
    super::apply::vector_edit_on_page(doc, "split-text-lines", page, 1, |session| {
        let outcome = session
            .split_text_object_in_form(page, leaf, &cuts)
            .inspect_err(refused)?;
        trace_applied(page, leaf, true, cuts.len(), 1);
        let mut said = vec![lines_inferred(cuts.len(), operators)];
        said.extend(outcome.disclosures);
        said.push(split_into(cuts.len() + 1));
        said.extend(crate::text::unshare::remedy_if_shared(
            outcome.invocations,
            outcome.pages,
        ));
        Ok::<_, EditError>(said)
    });
}

fn trace_applied(page: usize, object: usize, in_form: bool, cuts: usize, disclosed: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "split-text-lines-applied page={page} object={object} in_form={in_form} cuts={cuts} \
             pieces={} disclosed={disclosed}",
            cuts + 1,
        )
    });
}
