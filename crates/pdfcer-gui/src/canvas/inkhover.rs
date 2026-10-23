//! # `canvas::inkhover` — the inks under the pointer, for Output Preview
//!
//! While the separation preview is on, a pointer that rests on the page is
//! probed for its ink tints, and the Output Preview panel shows them beside
//! each plate, as Acrobat does.
//!
//! # Contract
//!
//! - **Rest, then read.** A probe starts once the pointer has stayed within
//!   [`STILL_PX`] screen pixels for [`REST_S`]; moving on cancels the one in
//!   flight. A region render interprets the whole content stream, so probing
//!   every frame of a sweep would queue seconds of work.
//! - **The same plates the panel lists.** The probe is composited under the
//!   document's [`pdfcer_render::SeparationPreview`], so an RGB page splits
//!   into CMYK exactly as the preview draws it. Tints are read before any
//!   plate is hidden, so hiding a plate does not change the numbers.
//! - A reading belongs to one document, page and edit, and is dropped when
//!   the pointer leaves the page or the preview is switched off.
//! - Nothing is drawn on the canvas; the reading lives in the panel.

use std::sync::{Arc, Mutex};

use egui::{Context, Id, Pos2};
use pdfcer_render::cancel::RenderCancel;

use crate::app::state::OpenDoc;
use crate::canvas::inkpick::{Found, Prepared};

/// Screen pixels the pointer may drift and still count as resting.
const STILL_PX: f32 = 1.5;
/// Seconds of rest before a probe starts.
const REST_S: f64 = 0.12;
/// The trace event for a finished probe.
const EVENT: &str = "ink-hover"; // ui-text-exempt: diagnostic trace event name

/// The reading under the pointer.
#[derive(Debug, Clone)]
pub struct Hover {
    doc: u64,
    /// Page index.
    pub page: usize,
    epoch: u64,
    /// PDF user space, y-up.
    pub at: (f64, f64),
    /// The answer, or that it is coming.
    pub found: Found,
}

#[derive(Default)]
struct State {
    /// Where the pointer came to rest (page space) and when (egui time).
    rest: Option<(Pos2, f64)>,
    /// The rest point last probed, so one rest is probed once.
    probed: Option<Pos2>,
    reading: Option<Hover>,
    cancel: Option<RenderCancel>,
}

type Shared = Arc<Mutex<State>>;

fn shared(ctx: &Context) -> Shared {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Shared>(Id::new("canvas-ink-hover"))
            .clone()
    })
}

/// The reading to show for `doc`, if it is of this document, page and edit.
#[must_use]
pub fn current(ctx: &Context, doc: &OpenDoc) -> Option<Hover> {
    let state = shared(ctx);
    let guard = state.lock().ok()?;
    let reading = guard.reading.as_ref()?;
    (doc.separations.is_some()
        && reading.doc == doc.serial
        && reading.page == doc.view.page_index
        && reading.epoch == doc.edit_epoch)
        .then(|| reading.clone())
}

/// One canvas frame: `pointer` is the hovered page-space point on page
/// `page_index`, or `None` when the pointer is not over the page; `zoom` is
/// screen points per page point.
pub fn frame(ctx: &Context, doc: &OpenDoc, page_index: usize, pointer: Option<Pos2>, zoom: f32) {
    let state = shared(ctx);
    let Ok(mut guard) = state.lock() else {
        return;
    };
    let Some(point) = pointer.filter(|_| doc.separations.is_some()) else {
        clear(&mut guard);
        return;
    };
    let now = ctx.input(|i| i.time);
    let still = guard
        .rest
        .is_some_and(|(p, _)| p.distance(point) * zoom <= STILL_PX);
    if !still {
        guard.rest = Some((point, now));
        guard.probed = None;
        if let Some(old) = guard.cancel.take() {
            old.cancel();
        }
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(REST_S));
        return;
    }
    let Some((rest_at, since)) = guard.rest else {
        return;
    };
    if guard.probed.is_some() {
        return;
    }
    let waited = now - since;
    if waited < REST_S {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(REST_S - waited));
        return;
    }
    guard.probed = Some(rest_at);
    drop(guard);
    start(ctx, doc, page_index, rest_at, zoom);
}

/// Forget the reading and cancel the probe in flight.
fn clear(guard: &mut State) {
    if let Some(old) = guard.cancel.take() {
        old.cancel();
    }
    if guard.reading.take().is_some() {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("{EVENT}-cleared"));
    }
    guard.rest = None;
    guard.probed = None;
}

/// Probe `point` off the UI thread.
fn start(ctx: &Context, doc: &OpenDoc, page_index: usize, point: Pos2, zoom: f32) {
    let Some(Prepared {
        at, job, cancel, ..
    }) = crate::canvas::inkpick::prepare(ctx, doc, page_index, point, zoom)
    else {
        return;
    };
    let job = job.with_separations(doc.separations.clone());
    let reading = Hover {
        doc: doc.serial,
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
    let repaint = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("ink-hover".into())
        .spawn(move || {
            let found = job.run();
            let Ok(mut guard) = state.lock() else {
                return;
            };
            if cancel.is_cancelled() {
                return;
            }
            crate::canvas::inkpick::trace_found(EVENT, page_index, at, &found);
            guard.cancel = None;
            guard.reading = Some(Hover { found, ..reading });
            drop(guard);
            repaint.request_repaint();
        });
    if spawned.is_err() {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("{EVENT}-refused reason=thread"));
    }
}

/// The tints a probe read: the four process plates (`None` when the page
/// was not composited in ink) and the spot plates by name.
#[derive(Debug, Clone, Copy)]
pub struct Tints<'a> {
    pub cmyk: Option<[f32; 4]>,
    pub spots: &'a [(String, f32)],
}

impl<'a> Tints<'a> {
    #[must_use]
    pub fn of(probe: &'a pdfcer_render::InkProbe) -> Self {
        Self {
            cmyk: probe.cmyk,
            spots: &probe.spots,
        }
    }

    /// The tint of plate `ink`, `0.0..=1.0`, or `None` if it was not read.
    #[must_use]
    pub fn tint(&self, ink: &str) -> Option<f32> {
        if let Some(i) = pdfcer_render::PROCESS_INKS.iter().position(|p| *p == ink) {
            return self.cmyk.map(|c| c[i]);
        }
        self.spots
            .iter()
            .find(|(name, _)| name == ink)
            .map(|(_, t)| *t)
    }

    /// Total area coverage: the sum of every plate's tint, `0.0..`.
    #[must_use]
    pub fn total(&self) -> Option<f32> {
        let cmyk = self.cmyk?;
        Some(cmyk.iter().sum::<f32>() + self.spots.iter().map(|(_, t)| *t).sum::<f32>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spots(list: &[(&str, f32)]) -> Vec<(String, f32)> {
        list.iter().map(|(n, t)| ((*n).to_owned(), *t)).collect()
    }

    #[test]
    fn a_process_plate_reads_its_channel_and_a_spot_its_name() {
        let s = spots(&[("PANTONE 185 C", 0.5)]);
        let t = Tints {
            cmyk: Some([0.1, 0.2, 0.3, 0.4]),
            spots: &s,
        };
        assert_eq!(t.tint("Magenta"), Some(0.2));
        assert_eq!(t.tint("Black"), Some(0.4));
        assert_eq!(t.tint("PANTONE 185 C"), Some(0.5));
        assert_eq!(t.tint("Orange"), None);
    }

    #[test]
    fn total_coverage_sums_process_and_spot_plates() {
        let s = spots(&[("Gold", 0.25)]);
        let t = Tints {
            cmyk: Some([1.0, 1.0, 0.0, 0.5]),
            spots: &s,
        };
        assert!((t.total().unwrap() - 2.75).abs() < 1e-6);
    }

    #[test]
    fn no_ink_buffer_has_no_tints_and_no_total() {
        let t = Tints {
            cmyk: None,
            spots: &[],
        };
        assert_eq!(t.tint("Cyan"), None);
        assert_eq!(t.total(), None);
    }
}
