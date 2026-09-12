//! `switching_the_page_display_recentres_and_a_facing_fit_fits_the_spread` —
//! **O177**, both halves, driven.
//!
//! # The report
//!
//! `OPERATOR_REQUESTS.md` **O177**, 2026-09-12:
//!
//! > *"when switching the view from scroll pages to show one page at a time or
//! > show two pages side by side the page or pages view should snap back to
//! > center of the canvas. also fit page when in 2 pages side by side views
//! > should fit the two side by side pages onto the canvas - right now it snaps
//! > to fitting one."*
//!
//! Two sentences, two independent defects, and they are checked in one run
//! because they share a fixture, a window size and a display mode — not because
//! they share a cause. They do not:
//!
//! | half | what is wrong |
//! |---|---|
//! | **recentre** | `Action::SetPageDisplay` deliberately suppresses the scroll-to-page that a page change would otherwise cause, so the continuous strip's offset survives the switch and the new arrangement is drawn wherever that offset happens to land |
//! | **fit the spread** | the fit's **scale** is computed from the ROW (`viewer::strip::fit_metrics`, which is facing-aware) and the fit's **placement** is converted back through the acting PAGE's rect (`canvas::offset`'s `strip_offset_for`), so a spread is placed as though the page were the thing being centred — one page lands in the middle and the other runs off the edge, which reads exactly as *"it snaps to fitting one"* |
//!
//! ★ That asymmetry is the finding worth carrying away: **when a feature's
//! scale rule is taught about a new layout unit and its placement rule is not,
//! the symptom presents as the scale being wrong.** The operator's sentence
//! says "fit", and the zoom was never the problem.
//!
//! # ★★★ Why `canvas-strip` exists and why neither half could be checked
//! without it
//!
//! The canvas has published `page` (the acting page's drawn rect) since Phase 1
//! and `canvas-viewport` since O23. Under a facing mode `page` is **half of
//! what the operator is looking at**, so a check asserting "the thing on screen
//! is centred" from it would have to reconstruct the other half from a
//! PDF-scanned page size and a hard-coded spread gap — i.e. re-derive the
//! application's own layout arithmetic in the harness, and agree with it. That
//! is the defect class `crop=` and `rot=` were added to the canvas trace to
//! end.
//!
//! `canvas-strip` is the union of the rects the canvas actually **drew** this
//! frame, folded from the same `drawn` vector the rasters went into. It cannot
//! disagree with what was painted, and it degenerates to the page rect under
//! `Single`, which is what lets one assertion cover both halves.
//!
//! # ⚠ Why every assertion is made in a NON-continuous mode
//!
//! `canvas-strip` is the union of the pages drawn *this frame*. Under a
//! continuous mode that is the visible run of the strip, which is by
//! construction about the size of the viewport and about centred in it —
//! **always**, on a correct build and on a broken one alike. Asserting
//! containment or centring there would be asserting a tautology.
//!
//! So the run uses continuous only to *create* the displaced state, and every
//! claim is made after the switch out of it. The displacement itself is
//! measured from the acting page's `rect=` on the `canvas` trace line, which
//! does move when the strip scrolls.
//!
//! # The fixture, and the cover rule
//!
//! `fixtures/four-pages.pdf`, pinned rather than taken from `--pdf`, because
//! the facing half needs a document with a genuine two-page row and the
//! operator's own drawings are frequently single-sheet.
//!
//! ★★ `viewer::display::PageDisplay::row_of` implements the **cover rule**:
//! row 0 holds page 0 **alone**, and rows 1.. hold `2r-1` and `2r`. A facing
//! fit measured at launch therefore measures **one page** and would pass on a
//! build that cannot fit two. The run presses **Page Down** once to reach page
//! index 1 before it measures anything, and then asserts that what it is
//! looking at really is wider than one page — a precondition that fails as a
//! SKIP rather than being assumed.
//!
//! # What a passing run does NOT prove
//!
//! That the zoom is the largest one that still fits (that is `fit_metrics`'
//! own subject and is unit-tested), that the spread's *gap* is right, or that
//! anything is correct under a continuous mode — see the warning above.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared};
use crate::checks::fit_places_the_view::{CANVAS_EVENT, invoke, page_rect};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use std::path::PathBuf;

/// The document, pinned. See the module header.
const FIXTURE: &str = "fixtures/four-pages.pdf";
/// The mode whose ribbon carries the View tab.
const MODE: &str = "review";
/// The canvas viewport every claim is made against.
const CANVAS_REGION: &str = "canvas-viewport";
/// The union of the pages drawn this frame. See the module header.
const STRIP_REGION: &str = "canvas-strip";

/// Where and how large the window is placed, as `PDFCER_DIAG_VIEWPORT` takes
/// it: `x,y,w,h`.
///
/// ★ Fixed rather than maximised so the numbers below mean the same thing on
/// every machine, and wide enough that a two-page spread at a readable zoom is
/// a shape the fit can actually produce. `PDFCER_DIAG_VIEWPORT` switches
/// `with_active` off, so the window lays out fully without taking the desktop.
const VIEWPORT: &str = "0,0,1600,1000";

/// How far the drawn strip's centre may sit from the viewport's, in logical
/// points, and still count as centred.
///
/// ★ Generous on purpose. The page rect is rounded to the pixel grid, the fit
/// divides in `f32`, and a scroll bar appearing or disappearing moves the
/// viewport by its own width. The defect this check is about moves the strip by
/// **half a page** — 300-plus points at the zoom this run reaches — so a
/// tolerance two orders of magnitude below that separates the two states with
/// room to spare, and a tighter one would report arithmetic rather than
/// behaviour.
const CENTRE_TOLERANCE_PT: f32 = 12.0;

/// How far outside the viewport a drawn edge may sit and still count as
/// contained. Same reasoning as [`CENTRE_TOLERANCE_PT`], one step tighter
/// because containment is a claim about an edge rather than about a midpoint.
const CONTAIN_TOLERANCE_PT: f32 = 4.0;

/// How far the acting page's rect must move for the wheel to have established
/// the displaced state this check needs.
///
/// ★ A precondition that is ASSERTED, not assumed. A run whose scroll did
/// nothing would switch display modes from an already-centred start and pass
/// while measuring nothing — the exact shape `fit_places_the_view`'s own pan
/// precondition exists to prevent.
const DISPLACED_PT: f32 = 40.0;

/// How many wheel notches to spend scrolling the continuous strip away from
/// wherever it opened.
///
/// Overshooting is free: the scroll area clamps, and a strip clamped to its
/// bottom is displaced just as thoroughly as one stopped in the middle.
const SCROLL_NOTCHES: i32 = 12;

/// Where on the canvas the wheel is rolled.
const SCROLL_AT: (f32, f32) = (0.5, 0.5);

/// The narrowest a two-page row may be, as a multiple of one page's width,
/// before this check believes it is looking at a spread.
///
/// ★ 1.5 rather than 2.0: the row is two pages **plus** a gap, so the true
/// ratio is a little over 2, and a floor at 1.5 is unambiguous against the
/// thing it has to exclude — a row of one page, ratio exactly 1.
const SPREAD_RATIO: f32 = 1.5;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// See the module documentation.
pub struct SwitchingThePageDisplayRecentresAndAFacingFitFitsTheSpread;

impl Check for SwitchingThePageDisplayRecentresAndAFacingFitFitsTheSpread {
    fn name(&self) -> &'static str {
        "switching_the_page_display_recentres_and_a_facing_fit_fits_the_spread"
    }

    fn defect(&self) -> &'static str {
        "switching out of a scrolled continuous view keeps the old scroll offset, so the page \
         lands wherever that offset happened to point instead of in the middle of the canvas — \
         and Fit page under a facing spread centres ONE of the two pages, because the fit's \
         scale is computed from the row and its placement from the acting page"
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

/// The drawn strip and the canvas viewport, as of the last frame.
///
/// Both are read from the **same** trace snapshot: reading them from two
/// snapshots is how a check comes to compare a strip from one frame against a
/// viewport from another, which on a frame where a scroll bar appeared is a
/// difference of fifteen points for no reason at all.
fn strip_and_viewport(session: &Session, ui_rect: &str) -> Result<Option<(LRect, LRect)>> {
    let trace = session.trace()?;
    let strip = declared(&trace, ui_rect, STRIP_REGION);
    let canvas = declared(&trace, ui_rect, CANVAS_REGION);
    Ok(match (strip, canvas) {
        (Some(s), Some(c)) => Some((s, c)),
        _ => None,
    })
}

/// The verdict for "what is drawn is centred in the canvas", or `None`.
///
/// Reported per axis so the failure text names which way the view is off,
/// which is the first thing anybody reading it wants to know.
fn centred(label: &str, strip: LRect, canvas: LRect) -> Option<String> {
    let dx = (strip.min.x + strip.max.x) / 2.0 - (canvas.min.x + canvas.max.x) / 2.0;
    let dy = (strip.min.y + strip.max.y) / 2.0 - (canvas.min.y + canvas.max.y) / 2.0;
    if dx.abs() <= CENTRE_TOLERANCE_PT && dy.abs() <= CENTRE_TOLERANCE_PT {
        return None;
    }
    Some(format!(
        "{label}: what the canvas drew is {dx:.0} pt right and {dy:.0} pt below the middle of \
         the canvas (tolerance {CENTRE_TOLERANCE_PT:.0} pt). Drawn \
         {:.0},{:.0}..{:.0},{:.0} in canvas {:.0},{:.0}..{:.0},{:.0}.",
        strip.min.x,
        strip.min.y,
        strip.max.x,
        strip.max.y,
        canvas.min.x,
        canvas.min.y,
        canvas.max.x,
        canvas.max.y
    ))
}

/// The verdict for "what is drawn is inside the canvas", or `None`.
fn contained(label: &str, strip: LRect, canvas: LRect) -> Option<String> {
    let over_left = canvas.min.x - strip.min.x;
    let over_top = canvas.min.y - strip.min.y;
    let over_right = strip.max.x - canvas.max.x;
    let over_bottom = strip.max.y - canvas.max.y;
    let worst = over_left.max(over_top).max(over_right).max(over_bottom);
    if worst <= CONTAIN_TOLERANCE_PT {
        return None;
    }
    Some(format!(
        "{label}: {worst:.0} pt of what the canvas drew is OUTSIDE the canvas (left {over_left:.0}, \
         top {over_top:.0}, right {over_right:.0}, bottom {over_bottom:.0}; tolerance \
         {CONTAIN_TOLERANCE_PT:.0} pt). Drawn {:.0},{:.0}..{:.0},{:.0} in canvas \
         {:.0},{:.0}..{:.0},{:.0}. Under Fit page every edge must be on screen: the whole point \
         of the command is that the operator can see all of it at once.",
        strip.min.x,
        strip.min.y,
        strip.max.x,
        strip.max.y,
        canvas.min.x,
        canvas.min.y,
        canvas.max.x,
        canvas.max.y
    ))
}

/// The display mode the canvas last said it was in, for the report's notes and
/// for a precondition that would otherwise be silent: a run whose ribbon click
/// missed would go on measuring a mode it never entered.
fn display_mode(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|line| line.get("display").map(str::to_owned)))
}

/// The page index the canvas last said was acting.
fn acting_page(session: &Session) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|line| line.get("page").and_then(|v| v.parse::<usize>().ok())))
}

/// Enter a display mode and assert the canvas agrees it is in it.
fn enter_display(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    item: &str,
    expect: &str,
) -> Result<()> {
    invoke(session, driver, ui_rect, item)?;
    session.settle(24);
    match display_mode(session)? {
        Some(mode) if mode == expect => Ok(()),
        other => Err(Error::new(format!(
            "pressed `{item}` and the canvas says its display is {other:?}, not `{expect}`. \
             The click may have landed on the wrong control, or the command may have been \
             dropped. Reported as SKIP rather than FAIL: nothing about the operator's report \
             has been measured."
        ))),
    }
}

/// Scroll the continuous strip away from wherever it opened, and prove it
/// moved. See [`DISPLACED_PT`].
fn displace(
    session: &Session,
    driver: &Driver,
    canvas: LRect,
    report: &mut CheckReport,
) -> Result<()> {
    let before = page_rect(session)?.ok_or_else(|| {
        Error::new("the canvas never published a page rect, so there is nothing to displace.")
    })?;
    let at = session
        .frame()?
        .declared_at(canvas, SCROLL_AT.0, SCROLL_AT.1);
    driver.scroll_at(at, -SCROLL_NOTCHES)?;
    session.settle(24);
    let after = page_rect(session)?
        .ok_or_else(|| Error::new("the canvas stopped publishing a page rect after the scroll."))?;
    let moved = (before.min.y - after.min.y).abs();
    if moved < DISPLACED_PT {
        return Err(Error::new(format!(
            "the wheel moved the view {moved:.0} pt, which is less than the {DISPLACED_PT:.0} pt \
             this check needs to have established a DISPLACED state. Switching display modes \
             from an already-centred start would pass while measuring nothing, so this run is \
             SKIPPED rather than reported."
        )));
    }
    report.note(format!(
        "scrolled the continuous strip {SCROLL_NOTCHES} notches and the view moved {moved:.0} pt"
    ));
    Ok(())
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses ribbon commands, rolls the \
             wheel and presses Page Down. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    let fixture = fixture_path();
    if !fixture.exists() {
        return Err(Error::new(format!(
            "{FIXTURE} is missing from the repository, so this check has no document with a \
             two-page row. SKIPPED."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was IGNORED; this check pins {FIXTURE} because the facing half needs a \
             document with a genuine two-page row and because the cover rule makes the page \
             INDEX part of the setup"
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("page-display-recentres.trace.txt"));
    spec.pdf = Some(fixture);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    } else {
        return Err(Error::new(format!(
            "the `{}` profile declares no viewport override, so this check cannot fix the \
             window size — and every tolerance below is quoted in points of a known window. \
             SKIPPED rather than measured at whatever size the window happened to open at.",
            ctx.profile.name
        )));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(16);

    let canvas = declared(&session.trace()?, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is the document open?")))?;

    // ---- half one: switching out of a scrolled continuous view recentres ---
    //
    // Twice, once for each of the two arrangements the operator named. They
    // are separate arms of `Action::SetPageDisplay` as far as this check is
    // concerned, and a fix that recentred one and not the other is exactly the
    // sort of half-fix a single-case check would sign off.
    for (item, mode, label) in [
        (
            "ribbon.item.view.page_single",
            "single",
            "CONTINUOUS -> SINGLE",
        ),
        (
            "ribbon.item.view.page_facing",
            "facing",
            "CONTINUOUS -> FACING",
        ),
    ] {
        enter_display(
            &session,
            &driver,
            ui_rect,
            "ribbon.item.view.page_continuous",
            "continuous",
        )?;
        displace(&session, &driver, canvas, report)?;
        enter_display(&session, &driver, ui_rect, item, mode)?;

        let Some((strip, canvas_now)) = strip_and_viewport(&session, ui_rect)? else {
            return Err(Error::new(format!(
                "{label}: the canvas published no `{STRIP_REGION}` after the switch, so it drew \
                 no pages at all on its last frame. That is a real state — an empty document — \
                 and not one this fixture can be in, so it is reported as SKIP rather than as a \
                 centring failure."
            )));
        };
        report.note(format!(
            "{label}: drawn {:.0},{:.0}..{:.0},{:.0} in canvas {:.0},{:.0}..{:.0},{:.0}",
            strip.min.x,
            strip.min.y,
            strip.max.x,
            strip.max.y,
            canvas_now.min.x,
            canvas_now.min.y,
            canvas_now.max.x,
            canvas_now.max.y
        ));
        if let Some(failure) = centred(label, strip, canvas_now) {
            return Ok(Some(format!(
                "{failure} The operator asked for the view to *snap back to the centre of the \
                 canvas* when he leaves the scrolling arrangement. \
                 `Action::SetPageDisplay` currently assigns `tracked_page` so the new \
                 arrangement does not read the current page as navigated-to — which suppresses \
                 the only thing that would have moved the view, leaving the continuous strip's \
                 scroll offset in force over a layout it no longer describes."
            )));
        }
    }

    // ---- half two: Fit page under a facing spread fits BOTH pages ---------
    //
    // Still in `facing` from the loop above. The cover rule means page 0 is a
    // row by itself, so the spread has to be reached before it can be
    // measured — see the module header.
    driver.press(crate::sys::vk::PAGE_DOWN)?;
    session.settle(24);
    let page = acting_page(&session)?;
    if page != Some(1) {
        return Err(Error::new(format!(
            "Page Down left the canvas acting on page {page:?}, not page index 1. The cover \
             rule puts page 0 in a row by ITSELF, so a facing fit measured here would be \
             measuring ONE page and would pass on a build that cannot fit two. SKIPPED rather \
             than measured on the wrong row."
        )));
    }

    let Some(one_page) = page_rect(&session)? else {
        return Err(Error::new(
            "the canvas stopped publishing a page rect on the spread.".to_owned(),
        ));
    };
    invoke(&session, &driver, ui_rect, "ribbon.item.view.zoom_fit_page")?;
    session.settle(28);

    let Some((strip, canvas_now)) = strip_and_viewport(&session, ui_rect)? else {
        return Err(Error::new(format!(
            "the canvas published no `{STRIP_REGION}` after Fit page, so it drew no pages on its \
             last frame. SKIPPED."
        )));
    };
    let Some(page_now) = page_rect(&session)? else {
        return Err(Error::new(
            "the canvas stopped publishing a page rect after Fit page.".to_owned(),
        ));
    };
    report.note(format!(
        "FIT PAGE (facing): the acting page is {:.0} pt wide and what was drawn is {:.0} pt wide \
         (one page before the fit: {:.0} pt)",
        page_now.width(),
        strip.width(),
        one_page.width()
    ));

    // ★ The precondition, asserted. A row that is one page wide is not a
    // spread, and every claim below would be a claim about single-page fitting
    // — which already works and is checked elsewhere.
    if page_now.width() > 1.0 && strip.width() < page_now.width() * SPREAD_RATIO {
        return Err(Error::new(format!(
            "what the canvas drew is {:.0} pt wide and the acting page is {:.0} pt wide, so this \
             is not a two-page row — the facing mode laid out ONE page. Either the cover rule in \
             `PageDisplay::row_of` changed, or Page Down did not reach a row with two pages in \
             it. SKIPPED: a fit measured on one page says nothing about O177.",
            strip.width(),
            page_now.width()
        )));
    }

    if let Some(failure) = contained("FIT PAGE (facing)", strip, canvas_now) {
        return Ok(Some(format!(
            "{failure} This is the operator's *\"it snaps to fitting one\"*: the fit's SCALE is \
             computed from the row (`viewer::strip::fit_metrics` is facing-aware, and the zoom \
             it picks is right), but the fit's PLACEMENT is converted back through the acting \
             PAGE's rect in `canvas::offset`, so the spread is positioned as though one page \
             were the thing being centred. Half the spread lands in the middle and the other \
             half runs off the edge."
        )));
    }
    if let Some(failure) = centred("FIT PAGE (facing)", strip, canvas_now) {
        return Ok(Some(format!(
            "{failure} Fit page's promise is *contained AND centred* — equal margins either \
             side. A spread that is entirely on screen but jammed against one edge is the same \
             row/page asymmetry as the containment failure, caught one step later."
        )));
    }

    Ok(None)
}
