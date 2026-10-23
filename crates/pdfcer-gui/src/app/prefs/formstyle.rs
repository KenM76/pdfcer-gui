//! # `app::prefs::formstyle` — writing Forms ▸ Style to the preferences file
//!
//! The look lives in `PdfcerApp::form_defaults` and changes from three places:
//! the Style band, *Use as default look*, and an accepted placement. Rather
//! than save at each, the frame compares the live look with the stored one and
//! saves on a difference — so a fourth writer cannot forget to. Saving waits
//! for every pointer button to be up, so dragging a width writes the file once
//! when the drag ends rather than once a frame during it. Its `form-style`
//! trace line, emitted on change, opens each process with the look it loaded.

use crate::app::prefs::Prefs;
use crate::canvas::formfield::Remembered;

/// Save `prefs` if `mem`'s look has moved away from what it holds.
pub fn keep(ctx: &egui::Context, prefs: &mut Prefs, mem: &Remembered) {
    let live = mem.chosen();
    crate::diag::trace_on_change("form-style", || match live {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        None => "set=0".to_owned(),
        Some(look) => format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "set=1 style={} font={:?} size={}",
            look.border_style.token(),
            look.text.font,
            look.text.size
        ),
    });
    if live == prefs.form_style.look || ctx.input(|i| i.pointer.any_down()) {
        return;
    }
    prefs.form_style.look = live;
    let saved = prefs.save().is_ok();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("form-style-saved ok={saved}")
    });
}
