//! Fit one text run to a width typed in points (`G038`).
//!
//! One engine call, one undo entry: `EditSession::set_text_run_width` always
//! holds what follows the run (`FollowerDisposition::Pin`), and that is said
//! off-canvas. Refusals share the restyle family's catalog.

use pdfcer_core::text_edit::{FollowerDisposition, FormatError};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::status as t;

/// Set run `run` of paint-order object `object` on `page` to `width` points.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, object: usize, run: usize, width: f64) {
    let mut outcome: Option<String> = None;
    super::super::apply::vector_edit_on_page(
        doc,
        "text-run-width",
        page,
        1,
        |session| match session.set_text_run_width(page, object, run, width) {
            Ok(report) => {
                let mut notes = Vec::new();
                if matches!(report.disposition, FollowerDisposition::Pin)
                    && report.h_scale_change.is_some()
                {
                    notes.push(t::text_run_width_held().to_owned());
                }
                notes.extend(report.disclosures);
                Ok(notes)
            }
            Err(error) => {
                decline::record_text_style(super::refusal_of(&error));
                outcome = Some(error.to_string());
                Err(FormatError::NoOp)
            }
        },
    );
    let detail = outcome.unwrap_or_else(|| "applied".to_owned());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "text-run-width page={page} object={object} run={run} width={width} detail={detail}"
        )
    });
}
