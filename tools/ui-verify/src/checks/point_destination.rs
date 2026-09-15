//! **A destination that names a point scrolls, and never sets the
//! magnification** — the driven half of `DEFECTS.md` D47, for
//! `OPERATOR_REQUESTS.md` O200.
//!
//! # The operator's report
//!
//! > *"after we made it so bookmarks in drawing files exported from solidworks
//! > zoom to the correct spot on the page, this behaviour carried over onto
//! > links in word documents saved as pdfs such as table of contents where they
//! > should just jump the position on the page pointed to without changing the
//! > zoom. If the position jumped to is visible on the page with the current
//! > horizontal position of the page, the horizontal position shouldn't be
//! > changed."*
//!
//! Two clauses, and they need different instruments. The first is an
//! **identity**: the magnification after the jump is the magnification before
//! it, to the last digit the canvas prints. The second is a **conditional**, and
//! a conditional cannot be measured by one drive — "the horizontal did not move"
//! is also what a shell that ignored the destination entirely would produce.
//!
//! # Why there are two checks over one fixture
//!
//! | check | link | destination `left` | what it asserts |
//! |---|---|---|---|
//! | [`APointDestinationLeavesTheMagnificationAlone`] | near | 168 — under the pointer | the zoom is unchanged **and** the horizontal is held where the operator left it |
//! | [`APointDestinationOffScreenMovesTheHorizontal`] | far | 1150 — across the sheet | the zoom is unchanged **and** the horizontal moved |
//!
//! `fixtures/xyz-null-zoom.pdf` is built so the two links differ in exactly one
//! property: same target page, same `top`, same null zoom, same rectangle size
//! and the same rectangle `x`, so both drives zoom about the same content point
//! and arrive at the click with the same horizontal geometry. Only the
//! destination's `left` differs. A difference in outcome therefore has one
//! candidate cause. Its `PROVENANCE.py` carries the argument for every number.
//!
//! The pair is what makes either one evidence. Without the far link, a shell
//! that never touched the horizontal at all would be green; without the near
//! link, a shell that always recentred would be.
//!
//! # Why it zooms in first
//!
//! For the reason `link_follow` records from a falsification that failed to
//! falsify: at a fitted view there is nothing for the check to measure. A
//! 1,224 pt page fitted into the window has no horizontal scroll range, so the
//! offset is pinned at the same number whatever the shell decides, and the
//! "held" assertion is satisfied by a build with no horizontal logic in it.
//! Eight Ctrl+wheel notches about the link's own centre put the page several
//! times the width of the viewport; the range is then **measured** before the
//! click, and a run that did not get enough of it reports SKIP with the numbers
//! rather than passing on a degenerate view.
//!
//! Zoom-to-cursor also does the check a second favour: it holds the point under
//! the pointer fixed, so the near destination is on screen **by construction**
//! and the check never has to predict a window size, a fit zoom or a notch step.
//!
//! # What it reads
//!
//! `canvas … zoom= page= off=` for the view the operator ends up with, and
//! `dest-scroll-solved … origin_x= keep_x= off_x=` for what the solver decided.
//! Both, deliberately: the trace of a stage records what that stage decided, not
//! what the frame settled on, and the offset chain has seven other ranks that
//! could overwrite it. `origin_x` is also asserted against the offset measured
//! before the click, which is the only way to catch the plumbing defect this
//! feature actually had — `canvas::strip::page_scroll_offset` centres the named
//! page horizontally on the frame the page turns, so an origin read after that
//! measures the centring rather than the operator.
//!
//! # Every way these report SKIP
//!
//! No binary, `--no-input`, no diagnostic channel, no canvas rect to aim
//! against, the fixture missing, or a window so wide that eight notches of zoom
//! still leave no horizontal scroll range. None of those is a pass, and each
//! says which it was.

use std::path::PathBuf;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::{Driver, Key};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// `link-click page=… index=… kind=…` — the shell's record of a link press.
const CLICK_EVENT: &str = "link-click";

/// The canvas's per-frame line, carrying `page=`, `zoom=` and `off=`.
const CANVAS: &str = "canvas";

/// `page-links page=… links=… unresolvable=… named=…` — quoted in a failure
/// report when no click line appeared, to separate "the link was not found" from
/// "the press never reached the link arm".
const RESOLVE_EVENT: &str = "page-links";

/// `dest-scroll-solved page=… frac_x=… keep_x=… origin_x=… off_x=…` — the
/// solver's own record, one `key=value` per field.
const SOLVED_EVENT: &str = "dest-scroll-solved";

/// `dest-scroll-dropped page=… showing=…` — the destination gave up waiting for
/// its page. Quoted when the solved line is missing, because it names the cause.
const DROPPED_EVENT: &str = "dest-scroll-dropped";

/// The region the canvas publishes for its scroll viewport.
const CANVAS_REGION: &str = "canvas-viewport";

/// Two `/XYZ … null` links on page 1, both targeting page 2.
const FIXTURE: &str = "fixtures/xyz-null-zoom.pdf";

/// Both pages, twice the width of US Letter. See the fixture's `PROVENANCE.py`
/// for why a Letter page cannot carry this check.
const PAGE_PT: (f64, f64) = (1224.0, 792.0);

/// Where both links land, 0-based.
const TARGET_PAGE: usize = 1;

/// The centre of the near link — rect 36,700–300,730.
const NEAR_AIM: (f64, f64) = (168.0, 715.0);

/// The near link's destination `left`, which is also the x this check zooms
/// about, which is what puts it on screen by construction.
const NEAR_DEST_LEFT: f64 = 168.0;

/// The centre of the far link — rect 36,640–300,670. Same x as the near aim, so
/// both drives reach the click with the same horizontal geometry.
const FAR_AIM: (f64, f64) = (168.0, 655.0);

/// The far link's destination `left`, 74 pt from the right edge of the sheet.
const FAR_DEST_LEFT: f64 = 1150.0;

/// Ctrl+wheel notches before the click.
///
/// Enough that a 1,224 pt page is several times the width of any window this
/// runs in. The range is measured afterwards regardless — this constant is the
/// thing to raise if that measurement starts reporting SKIP.
const ZOOM_NOTCHES: usize = 8;

/// The least horizontal scroll range, in logical points, that makes the
/// assertions mean anything.
///
/// Below this the offset has nowhere to go and both "held" and "moved" become
/// statements about the clamp instead of about the destination.
const MIN_RANGE: f32 = 200.0;

/// How far the horizontal must move for the far link to count as having moved.
///
/// The far destination is 982 pt to the right of the near one and the view is
/// magnified, so the real figure is several hundred points. This is a floor
/// well clear of a rounding difference, not an expected value.
const MIN_MOVE: f32 = 100.0;

/// How close `origin_x` and the measured pre-click offset must be to count as
/// the same number. They are the same `f32` written by two paths; the tolerance
/// is for the trace's one decimal place.
const SAME_OFFSET: f32 = 0.5;

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The view as the canvas reported it: page, the `zoom=` field verbatim, and the
/// scroll offset.
///
/// The zoom is carried as the **string** the application printed rather than a
/// parsed float, because the first clause of the request is an identity and the
/// honest test of an identity is byte equality. A parsed `f32` compared with a
/// tolerance would accept a shell that re-derived the same magnification by a
/// different route, which is the failure D47 was.
#[derive(Clone, Debug)]
struct View {
    page: usize,
    zoom: String,
    off: crate::geom::Pt,
}

fn view_from(line: &crate::trace::TraceLine) -> Option<View> {
    Some(View {
        page: line.get_usize("page")?,
        zoom: line.get("zoom")?.to_owned(),
        off: line.get_vec2("off")?,
    })
}

fn view_now(trace: &Trace) -> Option<View> {
    view_from(trace.last(CANVAS)?)
}

/// One link and what it is supposed to prove.
struct Subject {
    /// The document point to click, on page 0.
    aim: (f64, f64),
    /// The destination's `left`, quoted in failure reports so a reader does not
    /// have to open the fixture to know which link ran.
    dest_left: f64,
    /// What `keep_x` must be: `true` for the control, `false` for the witness.
    held: bool,
    /// Where the captured trace goes.
    trace_name: &'static str,
}

/// The near link: its destination is under the pointer, so the horizontal is
/// held where the operator left it.
pub struct APointDestinationLeavesTheMagnificationAlone;

impl Check for APointDestinationLeavesTheMagnificationAlone {
    fn name(&self) -> &'static str {
        "a_point_destination_leaves_the_magnification_alone"
    }

    fn defect(&self) -> &'static str {
        "Following a table-of-contents link in a Word-exported PDF magnifies the page to \
         several hundred percent, when the link asked only for a position — or it recentres \
         the sheet sideways away from what the operator was reading"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let subject = Subject {
            aim: NEAR_AIM,
            dest_left: NEAR_DEST_LEFT,
            held: true,
            trace_name: "point-dest-near.trace.txt",
        };
        match drive(ctx, &mut report, &subject) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The far link: its destination is across the sheet, so the horizontal must
/// move. The witness without which the control above proves nothing.
pub struct APointDestinationOffScreenMovesTheHorizontal;

impl Check for APointDestinationOffScreenMovesTheHorizontal {
    fn name(&self) -> &'static str {
        "a_point_destination_off_screen_moves_the_horizontal"
    }

    fn defect(&self) -> &'static str {
        "Following a link whose destination is off the side of the view leaves the operator \
         looking at the wrong part of the sheet, because the horizontal was held still for \
         every destination rather than for the ones already on screen"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let subject = Subject {
            aim: FAR_AIM,
            dest_left: FAR_DEST_LEFT,
            held: false,
            trace_name: "point-dest-far.trace.txt",
        };
        match drive(ctx, &mut report, &subject) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Launch on the fixture, provoke a canvas rect, and return the session, the
/// driver and a mapping to aim with. `Err` is a SKIP.
fn set_up(
    ctx: &CheckContext,
    report: &mut CheckReport,
    trace_name: &str,
) -> Result<(Session, Driver, CanvasMapping)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check zooms and clicks the canvas. Reported \
             as SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let pdf = workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the fixture is not at {}. Regenerate it with \
             `python fixtures/xyz-null-zoom.PROVENANCE.py`.",
            pdf.display()
        )));
    }

    // The fixture is pinned and `--pdf` is ignored: every assertion below names
    // a specific link at a specific rectangle with a specific destination.
    // Pointed at the operator's own drawing this would click empty paper and
    // report a working feature as broken.
    let mut spec = LaunchSpec::new(&exe, ctx.out(trace_name));
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
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

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

    // A layout probe, for `checks::delete_key`'s stated reason: some builds
    // trace their canvas rect only on a pointer event. The client-area centre on
    // this fixture is blank paper below both link rows.
    let trace = if trace.last(ctx.profile.vocab.canvas_event).is_some() {
        trace
    } else {
        driver.click_at(session.frame()?.layout_probe_point())?;
        session.settle(10);
        session.trace()?
    };

    let mapping = mapping_now(ctx, &trace)?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}",
        mapping.image_rect, mapping.zoom
    ));
    Ok((session, driver, mapping))
}

/// A mapping against the page the fixture opens on.
fn mapping_now(ctx: &CheckContext, trace: &Trace) -> Result<CanvasMapping> {
    CanvasMapping::from_trace(
        trace,
        &ctx.profile.vocab,
        crate::coords::PageGeometry {
            width_pt: PAGE_PT.0,
            height_pt: PAGE_PT.1,
        },
        0,
    )
}

/// Click a document point on page 0 and settle.
fn click_doc(
    session: &Session,
    driver: &Driver,
    mapping: &CanvasMapping,
    at: (f64, f64),
) -> Result<()> {
    let window = mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?;
    driver.click_at(session.frame()?.to_screen(window))?;
    session.settle(20);
    Ok(())
}

/// How much horizontal scroll range the view has: the drawn page width less the
/// viewport width.
///
/// `Err` is a SKIP rather than a failure, because a view with no range is a
/// statement about the window and the notch count, not about the shell.
fn horizontal_range(ctx: &CheckContext, trace: &Trace, report: &mut CheckReport) -> Result<f32> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    let viewport = driving::declared(trace, ui_rect, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "the canvas declared no `{CANVAS_REGION}` region, so this check cannot measure \
             whether the view has any horizontal scroll range — and without range, both of its \
             assertions are statements about a clamp."
        ))
    })?;
    let page = mapping_now(ctx, trace)?.image_rect;
    let range = page.width() - viewport.width();
    if range < MIN_RANGE {
        return Err(Error::new(format!(
            "after {ZOOM_NOTCHES} Ctrl+wheel notches the drawn page is {:.0} pt wide in a \
             {:.0} pt viewport, leaving {range:.0} pt of horizontal scroll range — less than \
             the {MIN_RANGE:.0} pt this check needs.\n\n\
             Reported as SKIPPED rather than passed: with no range the offset cannot move, so \
             `the horizontal was held' would be true of a build with no horizontal logic in it \
             at all. Raise `ZOOM_NOTCHES` in this file, or run on a narrower window.",
            page.width(),
            viewport.width()
        )));
    }
    report.note(format!(
        "horizontal scroll range {range:.0} pt — page {:.0} pt in a {:.0} pt viewport",
        page.width(),
        viewport.width()
    ));
    Ok(range)
}

/// Drive one link. `Ok(None)` is a pass; `Ok(Some(..))` is a failure report.
#[allow(clippy::too_many_lines)]
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    subject: &Subject,
) -> Result<Option<String>> {
    let (session, driver, mapping) = set_up(ctx, report, subject.trace_name)?;

    let opened = view_now(&session.trace()?).map_or(0, |v| v.page);
    if opened != 0 {
        return Err(Error::new(format!(
            "the document opened on page {opened} rather than page 0, so this check's aim — a \
             link on the FIRST page — is at the wrong sheet."
        )));
    }

    // Ctrl+wheel about the link's own centre. Two things at once: it leaves the
    // fitted view, which is what gives the horizontal anywhere to go, and it
    // holds the content under the pointer fixed, which is what puts the near
    // destination on screen without the check predicting anything.
    let at = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(0, subject.aim.0, subject.aim.1))?);
    driver.scroll_at_held(at, &[Key::Ctrl.vk()], 1, ZOOM_NOTCHES)?;
    session.settle(20);

    let trace = session.trace()?;
    horizontal_range(ctx, &trace, report)?;
    let Some(before) = view_now(&trace) else {
        return Err(Error::new(
            "the canvas reported no view state after the zoom, so this check has nothing to \
             compare against.",
        ));
    };
    report.note(format!(
        "before the click: page {}, zoom {}, offset {:.1},{:.1}",
        before.page, before.zoom, before.off.x, before.off.y
    ));

    // The zoom moved, so every screen coordinate taken before it is stale — a
    // standing finding in `D:/dev/rag/egui/`.
    let mapping = mapping_now(ctx, &trace)?;
    let mark = trace.mark();
    click_doc(&session, &driver, &mapping, subject.aim)?;

    let trace = session.trace()?;
    let Some(click) = trace.last_after(CLICK_EVENT, mark) else {
        return Ok(Some(format!(
            "THE CLICK ON A LINK PRODUCED NOTHING. No `{CLICK_EVENT}` line followed a press at \
             the centre of a `/Link` whose rectangle the engine reports on this page, with a \
             destination of `/XYZ {} 500 null`.\n\n\
             Either `canvas::links::under_pointer` found no link (`{RESOLVE_EVENT}`: {}) or \
             something above the link arm took the press. Trace: {}.",
            subject.dest_left,
            trace
                .last(RESOLVE_EVENT)
                .map_or_else(|| "not traced at all".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    if click.get("kind") != Some("page") {
        return Ok(Some(format!(
            "THE LINK WAS NOT RESOLVED TO A PAGE: `{}`. Both links in this fixture are direct \
             `/GoTo` actions with an array destination, so a shell that classified one as \
             anything else is disagreeing with the reader it called.",
            click.raw
        )));
    }

    let Some(after) = trace.last_after(CANVAS, mark).and_then(view_from) else {
        return Ok(Some(format!(
            "THE CANVAS SAID NOTHING AFTER THE CLICK. No `{CANVAS}` line was traced after the \
             press, so either the frame did not change at all — the link did nothing — or the \
             canvas stopped reporting. Those are different failures and this cannot tell them \
             apart; read {}.",
            session.trace_path().display()
        )));
    };

    // Clause one, and the reason this file exists: the magnification is the
    // magnification the operator had. Byte equality of the field the canvas
    // printed, not a tolerance on a re-derived number.
    if after.zoom != before.zoom {
        return Ok(Some(format!(
            "THE LINK CHANGED THE MAGNIFICATION: zoom {} before the click, {} after it, on a \
             destination spelled `/XYZ {} 500 null` — which names a position and, by \
             §12.3.2.2 Table 151, explicitly declines to name a magnification.\n\n\
             This is D47's shape: a point widened into a rectangle and handed to a framing \
             solver, which answers with whatever magnification that rectangle needs. \
             `canvas::destscroll::solve` must be what answers a point, and it cannot return a \
             zoom. Trace: {}.",
            before.zoom,
            after.zoom,
            subject.dest_left,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the magnification is unchanged at {} — the identity the request is about",
        after.zoom
    ));

    if after.page != TARGET_PAGE {
        return Ok(Some(format!(
            "THE LINK WENT TO THE WRONG PAGE: {} → {}, where {TARGET_PAGE} was named. {}",
            before.page,
            after.page,
            if after.page == before.page {
                "The page did not change at all, so the destination was recognised and then \
                 not performed."
            } else {
                "An off-by-one between the engine's 0-based page index and the 1-based numbers \
                 this program shows is the first thing to check."
            }
        )));
    }

    let Some(solved) = trace.last_after(SOLVED_EVENT, mark) else {
        return Ok(Some(format!(
            "THE DESTINATION WAS NEVER SOLVED. The click was recognised as a page \
             destination and the view reached page {TARGET_PAGE}, but no `{SOLVED_EVENT}` line \
             followed — so the arrival was whatever turning the page does, not what the \
             destination asked for. {}",
            trace.last_after(DROPPED_EVENT, mark).map_or_else(
                || format!(
                    "`{DROPPED_EVENT}` is absent too, so the destination was never \
                            parked: look at `canvas::destination::arrive`'s Point arm."
                ),
                |l| format!(
                    "`{}` — the destination gave up waiting for its page.",
                    l.raw
                ),
            )
        )));
    };
    report.note(format!("the solver answered: `{}`", solved.raw));

    let Some(keep_x) = solved.get("keep_x") else {
        return Ok(Some(format!(
            "THE SOLVER'S TRACE HAS NO `keep_x` FIELD: `{}`. That field is this check's oracle \
             for the second clause of the request; a line without it means the trace and the \
             harness have drifted apart. Fields present: {:?}.",
            solved.raw,
            solved.field_names()
        )));
    };
    let held = keep_x == "true";
    if held != subject.held {
        return Ok(Some(describe_wrong_hold(subject, &before, solved)));
    }

    // `origin_x` must be the offset the operator actually had. This is the half
    // that a unit test cannot reach: the strip centres the named page
    // horizontally on the frame the page turns, so an origin recorded any later
    // than `app::actions::view` measures that centring instead.
    if let Some(origin) = solved.get_f32("origin_x") {
        if (origin - before.off.x).abs() > SAME_OFFSET {
            return Ok(Some(format!(
                "THE SOLVER MEASURED THE WRONG HORIZONTAL POSITION. It recorded \
                 `origin_x={origin:.1}`, but the view was at {:.1} when the link was clicked.\n\n\
                 The request's second clause is about *the current horizontal position of the \
                 page* — the operator's, at the moment they clicked. \
                 `canvas::strip::page_scroll_offset` brings the named page into view and \
                 centres it horizontally on the frame the page turns, so an origin read after \
                 that measures the centring. `OpenDoc::dest_origin_x` must be written by \
                 `app::actions::view`, before any of it. Trace: {}.",
                before.off.x,
                session.trace_path().display()
            )));
        }
        report.note(format!(
            "the solver's `origin_x` is {origin:.1}, the offset measured before the click — so \
             it read the operator's position, not the page-turn's centring"
        ));
    }

    let moved = after.off.x - before.off.x;
    if subject.held {
        if moved.abs() > SAME_OFFSET {
            return Ok(Some(format!(
                "THE HORIZONTAL MOVED FOR A DESTINATION THAT WAS ALREADY ON SCREEN. The offset \
                 went from {:.1} to {:.1} — {moved:.1} pt — following a link whose destination \
                 is at x = {}, the same content point the view was zoomed about and therefore \
                 visibly on screen when the click landed.\n\n\
                 The solver said it would hold it (`{}`), so the offset was overwritten after \
                 the fact by another rank of `canvas::offset::decide` — `page-scroll` and \
                 `fit` are the two that outrank or follow `dest-scroll` and both recentre. \
                 Trace: {}.",
                before.off.x,
                after.off.x,
                subject.dest_left,
                solved.raw,
                session.trace_path().display()
            )));
        }
        report.note(format!(
            "the horizontal is still at {:.1} — where the operator left it",
            after.off.x
        ));
    } else {
        if moved < MIN_MOVE {
            return Ok(Some(format!(
                "THE HORIZONTAL DID NOT REACH A DESTINATION OFF THE SIDE OF THE VIEW. The \
                 offset went from {:.1} to {:.1} — {moved:.1} pt — following a link whose \
                 destination is at x = {} on a {:.0} pt sheet, which at this magnification is \
                 far to the right of anything that was on screen.\n\n\
                 The solver said it would move it (`{}`), so either the offset was overwritten \
                 afterwards or the visibility test in `canvas::destscroll::solve` is reading a \
                 clamped probe. Trace: {}.",
                before.off.x,
                after.off.x,
                subject.dest_left,
                PAGE_PT.0,
                solved.raw,
                session.trace_path().display()
            )));
        }
        report.note(format!(
            "the horizontal moved {moved:.1} pt to the right, to {:.1} — the destination is \
             across the sheet and the view went and got it",
            after.off.x
        ));
    }
    Ok(None)
}

/// The failure report for a `keep_x` that disagrees with the subject.
///
/// Split out because the two directions are different accusations and a single
/// formatted sentence with a conditional in it reads as neither.
fn describe_wrong_hold(
    subject: &Subject,
    before: &View,
    solved: &crate::trace::TraceLine,
) -> String {
    if subject.held {
        format!(
            "THE SOLVER MOVED A HORIZONTAL THAT WAS ALREADY ON SCREEN. `keep_x=false` on `{}`, \
             for a destination at x = {} — the same content point the view was zoomed about, \
             so it was under the pointer when the click landed and the view was at {:.1}.\n\n\
             `canvas::destscroll::axis_stays_where_it_is` decides this. It is `false` when the \
             point is outside the viewport span measured from `origin_x`, and both of its \
             clearances are one `CANVAS_MARGIN`; a probe clamped to the scroll range answers \
             `false` too, which is the likelier cause near either end of the content.",
            solved.raw, subject.dest_left, before.off.x
        )
    } else {
        format!(
            "THE SOLVER HELD A HORIZONTAL THAT COULD NOT BE SEEN. `keep_x=true` on `{}`, for a \
             destination at x = {} on a {:.0} pt sheet, from a view sitting at {:.1} and \
             magnified well past the fitted zoom.\n\n\
             A shell that holds the horizontal for every destination satisfies the request's \
             second clause by accident and fails it whenever the destination is somewhere \
             else — which on a wide drawing is most of the time. This is the witness half of \
             the pair, and it is the one that catches that.",
            solved.raw, subject.dest_left, PAGE_PT.0, before.off.x
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two aim points are inside the rectangles the fixture writes.
    ///
    /// Pinned because a click on empty paper is symptom-identical to a broken
    /// hit test, and these numbers are transcribed by hand from the fixture's
    /// `PROVENANCE.py`.
    #[test]
    fn both_aim_points_are_inside_their_link_rectangles() {
        // near: rect [36 700 300 730]
        assert!((36.0..=300.0).contains(&NEAR_AIM.0), "{NEAR_AIM:?}");
        assert!((700.0..=730.0).contains(&NEAR_AIM.1), "{NEAR_AIM:?}");
        // far: rect [36 640 300 670]
        assert!((36.0..=300.0).contains(&FAR_AIM.0), "{FAR_AIM:?}");
        assert!((640.0..=670.0).contains(&FAR_AIM.1), "{FAR_AIM:?}");
        for p in [NEAR_AIM, FAR_AIM] {
            assert!(p.0 > 0.0 && p.0 < PAGE_PT.0, "{p:?}");
            assert!(p.1 > 0.0 && p.1 < PAGE_PT.1, "{p:?}");
        }
    }

    /// The two drives differ in exactly one property.
    ///
    /// The whole argument for the pair rests on this: same x, so both zoom about
    /// the same content column and reach the click with the same horizontal
    /// geometry; different rows, so they are different links; different
    /// destination `left`, which is the one input the request's second clause
    /// turns on. If a future edit moves one aim sideways, the two outcomes stop
    /// being attributable to the destination.
    #[test]
    fn the_two_drives_differ_only_in_the_destination() {
        assert!(
            (NEAR_AIM.0 - FAR_AIM.0).abs() < f64::EPSILON,
            "the aims must share a column, or the horizontal geometry at the click differs \
             for a second reason: {NEAR_AIM:?} vs {FAR_AIM:?}"
        );
        assert!(
            (NEAR_AIM.1 - FAR_AIM.1).abs() > 30.0,
            "the aims must be different rows, or both clicks land on one link"
        );
        assert!(
            (NEAR_DEST_LEFT - FAR_DEST_LEFT).abs() > PAGE_PT.0 / 2.0,
            "the destinations must be most of a sheet apart, or the far one could be on \
             screen after all"
        );
    }

    /// The near destination is the point the check zooms about.
    ///
    /// That equality is what makes it visible by construction — zoom-to-cursor
    /// holds the content under the pointer fixed — so the check never has to
    /// predict a window size or a notch step. Break it and the control drive
    /// starts depending on the machine it runs on.
    #[test]
    fn the_near_destination_is_the_point_the_pointer_is_on() {
        assert!((NEAR_AIM.0 - NEAR_DEST_LEFT).abs() < f64::EPSILON);
    }

    /// The far destination is on the sheet, near its right edge.
    #[test]
    fn the_far_destination_is_on_the_sheet() {
        assert!(FAR_DEST_LEFT < PAGE_PT.0);
        assert!(FAR_DEST_LEFT > PAGE_PT.0 * 0.9);
    }

    /// The target is not page 0, so a defaulted index cannot reach it.
    #[test]
    fn the_target_cannot_be_reached_by_a_defaulted_index() {
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(
                TARGET_PAGE > 0,
                "a target of {TARGET_PAGE} is where an unresolved destination defaults to, so \
                 a green result would establish nothing"
            );
        }
    }

    /// Every trace event this check reads is spelled once.
    #[test]
    fn the_event_names_are_distinct() {
        let all = [
            CLICK_EVENT,
            CANVAS,
            RESOLVE_EVENT,
            SOLVED_EVENT,
            DROPPED_EVENT,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    /// The movement floor is well inside the range the check demands.
    ///
    /// If `MIN_MOVE` ever exceeded `MIN_RANGE` the witness drive could not pass
    /// on any view the precondition admits, and the failure would read as a
    /// product defect.
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn the_movement_floor_fits_inside_the_range_required() {
        assert!(MIN_MOVE < MIN_RANGE, "{MIN_MOVE} vs {MIN_RANGE}");
        assert!(SAME_OFFSET < MIN_MOVE, "{SAME_OFFSET} vs {MIN_MOVE}");
    }
}
