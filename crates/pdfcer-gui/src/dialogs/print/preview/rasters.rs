//! The print preview's sheets in colour, rendered off the UI thread and held
//! across sheets.
//!
//! # Contract
//!
//! - **Off the UI thread.** A dense page renders for seconds; on the UI
//!   thread that froze the strip's arrows with it. One render at a time; a
//!   sheet left before it lands is cancelled. Until it lands the preview
//!   draws the sheet's flat fill, never the previous sheet's picture: a
//!   picture of the wrong page under this page's number would be wrong.
//! - **Held across sheets.** Finished rasters are kept under their
//!   [`PreviewKey`], most recently shown last, up to [`HELD_BYTES`]. They are
//!   the engine's raster before the printer profile and recolouring, which
//!   are applied on upload, so changing those reuses what is held. The key
//!   carries the document stamp and the rendering options, so an edit or a
//!   changed option cannot show a stale raster.

use std::collections::VecDeque;
use std::sync::{Arc, mpsc};

use pdfcer_gui_base::printpreviewkey::PreviewKey;
use pdfcer_render::cancel::RenderCancel;
use pdfcer_render::tiny_skia::Pixmap;

use super::plates::Job;

/// Colour raster bytes held across sheets: about fifteen letter sheets at the
/// preview's 150 dpi.
const HELD_BYTES: usize = 128 << 20;

/// A sheet rendering on a thread. `None` from the thread is a failed render.
struct Pending {
    key: PreviewKey,
    cancel: RenderCancel,
    rx: mpsc::Receiver<Option<Arc<Pixmap>>>,
}

/// What [`Rasters::get`] found for a key.
pub(super) enum Found {
    /// The raster, held.
    Held(Arc<Pixmap>),
    /// The render failed; the sheet draws its flat fill.
    Failed,
    /// It is rendering; the sheet draws its flat fill until it lands.
    Rendering,
    /// Not held and not rendering: the caller starts it.
    Missing,
}

/// The colour rasters held, and the one rendering.
#[derive(Default)]
pub(in crate::dialogs::print) struct Rasters {
    /// Finished rasters, least recently shown first. `None` is a failed render.
    held: VecDeque<(PreviewKey, Option<Arc<Pixmap>>)>,
    pending: Option<Pending>,
}

impl Drop for Rasters {
    fn drop(&mut self) {
        if let Some(pending) = &self.pending {
            pending.cancel.cancel();
        }
    }
}

impl Rasters {
    /// `key`'s raster, collecting a finished render first. A render for
    /// another key is cancelled, so only the sheet on screen renders.
    pub(super) fn get(&mut self, ctx: &egui::Context, key: &PreviewKey) -> Found {
        self.collect();
        let at = self.held.iter().position(|(k, _)| k == key);
        if let Some(entry) = at.and_then(|at| self.held.remove(at)) {
            let found = entry.1.clone().map_or(Found::Failed, Found::Held);
            self.held.push_back(entry);
            return found;
        }
        if let Some(pending) = &self.pending {
            if pending.key == *key {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
                return Found::Rendering;
            }
            pending.cancel.cancel();
            self.pending = None;
        }
        Found::Missing
    }

    /// Whether a sheet is rendering.
    pub(super) fn rendering(&self) -> bool {
        self.pending.is_some()
    }

    /// Render `key`'s sheet on a thread.
    pub(super) fn start(&mut self, ctx: &egui::Context, key: &PreviewKey, mut job: Job) {
        let cancel = RenderCancel::new();
        job.options.cancel = Some(cancel.clone());
        let (tx, rx) = mpsc::channel();
        let repaint = ctx.clone();
        let stop = cancel.clone();
        let spawned = std::thread::Builder::new()
            .name("print-preview-sheet".into())
            .spawn(move || {
                let started = std::time::Instant::now();
                let view = job.session.view();
                let rendered =
                    pdfcer_render::render_page_with_view(&view, &job.page, job.scale, &job.options);
                let outcome = if stop.is_cancelled() {
                    "cancelled"
                } else if rendered.is_ok() {
                    "rendered"
                } else {
                    "failed"
                };
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "print-preview-sheet page={} ms={} outcome={outcome}",
                        job.index,
                        started.elapsed().as_millis()
                    )
                });
                if stop.is_cancelled() {
                    return;
                }
                let _ = tx.send(rendered.ok().map(|r| Arc::new(r.pixmap)));
                repaint.request_repaint();
            });
        if spawned.is_ok() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            self.pending = Some(Pending {
                key: key.clone(),
                cancel,
                rx,
            });
        }
    }

    /// Hold a finished render, if one has arrived, evicting the least
    /// recently shown past [`HELD_BYTES`]; the one just held is kept.
    fn collect(&mut self) {
        let Some(pending) = &self.pending else {
            return;
        };
        let raster = match pending.rx.try_recv() {
            Ok(raster) => raster,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => None,
        };
        let Some(Pending { key, .. }) = self.pending.take() else {
            return;
        };
        self.held.push_back((key, raster));
        let bytes = |r: &Option<Arc<Pixmap>>| r.as_ref().map_or(0, |p| p.data().len());
        let mut total: usize = self.held.iter().map(|(_, r)| bytes(r)).sum();
        while total > HELD_BYTES && self.held.len() > 1 {
            let Some((_, dropped)) = self.held.pop_front() else {
                break;
            };
            total -= bytes(&dropped);
        }
    }
}
