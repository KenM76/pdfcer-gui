//! # `panels::properties::widgetturn` — which way a form field's box faces,
//! and how solid it is drawn
//!
//! The quarter turns (`rotate_widget`, `/MK /R`), the free tilt
//! (`turn_widget`, the appearance `/Matrix`) and the opacity (`/CA`). A tilted
//! box refuses a quarter turn, so the two buttons grey with the reason and
//! *Stand upright* sits beside them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/widgetturn.md`.

use egui::Ui;
use pdfcer_core::edit::WidgetEdit;
use pdfcer_core::forms::{Field, Widget};
use pdfcer_gui_base::entry;
use pdfcer_gui_base::widgetread::{self, FreeTurn};

use super::widgetedit::{ROTATE_LEFT_REGION, ROTATE_RIGHT_REGION, ROTATION_REGION};
use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::app::state::OpenDoc;
use crate::canvas::markup::linestyle::DashReading;
use crate::text::panels::formfield as t;
use crate::text::panels::widgetturn as tt;

/// The *Stand upright* button.
// ui-text-exempt: trace region name, never displayed
pub const UPRIGHT_REGION: &str = "properties.widget_edit.upright";
/// The opacity field.
// ui-text-exempt: trace region name, never displayed
pub const OPACITY_REGION: &str = "properties.widget_edit.opacity";

/// What the widget section reads from the file beyond `forms::Widget`.
#[derive(Clone, Copy)]
pub struct WidgetFacts {
    /// The border's dash, as the border rows show it.
    pub dash: DashReading,
    /// The free tilt, `None` upright.
    pub turn: Option<FreeTurn>,
    /// `/CA`, `None` when unset.
    pub opacity: Option<f64>,
}

impl WidgetFacts {
    /// Read for widget `index` of `field`.
    pub(super) fn read(doc: &OpenDoc, field: &Field, index: usize, dash: DashReading) -> Self {
        let widget = field.widgets.get(index);
        Self {
            dash,
            turn: widget.and_then(|w| widgetread::free_turn(&doc.session, w)),
            opacity: widget.and_then(|w| widgetread::opacity(&doc.session, w)),
        }
    }
}

/// **Turn the box**: a quarter turn each way, or stand a tilted box upright.
pub(super) fn rotation_row(
    ui: &mut Ui,
    widget: &Widget,
    tilt: Option<FreeTurn>,
    (fqn, index): (&str, usize),
    actions: &mut Vec<Action>,
) {
    let current = widget.rotation.unwrap_or(0);
    ui.label(t::widget_rotation_label(widget.rotation));
    if let Some(tilt) = tilt {
        ui.label(tt::turned(tilt.degrees));
    }

    let quarter = |ui: &mut Ui, label: &str, region: &str, delta: i64| {
        let response = ui
            .add_enabled(tilt.is_none(), egui::Button::new(label))
            .on_disabled_hover_text(tt::quarter_needs_upright());
        // Gated: a check clicks these, and an off-screen button must be absent
        // from the trace rather than published where no click can land.
        crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
        if response.clicked() {
            // Normalised here as well as by the engine, so the trace and the
            // file carry the same number.
            let next = (current + delta).rem_euclid(360);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "widget-rotate-requested field={fqn:?} widget={index} was={current} now={next}"
                )
            });
            return Some(Action::Field(FieldAction::RotateWidget {
                field: fqn.to_owned(),
                widget: index,
                degrees: next,
            }));
        }
        None
    };

    let response = ui
        .horizontal(|ui| {
            // LEFT is +90 counterclockwise and RIGHT is -90: the one place the
            // operator's left/right becomes a sign.
            actions.extend(quarter(ui, t::widget_rotate_left(), ROTATE_LEFT_REGION, 90));
            actions.extend(quarter(
                ui,
                t::widget_rotate_right(),
                ROTATE_RIGHT_REGION,
                -90,
            ));
            if tilt.is_some() {
                let upright = ui.button(tt::stand_upright());
                crate::diag::ui_rect_visible(UPRIGHT_REGION, upright.rect, ui.clip_rect());
                if upright.clicked() {
                    actions.push(Action::Field(FieldAction::TurnWidget {
                        field: fqn.to_owned(),
                        widget: index,
                        degrees: 0.0,
                    }));
                }
            }
        })
        .response;
    crate::diag::ui_rect_visible(ROTATION_REGION, response.rect, ui.clip_rect());
    ui.small(t::widget_rotation_hint());
}

/// **The box's `/CA`**, 0–100 %, through `edit_widget`'s `with_opacity`.
pub(super) fn opacity_row(
    ui: &mut Ui,
    alpha: Option<f64>,
    (fqn, index): (&str, usize),
    actions: &mut Vec<Action>,
) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let read = (alpha.unwrap_or(1.0) * 100.0).round().clamp(0.0, 100.0) as u8;
    let mut percent = read;
    let mut send = |edit: WidgetEdit| {
        actions.push(Action::Field(FieldAction::EditWidget {
            field: fqn.to_owned(),
            widget: index,
            edit,
            touched: tt::touched_opacity(),
        }));
    };
    ui.horizontal(|ui| {
        ui.label(tt::opacity()).on_hover_text(tt::opacity_hint());
        let (widget, refusal) = entry::drag_value(ui, &mut percent, entry::Kind::Number(&["%"]));
        let response = refusal.show(
            ui.add(
                widget
                    .range(0..=100)
                    .speed(1.0)
                    .suffix(crate::text::panels::properties::markup_opacity_suffix()),
            ),
        );
        crate::diag::ui_rect_visible(OPACITY_REGION, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus()) && percent != read {
            send(WidgetEdit::new().with_opacity(f64::from(percent) / 100.0));
        }
        if alpha.is_some() && ui.button(tt::opacity_clear()).clicked() {
            send(WidgetEdit::new().clearing_opacity());
        }
    });
}
