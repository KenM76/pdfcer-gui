//! The print preview showing a page's separation plates, one ink at a time,
//! under Colour ▸ *Separations* (O438).
//!
//! A plate is the engine's labelled plate (`pdfcer_render::plates::render_plates`),
//! the call the spooler makes, so the preview shows the sheet that prints.
//!
//! # Contract
//!
//! - **Off the UI thread.** A page's plates are one render per ink plus a
//!   probe, seconds on a dense page, so they render on a thread; until they
//!   land the sheet shows the page in colour and the strip says the inks are
//!   being separated. One render at a time; a page left before its plates land
//!   is cancelled.
//! - **Held across pages.** Finished sets are kept under the page's colour
//!   [`PreviewKey`], most recently shown last, up to [`HELD_BYTES`]; going back
//!   to a page shows its plates without rendering again. The key carries the
//!   document stamp and the rendering options, so an edit or a changed option
//!   cannot show a stale set.
//! - A page the engine cannot separate shows in colour and the strip says so,
//!   as the receipt does. N-up, booklet and poster sheets print in colour and
//!   have no plates to show.
//! - Plates are drawn at the preview's base resolution: the zoomed detail
//!   render is skipped while a plate shows, since it renders in colour.

use std::collections::VecDeque;
use std::sync::{Arc, mpsc};

use egui::Ui;
use pdfcer_gui_base::icons::{self, Icon};
use pdfcer_gui_base::printpreviewkey::PreviewKey;
use pdfcer_gui_base::text::print as t;
use pdfcer_render::cancel::RenderCancel;
use pdfcer_render::plates::Plates;
use pdfcer_render::tiny_skia::Pixmap;

use super::super::PrintDialog;

/// The strip height the plates controls add under separations: one row of
/// glyph buttons above the small readout they share a line with.
pub(super) const ROW_PTS: f32 = 28.0;

/// Plate bytes held across pages. A letter page at the preview's 150 dpi is
/// 8.4 MB a plate, so this keeps the last five or six pages of a CMYK job.
const HELD_BYTES: usize = 384 << 20;

/// Trace region of the plates switch.
pub(super) const REGION_SWITCH: &str = "print.preview.plates";
/// Trace region of the previous-ink arrow.
pub(super) const REGION_PREVIOUS: &str = "print.preview.plate-previous";
/// Trace region of the next-ink arrow.
pub(super) const REGION_NEXT: &str = "print.preview.plate-next";

/// The plates of one page, as the engine rendered them.
enum Set {
    /// One `(ink, plate)` per ink, in plate order.
    Separated(Vec<(String, Pixmap)>),
    /// The engine could not separate the page; it shows and prints in colour.
    Refused,
    /// The render failed; the page shows in colour until the set is forgotten.
    Failed,
}

impl Set {
    fn bytes(&self) -> usize {
        match self {
            Self::Separated(plates) => plates.iter().map(|(_, p)| p.data().len()).sum(),
            Self::Refused | Self::Failed => 0,
        }
    }
}

/// A page's plates rendering on a thread.
struct Pending {
    key: PreviewKey,
    cancel: RenderCancel,
    rx: mpsc::Receiver<Set>,
}

/// What [`PlateView::start`] renders.
pub(super) struct Job {
    pub(super) session: Arc<pdfcer_core::edit::EditSession>,
    pub(super) page: pdfcer_core::page_tree::Page,
    /// Document page index, for the trace.
    pub(super) index: usize,
    pub(super) scale: f32,
    pub(super) options: pdfcer_render::RenderOptions,
}

/// Whether the preview shows plates, which one, and the sets held.
#[derive(Default)]
pub(in crate::dialogs::print) struct PlateView {
    /// The switch: show plates in place of the page in colour.
    on: bool,
    /// The plate shown, in the page's plate order.
    ink: usize,
    /// The colour key of the sheet shown, set each frame the sheet draws.
    shown: Option<PreviewKey>,
    /// Finished sets, least recently shown first.
    held: VecDeque<(PreviewKey, Arc<Set>)>,
    pending: Option<Pending>,
}

impl Drop for PlateView {
    fn drop(&mut self) {
        if let Some(pending) = &self.pending {
            pending.cancel.cancel();
        }
    }
}

impl PlateView {
    /// Whether this sheet shows plates: separations are chosen, the switch is
    /// on, the sheet is a plain page and the engine did not refuse it.
    pub(super) fn wanted(dialog: &PrintDialog, plain: bool, base: &PreviewKey) -> bool {
        dialog.separations
            && dialog.plates.on
            && plain
            && !matches!(dialog.plates.set(base), Some(Set::Refused | Set::Failed))
    }

    /// The set held for `base`, if one is.
    fn set(&self, base: &PreviewKey) -> Option<&Set> {
        self.held
            .iter()
            .find(|(key, _)| key == base)
            .map(|(_, set)| set.as_ref())
    }

    /// Take a finished render, record `base` as the sheet shown, and say
    /// whether `base`'s plates still need starting. Called once per frame the
    /// sheet draws.
    pub(super) fn frame(&mut self, ctx: &egui::Context, base: &PreviewKey, wanted: bool) -> bool {
        self.collect();
        if self.shown.as_ref() != Some(base) {
            self.shown = Some(base.clone());
            let at = self.held.iter().position(|(key, _)| key == base);
            if let Some(entry) = at.and_then(|at| self.held.remove(at)) {
                self.held.push_back(entry);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("print-preview-plates-held held={}", self.held.len())
                });
            }
        }
        if !wanted || self.set(base).is_some() {
            return false;
        }
        if let Some(pending) = &self.pending {
            if pending.key == *base {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
                return false;
            }
            pending.cancel.cancel();
            self.pending = None;
        }
        true
    }

    /// Render `base`'s plates on a thread.
    pub(super) fn start(&mut self, ctx: &egui::Context, base: &PreviewKey, mut job: Job) {
        let cancel = RenderCancel::new();
        job.options.cancel = Some(cancel.clone());
        let (tx, rx) = mpsc::channel();
        let repaint = ctx.clone();
        let stop = cancel.clone();
        let spawned = std::thread::Builder::new()
            .name("print-preview-plates".into())
            .spawn(move || {
                let started = std::time::Instant::now();
                let view = job.session.view();
                let rendered =
                    pdfcer_render::plates::render_plates(&view, &job.page, job.scale, &job.options);
                if stop.is_cancelled() {
                    trace_plates(job.index, started, "cancelled", 0);
                    return;
                }
                let set = match rendered {
                    Ok(Plates::Separated(plates)) => {
                        Set::Separated(plates.into_iter().map(|p| (p.ink, p.pixmap)).collect())
                    }
                    Ok(_) => Set::Refused,
                    Err(_) => Set::Failed,
                };
                let (outcome, inks) = match &set {
                    Set::Separated(p) => ("separated", p.len()),
                    Set::Refused => ("refused", 0),
                    Set::Failed => ("failed", 0),
                };
                trace_plates(job.index, started, outcome, inks);
                let _ = tx.send(set);
                repaint.request_repaint();
            });
        if spawned.is_ok() {
            self.pending = Some(Pending {
                key: base.clone(),
                cancel,
                rx,
            });
        }
    }

    /// Hold a finished render, if one has arrived, evicting the least
    /// recently shown sets past [`HELD_BYTES`]; the one just held is kept.
    fn collect(&mut self) {
        let Some(pending) = &self.pending else {
            return;
        };
        let set = match pending.rx.try_recv() {
            Ok(set) => set,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Set::Failed,
        };
        let Some(Pending { key, .. }) = self.pending.take() else {
            return;
        };
        self.held.push_back((key, Arc::new(set)));
        let mut total: usize = self.held.iter().map(|(_, s)| s.bytes()).sum();
        while total > HELD_BYTES && self.held.len() > 1 {
            let Some((_, dropped)) = self.held.pop_front() else {
                break;
            };
            total -= dropped.bytes();
        }
    }

    /// The key of the picture this sheet shows: the shown plate once `base`'s
    /// plates are held and [`Self::wanted`], else `base` in colour.
    pub(super) fn key(dialog: &mut PrintDialog, plain: bool, base: &PreviewKey) -> PreviewKey {
        if Self::wanted(dialog, plain, base)
            && let Some(Set::Separated(plates)) = dialog.plates.set(base)
        {
            let index = dialog.plates.ink.min(plates.len().saturating_sub(1));
            dialog.plates.ink = index;
            base.clone().with_plate(Some(index))
        } else {
            base.clone()
        }
    }

    /// The plate `key` names, from `base`'s held set. `None` when `key` names
    /// no plate or the set is not held.
    pub(super) fn pixmap(&self, base: &PreviewKey, key: &PreviewKey) -> Option<Pixmap> {
        let index = key.plate()?;
        let Some(Set::Separated(plates)) = self.set(base) else {
            return None;
        };
        let (ink, pixmap) = plates.get(index)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "print-preview-plate index={index} of={} ink={ink}",
                plates.len()
            )
        });
        Some(pixmap.clone())
    }

    /// Forget the sets that refused or failed, so they are asked again.
    fn forget_refusals(&mut self) {
        self.held
            .retain(|(_, set)| matches!(set.as_ref(), Set::Separated(_)));
    }
}

/// One plates render's outcome and wall time.
fn trace_plates(page: usize, started: std::time::Instant, outcome: &str, inks: usize) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "print-preview-plates page={page} ms={} outcome={outcome} inks={inks}",
            started.elapsed().as_millis()
        )
    });
}

/// The strip's plates row: the switch, and with it on the ink arrows side by
/// side and then the shown ink's name, so a longer or shorter name never moves
/// an arrow. Drawn only under separations.
pub(super) fn row(ui: &mut Ui, dialog: &mut PrintDialog, plain: bool) {
    if !dialog.separations {
        return;
    }
    let switch = ui
        .add(icons::glyph(ui, Icon::Separations).selected(dialog.plates.on))
        .on_hover_text(t::preview_plates())
        .on_hover_text(t::preview_plates_tooltip());
    crate::diag::ui_rect_visible(REGION_SWITCH, switch.rect, ui.clip_rect());
    if switch.clicked() {
        dialog.plates.on = !dialog.plates.on;
        dialog.plates.forget_refusals();
    }
    if !dialog.plates.on {
        return;
    }
    if !plain {
        ui.label(t::preview_plates_imposed());
        return;
    }
    let plates = &dialog.plates;
    let set = plates.shown.as_ref().and_then(|key| plates.set(key));
    let inks: Vec<String> = match set {
        Some(Set::Separated(plates)) => plates.iter().map(|(ink, _)| ink.clone()).collect(),
        Some(Set::Refused) => {
            ui.label(t::preview_plates_refused());
            return;
        }
        Some(Set::Failed) => return,
        None => {
            ui.add(egui::Spinner::new().color(ui.visuals().text_color()));
            ui.label(t::preview_plates_separating());
            return;
        }
    };
    let shown = dialog.plates.ink.min(inks.len().saturating_sub(1));
    let previous = ui
        .add(icons::glyph(ui, Icon::ChevronLeft))
        .on_hover_text(t::preview_plate_previous());
    crate::diag::ui_rect_visible(REGION_PREVIOUS, previous.rect, ui.clip_rect());
    if previous.clicked() {
        dialog.plates.ink = shown.saturating_sub(1);
    }
    let next = ui
        .add(icons::glyph(ui, Icon::ChevronRight))
        .on_hover_text(t::preview_plate_next());
    crate::diag::ui_rect_visible(REGION_NEXT, next.rect, ui.clip_rect());
    if next.clicked() && shown + 1 < inks.len() {
        dialog.plates.ink = shown + 1;
    }
    if let Some(ink) = inks.get(shown) {
        ui.label(t::preview_plate_position(ink, shown + 1, inks.len()));
    }
}

/// Start `base`'s plates for document page `page`.
pub(super) fn start(
    ctx: &egui::Context,
    inputs: &super::Inputs<'_>,
    dialog: &mut PrintDialog,
    page: usize,
    base: &PreviewKey,
) {
    if let Some(job) = job(inputs, dialog, page, base) {
        dialog.plates.start(ctx, base, job);
    }
}

/// The render of document page `page` under `key`, with the options and
/// scale the spooler's builder gives the preview. `None` when the page is
/// not in the document.
pub(super) fn job(
    inputs: &super::Inputs<'_>,
    dialog: &PrintDialog,
    page: usize,
    key: &PreviewKey,
) -> Option<Job> {
    let copy = dialog.running.copy();
    let source = super::super::running::Source::of(inputs.doc, copy.as_deref());
    let page_obj = source.pages.get(page)?;
    let size = *inputs.page_sizes.get(page)?;
    let scale = super::raster_scale(size);
    let mut options = super::super::commit::render_options(source.scope(dialog.scope), inputs.doc);
    key.apply_lines(&mut options, f64::from(scale));
    Some(Job {
        session: Arc::clone(source.session),
        page: page_obj.clone(),
        index: page,
        scale,
        options,
    })
}
