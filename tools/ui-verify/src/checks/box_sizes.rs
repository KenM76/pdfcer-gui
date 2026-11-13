//! `document_properties_list_page_box_sizes` — Document properties ▸ Page box
//! sizes lists every page's five boxes, grouping pages that agree.
//!
//! Driven off the desktop with the scripted pointer on `page-box-sizes.pdf`
//! (see its PROVENANCE file). The control: the section starts closed and no
//! `box-sizes` line is traced until its header is pressed. Then three groups
//! are owed — pages 1–2 with a written bleed and trim, page 3 with none
//! (not merged with them), page 4 with a written art box on a wider sheet —
//! each box marked written or default. Pressing `in` re-traces with
//! `unit=in`.

use crate::checks::driving::{SHELL_DIAG_ENV, list, live_names, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "page-box-sizes.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/page-box-sizes.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const EVENT: &str = "box-sizes"; // ui-text-exempt: a trace event name, never displayed
/// `panels::docprops::box_sizes::HEADER`.
const HEADER: &str = "docprops.box-sizes.header";
/// `panels::docprops::box_sizes::UNIT_PREFIX` + `in`.
const INCH: &str = "docprops.box-sizes.unit.in";
const FAMILY: &str = "docprops.box-sizes";

/// What the open section owes, in points.
const OWED: [(&str, &str); 12] = [
    ("unit", "mm"),
    ("groups", "3"),
    ("g1", "1-2"),
    ("g1_bleed", "580x780:written"),
    ("g1_trim", "560x760:written"),
    ("g1_crop", "600x800:default"),
    ("g1_art", "600x800:default"),
    ("g2", "3"),
    ("g2_trim", "600x800:default"),
    ("g3", "4"),
    ("g3_media", "1000x700:written"),
    ("g3_art", "800x500:written"),
];

/// See the module documentation.
pub struct DocumentPropertiesListPageBoxSizes;

impl Check for DocumentPropertiesListPageBoxSizes {
    fn name(&self) -> &'static str {
        "document_properties_list_page_box_sizes"
    }

    fn defect(&self) -> &'static str {
        "Document properties does not list each page's media, crop, bleed, trim and art box \
         sizes, or merges pages whose boxes differ"
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("box_sizes.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", "file.document_properties"),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("box_sizes.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer) = launch(ctx, report)?;
    let before = session.trace()?;
    if !live_names(&before, UI_RECT, FAMILY)
        .iter()
        .any(|n| n == HEADER)
    {
        return Ok(Some(format!(
            "Document properties is open and draws no `{HEADER}`: the section cannot be \
             reached. On screen: {}.",
            list(&live_names(&before, UI_RECT, "docprops."))
        )));
    }
    if before.last(EVENT).is_some() {
        return Err(Error::new(format!(
            "`{EVENT}` was traced before the header was pressed, so its appearance after the \
             press would prove nothing."
        )));
    }
    let mark = before.mark();
    press_region(&session, &pointer, HEADER, FAMILY)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(EVENT, mark) else {
        return Ok(Some(format!(
            "the header was pressed and no `{EVENT}` line followed: the section did not open."
        )));
    };
    report.note(line.raw.clone());
    if let Some((k, v)) = OWED.iter().find(|(k, v)| line.get(k) != Some(*v)) {
        return Ok(Some(format!(
            "{FIXTURE} owes `{k}={v}`; the section traced `{}`.",
            line.raw
        )));
    }
    let mark = trace.mark();
    press_region(&session, &pointer, INCH, FAMILY)?;
    session.settle(20);
    let trace = session.trace()?;
    pointer.gone(&session)?;
    match trace.last_after(EVENT, mark) {
        Some(l) if l.get("unit") == Some("in") && l.get("g1_trim") == Some("560x760:written") => {
            report.note(l.raw.clone());
            Ok(None)
        }
        Some(l) => Ok(Some(format!(
            "`in` was pressed; the section traced `{}`, not unit=in with the same boxes.",
            l.raw
        ))),
        None => Ok(Some(format!(
            "`in` was pressed and no `{EVENT}` line followed."
        ))),
    }
}
