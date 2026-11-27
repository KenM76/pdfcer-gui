//! The Picture tab's panorama settings: the angles, the tilt, the layout, and
//! what the band drawn from this view will be, said under them.

use egui::Ui;

use super::panorama::{self, Band};
use super::shot::{
    REGION_SHOT_PANO_ACROSS, REGION_SHOT_PANO_CYLINDRICAL, REGION_SHOT_PANO_FROM_VIEW,
    REGION_SHOT_PANO_TILT, REGION_SHOT_PANO_UP_DOWN, Shot,
};
use crate::text::model3dshot as s;

/// The angles, the switches and the band's description.
pub(super) fn controls(ui: &mut Ui, shot: &mut Shot) {
    let pano = &mut shot.pano;
    let tallest = if pano.cylindrical {
        panorama::UP_DOWN_CYLINDRICAL
    } else {
        *panorama::UP_DOWN.end()
    };
    ui.horizontal_wrapped(|ui| {
        degrees(
            ui,
            (s::PANO_ACROSS, s::PANO_ACROSS_TOOLTIP),
            REGION_SHOT_PANO_ACROSS,
            &mut pano.across,
            *panorama::ACROSS.end(),
            true,
        );
    });
    ui.horizontal_wrapped(|ui| {
        degrees(
            ui,
            (s::PANO_UP_DOWN, s::PANO_UP_DOWN_TOOLTIP),
            REGION_SHOT_PANO_UP_DOWN,
            &mut pano.up_down,
            tallest,
            !pano.from_view,
        );
        let from = ui
            .checkbox(&mut pano.from_view, s::PANO_FROM_VIEW)
            .on_hover_text(s::PANO_FROM_VIEW_TOOLTIP);
        crate::diag::ui_rect_visible(REGION_SHOT_PANO_FROM_VIEW, from.rect, ui.clip_rect());
    });
    pano.up_down = pano.up_down.min(tallest);
    for (on, label, tip, region) in [
        (
            &mut pano.tilted,
            s::PANO_TILT,
            s::PANO_TILT_TOOLTIP,
            REGION_SHOT_PANO_TILT,
        ),
        (
            &mut pano.cylindrical,
            s::PANO_CYLINDRICAL,
            s::PANO_CYLINDRICAL_TOOLTIP,
            REGION_SHOT_PANO_CYLINDRICAL,
        ),
    ] {
        let switch = ui.checkbox(on, label).on_hover_text(tip);
        crate::diag::ui_rect_visible(region, switch.rect, ui.clip_rect());
    }
    let [w, h] = shot.size();
    match &shot.band {
        Some(Ok(band)) => said(ui, band, [w, h]),
        Some(Err(why)) => {
            ui.weak(s::pano_refused(why));
        }
        None => {}
    }
}

/// A degrees entry from 1° to `most`, greyed when not `enabled`.
fn degrees(
    ui: &mut Ui,
    (label, tip): (&str, &str),
    region: &str,
    value: &mut f64,
    most: f64,
    enabled: bool,
) {
    ui.label(label);
    let entry = ui
        .add_enabled(
            enabled,
            egui::DragValue::new(value)
                .range(1.0..=most)
                .speed(1.0)
                .fixed_decimals(0)
                .suffix("°"),
        )
        .on_hover_text(tip);
    crate::diag::ui_rect_visible(region, entry.rect, ui.clip_rect());
}

/// The band this view will draw: its size and angles, where its middle
/// looks, and what is not as set.
fn said(ui: &mut Ui, band: &Band, [w, h]: [u32; 2]) {
    ui.weak(s::pano_size(w, h, band.across, band.up_down));
    if band.pitch != 0.0 {
        ui.weak(s::pano_centre(band.pitch));
    }
    if band.at_pole() {
        ui.weak(s::pano_at_pole(band.pitch > 0.0));
    }
    if band.cylindrical {
        ui.weak(s::PANO_UNTAGGED);
    }
}
