//! A brightness for each lighting choice, and a sun the operator aims and
//! colours. Both are the operator's and kept between sessions; the picture
//! carries no mark of either, and the sun drawn is said under its controls.

use egui::Ui;
use pdfcer_3d::vec3::{dot, length, scale, sub};
use pdfcer_3d::{
    MAX_LIGHT_GAIN, MAX_SUN_KELVIN, MIN_LIGHT_GAIN, MIN_SUN_KELVIN, ShadowLight, SunColour,
    SunLight,
};
use pdfcer_core::threed::ThreeDLighting;

use super::{ModelView, sundisc};
use crate::text::model3d as m;
use crate::text::model3dlight as g;

/// The brightness slider.
pub const REGION_BRIGHTNESS: &str = "model3d.brightness"; // ui-text-exempt: trace region name, never displayed
/// The sun disc.
pub const REGION_SUN_DISC: &str = "model3d.sun.disc"; // ui-text-exempt: trace region name, never displayed
/// The sun's bearing field.
pub const REGION_SUN_BEARING: &str = "model3d.sun.bearing"; // ui-text-exempt: trace region name, never displayed
/// The sun's height field.
pub const REGION_SUN_HEIGHT: &str = "model3d.sun.height"; // ui-text-exempt: trace region name, never displayed
/// The colour-by-height switch.
pub const REGION_SUN_BY_HEIGHT: &str = "model3d.sun.by_height"; // ui-text-exempt: trace region name, never displayed
/// The colour temperature slider.
pub const REGION_SUN_KELVIN: &str = "model3d.sun.kelvin"; // ui-text-exempt: trace region name, never displayed
/// The sun's strength slider.
pub const REGION_SUN_STRENGTH: &str = "model3d.sun.strength"; // ui-text-exempt: trace region name, never displayed

/// The range a light strength is chosen in.
pub(super) const GAIN_RANGE: std::ops::RangeInclusive<f64> = MIN_LIGHT_GAIN..=MAX_LIGHT_GAIN;
/// The range a sun bearing is chosen in, degrees.
pub(super) const BEARING_RANGE: std::ops::RangeInclusive<f64> = 0.0..=360.0;
/// The range a sun height is chosen in, degrees.
pub(super) const HEIGHT_RANGE: std::ops::RangeInclusive<f64> = 0.0..=90.0;
/// The range a colour temperature is chosen in, kelvin.
pub(super) const KELVIN_RANGE: std::ops::RangeInclusive<f64> = MIN_SUN_KELVIN..=MAX_SUN_KELVIN;

/// The operator's sun.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SunAim {
    /// The aimed sun casts the shadows, in place of the fixed one.
    pub on: bool,
    /// Degrees round the Up axis, 0 the model's front.
    pub bearing: f64,
    /// Degrees above the horizon.
    pub height: f64,
    /// Coloured by its height, else by [`SunAim::kelvin`].
    pub by_height: bool,
    pub kelvin: f64,
    /// The sun's strength, a multiple of full.
    pub strength: f64,
}

impl Default for SunAim {
    fn default() -> Self {
        Self {
            on: false,
            bearing: 45.0,
            height: 45.0,
            by_height: true,
            kelvin: 5500.0,
            strength: 1.0,
        }
    }
}

/// The light choices beyond the scheme: a brightness per scheme, in
/// [`m::lighting_schemes`]' order, and the sun.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Light {
    pub gains: [f64; 12],
    pub sun: SunAim,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            gains: [1.0; 12],
            sun: SunAim::default(),
        }
    }
}

impl Light {
    /// `scheme`'s brightness.
    pub(super) fn gain(&self, scheme: ThreeDLighting) -> f64 {
        slot(scheme)
            .and_then(|i| self.gains.get(i))
            .copied()
            .unwrap_or(1.0)
    }
}

/// `scheme`'s place in [`Light::gains`].
fn slot(scheme: ThreeDLighting) -> Option<usize> {
    m::lighting_schemes().iter().position(|s| *s == scheme)
}

impl ModelView {
    /// The brightness the live picture's scheme is drawn at.
    pub(super) fn light_gain(&self) -> f64 {
        self.light.gain(self.lit_by().0)
    }

    /// Which light casts shadows: the aimed sun, the fixed sun, or the
    /// scheme. The engine adds an aimed sun as a light even with shadows
    /// off, so it is drawn only while they are on.
    pub(super) fn shadow_light(&self) -> ShadowLight {
        let (up, negative) = (self.upright.up.up_axis(), self.upright.up.negative());
        let a = self.light.sun;
        match (self.sun, a.on && self.shadows) {
            (true, true) => ShadowLight::AimedSun(SunLight {
                up,
                negative,
                azimuth_deg: a.bearing,
                elevation_deg: a.height,
                colour: if a.by_height {
                    SunColour::FromElevation
                } else {
                    SunColour::Kelvin(a.kelvin)
                },
                gain: a.strength,
            }),
            (true, false) => ShadowLight::Sun { up, negative },
            _ => ShadowLight::FollowsScheme,
        }
    }

    /// The sun bearing the camera looks from, degrees: the bearing at the
    /// sun disc's bottom, so the ball stands where the sun is relative to
    /// the picture. 0 when there is no camera to ask.
    pub(super) fn sun_facing(&self) -> f64 {
        let Ok(camera) = self.view_camera(1.0) else {
            return 0.0;
        };
        let (up, negative) = (self.upright.up.up_axis(), self.upright.up.negative());
        let towards = |azimuth_deg, elevation_deg| {
            SunLight {
                up,
                negative,
                azimuth_deg,
                elevation_deg,
                colour: SunColour::FromElevation,
                gain: 1.0,
            }
            .direction()
        };
        let ground = (towards(0.0, 0.0), towards(90.0, 0.0), towards(0.0, 90.0));
        facing_from(sub(camera.eye, camera.target), camera.up, ground)
    }

    /// The brightness slider for the scheme lighting the picture.
    pub(super) fn brightness_control(&mut self, ui: &mut Ui) {
        let Some(i) = slot(self.lit_by().0) else {
            return;
        };
        let Some(gain) = self.light.gains.get_mut(i) else {
            return;
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(g::brightness_label())
                .on_hover_text(g::brightness_tooltip());
            let slider = ui.add(percent_slider(gain));
            crate::diag::ui_rect_visible(REGION_BRIGHTNESS, slider.rect, ui.clip_rect());
            slider.on_hover_text(g::brightness_tooltip());
        });
    }

    /// The aimed sun's controls and what it draws; nothing while the fixed
    /// sun or the scheme casts.
    pub(super) fn sun_control(&mut self, ui: &mut Ui) {
        if !(self.sun && self.light.sun.on) {
            return;
        }
        let facing = self.sun_facing();
        let a = &mut self.light.sun;
        if aim_row(ui, (&mut a.bearing, &mut a.height), facing) {
            let (bearing, height) = (a.bearing, a.height);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("sun-disc facing={facing:.0} bearing={bearing:.0} height={height:.0}")
            });
        }
        let by = ui
            .checkbox(&mut a.by_height, g::sun_colour_by_height())
            .on_hover_text(g::sun_colour_by_height_tooltip());
        crate::diag::ui_rect_visible(REGION_SUN_BY_HEIGHT, by.rect, ui.clip_rect());
        if !a.by_height {
            ui.horizontal_wrapped(|ui| {
                ui.label(g::sun_kelvin_label())
                    .on_hover_text(g::sun_kelvin_tooltip());
                let slider = ui.add(
                    egui::Slider::new(&mut a.kelvin, KELVIN_RANGE)
                        .logarithmic(true)
                        .step_by(50.0)
                        .suffix(g::kelvin_suffix()),
                );
                crate::diag::ui_rect_visible(REGION_SUN_KELVIN, slider.rect, ui.clip_rect());
                slider.on_hover_text(g::sun_kelvin_tooltip());
            });
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(g::sun_strength_label())
                .on_hover_text(g::sun_strength_tooltip());
            let slider = ui.add(percent_slider(&mut a.strength));
            crate::diag::ui_rect_visible(REGION_SUN_STRENGTH, slider.rect, ui.clip_rect());
            slider.on_hover_text(g::sun_strength_tooltip());
        });
        if let ShadowLight::AimedSun(sun) = self.shadow_light() {
            ui.weak(g::sun_said(sun.kelvin()));
        }
    }
}

/// A logarithmic slider over [`GAIN_RANGE`] reading as a percentage.
fn percent_slider(gain: &mut f64) -> egui::Slider<'_> {
    egui::Slider::new(gain, GAIN_RANGE)
        .logarithmic(true)
        .custom_formatter(|v, _| g::percent(v))
        .custom_parser(|s| {
            let s = s.trim().trim_end_matches('%').trim();
            s.parse::<f64>().ok().map(|p| p / 100.0)
        })
}

/// The sun disc, `facing` at its bottom, beside the bearing and height
/// fields that follow it and can be typed into. True when the disc moved
/// the sun; a typed number moves the ball on the next frame.
pub(super) fn aim_row(ui: &mut Ui, (bearing, height): (&mut f64, &mut f64), facing: f64) -> bool {
    ui.horizontal(|ui| {
        let moved = sundisc::sun_disc(
            ui,
            REGION_SUN_DISC,
            g::sun_disc_tooltip(),
            (&mut *bearing, &mut *height),
            facing,
        );
        ui.vertical(|ui| {
            let (b, h) = (g::sun_bearing_tooltip(), g::sun_height_tooltip());
            degrees_field(
                ui,
                REGION_SUN_BEARING,
                g::sun_bearing_label(),
                b,
                bearing,
                BEARING_RANGE,
            );
            degrees_field(
                ui,
                REGION_SUN_HEIGHT,
                g::sun_height_label(),
                h,
                height,
                HEIGHT_RANGE,
            );
        });
        moved
    })
    .inner
}

/// A labelled whole-degree number field under `region`.
fn degrees_field(
    ui: &mut Ui,
    region: &str,
    label: &str,
    tip: &str,
    value: &mut f64,
    range: std::ops::RangeInclusive<f64>,
) {
    ui.label(label).on_hover_text(tip);
    let field = ui.add(
        egui::DragValue::new(value)
            .range(range)
            .speed(1.0)
            .max_decimals(0)
            .suffix(g::degrees_suffix()),
    );
    crate::diag::ui_rect_visible(region, field.rect, ui.clip_rect());
    field.on_hover_text(tip);
}

/// The bearing of `back` (target towards eye) on the ground spanned by
/// `front` and `right`, degrees in 0..360. Looking straight down
/// `overhead`, the screen's bottom — opposite the camera's `up` — stands in.
fn facing_from(
    back: [f64; 3],
    up: [f64; 3],
    (front, right, overhead): ([f64; 3], [f64; 3], [f64; 3]),
) -> f64 {
    let ground = |v: [f64; 3]| sub(v, scale(overhead, dot(v, overhead)));
    let flat = ground(back);
    let flat = if length(flat) > 1e-9 * length(back).max(1.0) {
        flat
    } else {
        scale(ground(up), -1.0)
    };
    if length(flat) <= 1e-12 {
        return 0.0;
    }
    dot(flat, right)
        .atan2(dot(flat, front))
        .to_degrees()
        .rem_euclid(360.0)
}

/// The render trace's light strength and sun fields for `options`.
pub(super) fn trace(options: &pdfcer_3d::RenderOptions) -> String {
    let sun = match options.shadow_light {
        ShadowLight::AimedSun(s) => format!(
            // ui-text-exempt: diagnostic trace, never displayed
            " sun-bearing={:.0} sun-height={:.0} sun-kelvin={:.0} sun-gain={:.3}",
            s.azimuth_deg,
            s.elevation_deg,
            s.kelvin(),
            s.gain
        ),
        _ => String::new(),
    };
    format!(" light-gain={:.3}{sun}", options.light_gain) // ui-text-exempt: diagnostic trace, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_disc_faces_the_side_the_camera_looks_from() {
        // Z up: front -Y, right +X, overhead +Z.
        let ground = ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let up = [0.0, 0.0, 1.0];
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(near(facing_from([0.0, -5.0, 2.0], up, ground), 0.0));
        assert!(near(facing_from([3.0, 0.0, 1.0], up, ground), 90.0));
        assert!(near(facing_from([0.0, 4.0, 0.0], up, ground), 180.0));
        // Straight down, screen top towards +Y: the bottom is the front.
        assert!(near(
            facing_from([0.0, 0.0, 9.0], [0.0, 1.0, 0.0], ground),
            0.0
        ));
    }

    #[test]
    fn each_scheme_keeps_its_own_brightness() {
        let mut light = Light::default();
        let day = slot(ThreeDLighting::Day).expect("Day is offered");
        light.gains[day] = 2.0;
        assert!((light.gain(ThreeDLighting::Day) - 2.0).abs() < 1e-12);
        assert!((light.gain(ThreeDLighting::Artwork) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn every_default_is_inside_its_range() {
        let a = SunAim::default();
        assert!(BEARING_RANGE.contains(&a.bearing));
        assert!(HEIGHT_RANGE.contains(&a.height));
        assert!(KELVIN_RANGE.contains(&a.kelvin));
        assert!(GAIN_RANGE.contains(&a.strength));
    }
}
