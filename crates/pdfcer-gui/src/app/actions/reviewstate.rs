//! # `app::actions::reviewstate` — recording a comment's review status
//!
//! One verb: [`record`], which is
//! [`pdfcer_core::edit::EditSession::add_review_state`] behind the edit funnel.
//! Raised by [`crate::app::actions::Action::RecordReviewState`], which is
//! raised by [`crate::panels::comments::reviewstate`]'s *Record status*
//! chooser, and by nothing else. [`RecordStatus`] is that action's payload, and
//! its doc carries why the verb is not an `AnnotAction`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/reviewstate.md`.

use crate::app::state::OpenDoc;
use crate::text::reviewstate as t;

pub use pdfcer_gui_base::commentreviewstate::RecordStatus;

/// **Append this review status**, as one undoable command.
pub(super) fn record(status: RecordStatus, doc: &mut OpenDoc, author: &str) {
    // Taken before the closure, because `pdf_date_utc` can fail (a clock
    // before the epoch) and `add_review_state` takes an `Option<&str>` for
    // exactly that: `/M` is optional on a markup annotation, and writing a
    // fabricated date would be worse than writing none.
    let modified = crate::app::clock::pdf_date_utc();
    let signed = !author.is_empty();
    super::apply::vector_edit(doc, "record-review-state", 0, 1, |session| {
        session
            .add_review_state(status.id, status.state, author, modified.as_deref())
            .map(|added| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "record-review-state-applied target={} state={} model={} \
                             status={} attached={} depth={} signed={signed}",
                        added.target_id.num,
                        added.state.as_str(),
                        added.state.model(),
                        added.state_id.num,
                        added.attached_to.num,
                        added.chain_depth,
                    )
                });
                // The unsigned sentence goes SECOND, and the order is the
                // decision: the first line is the one an operator reads if they
                // read only one, and between "what was recorded" and "it was
                // recorded without a name", only the first answers the question
                // they just asked. The second is a consequence they will meet
                // later, and it is worded so it still makes sense read alone.
                let mut said = vec![t::status_recorded(
                    t::state_name(added.state),
                    added.chain_depth,
                )];
                if !signed {
                    said.push(t::status_recorded_unsigned().to_owned());
                }
                said
            })
    });
}
