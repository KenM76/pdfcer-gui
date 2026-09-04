//! `pages_stay_drawn_when_you_scroll_back` — **a page pdfcer has already drawn
//! is not drawn again**, measured by scrolling a real drawing set away and
//! back.
//!
//! # The operator's report, and why it needed driving
//!
//! 2026-08-19: *"increase cache to maximum for page view so they don't
//! constantly redraw with larger files."*
//!
//! He had diagnosed it correctly and the cause was not a size.
//! `render::strip::StripRasters::retain` was called once a frame as
//! `retain(&visible, current)` and its first line dropped **every entry not in
//! the visible set** — so the cache held exactly what was on screen. Scroll a
//! sheet off the top and it was gone; scroll back and it was rendered again
//! from the content stream, which `BENCHMARK.md` measures at **691 ms** for a
//! dense A1.
//!
//! ★★ **The budget had therefore never bitten.** 48 M texels is roughly
//! eighteen fit-width pages against a visible set of two or three, so the
//! eviction loop had never run on any document he had opened. *Raising the
//! number alone would have changed nothing at all* — which is exactly what
//! "increase the cache" invites a reader to do, and it is why this check
//! measures **re-requests** rather than the cache's size.
//!
//! # What it asserts, and why that is the only honest oracle
//!
//! `strip-raster-requested page=N` is emitted at the one place the strip asks
//! for a raster. **A page number appearing twice in that stream is the
//! defect**, verbatim: it means pdfcer drew a page, forgot it, and drew it
//! again.
//!
//! Nothing else would do. The cache's *size* is not the claim — a build that
//! held a gigabyte and still re-requested would pass a size assertion and fail
//! the operator. A screenshot is worse than useless here, because a re-rendered
//! page and a remembered one are **the same picture**; that is precisely why
//! this went unnoticed for the life of the shell, and why the only symptom was
//! a person waiting.
//!
//! # ★ It needs a continuous mode and a document with pages to spare
//!
//! Single-page mode keeps no strip at all (`fill_strip` clears the cache when
//! `strip_visible` is empty), so the check runs in **Read**, whose default is
//! continuous — `viewer::display::default_for_mode`. A document of at least a
//! few pages is required for a scroll to take one off screen; the check says so
//! rather than passing vacuously on a one-page fixture, which is
//! `CONTINUE.md` §7's rule about instruments that can only return one answer.

use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose default page display is continuous.
const MODE: &str = "read";
/// `strip-raster-requested page=N visible=M` — the one place the strip asks.
const REQUEST_EVENT: &str = "strip-raster-requested";
/// How many wheel notches to send in each direction.
///
/// ★ Enough to take **several** pages off screen and bring them back, and the
/// number was raised from 14 to 40 after the first driven run: 14 notches drew
/// only three pages, and a round trip over three pages is a weak sample for a
/// claim about a 36-sheet drawing set.
///
/// Sent as a burst rather than one at a time because the question is what
/// survives the round trip, not what happens during it.
///
/// ★★ **Falsified**, which is the part that makes the number meaningful: with
/// `retain` temporarily restored to keeping only the current page and its two
/// neighbours, this check FAILS and names the pages drawn twice. A check that
/// has never been shown to fail is a check that has measured nothing.
const NOTCHES: i32 = 40;

/// See the module documentation.
pub struct PagesStayDrawnWhenYouScrollBack;

impl Check for PagesStayDrawnWhenYouScrollBack {
    fn name(&self) -> &'static str {
        "pages_stay_drawn_when_you_scroll_back"
    }

    fn defect(&self) -> &'static str {
        "the page cache holds only what is on screen, so scrolling a sheet away and back \
         re-renders it from the content stream — 691 ms on a dense drawing, on every return, \
         for ever. Invisible to every other oracle: a re-rendered page and a remembered one are \
         the same picture, and the only symptom is a person waiting"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check scrolls a document until pages leave the viewport and come \
             back, so it needs one with pages to spare.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment and turns the \
             wheel over the canvas. Reported as SKIPPED rather than passed: a check that did \
             not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("page_cache.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env.push((
        driving::SHELL_DIAG_ENV.0.to_owned(),
        driving::SHELL_DIAG_ENV.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(50);
    let driver = Driver::new(session.window());

    // --- 1: Read, whose default page display is continuous ------------------
    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(40);

    // --- 2: aim at the middle of the canvas --------------------------------
    // ★ The canvas rect comes from the canvas's OWN trace event rather than
    // from a `ui-rect` region, because that is the event the strip's own
    // geometry is published in — so the point this check turns the wheel over
    // is guaranteed to be inside the thing whose scroll it is measuring.
    let trace = session.trace()?;
    let line = trace.last(ctx.profile.vocab.canvas_event).ok_or_else(|| {
        Error::new(format!(
            "the trace carries no `{}` event, so there is nowhere to put the wheel.",
            ctx.profile.vocab.canvas_event
        ))
    })?;
    let canvas = line
        .get_rect(ctx.profile.vocab.canvas_rect_field)
        .ok_or_else(|| {
            Error::new(format!(
                "the `{}` event has no parsable `{}=` field.",
                ctx.profile.vocab.canvas_event, ctx.profile.vocab.canvas_rect_field
            ))
        })?;
    let at = session.frame()?.declared_center(canvas);
    driver.move_to(at)?;
    session.settle(10);

    // --- 3: scroll away, and let the strip settle --------------------------
    //
    // ★ A settle after each direction rather than one at the end. The strip
    // renders one page per frame by design (`fill_strip`'s own budget), so a
    // check that scrolled and immediately looked would see a request stream
    // that had not finished — and would then read the *missing* requests as
    // evidence of a cache working.
    driver.scroll_at(at, -NOTCHES)?;
    session.settle(90);
    let away = requested(&session)?;
    if away.is_empty() {
        return Err(Error::new(format!(
            "no `{REQUEST_EVENT}` line after scrolling {NOTCHES} notches. Either this document \
             has too few pages for a strip, or the mode did not switch to a continuous display. \
             Reported as SKIPPED rather than PASSED: a check whose subject never happened has \
             measured nothing — and a cache that is never asked for anything trivially never \
             asks twice."
        )));
    }
    report.note(format!(
        "scrolling away drew {} page(s): {}",
        away.len(),
        list(&away)
    ));

    // --- 4: scroll back ----------------------------------------------------
    driver.scroll_at(at, NOTCHES)?;
    session.settle(90);
    let all = requested(&session)?;

    // ★★ THE ASSERTION. A page number appearing twice IS the defect.
    let mut seen = std::collections::BTreeSet::new();
    let mut twice: Vec<usize> = Vec::new();
    for page in &all {
        if !seen.insert(*page) && !twice.contains(page) {
            twice.push(*page);
        }
    }
    if twice.is_empty() {
        report.note(format!(
            "★★ scrolled {NOTCHES} notches away and {NOTCHES} back, and every one of the {} \
             page(s) drawn was drawn ONCE: {}",
            seen.len(),
            list(&all)
        ));
        return Ok(None);
    }

    Ok(Some(format!(
        "★ {} PAGE(S) WERE DRAWN TWICE: {}. The full request stream was {}.\n\
         A page that is re-requested after it has already been drawn is a page the cache \
         forgot, and on a dense drawing sheet that is 691 ms of waiting per return \
         (`BENCHMARK.md`).\n\
         Look at `render::strip::StripRasters::retain`. Until 2026-08-19 its first line was \
         `retain(|e| e.page != current && visible.contains(&e.page))`, which pruned the cache \
         to the visible set on every frame and made the texel budget decorative. If that line \
         is back, so is the operator's complaint. Trace: {}.",
        twice.len(),
        list(&twice),
        list(&all),
        session.trace_path().display()
    )))
}

/// Every page the strip has asked for, in order.
fn requested(session: &Session) -> Result<Vec<usize>> {
    Ok(session
        .trace()?
        .events(REQUEST_EVENT)
        .filter_map(|l| l.get_usize("page"))
        .collect())
}

/// A page list, for a message.
fn list(pages: &[usize]) -> String {
    if pages.is_empty() {
        // ui-text-exempt: a harness report, never rendered by the application.
        return "none".to_owned();
    }
    pages
        .iter()
        .map(|p| (p + 1).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
