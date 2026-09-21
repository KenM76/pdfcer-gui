//! # `raster_wall::backout` — part C: the way back down
//!
//! ## The clause
//!
//! `OPERATOR_REQUESTS.md` **O220**, in his own words:
//!
//! > *"when the error occurs it prevents me from pressing ctrl and using the
//! > zoom wheel to zoom back out — I have to click the zoom out control on the
//! > bottom bar."*
//!
//! and, from the same report, what he expects instead: *"zoom out and it will
//! draw again."*
//!
//! ## Why this is a separate part and not three more lines in part B
//!
//! ★★★ **Every `scroll_at_held` in this check's other two parts rolls direction
//! `1`.** Parts A and B climb. A check that only ever pushes a control in one
//! direction cannot see a handler that is dead in the other, and *"it prevents
//! me from pressing ctrl and using the zoom wheel to zoom back out"* names a
//! gesture being taken away **at the wall specifically** — which is a state no
//! other check stands in.
//!
//! ⚠ **This is not the only `-1` in the harness and must not be described as
//! one.** [`crate::checks::off_sheet`] wheels out of the `nothing-visible`
//! blank, and its own header calls itself *the only driven coverage of
//! `canvas::escape::offer` anywhere* — see "What this part does not reach"
//! below, which is where that claim is still true.
//!
//! It reuses part B's climbed state rather than re-climbing, because reaching
//! the wall costs sixty-eight or more Ctrl+wheel notches on a 5.7 MB vector
//! drawing, and a second check that pays that again is minutes of wall clock
//! for a state part B is already standing in.
//!
//! ## The two states part B can leave, and why both are this part's subject
//!
//! | part B's exit | what stands | what the gesture has to do |
//! |---|---|---|
//! | the learned ceiling held | the canvas is DRAWING, clamped | the first batch out must lower the zoom |
//! | `canvas-unavailable` | the canvas is drawing NOTHING | wheeling out must make it draw again |
//!
//! The second is O220's literal state and the stronger measurement: it is the
//! one where both of `canvas::present::show_in`'s early returns sit above every
//! input handler in that function, so without `canvas::escape`'s hatch there is
//! no pointer gesture left at all. Part B treats it as NOT MEASURED —
//! correctly, since its own subject is the clamp — and hands it here.
//!
//! ⚠ **Neither state may be turned into a SKIP by this part.** Part B has
//! already measured something real by the time control reaches here; an `Err`
//! would discard its findings along with the whole check.
//!
//! ## The oracle, and why it is not a screenshot
//!
//! `canvas` and `canvas-unavailable` share one trace slot, so exactly one of
//! them is the standing verdict and [`unavailable_now`] decides which by line
//! number. That yields both halves of the operator's sentence from one read:
//!
//! * the zoom fell — `canvas … zoom=` is lower than where he was stuck;
//! * it draws again — a `canvas` line now stands *above* the
//!   `canvas-unavailable` line that was the verdict when this part began.
//!
//! ★ While the canvas is blank, [`latest_canvas`] returns a **fossil**: the last
//! frame that drew, which is the zoom the operator last had on screen. That is
//! deliberately the baseline. The view's own zoom went on climbing above it
//! while the screen was blank, so requiring the zoom to fall *below the fossil*
//! asserts the wheel brought him back past the point it stopped drawing — not
//! merely that some number moved.
//!
//! ## What this part must not do
//!
//! ⚠ **It must never touch the status bar's zoom-out control.** That control is
//! the workaround the operator was left with, so a check that reaches for it
//! measures the workaround and reports the defect fixed. The only gesture here
//! is Ctrl+wheel at the canvas.
//!
//! ## What this part does not reach, and which arm any given run measured
//!
//! ★★★ **Which of the two rows above a run lands on is not this part's
//! choice — it is whatever part B left — so a PASS here is not a claim about
//! both.** The note this part writes names the arm in as many words, and that
//! note is the only place the distinction is legible; read it before quoting a
//! green run at anything.
//!
//! Since O218's ceiling absorbs a `BeyondRaster` refusal into a learned limit,
//! the dense drawing's climb now ends **drawing** rather than blank, so the
//! ordinary path is the one usually exercised and `canvas::escape::offer` is
//! not on it at all. ⇒ **O220's literal precondition — *"when the error
//! occurs"*, i.e. `reason=render-failed` — has no driven coverage from here**;
//! `off_sheet` covers the hatch's other arm, `nothing-visible`, and nothing
//! covers `render-failed`. The operator still reports reaching that state, so
//! it is reachable and this harness cannot yet reach it on demand; O221's
//! document-count dependence is the standing candidate for the lever.

use super::trace::{latest_canvas, unavailable_now};
use super::{CANVAS_REGION, MESSAGE_REGION, UNAVAILABLE_EVENT};
use crate::checks::CheckReport;
use crate::checks::driving::declared;
use crate::error::Result;
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::Session;
use crate::sys::vk;
use crate::trace::Trace;

/// How many Ctrl+wheel notches out this part will spend before calling the way
/// back down unreachable.
///
/// Budgeted against the climb rather than guessed: part B spends at most
/// `CLIMB_NOTCHES_B` (160) plus `EXTRA_NOTCHES_B` (12) getting up there, and a
/// symmetric wheel cannot need more than that to come down. The margin is for
/// the learned ceiling's pull-back, which moves the view without costing a
/// notch.
const BACKOUT_NOTCHES: usize = 200;

/// Notches per posted batch.
///
/// Matches part B's `CLIMB_BATCH_B` so the two directions cost the same per
/// settle, which is what makes *"it came down in about as many notches as it
/// went up"* a sentence worth reading in the report.
const BACKOUT_BATCH: usize = 4;

/// How far the zoom has to fall for a batch to count as having done anything.
///
/// A notch is a multiplicative step, so any real response clears this by orders
/// of magnitude; the tolerance exists only so a float that came back
/// bit-identical is not read as a fall.
const MIN_FALL: f32 = 1.0e-3;

/// The state part B left behind, as the trace reports it.
#[derive(Clone, Debug)]
enum Standing {
    /// The canvas is drawing, at this zoom.
    Drawing(f32),
    /// The canvas is drawing nothing, for this reason, and the last zoom it
    /// managed to draw was this.
    Blank(String, f32),
}

impl Standing {
    /// The zoom the operator can currently see, or last could.
    fn zoom(&self) -> f32 {
        match self {
            Self::Drawing(z) | Self::Blank(_, z) => *z,
        }
    }
}

/// Read the standing verdict.
///
/// `None` when the canvas has never drawn a frame, which leaves this part with
/// no baseline to measure a fall against.
fn standing(trace: &Trace) -> Option<Standing> {
    let last_drawn = latest_canvas(trace)?.zoom;
    Some(match unavailable_now(trace) {
        Some((reason, _)) => Standing::Blank(reason, last_drawn),
        None => Standing::Drawing(last_drawn),
    })
}

/// ★★★ Part C. The operator's O220 clause: Ctrl+wheel still zooms **out** from
/// the state the wall put him in.
///
/// Returns `Some(failure)` when the gesture is dead or the canvas never comes
/// back. A state part B could not reach is `Ok(None)` **with a note** — see the
/// module header on why this part never produces a SKIP.
///
/// # Errors
///
/// Only from the driver and the trace reader; every diagnosis this part can
/// reach is expressed as a failure string or a note.
pub(super) fn part_c(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    canvas: LRect,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let Some(start) = standing(&session.trace()?) else {
        report.note(
            "part C NOT MEASURED: the canvas has never published a drawn frame, so there is no \
             zoom to measure a fall against. Part B's own notes say what it reached."
                .to_owned(),
        );
        return Ok(None);
    };

    // ★ The canvas centre, not the page rect's. At the wall the page is far
    // larger than the window, so its midpoint is a window-logical coordinate a
    // long way outside the window — and a Ctrl+wheel posted there goes to
    // whatever owns that pixel, which is the operator's desktop. Same reasoning
    // as part B's, and the same reason the frame is re-read here rather than
    // carried: a dock may have been laid out again since.
    let centre = session.frame()?.declared_at(canvas, 0.5, 0.5);

    match &start {
        Standing::Drawing(z) => report.note(format!(
            "part C begins from a DRAWING canvas at zoom {z:.2} ({:.0} %) — part B's learned \
             ceiling held. The assertion is that the very first batch of {BACKOUT_BATCH} \
             Ctrl+wheel notches out lowers it.",
            z * 100.0
        )),
        Standing::Blank(reason, z) => report.note(format!(
            "part C begins from a BLANK canvas (`{UNAVAILABLE_EVENT} reason={reason}`), whose \
             last drawn frame was zoom {z:.2} ({:.0} %) — ★ this is O220's literal state, the \
             stronger of the two this part can be handed. Both of `canvas::present::show_in`'s \
             early returns sit above every input handler in that function, so the only thing that \
             can answer the wheel here is `canvas::escape::offer`.",
            z * 100.0
        )),
    };

    let target = start.zoom() * (1.0 - MIN_FALL);
    let mut spent = 0usize;

    while spent < BACKOUT_NOTCHES {
        driver.scroll_at_held(centre, &[vk::CONTROL], -1, BACKOUT_BATCH)?;
        session.settle(24);
        spent += BACKOUT_BATCH;
        let trace = session.trace()?;
        let Some(now) = standing(&trace) else {
            continue;
        };

        // ★ The first batch is the one that answers the operator's sentence
        // when he was never blank: *"it prevents me from … zoom back out"*
        // describes a gesture that does nothing NOW, not one that is slow. A
        // check content to say "it came down within two hundred notches" would
        // pass against a wheel that needed a hundred and ninety of them, which
        // is the same complaint.
        if spent == BACKOUT_BATCH && matches!(start, Standing::Drawing(_)) && now.zoom() >= target {
            return Ok(Some(format!(
                "★★★ THE WHEEL DOES NOT COME BACK DOWN. The canvas was drawing at the learned \
                 ceiling, zoom {:.2}, and the first {BACKOUT_BATCH} Ctrl+wheel notches OUT left \
                 it at {:.2}. Zooming IN from the fit took this run to the wall in tens of \
                 notches, so the wheel is not slow in this direction — it is not being read. \
                 Worth reading in order: `canvas::zoom::wheel_step`'s sign, which comes from \
                 egui's `zoom_delta` and not from the raw scroll; a `viewer::ceiling` clamp \
                 re-applying the learned ceiling as a floor as well as a ceiling; and the Ctrl \
                 being lost from the event batch, which shows as the view PANNING instead — \
                 check whether `canvas rect=` moved before blaming the zoom.",
                start.zoom(),
                now.zoom()
            )));
        }

        if let Standing::Drawing(z) = now
            && z < target
        {
            report.note(format!(
                "part C: {spent} Ctrl+wheel notches out brought the canvas from {:.2} to {z:.2} \
                 ({:.0} % to {:.0} %) and it is DRAWING — the operator's *\"zoom out and it will \
                 draw again\"*, measured. ★ The status bar's zoom-out control was never touched; \
                 that control is the workaround he was left with, and a check that used it would \
                 report the defect fixed.",
                start.zoom(),
                start.zoom() * 100.0,
                z * 100.0
            ));
            return Ok(finish(&trace, ui_rect, spent));
        }
    }

    // The budget went and the canvas never came back down. Which of the two
    // ways it failed decides which sentence is true, and they are different
    // defects.
    let trace = session.trace()?;
    Ok(Some(match standing(&trace) {
        Some(Standing::Blank(reason, z)) => format!(
            "★★★ THE WAY BACK DOWN IS SHUT. {BACKOUT_NOTCHES} Ctrl+wheel notches OUT at the \
             canvas centre left the canvas still drawing nothing (`{UNAVAILABLE_EVENT} \
             reason={reason}`, last drawn zoom {z:.2}). The operator's words are exact: *\"when \
             the error occurs it prevents me from pressing ctrl and using the zoom wheel to zoom \
             back out — I have to click the zoom out control on the bottom bar.\"* Both early \
             returns in `canvas::present::show_in` sit ABOVE every input handler in that \
             function, so the wheel reaches nothing unless `canvas::escape::offer` runs before \
             each of them with a `hovered` gate that is true. ⚠ The fix is NOT to move either \
             `return` below the handlers: everything past them assumes a page was drawn and \
             several of those assumptions are unchecked. Start at `canvas::escape`, and at the \
             response each call site reads its gate from — the `render-failed` arm reads the \
             message placeholder's own response, because the scroll area whose response the other \
             arm passes is built below that return and does not exist there."
        ),
        Some(Standing::Drawing(z)) => format!(
            "the canvas is drawing but {BACKOUT_NOTCHES} Ctrl+wheel notches OUT never took the \
             zoom below {target:.2} — it stands at {z:.2} ({:.0} %). Something is holding the \
             zoom up; the candidates are the same three the first-batch arm of this part names.",
            z * 100.0
        ),
        None => format!(
            "after {BACKOUT_NOTCHES} Ctrl+wheel notches out the canvas publishes neither a drawn \
             frame nor a reason for drawing none. Neither half of O220's clause can be read, and \
             this is a failure rather than a note because a canvas that has stopped saying \
             anything at all is worse than either state this part was written for."
        ),
    }))
}

/// The assertions that only make sense once the canvas is drawing again.
///
/// Split out so the success path above reads as one sentence. Returns
/// `Some(failure)` exactly as the caller's own arms do.
fn finish(trace: &Trace, ui_rect: &str, spent: usize) -> Option<String> {
    // ★ The sentence the operator was told to act on has to go when he acts on
    // it. `declared` answers by line number, so a region that was published and
    // later retired reads as absent here — which is the question being asked.
    if let Some(rect) = declared(trace, ui_rect, MESSAGE_REGION) {
        return Some(format!(
            "the zoom came back down after {spent} notches out and the canvas is drawing, but the \
             refusal sentence is STILL on the page, at {:.0},{:.0}..{:.0},{:.0}. \
             `{MESSAGE_REGION}` is published by the `render-failed` arm of \
             `canvas::present::show_in` and has to be retired on the first frame that draws — a \
             sentence reading *\"this page could not be drawn\"* standing over a page that just \
             drew is the operator's own defect, read from the other end.",
            rect.min.x, rect.min.y, rect.max.x, rect.max.y
        ));
    }
    if declared(trace, ui_rect, CANVAS_REGION).is_none() {
        return Some(format!(
            "the zoom came down after {spent} notches out and the trace says the canvas drew, but \
             no `{CANVAS_REGION}` region is standing — so the widget that did the drawing is not \
             laid out anywhere the operator can see it. Drawn is not seen."
        ));
    }
    None
}
