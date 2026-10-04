//! # `app::actions::forms::scripts` — set or clear one of a field's scripts
//!
//! `EditSession::set_field_format`, `set_field_validation` and
//! `set_field_calculation`, one undo entry each. The status lines name what
//! the engine did beyond the ask: a displaced script, the paired keystroke
//! filter, and the calculation-order entry.

use pdfcer_core::edit::FieldScriptChange;
use pdfcer_gui_base::fieldscript::ScriptEdit;

use crate::app::state::OpenDoc;

/// Apply `edit` to the field named `field`.
pub(super) fn set(doc: &mut OpenDoc, field: &str, edit: ScriptEdit) {
    super::super::apply::vector_edit(doc, "set-field-script", 0, 1, |session| {
        let change = match edit {
            ScriptEdit::Format(h) => session.set_field_format(field, h),
            ScriptEdit::Validate(h) => session.set_field_validation(field, h),
            ScriptEdit::Calculate(h) => session.set_field_calculation(field, h),
        }?;
        trace(&change);
        Ok::<_, pdfcer_core::edit::EditError>(crate::text::fieldscripts::changed(&change))
    });
}

fn trace(change: &FieldScriptChange) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        let order = change.calculation_order.as_ref();
        format!(
            "field-script-set field={} trigger={} applied={} replaced={} paired={} position={} entries={} created={}",
            change.name,
            change.trigger,
            change.applied.as_ref().map_or("none", |c| c.token()),
            change.replaced.as_ref().map_or("none", |c| c.token()),
            change.keystroke_paired,
            order
                .and_then(|o| o.position)
                .map_or_else(|| "none".to_owned(), |p| p.to_string()),
            order.map_or(0, |o| o.entries),
            order.is_some_and(|o| o.array_created),
        )
    });
}
