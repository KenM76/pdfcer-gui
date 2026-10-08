//! `a_selected_object_can_be_marked_for_redaction` — the driven proof of
//! `OPERATOR_REQUESTS.md` **O60**.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/redact_selection.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// Edit mode, the redact panel for its census, then the verb under test.
const INVOKE: &str = "mode.edit,edit.redact";

/// The panel's census: `redact-panel marks=N`.
const PANEL: &str = "redact-panel";

/// The shell's own line for the verb under test.
const REQUESTED: &str = "redact-mark-selection-requested";

/// What the canvas says when something is selected.
const SELECTION: &str = "canvas-selection";

/// The ribbon control for the verb under test.
const REDACT_BUTTON: &str = "ribbon.item.edit.redact_selection";

/// See the module documentation.
pub struct ASelectedObjectCanBeMarkedForRedaction;

impl Check for ASelectedObjectCanBeMarkedForRedaction {
    fn name(&self) -> &'static str {
        "a_selected_object_can_be_marked_for_redaction"
    }

    fn defect(&self) -> &'static str {
        "there is no way to redact something the search box cannot find — a drawn title block, a \
         scanned stamp, a logo — so on a CAD drawing the only routes are 'type the words', which \
         cannot reach vector strokes, and 'mark the whole page'"
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

/// How many redaction marks the panel says the document has.
fn marks(trace: &Trace) -> Option<usize> {
    trace
        .last(PANEL)
        .and_then(|l| l.get("marks"))
        .and_then(|v| v.parse().ok())
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check must click an object on the page to \
             select it. Reported as SKIPPED rather than passed.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a drawing with something on it."))?;
    let point = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs a coordinate with an OBJECT under it — the whole \
             subject is selecting something and redacting it, and a point on blank paper selects \
             nothing. Pass PAGE,X,Y in PDF user space.",
        )
    })?;
    let ui_rect = vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("redact-selection.trace.txt"));
    spec.pdf = Some(pdf);
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

    // Normalise the saved dock layout, or a panel TOGGLE alternates between
    // opening and closing across runs. Same rule, same file, as the bookmark
    // clipboard check — and it is the application that writes this state, not
    // the check.
    if let Some(dir) = exe.parent() {
        let layout = dir.join("userdata").join("layout.ron");
        if layout.exists() {
            let _ = std::fs::remove_file(&layout);
        }
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, "edit")?;
    session.settle(20);

    let before = marks(&session.trace()?).ok_or_else(|| {
        Error::new(format!(
            "the redaction panel published no `{PANEL} … marks=` line, so this check cannot count \
             marks and could not tell a working mark from a silent one. That panel is \
             `checks::redaction`'s surface; SKIPPED. Trace: {}",
            session.trace_path().display()
        ))
    })?;
    report.note(format!(
        "the document starts with {before} redaction mark(s)"
    ));

    // --- select something ---------------------------------------------------
    let mapping = CanvasMapping::from_trace(&session.trace()?, vocab, page, point.page)?;
    let at = DocPoint::new(point.page, point.x, point.y);
    driver.click_at(session.frame()?.to_screen(mapping.doc_to_window(at)?))?;
    session.settle(20);

    let selected = session
        .trace()?
        .last(SELECTION)
        .and_then(|l| l.get("first"))
        .is_some_and(|f| f != "none");
    if !selected {
        return Err(Error::new(format!(
            "the click at page {} ({:.1}, {:.1}) selected nothing, so there is no operand for the \
             verb under test. SKIPPED rather than failed: that is a fact about where this \
             coordinate points, not about redaction. Aim `--doc-point` at an object.",
            point.page + 1,
            point.x,
            point.y
        )));
    }
    report.note("★ selected an object on the page");

    // --- the verb under test ------------------------------------------------
    //
    // Clicked on the ribbon, because `PDFCER_DIAG_INVOKE` runs only at
    // START-UP and the selection this verb needs does not exist then. That is a
    // real constraint rather than a preference: an invoke chain cannot express
    // *"select something, then run this"*.
    //
    // ⇒ So this check does depend on the Redact band being drawn where the
    // manifest says. Named rather than hidden: if it fails at this step and
    // `the_ribbon_has_the_documented_shape` also fails, believe that one first.
    // THE SECURITY TAB FIRST. `mode.edit` sets the MODE, which decides which
    // tabs exist — it does not decide which one is showing, and the shell opens
    // on File. The first version of this check went straight to the item lookup
    // and reported the control missing while the trace held nine
    // `ribbon.item.file.*` regions and not one `ribbon.item.edit.*`.
    //
    // ⇒ Two different things wear the word "edit" here, and confusing them is
    // cheap to do: a MODE is which tabs you may see, a TAB is which one you are
    // looking at.
    let Some(tab) =
        crate::checks::driving::declared(&session.trace()?, ui_rect, "ribbon.tab.security")
    else {
        // The tab list is built HERE rather than in an `ok_or_else`
        // closure: building it needs `session.trace()?` and a closure cannot
        // carry the `?`. Worth the extra lines — a skip that names the tabs
        // that ARE there is diagnosable; one that says "not found" is not.
        let tabs =
            crate::checks::driving::declared_names(&session.trace()?, ui_rect, "ribbon.tab.");
        return Err(Error::new(format!(
            "no `ribbon.tab.security` region, so the Security tab is not on the ribbon and the \
             control under test cannot be reached by pointer. Tabs declared: {}.",
            crate::checks::driving::list(&tabs)
        )));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(18);

    // `declared_or_in_overflow`, not `declared`. The harness drives an
    // 1100 pt window, at which the ribbon correctly folds its rightmost groups
    // into an overflow menu — and a collapsed GROUP is a third place a command
    // can be. A check looking only at the tab surface reports *"the command is
    // missing"* about a ribbon that is behaving exactly as designed, which has
    // already cost this suite one standing FAIL that was written up as a
    // harness gap and left.
    let button = crate::checks::driving::declared_or_in_overflow(
        &session,
        &driver,
        ui_rect,
        REDACT_BUTTON,
    )?
    .ok_or_else(|| {
            Error::new(format!(
                "no `{REDACT_BUTTON}` region, so the Redact selection control is not on the ribbon and there is nothing to press. `edit.redact_selection` is gated on `selection.any` and an object IS selected, so this says the band is not drawn rather than that the control is disabled. SKIPPED."
            ))
        })?;
    driver.click_at(session.frame()?.declared_center(button))?;
    session.settle(30);

    let after_trace = session.trace()?;
    if after_trace.events(REQUESTED).next().is_none() {
        return Ok(Some(format!(
            "`edit.redact_selection` produced no `{REQUESTED}` line on a frame where an object \
             was selected. Either the command is not routed, or every selected outline was \
             filtered out — the arm keeps only entries whose page is the current one. Trace: {}",
            session.trace_path().display()
        )));
    }

    let after = marks(&after_trace).unwrap_or(before);
    if after <= before {
        return Ok(Some(format!(
            "★★★ THE REQUEST WAS MADE AND NO MARK EXISTS. The shell built quads and the panel \
             still counts {after}. The engine accepted nothing — look for a `redact-mark-selection \
             … refused` line, and suspect the canvas-to-PDF hop before suspecting the verb: a \
             quad built without normalising the y flip is inside-out, and one built from another \
             page's outlines lands off the sheet. Both look exactly like this. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ marks went from {before} to {after} — the mark reached the document"
    ));

    // --- and NOTHING was applied ----------------------------------------
    //
    // The most dangerous defect this feature can have is marking that silently
    // applies: the operator sees a mark, believes the content is gone, stops
    // reviewing, and saves a document that still contains every word — or, the
    // other way, loses content they had not agreed to lose. Either way the
    // review step they were promised did not happen.
    if after_trace.events("redact-written").next().is_some() {
        return Ok(Some(format!(
            "★★★ MARKING APPLIED THE REDACTION. A `redact-written` line is in the trace and \
             nothing pressed Apply. Marking must remove no content: a /Redact annotation is a \
             mark (12.5.6.23), the review list is the operator's chance to check it, and an \
             apply they did not ask for is the one act in this program that cannot be undone. \
             Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★★ and nothing was applied — the mark is a mark, as it must be");

    Ok(None)
}
