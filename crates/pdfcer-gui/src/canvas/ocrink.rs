//! # `canvas::ocrink` — the OCR layer, rendered by the engine
//!
//! The page's invisible text is drawn by `pdfcer_render` itself: the visible
//! part of the page, rendered with `RenderOptions::with_invisible_text` in
//! only-mode, which paints the invisible runs in the layer's colour on a
//! transparent backdrop and nothing else. The layer, a text edit's preview and
//! the committed text are therefore one renderer's drawing.
//!
//! Contract:
//! - [`paint`] asks for the visible region at the screen's density and draws
//!   whatever raster it holds at that raster's own page region, at the layer's
//!   opacity. Nothing waits: the render runs on a thread, is cancelled when
//!   superseded, and starts once the view has held still for [`SETTLE_SECS`].
//! - The previous raster stays on screen until its replacement arrives.
//! - The run an open edit is rewriting is cut out of the raster
//!   (`textedit::shaped` draws it), and stays cut out after the edit commits
//!   until a raster of the committed document arrives.
//! - Layer visibility and the canvas's own render options apply, so a hidden
//!   optional-content group hides its text here as on the page.
//!
//! Design: `docs/modules/pdfcer-gui/canvas/ocrink.md`.

use std::sync::{Arc, Mutex};

use egui::{Color32, Painter, Pos2, Rect};
use pdfcer_core::page_tree::Rect as PdfRect;
use pdfcer_render::InvisibleTextPaint;
use pdfcer_render::cancel::RenderCancel;

use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;
use crate::render::region::{PageFrame, page_region, region_on_screen};

const KEY: &str = "ocr-ink"; // ui-text-exempt: a memory key, never displayed.
const TEXTURE: &str = "ocr-ink-texture"; // ui-text-exempt: a texture name, never displayed.

/// How long the view must hold still before a render starts, seconds.
const SETTLE_SECS: f64 = 0.15;

/// Scale steps per doubling; the wanted density is rounded up onto this grid
/// so a small zoom change reuses the raster already made.
const STEPS_PER_OCTAVE: f32 = 4.0;

/// What a raster is of.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Want {
    /// The session's address: a document replaced in its tab is a new one.
    session: usize,
    epoch: u64,
    page: usize,
    rgb: [u8; 3],
    layers: u64,
    /// PDF user space, y-up.
    region: PdfRect,
    /// Pixels per page point.
    scale: f32,
}

impl Want {
    /// The same document state, whatever the region and density.
    fn same_content(&self, other: &Self) -> bool {
        (self.session, self.epoch, self.page, self.rgb, self.layers)
            == (
                other.session,
                other.epoch,
                other.page,
                other.rgb,
                other.layers,
            )
    }
}

/// A finished render, still as bytes.
struct Done {
    want: Want,
    size: [usize; 2],
    rgba: Vec<u8>,
    ms: u128,
}

type Slot = Arc<Mutex<Option<Result<Done, String>>>>;

/// The layer's raster, held across frames.
#[derive(Default)]
struct State {
    /// The view last asked for, and when it was first asked for.
    asked: Option<(Want, f64)>,
    /// The render in flight.
    pending: Option<(Want, RenderCancel, Slot)>,
    /// The raster on screen.
    shown: Option<(Want, egui::TextureHandle)>,
    /// The last edited run's canvas rectangle and the epoch it was edited at.
    hole: Option<(u64, Rect)>,
}

type Shared = Arc<Mutex<State>>;

/// **Draw the OCR layer** of `view`'s page in `rgb` at opacity `alpha`, the
/// run whose canvas rectangle is `held` cut out.
pub(super) fn paint(
    painter: &Painter,
    doc: &OpenDoc,
    view: &PageView,
    held: Option<Rect>,
    rgb: [u8; 3],
    alpha: u8,
) {
    let Some(page) = doc.pages.get(view.page) else {
        return;
    };
    let ctx = painter.ctx();
    let shared = ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Shared>(egui::Id::new(KEY))
            .clone()
    });
    let Ok(mut state) = shared.lock() else {
        return;
    };
    collect(ctx, &mut state);
    let now = ctx.input(|i| i.time);
    if let Some(want) = wanted(painter, doc, view, rgb) {
        schedule(ctx, doc, &mut state, want, now);
    }
    if let Some(rect) = held {
        state.hole = Some((doc.edit_epoch, rect));
    }
    let Some((shown, texture)) = &state.shown else {
        return;
    };
    let current = (Arc::as_ptr(&doc.session) as usize, view.page);
    if (shown.session, shown.page) != current {
        return;
    }
    let at = region_on_screen(
        shown.region,
        crate::viewer::page_extent_pts(page),
        PageFrame::of(page),
        view.map.image_rect(),
    );
    // The edited run stays cut out until a raster of what it committed to
    // arrives; once one has, the raster draws it.
    let hole = held.or_else(|| {
        state
            .hole
            .filter(|(epoch, _)| *epoch == shown.epoch && doc.edit_epoch != shown.epoch)
            .map(|(_, r)| r)
    });
    let hole = hole.map(|r| view.map.rect_to_screen(r));
    // NOT A THEME COLOUR: the identity tint, scaled to the layer's opacity.
    let tint = Color32::WHITE.gamma_multiply(f32::from(alpha) / 255.0);
    draw_holed(painter, texture.id(), at, hole, tint);
}

/// The region and density the view needs now. `None` when nothing of the
/// page is visible.
fn wanted(painter: &Painter, doc: &OpenDoc, view: &PageView, rgb: [u8; 3]) -> Option<Want> {
    let page = doc.pages.get(view.page)?;
    let visible = view.map.image_rect().intersect(painter.clip_rect());
    if !visible.is_positive() {
        return None;
    }
    let (w, h) = crate::viewer::page_extent_pts(page);
    let c = view.map.rect_to_page(visible);
    let canvas = (
        f64::from(c.min.x.max(0.0)),
        f64::from(c.min.y.max(0.0)),
        f64::from(c.max.x.min(w)),
        f64::from(c.max.y.min(h)),
    );
    if canvas.2 <= canvas.0 || canvas.3 <= canvas.1 {
        return None;
    }
    let zoom = view.map.page_vec_to_screen(egui::vec2(1.0, 0.0)).x;
    let density = zoom * painter.ctx().pixels_per_point();
    if !density.is_finite() || density <= 0.0 {
        return None;
    }
    let scale = (density.log2() * STEPS_PER_OCTAVE).ceil() / STEPS_PER_OCTAVE;
    Some(Want {
        session: Arc::as_ptr(&doc.session) as usize,
        epoch: doc.edit_epoch,
        page: view.page,
        rgb,
        layers: doc.layers.generation,
        region: page_region(canvas, PageFrame::of(page)),
        scale: scale.exp2(),
    })
}

/// Start a render of `want` once the view has settled on it, unless the
/// raster shown or the one in flight is already of it.
fn schedule(ctx: &egui::Context, doc: &OpenDoc, state: &mut State, want: Want, now: f64) {
    let held = |w: &Want| *w == want;
    if state.shown.as_ref().is_some_and(|(w, _)| held(w))
        || state.pending.as_ref().is_some_and(|(w, _, _)| held(w))
    {
        state.asked = None;
        return;
    }
    let since = match state.asked {
        Some((asked, at)) if asked == want => at,
        _ => {
            state.asked = Some((want, now));
            now
        }
    };
    // Nothing of this document state on screen: start at once rather than
    // leave the layer empty for the settle time.
    let empty = !state
        .shown
        .as_ref()
        .is_some_and(|(w, _)| w.same_content(&want));
    if !empty && now - since < SETTLE_SECS {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(SETTLE_SECS));
        return;
    }
    start(ctx, doc, state, want);
}

/// Render `want` on a thread, cancelling any render in flight.
fn start(ctx: &egui::Context, doc: &OpenDoc, state: &mut State, want: Want) {
    if let Some((_, cancel, _)) = state.pending.take() {
        cancel.cancel();
    }
    let Some(page) = doc.pages.get(want.page).cloned() else {
        return;
    };
    let cancel = RenderCancel::new();
    let mut options = doc
        .settings
        .render_options()
        .with_invisible_text(Some(InvisibleTextPaint::new(want.rgb).with_only(true)));
    options.layers = doc.layer_visibility();
    options.cancel = Some(cancel.clone());
    let session = Arc::clone(&doc.session);
    let slot: Slot = Arc::default();
    let out = Arc::clone(&slot);
    let repaint = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("ocr-layer".into())
        .spawn(move || {
            let began = std::time::Instant::now();
            let view = session.view();
            let done =
                pdfcer_render::render_page_region(&view, &page, want.scale, want.region, &options)
                    .map(|r| Done {
                        size: [r.pixmap.width() as usize, r.pixmap.height() as usize],
                        rgba: r.pixmap.data().to_vec(),
                        want,
                        ms: began.elapsed().as_millis(),
                    })
                    .map_err(|e| e.to_string());
            if let Ok(mut slot) = out.lock() {
                *slot = Some(done);
            }
            repaint.request_repaint();
        });
    if spawned.is_ok() {
        state.asked = None;
        state.pending = Some((want, cancel, slot));
    }
}

/// Take a finished render, if one has arrived, and upload it.
#[allow(clippy::cast_possible_truncation)]
fn collect(ctx: &egui::Context, state: &mut State) {
    let Some((want, _, slot)) = &state.pending else {
        return;
    };
    let want = *want;
    let Some(done) = slot.lock().ok().and_then(|mut s| s.take()) else {
        return;
    };
    state.pending = None;
    let done = match done {
        Ok(done) => done,
        Err(why) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("ocr-ink-refused page={} why={why}", want.page)
            });
            return;
        }
    };
    let [w, h] = done.size;
    crate::render::pressure::record_other(
        ctx,
        crate::render::pressure::Surface::OcrLayer,
        w as u32,
        h as u32,
    );
    let image = egui::ColorImage::from_rgba_premultiplied(done.size, &done.rgba);
    let texture = ctx.load_texture(TEXTURE, image, egui::TextureOptions::LINEAR);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "ocr-ink-built page={} epoch={} px={w}x{h} ms={}",
            done.want.page, done.want.epoch, done.ms
        )
    });
    state.shown = Some((done.want, texture));
}

/// Draw `texture` over `at`, all but the part under `hole`.
fn draw_holed(
    painter: &Painter,
    texture: egui::TextureId,
    at: Rect,
    hole: Option<Rect>,
    tint: Color32,
) {
    let full = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    let Some(hole) = hole.map(|h| h.intersect(at)).filter(|h| h.is_positive()) else {
        painter.image(texture, at, full, tint);
        return;
    };
    let pieces = [
        Rect::from_x_y_ranges(at.x_range(), at.min.y..=hole.min.y),
        Rect::from_x_y_ranges(at.x_range(), hole.max.y..=at.max.y),
        Rect::from_x_y_ranges(at.min.x..=hole.min.x, hole.y_range()),
        Rect::from_x_y_ranges(hole.max.x..=at.max.x, hole.y_range()),
    ];
    let uv = |p: Pos2| {
        Pos2::new(
            (p.x - at.min.x) / at.width(),
            (p.y - at.min.y) / at.height(),
        )
    };
    for piece in pieces.into_iter().filter(|r| r.is_positive()) {
        painter.image(
            texture,
            piece,
            Rect::from_min_max(uv(piece.min), uv(piece.max)),
            tint,
        );
    }
}
