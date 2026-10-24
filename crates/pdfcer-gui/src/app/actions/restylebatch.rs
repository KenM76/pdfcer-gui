//! # `app::actions::restylebatch` — one restyle gesture, one engine command
//!
//! Every show operator a restyle covers goes to `EditSession::format_texts` in
//! one call through the funnel: one plan of the page, one undo entry, one
//! re-render. The requests arrive in `pin::operators_of_runs` order, per
//! stream with span start descending, which is the order the engine applies
//! them in.
//!
//! A batch the engine must apply one request at a time (a decoration, a new
//! font or opacity resource, a synthesised slant) is refused above the undo
//! depth with `TooManyForOneUndo`. Only then is the gesture split into calls
//! of that depth, each folded into the first with `coalesce_last`, so it is
//! still one undo entry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/restylebatch.md`.

use std::time::Instant;

use pdfcer_core::edit::{CommandKind, EditSession};
use pdfcer_core::settings::StylePolicy;
use pdfcer_core::text_edit::{
    FormatBatch, FormatBatchError, FormatError, FormatOptions, FormatRequest,
};

use super::textstyle::{emit_carried, refusal_of, report_notes, resweep, stop};
use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::status as t;

/// One restyle gesture: what it covers and how it was asked for.
pub(super) struct Gesture {
    pub page: usize,
    pub policy: StylePolicy,
    pub change: &'static str,
    /// How many runs the gesture covers.
    pub runs: usize,
    /// A run with glyphs would not pin, so the gesture stops after these.
    pub stopped: bool,
}

/// What the engine did with the gesture's requests.
#[derive(Default)]
struct Outcome {
    applied: usize,
    /// Every call took the engine's one-walk path.
    planned_once: bool,
    calls: usize,
    notes: Vec<String>,
    /// The engine's refusal, already recorded on the bar.
    failed: Option<String>,
}

/// Restyle with `requests`, all in one engine call where the engine allows.
pub(super) fn run(doc: &mut OpenDoc, gesture: &Gesture, requests: &[FormatRequest]) {
    let began = Instant::now();
    let mut out = Outcome {
        planned_once: true,
        ..Outcome::default()
    };
    let (page, policy) = (gesture.page, gesture.policy);
    super::apply::vector_edit(doc, "format-text", page, requests.len(), |session| {
        apply_all(session, page, policy, requests, &mut out);
        if out.applied == 0 {
            return Err(FormatError::NoOp);
        }
        Ok(out.notes.clone())
    });
    let wall_ms = began.elapsed().as_millis();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "text-style-batch page={page} ops={} applied={} calls={} planned_once={} wall_ms={wall_ms}",
            requests.len(),
            out.applied,
            out.calls,
            u8::from(out.planned_once),
        )
    });
    summarise(doc, gesture, &out);
    resweep(doc, page);
}

/// Every request through `format_texts`, split only when the engine says the
/// batch is longer than one undo entry can hold.
fn apply_all(
    session: &mut EditSession,
    page: usize,
    policy: StylePolicy,
    requests: &[FormatRequest],
    out: &mut Outcome,
) {
    let options = FormatOptions::default().with_style_policy(policy);
    // The engine's one-at-a-time path holds one undo entry per request until
    // the fold, so it refuses more than the undo depth at once.
    let Err(FormatBatchError::TooManyForOneUndo { limit, .. }) =
        call(session, page, policy, &options, requests, out)
    else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-style-chunked ops={} limit={limit}", requests.len())
    });
    out.notes
        .push(t::text_style_in_steps(requests.len(), limit));
    for chunk in requests.chunks(limit.max(1)) {
        if call(session, page, policy, &options, chunk, out).is_err() {
            return;
        }
        if out.calls > 1 && !session.coalesce_last(2, CommandKind::FormatText) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "text-style-fold-failed".to_owned()
            });
        }
    }
}

/// One `format_texts` call; its notes and counts go into `out`, and a refusal
/// of one request is recorded on the bar.
fn call(
    session: &mut EditSession,
    page: usize,
    policy: StylePolicy,
    options: &FormatOptions,
    requests: &[FormatRequest],
    out: &mut Outcome,
) -> Result<(), FormatBatchError> {
    match session.format_texts(requests, options) {
        Ok(batch) => {
            out.calls += 1;
            absorb(page, policy, batch, out);
            Ok(())
        }
        Err(FormatBatchError::Request { index, error }) => {
            decline::record_text_style(refusal_of(&error));
            out.failed = Some(format!("request {index}: {error}")); // ui-text-exempt: trace detail, never displayed
            Err(FormatBatchError::Request { index, error })
        }
        Err(other) => {
            if !matches!(other, FormatBatchError::TooManyForOneUndo { .. }) {
                decline::record_text_style(t::TextStyleRefusal::Other);
                out.failed = Some(other.to_string());
            }
            Err(other)
        }
    }
}

/// A batch's reports as the operator's notes, each sentence once.
fn absorb(page: usize, policy: StylePolicy, batch: FormatBatch, out: &mut Outcome) {
    out.applied += batch.reports.len();
    out.planned_once &= batch.planned_once;
    let said = batch
        .reports
        .into_iter()
        .flat_map(|report| report_notes(page, policy, report))
        .chain(batch.disclosures);
    for note in said {
        if !out.notes.contains(&note) {
            out.notes.push(note);
        }
    }
}

/// The traces and disclosures the gesture owes, once it is over.
fn summarise(doc: &mut OpenDoc, gesture: &Gesture, out: &Outcome) {
    let (page, applied, runs) = (gesture.page, out.applied, gesture.runs);
    if let Some(detail) = &out.failed {
        if applied > 0 {
            decline::record_text_style(t::TextStyleRefusal::PartOnly);
            emit_carried(doc, page, applied, runs, &out.notes);
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("text-style-declined page={page} applied={applied} runs={runs} detail={detail}")
        });
        return;
    }
    emit_carried(doc, page, applied, runs, &out.notes);
    if gesture.stopped {
        stop(doc, applied, t::TextStyleRefusal::Unpinnable);
        return;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            // `applied` counts show operators and `runs` counts runs.
            "text-style-applied page={page} change={} applied={applied} runs={runs}",
            gesture.change
        )
    });
}
