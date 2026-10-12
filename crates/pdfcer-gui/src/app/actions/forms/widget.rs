//! # `app::actions::forms::widget` — verbs about the BOX, not the field
//!
//! `rotate_widget` and `turn_widget`. The seam is the one `EditSession` already draws
//! between `edit_field` and `edit_widget`: a field has one name, one value and
//! one set of flags, and it may draw **three boxes on three pages**. A verb
//! that turns one of those boxes is a statement about a placement, not about
//! the field — which is why every function here takes a widget INDEX and why
//! each reports how many siblings it left alone.
//!
//! Its own file under **R2**, along that seam rather than at a line count.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/forms/widget.md`.

use pdfcer_core::edit::EditError;

use crate::app::state::OpenDoc;
use crate::text::panels::widgetturn as t;

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

/// **Tilt one of a field's boxes** to `degrees` counterclockwise on top of
/// its quarter turn — absolute, so `0.0` stands it back up.
pub(super) fn turn(doc: &mut OpenDoc, fqn: &str, index: usize, degrees: f64) {
    let page = doc.view.page_index;
    crate::app::actions::apply::vector_edit(doc, "turn-widget", page, 1, |session| {
        session
            .turn_widget(fqn, index, degrees)
            .map(|turn| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "turn-widget-applied field={fqn:?} widget={index} was={:.2} now={:.2} \
                         changed={}",
                        turn.was, turn.now, turn.changed
                    )
                });
                let mut notes = vec![t::tilted(turn.now)];
                notes.extend(turn.disclosures);
                notes
            })
            .map_err(Worded)
    });
}

/// An engine refusal, worded for the operator where the engine's sentence
/// names a verb or a key.
pub(super) struct Worded(pub(super) EditError);

impl std::fmt::Display for Worded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            EditError::WidgetTurnNeedsAppearance { .. } => f.write_str(t::needs_appearance()),
            EditError::WidgetTurnSharedAppearance { .. } => f.write_str(t::shared_appearance()),
            EditError::WidgetTurned { degrees, .. } => f.write_str(&t::needs_upright(*degrees)),
            other => other.fmt(f),
        }
    }
}

impl crate::app::unlock::Refusal for Worded {
    fn encrypted(&self) -> bool {
        crate::app::unlock::Refusal::encrypted(&self.0)
    }
}
