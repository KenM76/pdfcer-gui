//! # `panels::properties::widgetdash` — a dashed widget border's pattern (`/BS /D`)
//!
//! Drawn by `widgetedit::border_rows` under the style, only while the border
//! is `Dashed`: the engine draws `/D` on that style alone. The three dashes of
//! [`LineStyle`] are offered; solid is the style row's choice. A pick raises
//! `FieldAction::EditWidget` with `WidgetEdit::with_border_dash` and
//! `ForeignAppearance::Replace`, so artwork another program drew is redrawn
//! with the new pattern rather than kept.
//!
//! The current pattern is read from the widget dictionary by the shell's
//! `linestyle::read`, because `forms::Widget` carries no dash (request G126).

use egui::Ui;
use pdfcer_core::edit::{ForeignAppearance, WidgetEdit};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::canvas::markup::linestyle::{DashReading, LineStyle};
use crate::text::panels::formfield as t;

/// The picker's rect, for `ui-verify`; entry `i` of [`DASHES`] is `{REGION}.{i}`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.widget_edit.dash";

/// The patterns offered, in the order the picker lists them.
const DASHES: [LineStyle; 3] = [LineStyle::Dashed, LineStyle::LongDash, LineStyle::DashDot];

/// The trace token for what the file says.
const fn token(reading: DashReading) -> &'static str {
    match reading {
        DashReading::Solid | DashReading::Offered(LineStyle::Solid) => "solid", // ui-text-exempt: trace token
        DashReading::Offered(LineStyle::Dashed) => "dashed", // ui-text-exempt: trace token
        DashReading::Offered(LineStyle::LongDash) => "long-dash", // ui-text-exempt: trace token
        DashReading::Offered(LineStyle::DashDot) => "dash-dot", // ui-text-exempt: trace token
        DashReading::Foreign => "foreign",                   // ui-text-exempt: trace token
    }
}

/// The dash row, for a widget whose border is `Dashed`.
pub fn row(
    ui: &mut Ui,
    reading: DashReading,
    fqn: &str,
    widget_index: usize,
    actions: &mut Vec<Action>,
) {
    crate::diag::trace_changed(REGION, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "widget-dash-shown field={fqn} widget={widget_index} dash={}",
            token(reading)
        )
    });
    ui.horizontal(|ui| {
        ui.label(t::label_dash()).on_hover_text(t::dash_hover());
        let combo = egui::ComboBox::from_id_salt(REGION)
            .selected_text(reading.label())
            .show_ui(ui, |ui| {
                for (i, style) in DASHES.into_iter().enumerate() {
                    let selected = reading.selected() == Some(style);
                    let entry = ui.selectable_label(selected, style.label());
                    crate::diag::ui_rect_visible(
                        &format!("{REGION}.{i}"), // ui-text-exempt: trace region name
                        entry.rect,
                        ui.clip_rect(),
                    );
                    if entry.clicked() && !selected {
                        let edit = WidgetEdit::new()
                            .with_border_dash(style.dash())
                            .with_foreign_appearance(ForeignAppearance::Replace);
                        actions.push(
                            FieldAction::EditWidget {
                                field: fqn.to_owned(),
                                widget: widget_index,
                                edit,
                                touched: t::touched_dash(),
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
    use pdfcer_core::edit::{BorderSpec, BorderStyle, EditSession, NewTextField, TooltipChoice};
    use pdfcer_core::object::Object;
    use pdfcer_core::page_tree::Rect;

    /// The edit this row pushes must reach `/BS /D` as the pattern the shell's
    /// reader names, which is what the row shows on the next frame.
    #[test]
    fn a_picked_dash_reads_back_as_that_dash() {
        let path = crate::panels::objects::test_support::engine_fixture("forms/demo-form.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let mut session = EditSession::new(doc);
        let rect = Rect {
            llx: 300.0,
            lly: 300.0,
            urx: 400.0,
            ury: 320.0,
        };
        let mut spec = NewTextField::new(0, "dash_probe", rect);
        spec.tooltip = TooltipChoice::Declined;
        spec.border = BorderSpec {
            style: BorderStyle::Dashed,
            width: 1.0,
        };
        session
            .add_text_field(&spec)
            .expect("a text field is placed");
        let edit = WidgetEdit::new()
            .with_border_dash(LineStyle::DashDot.dash())
            .with_foreign_appearance(ForeignAppearance::Replace);
        session
            .edit_widget("dash_probe", 0, &edit)
            .expect("the dash is accepted");
        let id = {
            let view = session.view();
            let form = pdfcer_core::forms::parse_acroform(&view).expect("a form");
            form.fields
                .iter()
                .find(|f| f.fully_qualified_name == "dash_probe")
                .expect("the probe is in the form")
                .widgets[0]
                .id
        };
        let graph = session.graph();
        let Some(Object::Dict(dict)) = session.value(id) else {
            panic!("the widget is a dictionary");
        };
        let reading = crate::canvas::markup::linestyle::read(&graph, dict);
        assert_eq!(reading, DashReading::Offered(LineStyle::DashDot));
        assert_eq!(token(reading), "dash-dot");
    }
}
