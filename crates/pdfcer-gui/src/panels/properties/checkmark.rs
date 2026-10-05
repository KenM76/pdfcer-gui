//! # `panels::properties::checkmark` — a check box's or radio button's mark (`/MK /CA`)
//!
//! Drawn by `widgetedit::section` in place of the caption box: the engine draws
//! `/MK /CA` as the glyph (`CheckStyle`), so a typed caption there is one
//! character of ZapfDingbats the operator cannot read. A pick raises
//! `FieldAction::EditWidget` with the style's character and
//! `ForeignAppearance::Replace`, because a box another program drew keeps its
//! own artwork otherwise and the pick would show nothing. A widget with no
//! `/MK /CA` is drawn with the kind's default: a tick, or a radio's dot.

use egui::Ui;
use pdfcer_core::annot_author::CheckStyle;
use pdfcer_core::edit::{ForeignAppearance, WidgetEdit};
use pdfcer_core::forms::{ButtonKind, Field, FieldType, Widget};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::text::formfield as t;
use crate::text::panels::formfield as tp;

/// The picker's rect, for `ui-verify`; entry `i` of `CHECK_STYLES` is `{REGION}.{i}`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.widget_edit.mark";

/// Whether `field` is a check box or a radio button, the kinds this row is
/// drawn for.
#[must_use]
pub fn applies(field: &Field) -> bool {
    field.field_type == Some(FieldType::Button)
        && matches!(
            field.button_kind,
            Some(ButtonKind::Check | ButtonKind::Radio)
        )
}

/// The mark `widget` is drawn with: `None` when its `/MK /CA` names a symbol
/// pdfcer has no name for, which another program chose.
fn current(field: &Field, widget: &Widget) -> Option<CheckStyle> {
    match widget.caption.as_deref().and_then(|c| c.first().copied()) {
        None if field.button_kind == Some(ButtonKind::Radio) => Some(CheckStyle::Circle),
        None => Some(CheckStyle::Check),
        Some(c) => CheckStyle::from_mk_caption_char(c),
    }
}

/// The mark row.
pub fn row(
    ui: &mut Ui,
    field: &Field,
    widget: &Widget,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    let now = current(field, widget);
    crate::diag::trace_changed(REGION, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "widget-mark-shown field={fqn} widget={widget_index} mark={}",
            now.map_or('?', |s| char::from(s.mk_caption_char()))
        )
    });
    ui.horizontal(|ui| {
        ui.label(t::mark_label()).on_hover_text(t::mark_hover());
        let shown = now.map_or_else(tp::mark_unnamed, t::check_style_label);
        let combo = egui::ComboBox::from_id_salt(REGION)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for (i, style) in t::CHECK_STYLES.into_iter().enumerate() {
                    let selected = now == Some(style);
                    let entry = ui.selectable_label(selected, t::check_style_label(style));
                    crate::diag::ui_rect_visible(
                        &format!("{REGION}.{i}"), // ui-text-exempt: trace region name
                        entry.rect,
                        ui.clip_rect(),
                    );
                    if entry.clicked() && !selected {
                        let edit = WidgetEdit::new()
                            .with_caption(char::from(style.mk_caption_char()).to_string())
                            .with_foreign_appearance(ForeignAppearance::Replace);
                        actions.push(
                            FieldAction::EditWidget {
                                field: fqn.to_owned(),
                                widget: widget_index,
                                edit,
                                touched: tp::touched_mark(),
                            }
                            .into(),
                        );
                    }
                }
            });
        crate::diag::ui_rect_visible(REGION, combo.response.rect, ui.clip_rect());
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::edit::{EditSession, NewCheckBox, TooltipChoice};
    use pdfcer_core::page_tree::Rect;

    /// The edit this row pushes must redraw the box, not only record the
    /// character: the engine recovers the style from `/MK /CA` when it draws.
    #[test]
    fn picking_a_mark_redraws_the_box() {
        let path = crate::panels::objects::test_support::engine_fixture("forms/demo-form.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let mut session = EditSession::new(doc);
        let rect = Rect {
            llx: 300.0,
            lly: 300.0,
            urx: 320.0,
            ury: 320.0,
        };
        let mut spec = NewCheckBox::new(0, "mark_probe", rect);
        spec.tooltip = TooltipChoice::Declined;
        session.add_check_box(&spec).expect("a check box is placed");
        let edit = WidgetEdit::new()
            .with_caption(char::from(CheckStyle::Star.mk_caption_char()).to_string())
            .with_foreign_appearance(ForeignAppearance::Replace);
        let out = session
            .edit_widget("mark_probe", 0, &edit)
            .expect("the mark is accepted");
        assert!(out.appearance_regenerated, "the box must be redrawn");
        let view = session.view();
        let form = pdfcer_core::forms::parse_acroform(&view).expect("a form");
        let field = form
            .fields
            .iter()
            .find(|f| f.fully_qualified_name == "mark_probe")
            .expect("the probe is in the form");
        assert!(applies(field));
        assert_eq!(current(field, &field.widgets[0]), Some(CheckStyle::Star));
    }
}
