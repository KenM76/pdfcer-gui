//! # `app::actions::forms::widget` — verbs about the BOX, not the field
//!
//! `rotate_widget` today. The seam is the one `EditSession` already draws
//! between `edit_field` and `edit_widget`: a field has one name, one value and
//! one set of flags, and it may draw **three boxes on three pages**. A verb
//! that turns one of those boxes is a statement about a placement, not about
//! the field — which is why every function here takes a widget INDEX and why
//! each reports how many siblings it left alone.
//!
//! Split out of `super` under R2 on 2026-08-30, when rotation took that file
//! past 1,500 lines.

use crate::app::state::OpenDoc;

/// **Turn one of a field's boxes.**
///
/// # ★★ What the engine may not be able to do, and why it says so
///
/// `WidgetRotation::appearance_stale` carries a reason when the widget's baked
/// `/AP` could not be regenerated at the new angle. That is not a failure — the
/// rotation is written and the file is correct — but the box will draw at its
/// old orientation until something regenerates it, and an operator watching a
/// box refuse to turn deserves the sentence rather than a mystery.
///
/// ★ `siblings_untouched` is surfaced for the same reason `edit_widget`'s is: a
/// field with three boxes has three orientations, and turning one is a
/// statement about one placement. Saying how many were left alone is what stops
/// *"I rotated the field"* meaning two different things.
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
            notes
        })
    });
}
