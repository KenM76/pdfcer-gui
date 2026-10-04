//! A placed field's less-used properties: a text field's scrolling,
//! spell-check and file-select flags, and every field's export name (`/TM`)
//! and whether it is sent with the form (`/Ff` bit 3, NoExport).
//!
//! The flags are advisory to the reader that fills the form in; pdfcer neither
//! scrolls, spell-checks nor submits differently for them. File select changes
//! what the field is (a submit then sends a local file's contents), so while it
//! is set a sentence under the checkbox says so.

use egui::Ui;
use pdfcer_core::edit::FieldEdit;
use pdfcer_core::forms::{Field, FieldFlags};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::panels::PanelsState;
use crate::text::panels::choiceopts as spell;
use crate::text::panels::fieldextras as t;

/// The Scroll long text checkbox's region.
// ui-text-exempt: trace region name, never displayed
pub const SCROLL_REGION: &str = "properties.field_edit.scroll";
/// A text field's Check spelling checkbox's region.
// ui-text-exempt: trace region name, never displayed
pub const SPELL_REGION: &str = "properties.field_edit.spell_check";
/// The Sends a file checkbox's region.
// ui-text-exempt: trace region name, never displayed
pub const FILE_SELECT_REGION: &str = "properties.field_edit.file_select";
/// The file-select sentence's region, published only while the flag is set.
// ui-text-exempt: trace region name, never displayed
pub const FILE_SELECT_NOTE_REGION: &str = "properties.field_edit.file_select.note";
/// The Sent with the form checkbox's region.
// ui-text-exempt: trace region name, never displayed
pub const SENT_REGION: &str = "properties.field_edit.sent";
/// The Export name box's region.
// ui-text-exempt: trace region name, never displayed
pub const EXPORT_NAME_REGION: &str = "properties.field_edit.export_name";

/// One checkbox: label, hover, checked, region, the name a refusal uses, and
/// the edit a new state makes.
type FlagRow = (
    &'static str,
    &'static str,
    bool,
    &'static str,
    &'static str,
    fn(bool) -> FieldEdit,
);

/// A text field's three flags. Call only for a `/Tx`.
pub fn text_flags(ui: &mut Ui, field: &Field, fqn: &str, actions: &mut Vec<Action>) {
    let flags = field.flags;
    let rows: [FlagRow; 3] = [
        (
            t::flag_scroll(),
            t::flag_scroll_hover(),
            !flags.has(FieldFlags::DO_NOT_SCROLL),
            SCROLL_REGION,
            // ui-text-exempt: a control name carried for a refusal message.
            "scroll long text",
            |on| FieldEdit::new().with_no_scroll(!on),
        ),
        (
            spell::flag_spell_check(),
            spell::flag_spell_check_hover(),
            !flags.has(FieldFlags::DO_NOT_SPELL_CHECK),
            SPELL_REGION,
            // ui-text-exempt: a control name carried for a refusal message.
            "check spelling",
            |on| FieldEdit::new().with_no_spell_check(!on),
        ),
        (
            t::flag_file_select(),
            t::flag_file_select_hover(),
            flags.has(FieldFlags::FILE_SELECT),
            FILE_SELECT_REGION,
            // ui-text-exempt: a control name carried for a refusal message.
            "sends a file",
            |on| FieldEdit::new().with_file_select(on),
        ),
    ];
    for (label, hover, checked, region, touched, edit) in rows {
        let mut on = checked;
        let response = ui.checkbox(&mut on, label);
        crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
        if response.on_hover_text(hover).changed() {
            actions.push(
                FieldAction::EditProperties {
                    field: fqn.to_owned(),
                    edit: edit(on),
                    touched,
                }
                .into(),
            );
        }
    }
    if flags.has(FieldFlags::FILE_SELECT) {
        let note = ui.weak(t::file_select_note());
        crate::diag::ui_rect_visible(FILE_SELECT_NOTE_REGION, note.rect, ui.clip_rect());
    }
}

/// `/Ff` bit 3 (NoExport), drawn as its positive. Every field type has it.
pub fn sent_row(ui: &mut Ui, field: &Field, fqn: &str, actions: &mut Vec<Action>) {
    let mut on = !field.flags.no_export();
    let response = ui.checkbox(&mut on, t::flag_sent());
    crate::diag::ui_rect_visible(SENT_REGION, response.rect, ui.clip_rect());
    if response.on_hover_text(t::flag_sent_hover()).changed() {
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit: FieldEdit::new().with_no_export(!on),
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "sent with the form",
            }
            .into(),
        );
    }
}

/// `/TM`, committed when the box loses focus with a changed value; an empty
/// box removes it.
pub fn export_name_row(
    ui: &mut Ui,
    field: &Field,
    fqn: &str,
    state: &mut PanelsState,
    actions: &mut Vec<Action>,
) {
    ui.label(t::label_export_name());
    let draft = state.field_props_mut();
    let response = ui.add(
        // escape-disposition: commits — on `lost_focus`, which Escape triggers.
        // An empty draft removes `/TM`.
        egui::TextEdit::singleline(&mut draft.export_name)
            .desired_width(f32::INFINITY)
            .hint_text(t::label_export_name_hint()),
    );
    crate::diag::ui_rect_visible(EXPORT_NAME_REGION, response.rect, ui.clip_rect());
    let response = response.on_hover_text(t::label_export_name_hover());
    let typed = draft.export_name.trim().to_owned();
    let stored = draft.export_name_stored.clone();
    trace_read(field, fqn, &stored);
    if response.lost_focus() && typed != stored {
        let edit = if typed.is_empty() {
            FieldEdit::new().clearing_mapping_name()
        } else {
            FieldEdit::new().with_mapping_name(typed)
        };
        actions.push(
            FieldAction::EditProperties {
                field: fqn.to_owned(),
                edit,
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "export name",
            }
            .into(),
        );
    }
}

/// What the document holds, every frame the section draws, for `ui-verify`.
fn trace_read(field: &Field, fqn: &str, export: &str) {
    let flags = field.flags;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "field-extras-read field={fqn} scroll={} spell={} file={} sent={} export={}",
            u8::from(!flags.has(FieldFlags::DO_NOT_SCROLL)),
            u8::from(!flags.has(FieldFlags::DO_NOT_SPELL_CHECK)),
            u8::from(flags.has(FieldFlags::FILE_SELECT)),
            u8::from(!flags.no_export()),
            if export.is_empty() { "-" } else { export },
        )
    });
}
