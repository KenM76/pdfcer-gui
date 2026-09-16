//! `the_outline_follows_the_pointer_and_is_what_gets_placed` — **the live
//! preview of a form field's size and placement** — `OPERATOR_REQUESTS.md`
//! O203.
//!
//! # The operator's report
//!
//! > *"when placing form items, there should be a live preview of their size
//! > and placement of what we will get if we just click once to place them."*
//!
//! Read the last clause carefully. He did not ask for *an outline*; he asked
//! for the outline **of what he will get**. A preview that tracks the pointer
//! and promises a rectangle the click does not write is worse than no preview,
//! because it is a promise the program then breaks.
//!
//! # The three assertions, in the order they build on each other
//!
//! 1. **There is an outline at all**, with a form tool armed and the pointer
//!    over the canvas: a `form-ghost` line.
//! 2. **It follows.** Move the pointer by a known distance in PDF points and
//!    the rectangle moves by the same distance. A ghost drawn once and left
//!    where it was satisfies assertion 1 completely.
//! 3. **It was the truth.** Click, let the placement dialog accept its own
//!    defaults, and the rectangle the application publishes for the field it
//!    just authored is the rectangle the ghost was drawing. This is the one
//!    that makes the feature the feature.
//!
//! # Why a screenshot cannot do any of it
//!
//! A ghost that tracks the pointer while promising the wrong `/Rect` and a
//! ghost that promises the right one while drawn in the wrong place are
//! **the same picture** at the moment of the screenshot, and the second is
//! only visible one frame later when the field appears somewhere else. The
//! trace line carries both rectangles — the PDF `/Rect` the click would write
//! and the screen box being drawn — precisely so a harness can separate them.
//!
//! # And why the click point is checked against every corner
//!
//! `formfield::ghost::click_rect` anchors the click at the **lower-left**
//! corner in PDF space, to match what a drag does. On a `/Rotate 90` sheet the
//! same PDF corner is a different corner of what the operator sees, so the
//! check asserts the click point is *a* corner rather than naming one. An
//! assertion that named the corner would be a claim about the fixture's
//! rotation dressed up as a claim about the feature.

use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// Enter Edit and arm the text-field tool. No `file.properties`: this check
/// never reads the properties panel, and a command it does not need is a
/// command that can fail on its behalf.
const INVOKE: &str = "mode.edit,edit.form_text_field";
/// The placement dialog presses its own Add on the first frame it is
/// authorable. See `form_field`'s header: the dialog is a deferred viewport
/// with a window of its own and this harness drives one window.
const ACCEPT_ENV: (&str, &str) = ("PDFCER_DIAG_FORM_ACCEPT", "1");
/// `form-tool-armed kind=…`.
const ARMED: &str = "form-tool-armed";
/// `form-ghost kind=… rect=llx,lly,urx,ury screen=…` — one per frame the
/// pointer is over the canvas with a tool armed.
const GHOST: &str = "form-ghost";
/// The fillable census, which is where the placed field's rectangle is read
/// back from.
const BOX_LINE: &str = "form-box";

/// How far the pointer is moved between the two ghost samples, in PDF points.
///
/// Large enough that it cannot be confused with the jitter of a pointer
/// landing on a pixel boundary — a page point is well under a screen pixel at
/// the zoom a fitted A1 gets, so a one-pixel rounding is a fraction of a
/// point — and small enough to stay on the sheet and inside the viewport from
/// any aim point a caller would choose.
const STEP_PT: f64 = 80.0;

/// How far the measured motion may differ from [`STEP_PT`], in PDF points.
///
/// The pointer is set in whole screen pixels and the ghost is computed from
/// the pointer's canvas position, so the round trip through screen space
/// quantises to one pixel. At a fitted A1 that is roughly two page points;
/// four is one pixel of headroom on top, and it is still a twentieth of the
/// motion being measured — a ghost that did not follow at all is out by 80.
const TOLERANCE_PT: f64 = 4.0;

/// See the module documentation.
pub struct TheOutlineFollowsThePointerAndIsWhatGetsPlaced;

impl Check for TheOutlineFollowsThePointerAndIsWhatGetsPlaced {
    fn name(&self) -> &'static str {
        "the_outline_follows_the_pointer_and_is_what_gets_placed"
    }

    fn defect(&self) -> &'static str {
        "placing a form field is a blind click: nothing shows where the control will land or how \
         big it will be until it is already in the document, so the only way to find out is to \
         place one and undo it. And the half a screenshot cannot see — an outline that tracks \
         the pointer while promising a rectangle the click does not write"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A PDF rectangle from a `form-ghost` line, as `(llx, lly, urx, ury)`.
type Quad = (f64, f64, f64, f64);

/// The most recent ghost rectangle.
fn last_ghost(trace: &Trace) -> Option<Quad> {
    let raw = trace.events(GHOST).last()?.get("rect")?.to_owned();
    let mut parts = raw.split(',').map(|n| n.trim().parse::<f64>());
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(Ok(a)), Some(Ok(b)), Some(Ok(c)), Some(Ok(d))) => Some((a, b, c, d)),
        _ => None,
    }
}

/// Whether `(x, y)` is one of the rectangle's four corners.
///
/// Rotation-agnostic on purpose — see the module header.
fn is_a_corner(rect: Quad, x: f64, y: f64) -> bool {
    let (llx, lly, urx, ury) = rect;
    [(llx, lly), (llx, ury), (urx, lly), (urx, ury)]
        .iter()
        .any(|(cx, cy)| (cx - x).abs() <= TOLERANCE_PT && (cy - y).abs() <= TOLERANCE_PT)
}

/// The canvas-space rectangle the application publishes for `field`.
fn census_rect(trace: &Trace, field: &str) -> Option<(f64, f64, f64, f64)> {
    let raw = trace
        .events(BOX_LINE)
        .filter(|l| l.get("field") == Some(field))
        .last()?
        .get("rect")?
        .to_owned();
    // `rect=(x,y)+(w,h)`.
    let (min, size) = raw.split_once(")+(")?;
    let (x, y) = min.trim_start_matches('(').split_once(',')?;
    let (w, h) = size.trim_end_matches(')').split_once(',')?;
    let x: f64 = x.trim().parse().ok()?;
    let y: f64 = y.trim().parse().ok()?;
    let w: f64 = w.trim().parse().ok()?;
    let h: f64 = h.trim().parse().ok()?;
    Some((x, y, w, h))
}

#[allow(
    clippy::too_many_lines,
    reason = "one driven sequence; splitting it would hide the ORDER, which is the subject" // ui-text-exempt: lint justification
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check hovers and clicks a real page."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check moves the pointer twice and clicks once. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space on the page this check should hover \
             over. There is deliberately no default: a point off the sheet produces no ghost, \
             and \"no ghost\" is the defect this check exists to report.",
        )
    })?;
    let vocab = &ctx.profile.vocab;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    // --- 1: launch with the text-field tool already armed ------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("form_ghost.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so {}={} did not reach the process. Trace: {}.",
            vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    let Some(armed) = trace.last(ARMED) else {
        return Ok(Some(format!(
            "no `{ARMED}` line. The commands `{INVOKE}` were rung and no form tool was armed, so \
             there is no placement for a preview to preview. This is `form_field`'s phase A \
             failing, not O203's subject — but it fails here too, so the sentence names it. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("armed kind={:?}", armed.get("kind")));

    // --- 2: hover, and there is an outline ---------------------------------
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    let driver = Driver::new(session.window());

    let a = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    driver.move_to(frame.to_screen(a))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(first) = last_ghost(&trace) else {
        return Ok(Some(format!(
            "the pointer is over the page with a form tool armed and nothing is drawn: no \
             `{GHOST}` line. This is the operator's report verbatim — placing a form field is a \
             blind click, and the only way to see what you will get is to place one and undo it. \
             Look at `canvas::painting`'s call of `formfield::ghost::preview` and at \
             `canvas::tool` for whether the armed kind reaches it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if !is_a_corner(first, target.x, target.y) {
        return Ok(Some(format!(
            "the outline is drawn at ({:.1}, {:.1})-({:.1}, {:.1}) and the pointer is at ({}, \
             {}), which is not one of its corners. `formfield::ghost::click_rect` anchors the \
             click at a corner — the same thing a drag does with its press point — so an outline \
             the pointer is not on a corner of is one whose geometry disagrees with the \
             placement it is previewing. Trace: {}.",
            first.0,
            first.1,
            first.2,
            first.3,
            target.x,
            target.y,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the outline is at ({:.1}, {:.1})-({:.1}, {:.1}), with the pointer on a corner of it",
        first.0, first.1, first.2, first.3
    ));

    // --- 3: move, and it follows -------------------------------------------
    //
    // The second point is offset on BOTH axes. One axis would let a ghost that
    // ignored the other pass, and "the preview only follows sideways" is a
    // defect the operator would meet on his first vertical placement.
    let (bx, by) = (target.x + STEP_PT, target.y + STEP_PT);
    let b = mapping
        .doc_to_window(DocPoint::new(target.page, bx, by))
        .map_err(|why| {
            Error::new(format!(
                "the second hover point ({bx}, {by}) — {STEP_PT} pt from the aim point on both \
                 axes — is not reachable: {why}. Reported as a SKIP: that is a property of \
                 `--doc-point` and the page size, not of the preview. Aim further from the edge."
            ))
        })?;
    driver.move_to(frame.to_screen(b))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(second) = last_ghost(&trace) else {
        return Ok(Some(format!(
            "the outline appeared at the first hover and there is no `{GHOST}` line after the \
             second. The preview is drawn once and does not survive the pointer moving, which is \
             not a live preview. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let moved = (second.0 - first.0, second.1 - first.1);
    if (moved.0 - STEP_PT).abs() > TOLERANCE_PT || (moved.1 - STEP_PT).abs() > TOLERANCE_PT {
        return Ok(Some(format!(
            "the pointer moved {STEP_PT} pt on each axis and the outline moved ({:.1}, {:.1}). \
             A preview that does not track the pointer is an outline, not a preview: it answers \
             \"how big\" and refuses \"where\", and where is the half he cannot recover from \
             without an undo. If one axis is right and the other is zero, look at \
             `markup::band::endpoints`, which is where the canvas point becomes a PDF one. \
             Trace: {}.",
            moved.0,
            moved.1,
            session.trace_path().display()
        )));
    }
    let size = ((second.2 - second.0), (second.3 - second.1));
    if (size.0 - (first.2 - first.0)).abs() > TOLERANCE_PT
        || (size.1 - (first.3 - first.1)).abs() > TOLERANCE_PT
    {
        return Ok(Some(format!(
            "the outline changed SIZE as the pointer moved: {:.1}x{:.1} became {:.1}x{:.1}. The \
             preview promises the size of a single click's field, which is the armed kind's \
             default and does not depend on where the pointer is. Trace: {}.",
            first.2 - first.0,
            first.3 - first.1,
            size.0,
            size.1,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the outline followed the pointer by ({:.1}, {:.1}) pt without changing its {:.1}x{:.1} \
         pt size",
        moved.0, moved.1, size.0, size.1
    ));

    // --- 4: click, and the field is where the outline said ------------------
    driver.click_at(frame.to_screen(b))?;
    session.settle(35);

    let trace = session.trace()?;
    // The census is canvas space and the ghost is PDF space, so one of them
    // has to be converted. Through the mapping, never through a
    // `page_height - y` flip written here: the flip is right only on an
    // upright page, and the operator's own drawing is /Rotate 270.
    let Some(opened) = trace.last("form-field-open") else {
        return Ok(Some(format!(
            "the click placed nothing: no `form-field-open` line, on a run where the tool was \
             armed and the outline was tracking. The gesture did not resolve to a placement. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let Some(name) = opened.get("name").filter(|n| !n.is_empty()) else {
        return Ok(Some(
            "the placement dialog opened with no generated field name, so nothing could be \
             authored and there is no placed rectangle to compare the outline against."
                .to_owned(),
        ));
    };
    let name = name.to_owned();
    let Some((cx, cy, cw, ch)) = census_rect(&trace, &name) else {
        return Ok(Some(format!(
            "the field {name:?} was placed and the canvas publishes no `{BOX_LINE}` line for it, \
             so this check cannot read back where it landed. Either the field was authored \
             without a drawn appearance — in which case it is not on the canvas at all, which is \
             a bigger defect than the one under test — or the census stopped being written. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let promised = mapping.user_rect_to_canvas(second);
    let placed = (cx, cy, cx + cw, cy + ch);
    let off = (
        (placed.0 - promised.0).abs(),
        (placed.1 - promised.1).abs(),
        (placed.2 - promised.2).abs(),
        (placed.3 - promised.3).abs(),
    );
    if off.0 > TOLERANCE_PT || off.1 > TOLERANCE_PT || off.2 > TOLERANCE_PT || off.3 > TOLERANCE_PT
    {
        return Ok(Some(format!(
            "★ THE OUTLINE PROMISED ({:.1}, {:.1})-({:.1}, {:.1}) AND THE CLICK PLACED {name:?} \
             AT ({:.1}, {:.1})-({:.1}, {:.1}). That is the whole of what he asked for — *\"a \
             live preview … of what we will get if we just click once\"* — and a preview that \
             lies is worse than none, because it is a promise the program then breaks.\n\
             `formfield::ghost::click_rect` is supposed to be the ONLY producer of a click's \
             /Rect, with `canvas::clicking` calling it rather than computing its own. A \
             disagreement here means there are two derivations again, and they agree on an \
             unrotated page and on nothing else. Trace: {}.",
            promised.0,
            promised.1,
            promised.2,
            promised.3,
            placed.0,
            placed.1,
            placed.2,
            placed.3,
            session.trace_path().display()
        )));
    }

    report.note(format!(
        "★★ the outline tracked the pointer and the click placed {name:?} at exactly the \
         rectangle it was drawing: ({:.1}, {:.1})-({:.1}, {:.1})",
        placed.0, placed.1, placed.2, placed.3
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_click_point_is_accepted_at_any_of_the_four_corners() {
        let rect = (100.0, 200.0, 300.0, 250.0);
        for (x, y) in [
            (100.0, 200.0),
            (100.0, 250.0),
            (300.0, 200.0),
            (300.0, 250.0),
        ] {
            assert!(
                is_a_corner(rect, x, y),
                "({x}, {y}) is a corner of {rect:?}"
            );
        }
    }

    #[test]
    fn the_centre_of_the_rectangle_is_not_a_corner() {
        // The defect this distinguishes: a `click_rect` that centred the field
        // on the pointer instead of anchoring it. It is a plausible reading of
        // "place it here" and it disagrees with what a drag does.
        assert!(!is_a_corner((100.0, 200.0, 300.0, 250.0), 200.0, 225.0));
    }

    #[test]
    fn a_point_one_tolerance_off_a_corner_is_still_that_corner() {
        let rect = (100.0, 200.0, 300.0, 250.0);
        assert!(is_a_corner(
            rect,
            100.0 + TOLERANCE_PT,
            200.0 - TOLERANCE_PT
        ));
        assert!(!is_a_corner(
            rect,
            100.0 + TOLERANCE_PT * 2.0,
            200.0 - TOLERANCE_PT * 2.0
        ));
    }
}
