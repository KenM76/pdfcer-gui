//! # `dialogs::print::preview::detail` — the preview, sharp at any zoom, up to the print's own resolution
//!
//! The whole-page preview raster is bounded ([`super::TARGET_DPI`],
//! [`super::MAX_SIDE_PX`]), so zooming in on a large sheet magnifies a coarse
//! bitmap. This module renders the **visible part** of the page at the
//! resolution the screen needs — never more than the job will print at — and
//! draws it over the base raster. Zoomed in far enough, the preview shows the
//! print's own pixels.
//!
//! # Contract
//!
//! - **Ceiling:** `job dpi / 72 × placement scale` pixels per page point —
//!   exactly the density the spooler rasterises this page at. The preview can
//!   therefore show what the paper will get and never implies detail it will
//!   not.
//! - **Off the UI thread**, one render at a time, cancelled when superseded:
//!   a region render still interprets the whole content stream
//!   (`pdfcer_render::render_page_region`), which is seconds on a dense
//!   sheet.
//! - **Settled views only:** a render starts once the view has held still for
//!   [`SETTLE_SECS`], so a wheel flick does not start one render per notch.
//! - **Same options as the spooler** (`commit::render_options`), same
//!   `session.view()`, so the detail cannot disagree with the base raster or
//!   the print.
//! - A failed render leaves the base raster showing and is not reported; the
//!   spool reports a real failure once, where it matters.

use std::sync::mpsc;

use egui::{Color32, Painter, Rect, TextureHandle, TextureOptions};

use super::PreviewKey;

/// How long the view must hold still before a detail render starts, seconds.
const SETTLE_SECS: f64 = 0.15;

/// Detail is only worth rendering when it is at least this much finer than
/// the base raster; below it the two look the same and the render is waste.
const WORTH_IT: f32 = 1.15;

/// Scale steps per doubling. The wanted scale is rounded onto this grid so a
/// small zoom change reuses the raster already made.
const STEPS_PER_OCTAVE: f32 = 4.0;

/// What a detail raster was, or is being, rendered for.
#[derive(Debug, Clone, PartialEq)]
struct Want {
    key: PreviewKey,
    /// PDF user space, y-up — `render_page_region`'s rectangle.
    region: pdfcer_core::page_tree::Rect,
    /// Pixels per page point.
    scale: f32,
}

/// A finished render, still as bytes.
struct Done {
    want: Want,
    size: [usize; 2],
    rgba: Vec<u8>,
}

/// The preview's detail layer, held on the print dialog across frames.
#[derive(Default)]
pub(crate) struct Detail {
    /// The view last asked for, and when it was first asked for.
    asked: Option<(Want, f64)>,
    /// The render in flight.
    pending: Option<(
        Want,
        pdfcer_render::cancel::RenderCancel,
        mpsc::Receiver<Option<Done>>,
    )>,
    /// The raster on screen.
    shown: Option<(Want, TextureHandle)>,
}

impl std::fmt::Debug for Detail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Detail")
            .field("asked", &self.asked.as_ref().map(|(w, _)| w))
            .field("pending", &self.pending.as_ref().map(|(w, ..)| w))
            .field("shown", &self.shown.as_ref().map(|(w, _)| w))
            .finish()
    }
}

impl Drop for Detail {
    fn drop(&mut self) {
        if let Some((_, cancel, _)) = &self.pending {
            cancel.cancel();
        }
    }
}

/// What one frame hands the detail layer.
pub(super) struct Frame<'a> {
    pub(super) doc: &'a crate::app::state::OpenDoc,
    pub(super) key: PreviewKey,
    /// Document page index.
    pub(super) page: usize,
    /// The page's extent in canvas points (`/Rotate` resolved).
    pub(super) size: (f64, f64),
    /// Where the whole page is drawn on screen.
    pub(super) placed: Rect,
    /// The preview canvas; only what is inside it is visible.
    pub(super) canvas: Rect,
    /// The base raster's pixels per page point.
    pub(super) base_scale: f32,
    /// The job's print density for this page, pixels per page point.
    pub(super) print_scale: f32,
    pub(super) scope: pdfcer_render::AnnotationScope,
}

/// The pixels per page point a view needs, capped at the print density,
/// rounded onto the [`STEPS_PER_OCTAVE`] grid. `None` when the base raster
/// is already as fine as that.
fn wanted_scale(screen_px_per_pt: f32, base_scale: f32, print_scale: f32) -> Option<f32> {
    if !(screen_px_per_pt.is_finite() && base_scale > 0.0 && print_scale > 0.0) {
        return None;
    }
    let need = screen_px_per_pt.min(print_scale);
    let stepped = (need.log2() * STEPS_PER_OCTAVE).ceil() / STEPS_PER_OCTAVE;
    let scale = stepped.exp2().min(print_scale);
    (scale > base_scale * WORTH_IT).then_some(scale)
}

impl Detail {
    /// Collect a finished render, start the one this view needs if it has
    /// settled, and draw whatever raster matches the page on screen.
    pub(super) fn frame(&mut self, painter: &Painter, frame: &Frame<'_>) {
        let ctx = painter.ctx().clone();
        self.collect(&ctx);

        let want = self.want(&ctx, frame);
        let now = ctx.input(|i| i.time);
        match (&want, &self.asked) {
            (Some(w), Some((asked, _))) if w == asked => {}
            (Some(w), _) => self.asked = Some((w.clone(), now)),
            (None, _) => self.asked = None,
        }
        if let Some((w, since)) = &self.asked {
            let covered = self.shown.as_ref().is_some_and(|(s, _)| s == w)
                || self.pending.as_ref().is_some_and(|(p, ..)| p == w);
            if !covered {
                if now - since >= SETTLE_SECS {
                    self.start(&ctx, frame, w.clone());
                } else {
                    ctx.request_repaint_after(std::time::Duration::from_secs_f64(SETTLE_SECS));
                }
            }
        }

        if let Some((shown, texture)) = &self.shown
            && shown.key == frame.key
            && want.is_some()
        {
            let pdf = &frame.doc.pages[frame.page];
            let at = crate::render::region::region_on_screen(
                shown.region,
                (frame.size.0 as f32, frame.size.1 as f32),
                crate::render::region::PageFrame::of(pdf),
                frame.placed,
            );
            painter
                .with_clip_rect(frame.placed.intersect(frame.canvas))
                .image(
                    texture.id(),
                    at,
                    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE, // NOT A THEME COLOUR: pass-through tint over a rendered page bitmap
                );
        }
        crate::diag::trace_changed("print-preview-detail", || {
            format!(
                "print-preview-detail page={} base_scale={:.3} print_scale={:.3} want_scale={:.3} shown_scale={:.3} pending={}",
                frame.page,
                frame.base_scale,
                frame.print_scale,
                want.as_ref().map_or(0.0, |w| w.scale),
                self.shown
                    .as_ref()
                    .filter(|(w, _)| w.key == frame.key)
                    .map_or(0.0, |(w, _)| w.scale),
                u8::from(self.pending.is_some()),
            )
        });
    }

    /// The region and scale this view calls for, or `None` when the base
    /// raster suffices.
    fn want(&self, ctx: &egui::Context, frame: &Frame<'_>) -> Option<Want> {
        let (w, h) = frame.size;
        if w <= 0.0 || h <= 0.0 || frame.placed.width() <= 0.0 {
            return None;
        }
        let visible = frame.placed.intersect(frame.canvas);
        if !visible.is_positive() {
            return None;
        }
        let screen_pt_per_page_pt = frame.placed.width() / w as f32;
        let scale = wanted_scale(
            screen_pt_per_page_pt * ctx.pixels_per_point(),
            frame.base_scale,
            frame.print_scale,
        )?;
        let to_page = |x: f32, y: f32| {
            (
                f64::from((x - frame.placed.min.x) / screen_pt_per_page_pt),
                f64::from((y - frame.placed.min.y) / screen_pt_per_page_pt),
            )
        };
        let (x0, y0) = to_page(visible.min.x, visible.min.y);
        let (x1, y1) = to_page(visible.max.x, visible.max.y);
        let pdf = frame.doc.pages.get(frame.page)?;
        let region = crate::render::region::page_region(
            (x0.max(0.0), y0.max(0.0), x1.min(w), y1.min(h)),
            crate::render::region::PageFrame::of(pdf),
        );
        Some(Want {
            key: frame.key.clone(),
            region,
            scale,
        })
    }

    /// Start rendering `want` on a thread, cancelling any render in flight.
    fn start(&mut self, ctx: &egui::Context, frame: &Frame<'_>, want: Want) {
        if let Some((_, cancel, _)) = self.pending.take() {
            cancel.cancel();
        }
        let Some(page) = frame.doc.pages.get(frame.page).cloned() else {
            return;
        };
        let cancel = pdfcer_render::cancel::RenderCancel::new();
        let mut options = super::super::commit::render_options(frame.scope, &frame.doc.settings);
        frame.key.apply_lines(&mut options, f64::from(want.scale));
        options.cancel = Some(cancel.clone());
        let session = std::sync::Arc::clone(&frame.doc.session);
        let (tx, rx) = mpsc::channel();
        let job = want.clone();
        let repaint = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("print-preview-detail".into())
            .spawn(move || {
                let view = session.view();
                let done = pdfcer_render::render_page_region(
                    &view, &page, job.scale, job.region, &options,
                )
                .ok()
                .map(|r| Done {
                    size: [r.pixmap.width() as usize, r.pixmap.height() as usize],
                    rgba: r.pixmap.data().to_vec(),
                    want: job,
                });
                let _ = tx.send(done);
                repaint.request_repaint();
            });
        if spawned.is_ok() {
            self.pending = Some((want, cancel, rx));
        }
    }

    /// Take a finished render, if one has arrived, and upload it.
    fn collect(&mut self, ctx: &egui::Context) {
        let Some((_, _, rx)) = &self.pending else {
            return;
        };
        let done = match rx.try_recv() {
            Ok(done) => done,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => None,
        };
        self.pending = None;
        let Some(done) = done else {
            return;
        };
        crate::render::pressure::record_other(
            ctx,
            crate::render::pressure::Surface::Preview,
            done.size[0] as u32,
            done.size[1] as u32,
        );
        let image = egui::ColorImage::from_rgba_premultiplied(done.size, &done.rgba);
        let texture = ctx.load_texture(DETAIL_TEXTURE_ID, image, TextureOptions::LINEAR);
        self.shown = Some((done.want, texture));
    }
}

/// The egui texture name the detail raster is uploaded under.
const DETAIL_TEXTURE_ID: &str = "print-preview-detail";

#[cfg(test)]
mod tests {
    use super::wanted_scale;

    /// At fit the base raster is enough, and no render is asked for.
    #[test]
    fn a_view_the_base_raster_covers_asks_for_nothing() {
        assert_eq!(wanted_scale(0.5, 0.9, 8.33), None);
    }

    /// Zoomed in, the wanted scale rises with the zoom and stops at the
    /// print's own density — the preview never claims detail the paper
    /// will not get.
    #[test]
    fn detail_follows_the_zoom_and_stops_at_the_print_density() {
        let print = 600.0 / 72.0;
        let mid = wanted_scale(3.0, 0.9, print).expect("3 px/pt is finer than 0.9");
        assert!((3.0..3.0 * 1.19).contains(&mid), "{mid}");
        assert_eq!(wanted_scale(40.0, 0.9, print), Some(print));
    }

    /// A small zoom change lands on the same grid step, so it reuses the
    /// raster already made.
    #[test]
    fn a_small_zoom_change_keeps_the_same_scale() {
        assert_eq!(
            wanted_scale(3.0, 0.9, 100.0),
            wanted_scale(3.05, 0.9, 100.0)
        );
    }
}
