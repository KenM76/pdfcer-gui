//! # `panels::properties::buttonicon` — a push button's picture (`/MK /I`)
//! and where its caption sits beside it (`/MK /TP`)
//!
//! Drawn by `widgetedit::section` under the caption, for a push button only:
//! the engine refuses an icon on any other field kind
//! (`EditError::NotAPushButton`), so another kind renders nothing rather than
//! a disabled row. *Choose picture…* raises `FieldAction::PickButtonIcon`,
//! whose picker runs in the apply phase; *Remove picture* and the caption
//! combo raise `FieldAction::EditWidget`. The combo is drawn only over an
//! icon, because with none the button draws its caption alone whatever `/TP`
//! says.

use egui::Ui;
use pdfcer_core::annot_author::CaptionPosition;
use pdfcer_core::edit::WidgetEdit;
use pdfcer_core::forms::{ButtonKind, Field, FieldType, Widget};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::text::panels::buttonicon as t;

/// The rows' rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.widget_edit.button_icon";
/// *Choose picture…* / *Replace picture…*.
// ui-text-exempt: trace region name, never displayed
pub const CHOOSE_REGION: &str = "properties.widget_edit.button_icon.choose";
/// *Remove picture*.
// ui-text-exempt: trace region name, never displayed
pub const REMOVE_REGION: &str = "properties.widget_edit.button_icon.remove";
/// The caption-position combo. Each entry, while it is open, is this
/// suffixed with `.` and its `/TP` number.
// ui-text-exempt: trace region name, never displayed
pub const POSITION_REGION: &str = "properties.widget_edit.button_icon.position";

/// `/TP` 0–6 in Table 189's order, which is the combo's.
const POSITIONS: [CaptionPosition; 7] = [
    CaptionPosition::CaptionOnly,
    CaptionPosition::IconOnly,
    CaptionPosition::CaptionBelow,
    CaptionPosition::CaptionAbove,
    CaptionPosition::CaptionRight,
    CaptionPosition::CaptionLeft,
    CaptionPosition::Overlaid,
];

/// Whether `field` is a push button, the one kind that takes an icon.
#[must_use]
pub fn is_push_button(field: &Field) -> bool {
    field.field_type == Some(FieldType::Button) && field.button_kind == Some(ButtonKind::Push)
}

/// Draw the picture rows for placement `widget_index` of `field`, or nothing
/// when it is not a push button.
pub fn rows(
    ui: &mut Ui,
    field: &Field,
    widget: &Widget,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    if !is_push_button(field) {
        return;
    }
    let present = widget.icon.is_some();
    let position = widget.caption_position.unwrap_or_default();
    trace_shown(ui.ctx(), fqn, widget_index, present, position);
    let top = ui.min_rect().bottom();
    ui.horizontal(|ui| {
        ui.label(t::label_icon());
        ui.small(if present { t::has_icon() } else { t::no_icon() });
    });
    ui.horizontal(|ui| {
        let choose = ui.button(if present { t::replace() } else { t::choose() });
        crate::diag::ui_rect_visible(CHOOSE_REGION, choose.rect, ui.clip_rect());
        if choose.clicked() {
            actions.push(
                FieldAction::PickButtonIcon {
                    field: fqn.to_owned(),
                    widget: widget_index,
                }
                .into(),
            );
        }
        if present {
            let remove = ui.button(t::remove()).on_hover_text(t::remove_hint());
            crate::diag::ui_rect_visible(REMOVE_REGION, remove.rect, ui.clip_rect());
            if remove.clicked() {
                push_edit(
                    actions,
                    fqn,
                    widget_index,
                    WidgetEdit::new().without_button_icon(),
                );
            }
        }
    });
    if present {
        position_row(ui, fqn, widget_index, position, actions);
    }
    let mut rect = ui.min_rect();
    rect.min.y = top;
    crate::diag::ui_rect(REGION, rect);
}

/// The caption-position combo.
fn position_row(
    ui: &mut Ui,
    fqn: &str,
    widget_index: usize,
    current: CaptionPosition,
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::label_caption_position());
        let combo = egui::ComboBox::from_id_salt("button-icon-position")
            .selected_text(t::caption_position(current))
            .show_ui(ui, |ui| {
                for choice in POSITIONS {
                    let item = ui.selectable_label(choice == current, t::caption_position(choice));
                    let name = format!("{POSITION_REGION}.{}", choice.to_tp()); // ui-text-exempt: trace region name
                    crate::diag::ui_rect_visible(&name, item.rect, ui.clip_rect());
                    if item.clicked() && choice != current {
                        let edit = WidgetEdit::new().with_caption_position(choice);
                        push_edit(actions, fqn, widget_index, edit);
                    }
                }
            });
        crate::diag::ui_rect_visible(POSITION_REGION, combo.response.rect, ui.clip_rect());
    });
}

/// Every icon edit may replace artwork another program drew, because a
/// button that keeps it shows no picture; the status line says when it did.
fn push_edit(actions: &mut Vec<Action>, fqn: &str, widget_index: usize, edit: WidgetEdit) {
    let edit = edit.with_replace_foreign_appearance(true);
    let touched = if edit.caption_position.is_some() {
        t::touched_caption_position()
    } else {
        t::touched_icon()
    };
    actions.push(
        FieldAction::EditWidget {
            field: fqn.to_owned(),
            widget: widget_index,
            edit,
            touched,
        }
        .into(),
    );
}

/// Trace the icon state the panel shows, once per change, which is what a
/// driven check asserts on after an edit.
fn trace_shown(
    ctx: &egui::Context,
    fqn: &str,
    widget_index: usize,
    present: bool,
    position: CaptionPosition,
) {
    let shown = format!(
        // ui-text-exempt: diagnostic trace, never displayed
        "button-icon-shown field={fqn} widget={widget_index} icon={} position={}",
        if present { "present" } else { "absent" },
        position.to_tp()
    );
    let id = egui::Id::new("button-icon-shown"); // ui-text-exempt: memory key, never displayed
    let last: Option<String> = ctx.data(|d| d.get_temp(id));
    if last.as_deref() != Some(shown.as_str()) {
        crate::diag::trace(|| shown.clone());
        ctx.data_mut(|d| d.insert_temp(id, shown));
    }
}
