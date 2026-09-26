//! **Graphics-memory pressure, made observable** — the blank page that
//! nothing reports. The ledger is `pdfcer_gui_base::pressure`; this is the
//! page-raster recorder and the per-frame read of the GL error flag.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pressure.md`.

use pdfcer_gui_base::pressure::record;
pub use pdfcer_gui_base::pressure::{
    Attribution, Raster, Surface, Unattributed, Upload, attribute, record_other,
};

use crate::render::worker::RenderKey;

/// Note that an upload of a **page raster** has been ordered this frame.
pub fn record_raster(ctx: &egui::Context, surface: Surface, key: &RenderKey, w: u32, h: u32) {
    record(
        ctx,
        Upload {
            surface,
            pixels: u64::from(w) * u64::from(h),
            raster: Some(Raster {
                page: key.page(),
                raster_scale: key.raster_scale(),
                whole_page: key.region().is_none(),
            }),
        },
    );
}

/// Read the error flag at the top of a frame and trace what it held.
pub fn poll(ctx: &egui::Context, gl: Option<&eframe::glow::Context>) {
    let uploads = pdfcer_gui_base::pressure::take(ctx);
    let Some(gl) = gl else { return };

    let drained = native_gl::drain(gl);
    if drained.is_clean() {
        return;
    }

    // The verdict is assembled inside the closure, not before it: `diag::trace`
    // takes a thunk precisely so that a build with tracing off pays nothing,
    // and a string formatted at the call site is paid for either way.
    crate::diag::trace(move || {
        let verdict = match attribute(&uploads, drained) {
            // `is_clean` was false, so `attribute` cannot return this. Spelled
            // out rather than unreachable-panicking: a trace is not worth a
            // crash.
            Attribution::Clean => "clean".to_owned(),
            Attribution::Blamed(u) => format!("blamed {}", u.trace_fragment()),
            Attribution::Unattributed(u) => format!("unattributed={u:?}"),
        };
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!(
            "gl-pressure oom={} count={} other={:?} truncated={} {verdict}",
            drained.out_of_memory,
            drained.count,
            drained.first_other,
            drained.truncated(),
        )
    });
}

/// Trace the device's real single-axis texture limit whenever it changes.
pub fn trace_texture_limit(ctx: &egui::Context) {
    let side = ctx.input(|i| i.max_texture_side);
    crate::diag::trace_on_change("gl-max-texture-side", move || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        format!("side={side}")
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_gui_base::pressure::take;

    /// The record survives being written and read back through `ctx.data`.
    #[test]
    fn a_recorded_upload_comes_back_out_once() {
        let ctx = egui::Context::default();
        let key = RenderKey::new(7, 4.0, true, 0, pdfcer_render::font::StrokeDisplay::Actual);
        record_raster(&ctx, Surface::Canvas, &key, 1_000, 2_000);
        record_other(&ctx, Surface::Icon, 32, 32);

        let first = take(&ctx);
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].surface, Surface::Canvas);
        assert_eq!(first[0].pixels, 2_000_000);
        assert_eq!(first[0].raster.map(|r| r.page), Some(7));
        assert!(first[0].raster.is_some_and(|r| r.whole_page));
        assert_eq!(first[1].raster, None);

        // And is GONE — a record read twice would be attributed to two
        // frames, the second of which uploaded nothing.
        assert!(take(&ctx).is_empty());
    }
}
