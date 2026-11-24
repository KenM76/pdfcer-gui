//! # `panels::properties::geometry::scaled` — **width and height in a ce
//! dimension group's scale**
//!
//! Drawn under the W and H point fields when the authoring group (the one the
//! measure tools draw into, else the default group) has a scale set. The
//! readout is `format_measurement_on`, so it reads exactly as a ce dimension
//! of that length would; a typed length goes back through
//! `user_units_from_measurement_on`, so this module does no scale arithmetic.
//! A read length only changes the draft: Apply commits it with the point
//! fields, as one undo step.

use egui::Ui;
use pdfcer_core::dimension::{
    DEFAULT_GROUP_ID, ScaleState, format_measurement_on, user_units_from_measurement_on,
};

use crate::app::state::OpenDoc;
use crate::text::panels::properties as t;

use super::GeometryDraft;

/// The scaled **Width** field's own region.
pub const WIDTH_REGION: &str = "properties.geometry.scaledwidth"; // ui-text-exempt: diagnostic region name

/// Draw the two scaled rows for `page`, or nothing when the group has no scale.
pub(super) fn rows(
    ui: &mut Ui,
    doc: &OpenDoc,
    page: usize,
    draft: &mut GeometryDraft,
    locked: Option<&str>,
) {
    let model = doc.session.dimension_model();
    let id = crate::canvas::measure::active_group(ui.ctx()).unwrap_or(DEFAULT_GROUP_ID);
    let (Some(group), Some(page_ref)) = (model.group(id), doc.pages.get(page)) else {
        return;
    };
    if matches!(group.scale, ScaleState::NeverSet) {
        return;
    }
    let (scale, format) = (group.scale, group.format);
    ui.label(t::geometry_scaled_heading(&group.name));
    let shown = |v: f64| format_measurement_on(page_ref, v, scale, format).text;
    let read = |typed: &str| user_units_from_measurement_on(page_ref, typed, scale, format);
    for (axis, label, value, region) in [
        ("w", t::geometry_w(), &mut draft.w, Some(WIDTH_REGION)),
        ("h", t::geometry_h(), &mut draft.h, None),
    ] {
        row(ui, (axis, label, region, locked), value, &shown, &read);
    }
    ui.label(
        egui::RichText::new(t::geometry_scaled_note())
            .small()
            .weak(),
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "geometry-scaled group={} w={:?} h={:?}",
            id.0,
            shown(draft.w),
            shown(draft.h)
        )
    });
}

/// One scaled length: the readout while idle, the operator's text while typed.
fn row(
    ui: &mut Ui,
    (axis, label, region, locked): (&str, &str, Option<&'static str>, Option<&str>),
    value: &mut f64,
    shown: &dyn Fn(f64) -> String,
    read: &dyn Fn(&str) -> Result<f64, pdfcer_core::dimension::LengthParseError>,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let id = ui.make_persistent_id(("geometry-scaled", axis));
        let typing: Option<String> = ui.data(|d| d.get_temp(id));
        let mut text = typing.clone().unwrap_or_else(|| shown(*value));
        // escape-disposition: commits — the read is keyed on `lost_focus()`.
        let edit = egui::TextEdit::singleline(&mut text).desired_width(110.0);
        let response = match locked {
            Some(reason) => ui.add_enabled(false, edit).on_disabled_hover_text(reason),
            None => ui.add(edit),
        };
        if let Some(region) = region {
            crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
        }
        let why_id = id.with("why");
        if response.has_focus() {
            ui.data_mut(|d| d.insert_temp(id, text.clone()));
        } else if response.lost_focus() || typing.is_some() {
            ui.data_mut(|d| d.remove::<String>(id));
            if text.trim() != shown(*value) {
                let why = commit(axis, &text, value, read);
                ui.data_mut(|d| match why {
                    Some(why) => {
                        d.insert_temp(why_id, why);
                    }
                    None => d.remove::<String>(why_id),
                });
            }
        }
    });
    if let Some(why) = ui.data(|d| {
        d.get_temp::<String>(ui.make_persistent_id(("geometry-scaled", axis)).with("why"))
    }) {
        ui.label(
            egui::RichText::new(t::geometry_scaled_refused(&why))
                .small()
                .weak(),
        );
    }
}

/// Read `text` into `value`, or return why it could not be read.
fn commit(
    axis: &str,
    text: &str,
    value: &mut f64,
    read: &dyn Fn(&str) -> Result<f64, pdfcer_core::dimension::LengthParseError>,
) -> Option<String> {
    let why = match read(text) {
        Ok(pts) if pts > 0.0 => {
            *value = pts;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("geometry-scaled-typed axis={axis} typed={text:?} pts={pts:.3}")
            });
            return None;
        }
        Ok(_) => pdfcer_core::dimension::LengthParseError::NotPositive {
            input: text.to_owned(),
        }
        .to_string(),
        Err(e) => e.to_string(),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("geometry-scaled-refused axis={axis} typed={text:?} why={why:?}")
    });
    Some(why)
}
