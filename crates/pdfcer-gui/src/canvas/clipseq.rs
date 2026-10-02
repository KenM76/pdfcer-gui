//! Whether another program has written the clipboard since pdfcer last
//! copied, so a paste takes the newer content rather than pdfcer's own clip.
//!
//! Contract: [`mark`] is called when pdfcer stores a clip. pdfcer's own write
//! to the OS clipboard finishes after that frame (eframe applies `copy_text`
//! once the frame ends), so the counter is read at the start of the next
//! frame by [`settle`]. [`changed`] is false until then, and always false
//! where the counter is unavailable.

const PENDING: &str = "pdfcer.clip.sequence.pending"; // ui-text-exempt: temp-data key
const RECORDED: &str = "pdfcer.clip.sequence"; // ui-text-exempt: temp-data key

/// Note that a clip was just stored; its counter is read next frame.
pub fn mark(ctx: &egui::Context) {
    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new(PENDING), true);
        d.remove::<u32>(egui::Id::new(RECORDED));
    });
    ctx.request_repaint();
}

/// Record the clipboard counter for a clip [`mark`]ed last frame.
pub fn settle(ctx: &egui::Context) {
    let pending = ctx.data_mut(|d| d.remove_temp::<bool>(egui::Id::new(PENDING)));
    if pending == Some(true) {
        let now = pdfcer_gui_base::clippaste::sequence();
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(RECORDED), now));
        crate::diag::trace(|| format!("clip-sequence-recorded seq={now}"));
    }
}

/// Whether the clipboard has been written since pdfcer's clip was recorded.
#[must_use]
pub fn changed(ctx: &egui::Context) -> bool {
    let recorded = ctx.data(|d| d.get_temp::<u32>(egui::Id::new(RECORDED)));
    recorded.is_some_and(|seq| seq != 0 && seq != pdfcer_gui_base::clippaste::sequence())
}
