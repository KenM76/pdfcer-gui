//! The parts of the window the system draws over: Android's status bar,
//! navigation bar and display cutout.
//!
//! eframe reports the safe area only on iOS, so on Android the window is
//! edge-to-edge and every top and bottom panel draws under the system bars.
//! The entry point installs an [`Insets`] source; [`apply`] copies it into
//! `RawInput::safe_area_insets` each frame, and [`reserve`] then takes those
//! bands off the root `Ui` as empty panels before any other panel is laid out.
//! eframe's root `Ui` spans `Context::viewport_rect`, which includes the
//! covered bands, so the safe area alone moves nothing. On the desktop nothing
//! is installed and both do nothing.

use std::sync::OnceLock;

/// What the platform supplies: the system-bar insets.
pub trait Insets: Send + Sync {
    /// The insets in physical pixels, as `[left, top, right, bottom]`.
    fn system_bars_px(&self) -> [f32; 4];
}

static SOURCE: OnceLock<Box<dyn Insets>> = OnceLock::new();

/// Installs the insets source. The first call wins.
pub fn install(source: Box<dyn Insets>) {
    let _ = SOURCE.set(source);
}

/// Writes the installed source's insets, in points, into `raw`.
pub fn apply(ctx: &egui::Context, raw: &mut egui::RawInput) {
    let Some(source) = SOURCE.get() else {
        return;
    };
    let ppp = ctx.pixels_per_point().max(0.1);
    let [left, top, right, bottom] = source.system_bars_px().map(|px| px / ppp);
    raw.safe_area_insets = Some(egui::SafeAreaInsets(egui::epaint::MarginF32 {
        left,
        top,
        right,
        bottom,
    }));
}

/// Covers the system-bar bands with empty panels, so every later panel lays
/// out inside `Context::content_rect`. Call first in the frame.
pub fn reserve(ui: &mut egui::Ui) {
    let full = ui.ctx().viewport_rect();
    let safe = ui.ctx().content_rect();
    let band = |panel: egui::Panel, size: f32, ui: &mut egui::Ui| {
        if size > 0.0 {
            panel.exact_size(size).resizable(false).show(ui, |_| {});
        }
    };
    band(
        egui::Panel::top("pdfcer/safe-area/top"),
        safe.min.y - full.min.y,
        ui,
    );
    band(
        egui::Panel::bottom("pdfcer/safe-area/bottom"),
        full.max.y - safe.max.y,
        ui,
    );
    band(
        egui::Panel::left("pdfcer/safe-area/left"),
        safe.min.x - full.min.x,
        ui,
    );
    band(
        egui::Panel::right("pdfcer/safe-area/right"),
        full.max.x - safe.max.x,
        ui,
    );
}
