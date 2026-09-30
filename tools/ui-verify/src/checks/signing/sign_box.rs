//! `checks::signing::sign_box` — **a click on an empty signature box opens
//! the Sign window pointed at that box** (O266, as Acrobat)
//!
//! Drives the canvas off the desktop through the scripted pointer (no OS mouse
//! or keyboard), on the engine corpus's document with one pre-placed, empty
//! `/FT /Sig` field. A build without the `signing` feature draws the tag but
//! opens nothing, and the check fails with that reason.
//!
//! Oracles: the `form.sign-box` region (the tag was drawn), the
//! `sign-box-click` trace line (the click reached the box), and
//! `sign-field-chosen found=1` (the window opened with that field chosen).

use super::reaching::engine_fixture;
use crate::checks::driving::{SHELL_DIAG_ENV, declared_in};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const REGION_BOX: &str = "form.sign-box";
const FIELD_DOC: &str = "signing/sig-field-empty.pdf";

/// See the module documentation.
pub struct ClickingASignatureBoxOpensSign;

impl Check for ClickingASignatureBoxOpensSign {
    fn name(&self) -> &'static str {
        "clicking_a_signature_box_opens_sign"
    }

    fn defect(&self) -> &'static str {
        "an empty signature box on the page carries no tag, or a click on it does nothing, or \
         the Sign window opens without that box chosen"
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
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let source = engine_fixture(FIELD_DOC, "the document with one empty /FT /Sig field")?;
    // Driven on a copy: the source fixture belongs to the engine repository.
    let doc = ctx.out("sign-box-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIELD_DOC}: {e}")))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("sign-box.trace.txt"));
    spec.pdf = Some(doc);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("sign-box.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let trace = session.trace()?;
    let Some((rect, viewport)) = declared_in(&trace, ui_rect, REGION_BOX) else {
        return Ok(Some(format!(
            "no `{REGION_BOX}` region: the empty signature box was not tagged on the page."
        )));
    };
    pointer.hover(&session, WindowPoint::centre_of(rect))?;
    session.settle(10);
    let hover = ctx.out("sign-box-hover.png");
    if pointer.screenshot(&session, &hover).is_ok() {
        report.artifact(hover);
    }
    pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    let shot = ctx.out("sign-box-after.png");
    if pointer.screenshot(&session, &shot).is_ok() {
        report.artifact(shot);
    }
    pointer.gone(&session)?;

    let trace = session.trace()?;
    let clicked = trace.events("sign-box-click").last().is_some();
    let opened = trace.events("sign-opened").last().is_some();
    let chosen = trace
        .events("sign-field-chosen")
        .last()
        .and_then(|l| l.get("found").map(str::to_owned));
    drop(session);
    report.note(format!(
        "clicked={clicked}; sign window opened={opened}; field chosen={chosen:?}"
    ));

    let mut findings = Vec::new();
    if !clicked {
        findings.push("the click on the tagged box raised no `sign-box-click`.".to_owned());
    }
    if !opened {
        findings.push(
            "no Sign window opened (a build without the `signing` feature cannot sign).".to_owned(),
        );
    }
    if chosen.as_deref() != Some("1") {
        findings.push(format!(
            "the Sign window reported found={chosen:?}; the clicked box should be chosen."
        ));
    }
    Ok((!findings.is_empty()).then(|| findings.join("\n")))
}
