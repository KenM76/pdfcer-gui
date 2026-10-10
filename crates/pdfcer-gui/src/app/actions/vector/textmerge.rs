//! Commit `VectorAction::MergeTextRuns`: join consecutive runs of one text
//! object, on the page or inside a placed drawing (`G158`), as one undo entry.
//!
//! A page object goes to `EditSession::merge_text_runs`, a form leaf to
//! `merge_text_runs_in_form`; the in-form merge shows wherever the drawing is
//! drawn, so the shared-content remedy is added when that is more than once.
//! When the engine discloses nothing but the run's width changed, the width
//! sentence says so.

use pdfcer_core::text_edit::{FormatError, MergeOptions, MergeReport};

use crate::app::actions::apply::vector_edit_on_page;
use crate::app::state::OpenDoc;
use crate::canvas::target::TargetId;
use crate::text::runmerge::{RunMergeRefusal, width_changed};

/// Merge `runs` (ascending, consecutive) of the text object `target` on `page`.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, target: TargetId, runs: &[usize]) {
    let in_form = target.is_leaf();
    let index = target.page_object_index().or(target.leaf_index());
    vector_edit_on_page(doc, "merge-text-runs", page, runs.len(), |session| {
        let opts = MergeOptions::default();
        let merged = match (target, index) {
            (TargetId::Object(_), Some(object)) => session
                .merge_text_runs(page, object, runs, &opts)
                .map(|report| (report, None)),
            (TargetId::Leaf(_), Some(leaf)) => session
                .merge_text_runs_in_form(page, leaf, runs, &opts)
                .map(|o: pdfcer_core::edit::FormTextOutcome<MergeReport>| {
                    let remedy = crate::text::unshare::remedy_if_shared(o.invocations, o.pages);
                    (o.report, remedy)
                }),
            (_, None) => Err(FormatError::PageIndex(page)),
        };
        merged
            .inspect_err(|e| {
                crate::app::status::decline::record_run_merge(RunMergeRefusal::of_format(e));
            })
            .map(|(report, remedy)| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "merge-text-runs-applied page={page} object={} in_form={in_form} \
                         merged={} scale={:?}",
                        index.unwrap_or_default(),
                        report.runs_merged,
                        report.h_scale_change,
                    )
                });
                said_of(report, remedy)
            })
    });
}

/// The status-line notes for a merge: the engine's disclosures (or the width
/// sentence when it has none and the width changed), then the remedy.
fn said_of(report: MergeReport, remedy: Option<String>) -> Vec<String> {
    let mut said = report.disclosures;
    if let (Some((before, after)), true) = (report.h_scale_change, said.is_empty()) {
        said.push(width_changed(before, after));
    }
    said.extend(remedy);
    said
}
