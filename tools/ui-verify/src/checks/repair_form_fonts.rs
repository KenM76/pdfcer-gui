//! `repair_form_fonts` — **Edit ▸ Forms ▸ Repair fonts on a form that keeps its
//! font inline moves it, and a second press says there is nothing left.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/repair_form_fonts.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose ribbon carries the Edit tab.
const MODE: &str = "edit";
/// One filled text field; `/AcroForm /DR /Font /Helv` is an inline dictionary.
const FIXTURE: &str = "inline-dr-font.pdf";
/// The Edit tab, and the command's item on it.
const TAB: &str = "ribbon.tab.edit";
const ITEM: &str = "ribbon.item.edit.form_repair_fonts";
/// `form-repair-fonts-applied fonts=N`, written after the engine moved N fonts.
const APPLIED_EVENT: &str = "form-repair-fonts-applied";
/// `form-repair-fonts-refused … detail=…`, the funnel's line for a press that
/// wrote nothing.
const REFUSED_EVENT: &str = "form-repair-fonts-refused";
/// The detail the action gives when no inline font is left.
const NOTHING_DETAIL: &str = "no inline /DR font";

/// See the module documentation.
pub struct RepairingFormFontsReachesTheDocument;

impl Check for RepairingFormFontsReachesTheDocument {
    fn name(&self) -> &'static str {
        "repair_form_fonts"
    }

    fn defect(&self) -> &'static str {
        "Edit ▸ Forms ▸ Repair fonts is offered and pressing it on a form whose font is \
         inline moves nothing, or moves it again on a second press"
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

/// Find the item on the Edit tab, following the overflow, and press it.
fn press(session: &Session, driver: &Driver, ui_rect: &str) -> Result<bool> {
    let Some(item) = driving::declared_or_in_overflow(session, driver, ui_rect, ITEM)? else {
        return Ok(false);
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(30);
    Ok(true)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the committed fixture is not at {}: a broken checkout.",
            pdf.display()
        )));
    }
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the ribbon.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("repair_form_fonts.trace.txt"));
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
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit mode, Edit tab --------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    let trace = session.trace()?;
    let tab = driving::declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{TAB}` region in Edit mode. Tabs declared: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    // --- 2: the first press moves the inline font --------------------------
    if !press(&session, &driver, ui_rect)? {
        return Ok(Some(format!(
            "no `{ITEM}` on the Edit tab or in its overflow: Repair fonts is missing. Items \
             declared: {}. Trace: {}.",
            driving::list(&driving::declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.edit."
            )),
            session.trace_path().display()
        )));
    }
    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        let why = trace
            .events(REFUSED_EVENT)
            .last()
            .map_or_else(|| "none".to_owned(), |l| l.raw.clone());
        return Ok(Some(format!(
            "Repair fonts was pressed on a form whose font is inline and no `{APPLIED_EVENT}` \
             line followed. Refusal: {why}. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("first press: `{}`", applied.raw));
    if applied.get("fonts") != Some("1") {
        return Ok(Some(format!(
            "the fixture holds exactly one inline font and the engine moved another count: `{}`.",
            applied.raw
        )));
    }

    // --- 3: the second press finds nothing left ----------------------------
    let applied_before = trace.events(APPLIED_EVENT).count();
    if !press(&session, &driver, ui_rect)? {
        return Err(Error::new(format!(
            "`{ITEM}` was pressed once and could not be found for the second press. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let trace = session.trace()?;
    if trace.events(APPLIED_EVENT).count() != applied_before {
        return Ok(Some(format!(
            "a second press moved fonts again, so the first did not leave them referenced. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some(refused) = trace.events(REFUSED_EVENT).last() else {
        return Ok(Some(format!(
            "a second press wrote no `{REFUSED_EVENT}` line: nothing tells the operator the \
             form is already sound. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("second press: `{}`", refused.raw));
    if !refused.raw.contains(NOTHING_DETAIL) {
        return Ok(Some(format!(
            "the second press was refused for another reason than an empty repair: `{}`.",
            refused.raw
        )));
    }
    report.note("the inline font was moved once, through the engine, and not again");
    Ok(None)
}
