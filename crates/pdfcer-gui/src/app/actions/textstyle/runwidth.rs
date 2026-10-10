//! Fit one text run to a width typed in points (`G038`), on a page text
//! object or a text object inside a placed drawing (`G158`).
//!
//! One engine call, one undo entry: `EditSession::set_text_run_width` (or its
//! `_in_form` twin) always holds what follows the run
//! (`FollowerDisposition::Pin`), and that is said off-canvas. An in-form fit
//! changes every place the drawing is drawn, so the shared-content remedy is
//! added when it is drawn more than once. Refusals share the restyle family's
//! catalog.

use pdfcer_core::text_edit::{FollowerDisposition, FormatError, FormatReport};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::canvas::target::TargetId;
use crate::text::status as t;

/// Set run `run` of the text object `target` on `page` to `width` points.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, target: TargetId, run: usize, width: f64) {
    let mut outcome: Option<String> = None;
    let in_form = target.is_leaf();
    super::super::apply::vector_edit_on_page(doc, "text-run-width", page, 1, |session| {
        let fitted = match target {
            TargetId::Object(object) => usize::try_from(object)
                .map_err(|_| FormatError::PageIndex(page))
                .and_then(|object| session.set_text_run_width(page, object, run, width))
                .map(|report| (report, None)),
            TargetId::Leaf(leaf) => usize::try_from(leaf)
                .map_err(|_| FormatError::PageIndex(page))
                .and_then(|leaf| session.set_text_run_width_in_form(page, leaf, run, width))
                .map(|o| {
                    let remedy = crate::text::unshare::remedy_if_shared(o.invocations, o.pages);
                    (o.report, remedy)
                }),
        };
        match fitted {
            Ok((report, remedy)) => Ok(notes_of(report, remedy)),
            Err(error) => {
                decline::record_text_style(super::refusal_of(&error));
                outcome = Some(error.to_string());
                Err(FormatError::NoOp)
            }
        }
    });
    let detail = outcome.unwrap_or_else(|| "applied".to_owned());
    let index = target.page_object_index().or(target.leaf_index());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "text-run-width page={page} object={} in_form={in_form} run={run} width={width} \
             detail={detail}",
            index.map_or_else(|| "?".to_owned(), |i| i.to_string()),
        )
    });
}

/// The status-line notes for a fit: the held-follower sentence, the engine's
/// disclosures, then the shared-drawing remedy.
fn notes_of(report: FormatReport, remedy: Option<String>) -> Vec<String> {
    let mut notes = Vec::new();
    if matches!(report.disposition, FollowerDisposition::Pin) && report.h_scale_change.is_some() {
        notes.push(t::text_run_width_held().to_owned());
    }
    notes.extend(report.disclosures);
    notes.extend(remedy);
    notes
}
