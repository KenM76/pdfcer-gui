//! `a_field_is_calculated_from_others` — the Properties panel's
//! Calculate tab makes a text field the sum of two others, and its Validate
//! tab gives it a range, each written as the engine's helper script and read
//! back as one.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_scripts.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1200,1350";
const INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: &str = "three-text-fields.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/three-text-fields.PROVENANCE.py`.";
const FIELD: &str = "FieldThree";
const PANEL_BODY: &str = "dock.body.file.properties";
const TAB_CALCULATE: &str = "properties.field_scripts.tab.calculate";
const TAB_VALIDATE: &str = "properties.field_scripts.tab.validate";
const CALCULATED: &str = "properties.field_scripts.calculated";
const OPERANDS: [&str; 2] = [
    "properties.field_scripts.operand.FieldOne",
    "properties.field_scripts.operand.FieldTwo",
];
const LOWEST: &str = "properties.field_scripts.lowest";
const LOWEST_VALUE: &str = "properties.field_scripts.lowest.value";
const APPLY: &str = "properties.field_scripts.apply";
const READ: &str = "field-scripts-read";
const SET: &str = "field-script-set";
const MAX_SCROLL: usize = 12;

/// See the module documentation.
pub struct AFieldIsCalculatedFromOthers;

impl Check for AFieldIsCalculatedFromOthers {
    fn name(&self) -> &'static str {
        "a_field_is_calculated_from_others"
    }

    fn defect(&self) -> &'static str {
        "a text field's Properties panel offers no calculation or range, the choices never \
         reach set_field_calculation / set_field_validation, or what is written is not read \
         back as the helper it was meant to be"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = steps(ctx, &mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    // Driven on a copy, so a stray save never touches the repository's fixture.
    let doc = ctx.out("field-scripts.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("field_scripts.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
        ("PDFCER_DIAG_SELECT_FIELD", FIELD),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("field_scripts.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

/// Scroll the Properties panel until `region` is declared, then click it;
/// returns the viewport it was declared in.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    for _ in 0..=MAX_SCROLL {
        let trace = session.trace()?;
        if let Some((rect, vp)) = declared_in(&trace, ui_rect, region) {
            pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
            session.settle(20);
            return Ok(vp);
        }
        let (panel, vp) = declared_in(&trace, ui_rect, PANEL_BODY).ok_or_else(|| {
            Error::new(format!(
                "no `{PANEL_BODY}` region, so the Properties panel is not open."
            ))
        })?;
        pointer.wheel_in(session, vp.as_deref(), WindowPoint::centre_of(panel), -3.0)?;
        session.settle(10);
    }
    Err(Error::new(format!(
        "no visible `{region}` after {MAX_SCROLL} wheel turns. Trace: {}",
        session.trace_path().display()
    )))
}

/// The newest `field-scripts-read` line's value for `trigger`.
fn held(session: &Session, trigger: &str) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .last(READ)
        .filter(|l| l.get("field") == Some(FIELD))
        .and_then(|l| l.get(trigger).map(str::to_owned)))
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if held(session, "calculate")?.as_deref() != Some("none") {
        return Ok(Some(format!(
            "with {FIELD} selected the panel traced no `{READ} field={FIELD} … calculate=none` \
             line, so the scripts section never drew for it."
        )));
    }
    press(ctx, session, pointer, TAB_CALCULATE)?;
    press(ctx, session, pointer, CALCULATED)?;
    for operand in OPERANDS {
        press(ctx, session, pointer, operand)?;
    }
    press(ctx, session, pointer, APPLY)?;
    let want = [
        ("field", FIELD),
        ("trigger", "calculate"),
        ("applied", "AFSimple_Calculate"),
        ("replaced", "none"),
        ("position", "0"),
        ("entries", "1"),
        ("created", "true"),
    ];
    if let Some(failure) = set_line(session, report, &want)? {
        return Ok(Some(failure));
    }
    if held(session, "calculate")?.as_deref() != Some("helper") {
        return Ok(Some(format!(
            "★★★ after Apply the panel re-read {FIELD}'s calculation as {:?}, not `helper`: \
             what was written is not the helper the engine reads back.",
            held(session, "calculate")?
        )));
    }
    press(ctx, session, pointer, TAB_VALIDATE)?;
    press(ctx, session, pointer, LOWEST)?;
    let vp = press(ctx, session, pointer, LOWEST_VALUE)?;
    pointer.type_text(session, vp.as_deref(), "5")?;
    session.settle(10);
    press(ctx, session, pointer, APPLY)?;
    let want = [
        ("field", FIELD),
        ("trigger", "validate"),
        ("applied", "AFRange_Validate"),
    ];
    set_line(session, report, &want)
}

/// The newest `field-script-set` line must carry every pair in `want`.
fn set_line(
    session: &Session,
    report: &mut CheckReport,
    want: &[(&str, &str)],
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last(SET) else {
        return Ok(Some(format!(
            "★★★ Apply traced no `{SET}` line: the panel's choice never reached the engine. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(line.raw.clone());
    let missing: Vec<String> = want
        .iter()
        .filter(|(k, v)| line.get(k) != Some(*v))
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    Ok((!missing.is_empty()).then(|| format!("★★★ `{}` lacks {}.", line.raw, missing.join(" "))))
}
