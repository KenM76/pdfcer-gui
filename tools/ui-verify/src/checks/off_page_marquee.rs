//! `a_band_dragged_into_the_margin_reaches_an_object_off_the_page` — **the
//! object he dropped over the edge and could not get back.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/off_page_marquee.md`.

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

/// Single-page display, then **100%** — so the aim is a statement about the
/// document rather than about whatever scroll the run inherited
/// (`marquee_table`'s own lesson), and so that the grey is wide enough to aim
/// into.
const INVOKE: &str = "mode.edit,view.page_single,view.zoom_actual";

/// The selection census.
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed

/// The band's mode and kind breakdown.
const MODE: &str = "marquee-mode"; // ui-text-exempt: a trace event name, never displayed

/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// **The scroll area the page is drawn inside**, whose grey margin is where a
/// dropped object lives.
///
/// Not the page's own rect. That is `image_rect`, and every off-page point is
/// outside it by construction — bounding against it rejects this whole check.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name

/// The fixture, relative to the workspace root.
const FIXTURE: &str = "fixtures/off-page-object.pdf";

/// Its page.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 200.0,
    height_pt: 200.0,
};

/// Blank paper in the top-right of the sheet — the band's origin.
const BAND_FROM: (f64, f64) = (160.0, 170.0);

/// **Off the left edge of the sheet**, in the grey margin.
const BAND_TO: (f64, f64) = (-100.0, 120.0);

/// See the module documentation.
pub struct ABandDraggedIntoTheMarginReachesAnObjectOffThePage;

impl Check for ABandDraggedIntoTheMarginReachesAnObjectOffThePage {
    fn name(&self) -> &'static str {
        "a_band_dragged_into_the_margin_reaches_an_object_off_the_page"
    }

    fn defect(&self) -> &'static str {
        "an object dropped over the edge of the sheet cannot be picked up with the gesture \
         anybody would reach for — a box drawn round it in the margin — so the only way back \
         is Select All and everything else comes with it"
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
            "input is disabled (--no-input). This check drags a rubber band. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
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
            "the off-page fixture is not at {}. It is 485 bytes of hand-written PDF syntax; \
             the generator is in its commit message.",
            pdf.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("off-page-marquee.trace.txt"));
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
    // The ribbon's groups collapse at the default width and a collapsed group
    // publishes no item rects — `checks::ocr`'s repair, applied here from the
    // start. Maximising also widens the grey margin this check drags into.
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
             started. Nothing about what a band reaches was measured.",
        ));
    }

    // --- aim -----------------------------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE_PAGE, 0)?;
    let frame = session.frame()?;
    let from =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(0, BAND_FROM.0, BAND_FROM.1))?);
    // The off-page conversion, named, and bounded by the canvas VIEWPORT
    // rather than by the page. See its own doc comment for why the ordinary
    // conversion refuses this point and why that refusal is right everywhere
    // else — and for why bounding it against `image_rect` would reject the
    // whole class it exists for.
    let viewport = declared(&trace, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{VIEWPORT_REGION}` region, so this check has \
             no bound to convert an off-page point against. It cannot fall back to the \
             page's own rect: every off-page point is outside that by construction."
        ))
    })?;
    let to = frame.to_screen(
        mapping.doc_to_window_off_page(DocPoint::new(0, BAND_TO.0, BAND_TO.1), viewport)?,
    );
    report.note(format!(
        "band ({:.0}, {:.0}) → ({:.0}, {:.0}) in page points — the second corner is {:.0} pt \
         LEFT of the sheet, in the grey margin, and the drag is right-to-left so it is a \
         crossing window",
        BAND_FROM.0, BAND_FROM.1, BAND_TO.0, BAND_TO.1, -BAND_TO.0
    ));

    driver.drag(from, to)?;
    session.settle(40);

    // --- read ----------------------------------------------------------------
    let trace = session.trace()?;
    let Some(mode) = trace.events(MODE).last() else {
        return Ok(Some(format!(
            "★★★ NO RUBBER BAND EVER BEGAN: no `{MODE}` line followed the drag.\n\n\
             ★★ Look for `selection-set … via=press` in the trace before reading this as a hit \
             test that excluded everything. That means the origin was on ink and the press \
             selected instead of banding — `canvas::presspick`'s rule is *\"a press on empty \
             paper still marquees\"*, and the pick tolerance is several page points wide. This \
             check's origin is 92 pt from the nearest mark, so if that is what happened the \
             fixture has changed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the band ran: `{}`", mode.raw));

    if mode.get("crossing") != Some("true") {
        return Ok(Some(format!(
            "A RIGHT-TO-LEFT DRAG WAS NOT READ AS A CROSSING WINDOW: `{}`. The band went from \
             x={:.0} to x={:.0}, which is leftward. See `Drag::outcome`.",
            mode.raw, BAND_FROM.0, BAND_TO.0
        )));
    }

    let hits = mode.get_usize("hits").unwrap_or_default();
    if hits == 0 {
        return Ok(Some(format!(
            "★★★ THE BAND REACHED INTO THE MARGIN AND FOUND NOTHING: `{}`.\n\n\
             This is `OPERATOR_REQUESTS.md` O92 unfixed — *\"I sometimes drop objects there, \
             and when I do I can't get them back\"*. The band covers x −100…160, y 120…170, \
             and the fixture's off-page square occupies x −160…−40, y 100…140, so it overlaps \
             by 60 × 20 pt. Zero hits means either the band's rect never left the page box, or \
             `hit_test_rect` is refusing bounds outside it.\n\n\
             ★ Check the marquee's canvas→PDF conversion first: `Select All` had exactly this \
             shape of failure when `Rect::EVERYTHING`'s infinities became NaN through that \
             transform, and every comparison against NaN is false — a rect meaning *all of it* \
             became arithmetically identical to one meaning *none*. Trace: {}.",
            mode.raw,
            session.trace_path().display()
        )));
    }
    if hits > 1 {
        return Ok(Some(format!(
            "★★ THE BAND TOOK MORE THAN THE OFF-PAGE SQUARE: {hits} hits from `{}`.\n\n\
             This fixture has exactly two objects and the band is aimed to miss the on-page \
             one — its top edge is y=100 and the band's bottom is y=120. More than one hit \
             means the band covered more than it was aimed at, so the count no longer proves \
             anything about reaching off the page and this check's whole argument is void. \
             Re-derive the geometry before touching the feature.",
            mode.raw
        )));
    }

    let selection = trace
        .events(SELECTION)
        .filter(|l| l.get("via") == Some("pv.marquee"))
        .last();
    report.note(format!(
        "★★★ exactly one object — and on this fixture the only thing inside that band is the \
         square that lies ENTIRELY off the left edge of the sheet, so the marquee reached into \
         the margin{}",
        selection.map_or_else(String::new, |l| format!(": `{}`", l.raw))
    ));
    report.note(
        "★ O92's other half was Select All, which already reached these objects. What this \
         adds is the gesture the operator would actually reach for, and it works because a \
         right-to-left band takes what it TOUCHES — an enclosing band over this same rect \
         surrounds nothing and returns zero",
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The band must touch the off-page square without enclosing it.**
    #[test]
    fn the_band_touches_the_off_page_square_but_cannot_enclose_it() {
        // The fixture's off-page square, from its own content stream.
        let (sq_left, sq_right) = (-160.0_f64, -40.0_f64);
        let (sq_bottom, sq_top) = (100.0_f64, 140.0_f64);
        let (band_left, band_right) = (BAND_TO.0.min(BAND_FROM.0), BAND_TO.0.max(BAND_FROM.0));
        let (band_bottom, band_top) = (BAND_TO.1.min(BAND_FROM.1), BAND_TO.1.max(BAND_FROM.1));

        assert!(
            band_left < sq_right && band_right > sq_left,
            "the band must overlap the square horizontally"
        );
        assert!(
            band_bottom < sq_top && band_top > sq_bottom,
            "the band must overlap the square vertically"
        );
        assert!(
            band_left > sq_left,
            "the band must NOT reach the square's left edge, or an enclosing band would \
             satisfy this check and it would stop discriminating between the two modes"
        );
    }

    /// **The band must miss the on-page square**, or `hits == 1` proves nothing.
    #[test]
    fn the_band_misses_the_on_page_square() {
        // A: x 40–100, y 40–100.
        let a_top = 100.0_f64;
        let band_bottom = BAND_TO.1.min(BAND_FROM.1);
        assert!(
            band_bottom > a_top,
            "the band's lower edge ({band_bottom}) must sit above the on-page square's top \
             ({a_top}), or a single hit could be either object and the count stops being an \
             oracle"
        );
    }

    /// The origin is on the page and the destination is off it.
    #[test]
    fn the_origin_is_on_the_page_and_the_destination_is_not() {
        assert!(BAND_FROM.0 >= 0.0 && BAND_FROM.0 <= FIXTURE_PAGE.width_pt);
        assert!(BAND_FROM.1 >= 0.0 && BAND_FROM.1 <= FIXTURE_PAGE.height_pt);
        assert!(
            BAND_TO.0 < 0.0,
            "the destination must be outside the media box — that is the whole subject"
        );
    }

    /// The drag is leftward, so it is a crossing window.
    #[test]
    fn the_drag_is_right_to_left() {
        assert!(BAND_TO.0 < BAND_FROM.0);
    }
}
