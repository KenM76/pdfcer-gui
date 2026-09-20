//! **The page can be moved on the paper, and the hatch follows it to all four
//! edges** — operator request O208, driven against the real binary.
//!
//! The request, verbatim:
//!
//! > can we add a control to our print preview screen so that when we are
//! > printing at a scale that will lose content we have the option to drag the
//! > drawing to a new position on the print page? That way we can choose what
//! > gets cropped. we could also have an option to reset position, center,
//! > center horizontally, or center vertically. This can be remembered for
//! > each page. Also the hash lines we use to show what won't be printed
//! > should have a line for each edge of the page.
//!
//! # Why this check has to exist, and why the unit tests are not enough
//!
//! `dialogs::print::position` has fifteen unit tests and they cover the
//! arithmetic completely: centre is not reset, centring is idempotent, a
//! fitting page dragged off the near edge clips, the engine still starts an
//! oversized page flush at the corner. Every one of them calls the verb
//! directly.
//!
//! None of them can see the chain in FRONT of the verb, which is where this
//! feature can fail in at least five ways that all leave the tests green:
//!
//! * the drag is classified as a **pan** — same mouse button, same rectangle,
//!   and the only thing separating the two gestures is where the press landed;
//! * the displacement is applied to the placement after somebody has already
//!   read it, so the preview moves and the print does not, or the reverse;
//! * the delta is not divided by the preview scale, so the page crawls;
//! * the sign is flipped on one axis, which looks like a working feature until
//!   the operator tries to recover content off the bottom of the sheet;
//! * the position is keyed on the plan position rather than the document page
//!   index, so it is remembered against the wrong page as soon as the job is a
//!   custom range or an odd/even subset.
//!
//! # What it asserts
//!
//! | # | Gesture | Property |
//! |---|---|---|
//! | 1 | open the dialog, choose **Actual size** | the page starts unmoved: `pos=0.00,0.00 moved=0` |
//! | 2 | drag inside the page, up and to the left | `grab=page` on some frame, `pos=` negative on both axes, and larger in paper points than the pointer moved in screen points |
//! | 3 | — | `edges=` gains BOTH near edges: `l` and `t` are now set |
//! | 4 | **Centre** | `pos=` changed and `moved=1` |
//! | 5 | **Centre horizontally**, then **Centre vertically** | `pos=` does not move — centring twice lands in the same place |
//! | 6 | **Reset all pages** | `pos=0.00,0.00 moved=0` |
//! | 7 | **Centre**, then **Reset position** | back to zero again, by the per-page route |
//!
//! Assertion 3 is the second clause of the request and it is the one a
//! screenshot cannot settle: three hatched bands and two hatched bands are the
//! same picture to anything that is not a human looking for the difference.
//! `edges=` is a four-character word in the band order left, right, top,
//! bottom — `.r.b` for a page flush at the corner, `lrtb` for one dragged off
//! all four — so the check reads *which* edges rather than how many.
//!
//! # What it deliberately does NOT do
//!
//! **It never presses the commit button**, and no future edit may make it do
//! so. Same rule and same reason as `print_dialog`, `print_paper` and
//! `print_clip_claim`: that button is the one control in the application that
//! consumes paper and cannot be undone, and a harness able to start a print
//! job will eventually start one by accident.
//!
//! It also never asserts the per-edge crop sentence. That disclosure is a
//! label, a label's text is not in the trace, and `position::cropped`'s unit
//! tests own the numbers in it.
//!
//! # The fixture
//!
//! A sheet bigger than the machine's printable area, so that **Actual size**
//! crops and the four-edge half has something to be about:
//! `fixtures/a1-titleblock.pdf`. Assertions 1, 2 and 4–7 hold on any fixture —
//! a page that fits can be dragged off the paper too — and assertion 3 states
//! which edges the page hangs over after being dragged up and left, which is
//! the near two whatever the sheet's size.

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, list_str,
    shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::trace::Trace;

/// The ribbon control that opens the dialog, and the tab it lives on.
const SUBJECT: &str = "ribbon.item.file.print";
const TAB_ID: &str = "file";
const TAB: &str = "ribbon.tab.file";

/// The trace event the dialog emits once, when it is built.
const OPEN_EVENT: &str = "print-open";

/// The per-frame line carrying `pos=`, `grab=`, `moved=`, `edges=` and
/// `scale=`.
const PREVIEW_EVENT: &str = "print-preview";

/// The scale radio that makes the page crop. The dialog opens on **Fit**,
/// which scales a page down to the printable area and therefore loses nothing,
/// so every claim this check makes is unreachable from the opening state.
const SCALE_ACTUAL: &str = "print.scale.actual";

/// The Position tab's own button — both the control that opens the tab and the
/// thing the wheel is rolled over to bring the group below it into view.
///
/// A button, deliberately: it does not consume a wheel notch, so the notch
/// reaches the scrolling body behind it. A `DragValue` — and the Position group
/// has two — would have eaten the notch and changed the number it was sitting
/// on, which is a silent edit to the very geometry being measured.
///
/// It is also the only anchor that can work. The scale radios, which this
/// check used to scroll over, are on a different tab: once Position is open
/// they are not drawn at all, so an anchor there would be absent exactly when
/// it was needed and the check would read "the group drew nothing".
const TAB_POSITION: &str = "print.tab.position";

/// The page's grabbable rectangle inside the preview canvas.
const PAGE: &str = "print.preview.page";

/// The Position group's five buttons.
const RESET: &str = "print.position.reset";
const CENTRE: &str = "print.position.centre";
const CENTRE_H: &str = "print.position.centre-h";
const CENTRE_V: &str = "print.position.centre-v";
const RESET_ALL: &str = "print.position.reset-all";

/// How far the pointer is dragged, in logical points, on each axis.
///
/// Bounded above by the preview canvas, which a measured run reported as
/// **340x438 logical points** on this machine's default window: the gesture has
/// to start and finish inside the page, the page at Actual size fills the
/// canvas, and so the canvas's short side is the ceiling. 240 pt was tried
/// first and skipped every run for want of room.
///
/// Bounded below by [`EGUI_MAX_CLICK_DIST`]: the driver walks the pointer in
/// [`DRAG_STEPS`] equal steps, and the prediction below is only the truth while
/// one of those steps is longer than that distance. `travel_per_step_clears_egui`
/// holds the two numbers against each other.
const DRAG_PT: f32 = 120.0;

/// egui's own click-versus-drag distance, logical points:
/// `InputOptions::max_click_dist`, whose default is 6.0
/// (`egui-0.35.0/src/input_state/mod.rs:115`).
///
/// A press becomes a drag on the first frame whose travel from the press origin
/// exceeds this, and that frame's `drag_delta()` carries its entire step. So a
/// driver whose step is longer than this loses nothing to the decision, and one
/// whose step is shorter loses every step before the crossing. The prediction
/// below is the whole travel, which assumes the first case — hence the test.
const EGUI_MAX_CLICK_DIST: f32 = 6.0;

/// How much of the predicted movement must actually arrive.
///
/// The prediction is the whole pointer travel, so the floor is for steps the
/// application misses under load — two of the eight — and the ceiling is for
/// rounding. Deliberately too narrow to be satisfied by a delta applied twice
/// (a ratio near 2) or by one never divided by the preview scale (a ratio near
/// the scale itself, about 0.4). The sharp assertion about magnitude is not this
/// band — it is [`assert_magnified`].
const MOVE_BAND: (f64, f64) = (0.70, 1.10);

/// Above this preview scale, dividing the screen delta by it no longer
/// magnifies, so [`assert_magnified`] has nothing to say and says so.
const MAGNIFY_SCALE_CEILING: f32 = 0.7;

/// Wheel notches per scroll attempt, and how many attempts, when the Position
/// group is below the fold. Negative is downward in this harness.
const SCROLL_NOTCHES: i32 = -3;
const SCROLL_TRIES: usize = 8;

/// How much bigger than the drag the grabbable page must be for the gesture to
/// start and finish inside it.
///
/// The drag runs from the page's centre outward, so the requirement is that
/// half the smaller side exceeds the travel — a factor of two — plus a margin
/// for the pointer landing a pixel off.
const ROOM_FOR_THE_DRAG: f32 = 2.2;

pub struct ThePrintedPageCanBeMovedOnThePaper;

impl Check for ThePrintedPageCanBeMovedOnThePaper {
    fn name(&self) -> &'static str {
        "the_printed_page_can_be_moved_on_the_paper"
    }

    fn defect(&self) -> &'static str {
        "the page cannot be positioned on the paper, so what gets cropped at a scale that loses content is not the operator's choice"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// One reading of the preview's own account of where the page is.
#[derive(Clone, Debug)]
struct Position {
    /// The operator's displacement, paper points, right and down positive.
    dx_pt: f64,
    dy_pt: f64,
    /// How many pages in the job carry a displacement.
    moved: usize,
    /// Which edges the page hangs over, in the band order left, right, top,
    /// bottom. Kept as the raw word so a failure message can quote it.
    edges: String,
    /// Screen points per paper point, this frame.
    scale: f32,
}

impl Position {
    fn over(&self, letter: char) -> bool {
        self.edges.contains(letter)
    }

    /// Whether two readings are the same place, to the two decimals the trace
    /// prints.
    fn same_place_as(&self, other: &Self) -> bool {
        (self.dx_pt - other.dx_pt).abs() < 0.005 && (self.dy_pt - other.dy_pt).abs() < 0.005
    }

    fn at_the_origin(&self) -> bool {
        self.moved == 0 && self.dx_pt == 0.0 && self.dy_pt == 0.0
    }

    fn where_it_is(&self) -> String {
        format!(
            "pos={:.2},{:.2} moved={} edges={}",
            self.dx_pt, self.dy_pt, self.moved, self.edges
        )
    }
}

/// Read the last `print-preview` line as a [`Position`].
fn position(trace: &Trace) -> Result<Position> {
    let line = trace.last(PREVIEW_EVENT).ok_or_else(|| {
        Error::new(format!(
            "the dialog is open and emitted no `{PREVIEW_EVENT}` line, so the preview column \
             drew nothing. `print_layout` is where that is diagnosed, not here."
        ))
    })?;
    let pos = line.get_f64_list("pos").ok_or_else(|| {
        Error::new(format!(
            "`{PREVIEW_EVENT}` carries no readable `pos=` field. Fields present: {}. Without it \
             there is no headless evidence of the page's position at all — the picture moves and \
             nothing outside the process can tell.",
            list_str(&line.field_names())
        ))
    })?;
    if pos.len() != 2 {
        return Err(Error::new(format!(
            "`pos=` carried {} value(s), not the two this check reads.",
            pos.len()
        )));
    }
    Ok(Position {
        dx_pt: pos[0],
        dy_pt: pos[1],
        moved: line.get_usize("moved").ok_or_else(|| {
            Error::new(format!(
                "`{PREVIEW_EVENT}` carries no readable `moved=` field, so how many pages the \
                 job-wide reset would clear cannot be read."
            ))
        })?,
        edges: line
            .get("edges")
            .ok_or_else(|| {
                Error::new(format!(
                    "`{PREVIEW_EVENT}` carries no `edges=` field, so which edges the page hangs \
                     over cannot be read. That is the second half of O208 and a capture cannot \
                     supply it: three hatched bands and two are the same picture."
                ))
            })?
            .to_owned(),
        scale: line.get_f32("scale").ok_or_else(|| {
            Error::new(format!(
                "`{PREVIEW_EVENT}` carries no `scale=` field, so a movement in screen points \
                 cannot be converted to the paper points `pos=` is in."
            ))
        })?,
    })
}

/// A declared region, scrolling the dialog body downward until it appears.
///
/// The Position group fills a scrolling options column, so on a short window
/// its lower controls are genuinely off screen and their regions are genuinely
/// absent — `ui_rect_visible` is what publishes them and it refuses below 60 %
/// visible. An absent region here is therefore a scroll position, not a missing
/// control, and a check that read it as the latter would report a defect in a
/// button that is drawn every time the dialog opens.
fn scrolled_into_view(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    name: &str,
) -> Result<Option<LRect>> {
    for _ in 0..SCROLL_TRIES {
        let trace = session.trace()?;
        if let Some(rect) = declared(&trace, ui_rect, name) {
            return Ok(Some(rect));
        }
        let Some(anchor) = declared(&trace, ui_rect, TAB_POSITION) else {
            return Ok(None);
        };
        driver.scroll_at(
            frame_of(session, &trace, ui_rect, TAB_POSITION)?.declared_center(anchor),
            SCROLL_NOTCHES,
        )?;
        session.settle(10);
    }
    Ok(declared(&session.trace()?, ui_rect, name))
}

/// Press one Position button by name, scrolling to it first, and read the
/// preview afterwards.
fn press(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    name: &str,
    report: &mut CheckReport,
) -> Result<Position> {
    let Some(rect) = scrolled_into_view(session, driver, ui_rect, name)? else {
        return Err(Error::new(format!(
            "`{name}` is not declared even after scrolling the dialog body to the bottom. \
             Regions declared under `print.position.`: {}. Either the Position group drew \
             nothing, or the button's `Response` is no longer published.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "print.position."
            ))
        )));
    };
    driver.click_at(frame_of(session, &session.trace()?, ui_rect, name)?.declared_center(rect))?;
    session.settle(12);
    let after = position(&session.trace()?)?;
    report.note(format!("after {name}: {}", after.where_it_is()));
    Ok(after)
}

/// The sharp half of the magnitude assertion: at a preview scale below 1, one
/// screen point of pointer travel is MORE than one paper point of page
/// movement, so the page must have moved further in paper points than the
/// pointer moved in screen points.
///
/// This is the assertion a missing `/ scale` fails, and it fails it by a factor
/// rather than by a percentage — which the wide [`MOVE_BAND`] cannot promise,
/// because at a typical fit scale of 0.4 a missing division lands inside the
/// band.
fn assert_magnified(moved_pt: f64, scale: f32, axis: char) -> Option<String> {
    if scale >= MAGNIFY_SCALE_CEILING || moved_pt.abs() > f64::from(DRAG_PT) {
        return None;
    }
    Some(format!(
        "the pointer travelled {DRAG_PT} screen points on {axis} at a preview scale of {scale}, \
         and the page moved {moved_pt:.2} paper points. At that scale one screen point is more \
         than one paper point, so in paper points the page must move FURTHER than the pointer \
         did. It did not, which is what a screen delta applied without dividing out the preview \
         scale looks like: the page crawls behind the pointer, and the offset the operator types \
         in millimetres means something different from the one the drag produces."
    ))
}

#[expect(
    clippy::too_many_lines,
    reason = "one driven gesture sequence; splitting it would hide the order the assertions depend on"
)]
fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. `file.print` is gated on `doc.open`, so with nothing open the control is \
             greyed and there is no dialog to reach. A large-format 1:1 sheet such as \
             fixtures/a1-titleblock.pdf is the fixture this check is about.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is a drag and six clicks. Reported \
             as SKIPPED rather than passed — a check that did not run has learned nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("print_position.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so {}={} did not reach the process.",
            ctx.profile.vocab.start_event, ctx.profile.diag_env.0, ctx.profile.diag_env.1,
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- open the dialog ----------------------------------------------------
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(frame_of(&session, &trace, ui_rect, TAB)?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so no click \
             reached the ribbon."
        )));
    }
    let Some(control) =
        crate::checks::driving::declared_or_in_overflow(&session, &driver, ui_rect, SUBJECT)?
    else {
        let trace = session.trace()?;
        return Err(Error::new(format!(
            "the File tab is active and neither it nor its overflow declares `{SUBJECT}`. \
             Controls declared: {}. That is `print_dialog`'s defect, not this one.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    driver.click_at(
        frame_of(&session, &session.trace()?, ui_rect, SUBJECT)?.declared_center(control),
    )?;
    // Enumerating printers touches the spooler, which BLOCKS on a network
    // printer. The same settle every other print check uses, for the same
    // reason.
    session.settle(40);

    let trace = session.trace()?;
    let Some(open) = trace.events(OPEN_EVENT).next() else {
        return Err(Error::new(format!(
            "the click on `{SUBJECT}` produced no `{OPEN_EVENT}` line, so the dialog never \
             opened. That is `print_dialog`'s subject; nothing about the page's position can be \
             learned here."
        )));
    };
    if open.get("unavailable").unwrap_or("<absent>") != "None" {
        return Err(Error::new(
            "the spooler refused on this machine, so there is no device geometry and therefore \
             no paper for the page to be positioned on. Reported as SKIPPED.",
        ));
    }

    // --- Actual size, so the page is at 1:1 on the paper --------------------
    let Some(actual) = declared(&trace, ui_rect, SCALE_ACTUAL) else {
        return Err(Error::new(format!(
            "the dialog declares no `{SCALE_ACTUAL}` region. Scale regions declared: {}. The \
             dialog opens on Fit, which scales the page down to the printable area and crops \
             nothing, so without this region the geometry the request is about is unreachable.",
            list(&declared_names(&trace, ui_rect, "print.scale."))
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, SCALE_ACTUAL)?.declared_center(actual))?;
    session.settle(16);

    // --- the Position tab, which is where every control below now lives -----
    //
    // Pressed AFTER the scale radio and not before: the radio is on the Pages
    // & Layout tab, and the two tabs are never drawn at once. The preview is a
    // separate column and stays visible either way, which is why the drag
    // below needs no tab of its own.
    let trace = session.trace()?;
    let Some(tab_position) = declared(&trace, ui_rect, TAB_POSITION) else {
        return Err(Error::new(format!(
            "the dialog declares no `{TAB_POSITION}` region. Print-dialog tabs declared: {}. \
             Every position control is behind that tab, so without it none of them is \
             reachable.",
            list(&declared_names(&trace, ui_rect, "print.tab."))
        )));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, TAB_POSITION)?.declared_center(tab_position),
    )?;
    session.settle(16);

    // --- assertion 1: it starts unmoved -------------------------------------
    let start = position(&session.trace()?)?;
    report.note(format!("at actual size: {}", start.where_it_is()));
    if !start.at_the_origin() {
        return Ok(Some(format!(
            "the dialog opened with the page already displaced ({}). An untouched job must plan \
             identically to one from before positions existed — that is the whole basis for this \
             feature being unable to regress anything.",
            start.where_it_is()
        )));
    }

    // --- assertion 2: the drag moves the PAGE, not the view -----------------
    let Some(page) = declared(&session.trace()?, ui_rect, PAGE) else {
        return Err(Error::new(format!(
            "the preview declares no `{PAGE}` region, so there is no published rectangle to \
             start a drag inside. A drag aimed anywhere else on the canvas pans the view, which \
             is a different gesture on the same mouse button — guessing the rectangle would \
             measure the pan half the time."
        )));
    };
    let (w, h) = (page.width(), page.height());
    if w < DRAG_PT * ROOM_FOR_THE_DRAG || h < DRAG_PT * ROOM_FOR_THE_DRAG {
        return Err(Error::new(format!(
            "the grabbable part of the page is {w:.0}x{h:.0} logical points, too small to drag \
             {DRAG_PT} points inside without leaving it. A press that STARTS outside the page is \
             a pan, so rather than risk measuring the wrong gesture this is reported as SKIPPED. \
             A larger window, or a fixture whose sheet fills more of the preview, makes it bite."
        )));
    }
    let frame = frame_of(&session, &session.trace()?, ui_rect, PAGE)?;
    driver.drag(
        frame.declared_at(page, 0.5, 0.5),
        frame.declared_at(page, 0.5 - DRAG_PT / w, 0.5 - DRAG_PT / h),
    )?;
    session.settle(16);

    let trace = session.trace()?;
    let dragged = position(&trace)?;
    report.note(format!("after the drag: {}", dragged.where_it_is()));

    if !trace
        .events(PREVIEW_EVENT)
        .any(|l| l.get("grab") == Some("page"))
    {
        return Ok(Some(format!(
            "no frame of the drag reported `grab=page`; the preview classified a press that \
             started inside the published page rectangle as something else. The reading after \
             the gesture was {}. A drag that pans when it should have moved the page leaves \
             `pos=` alone and `pan=` changed, which is indistinguishable from a drag that never \
             reached the canvas at all — separating those two is what this field is for.",
            dragged.where_it_is()
        )));
    }

    if dragged.dx_pt >= 0.0 || dragged.dy_pt >= 0.0 {
        return Ok(Some(format!(
            "the pointer was dragged UP and LEFT and the page reports {}. The placement offsets \
             are handed to the device context as its own destination origin, which is the same \
             right-and-down sense as the screen, so both numbers must go negative and neither \
             axis is flipped. A sign flip on one axis looks like a working feature until the \
             operator tries to recover content off the bottom of the sheet.",
            dragged.where_it_is()
        )));
    }

    if let Some(why) = assert_magnified(dragged.dx_pt, dragged.scale, 'x')
        .or_else(|| assert_magnified(dragged.dy_pt, dragged.scale, 'y'))
    {
        return Ok(Some(why));
    }

    // The prediction is the WHOLE pointer travel, and that is only what arrives
    // when egui has already called the press a drag by the end of the first step.
    // Held here as well as in `travel_per_step_clears_egui`, because a check that
    // mis-predicts does not report a broken prediction — it reports a defect in
    // the application, in a sentence naming functions that are correct.
    let step_pt = f64::from(DRAG_PT) / f64::from(crate::input::DRAG_STEPS);
    if step_pt <= f64::from(EGUI_MAX_CLICK_DIST) {
        return Ok(Some(format!(
            "the driver walks this {DRAG_PT} pt drag in steps of {step_pt:.1} pt, which does \
             not exceed egui's {EGUI_MAX_CLICK_DIST} pt click distance. The steps before \
             egui calls the press a drag reach no `drag_delta()`, so the movement predicted \
             below is larger than any correct application would produce. This is a fact about \
             the harness, not about the application."
        )));
    }

    let predicted = f64::from(DRAG_PT / dragged.scale);
    for (axis, got) in [('x', dragged.dx_pt), ('y', dragged.dy_pt)] {
        let ratio = got.abs() / predicted;
        if ratio < MOVE_BAND.0 || ratio > MOVE_BAND.1 {
            return Ok(Some(format!(
                "on {axis} the page moved {got:.2} paper points against a predicted \
                 {predicted:.2} — a ratio of {ratio:.2}, outside the {:.2}..{:.2} band. The \
                 prediction is the whole pointer travel, which is what reaches the page while \
                 one driver step is longer than egui's click distance. Either the delta is \
                 being scaled by something other than the preview scale, or it is being \
                 applied more than once per frame.",
                MOVE_BAND.0, MOVE_BAND.1
            )));
        }
    }
    report.note(format!(
        "the drag moved the page {:.2},{:.2} paper points at scale {:.4}, against {predicted:.2} \
         predicted per axis",
        dragged.dx_pt, dragged.dy_pt, dragged.scale
    ));

    // --- assertion 3: the near edges are now overhung -----------------------
    report.note(format!(
        "edges before the drag: {}; after: {}",
        start.edges, dragged.edges
    ));
    let missing: String = ['l', 't']
        .into_iter()
        .filter(|&e| !dragged.over(e))
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!(
            "the page was dragged off the left and top of the printable area — {} — and \
             `edges={}` does not report `{missing}`. The bands the hatch is drawn from are the \
             same array this word is built from, so an edge missing here is an edge with no hash \
             lines on it: exactly the half of the request that asks for a line on every edge of \
             the page. Before the drag the word read `{}`.",
            dragged.where_it_is(),
            dragged.edges,
            start.edges
        )));
    }

    // --- assertions 4 and 5: centre, and centring twice ---------------------
    let centred = press(&session, &driver, ui_rect, CENTRE, report)?;
    if centred.same_place_as(&dragged) {
        return Ok(Some(format!(
            "Centre left the page exactly where the drag did ({}). The button either did not \
             reach the position map, or computed a target equal to the offset already there — \
             which is what a centring written as an absolute assignment rather than as a \
             displacement from the engine's own placement does.",
            centred.where_it_is()
        )));
    }
    if centred.moved != 1 {
        return Ok(Some(format!(
            "one page is displaced and the job reports moved={}. That count is what the job-wide \
             reset's sentence is written from, so a wrong one names the wrong number of pages \
             ({}).",
            centred.moved,
            centred.where_it_is()
        )));
    }
    let across = press(&session, &driver, ui_rect, CENTRE_H, report)?;
    let down = press(&session, &driver, ui_rect, CENTRE_V, report)?;
    if !across.same_place_as(&centred) || !down.same_place_as(&centred) {
        return Ok(Some(format!(
            "centring is not idempotent: Centre left the page at {}, then Centre horizontally \
             moved it to {} and Centre vertically to {}. Each press sees the DISPLACED placement \
             the preview is drawing, so the new offset has to be computed as the old one plus \
             the distance to the target. A press that assigns the target instead walks the page \
             further every time it is pressed.",
            centred.where_it_is(),
            across.where_it_is(),
            down.where_it_is()
        )));
    }

    // --- assertion 5b: each axis button moves ONE axis ----------------------
    //
    // Assertion 5 presses the two axis buttons on an already-centred page,
    // where not moving is the pass. A button that does nothing at all satisfies
    // that, and so does one wired to the axis it does not own. The state that
    // separates the three is an UNcentred one, so the page goes back to the
    // engine's placement and each button is pressed from there: the axis it owns
    // has to reach the value Centre produced, and the other axis has to stay.
    let origin = press(&session, &driver, ui_rect, RESET, report)?;
    if !origin.at_the_origin() {
        return Ok(Some(format!(
            "Reset left {} rather than the engine's own placement, so the two axis buttons \
             below would be pressed from a state this check does not know.",
            origin.where_it_is()
        )));
    }
    let only_across = press(&session, &driver, ui_rect, CENTRE_H, report)?;
    if (only_across.dx_pt - centred.dx_pt).abs() >= 0.005 || only_across.dy_pt.abs() >= 0.005 {
        return Ok(Some(format!(
            "Centre horizontally, pressed on a page at the engine's placement, left {} — it \
             was asked to put x at {:.2} and to leave y at 0. A button that moves NEITHER axis \
             passes the idempotence assertion above, and so does one wired to the other axis: \
             this is the assertion neither of those can pass.",
            only_across.where_it_is(),
            centred.dx_pt
        )));
    }
    let then_down = press(&session, &driver, ui_rect, CENTRE_V, report)?;
    if !then_down.same_place_as(&centred) {
        return Ok(Some(format!(
            "Centre vertically, pressed after Centre horizontally, left {} rather than the \
             {} that Centre itself produces. The two axis buttons have to compose into Centre, \
             or one of them is moving the axis it does not own.",
            then_down.where_it_is(),
            centred.where_it_is()
        )));
    }
    report.note(
        "each axis button moved only its own axis, and the two together composed into Centre"
            .to_owned(),
    );

    // --- assertion 6: the job-wide reset ------------------------------------
    let all = press(&session, &driver, ui_rect, RESET_ALL, report)?;
    if !all.at_the_origin() {
        return Ok(Some(format!(
            "Reset all pages left {}. Reset means the placement the engine chose, which for an \
             oversized page is flush at the corner and NOT the centre — the engine clamps its \
             own offsets at zero.",
            all.where_it_is()
        )));
    }

    // --- assertion 7: the per-page reset, by its own route ------------------
    let recentred = press(&session, &driver, ui_rect, CENTRE, report)?;
    if recentred.moved != 1 {
        return Ok(Some(format!(
            "after a job-wide reset, centring one page again reports {}. The map is sparse and \
             keyed on the document page index; a reset that removed the key rather than clearing \
             the value would leave the page unmovable afterwards.",
            recentred.where_it_is()
        )));
    }
    let reset = press(&session, &driver, ui_rect, RESET, report)?;
    if !reset.at_the_origin() {
        return Ok(Some(format!(
            "Reset position left {}. The per-page route must clear the page it is shown beside, \
             and it is the control the operator reaches for to undo an experiment.",
            reset.where_it_is()
        )));
    }

    report.note(
        "the page moved with the pointer, gained hash lines on the near edges, centred \
         idempotently, and came back to the engine's own placement by both reset routes"
            .to_owned(),
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{DRAG_PT, EGUI_MAX_CLICK_DIST};
    use crate::input::DRAG_STEPS;

    /// The movement this check predicts is the whole pointer travel, and that is
    /// only right while one step of the driver's walk is longer than egui's
    /// click-versus-drag distance. Shorten the drag, raise the step count, or let
    /// egui's default move, and the prediction is wrong by however many steps
    /// precede the crossing — which the check would report as the application
    /// scaling the delta by something other than the preview scale.
    #[test]
    fn travel_per_step_clears_egui() {
        let step = f64::from(DRAG_PT) / f64::from(DRAG_STEPS);
        assert!(
            step > f64::from(EGUI_MAX_CLICK_DIST),
            "one driver step is {step} pt, which does not exceed egui's {EGUI_MAX_CLICK_DIST} pt"
        );
    }
}
