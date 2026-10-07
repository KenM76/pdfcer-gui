//! # `app::actions::forms::widget` — verbs about the BOX, not the field
//!
//! `rotate_widget` today. The seam is the one `EditSession` already draws
//! between `edit_field` and `edit_widget`: a field has one name, one value and
//! one set of flags, and it may draw **three boxes on three pages**. A verb
//! that turns one of those boxes is a statement about a placement, not about
//! the field — which is why every function here takes a widget INDEX and why
//! each reports how many siblings it left alone.
//!
//! Its own file under **R2**, along that seam rather than at a line count.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/forms/widget.md`.

use crate::app::state::OpenDoc;

/// **Turn one of a field's boxes.**
pub(super) fn rotate(doc: &mut OpenDoc, fqn: &str, index: usize, degrees: i64) {
    let page = doc.view.page_index;
    crate::app::actions::apply::vector_edit(doc, "rotate-widget", page, 1, |session| {
        session.rotate_widget(fqn, index, degrees).map(|report| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "rotate-widget-applied field={fqn:?} widget={index} was={:?} now={:?} regenerated={} siblings={}",
                    report.was,
                    report.now,
                    report.appearance_regenerated,
                    report.siblings_untouched
                )
            });
            let mut notes = vec![crate::text::panels::formfield::widget_rotated(
                report.now.unwrap_or(0),
                report.siblings_untouched,
            )];
            if let Some(why) = report.appearance_stale.as_deref() {
                notes.push(crate::text::panels::formfield::widget_rotation_stale(why));
            }
            notes.extend(crate::text::forms::redraw_notes(
                crate::text::forms::RedrawSubject::Field(fqn),
                &report.layout,
            ));
            notes
        })
    });
}
