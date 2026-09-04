//! `a_fit_command_puts_the_page_on_screen` — the three fit modes, driven, with
//! the view deliberately thrown away first.
//!
//! # The reports
//!
//! `OPERATOR_REQUESTS.md` O28 and O29, 2026-08-24:
//!
//! > *"If I press the Fit width or fit page button the view should center to
//! > the width as well or center the page."*
//!
//! > *"Adobe has fit height, so add that too."*
//!
//! # ★★★ Why the run must pan into the pasteboard FIRST
//!
//! This is the property that makes the check able to fail, and it is the same
//! shape `zoom_keeps_place`'s pan is: **the state O28 is about did not exist
//! before O23's pasteboard.** A page no larger than the viewport used to have
//! nowhere to be except the middle, so a fit that set only the scale looked
//! centred anyway. With a whole viewport of slack on every side, "the scale is
//! right and the page is not on screen" is reachable — and a check that
//! pressed Fit page from a centred start would watch the page "stay" where the
//! bug would have put it, pass, and mean nothing.
//!
//! So the run scrolls hard into the pasteboard, **asserts that it got there**
//! (a run that failed to displace the view has not set up its own precondition
//! and is a SKIP, not a pass), and only then presses the button.
//!
//! # What is asserted, per mode
//!
//! Read from the `canvas` trace line's `rect=`, which is the page's true drawn
//! rect on screen, against `canvas-viewport`'s:
//!
//! | mode | the claim |
//! |---|---|
//! | **Fit page** | every edge of the page is inside the viewport, **and the margins are equal on both axes** |
//! | **Fit width** | both vertical edges are flush with the viewport's, so the full width is what is on screen |
//! | **Fit height** | the mirror: both horizontal edges flush |
//!
//! ★★ Fit page's claim is *contained and centred*, not "fills both axes". A
//! landscape sheet in a tall window fills the width and floats in the middle
//! vertically, and a check that demanded both axes fill would fail on every
//! page whose aspect differs from the window's. **Equal margins is the direct
//! statement of what the operator asked for**, and it is also the one thing a
//! fit that set only the scale cannot produce: a page can be entirely inside
//! the viewport and still jammed against one edge with a viewport of
//! pasteboard on the other.
//!
//! ★ "Flush" within a tolerance, not exactly: the fit divides in `f32` and the
//! page rect is rounded to the pixel grid, so demanding exact equality would
//! be a check that fails on arithmetic rather than on behaviour. A few points
//! — far below the *hundreds* the defect moves the page by, and far above the
//! rounding.

use crate::checks::driving::{declared, declared_names, declared_or_in_overflow, list};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The canvas viewport, whose rect every claim below is made against.
const CANVAS_REGION: &str = "canvas-viewport";

/// The canvas's own state line, which carries the page's drawn `rect=`.
pub(super) const CANVAS_EVENT: &str = "canvas";

/// Where to roll the wheel to throw the view off the page, as a fraction of
/// the canvas.
const PAN_AT: (f32, f32) = (0.5, 0.5);

/// How many wheel notches to spend getting into the pasteboard.
///
/// O23 gives a whole viewport of slack on each side, so this has to be enough
/// to cross it. Overshooting is free — the scroll area clamps — and
/// undershooting would leave the page still on screen, which the precondition
/// below catches rather than silently accepting.
const PAN_NOTCHES: i32 = 30;

/// How far a fitted edge may miss the viewport's, in logical points.
///
/// ★ Not a tolerance on the DEFECT, which moves the page by a whole viewport
/// or more. This absorbs the `f32` fit division and the pixel-grid rounding of
/// the page's drawn rect, and nothing else: a build that placed the page one
/// tenth of a viewport out would still fail by a wide margin.
const EDGE_TOLERANCE: f32 = 4.0;

/// The gap the canvas deliberately leaves between the page and the panel
/// edges, in logical points — `canvas::CANVAS_MARGIN`.
///
/// ★★ Read once, from the application's own reason for existing rather than
/// from an observation. `canvas`'s comment: the margin is subtracted from the
/// viewport BEFORE the fit divides, *"so 'fit page' really does fit with the
/// gap visible instead of fitting exactly and then being clipped by the
/// gap"*. A check that demanded the page sit flush against the viewport would
/// therefore fail a correct build by exactly this number — which is what the
/// first driven run of this check did, and it is worth keeping the reason
/// rather than the constant: **an extreme-end mismatch is usually the
/// instrument, and here the instrument was asserting a promise the
/// application had never made.**
const CANVAS_MARGIN: f32 = 16.0;

/// How much smaller the window is made, in **physical pixels**, to test that a
/// fit survives a resize.
///
/// ★ Large enough that the change dwarfs [`EDGE_TOLERANCE`] — a resize the
/// check cannot distinguish from noise would make the phase vacuous — and
/// small enough that the ribbon still lays out, since a window too narrow to
/// draw the ribbon fails for a reason that is not the subject.
const RESIZE_BY_PX: i32 = 160;

/// The window chrome either side of the client area, in physical pixels.
///
/// ★★ Approximate on purpose, and the check does not depend on the number
/// being right: it resizes by a delta and asserts the CANVAS changed, then
/// restores by the same arithmetic. An error here makes the window a few
/// pixels different from where it started and is invisible to every claim —
/// whereas assuming `client_size` IS the window size would shrink it by the
/// chrome on every iteration, which compounds.
const BORDER_PX: i32 = 16;
/// The title bar and border, vertically. See [`BORDER_PX`].
const TITLEBAR_PX: i32 = 39;

/// See the module documentation.
pub struct AFitCommandPutsThePageOnScreen;

impl Check for AFitCommandPutsThePageOnScreen {
    fn name(&self) -> &'static str {
        "a_fit_command_puts_the_page_on_screen"
    }

    fn defect(&self) -> &'static str {
        "a fit command sets the scale and leaves the view wherever it was, so with O23's \
         pasteboard the page can be correctly sized and entirely off screen — and there is no \
         Fit height at all"
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

/// The page's drawn rect, as the canvas last reported it.
pub(super) fn page_rect(session: &Session) -> Result<Option<LRect>> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|line| line.get_rect("rect")))
}

/// Click a View-tab command by its ribbon item id.
pub(super) fn invoke(session: &Session, driver: &Driver, ui_rect: &str, item: &str) -> Result<()> {
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.view").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.view` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let region = declared_or_in_overflow(session, driver, ui_rect, item)?.ok_or_else(|| {
        Error::new(format!(
            "no `{item}` region on the View tab or in its overflow. Items declared: {}. \
             A registered command with no ribbon item is O29's whole failure mode.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.view."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(region))?;
    session.settle(24);
    Ok(())
}

/// What one fit mode promises about where the page ends up.
///
/// ★ Three claims rather than a pair of `fill` booleans, because the three
/// modes do not differ by a flag — they differ by **what they promise**, and
/// fit-page's promise is the odd one: it does not fill either axis in general
/// (a landscape sheet in a tall window fills the width and floats in the
/// middle vertically), it *contains and centres*. Writing that as two
/// booleans is what produced a first draft that asserted fit-page filled both
/// axes, which is false for every page whose aspect differs from the window's.
#[derive(Clone, Copy)]
enum Claim {
    /// Every edge inside the viewport, and equal margins on both axes.
    ContainedAndCentred,
    /// Centred horizontally, with the page using the full available width —
    /// the viewport less [`CANVAS_MARGIN`]. Says nothing about the other
    /// axis, which is expected to overflow and scroll.
    FillsWidth,
    /// The mirror of [`Self::FillsWidth`]: centred vertically, using the full
    /// available height, and expected to overflow sideways.
    FillsHeight,
}

/// One mode's claim, checked against the two rects.
fn verdict(label: &str, page: LRect, canvas: LRect, claim: Claim) -> Option<String> {
    let mut faults: Vec<String> = Vec::new();
    // An axis is "filled" when the page is centred on it AND the slack either
    // side is no more than the canvas's own margin. Both halves are needed:
    // centred alone would accept a page half the size of the window, and
    // slack alone would accept one jammed against an edge.
    let mut fills = |axis: &str, lo: f32, hi: f32, clo: f32, chi: f32| {
        let before = lo - clo;
        let after = chi - hi;
        if (before - after).abs() > EDGE_TOLERANCE {
            faults.push(format!(
                "the {axis} margins are {before:.1} and {after:.1}, so the page is not centred"
            ));
        }
        if before + after > CANVAS_MARGIN + EDGE_TOLERANCE {
            faults.push(format!(
                "the page spans {axis} {lo:.1}..{hi:.1} inside a canvas of {clo:.1}..{chi:.1}, leaving {:.1} points of slack where at most {CANVAS_MARGIN:.0} is the canvas's own margin — so its full extent is not what is on screen",
                before + after
            ));
        }
    };
    match claim {
        Claim::ContainedAndCentred => {
            if page.min.x < canvas.min.x - EDGE_TOLERANCE
                || page.max.x > canvas.max.x + EDGE_TOLERANCE
                || page.min.y < canvas.min.y - EDGE_TOLERANCE
                || page.max.y > canvas.max.y + EDGE_TOLERANCE
            {
                faults.push("part of the page is outside the canvas".to_owned());
            }
            // ★ THE claim of O28, stated directly: equal margins. A page can
            // be entirely inside the viewport and still jammed against one
            // edge with a viewport of pasteboard on the other, which is what
            // a fit that set only the scale produced.
            let left = page.min.x - canvas.min.x;
            let right = canvas.max.x - page.max.x;
            let top = page.min.y - canvas.min.y;
            let bottom = canvas.max.y - page.max.y;
            if (left - right).abs() > EDGE_TOLERANCE {
                faults.push(format!(
                    "the horizontal margins are {left:.1} and {right:.1}, so the page is not centred"
                ));
            }
            if (top - bottom).abs() > EDGE_TOLERANCE {
                faults.push(format!(
                    "the vertical margins are {top:.1} and {bottom:.1}, so the page is not centred"
                ));
            }
        }
        Claim::FillsWidth => fills("x", page.min.x, page.max.x, canvas.min.x, canvas.max.x),
        Claim::FillsHeight => fills("y", page.min.y, page.max.y, canvas.min.y, canvas.max.y),
    }
    if faults.is_empty() {
        return None;
    }
    Some(format!(
        "★★★ {label} DID NOT PUT THE PAGE ON SCREEN. {}. The view had been panned into O23's \
         pasteboard before the command was pressed, which is the state the request is about: \
         before O28 a fit set the SCALE and left the position alone, so the page could be \
         correctly sized and entirely off screen. `FitMode::pinned_axes` and \
         `geometry::fit_placement_offset` are the two halves of the rule; `canvas::fit` spends \
         the request a frame later, once the re-fitted zoom has landed.",
        faults.join("; ")
    ))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. There is nothing to fit."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check pans the canvas and presses ribbon \
             commands. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("fit-places-the-view.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    let canvas = declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;
    let at = frame.declared_at(canvas, PAN_AT.0, PAN_AT.1);

    for (item, label, claim) in [
        (
            "ribbon.item.view.zoom_fit_page",
            "FIT PAGE",
            Claim::ContainedAndCentred,
        ),
        (
            "ribbon.item.view.zoom_fit_width",
            "FIT WIDTH",
            Claim::FillsWidth,
        ),
        (
            "ribbon.item.view.zoom_fit_height",
            "FIT HEIGHT",
            Claim::FillsHeight,
        ),
    ] {
        // --- throw the view away ----------------------------------------
        //
        // ★ The precondition, established and ASSERTED rather than assumed —
        // rule 3 for a new check. A run whose pan did nothing would press the
        // button from a centred start and pass while measuring nothing.
        let before = page_rect(&session)?;
        driver.scroll_at(at, -PAN_NOTCHES)?;
        session.settle(20);
        let panned = page_rect(&session)?;
        match (before, panned) {
            (Some(a), Some(b)) if (a.min.y - b.min.y).abs() > EDGE_TOLERANCE => {}
            (_, Some(b)) => {
                return Err(Error::new(format!(
                    "the pan did not move the page (it is still at y {:.1}), so this run cannot \
                     tell a fit that PLACES the view from one that merely sets the scale. \
                     SKIPPED rather than passed.",
                    b.min.y
                )));
            }
            _ => {
                return Err(Error::new(
                    "the canvas never published a page rect. SKIPPED.".to_owned(),
                ));
            }
        }
        report.note(format!(
            "{label}: panned {PAN_NOTCHES} notches into the pasteboard first, and the page moved"
        ));

        // --- and press the button ----------------------------------------
        invoke(&session, &driver, ui_rect, item)?;
        let Some(page) = page_rect(&session)? else {
            return Err(Error::new(
                "the canvas stopped publishing a page rect. SKIPPED.".to_owned(),
            ));
        };
        // The canvas rect is re-read: a fit can change whether a scroll bar is
        // showing, and a scroll bar changes the viewport it is measured
        // against. Measuring the page against last frame's canvas is how a
        // correct build fails by a scrollbar's width.
        let canvas_now = declared(&session.trace()?, ui_rect, CANVAS_REGION).unwrap_or(canvas);
        report.note(format!(
            "{label}: page {:.1},{:.1}..{:.1},{:.1} in canvas {:.1},{:.1}..{:.1},{:.1}",
            page.min.x,
            page.min.y,
            page.max.x,
            page.max.y,
            canvas_now.min.x,
            canvas_now.min.y,
            canvas_now.max.x,
            canvas_now.max.y
        ));
        if let Some(failure) = verdict(label, page, canvas_now, claim) {
            return Ok(Some(failure));
        }

        // --- ★★★ AND THE SAME CLAIM AFTER A RESIZE — `O55` ----------------
        //
        // > *"if the canvas window is resized the pdf should resize to match"*
        //
        // ## What this catches, and why the phase above cannot
        //
        // `ViewState::apply_fit` recomputes the **zoom** from the viewport on
        // every frame a fit is active, so a resized window has always re-scaled
        // correctly. The **placement** was a one-shot: `canvas::fit::placement`
        // read `doc.fit_placement.take()`, set only by `Action::Fit`.
        //
        // ⇒ So the page grew or shrank about whatever offset it was sitting
        // at, and drifted off centre. The scale right, the position stale —
        // and every assertion above still passing, because they all run on the
        // frame after the button.
        //
        // **Re-asserting the identical claim after a resize is the whole
        // phase.** Nothing new is being measured; the same three claims are
        // being asked again in the state the operator reported.
        //
        // ★★ It resizes and then resizes BACK, so each loop iteration hands
        // the next one the window it was given. A check that shrank the window
        // three times would be measuring an ever-smaller viewport and would
        // eventually assert against one too small to lay the ribbon out.
        let Some(handle) = session.window() else {
            return Err(Error::new(
                "no window handle, so this check cannot resize. SKIPPED.".to_owned(),
            ));
        };
        let start = session.frame()?;
        // ★ `client_size` is the CLIENT area and `resize_window` takes the
        // whole window, so the borders are added back. Getting this wrong
        // shrinks the window a little more each iteration rather than
        // restoring it, which is the quiet kind of harness bug.
        let (cw, ch) = start.client_size;
        let (w, h) = (
            i32::try_from(cw).unwrap_or(1100) + BORDER_PX,
            i32::try_from(ch).unwrap_or(800) + TITLEBAR_PX,
        );
        crate::sys::resize_window(handle, w - RESIZE_BY_PX, h - RESIZE_BY_PX);
        session.settle(30);

        let resized_canvas = declared(&session.trace()?, ui_rect, CANVAS_REGION);
        let after = page_rect(&session)?;
        // Put it back before judging, so a failure does not leave the next
        // iteration measuring a window this one shrank.
        crate::sys::resize_window(handle, w, h);
        session.settle(25);

        let (Some(page_after), Some(canvas_after)) = (after, resized_canvas) else {
            return Err(Error::new(
                "the canvas stopped publishing its rect across the resize. SKIPPED.".to_owned(),
            ));
        };
        if (canvas_after.width() - canvas_now.width()).abs() < EDGE_TOLERANCE {
            return Err(Error::new(format!(
                "the resize did not change the canvas (still {:.1} pt wide), so this phase cannot tell a fit that RE-PLACES from one that placed once. The window may be at a size limit, or the desktop may have refused the call. SKIPPED rather than passed.",
                canvas_after.width()
            )));
        }
        report.note(format!(
            "{label} after resize: page {:.1},{:.1}..{:.1},{:.1} in canvas {:.1},{:.1}..{:.1},{:.1}",
            page_after.min.x,
            page_after.min.y,
            page_after.max.x,
            page_after.max.y,
            canvas_after.min.x,
            canvas_after.min.y,
            canvas_after.max.x,
            canvas_after.max.y
        ));
        if let Some(failure) = verdict(
            &format!("{label} AFTER A RESIZE"),
            page_after,
            canvas_after,
            claim,
        ) {
            return Ok(Some(format!(
                "{failure}
                 ★★★ **The claim held before the resize and not after**, which is
                 `OPERATOR_REQUESTS.md` O55 exactly: the zoom re-fits every frame and the PLACEMENT used to be a one-shot spent on the button press. `canvas::fit::placement` must fall back to `doc.view.fit` when no request is pending, not return `None`."
            )));
        }
    }

    Ok(None)
}
