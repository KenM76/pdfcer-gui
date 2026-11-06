//! The options menu's Picture tab, and the picture *Save picture…* draws
//! with it, made off the screen's thread so the viewer stays live while a
//! large one is drawn.
//!
//! 1× is the picture as it is on screen, in device pixels. The width and
//! height keep that shape: an entry or the slider sets one scale and both
//! sides follow it. The scale stops at 8× or at the renderer's pixel
//! ceiling (`pdfcer_3d::MAX_RENDER_PIXELS`), whichever comes first, and the
//! ceiling is said when it is the one that stops it. The resolution is only
//! the PNG's `pHYs` record; it changes no pixel. A 360° picture replaces the
//! size and resolution with a choice of 2:1 sizes, and is a JPEG on an opaque
//! background (see `sphere`).

use std::sync::mpsc::{Receiver, TryRecvError};

use egui::Ui;
use pdfcer_3d::RenderOptions as ModelRenderOptions;
use pdfcer_3d::{MAX_LAMP_SHADOW_MAP_SIZE, MAX_RENDER_PIXELS, MAX_SHADOW_MAP_SIZE, render_model};

use super::{Drawn, ModelView, sphere};
use crate::text::model3dshot as s;

/// The width entry.
pub const REGION_SHOT_WIDTH: &str = "model3d.shot.width"; // ui-text-exempt: trace region name, never displayed
/// The height entry.
pub const REGION_SHOT_HEIGHT: &str = "model3d.shot.height"; // ui-text-exempt: trace region name, never displayed
/// The 1×–8× slider.
pub const REGION_SHOT_SCALE: &str = "model3d.shot.scale"; // ui-text-exempt: trace region name, never displayed
/// The resolution entry.
pub const REGION_SHOT_DPI: &str = "model3d.shot.dpi"; // ui-text-exempt: trace region name, never displayed
/// The background drop-down; its items add `.white`, `.screen`, `.clear`.
pub const REGION_SHOT_BACKGROUND: &str = "model3d.shot.background"; // ui-text-exempt: trace region name, never displayed
/// The shadow detail drop-down; its items add `.screen`, `.fine`, `.finest`.
pub const REGION_SHOT_DETAIL: &str = "model3d.shot.detail"; // ui-text-exempt: trace region name, never displayed
/// The 360° picture switch.
pub const REGION_SHOT_SPHERE: &str = "model3d.shot.sphere"; // ui-text-exempt: trace region name, never displayed
/// The 360° size drop-down; its items add `.small`, `.medium`, `.large`.
pub const REGION_SHOT_SPHERE_SIZE: &str = "model3d.shot.sphere_size"; // ui-text-exempt: trace region name, never displayed
/// The switch that widens lines with the picture.
pub const REGION_SHOT_LINES: &str = "model3d.shot.lines"; // ui-text-exempt: trace region name, never displayed

/// The scale slider's rail, points; its region is the rail alone, not its label.
const RAIL: f32 = 150.0;
/// The largest scale, times the picture on screen.
const MOST: f64 = 8.0;
/// The shortest long side an entry may ask for, pixels.
const SMALLEST: f64 = 16.0;
/// The resolutions an entry may ask for, pixels per inch.
const DPI: std::ops::RangeInclusive<u32> = 72..=2400;

/// The [`BACKDROPS`] entry a JPEG cannot hold.
const CLEAR: usize = 2;
/// The 360° sizes' region words, smallest first.
const SPHERE_WORDS: [&str; 3] = ["small", "medium", "large"]; // ui-text-exempt: trace region names, never displayed

/// What is behind the model: the trace words are `white`, `screen`, `clear`.
const BACKDROPS: [&str; 3] = ["white", "screen", "clear"]; // ui-text-exempt: trace values, never displayed
/// The shadow map's growth over the screen's: `screen`, `fine`, `finest`.
const DETAILS: [(&str, u32); 3] = [("screen", 1), ("fine", 2), ("finest", 4)]; // ui-text-exempt: trace values, never displayed

/// The Picture tab's settings, the screen's size and background they are
/// measured from, and the picture being drawn.
pub(super) struct Shot {
    /// Times the picture on screen.
    pub(super) scale: f64,
    /// Pixels per inch, written into the PNG.
    pub(super) dpi: u32,
    /// An index into [`BACKDROPS`].
    pub(super) backdrop: usize,
    /// An index into [`DETAILS`].
    pub(super) detail: usize,
    /// Lines widen with the scale.
    pub(super) wide_lines: bool,
    /// The picture on screen, device pixels: 1×.
    pub(super) base: [u32; 2],
    /// The screen's background, straight RGBA.
    pub(super) screen: [u8; 4],
    /// The picture being drawn and its trace line's head.
    drawing: Option<(Receiver<Result<Drawn, String>>, String)>,
    /// The picture's own look, else the screen's.
    pub(super) own: Option<super::shotlook::LookSet>,
    /// *Save picture…* draws the whole sphere round the eye.
    pub(super) sphere: bool,
    /// An index into [`sphere::SIZES`].
    pub(super) sphere_size: usize,
    /// The picture being drawn, or the last, is a 360° one.
    drawn_sphere: bool,
}

impl Default for Shot {
    fn default() -> Self {
        Self {
            scale: 1.0,
            dpi: 96,
            backdrop: 0,
            detail: 0,
            wide_lines: true,
            base: [4, 3],
            screen: [255; 4],
            drawing: None,
            own: None,
            sphere: false,
            sphere_size: 1,
            drawn_sphere: false,
        }
    }
}

impl Shot {
    /// The largest scale: [`MOST`], or less where the pixel ceiling is met.
    fn most(&self) -> f64 {
        let [w, h] = self.base.map(|b| f64::from(b.max(1)));
        (MAX_RENDER_PIXELS as f64 / (w * h)).sqrt().min(MOST)
    }

    /// The smallest scale an entry may ask for.
    fn least(&self) -> f64 {
        SMALLEST / f64::from(self.base[0].max(self.base[1]).max(1))
    }

    /// The picture's size at `scale`, pixels.
    fn size_at(&self, scale: f64) -> [u32; 2] {
        self.base
            .map(|b| (f64::from(b.max(1)) * scale + 1e-6).floor().max(1.0) as u32)
    }

    /// The picture's size, pixels.
    pub(super) fn size(&self) -> [u32; 2] {
        if self.sphere {
            return sphere::SIZES[self.sphere_size.min(sphere::SIZES.len() - 1)];
        }
        self.size_at(self.scale.clamp(self.least(), self.most()))
    }

    /// The background the picture is drawn on: a 360° picture has no clear
    /// one, and draws on white while the setting stays as chosen.
    fn backdrop_drawn(&self) -> usize {
        if self.sphere && self.backdrop == CLEAR {
            0
        } else {
            self.backdrop
        }
    }

    /// `drawn`'s file bytes, and whether they are a 360° picture's.
    pub(super) fn encoded(&self, drawn: &Drawn) -> Result<(Vec<u8>, bool), String> {
        if self.drawn_sphere {
            Ok((super::picture::sphere_jpeg(drawn)?, true))
        } else {
            Ok((super::picture::png(drawn, self.png_dpi())?, false))
        }
    }

    /// Set the scale so side `axis` is `pixels` long.
    fn set_side(&mut self, axis: usize, pixels: f64) {
        let base = f64::from(self.base[axis].max(1));
        self.scale = (pixels / base).clamp(self.least(), self.most());
    }

    /// `options` drawn at this size, background, shadow detail and line
    /// width.
    fn apply(&self, options: &mut ModelRenderOptions) {
        let [width, height] = self.size();
        options.width = width;
        options.height = height;
        match BACKDROPS[self.backdrop_drawn()] {
            "screen" => options.background = self.screen,
            "clear" => options.background[3] = 0,
            _ => {}
        }
        let k = DETAILS[self.detail].1;
        options.shadow_map_size = (options.shadow_map_size * k).min(MAX_SHADOW_MAP_SIZE);
        options.lamp_shadow_map_size =
            (options.lamp_shadow_map_size * k).min(MAX_LAMP_SHADOW_MAP_SIZE);
        if self.wide_lines {
            let scale = if self.sphere {
                f64::from(width) / f64::from(self.base[0].max(1))
            } else {
                self.scale
            };
            let wide = f64::from(options.line_width.max(1)) * scale.max(1.0);
            options.line_width = wide.round() as u32;
        }
    }

    /// The resolution, as the PNG encoder takes it.
    pub(super) fn png_dpi(&self) -> f32 {
        self.dpi as f32
    }

    /// ` base=WxH scale=s w= h= dpi= background= detail= lines= sphere=`, for the
    /// trace a driven check reads.
    fn trace(&self) -> String {
        let [bw, bh] = self.base;
        let [w, h] = self.size();
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            " base={bw}x{bh} scale={:.3} w={w} h={h} dpi={} background={} detail={} lines={} sphere={}",
            self.scale.clamp(self.least(), self.most()),
            self.dpi,
            BACKDROPS[self.backdrop_drawn()],
            DETAILS[self.detail].0,
            u8::from(self.wide_lines),
            u8::from(self.sphere)
        )
    }
}

impl ModelView {
    /// The Picture tab: the size, the resolution and the finish.
    pub(super) fn shot_control(&mut self, ui: &mut Ui) {
        let before = self.shot.trace();
        ui.weak(s::note());
        self.shot_sphere(ui);
        if !self.shot.sphere {
            self.shot_size(ui);
            self.shot_dpi(ui);
        }
        self.shot_finish(ui);
        ui.separator();
        self.shot_look_control(ui);
        let after = self.shot.trace();
        if after != before {
            crate::diag::trace(|| format!("model-view-picture-size{after}")); // ui-text-exempt: diagnostic trace, never displayed
        }
    }

    /// The 360° switch, and the sizes when it is on.
    fn shot_sphere(&mut self, ui: &mut Ui) {
        let shot = &mut self.shot;
        let switch = ui
            .checkbox(&mut shot.sphere, s::SPHERE)
            .on_hover_text(s::SPHERE_TOOLTIP);
        crate::diag::ui_rect_visible(REGION_SHOT_SPHERE, switch.rect, ui.clip_rect());
        if shot.sphere {
            let words = SPHERE_WORDS.map(str::to_owned);
            ui.horizontal_wrapped(|ui| {
                choice(
                    ui,
                    (s::SPHERE_SIZE, s::SPHERE_SIZE_TOOLTIP),
                    REGION_SHOT_SPHERE_SIZE,
                    &words,
                    s::sphere_size,
                    &mut shot.sphere_size,
                    None,
                );
            });
        }
    }

    /// The width and height entries, the slider, and the ceiling when met.
    fn shot_size(&mut self, ui: &mut Ui) {
        let shot = &mut self.shot;
        let [w, h] = shot.size();
        let [most_w, most_h] = shot.size_at(shot.most());
        ui.horizontal_wrapped(|ui| {
            ui.label(s::size_label());
            for (axis, value, most, region) in [
                (0, w, most_w, REGION_SHOT_WIDTH),
                (1, h, most_h, REGION_SHOT_HEIGHT),
            ] {
                let mut pixels = f64::from(value);
                let entry = ui
                    .add(
                        egui::DragValue::new(&mut pixels)
                            .range(1.0..=f64::from(most))
                            .speed(1.0)
                            .fixed_decimals(0)
                            .suffix(s::pixels()),
                    )
                    .on_hover_text(s::size_tooltip());
                crate::diag::ui_rect_visible(region, entry.rect, ui.clip_rect());
                if entry.changed() {
                    shot.set_side(axis, pixels);
                }
                if axis == 0 {
                    ui.label("×");
                }
            }
        });
        let most = shot.most();
        let width = std::mem::replace(&mut ui.spacing_mut().slider_width, RAIL);
        let slider = ui
            .add(
                egui::Slider::new(&mut shot.scale, 1.0_f64.min(most)..=most)
                    .clamping(egui::SliderClamping::Never)
                    .custom_formatter(|v, _| s::scale(v))
                    .text(s::scale_label()),
            )
            .on_hover_text(s::scale_tooltip(shot.base[0], shot.base[1]));
        ui.spacing_mut().slider_width = width;
        let rail =
            egui::Rect::from_min_size(slider.rect.min, egui::vec2(RAIL, slider.rect.height()));
        crate::diag::ui_rect_visible(REGION_SHOT_SCALE, rail, ui.clip_rect());
        shot.scale = shot.scale.clamp(shot.least(), most);
        if most < MOST {
            ui.weak(s::limited(most_w, most_h));
        }
    }

    /// The resolution entry and the size it prints at.
    fn shot_dpi(&mut self, ui: &mut Ui) {
        let shot = &mut self.shot;
        ui.horizontal_wrapped(|ui| {
            ui.label(s::dpi_label());
            let entry = ui
                .add(
                    egui::DragValue::new(&mut shot.dpi)
                        .range(DPI)
                        .suffix(s::dpi_unit()),
                )
                .on_hover_text(s::dpi_tooltip());
            crate::diag::ui_rect_visible(REGION_SHOT_DPI, entry.rect, ui.clip_rect());
        });
        let inches = shot
            .size()
            .map(|side| f64::from(side) / f64::from(shot.dpi.max(1)));
        ui.weak(s::print_size(inches));
    }

    /// The background and shadow detail drop-downs and the line switch.
    fn shot_finish(&mut self, ui: &mut Ui) {
        let shot = &mut self.shot;
        let words = BACKDROPS.map(|w| w.to_owned());
        let mut backdrop = shot.backdrop_drawn();
        ui.horizontal_wrapped(|ui| {
            choice(
                ui,
                (s::background_label(), s::background_tooltip()),
                REGION_SHOT_BACKGROUND,
                &words,
                s::background,
                &mut backdrop,
                shot.sphere.then_some((CLEAR, s::SPHERE_NOT_CLEAR)),
            );
        });
        if backdrop != shot.backdrop_drawn() {
            shot.backdrop = backdrop;
        }
        let words = DETAILS.map(|(w, _)| w.to_owned());
        ui.horizontal_wrapped(|ui| {
            choice(
                ui,
                (s::detail_label(), s::detail_tooltip()),
                REGION_SHOT_DETAIL,
                &words,
                s::detail,
                &mut shot.detail,
                None,
            );
        });
        let lines = ui
            .checkbox(&mut shot.wide_lines, s::wide_lines())
            .on_hover_text(s::wide_lines_tooltip());
        crate::diag::ui_rect_visible(REGION_SHOT_LINES, lines.rect, ui.clip_rect());
    }

    /// Start drawing *Save picture…*'s picture of this view on its own
    /// thread, from a copy of the model as it is shown.
    pub(super) fn start_shot(&mut self) -> Result<(), String> {
        let [w, h] = self.shot.size();
        let aspect = f64::from(w) / f64::from(h);
        let mut options = ModelRenderOptions {
            threads: 0,
            ..self.in_shot_look(|view| view.look()).options()
        };
        self.shot.apply(&mut options);
        let mut camera = self.view_camera(aspect).map_err(|e| e.to_string())?;
        self.shot.drawn_sphere = self.shot.sphere;
        if self.shot.sphere {
            camera = camera
                .levelled_panorama(self.upright.up.vector(), self.upright.front.vector())
                .ok_or_else(|| s::SPHERE_NO_HEADING.to_owned())?;
        }
        let model = self
            .parts
            .painted(&mut self.model.drawn, None, Clone::clone);
        let head = format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "model-view-poster yaw={:.3} pitch={:.3}{} look={} shadows={} occlusion={} {}",
            self.orbit.yaw,
            self.orbit.pitch,
            self.shot.trace(),
            if self.shot.own.is_some() {
                "own"
            } else {
                "screen"
            }, // ui-text-exempt: trace value, never displayed
            u8::from(options.shadows),
            u8::from(options.ambient_occlusion),
            super::picture::aim_of(&camera)
        );
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("model-picture".to_owned()) // ui-text-exempt: thread name, never displayed
            .spawn(move || {
                let drawn = render_model(&model, &camera, &options)
                    .map(|image| Drawn {
                        width: image.width,
                        height: image.height,
                        rgba: image.rgba,
                    })
                    .map_err(|e| e.to_string());
                // The viewer closed: nothing waits for the picture.
                let _ = send.send(drawn);
            })
            .map_err(|e| e.to_string())?;
        self.shot.drawing = Some((receive, head));
        Ok(())
    }

    /// A picture is being drawn.
    pub(super) fn shot_drawing(&self) -> bool {
        self.shot.drawing.is_some()
    }

    /// The picture being drawn, once it is; asks for a frame until then.
    pub(super) fn finished_shot(&mut self, ctx: &egui::Context) -> Option<Result<Drawn, String>> {
        let (receive, _) = self.shot.drawing.as_ref()?;
        let done = match receive.try_recv() {
            Ok(done) => done,
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
                return None;
            }
            Err(TryRecvError::Disconnected) => Err(s::lost().to_owned()),
        };
        let (_, head) = self.shot.drawing.take()?;
        let bytes = done.as_ref().map_or(0, |d| d.rgba.len());
        crate::diag::trace(|| format!("{head} bytes={bytes}")); // ui-text-exempt: diagnostic trace, never displayed
        Some(done)
    }
}

/// A label and a drop-down of `words.len()` items named by `name`, the
/// button published as `region` and each item as `region.<word>`; `barred`
/// greys one item and says why.
fn choice(
    ui: &mut Ui,
    (label, tip): (&str, &str),
    region: &str,
    words: &[String],
    name: fn(usize) -> &'static str,
    chosen: &mut usize,
    barred: Option<(usize, &str)>,
) {
    ui.label(label);
    let combo = egui::ComboBox::from_id_salt(region)
        .selected_text(name(*chosen))
        .show_ui(ui, |ui| {
            for (i, word) in words.iter().enumerate() {
                let why = barred.filter(|(at, _)| *at == i).map(|(_, why)| why);
                let item = ui
                    .add_enabled_ui(why.is_none(), |ui| {
                        ui.selectable_value(chosen, i, name(i))
                            .on_disabled_hover_text(why.unwrap_or_default())
                    })
                    .inner;
                crate::diag::ui_rect_visible(
                    &format!("{region}.{word}"),
                    item.rect,
                    ui.clip_rect(),
                );
            }
        });
    crate::diag::ui_rect_visible(region, combo.response.rect, ui.clip_rect());
    combo.response.on_hover_text(tip);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(base: [u32; 2]) -> Shot {
        Shot {
            base,
            ..Shot::default()
        }
    }

    #[test]
    fn one_side_sets_the_other_at_the_screens_shape() {
        let mut shot = at([800, 500]);
        shot.set_side(0, 2000.0);
        assert_eq!(shot.size(), [2000, 1250]);
        shot.set_side(1, 1000.0);
        assert_eq!(shot.size(), [1600, 1000]);
    }

    #[test]
    fn the_scale_stops_at_eight_or_at_the_ceiling() {
        let mut shot = at([800, 500]);
        shot.set_side(0, 1.0e9);
        assert_eq!(shot.size(), [6400, 4000]);
        let mut big = at([4000, 3000]);
        big.set_side(0, 1.0e9);
        let [w, h] = big.size();
        assert!(
            u64::from(w) * u64::from(h) <= MAX_RENDER_PIXELS,
            "{w} x {h}"
        );
        assert!(big.most() < MOST);
    }
}
