//! # `app::status::decline::record` — every writer of the decline slot
//!
//! **The recording half of [`super`].**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/decline/record.md`.

use super::{Declined, LAST};
use crate::canvas::zoom::ZoomOutcome;

/// Record what a framing zoom did, so the bar can say so if it declined.
pub(crate) fn record(outcome: ZoomOutcome) {
    let declined = Declined::of(outcome);
    LAST.with_borrow_mut(|slot| *slot = declined);
}

/// Record that a restyle of existing text refused, and why.
pub(crate) fn record_text_style(why: crate::text::status::TextStyleRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::TextStyle(why)));
}

/// Record that a flatten was refused by the document's certification.
pub(crate) fn record_flatten_certified() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::FlattenCertified));
}

/// Record that a field-group deletion **preview** was refused.
pub(crate) fn record_field_group_preview_refused() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::FieldGroupPreviewRefused));
}

/// Record that a field-group deletion was refused after confirmation.
/// See [`record_field_group_preview_refused`].
pub(crate) fn record_field_group_delete_refused() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::FieldGroupDeleteRefused));
}

/// Record that **one of the operator's own stamps could not be placed** —
/// [`Declined::CustomStampUnavailable`], `OPERATOR_REQUESTS.md` O172.
pub(crate) fn record_custom_stamp_unavailable(why: crate::text::stamps::CustomStampUnavailable) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::CustomStampUnavailable(why)));
}

/// Record that the **Points tool** was asked for in a mode that cannot change
/// page content — [`Declined::NodeToolNeedsEditMode`].
pub(crate) fn record_node_tool_needs_edit_mode() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::NodeToolNeedsEditMode));
}

/// Record that a **corner of a ce dimension** could not be added or taken
/// away — [`Declined::VertexEditRefused`].
pub(crate) fn record_vertex_edit_refused(why: crate::text::measure::VertexEditRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::VertexEditRefused(why)));
}

/// Record that a **form-field or widget delete** was refused by the document's
/// structure gate.
pub(crate) fn record_field_delete_refused() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::FieldDeleteRefused));
}

/// Record that a **bookmark move** did not happen, and which of the two
/// sentences it owes.
pub(crate) fn record_bookmark_move_refused(own_subtree: bool) {
    LAST.with_borrow_mut(|slot| {
        *slot = Some(if own_subtree {
            Declined::BookmarkMoveIntoOwnSubtree
        } else {
            Declined::BookmarkMoveRefused
        });
    });
}

/// Record that a resize was refused because the artwork cannot be rebuilt.
pub(crate) fn record_resize_not_rebuildable(uniform: bool) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::ResizeNotRebuildable { uniform }));
}

/// Record that a resize was refused because the annotation is a fixed-size
/// marker — a `/Text` sticky note (`by_flag == false`) or a `NoZoom`
/// annotation (`by_flag == true`).
pub(crate) fn record_resize_fixed_size_marker(by_flag: bool) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::ResizeFixedSizeMarker { by_flag }));
}

/// Record that a **rotation** did not happen, and why.
pub(crate) fn record_rotate(why: crate::text::rotating::RotateRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::Rotate(why)));
}

/// Record that *"give this page its own copy"* did not happen.
pub(crate) fn record_unshare(why: crate::text::unshare::UnshareRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::Unshare(why)));
}

/// Record why `format.merge_text_runs` merged nothing. A success is narrated
/// through the funnel's disclosures instead.
pub(crate) fn record_run_merge(why: crate::text::runmerge::RunMergeRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::RunMerge(why)));
}

/// Record that `file.save_copy` was given a destination and produced no file.
pub(crate) fn record_save_failure() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::SaveFailed));
}

/// Record that the Settings window's Save reached no disk.
pub(crate) fn record_settings_not_saved() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::SettingsNotSaved));
}

/// Record that `edit.undo` or `edit.redo` arrived with an empty stack.
pub(crate) fn record_history_empty(declined: Declined) {
    LAST.with_borrow_mut(|slot| *slot = Some(declined));
}

/// Record that `adopt_widget` refused, and which of its two correctable
/// refusals it was.
pub(crate) fn record_adopt_refusal(declined: Declined) {
    LAST.with_borrow_mut(|slot| *slot = Some(declined));
}

/// Record that **authoring a new form field** refused, and which refusal it
/// was — today, only [`super::Declined::FieldPathCrossesTerminal`].
pub(crate) fn record_field_author_refusal(declined: Declined) {
    LAST.with_borrow_mut(|slot| *slot = Some(declined));
}

/// Record that **renaming a form field** refused, and which refusal it was —
/// in practice always [`super::Declined::FieldNameTaken`], because that is the
/// only one the Rename button's gate lets through.
pub(crate) fn record_field_rename_refusal(declined: Declined) {
    LAST.with_borrow_mut(|slot| *slot = Some(declined));
}

/// Record that a **node of a markup shape** could not be moved, added or taken
/// away — [`super::Declined::MarkupNodeRefused`].
pub(crate) fn record_markup_node_refused(why: crate::text::markup::NodeEditRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::MarkupNodeRefused(why)));
}

/// Record that an OCR-layer write or removal did nothing, and why.
///
/// Called from inside the edit closure in `crate::app::actions`, so the
/// funnel's floor yields to this sentence rather than the generic one.
pub(crate) fn record_ocr_layer(why: crate::text::ocr::OcrLayerRefusal) {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::OcrLayer(why)));
}
