//! # `text::status::formdelete` — the worded decline for a refused FORM-FIELD
//! delete
//!
//! One sentence, and it is in its own file for [`super`]'s stated reason: a
//! catalog area is keyed by the consumer it serves, and this one's consumer is
//! `pdfcer_gui::app::status::decline`'s [`Declined::FieldDeleteRefused`] arm alone.
//! R2 (no file over 1,500 lines) is what forced the split; the subject boundary
//! is what decided where.
//!
//! [`Declined::FieldDeleteRefused`]: crate::app::status::decline
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/status/formdelete.md`.

/// **A form field, or one of its boxes, was asked to be deleted on a document
/// whose form structure is frozen** — `EditSession::deletion_refusal` answered
/// `Some`.
#[must_use]
pub const fn field_delete_declined_structural() -> &'static str {
    "This document does not allow form fields to be removed, so nothing was deleted. Its \
     structure is fixed; the values in it can still be filled in and changed."
}
