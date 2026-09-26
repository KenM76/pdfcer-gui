//! **Graphics-memory pressure, made observable** — the blank page that
//! nothing reports. The ledger is `pdfcer_gui_base::pressure`; this is the
//! per-frame read of the GL error flag.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pressure.md`.

pub use pdfcer_gui_base::pressure::{
    Attribution, Raster, Surface, Unattributed, Upload, attribute, record_other, record_raster,
};

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
