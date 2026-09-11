//! `a_band_that_starts_in_the_margin_reaches_an_object_off_the_page` — **the
//! half of O23 that had nothing to enable.**
//!
//! # The report
//!
//! Ken, 2026-08-21, `OPERATOR_REQUESTS.md` **O23**:
//!
//! > *"also objects should still be reachable even if they are off the page."*
//!
//! and again on 2026-09-10, three weeks after part A shipped:
//!
//! > *"how do I view and edit objects that are off of the page? we added this
//! > feature but I didn't see how to enable it."*
//!
//! There was nothing to enable. **Part A** shipped on the day: `content_extent`
//! adds a full viewport of scrollable slack on every side of the strip, so the
//! grey the operator has to scroll into in order to *see* where an off-page
//! object is exists. **Part B did not**: that grey was allocated with
//! `Sense::hover()`, so a press out there never became a gesture at all.
//!
//! # ★★★ How this differs from `off_page_marquee`, and why that is not a
//! duplicate
//!
//! `off_page_marquee` is the sibling check for O92, and it drags a band **from
//! blank paper** into the margin. That gesture already worked, and it worked
//! for a reason that looks like this one and is not: an `egui::Response` keeps
//! reporting `interact_pointer_pos` after the pointer has left the widget, so a
//! drag that BEGINS on the sheet stays with the sheet for its whole life no
//! matter how far out it goes.
//!
//! This check **begins the press in the grey**, and nothing carries that. Before
//! 2026-09-10 the sequence produced no `marquee-mode` line at all — not a band
//! that found nothing, *no band* — because the only rectangle under the pointer
//! sensed hover. The two checks therefore fail on disjoint causes, and a single
//! check covering both would have been green throughout the three weeks the
//! operator could not find the feature.
//!
//! ⇒ The discriminating assertion here is **`canvas-surface surface=pasteboard
//! … pastegesture=true`, followed by a band**. Either alone is weak: the trace
//! line without a band would say the decision ran and the gesture still died,
//! and a band without the line would not say WHICH surface produced it.
//!
//! ## ⚠ …and `surface=pasteboard` alone is not the assertion, for a measured
//! reason
//!
//! `canvas::trace::surface` is emitted from the pointer's **position**, every
//! frame, so `surface=pasteboard` appears whenever the pointer is off the sheet
//! — *including on a build whose scroll content senses nothing but hover*. The
//! first draft of this check asserted on exactly that, and when it was run
//! against a deliberately falsified build (the `Sense` put back to `hover()`,
//! 2026-09-10) it went red with a message beginning **"the `Sense` is right"**.
//! It was not right; it was the one thing that had been broken.
//!
//! `pastegesture=true` is the field that cannot exist without a drag sense, so
//! it is the field this check turns on. The general lesson, recorded because it
//! has bitten this suite before: **a falsification is a measurement of the
//! check as well as of the feature.** A check that goes red for the right
//! reason while naming the wrong cause is a check that will send the next
//! reader to the wrong file.
//!
//! # The fixture, and why `hits == 1` is an oracle
//!
//! `fixtures/off-page-object.pdf` — the same 485 bytes `off_page_marquee` uses,
//! deliberately, because two fixtures for one property is two chances for one of
//! them to stop having it. A 200 × 200 page with exactly two filled squares:
//!
//! | | where | on the page? |
//! |---|---|---|
//! | **A** | x 40–100, y 40–100 | yes |
//! | **B** | x −160 – −40, y 100–140 | **no — entirely left of the media box** |
//!
//! The band runs from `(−20, 190)` — **grey, above and left of everything** —
//! leftward and downward to `(−100, 120)`, covering x −100…−20, y 120…190.
//!
//! * It **misses A twice over**: A's right edge is x = 40 and the band's right
//!   is x = −20; A's top is y = 100 and the band's bottom is y = 120. Either
//!   miss alone would do; both is deliberate, so a change to one axis of the
//!   fixture cannot quietly make the count ambiguous.
//! * It **touches B** — x −100…−40 and y 120…140 are inside both.
//! * It **cannot enclose B**, which reaches x = −160 where the band stops at
//!   x = −100. So this is still a *crossing* window and the check still
//!   discriminates between the two marquee modes rather than passing under
//!   either.
//!
//! ⇒ **One hit can only be B.** The unit tests at the foot of this file assert
//! every clause of that paragraph against the fixture's own coordinates, so the
//! argument fails at build time if the geometry is ever edited out from under
//! it.
//!
//! # ★★ Why the origin is at y = 190 rather than beside the square
//!
//! `canvas::presspick`'s rule is that a press on *ink* starts a move and a press
//! on empty paper starts a band. The pick tolerance is several page points wide
//! and is measured in screen pixels, so it grows in page terms as the zoom
//! falls. `(−20, 190)` is 36 pt from B's nearest corner at `(−40, 140)` — far
//! enough at any zoom this check can be driven at, and the alternative failure
//! is silent: the press would select B directly, no band would run, and the
//! check would report a marquee defect that does not exist. `off_page_marquee`
//! learned this at 92 pt; this one has less room and states the margin.
//!
//! # What this check does NOT claim
//!
//! **It does not claim the operator can SEE the object.** The raster is still
//! sized to the crop box, so square B is not painted — what this measures is
//! that it can be *reached*: banded, selected, and therefore outlined (the
//! selection overlay is clipped to the canvas viewport, not to the page) and
//! dragged home. Making it visible is the render half of part B and will have
//! its own check. Conflating the two would produce one check that cannot say
//! which half broke.
//!
//! # Every way this reports SKIP
//!
//! No binary, `--no-input`, no diagnostic channel, no page on screen, the Select
//! tool unreachable from the ribbon, or **not enough grey margin on screen to
//! reach x = −100** — a property of the window size on the day, reported as a
//! skip that names the geometry and never as a pass.

use crate::checks::driving::{
    SHELL_DIAG_ENV, arm_select_from_ribbon, click_mode_segment, declared, declared_names, list,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Content selection needs Edit.
const RIBBON_MODE: &str = "edit";

/// Single-page display, then **100%** — not fit-page.
///
/// ★★★ **The zoom is load-bearing and fit-page is wrong here.** This check
/// aims at x = -100 pt, which is 100 pt of grey to the left of the sheet, and
/// what matters is how many SCREEN pixels that is. Fit-page on a 200 x 200
/// fixture in a maximised window puts the sheet at roughly 3.8 px per point, so
/// the aim lands 381 px left of the page edge where only ~243 px of viewport
/// exists -- `doc_to_window_off_page` refuses, correctly, and the check SKIPS.
///
/// Measured, 2026-09-10: the sibling `off_page_marquee` has been skipping for
/// exactly this reason, silently, because a SKIP is not red. At 100% the sheet
/// is ~200 px wide in a 1250 px viewport and every point this check aims at is
/// comfortably inside it.
///
/// ★★ It also fixes the aim in a way fit-page cannot: 100% is a property of
/// the DOCUMENT, so the geometry this check depends on no longer varies with
/// the window size on the day.
const INVOKE: &str = "view.page_single,view.zoom_actual";

/// The band's mode and kind breakdown.
const MODE: &str = "marquee-mode"; // ui-text-exempt: a trace event name, never displayed

/// ★★★ **Which of the canvas's two interactive rectangles owned the frame** —
/// the direct evidence that the mechanism under test ran, as opposed to the
/// band having arrived by some other route.
const SURFACE: &str = "canvas-surface"; // ui-text-exempt: a trace event name, never displayed

/// The word that means the press landed on the pasteboard.
const PASTEBOARD: &str = "pasteboard"; // ui-text-exempt: a trace field value, never displayed

/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// **The scroll area the page is drawn inside**, whose grey margin is the
/// subject. Not the page's own rect: every point this check aims at is outside
/// that by construction.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name

/// The fixture, relative to the workspace root. Shared with `off_page_marquee`.
const FIXTURE: &str = "fixtures/off-page-object.pdf";

/// Its page.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 200.0,
    height_pt: 200.0,
};

/// **The band's origin — in the grey, and this is the whole subject.**
///
/// Left of the media box and above every mark in the file. See the module
/// header for why it is 36 pt clear of the off-page square rather than beside
/// it.
const BAND_FROM: (f64, f64) = (-20.0, 190.0);

/// Further into the grey and down, so the drag is right-to-left and the band is
/// a crossing window. x = −100 rather than −200 for the same reason as in
/// `off_page_marquee`: it must touch the square without being able to enclose
/// it.
const BAND_TO: (f64, f64) = (-100.0, 120.0);

/// The fixture's off-page square `(left, bottom, right, top)`, transcribed from
/// its own content stream — the oracle the unit tests below measure the band
/// against.
///
/// ★★ `#[cfg(test)]` because the DRIVEN half must not read it. The check's
/// oracle at run time is `hits == 1`, and it is airtight only because the
/// geometry was argued in advance; a run-time comparison against these numbers
/// would be the harness agreeing with itself. They exist so that an edit to the
/// fixture fails the build instead of quietly making the count ambiguous.
#[cfg(test)]
const SQUARE_B: (f64, f64, f64, f64) = (-160.0, 100.0, -40.0, 140.0);

/// The fixture's on-page square, likewise.
#[cfg(test)]
const SQUARE_A: (f64, f64, f64, f64) = (40.0, 40.0, 100.0, 100.0);

/// See the module documentation.
pub struct ABandThatStartsInTheMarginReachesAnObjectOffThePage;

impl Check for ABandThatStartsInTheMarginReachesAnObjectOffThePage {
    fn name(&self) -> &'static str {
        "a_band_that_starts_in_the_margin_reaches_an_object_off_the_page"
    }

    fn defect(&self) -> &'static str {
        "a press that begins in the grey beside the sheet is not a gesture at all, so the space \
         O23 added to scroll into cannot be used to reach anything — the operator scrolls out \
         to where a dropped object is and finds the canvas inert"
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

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses in the grey margin and drags. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so this check cannot reach the \
             mode selector or the Select tool.",
            ctx.profile.name
        ))
    })?;
    let pdf = workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the off-page fixture is not at {}. It is 485 bytes of hand-written PDF syntax, \
             shared with `off_page_marquee`; the generator is in its commit message.",
            pdf.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("off-page-press.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // Maximising widens the grey this check presses into, and a collapsed
    // ribbon group publishes no item rects — `checks::ocr`'s repair.
    session.maximize();
    session.settle(16);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, RIBBON_MODE)?;
    session.settle(12);

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }
    if !arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        return Err(Error::new(
            "the select tool could not be armed from the ribbon, so no rubber band could be \
             started. Nothing about pressing in the margin was measured.",
        ));
    }

    // --- aim -----------------------------------------------------------------
    //
    // ★ BOTH corners go through the off-page conversion here, where
    // `off_page_marquee` sends only its destination that way. That asymmetry is
    // the difference between the two checks stated in arithmetic: this gesture
    // has no point on the paper at all.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE_PAGE, 0)?;
    let frame = session.frame()?;
    let viewport = declared(&trace, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{VIEWPORT_REGION}` region, so this check has no bound \
             to convert an off-page point against. It cannot fall back to the page's own rect: \
             every point this check aims at is outside that by construction."
        ))
    })?;
    let from = frame.to_screen(
        mapping.doc_to_window_off_page(DocPoint::new(0, BAND_FROM.0, BAND_FROM.1), viewport)?,
    );
    let to = frame.to_screen(
        mapping.doc_to_window_off_page(DocPoint::new(0, BAND_TO.0, BAND_TO.1), viewport)?,
    );
    report.note(format!(
        "band ({:.0}, {:.0}) → ({:.0}, {:.0}) in page points — NEITHER corner is on the sheet, \
         and the press begins {:.0} pt left of its edge",
        BAND_FROM.0, BAND_FROM.1, BAND_TO.0, BAND_TO.1, -BAND_FROM.0
    ));

    driver.drag(from, to)?;
    session.settle(40);

    // --- read ----------------------------------------------------------------
    let trace = session.trace()?;

    // ★★★ First: **did the pasteboard ever hold a GESTURE?** — not merely
    // "was the pasteboard chosen".
    //
    // ⚠ MEASURED, 2026-09-10, and this check got it wrong on its first draft.
    // `canvas::trace::surface` is emitted every frame from the pointer's
    // POSITION, so `surface=pasteboard` appears whenever the pointer is off the
    // sheet — **including on a build whose scroll content senses nothing but
    // hover.** Asserting on that value alone produced, against a deliberately
    // falsified build, a failure message reading *"the `Sense` is right"* when
    // the `Sense` was the one thing that had been broken. A confident wrong
    // attribution sends the next reader to the wrong file, which is worse than
    // reporting nothing.
    //
    // `pastegesture=true` is the field a hover-only rectangle cannot produce,
    // because `Response::dragged()` is false forever without a drag sense. It
    // is therefore the field that separates the two repairs, and it is the one
    // asserted on.
    let surfaces: Vec<String> = trace
        .events(SURFACE)
        .filter_map(|l| l.get("surface").map(str::to_owned))
        .collect();
    let grasped = trace
        .events(SURFACE)
        .any(|l| l.get("surface") == Some(PASTEBOARD) && l.get("pastegesture") == Some("true"));
    if !grasped {
        return Ok(Some(format!(
            "★★★ THE GREY NEVER HELD THE GESTURE: no `{SURFACE}` line reports \
             `surface={PASTEBOARD} … pastegesture=true`. Surfaces decided this run: {}.\n\n\
             ★★ Suspect the `Sense` on the scroll content FIRST — and note what this check is \
             NOT saying. `surface={PASTEBOARD}` on its own means only that the pointer was off \
             the sheet, and it is emitted on a hover-only build too; that is why it is not the \
             assertion. `canvas::present::show` allocates that rectangle once, before any page, \
             and it read `Sense::hover()` from the day O23's pasteboard shipped until \
             2026-09-10. With hover, `Response::dragged()` is false forever and this field can \
             never be true.\n\n\
             ★ If it already reads `click_and_drag`, the next suspect is allocation ORDER: the \
             content must be allocated BEFORE the pages, because egui gives an overlap to the \
             widget registered later. Reversed, the pages would swallow every press — including \
             the ones out here. Trace: {}.",
            list(&surfaces),
            session.trace_path().display()
        )));
    }

    let Some(mode) = trace.events(MODE).last() else {
        return Ok(Some(format!(
            "★★★ THE GREY HELD THE GESTURE AND NO BAND EVER BEGAN: `{SURFACE}` reported \
             `surface={PASTEBOARD} … pastegesture=true`, so the scroll content DID sense the \
             drag, and no `{MODE}` line followed it.\n\n\
             The `Sense` is therefore right — this is the one branch of this check entitled to \
             say so — and the gesture is being lost after it. Two candidates, in order: the \
             response handed to `canvas::interact` is still the acting PAGE's, since a frame \
             that decides `Pasteboard` and then passes `&image_response` produces exactly this \
             trace; or `canvas::presspick` read the press as landing on ink and started a move \
             instead of a band. Look for `selection-set … via=press` in the trace to tell those \
             apart. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the band ran: `{}`", mode.raw));

    if mode.get("crossing") != Some("true") {
        return Ok(Some(format!(
            "A RIGHT-TO-LEFT DRAG WAS NOT READ AS A CROSSING WINDOW: `{}`. The band went from \
             x={:.0} to x={:.0}, which is leftward. See `Drag::outcome`. This matters here \
             rather than being cosmetic: an enclosing band over this rect surrounds nothing, so \
             the hit count below would be zero for a reason that has nothing to do with the \
             pasteboard.",
            mode.raw, BAND_FROM.0, BAND_TO.0
        )));
    }

    let hits = mode.get_usize("hits").unwrap_or_default();
    if hits == 0 {
        return Ok(Some(format!(
            "★★ A BAND BEGAN IN THE GREY AND FOUND NOTHING: `{}`.\n\n\
             The pasteboard took the press and the band ran, so O23's part B is live at the \
             input surface and the failure is in the hit test. The band covers x −100…−20, \
             y 120…190 and the fixture's off-page square occupies x −160…−40, y 100…140, so \
             they overlap by 60 × 20 pt.\n\n\
             ★ Check the marquee's canvas→PDF conversion first, and check whether the rect is \
             being clamped to the media box on the way: every coordinate in this band is \
             NEGATIVE in x, and a clamp to zero collapses it to a zero-width rect that \
             intersects nothing while still looking like a rect. Trace: {}.",
            mode.raw,
            session.trace_path().display()
        )));
    }
    if hits > 1 {
        return Ok(Some(format!(
            "★★ THE BAND TOOK MORE THAN THE OFF-PAGE SQUARE: {hits} hits from `{}`.\n\n\
             This fixture has exactly two objects and the band is aimed to miss the on-page one \
             on BOTH axes — it ends at x=−20 where that square starts at x=40, and its lower \
             edge is y=120 where that square's top is y=100. More than one hit means the band \
             covered more than it was aimed at, so the count no longer proves anything about \
             reaching off the page and this check's whole argument is void. Re-derive the \
             geometry before touching the feature.",
            mode.raw
        )));
    }

    report.note(
        "★★★ a press that began in the grey beside the sheet became a rubber band, and that \
         band took exactly one object — which on this fixture can only be the square lying \
         ENTIRELY off the left edge. That is O23's second half at the input surface: the space \
         part A gave the operator to scroll into is now a space they can reach into",
    );
    report.note(format!(
        "★ surfaces decided this run, in order: {}, and at least one of them carried \
         `pastegesture=true` — the field a hover-only rectangle cannot produce. The trace is \
         de-duplicated per slot, so this is the sequence of CHANGES and not one entry per frame",
        list(&surfaces)
    ));

    let shot = ctx.out("off-page-press.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{BAND_FROM, BAND_TO, FIXTURE_PAGE, SQUARE_A, SQUARE_B};

    /// The band's rect, normalised.
    fn band() -> (f64, f64, f64, f64) {
        (
            BAND_FROM.0.min(BAND_TO.0),
            BAND_FROM.1.min(BAND_TO.1),
            BAND_FROM.0.max(BAND_TO.0),
            BAND_FROM.1.max(BAND_TO.1),
        )
    }

    /// ★★★ **The whole subject in one assertion: the press is not on the
    /// sheet.**
    ///
    /// This is the ONLY thing that distinguishes this check from
    /// `off_page_marquee`, whose origin is deliberately on blank paper. If this
    /// ever became true the two checks would test the same thing, one of them
    /// would be deleted as a duplicate, and the cause that hid for three weeks
    /// would be uncovered again.
    #[test]
    fn the_press_does_not_begin_on_the_page() {
        assert!(
            BAND_FROM.0 < 0.0,
            "the band's ORIGIN must be outside the media box — a band that starts on paper is \
             carried by the page's own response no matter where it goes, which is the sibling \
             check and not this one"
        );
        assert!(
            BAND_TO.0 < 0.0,
            "and so must its destination: this gesture has no point on the sheet at all"
        );
        let _ = FIXTURE_PAGE;
    }

    /// ★★★ **The band must touch the off-page square without enclosing it**, or
    /// the check stops discriminating between the two marquee modes and would
    /// be green under the pre-O88 behaviour it is written to exclude.
    #[test]
    fn the_band_touches_the_off_page_square_but_cannot_enclose_it() {
        let (bl, bb, br, bt) = band();
        let (sl, sb, sr, st) = SQUARE_B;
        assert!(bl < sr && br > sl, "the band must overlap the square in x");
        assert!(bb < st && bt > sb, "the band must overlap the square in y");
        assert!(
            bl > sl,
            "the band must NOT reach the square's left edge, or an ENCLOSING band would satisfy \
             this check too"
        );
    }

    /// ★★ **The band must miss the on-page square**, or `hits == 1` proves
    /// nothing about where the objects are.
    ///
    /// Asserted on both axes independently, because the check's failure message
    /// claims both and a message that claims more than the test holds is how a
    /// reader is sent to the wrong place.
    #[test]
    fn the_band_misses_the_on_page_square_on_both_axes() {
        let (_, bb, br, _) = band();
        let (al, _, _, at) = SQUARE_A;
        assert!(
            br < al,
            "the band's right edge ({br}) must sit left of the on-page square's left ({al})"
        );
        assert!(
            bb > at,
            "and its lower edge ({bb}) must sit above that square's top ({at})"
        );
    }

    /// The drag is right-to-left, so it is a crossing window.
    ///
    /// Pinned separately from the enclosure argument above: they are two
    /// reasons for the same coordinate and a future edit is likely to satisfy
    /// one while breaking the other.
    #[test]
    fn the_drag_is_right_to_left() {
        assert!(BAND_TO.0 < BAND_FROM.0);
    }

    /// ★★ **The origin must clear the off-page square by more than the pick
    /// tolerance**, or the press selects that square directly, no band runs,
    /// and the check reports a marquee defect that does not exist.
    ///
    /// 20 pt is the floor asserted here rather than the 36 pt the current
    /// coordinates give, so the test states a requirement rather than
    /// restating the constant.
    #[test]
    fn the_origin_clears_the_off_page_square() {
        let (sl, sb, sr, st) = SQUARE_B;
        let dx = if BAND_FROM.0 < sl {
            sl - BAND_FROM.0
        } else if BAND_FROM.0 > sr {
            BAND_FROM.0 - sr
        } else {
            0.0
        };
        let dy = if BAND_FROM.1 < sb {
            sb - BAND_FROM.1
        } else if BAND_FROM.1 > st {
            BAND_FROM.1 - st
        } else {
            0.0
        };
        let gap = dx.hypot(dy);
        assert!(
            gap >= 20.0,
            "the press origin is {gap:.1} pt from the off-page square, which is inside the \
             range `canvas::presspick`'s tolerance can reach at a low zoom — the press would \
             select it instead of starting a band"
        );
    }
}
