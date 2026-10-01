//! # `canvas::textedit::fallback` — why the open draft's preview is in a
//! stand-in font, for the status bar
//!
//! [`super::paint::preview`] publishes, every frame it draws a draft on an
//! existing run, whether that draft is shown in the run's own font and, when
//! not, why. [`live`] answers for the draft open now, so a closed or moved
//! draft reads `None` without anyone clearing the slot. The reason changing is
//! traced once as `text-edit-preview-fallback`.

use pdfcer_gui_base::text::previewfallback::PreviewFallback;

use super::{Anchor, Draft};

const KEY: &str = "textedit-preview-fallback"; // ui-text-exempt: a memory key, never displayed.

/// What [`publish`] last recorded: the draft it was about, and the reason.
#[derive(Clone, PartialEq)]
struct Held {
    page: usize,
    run: usize,
    why: Option<PreviewFallback>,
}

/// Record whether the draft's preview fell back, and why. `font_pt` is the
/// stand-in font's size in screen points, traced so a driven check can
/// compare it with the run's size times the zoom.
pub fn publish(ctx: &egui::Context, draft: &Draft, why: Option<PreviewFallback>, font_pt: f32) {
    let Anchor::Run { run, .. } = &draft.anchor else {
        return;
    };
    let held = Held {
        page: draft.page,
        run: *run,
        why,
    };
    let id = egui::Id::new(KEY);
    if ctx.data(|d| d.get_temp::<Held>(id)).as_ref() == Some(&held) {
        return;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "text-edit-preview-fallback page={} run={} reason={} font_pt={font_pt:.2}",
            held.page,
            held.run,
            why.map_or("none", PreviewFallback::token),
        )
    });
    ctx.data_mut(|d| d.insert_temp(id, held));
}

/// Why the open draft's preview is in a stand-in font, if it is.
#[must_use]
pub fn live(ctx: &egui::Context) -> Option<PreviewFallback> {
    let draft = super::read(ctx)?;
    let Anchor::Run { run, .. } = &draft.anchor else {
        return None;
    };
    let held = ctx.data(|d| d.get_temp::<Held>(egui::Id::new(KEY)))?;
    (held.page == draft.page && held.run == *run)
        .then_some(held.why)
        .flatten()
}
