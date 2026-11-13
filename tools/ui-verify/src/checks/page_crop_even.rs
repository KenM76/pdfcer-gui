//! `crop_even_pages_crops_only_even_pages` — Pages ▸ Crop… with Even pages
//! chosen crops pages 2 and 4 of four, each against its own sheet, as one step.
//!
//! Driven off the desktop with the scripted pointer: `pages.crop` rung on
//! `four-pages-unrotated.pdf`, 10 typed into the left margin, the Even radio
//! pressed, Crop pressed. The commit must name two pages under scope `even`
//! with both set, and the engine must report two changed pages. The defect it
//! catches is a scope that names pages the commit ignores: the old window
//! cropped the picked pages whatever was chosen.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "four-pages-unrotated.pdf";
const METHOD: &str = "Rebuild it from its PROVENANCE file in `fixtures/`.";
const UI_RECT: &str = "ui-rect";
const OPENED: &str = "page-crop-opened";
const COMMIT: &str = "page-crop-commit";
const APPLIED: &str = "page-crop-applied";
const LEFT: &str = "page-crop.margin.left";
const EVEN: &str = "page-crop.scope.even";
const APPLY: &str = "page-crop.apply";

/// See the module documentation.
pub struct CropEvenPages;

impl Check for CropEvenPages {
    fn name(&self) -> &'static str {
        "crop_even_pages_crops_only_even_pages"
    }

    fn defect(&self) -> &'static str {
        "Pages > Crop with Even pages chosen crops some other set of pages, or none"
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("page_crop_even.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "pages.crop"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("page_crop_even.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Replace the crop window's left margin text with 10.
pub(crate) fn type_left(session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    let (rect, viewport) = declared_in(&session.trace()?, UI_RECT, LEFT)
        .ok_or_else(|| Error::new(format!("no `{LEFT}` region in the crop window.")))?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(10);
    pointer.key(session, viewport.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(session, viewport.as_deref(), "10")?;
    session.settle(10);
    Ok(())
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    if session.trace()?.last(OPENED).is_none() {
        return Err(Error::new(format!(
            "`pages.crop` was rung and no `{OPENED}` line followed."
        )));
    }
    type_left(&session, &pointer)?;
    press_region(&session, &pointer, EVEN, "page-crop.scope.")?;
    press_region(&session, &pointer, APPLY, "page-crop.")?;
    session.settle(120);
    let trace = session.trace()?;
    let Some(commit) = trace.last(COMMIT) else {
        return Ok(Some(format!(
            "Crop was pressed and no `{COMMIT}` line followed."
        )));
    };
    report.note(commit.raw.clone());
    let got = (commit.get("n"), commit.get("scope"), commit.get("set"));
    if got != (Some("2"), Some("even"), Some("2")) {
        return Ok(Some(format!(
            "★★★ Even pages of four, left margin 10 mm, should commit n=2 scope=even set=2: `{}`.",
            commit.raw
        )));
    }
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "the commit was raised and no `{APPLIED}` line followed."
        )));
    };
    report.note(applied.raw.clone());
    pointer.gone(&session)?;
    if applied.get("n") != Some("2") {
        return Ok(Some(format!(
            "★★★ the engine changed a different number of pages than the two committed: `{}`.",
            applied.raw
        )));
    }
    Ok(None)
}
