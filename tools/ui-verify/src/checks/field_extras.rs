//! `a_text_fields_extras_reach_the_file` — the Properties panel's Scroll long
//! text, Check spelling and Sends a file checkboxes and its Export name box each
//! change the selected text field in the document, read back from it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_extras.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1200,1350";
const INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: &str = "three-text-fields.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/three-text-fields.PROVENANCE.py`.";
const FIELD: &str = "FieldOne";
const SCROLL: &str = "properties.field_edit.scroll";
const SPELL: &str = "properties.field_edit.spell_check";
const FILE_SELECT: &str = "properties.field_edit.file_select";
const FILE_NOTE: &str = "properties.field_edit.file_select.note";
const EXPORT: &str = "properties.field_edit.export_name";
const READ: &str = "field-extras-read";
const TYPED: &str = "qty_total";

/// See the module documentation.
pub struct ATextFieldsExtrasReachTheFile;

impl Check for ATextFieldsExtrasReachTheFile {
    fn name(&self) -> &'static str {
        "a_text_fields_extras_reach_the_file"
    }

    fn defect(&self) -> &'static str {
        "a text field's Properties panel offers no scroll, spell-check, file-select or export-name \
         control, or a change to one never reaches edit_field and is not read back from the file"
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
    let doc = ctx.out("field-extras.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("field_extras.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("field_extras.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

/// The newest `field-extras-read` line for the field must carry `want`.
fn reads(
    session: &Session,
    report: &mut CheckReport,
    after: &str,
    want: &[(&str, &str)],
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last(READ).filter(|l| l.get("field") == Some(FIELD)) else {
        return Ok(Some(format!(
            "{after}: no `{READ} field={FIELD}` line, so the section never drew for the field. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("{after}: {}", line.raw));
    let missing: Vec<String> = want
        .iter()
        .filter(|(k, v)| line.get(k) != Some(*v))
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    Ok((!missing.is_empty())
        .then(|| format!("★★★ {after}, `{}` lacks {}.", line.raw, missing.join(" "))))
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let start = [
        ("scroll", "1"),
        ("spell", "1"),
        ("file", "0"),
        ("export", "-"),
    ];
    if let Some(failure) = reads(session, report, "before any press", &start)? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, SCROLL, 30)?;
    if let Some(failure) = reads(
        session,
        report,
        "after Scroll long text",
        &[("scroll", "0")],
    )? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, SPELL, 30)?;
    if let Some(failure) = reads(session, report, "after Check spelling", &[("spell", "0")])? {
        return Ok(Some(failure));
    }
    let vp = properties_pane::press(ctx, session, pointer, EXPORT, 20)?;
    pointer.type_text(session, vp.as_deref(), TYPED)?;
    pointer.key(session, vp.as_deref(), "Enter", None)?;
    session.settle(30);
    if let Some(failure) = reads(
        session,
        report,
        "after typing an export name",
        &[("export", TYPED)],
    )? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, FILE_SELECT, 30)?;
    let after = [
        ("file", "1"),
        ("scroll", "0"),
        ("spell", "0"),
        ("export", TYPED),
    ];
    if let Some(failure) = reads(session, report, "after Sends a file", &after)? {
        return Ok(Some(failure));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    Ok(declared(&session.trace()?, ui_rect, FILE_NOTE)
        .is_none()
        .then(|| {
            format!(
                "★★★ the field now sends a file, but no `{FILE_NOTE}` sentence says what that \
             changes. Trace: {}.",
                session.trace_path().display()
            )
        }))
}
