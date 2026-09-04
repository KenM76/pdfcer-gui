//! # `text::status::formdelete` — the worded decline for a refused FORM-FIELD
//! delete
//!
//! One sentence, and it is in its own file for [`super`]'s stated reason: a
//! catalog area is keyed by the consumer it serves, and this one's consumer is
//! `crate::app::status::decline`'s [`Declined::FieldDeleteRefused`] arm alone.
//! R2 (no file over 1,500 lines) is what forced the split; the subject boundary
//! is what decided where.
//!
//! [`Declined::FieldDeleteRefused`]: crate::app::status::decline

/// **A form field, or one of its boxes, was asked to be deleted on a document
/// whose form structure is frozen** — `EditSession::deletion_refusal` answered
/// `Some`.
///
/// # ★★★ Why a sentence exists for a state every control already withholds
///
/// It should be unreachable. `panels::properties::formfield::refuses_delete` is
/// asked by the panel's two delete buttons, by the condition behind the
/// `canvas.field` menu's Delete, by the Delete key's rung 0 and by
/// `app::dispatch::format`'s arm — four doors, one question. What is left is
/// the residue a gate cannot cover: a **chord** bound to `format.delete`
/// (a chord consults no `visible_when`), a condition gone stale within a frame,
/// and any engine guard the query does not forecast.
///
/// Before 2026-08-29 that residue was silence —
/// `app::actions::apply::vector_edit`'s `Err` arm traced and said nothing (it
/// words an un-categorised floor since O116, which names no gate) — and
/// the verb had *already cleared* `doc.selected_field`, so the press also took
/// away the panel sentence below that was explaining the refusal. **A refusal
/// must be a sentence, never a silence**, and least of all a silence that
/// destroys its own explanation.
///
/// # ★★ Why the wording is not
/// [`crate::text::panels::formfield::delete_refused`]'s, when the fact is the
/// same
///
/// That one is a **standing description** drawn in a panel from the moment the
/// field is selected: *"This document does not allow form fields to be
/// removed."* This one is a **decline** — it reports that a gesture just
/// happened and did not take effect — and `app::status::decline`'s header
/// insists the two speech acts must not wear the same words in the same place.
/// So this says *what you just pressed did nothing, and why*, and the panel
/// says *what this document is*.
///
/// ★ Like [`flatten_declined_certified`] it names **which gate refused**,
/// because deletion and filling take different ones: on the ordinary shape —
/// a certified fillable form at `/P 2` — §12.8.2.2 Table 257 permits filling
/// and forbids restructuring, so an operator who has just typed a value into
/// the very box they are trying to remove would otherwise conclude the program
/// is broken.
///
/// ★ And like it, it offers **no way round**. There is one — remove the
/// signature — and pdfcer will not suggest defeating a certification as a
/// workaround for a convenience.
#[must_use]
pub const fn field_delete_declined_structural() -> &'static str {
    "This document does not allow form fields to be removed, so nothing was deleted. Its \
     structure is fixed; the values in it can still be filled in and changed."
}
