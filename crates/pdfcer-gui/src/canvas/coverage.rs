//! # `canvas::coverage` — Output Preview's ink coverage warnings on the page
//!
//! While the separation preview is on and the panel has a warning chosen, the
//! visible part of the page is checked with
//! [`pdfcer_render::coverage_warning_masks`] and the flagged pixels are painted
//! over it in the warning's colour, as Acrobat's Output Preview does. It is a
//! view the operator switches on, of the ink the preview composites, and it
//! goes with the preview.
//!
//! Contract:
//! - The rules chosen ([`Warnings`]) are the operator's for the session, held
//!   here; [`set`] is the panel's only way in.
//! - One thread renders each chosen rule's mask for the visible region at
//!   screen density once the view has held still for [`SETTLE_SECS`], and is
//!   cancelled when superseded. The overlay shown stays until its replacement
//!   arrives, while it is of the same document state and rules.
//! - Rich black is painted over total area coverage where both flag.
//! - [`status`] is what the shown overlay flagged, for the panel. A check the
//!   engine refuses is not retried until the document, page or rules change.
//!
//! Traces `coverage-mask page= tac=<n|-> rich=<n|-> tac_box= rich_box= px=WxH
//! ms=` per finished check (flagged pixels; boxes `x0,y0,x1,y1` in canvas
//! points, y down from the upright page's top-left, `-` when none), and
//! `coverage-mask-refused`.

use std::sync::{Arc, Mutex};

use egui::{Color32, Painter, Pos2, Rect};
use pdfcer_core::page_tree::Rect as PdfRect;
use pdfcer_render::cancel::RenderCancel;
use pdfcer_render::{CoverageMask, CoverageWarning};

use crate::app::settings::SettingsExt;
use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;
use crate::render::region::{PageFrame, region_on_screen};

const KEY: &str = "coverage-mask"; // ui-text-exempt: a memory key, never displayed.
const TEXTURE: &str = "coverage-mask-texture"; // ui-text-exempt: a texture name, never displayed.

/// How long the view must hold still before a check starts, seconds.
const SETTLE_SECS: f64 = 0.15;

/// Total area coverage's overlay colour. NOT A THEME COLOUR: a warning
/// painted over ink must read against any ink, so it is fixed.
pub const TAC_COLOUR: Color32 = Color32::from_rgb(0, 220, 0);
/// Rich black's overlay colour; fixed for the same reason.
pub const RICH_COLOUR: Color32 = Color32::from_rgb(255, 140, 0);

/// The warnings the panel has switched on: each rule's threshold in percent,
/// or `None` when it is off.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Warnings {
    /// Total area coverage strictly above this.
    pub total: Option<u16>,
    /// Black at or above this with any cyan, magenta or yellow.
    pub rich_black: Option<u16>,
}

impl Warnings {
    /// Whether any rule is on.
    #[must_use]
    pub fn any(self) -> bool {
        self.total.is_some() || self.rich_black.is_some()
    }
}

/// Which rules the shown overlay flagged anything for, in the part of the page
/// it checked; `None` for a rule that is off.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Found {
    pub total: Option<bool>,
    pub rich_black: Option<bool>,
}

/// What the panel is told about the part of the page on screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    /// No overlay of the current state yet.
    Checking,
    /// The engine refused the check.
    Refused,
    /// The overlay on screen, of the current state.
    Done(Found),
}

/// What a mask is of.
#[derive(Clone, PartialEq, Debug)]
struct Want {
    /// The session's address: a document replaced in its tab is a new one.
    session: usize,
    epoch: u64,
    page: usize,
    layers: u64,
    annotations: bool,
    /// The preview's hidden plates, which count as no ink.
    hidden: Vec<String>,
    warnings: Warnings,
    /// PDF user space, y-up.
    region: PdfRect,
    /// Pixels per page point.
    scale: f32,
}

impl Want {
    /// The same document state and rules, whatever the region and density.
    fn same_content(&self, other: &Self) -> bool {
        (
            self.session,
            self.epoch,
            self.page,
            self.layers,
            self.annotations,
        ) == (
            other.session,
            other.epoch,
            other.page,
            other.layers,
            other.annotations,
        ) && self.hidden == other.hidden
            && self.warnings == other.warnings
    }
}

/// A finished check, still as bytes.
struct Done {
    want: Want,
    size: [usize; 2],
    rgba: Vec<u8>,
    found: Found,
}

type Slot = Arc<Mutex<Option<Result<Done, String>>>>;

#[derive(Default)]
struct State {
    warnings: Warnings,
    asked: Option<(Want, f64)>,
    pending: Option<(Want, RenderCancel, Slot)>,
    shown: Option<(Want, egui::TextureHandle, Found)>,
    /// The last check the engine refused.
    refused: Option<Want>,
    /// How much of the overlay lets the page through, percent; 0 is opaque.
    /// A paint setting, so not part of a [`Want`].
    see_through: u8,
}

type Shared = Arc<Mutex<State>>;

fn shared(ctx: &egui::Context) -> Shared {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Shared>(egui::Id::new(KEY))
            .clone()
    })
}

/// The rules switched on.
#[must_use]
pub fn warnings(ctx: &egui::Context) -> Warnings {
    shared(ctx).lock().map(|s| s.warnings).unwrap_or_default()
}

/// Switch rules on or off.
pub fn set(ctx: &egui::Context, warnings: Warnings) {
    if let Ok(mut state) = shared(ctx).lock() {
        state.warnings = warnings;
    }
    ctx.request_repaint();
}

/// Set the overlay's opacity, percent (clamped to 0..=100).
pub fn set_opacity(ctx: &egui::Context, percent: u8) {
    if let Ok(mut state) = shared(ctx).lock() {
        state.see_through = 100 - percent.min(100);
    }
    ctx.request_repaint();
}

/// What the overlay of `doc`'s current page and state says, under the rules
/// switched on.
#[must_use]
pub fn status(ctx: &egui::Context, doc: &OpenDoc) -> Status {
    let state = shared(ctx);
    let Ok(state) = state.lock() else {
        return Status::Checking;
    };
    let current = (
        Arc::as_ptr(&doc.session) as usize,
        doc.edit_epoch,
        doc.view.page_index,
    );
    let of = |w: &Want| (w.session, w.epoch, w.page) == current && w.warnings == state.warnings;
    if state.refused.as_ref().is_some_and(of) {
        return Status::Refused;
    }
    match &state.shown {
        Some((want, _, found)) if of(want) => Status::Done(*found),
        _ => Status::Checking,
    }
}

/// Paint the warnings over the current page, when it is on screen.
pub(super) fn draw(painter: &Painter, doc: &OpenDoc, pages: &[PageView]) {
    if let Some(view) = pages.iter().find(|v| v.page == doc.view.page_index) {
        paint(painter, doc, view);
    }
}

/// **Paint the warnings** over `view`'s page, or forget them when the
/// preview is off or no rule is on.
fn paint(painter: &Painter, doc: &OpenDoc, view: &PageView) {
    let ctx = painter.ctx();
    let shared = shared(ctx);
    let Ok(mut state) = shared.lock() else {
        return;
    };
    let Some(preview) = doc.separations.as_ref().filter(|_| state.warnings.any()) else {
        clear(&mut state);
        return;
    };
    collect(ctx, &mut state);
    let now = ctx.input(|i| i.time);
    let Some(page) = doc.pages.get(view.page) else {
        return;
    };
    if let Some((region, scale)) = super::ocrink::visible_raster(painter, doc, view) {
        let want = Want {
            session: Arc::as_ptr(&doc.session) as usize,
            epoch: doc.edit_epoch,
            page: view.page,
            layers: doc.layers.generation,
            annotations: doc.annotations_visible(),
            hidden: preview.hidden.clone(),
            warnings: state.warnings,
            region,
            scale,
        };
        schedule(ctx, doc, &mut state, want, now);
    }
    let Some((shown, texture, _)) = &state.shown else {
        return;
    };
    let current = Arc::as_ptr(&doc.session) as usize;
    if (shown.session, shown.page) != (current, view.page) || shown.warnings != state.warnings {
        return;
    }
    let at = region_on_screen(
        shown.region,
        crate::viewer::page_extent_pts(page),
        PageFrame::of(page),
        view.map.image_rect(),
    );
    crate::diag::ui_rect("coverage-overlay", at); // ui-text-exempt: trace region name
    let full = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    let opacity = f32::from(100 - state.see_through) / 100.0;
    painter.image(
        texture.id(),
        at,
        full,
        Color32::WHITE.gamma_multiply(opacity), // NOT A THEME COLOUR: an image tint; white passes the texture through at this alpha
    );
}

fn clear(state: &mut State) {
    if let Some((_, cancel, _)) = state.pending.take() {
        cancel.cancel();
    }
    state.asked = None;
    state.shown = None;
    state.refused = None;
}

/// Start a check of `want` once the view has settled on it, unless the
/// overlay shown or the check in flight is already of it.
fn schedule(ctx: &egui::Context, doc: &OpenDoc, state: &mut State, want: Want, now: f64) {
    if state.shown.as_ref().is_some_and(|(w, _, _)| *w == want)
        || state.pending.as_ref().is_some_and(|(w, _, _)| *w == want)
        || state
            .refused
            .as_ref()
            .is_some_and(|w| w.same_content(&want))
    {
        state.asked = None;
        return;
    }
    let since = match &state.asked {
        Some((asked, at)) if *asked == want => *at,
        _ => {
            state.asked = Some((want.clone(), now));
            now
        }
    };
    let empty = !state
        .shown
        .as_ref()
        .is_some_and(|(w, _, _)| w.same_content(&want));
    if !empty && now - since < SETTLE_SECS {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(SETTLE_SECS));
        return;
    }
    start(ctx, doc, state, want);
}

/// Check `want` on a thread, cancelling any check in flight.
fn start(ctx: &egui::Context, doc: &OpenDoc, state: &mut State, want: Want) {
    if let Some((_, cancel, _)) = state.pending.take() {
        cancel.cancel();
    }
    let Some(page) = doc.pages.get(want.page).cloned() else {
        return;
    };
    let Some(preview) = doc.separations.clone() else {
        return;
    };
    let cancel = RenderCancel::new();
    // The soft proof never reaches the colorant buffer the mask reads.
    let mut options = doc
        .settings
        .render_options()
        .with_separation_preview(Some(preview.with_soft_proof(None)));
    options.layers = doc.layer_visibility();
    options.annotations = want.annotations;
    options.cancel = Some(cancel.clone());
    let session = Arc::clone(&doc.session);
    let slot: Slot = Arc::default();
    let out = Arc::clone(&slot);
    let repaint = ctx.clone();
    let job = want.clone();
    let spawned = std::thread::Builder::new()
        .name("coverage-mask".into())
        .spawn(move || {
            let done = check(&session, &page, &options, job);
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

/// Every chosen rule's mask of `want`, painted into one overlay.
fn check(
    session: &pdfcer_core::edit::EditSession,
    page: &pdfcer_core::page_tree::Page,
    options: &pdfcer_render::RenderOptions,
    want: Want,
) -> Result<Done, String> {
    let began = std::time::Instant::now();
    let view = session.view();
    // One render answers both rules; the masks come back in the order asked.
    let rules: Vec<CoverageWarning> = [
        want.warnings
            .total
            .map(|limit_percent| CoverageWarning::TotalAreaCoverage { limit_percent }),
        want.warnings
            .rich_black
            .map(|black_start_percent| CoverageWarning::RichBlack {
                black_start_percent,
            }),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut masks = pdfcer_render::coverage_warning_masks(
        &view,
        page,
        want.scale,
        Some(want.region),
        options,
        &rules,
    )
    .map_err(|e| e.to_string())?
    .into_iter();
    let total = want.warnings.total.and_then(|_| masks.next());
    let rich = want.warnings.rich_black.and_then(|_| masks.next());
    let Some(first) = total.as_ref().or(rich.as_ref()) else {
        return Err("no rule".to_owned()); // ui-text-exempt: diagnostic trace value
    };
    let (w, h) = (first.width as usize, first.height as usize);
    let mut rgba = vec![0u8; w * h * 4];
    for (mask, colour) in [(&total, TAC_COLOUR), (&rich, RICH_COLOUR)] {
        let Some(mask) = mask.as_ref().filter(|m| m.mask.len() == w * h) else {
            continue;
        };
        for (px, _) in rgba
            .chunks_exact_mut(4)
            .zip(&mask.mask)
            .filter(|(_, b)| **b == 1)
        {
            px.copy_from_slice(&colour.to_array());
        }
    }
    let frame = PageFrame::of(page);
    let canvas = frame.canvas_box_of(want.region);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "coverage-mask page={} tac={} rich={} tac_box={} rich_box={} px={w}x{h} ms={}",
            want.page,
            count(total.as_ref()),
            count(rich.as_ref()),
            flagged_box(total.as_ref(), canvas),
            flagged_box(rich.as_ref(), canvas),
            began.elapsed().as_millis()
        )
    });
    let found = Found {
        total: total.as_ref().map(|m| m.flagged > 0),
        rich_black: rich.as_ref().map(|m| m.flagged > 0),
    };
    Ok(Done {
        want,
        size: [w, h],
        rgba,
        found,
    })
}

fn count(mask: Option<&CoverageMask>) -> String {
    mask.map_or_else(|| "-".to_owned(), |m| m.flagged.to_string())
}

/// The flagged pixels' bounding box in canvas points, `canvas` being the
/// raster's own box.
#[allow(clippy::cast_precision_loss)]
fn flagged_box(mask: Option<&CoverageMask>, canvas: (f64, f64, f64, f64)) -> String {
    let Some(mask) = mask.filter(|m| m.flagged > 0 && m.width > 0 && m.height > 0) else {
        return "-".to_owned();
    };
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..mask.height {
        for x in 0..mask.width {
            if mask.is_flagged(x, y) {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
            }
        }
    }
    let sx = (canvas.2 - canvas.0) / f64::from(mask.width);
    let sy = (canvas.3 - canvas.1) / f64::from(mask.height);
    format!(
        "{:.1},{:.1},{:.1},{:.1}",
        canvas.0 + f64::from(x0) * sx,
        canvas.1 + f64::from(y0) * sy,
        canvas.0 + f64::from(x1) * sx,
        canvas.1 + f64::from(y1) * sy
    )
}

/// Take a finished check, if one has arrived, and upload it.
#[allow(clippy::cast_possible_truncation)]
fn collect(ctx: &egui::Context, state: &mut State) {
    let Some((want, _, slot)) = &state.pending else {
        return;
    };
    let Some(done) = slot.lock().ok().and_then(|mut s| s.take()) else {
        return;
    };
    let want = want.clone();
    state.pending = None;
    match done {
        Ok(done) => {
            let image = egui::ColorImage::from_rgba_premultiplied(done.size, &done.rgba);
            let [w, h] = done.size;
            crate::render::pressure::record_other(
                ctx,
                crate::render::pressure::Surface::CoverageWarning,
                w as u32,
                h as u32,
            );
            let texture = ctx.load_texture(TEXTURE, image, egui::TextureOptions::NEAREST);
            state.refused = None;
            state.shown = Some((done.want, texture, done.found));
        }
        Err(why) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("coverage-mask-refused why={why}")
            });
            state.shown = None;
            state.refused = Some(want);
        }
    }
}
