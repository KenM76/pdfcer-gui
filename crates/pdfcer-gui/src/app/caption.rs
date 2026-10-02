//! Keeps every OS window's title bar in the application's light or dark mode.
//!
//! Contract: after [`sync`] runs on a frame, every top-level window of this
//! process carries the effective theme's `dark_mode` as its caption mode. It
//! re-applies when that mode, the host's own mode or the number of viewports
//! changes — a new dialog, a host settings change that winit answers by
//! re-deriving its own caption state — and stays quiet otherwise.

/// What the captions were last set for.
type Key = (bool, Option<egui::Theme>, usize);

/// The `egui` temp-data slot holding the last [`Key`].
const SLOT: &str = "pdfcer-caption-theme"; // ui-text-exempt: data key, never displayed

/// Call after the frame's theme is applied.
pub(crate) fn sync(ctx: &egui::Context) {
    let dark = ctx.global_style().visuals.dark_mode;
    let key: Key = (
        dark,
        ctx.system_theme(),
        ctx.input(|i| i.raw.viewports.len()),
    );
    let id = egui::Id::new(SLOT);
    if ctx.data(|d| d.get_temp::<Key>(id)) == Some(key) {
        return;
    }
    let windows = native_window::set_dark_captions(dark);
    // No window yet means none was set: try again next frame.
    if windows == 0 {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(id, key));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace fields, never displayed
        format!("caption-theme dark={dark} windows={windows}")
    });
}
