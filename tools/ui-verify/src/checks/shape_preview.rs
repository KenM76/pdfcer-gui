//! `dragging_a_node_bends_the_line` — the driven proof of
//! `OPERATOR_REQUESTS.md` **O63**'s first and third pieces.
//!
//! # What this is about
//!
//! **Ken, 2026-08-30:** *"if I moved the end of a line, it didn't show me the
//! shape change of the line, it just had a perimeter box around it. this goes
//! for anything I change right now. there isn't a real preview like there is in
//! inkscape."*
//!
//! He was right, and it was a written convention rather than an oversight —
//! `canvas/handledrag.rs` said *"a preview shows the cursor, the render shows
//! the document."* That convention is now overruled by operator ruling.
//!
//! # ★★★ Why this needs TWO trace lines and not one
//!
//! The shell publishes:
//!
//! | line | question it answers |
//! |---|---|
//! | `canvas-shape-preview` | was a preview **built**? |
//! | `canvas-shape-drawn` | did it reach the **painter**? |
//!
//! A preview that is built and never painted — a `None` lost on the way through
//! `interact`, a painter arm never reached, a page index that does not match —
//! **looks exactly like a preview that was never built** to anything reading one
//! line. This project has shipped that shape of defect before: a panel that
//! rendered off-screen with every gate green, and a control published at its
//! content position that no pointer could ever reach.
//!
//! ⇒ So both are asserted, in order, and the failure message names which of the
//! two stages the build got to. *"Nothing happened"* and *"something happened
//! and nobody saw it"* have different causes and different fixes.
//!
//! # And the third piece: it has to OUTLIVE the release
//!
//! `canvas-held-preview` is written when the gesture hands its geometry to the
//! document to keep on screen while the page raster catches up. Without it the
//! operator watches the object snap back to where it started and then jump
//! forward a second later, which reads as the program refusing the edit and
//! changing its mind.
//!
//! ★ The hold is asserted **after** the drag, from the same trace, because it is
//! the half that has no visible difference from "no preview at all" on a page
//! that rasterises quickly — and every fixture in this repository rasterises
//! quickly. A check that only drove the fast case would pass on a build with the
//! hold deleted.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode segment to press: node editing lives in Edit.
const MODE: &str = "edit";

/// The anchor census, which tells this check the descent worked.
const ANCHORS_EVENT: &str = "canvas-anchors";

/// The preview was **built**.
const BUILT: &str = "canvas-shape-preview";

/// The preview reached the **painter**.
const DRAWN: &str = "canvas-shape-drawn";

/// The preview was handed to the document to outlive the gesture.
const HELD: &str = "canvas-held-preview";

/// How far to drag the anchor, in screen pixels.
///
/// ★ Far enough that the shape visibly changes and the move is not mistaken for
/// a click, and short enough to stay on the page. `multi_node` uses the same
/// figure for the same reasons.
const DRAG_PX: f32 = 25.0;

/// See the module documentation.
pub struct DraggingANodeBendsTheLine;

impl Check for DraggingANodeBendsTheLine {
    fn name(&self) -> &'static str {
        "dragging_a_node_bends_the_line"
    }

    fn defect(&self) -> &'static str {
        "dragging a line's end point shows a rectangle round it and never the line bending — the \
         operator's own comparison was Inkscape, and every gesture in this shell drew a bounding \
         outline and nothing else. Or the shape is built and never painted, which reads \
         identically from outside"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check descends to a node and drags it. \
             Reported as SKIPPED rather than passed.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a drawing with a path on page 1."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point on a SHAPE — the \
             Node rung only exists inside a path.",
        )
    })?;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("shape-preview.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- Edit mode, then descend twice to the Part rung ---------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let at = session.frame()?.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(12);
    driver.double_click_at(at)?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(anchors) = trace.last(ANCHORS_EVENT) else {
        return Err(Error::new(format!(
            "no `{ANCHORS_EVENT}` line after the descent, so the Part rung was never entered — \
             the point named a text run or an image, or the double-click was read as two single \
             clicks. Both are facts about the fixture and the harness rather than about the \
             preview. Trace: {}",
            session.trace_path().display()
        )));
    };
    report.note(format!("descended to a part: `{}`", anchors.raw));

    // --- aim at a published anchor and descend onto it ----------------------
    //
    // ★ The aim comes from the application's own published anchor rect, never
    // from arithmetic here. An anchor's screen position is a fact about the
    // page's decomposition, and a test tool that re-derived it could be wrong in
    // the same direction as the thing it is testing.
    let anchor = driving::declared(&trace, ui_rect, anchor_region(0)).ok_or_else(|| {
        Error::new(format!(
            "no `{}` was published, so there is no anchor to aim at. On an object with more \
             anchors than the overlay's cap the unselected marks are suppressed deliberately — \
             aim --doc-point at a simpler shape. Trace: {}",
            anchor_region(0),
            session.trace_path().display()
        ))
    })?;
    let from = session.frame()?.declared_center(anchor);
    // A DOUBLE click, because the Node rung is descended into rather than merely
    // clicked — `multi_node`'s header carries the driven run that established
    // this, and a plain click at the Part rung re-picks the *part*.
    driver.double_click_at(from)?;
    session.settle(16);

    // --- drag it ------------------------------------------------------------
    // ★ The destination is produced by `WindowFrame::offset_from`, never by
    // arithmetic on a `ScreenPoint`'s fields — `coords` keeps those private
    // precisely so a check cannot invent a desktop coordinate. One conversion,
    // one place.
    let frame = session.frame()?;
    driver.drag(from, frame.offset_from(from, DRAG_PX, DRAG_PX))?;
    session.settle(30);

    let trace = session.trace()?;

    // --- 1: was a preview BUILT? -------------------------------------------
    let built = trace
        .events(BUILT)
        .filter_map(|l| l.get_usize("segments"))
        .max()
        .unwrap_or(0);
    if built == 0 {
        return Ok(Some(format!(
            "★★★ NO SHAPE PREVIEW WAS BUILT. Not one `{BUILT}` line across the whole drag \
             carried `segments>0`, so `canvas::shapes::for_move_subject` either was not reached \
             or returned nothing for this rung. The operator gets the bounding box back and the \
             line does not bend — which is the exact complaint this feature answers. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ a preview was built, {built} segment(s) at its largest"
    ));

    // --- 2: did it reach the PAINTER? --------------------------------------
    //
    // ★★★ The assertion that makes this check worth having. A preview built and
    // never painted reads identically to one never built, from outside the
    // program — and this project has shipped that shape of defect before.
    let drawn = trace
        .events(DRAWN)
        .filter_map(|l| l.get_usize("segments"))
        .max()
        .unwrap_or(0);
    if drawn == 0 {
        return Ok(Some(format!(
            "★★★ THE PREVIEW WAS BUILT AND NEVER PAINTED. `{BUILT}` reports up to {built} \
             segments and no `{DRAWN}` line carries any, so the geometry exists and nothing put \
             it on screen. Look between `moving::drag` and `canvas::painting::draw`: a `None` \
             dropped in `dragroute`, a slot never copied into `painting::Frame`, or a page index \
             the painter arm does not match. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ and it reached the painter, {drawn} segment(s)"));

    // --- 3: did it OUTLIVE the release? ------------------------------------
    let held = trace
        .events(HELD)
        .filter_map(|l| l.get_usize("segments"))
        .max()
        .unwrap_or(0);
    if held == 0 {
        return Ok(Some(format!(
            "★★ THE PREVIEW DID NOT OUTLIVE THE GESTURE. No `{HELD}` line carries segments, so \
             the shape was discarded at release — and the page raster underneath still shows the \
             object where it STARTED until the new one lands. What the operator sees is the \
             object snapping back to where it was and then jumping forward, which reads as the \
             program refusing the edit and changing its mind. This is O63's third piece and it is \
             the operator's own sentence: \"the live preview should remain while the update to \
             the pdf structure runs in the background.\" Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ and it was held past the release, {held} segment(s), until the raster catches up"
    ));

    Ok(None)
}

/// The overlay's published anchor marks, by index.
///
/// ★ A local copy of `multi_node`'s, deliberately rather than a shared helper:
/// the list is the OVERLAY's contract about what it publishes, and two checks
/// naming it independently is what would catch a rename in one of them. A
/// shared constant would make both agree with each other and neither with the
/// application.
fn anchor_region(n: usize) -> &'static str {
    const NAMES: [&str; 6] = [
        "canvas.anchor.0",
        "canvas.anchor.1",
        "canvas.anchor.2",
        "canvas.anchor.3",
        "canvas.anchor.4",
        "canvas.anchor.5",
    ];
    NAMES[n.min(NAMES.len() - 1)]
}
