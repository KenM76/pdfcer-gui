//! # `canvas::inkpick` — Tools ▸ Diagnostics ▸ Ink picker
//!
//! A click with the tool armed reads the ink at that point: the four process
//! tints and any spot inks, as the page's colorant buffer held them before the
//! conversion to screen colour (`pdfcer_render::InkProbe`). The reading goes
//! to the tool strip, never onto the page.
//!
//! # Contract
//!
//! - **Off the UI thread**, one reading at a time; a new click cancels the
//!   one in flight. A region render still interprets the whole content
//!   stream, which is seconds on a dense sheet.
//! - **What prints, at the zoom on screen**: the options come through the
//!   settings funnel with the canvas's annotation and layer choices, and
//!   without the display-only toggles (line weights, skip tiny details), so
//!   the tints are the ones a press would be sent.
//! - The probe renders a 3 × 3 device-pixel region centred on the click and
//!   reads its centre pixel, so the answer does not depend on which way the
//!   engine rounds the region's edges or turns the page.
//! - A reading belongs to one document, page and edit; it is not shown once
//!   any of the three has moved on.

use std::sync::{Arc, Mutex, Weak};

use egui::{Context, Id, Pos2};
use pdfcer_core::edit::EditSession;
use pdfcer_render::cancel::RenderCancel;

use crate::app::state::OpenDoc;
use crate::canvas::tool::CanvasTool;

/// Device pixels across the probed region; its centre pixel is read.
const PROBE_PX: f32 = 3.0;

/// The probe's raster scale bounds, device pixels per page point.
const SCALE_RANGE: (f32, f32) = (0.25, 64.0);

/// What one click found.
#[derive(Debug, Clone)]
pub enum Found {
    /// The render is running.
    Reading,
    /// The engine's answer.
    Probe(pdfcer_render::InkProbe),
    /// The render failed; the engine's reason.
    Failed(String),
}

/// One click's reading, and what it is a reading of.
#[derive(Debug, Clone)]
pub struct Reading {
    /// The document it was taken on.
    session: Weak<EditSession>,
    /// Page index.
    pub page: usize,
    /// The edit it was taken after.
    epoch: u64,
    /// PDF user space, y-up.
    pub at: (f64, f64),
    /// The answer, or that it is coming.
    pub found: Found,
}

#[derive(Default)]
struct State {
    reading: Option<Reading>,
    cancel: Option<RenderCancel>,
    /// What the tool strip last showed, traced on change.
    shown: &'static str,
}

type Shared = Arc<Mutex<State>>;

fn shared(ctx: &Context) -> Shared {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Shared>(Id::new("canvas-ink-picker"))
            .clone()
    })
}

/// Arm the ink picker, or put it down when it is already armed.
pub fn toggle(ctx: &Context) -> CanvasTool {
    let next = if crate::canvas::tool::selected(ctx) == CanvasTool::InkPicker {
        CanvasTool::Select
    } else {
        CanvasTool::InkPicker
    };
    crate::canvas::tool::select(ctx, next);
    let armed = next == CanvasTool::InkPicker;
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| format!("ink-picker armed={armed}"));
    next
}

/// The reading to show for `doc`, if the last click was on this document,
/// this page and this edit.
#[must_use]
pub fn current(ctx: &Context, doc: &OpenDoc) -> Option<Reading> {
    let state = shared(ctx);
    let guard = state.lock().ok()?;
    let reading = guard.reading.as_ref()?;
    let same_doc = reading
        .session
        .upgrade()
        .is_some_and(|s| Arc::ptr_eq(&s, &doc.session));
    (same_doc && reading.page == doc.view.page_index && reading.epoch == doc.edit_epoch)
        .then(|| reading.clone())
}

/// A click at canvas `point` on page `page_index`, drawn at `zoom` screen
/// points per page point.
pub fn click(ctx: &Context, doc: &OpenDoc, page_index: usize, point: Pos2, zoom: f32) {
    let Some(page) = doc.pages.get(page_index) else {
        return;
    };
    let Some((at, _)) = crate::canvas::markup::band::endpoints(point, point, page) else {
        return;
    };
    let scale = (zoom * ctx.pixels_per_point()).clamp(SCALE_RANGE.0, SCALE_RANGE.1);
    let Some(request) = doc.render_request_for(page_index, scale) else {
        return;
    };
    let cancel = RenderCancel::new();
    let job = Job {
        options: options_for(&request, &cancel),
        region: region_around(at, scale),
        scale,
        session: Arc::clone(&request.session),
        page: request.page,
    };
    let reading = Reading {
        session: Arc::downgrade(&doc.session),
        page: page_index,
        epoch: doc.edit_epoch,
        at,
        found: Found::Reading,
    };
    let state = shared(ctx);
    if let Ok(mut guard) = state.lock() {
        if let Some(old) = guard.cancel.replace(cancel.clone()) {
            old.cancel();
        }
        guard.reading = Some(reading.clone());
    }
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| {
        format!(
            "ink-picker-click page={page_index} x={:.1} y={:.1} scale={scale:.3}",
            at.0, at.1
        )
    });
    let repaint = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("ink-picker".into())
        .spawn(move || {
            let found = job.run();
            // Under the lock: a newer click cancels this one under the same
            // lock, so a superseded reading can never overwrite it.
            let Ok(mut guard) = state.lock() else {
                return;
            };
            if cancel.is_cancelled() {
                return;
            }
            trace_found(page_index, at, &found);
            guard.cancel = None;
            guard.reading = Some(Reading { found, ..reading });
            drop(guard);
            repaint.request_repaint();
        });
    if spawned.is_err() {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| "ink-picker-refused reason=thread".to_owned());
    }
}

/// One probe render, owned so it can cross to the worker thread.
struct Job {
    session: Arc<EditSession>,
    page: pdfcer_core::page_tree::Page,
    scale: f32,
    region: pdfcer_core::page_tree::Rect,
    options: pdfcer_render::RenderOptions,
}

impl Job {
    fn run(self) -> Found {
        let view = self.session.view();
        match pdfcer_render::render_page_region(
            &view,
            &self.page,
            self.scale,
            self.region,
            &self.options,
        ) {
            Ok(r) => r.diagnostics.ink_probe.map_or_else(
                || Found::Failed(crate::text::inkpick::no_probe().to_owned()),
                Found::Probe,
            ),
            Err(e) => Found::Failed(e.to_string()),
        }
    }
}

/// [`PROBE_PX`] device pixels square, centred on `at` (page space, y-up).
fn region_around(at: (f64, f64), scale: f32) -> pdfcer_core::page_tree::Rect {
    let half = f64::from(PROBE_PX / scale / 2.0);
    pdfcer_core::page_tree::Rect {
        llx: at.0 - half,
        lly: at.1 - half,
        urx: at.0 + half,
        ury: at.1 + half,
    }
}

/// The funnel's options with the canvas's annotation and layer choices and
/// the probe on the region's centre pixel; no display-only toggles.
fn options_for(
    request: &pdfcer_gui_base::renderworker::RenderRequest,
    cancel: &RenderCancel,
) -> pdfcer_render::RenderOptions {
    use crate::app::settings::SettingsExt;
    let centre = (PROBE_PX / 2.0) as u32;
    let mut options = request
        .settings
        .render_options()
        .with_ink_probe(centre, centre);
    options.annotations = request.annotations;
    options.layers = request.layers.clone();
    options.cancel = Some(cancel.clone());
    options
}

/// The tool strip's sentence while the picker is armed: the last reading on
/// this document, page and edit, or the instruction.
#[must_use]
pub fn sentence(ctx: &Context, doc: &OpenDoc) -> (String, Option<String>) {
    use crate::text::inkpick as t;
    let help = t::help();
    let (kind, line) = match current(ctx, doc) {
        None => ("instruction", t::instruction().to_owned()),
        Some(reading) => {
            let (x, y) = reading.at;
            match &reading.found {
                Found::Reading => ("reading", t::reading(x, y)),
                Found::Failed(why) => ("failed", t::failed(why)),
                Found::Probe(p) => probe_line(x, y, p),
            }
        }
    };
    trace_shown(ctx, kind);
    (line, Some(help))
}

/// The probe in words, and which kind of sentence it is (for the trace).
fn probe_line(x: f64, y: f64, p: &pdfcer_render::InkProbe) -> (&'static str, String) {
    use crate::text::inkpick as t;
    use pdfcer_render::InkProbeSource as S;
    match p.source {
        S::OutOfRange => ("off_page", t::off_page(x, y)),
        S::CmykBuffer if p.alpha.is_some_and(|a| a < 0.001) => ("bare", t::bare_paper(x, y)),
        S::CmykBuffer => p.cmyk.map_or_else(
            || ("failed", t::failed(t::no_probe())),
            |cmyk| ("inks", t::inks(x, y, cmyk, &p.spots)),
        ),
        _ => ("screen", t::screen_only(x, y, p.srgb)),
    }
}

/// `ink-picker-shown kind=` when the strip's sentence changes kind.
fn trace_shown(ctx: &Context, kind: &'static str) {
    let state = shared(ctx);
    let Ok(mut guard) = state.lock() else {
        return;
    };
    if guard.shown != kind {
        guard.shown = kind;
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("ink-picker-shown kind={kind}"));
    }
}

/// Cancel a reading in flight once the tool is no longer armed.
pub fn settle(ctx: &Context) {
    if crate::canvas::tool::selected(ctx) == CanvasTool::InkPicker {
        return;
    }
    let state = shared(ctx);
    if let Ok(mut guard) = state.lock()
        && let Some(cancel) = guard.cancel.take()
    {
        cancel.cancel();
    }
}

/// `ink-probe page= x= y= source= c= m= y= k= alpha= spots= srgb=`.
fn trace_found(page: usize, at: (f64, f64), found: &Found) {
    use pdfcer_render::InkProbeSource as S;
    let fields = match found {
        Found::Reading => return,
        // ui-text-exempt: diagnostic trace, never displayed.
        Found::Failed(why) => format!("source=failed why={}", why.replace(' ', "_")),
        Found::Probe(p) => {
            let source = match p.source {
                S::CmykBuffer => "ink",
                S::ScreenSrgb => "screen",
                S::OutOfRange => "out_of_range",
                _ => "other",
            };
            let [c, m, y, k] = p.cmyk.unwrap_or([f32::NAN; 4]);
            let spots = p
                .spots
                .iter()
                .map(|(name, tint)| format!("{}:{tint:.3}", name.replace(' ', "_")))
                .collect::<Vec<_>>()
                .join(",");
            let alpha = p
                .alpha
                .map_or_else(|| "none".to_owned(), |a| format!("{a:.3}"));
            let srgb = p.srgb.map_or_else(
                || "none".to_owned(),
                |[r, g, b]| format!("{r:02x}{g:02x}{b:02x}"),
            );
            format!(
                "source={source} c={c:.3} m={m:.3} y={y:.3} k={k:.3} alpha={alpha} \
                 spots={} srgb={srgb}",
                if spots.is_empty() { "none" } else { &spots }
            )
        }
    };
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| format!("ink-probe page={page} x={:.1} y={:.1} {fields}", at.0, at.1));
}
