//! # `panels::properties::markup::anyannot` — what every annotation takes
//!
//! Opacity for an annotation outside `set_markup_style`'s family
//! (`set_annot_opacity`), the icon colour of a caret, attachment, sound or
//! screen mark (`set_marker_style`), and the redraw offer a refused restyle
//! of the selected annotation leaves behind.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/markup/anyannot.md`.

use egui::Ui;
use pdfcer_core::annot_author::Color;
use pdfcer_core::edit::StyleEdit;
use pdfcer_gui_base::entry;
use pdfcer_gui_base::redrawoffer::{RedrawOffer, Why};

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::canvas::selection::annot::AnnotTarget;
use crate::text::panels::properties as t;
use crate::text::panels::textannotstyle as ts;

/// The opacity field's region.
pub(super) const OPACITY_REGION: &str = "properties.markup.any.opacity"; // ui-text-exempt: trace region name, never displayed
/// The marker colour swatch's region.
pub(super) const MARKER_REGION: &str = "properties.markup.any.marker"; // ui-text-exempt: trace region name, never displayed
/// The redraw button's region.
pub(super) const REDRAW_REGION: &str = "properties.markup.any.redraw"; // ui-text-exempt: trace region name, never displayed

/// **The annotation's `/CA`**, through `set_annot_opacity`, which writes the
/// dictionary only and leaves the appearance bytes alone.
pub(super) fn opacity_row(
    ui: &mut Ui,
    alpha: Option<f64>,
    target: &AnnotTarget,
    actions: &mut Vec<Action>,
) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let read = (alpha.unwrap_or(1.0) * 100.0).round().clamp(0.0, 100.0) as u8;
    let mut percent = read;
    ui.horizontal(|ui| {
        ui.label(t::markup_opacity_label());
        let (widget, refusal) = entry::drag_value(ui, &mut percent, entry::Kind::Number(&["%"]));
        let response = refusal.show(
            ui.add(
                widget
                    .range(0..=100)
                    .speed(1.0)
                    .suffix(t::markup_opacity_suffix()),
            ),
        );
        crate::diag::ui_rect_visible(OPACITY_REGION, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus()) && percent != read {
            actions.push(Action::Annot(AnnotAction::SetOpacity {
                id: target.id,
                opacity: StyleEdit::Set(f64::from(percent) / 100.0),
            }));
        }
        if alpha.is_some() && ui.button(t::markup_clear()).clicked() {
            actions.push(Action::Annot(AnnotAction::SetOpacity {
                id: target.id,
                opacity: StyleEdit::Clear,
            }));
        }
    });
}

/// **A marker's icon colour**, through `set_marker_style`.
pub(super) fn marker_row(
    ui: &mut Ui,
    colour: super::Swatch,
    target: &AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let seed = colour.rgb.unwrap_or([0, 0, 0]);
    ui.horizontal(|ui| {
        ui.label(ts::marker_colour_label());
        // The hint is shown for a mixed value only, which one mark never has.
        let picked = crate::panels::properties::swatch::show(
            ui,
            // ui-text-exempt: an egui id salt, never displayed
            "properties-annot-marker",
            crate::panels::properties::swatch::Value::Agreed(seed),
            MARKER_REGION,
            "",
        );
        if let Some(rgb) = picked {
            actions.push(Action::Annot(AnnotAction::SetMarkerColour {
                id: target.id,
                color: Color::Rgb(
                    f64::from(rgb[0]) / 255.0,
                    f64::from(rgb[1]) / 255.0,
                    f64::from(rgb[2]) / 255.0,
                ),
                redraw_as_plain: false,
            }));
        }
    });
    if colour.narrowed {
        ui.label(
            egui::RichText::new(t::markup_colour_narrowed())
                .small()
                .weak(),
        );
    }
}

/// **The redraw offer** for the selected annotation, when its last restyle
/// was refused for a look pdfcer cannot redraw faithfully: the reason, and the
/// button that retries the same change with the redraw allowed.
pub(super) fn offer_row(
    ui: &mut Ui,
    offer: Option<&RedrawOffer>,
    target: &AnnotTarget,
    actions: &mut Vec<Action>,
) {
    let Some(offer) = offer.filter(|o| o.id == target.id) else {
        return;
    };
    let (why, button) = match offer.why {
        Why::TextBoxForeign => (
            ts::redraw_offer_text_box_foreign(),
            ts::redraw_as_plain_button(),
        ),
        Why::TextBoxRich => (
            ts::redraw_offer_text_box_rich(),
            ts::redraw_as_plain_button(),
        ),
        Why::MarkerForeign => (
            ts::redraw_offer_marker_foreign(),
            ts::redraw_marker_button(),
        ),
    };
    ui.label(egui::RichText::new(why).small());
    let response = ui.button(button);
    crate::diag::ui_rect_visible(REDRAW_REGION, response.rect, ui.clip_rect());
    if response.clicked() {
        actions.push(Action::Annot(offer.retry()));
    }
}
