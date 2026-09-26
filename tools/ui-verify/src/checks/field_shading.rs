//! `fillable_fields_are_shaded_on_the_page` — every box you can type into wears
//! a wash, the way Acrobat's does.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_shading.md`.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The canvas's wash trace.
const SHADE: &str = "canvas-form-shade";
/// The fixture: two widgets on one page, both visible at once.
const FIXTURE: &str = "forms/demo-form.pdf";
/// Where and how large the window is placed, as `PDFCER_DIAG_VIEWPORT` takes it.
const VIEWPORT: &str = "0,0,1400,900";

/// See the module documentation.
pub struct FillableFieldsAreShadedOnThePage;

impl Check for FillableFieldsAreShadedOnThePage {
    fn name(&self) -> &'static str {
        "fillable_fields_are_shaded_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "the boxes an operator can type into are not marked on the page, so a form whose fields \
         have no border and no background is indistinguishable from a drawing that merely looks \
         like one"
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

    // Its own fixture, never `--pdf`. The harness's usual fixture is a CAD
    // drawing with no `/AcroForm`, which would trace `on=1 boxes=0` — a state
    // this check correctly treats as "nothing to say", so falling back would
    // make it SKIP forever, and a SKIP is not red.
    let fixture = form_fixture().ok_or_else(|| {
        Error::new(format!(
            "the engine fixture `{FIXTURE}` is not on disk, so there is no document with form \
             fields to shade. This check does NOT fall back to `--pdf`: the usual fixture has \
             no form, and a run against it could not distinguish a working wash from a dead \
             one."
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("field_shading.trace.txt"));
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
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} — no input is sent",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(48);

    let trace = session.trace()?;
    let Some(line) = trace.last(SHADE) else {
        return Ok(Some(format!(
            "the document opened and the canvas traced no `{SHADE}` line at all, so the wash \
             code never ran. It is called from the form overlay, so either that overlay is not \
             drawn on this document or the call was removed."
        )));
    };
    report.note(format!("canvas: `{}`", line.raw));

    // --- 1: the option is on by default ------------------------------------
    if line.get("on") != Some("1") {
        return Ok(Some(format!(
            "the wash reports `on=0`, so the preference is off in a fresh profile. It is meant \
             to default to ON — an operator who has never opened the settings should see which \
             boxes are fillable. Check `app::prefs`' default, and check that it survives into \
             `OpenDoc::prefs`, which is a SNAPSHOT taken when the document opened rather than \
             a live read. Line: `{}`.",
            line.raw
        )));
    }

    // --- 2: the fixture really does carry fields ---------------------------
    //
    // Asserted rather than assumed, and it guards the check itself rather
    // than the feature: if `demo-form.pdf` ever loses its widgets, every
    // assertion below becomes vacuous and this run would report PASS while
    // having tested nothing.
    let boxes: usize = line.get("boxes").and_then(|v| v.parse().ok()).unwrap_or(0);
    if boxes == 0 {
        return Err(Error::new(format!(
            "the canvas found no form-field boxes in `{FIXTURE}`, so there is nothing to shade \
             and this check cannot distinguish a working wash from a dead one. Reported as SKIP \
             rather than PASS or FAIL: the fixture no longer exercises the case it was chosen \
             for. Line: `{}`.",
            line.raw
        )));
    }

    // --- 3: and they were actually painted -------------------------------
    let drawn: usize = line.get("drawn").and_then(|v| v.parse().ok()).unwrap_or(0);
    if drawn == 0 {
        return Ok(Some(format!(
            "the wash is on and the canvas knows about {boxes} field box(es) and painted NONE \
             of them. This is the one shape that is a defect rather than an absence: either \
             the page-view walk and the box census disagree about which page a box is on, or \
             the boxes are all off screen — and this fixture is a single page opened at fit, \
             so they cannot be. Line: `{}`.",
            line.raw
        )));
    }
    report.note(format!(
        "the wash painted {drawn} of {boxes} field box(es) on the visible page"
    ));

    Ok(None)
}

/// Resolve [`FIXTURE`] under the engine repository's synthetic corpus.
///
/// Read-only, as everything under `D:\Dev\pdfcer` is until fold-in day. The
/// harness opens it and never saves.
fn form_fixture() -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(FIXTURE);
    path.is_file().then_some(path)
}
