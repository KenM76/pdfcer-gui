//! `render_diagnostics_opens_its_report` — the regression test for **the
//! inert control whose data was already there.**
//!
//! # The defect class this exists for
//!
//! `shell::commands::reach` called `tools.render_diagnostics` the least
//! defensible entry on its allow-list, and the reason was not that the feature
//! was hard:
//!
//! > NO RECORDED REASON for the missing arm, and the data already exists. […]
//! > That is an argument for moving a readout that is already being computed,
//! > which makes the inert control the least defensible kind — the work behind
//! > it is done.
//!
//! The renderer has produced `pdfcer_render::Diagnostics` since S0 and the
//! status bar has shown a one-line summary of it since S2. What was missing was
//! a `match` arm and a window. Both landed on 2026-08-15
//! (`crate::dialogs::diagnostics`), and this is the check that says so from
//! outside the process.
//!
//! # ★ Why a unit test cannot cover it
//!
//! [`crate::checks`]' rule: *"it must fail against a build where the wiring is
//! absent, and the wiring must be something no unit test in the workspace can
//! observe."*
//!
//! `DialogsState`'s guards are unit-tested — no document means no dialog, a
//! second press does not rebuild the first. `text::diagnostics` is unit-tested
//! down to its units. **Every one of those passes against a build with no
//! dispatch arm**, because a dialog nothing opens is still a dialog that
//! refuses correctly. The join — a ribbon token reaching
//! `DialogsState::open_diagnostics`, and the resulting window being *drawn* —
//! is a property of two call sites, and `measure_linear`'s recorded incident is
//! what a missing call site looks like: four passing unit tests and a control
//! that did nothing visible.
//!
//! # The oracle is a region the dialog declares while drawing itself
//!
//! `dialogs::diagnostics` publishes `dialog:render-diagnostics` from inside its
//! body closure, so the region exists **only on frames where egui actually laid
//! the window out**. That is a stronger statement than "the state field became
//! `Some`", and it is stronger than a pixel diff in one specific way that
//! matters here: a window whose body panicked, whose scroll area collapsed to
//! nothing, or which was laid out entirely off-screen would still change
//! pixels somewhere. This asserts that the surface has a real rect.
//!
//! The rect is then required to be **substantial**, for the reason
//! `panels::mod`'s header records: three panels in the old shell shipped with a
//! body, a rail entry and no control anyone could click, and passed every
//! verification for their whole shipped life.
//!
//! # What it deliberately does not assert
//!
//! **What the report says.** The findings, the duration and the raster line are
//! read from a live texture, so their *content* depends on the fixture and on
//! how far the render had got — asserting on them here would make this check a
//! test of `a1-titleblock.pdf` rather than of the wiring. The editorial rules
//! behind the list are pinned where they live
//! (`app::status::notes::findings`'s tests) and the wording and units in
//! `text::diagnostics`'s.
//!
//! **That pressing it twice does nothing.** That is `DialogsState`'s
//! already-open guard and it has a unit test with a pointer-identity assertion,
//! which is a stronger check than anything a window can offer.

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, SHELL_TRACE_PREFIX, TAB_EVENT,
    UNIMPLEMENTED_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The command this check is about.
const SUBJECT_ID: &str = "tools.render_diagnostics";

/// The region the ribbon publishes for its control.
const SUBJECT: &str = "ribbon.item.tools.render_diagnostics";

/// The tab it lives on, and the region that activates it.
const TAB_ID: &str = "tools";
const TAB: &str = "ribbon.tab.tools";

/// The region the dialog publishes while it is drawing its own body.
///
/// `dialogs::diagnostics::REGION_BODY`. Spelled here as a literal because that
/// is the contract between the two crates; the application's own constant
/// carries a comment saying that renaming it un-aims whatever check was
/// measuring it, and this is that check.
const DIALOG: &str = "dialog:render-diagnostics";

pub struct RenderDiagnosticsOpensItsReport;

impl Check for RenderDiagnosticsOpensItsReport {
    fn name(&self) -> &'static str {
        "render_diagnostics_opens_its_report"
    }

    fn defect(&self) -> &'static str {
        "Tools ▸ Render diagnostics is drawn, enabled, and opens nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // A document is a precondition and not a convenience: the command is
    // registered `enabled_when("doc.open")`, so with nothing open the control is
    // greyed and a click on it proves nothing about the arm behind it.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. `tools.render_diagnostics` is gated on `doc.open`, so with nothing open \
             the control is greyed and a click on it would be measuring the enable predicate \
             rather than the dispatch arm.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is two clicks on ribbon controls. \
             Reported as SKIPPED rather than passed — a check that did not run has learned \
             nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("render_diagnostics.trace.txt"));
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
    // ★ Long, and the length is load-bearing rather than cautious. The dialog
    // draws one sentence — "this page has not been drawn yet" — until a raster
    // exists, and that branch would satisfy every assertion below while telling
    // the operator nothing. Waiting for the first page of a dense CAD sheet is
    // what makes the window this check photographs the real report.
    session.settle(80);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and nothing below could be observed. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    if declared(
        &trace,
        ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect"),
        DIALOG,
    )
    .is_some()
    {
        return Ok(Some(format!(
            "`{DIALOG}` was declared before anything was clicked. A diagnostic window that \
             opens on its own is the specified default of `view.app_initiative` — **Never** — \
             broken in the most visible way available: pdfcer may not float a surface over the \
             canvas unasked."
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- A. Edit mode, then the Tools tab ----------------------------------
    //
    // ★ The mode click is a precondition rather than a flourish. pdfcer opens in
    // **Read**, whose tab list is `["file", "view"]` — so Tools does not exist
    // in the mode this process starts in, and a check that went straight for
    // the tab would SKIP with *"the tab strip is too narrow"*, which is a
    // confident wrong diagnosis of a build that is fine.
    //
    // That is `shell::manifest::tools`' own recorded trap, from the other side:
    // OCR was specified onto this tab and had to move to File, because *"a
    // command on the Tools tab is therefore not merely inconvenient in Read, it
    // is unreachable"*. The same sentence decides which mode this check has to
    // be in.
    driving::click_mode_segment(&session, &driver, ui_rect, "edit")?;

    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Either this build does not show that \
             tab in the mode it opened in, or the tab strip is too narrow and it has moved into \
             the overflow menu — which this check cannot open, because a menu's contents are \
             not published as regions. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so no click \
             reached the ribbon and the control below was never on screen."
        )));
    }

    // --- B. the control ----------------------------------------------------
    let trace = session.trace()?;
    let Some(control) = declared(&trace, ui_rect, SUBJECT) else {
        return Ok(Some(format!(
            "the Tools tab is active and its controls publish their rects, but none of them is \
             `{SUBJECT}`. Tools has three groups and the smallest band in the ribbon, so this is \
             not an overflow — it is a registered command with no reachable control. Controls \
             declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    if !control.is_substantial() {
        return Ok(Some(format!(
            "`{SUBJECT}` was declared at {control:?}, which has no usable area — the control is \
             laid out and not on screen."
        )));
    }

    let invokes_before = shell_trace(&session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(SUBJECT_ID))
        .count();
    driver.click_at(session.frame()?.declared_center(control))?;
    session.settle(24);

    let invokes_after = shell_trace(&session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(SUBJECT_ID))
        .count();
    if invokes_after <= invokes_before {
        let shell = shell_trace(&session)?;
        return Err(Error::new(format!(
            "the click on `{SUBJECT}` produced no new `{INVOKE_EVENT} id={SUBJECT_ID}` line, so \
             no click reached the ribbon and nothing after it would mean anything. Two readings, \
             and this check declines to choose between them: the pointer injection is not \
             reaching this window, or the shell diagnostic switch {}={} did not reach the \
             process — the shell trace carries {} line(s) under `{SHELL_TRACE_PREFIX}`. \
             Trace: {}.",
            SHELL_DIAG_ENV.0,
            SHELL_DIAG_ENV.1,
            shell.lines.len(),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "the shell traced `{INVOKE_EVENT} id={SUBJECT_ID}`, so the click reached the control"
    ));

    // --- C. is there a window? ---------------------------------------------
    let trace = session.trace()?;
    let Some(body) = declared(&trace, ui_rect, DIALOG) else {
        let unimplemented = trace
            .events(UNIMPLEMENTED_EVENT)
            .any(|l| l.get("id") == Some(SUBJECT_ID));
        return Ok(Some(format!(
            "`{SUBJECT_ID}` was invoked and no `{DIALOG}` region was ever declared, so no \
             window was laid out. {}",
            if unimplemented {
                format!(
                    "The application traced `{UNIMPLEMENTED_EVENT} id={SUBJECT_ID}`, so the \
                     token arrived at `app::dispatch` and there is no arm for it — which is \
                     exactly the state this command shipped in until 2026-08-15, and the fix \
                     is one match arm calling `DialogsState::open_diagnostics`."
                )
            } else {
                "The application traced no `command-unimplemented` for it either, so the token \
                 reached an arm that built the dialog and something stopped it being drawn — \
                 look at `DialogsState::show`, where a document-scoped dialog added later is \
                 the one that gets forgotten."
                    .to_owned()
            }
        )));
    };
    if !body.is_substantial() {
        return Ok(Some(format!(
            "`{DIALOG}` was declared at {body:?}, which has no usable area. The window exists \
             and has collapsed — check the scroll area's `max_height` floor, which
             `dialogs::about` records as the trap here: a negative height is not an error, it \
             is a scroll area that silently draws nothing."
        )));
    }
    report.note(format!(
        "the report was laid out at {:.0} x {:.0} pt",
        body.width(),
        body.height()
    ));

    let png = ctx.out("render_diagnostics.png");
    crate::capture::window_to_png(&session, &png)?;
    report.artifact(png);
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The region names are derived from the ids they describe.
    ///
    /// The seam where a rename in `egui_shell::ribbon::report` would otherwise
    /// turn every assertion above into a silent SKIP — *"the application
    /// declared no region"* — which reads as a missing control rather than as a
    /// renamed one.
    #[test]
    fn the_region_names_match_the_ids_they_describe() {
        assert_eq!(SUBJECT, format!("{ITEM_PREFIX}{SUBJECT_ID}"));
        assert_eq!(TAB, format!("ribbon.tab.{TAB_ID}"));
    }

    /// The dialog's region name is not in the ribbon's namespaces.
    ///
    /// It is declared by the application rather than by `egui-shell`, so a
    /// `declared_names` sweep over `ribbon.` must not catch it and a sweep for
    /// it must not catch a ribbon item. Cheap to assert, and the alternative is
    /// a filter that quietly returns the wrong rect.
    #[test]
    fn the_dialogs_region_is_in_its_own_namespace() {
        assert!(!DIALOG.starts_with("ribbon."));
        assert!(!DIALOG.starts_with(ITEM_PREFIX));
    }
}
