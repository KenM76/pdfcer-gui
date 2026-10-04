//! Commit `VectorAction::SplitTextLines`: plan the line cuts, then cut.
//!
//! Both engine calls run inside one funnel call, so the plan is read from the
//! same session state the cut applies to. The plan's inference disclosure (the
//! file does not record where its lines are) goes to the status line with the
//! piece count; one undo entry, `CommandKind::SplitTextObject`.

use pdfcer_core::vector::SplitGranularity;

use crate::app::state::OpenDoc;
use crate::text::runsplit::{RunSplitRefusal, split_into};

/// Split page `page`'s object `object` into one text object per line.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, object: usize) {
    super::apply::vector_edit_on_page(doc, "split-text-lines", page, 1, |session| {
        let refused = |e: &pdfcer_core::edit::EditError| {
            crate::app::status::decline::record_run_split(RunSplitRefusal::of_edit(e));
        };
        let (cuts, mut said) = session
            .text_object_split_plan(page, object, SplitGranularity::Line)
            .inspect_err(refused)?;
        let inferred = said.len();
        said.extend(
            session
                .split_text_object(page, object, &cuts)
                .inspect_err(refused)?,
        );
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "split-text-lines-applied page={page} object={object} cuts={} pieces={} \
                 disclosed={inferred}",
                cuts.len(),
                cuts.len() + 1,
            )
        });
        said.push(split_into(cuts.len() + 1));
        Ok::<_, pdfcer_core::edit::EditError>(said)
    });
}
